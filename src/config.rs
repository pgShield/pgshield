// config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub db_hosts: Vec<String>,
    pub listen_port: String,
    pub max_conns: usize,
    pub cache_ttl: u64,
    pub health_check_interval: u64,
    pub replication_mode: bool,
    pub query_cache_ttl: u64,
    pub database_discovery: bool,
    pub discovery_interval: u64,
}

impl Config {
    pub fn from_file(filename: &str) -> Result<Self, Box<dyn Error>> {
        let file = fs::File::open(filename)?;
        let config: Config = serde_json::from_reader(file)?;
        Ok(config)
    }
}
