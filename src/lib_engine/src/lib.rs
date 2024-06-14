extern crate lib_logger;
extern crate lib_config;
extern crate lib_pool;
extern crate lib_cache;

use lib_logger::LoggerConfig;
use lib_config::Config;
use lib_pool::PoolConfig;
use lib_cache::CacheConfig;

pub struct Engine {
    logger: lib_logger::Logger,
    config: Config,
    pool: lib_pool::Pool,
    cache: lib_cache::Cache,
}

impl Engine {
    pub fn new() -> Result<Self, String> {

        
        // Initialize logger
        let logger_config = LoggerConfig {
            log_to_file: true,
            log_to_console: true,
            log_to_syslog: false,
            log_dir: Some("logs".into()),
            syslog_facility: syslog::Facility::LOG_USER,
            syslog_process_name: "pgShield".into(),
            syslog_remote_addr: None,
        };
        let logger = lib_logger::init_logger(&logger_config)?;

        // Initialize config
        let config = Config::new()?;

        // Initialize pool
        let pool_config = PoolConfig::default();
        let pool = lib_pool::Pool::new(pool_config)?;

        // Initialize cache
        let cache_config = CacheConfig::default();
        let cache = lib_cache::Cache::new(cache_config)?;

        Ok(Engine {
            logger,
            config,
            pool,
            cache,
        })
    }

    pub fn start(&self) {
        // Start the engine, including the pool and cache
        self.pool.start();
        self.cache.start();

        log::info!("pgShield engine started");
    }
}