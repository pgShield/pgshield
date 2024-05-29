use crate::cache::{ConnCache, QueryCache};
use crate::db::DB;
use crate::logger::{log_error, log_info};
use postgres::{Client, NoTls, Row};
use std::collections::HashMap;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct QueryResult {
    rows: Vec<HashMap<String, String>>,
}

fn rows_to_query_result(rows: Vec<Row>) -> QueryResult {
    let mut result = Vec::new();
    for row in rows {
        let mut map = HashMap::new();
        for column in row.columns() {
            let value: String = match row.try_get(column.name()) {
                Ok(value) => value,
                Err(_) => "NULL".to_string(),
            };
            map.insert(column.name().to_string(), value);
        }
        result.push(map);
    }
    QueryResult { rows: result }
}

pub struct Proxy {
    db: Arc<DB>,
    conn_cache: Arc<ConnCache>,
    query_cache: Arc<QueryCache>,
    replication_mode: bool,
}

impl Proxy {
    pub fn new(
        db: Arc<DB>,
        conn_cache: Arc<ConnCache>,
        query_cache: Arc<QueryCache>,
        replication_mode: bool,
    ) -> Self {
        Proxy {
            db,
            conn_cache,
            query_cache,
            replication_mode,
        }
    }

    pub async fn listen_and_serve(&self, listen_port: &str) {
        let addr = format!("0.0.0.0:{}", listen_port);
        let listener = TcpListener::bind(&addr).await.unwrap();
        log_info(&format!("Proxy listening on {}", addr));

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    let db = Arc::clone(&self.db);
                    let conn_cache = Arc::clone(&self.conn_cache);
                    let query_cache = Arc::clone(&self.query_cache);
                    let replication_mode = self.replication_mode;

                    tokio::spawn(async move {
                        let mut proxy = ProxyConnection::new(socket, db, conn_cache, query_cache, replication_mode);
                        if let Err(e) = proxy.handle_connection().await {
                            log_error(&format!("Error handling connection from {}: {}", addr, e));
                        }
                    });
                }
                Err(e) => {
                    log_error(&format!("Failed to accept connection: {}", e));
                }
            }
        }
    }
}

struct ProxyConnection {
    socket: tokio::net::TcpStream,
    db: Arc<DB>,
    conn_cache: Arc<ConnCache>,
    query_cache: Arc<QueryCache>,
    replication_mode: bool,
}

impl ProxyConnection {
    pub fn new(
        socket: tokio::net::TcpStream,
        db: Arc<DB>,
        conn_cache: Arc<ConnCache>,
        query_cache: Arc<QueryCache>,
        replication_mode: bool,
    ) -> Self {
        ProxyConnection {
            socket,
            db,
            conn_cache,
            query_cache,
            replication_mode,
        }
    }


    pub async fn handle_connection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        let n = self.socket.read(&mut buffer).await?;
        let initial_request = String::from_utf8_lossy(&buffer[..n]);
    
        // Extract database name from the initial request
        let database = self.extract_database_name(&initial_request);
        if database.is_empty() {
            return Err("Failed to extract database name from request".into());
        }
    
        let pool = self.db.get_pool(&database);
        if pool.is_none() {
            return Err(format!("No pool found for database {}", database).into());
        }
    
        let host = match self.db.get_healthy_host() {
            Some(h) => h,
            None => return Err("No healthy hosts available".into()),
        };

        let key = format!("{}_{}", host, database);
        let conn_str = self.conn_cache.get(&key);
        let dsn = format!("postgresql://user:password@{}/{}", host, database);
        let mut client;
        match conn_str {
            Some(_) => {
                
                client = Client::connect(&dsn, NoTls)?;
            }
            
            None => {
                client = Client::connect(&dsn, NoTls)?;
                let addr: String = dsn.split("@").nth(1).unwrap().split("/").next().unwrap().to_string();
                let stream = TcpStream::connect(&addr)?;
                self.conn_cache.set(key.clone(), stream);
            }
        }
    
        self.forward_query(&mut client).await?;
        Ok(())
    }


    async fn forward_query(&mut self, client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];

        loop {
            let n = self.socket.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            let query = String::from_utf8_lossy(&buffer[..n]).to_string();
            let cache_result = self.query_cache.get(&query);

            let response = match cache_result {
                Some(result) => result,
                None => {
                    let rows = client.query(&query, &[])?;
                    let query_result = rows_to_query_result(rows);
                    let serialized_result = serde_json::to_string(&query_result)?;
                    self.query_cache.set(&query, serialized_result.clone());
                    serialized_result
                }
            };

            self.socket.write_all(response.as_bytes()).await?;
        }

        Ok(())
    }

    fn extract_database_name(&self, initial_request: &str) -> String {
        // Example expected format: "CONNECT database_name"
        let parts: Vec<&str> = initial_request.split_whitespace().collect();
        if parts.len() == 2 && parts[0] == "CONNECT" {
            parts[1].to_string()
        } else {
            "".to_string() // Return an empty string if the format is incorrect
        }
    }
}