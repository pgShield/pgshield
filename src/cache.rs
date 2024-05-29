// cache.rs
use crate::logger::log_info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct ConnCache {
    cache: Arc<Mutex<HashMap<String, (std::net::TcpStream, Instant)>>>,
    ttl: Duration,
}

impl ConnCache {
    pub fn new(ttl: Duration) -> Self {
        ConnCache {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<std::net::TcpStream> {
        let mut cache = self.cache.lock().unwrap();
        if let Some((conn, last_used)) = cache.get(key) {
            if last_used.elapsed() < self.ttl {
                return Some(conn.try_clone().unwrap());
            }
        }
        None
    }

    pub fn set(&self, key: String, conn: std::net::TcpStream) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key.clone(), (conn, Instant::now()));
        log_info(&format!("Cached connection for key {}", key));
    }

    pub fn cleanup(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.retain(|key, (_conn, last_used)| {
            let retain = last_used.elapsed() < self.ttl;
            if !retain {
                log_info(&format!("Cleaned up cached connection for key {}", key));
            }
            retain
        });
    }
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct QueryCache {
    cache: Arc<Mutex<HashMap<u64, (String, Instant)>>>,
    ttl: Duration,
}

impl QueryCache {
    pub fn new(ttl: Duration) -> Self {
        QueryCache {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    pub fn get(&self, query: &str) -> Option<String> {
        let mut cache = self.cache.lock().unwrap();
        let key = self.hash(query);
        if let Some((result, last_used)) = cache.get(&key) {
            if last_used.elapsed() < self.ttl {
                return Some(result.clone());
            }
        }
        None
    }

    pub fn set(&self, query: &str, result: String) {
        let mut cache = self.cache.lock().unwrap();
        let key = self.hash(query);
        cache.insert(key, (result, Instant::now()));
        log_info(&format!("Cached query result for key {}", key));
    }

    fn hash<T: Hash>(&self, t: T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}
