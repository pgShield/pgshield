use crate::config::Config;
use crate::logger::{log_error, log_info};
use postgres::{Client, NoTls};
use std::collections::HashMap;
use std::net;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct DB {
    pools: Arc<Mutex<HashMap<String, String>>>, // Store DSNs instead of Client
    hosts: Vec<String>,
    healthy_hosts: Arc<Mutex<HashMap<String, bool>>>,
    database_map: Arc<Mutex<HashMap<String, String>>>,
    health_check_interval: Duration,
    discovery_interval: Duration,
}

impl DB {
    pub fn new(config: &Config) -> Self {
        let db = DB {
            pools: Arc::new(Mutex::new(HashMap::new())),
            hosts: config.db_hosts.clone(),
            healthy_hosts: Arc::new(Mutex::new(HashMap::new())),
            database_map: Arc::new(Mutex::new(HashMap::new())),
            health_check_interval: Duration::from_secs(config.health_check_interval),
            discovery_interval: Duration::from_secs(config.discovery_interval),
        };

        db.discover_databases(config);

        let db_clone = db.clone();
        thread::spawn(move || {
            db_clone.health_check();
        });

        let db_clone = db.clone();
        let config_clone = config.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(3600)); // 1 hour
                db_clone.discover_databases(&config_clone);
            }
        });

        db
    }

    pub fn get_pool(&self, database: &str) -> Option<Client> {
        let host = self.database_map.lock().unwrap().get(database).cloned()?;
        let key = format!("{}_{}", host, database);
        let dsn = self.pools.lock().unwrap().get(&key).cloned()?;
        Client::connect(&dsn, NoTls).ok()
    }

    pub fn get_healthy_host(&self) -> Option<String> {
        for host in &self.hosts {
            if *self.healthy_hosts.lock().unwrap().get(host).unwrap_or(&false) {
                return Some(host.clone());
            }
        }
        None
    }

    fn health_check(&self) {
        loop {
            thread::sleep(self.health_check_interval);
            let mut healthy_hosts = self.healthy_hosts.lock().unwrap();
            for host in &self.hosts {
                match net::TcpStream::connect_timeout(&host.parse().unwrap(), Duration::from_secs(2)) {
                    Ok(_) => {
                        healthy_hosts.insert(host.clone(), true);
                        log_info(&format!("Host {} is up", host));
                    }
                    Err(e) => {
                        healthy_hosts.insert(host.clone(), false);
                        log_error(&format!("Host {} is down: {}", host, e));
                    }
                }
            }
        }
    }

    fn discover_databases(&self, config: &Config) {
        for host in &self.hosts {
            let dsn = format!("postgresql://user:password@{}/postgres", host);
            match Client::connect(&dsn, NoTls) {
                Ok(mut client) => {
                    if let Ok(rows) = client.query("SELECT datname FROM pg_database WHERE datistemplate = false", &[]) {
                        let mut database_map = self.database_map.lock().unwrap();
                        for row in rows {
                            let db_name: String = row.get(0);
                            database_map.insert(db_name.clone(), host.clone());

                            let pool_key = format!("{}_{}", host, db_name);
                            if !self.pools.lock().unwrap().contains_key(&pool_key) {
                                let pool_dsn = format!("postgresql://user:password@{}/{}", host, db_name);
                                self.pools.lock().unwrap().insert(pool_key, pool_dsn);
                                log_info(&format!("Connected to database {} on host {}", db_name, host));
                            }
                        }
                    }
                }
                Err(e) => {
                    log_error(&format!("Failed to connect to host {} for discovery: {}", host, e));
                }
            }
        }
    }
}
