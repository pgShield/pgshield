mod config;
mod logger;
mod db;
mod cache;
mod proxy;

use crate::config::Config;
use crate::db::DB;
use crate::logger::init_logger;
use crate::cache::{ConnCache, QueryCache};
use crate::proxy::Proxy;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn main() {
    // Initialize the logger
    init_logger();

    // Load configuration
    let config = Config::from_file("config.json").expect("Failed to load configuration");

    // Create DB, Cache, and Proxy instances
    let db = DB::new(&config);
    let conn_cache = ConnCache::new(std::time::Duration::from_secs(config.cache_ttl));
    let query_cache = QueryCache::new(std::time::Duration::from_secs(config.query_cache_ttl));
    let proxy = Proxy::new(Arc::new(db), Arc::new(conn_cache), Arc::new(query_cache), config.replication_mode);

    // Run the proxy server
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        proxy.listen_and_serve(&config.listen_port).await;
    });
}
