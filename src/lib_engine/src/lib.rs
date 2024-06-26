extern crate lib_logger;
extern crate lib_config;
extern crate lib_pool;
extern crate lib_cache;
extern crate syslog;
extern crate lib_pgsqlcli;

use syslog::Facility;
use lib_logger::{LoggerConfig, init_logger};
use lib_config::Config;
use lib_pool::Pool;
use lib_cache::Cache;
use lib_pgsqlcli::PostgresError;

use std::time::Duration;

pub struct Engine {
    config: Config,
    pool: Pool,
    cache: Cache,
}

impl Engine {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize logger
        let logger_config = LoggerConfig {
            log_to_file: false,
            log_to_console: true,
            log_to_syslog: false,
            log_dir: Some("C:\\logs".into()),
            syslog_facility: Facility::LOG_USER,
            syslog_process_name: "pgShield".into(),
            syslog_remote_addr: None,
        };

        let _ = init_logger(&logger_config)?;

        // Initialize config
        let config = Config::from_file("path/to/config/file")?;

        // Initialize pool
        let connection_string = format!(
            "postgresql://{}:{}@{}/{}",
            config.db_user, config.db_password, config.db_host, config.db_name
        );
        let pool = Pool::new(&connection_string, 100).await?;

        // Initialize cache
        let cache = Cache::new(Duration::from_secs(config.cache_ttl));

        Ok(Engine {
            config,
            pool,
            cache,
        })
    }

    pub fn start(&self) {
        log::info!("pgShield engine started");
    }
}