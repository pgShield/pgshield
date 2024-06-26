use std::sync::{Arc, Mutex};
use lib_pgsqlcli::{client::PostgresClient, error::PostgresError, config::ConnectionConfig};

pub struct Pool {
    clients: Arc<Mutex<Vec<PostgresClient>>>,
    config: ConnectionConfig,
    max_size: usize,
}

impl Pool {
    pub async fn new(connection_string: &str, max_size: usize) -> Result<Self, PostgresError> {
        let mut clients = Vec::new();
        let config = ConnectionConfig::from_connection_string(connection_string)?;

        for _ in 0..max_size {
            let client = PostgresClient::connect(connection_string).await?;
            clients.push(client);
        }

        Ok(Pool {
            clients: Arc::new(Mutex::new(clients)),
            config,
            max_size,
        })
    }

    pub async fn get_client(&self) -> Result<PostgresClient, PostgresError> {
        PostgresClient::connect(&self.config.to_string()).await // Adjusted the call to connect
    }

    pub async fn release_client(&self, client: PostgresClient) {
        let mut clients = self.clients.lock().unwrap();

        if clients.len() < self.max_size {
            clients.push(client);
        } else {
            // Log error or handle exceeding pool size gracefully (e.g., backoff)
            eprintln!("Connection pool reached max size, discarding client");
        }
    }

    pub async fn execute<F, T>(&self, f: F) -> Result<T, PostgresError>
    where
        F: FnOnce(&mut PostgresClient) -> Result<T, PostgresError>,
    {
        let mut client = self.get_client().await?;
        let result = f(&mut client);
        self.release_client(client).await;
        result
    }
}