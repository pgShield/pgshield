use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum PostgresqlAuthType {
    Trust,
    Password,
    Md5,
    Scm,
    Gss,
    Sspi,
    Ident,
    Peer,
    Ldap,
    Radius,
    Cert,
    Pam,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostgresqlHost {
    pub host: String,
    pub admin_auth_type: Option<PostgresqlAuthType>,
    pub admin_username: Option<String>,
    pub admin_password: Option<String>,
    pub database_discovery: Option<bool>,
    pub discovery_interval: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoggingConfig {
    pub log_to_file: bool,
    pub log_to_console: bool,
    pub log_to_syslog: bool,
    pub log_dir: Option<String>,
    pub syslog_facility: Option<String>,
    pub syslog_process_name: Option<String>,
    pub syslog_remote_addr: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub postgresql_hosts: Vec<PostgresqlHost>,
    pub listen_port: String,
    pub max_conns: usize,
    pub cache_ttl: u64,
    pub health_check_interval: u64,
    pub replication_mode: bool,
    pub query_cache_ttl: u64,
    pub logging: LoggingConfig,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn Error>> {
        let file_path = filename.as_ref();

        // Check if the configuration file exists
        if !file_path.exists() {
            info!("Configuration file not found, creating one with dummy data");
            let dummy_config = Config {
                postgresql_hosts: vec![PostgresqlHost {
                    host: "localhost:5432".to_string(),
                    admin_auth_type: Some(PostgresqlAuthType::Trust),
                    admin_username: None,
                    admin_password: None,
                    database_discovery: Some(true),
                    discovery_interval: Some(3600),
                }],
                listen_port: "8558".to_string(),
                max_conns: 1000,
                cache_ttl: 3600,
                health_check_interval: 60,
                replication_mode: false,
                query_cache_ttl: 600,
                logging: LoggingConfig {
                    log_to_file: true,
                    log_to_console: true,
                    log_to_syslog: false,
                    log_dir: Some("/var/log/pgShield".to_string()),
                    syslog_facility: Some("LOG_USER".to_string()),
                    syslog_process_name: Some("pgShield".to_string()),
                    syslog_remote_addr: None,
                },
            };

            let config_file = fs::File::create(file_path)?;
            serde_json::to_writer_pretty(config_file, &dummy_config)?;
            return Ok(dummy_config);
        }

        // Load the configuration file
        let file = fs::File::open(file_path)?;
        let mut config: Config = serde_json::from_reader(file)?;

        // Check for missing admin credentials for PostgreSQL hosts with specific auth types
        for host in &mut config.postgresql_hosts {
            if let Some(auth_type) = &host.admin_auth_type {
                match auth_type {
                    PostgresqlAuthType::Password
                    | PostgresqlAuthType::Md5
                    | PostgresqlAuthType::Ldap
                    | PostgresqlAuthType::Cert => {
                        if host.admin_username.is_none() || host.admin_password.is_none() {
                            host.admin_username = None;
                            host.admin_password = None;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(config)
    }
}