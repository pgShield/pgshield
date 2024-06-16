use std::sync::{Arc, Mutex};
use tokio_postgres::{Client, Config, Error, NoTls};

pub struct Pool {
    clients: Arc<Mutex<Vec<Client>>>,
    config: Config,
    max_size: usize,
}

impl Pool {
    pub async fn new(config: Config, max_size: usize) -> Result<Self, Error> {
        let mut clients = Vec::new();

        for _ in 0..max_size {
            let (client, _connection) = config.connect(NoTls).await?;
            clients.push(client);
            
        }

        Ok(Pool {
            clients: Arc::new(Mutex::new(clients)),
            config,
            max_size,
        })
    }

    pub async fn get_client(&self) -> Result<Client, Error> {
        let mut clients = self.clients.lock().unwrap();
        if let Some(client) = clients.pop() {
            Ok(client)
        } else {
            let (client, _connection) = self.config.connect(NoTls).await?;
            Ok(client)
        }
    }

    pub async fn release_client(&self, client: Client) -> Result<(), Error> {
        let mut clients = self.clients.lock().unwrap();

        if clients.len() < self.max_size {
            clients.push(client);
        } else {
            // Log error or handle exceeding pool size gracefully (e.g., backoff)
            eprintln!("Connection pool reached max size, discarding client");
        }

        Ok(())
    }

    pub async fn execute<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Client) -> Result<T, Error>,
    {
        let mut client = self.get_client().await?;
        let result = f(&mut client);
        self.release_client(client).await?;
        result
    }
}