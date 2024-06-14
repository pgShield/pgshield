use std::sync::{Arc, Mutex};
use tokio_postgres::{Client, Config, Error, NoTls, Socket, tls::NoTlsStream};

pub struct ConnectionPool {
    clients: Arc<Mutex<Vec<(Client, Client)>>>,
    config: Config,
    max_size: usize,
}

impl ConnectionPool {
    pub async fn new(config: Config, max_size: usize) -> Result<Self, Error> {
        let mut clients = Vec::new();

        for _ in 0..max_size {
            let client = config.connect(NoTls).await?;
            clients.push((client.clone(), client));
        }

        Ok(ConnectionPool {
            clients: Arc::new(Mutex::new(clients)),
            config,
            max_size,
        })
    }

    pub async fn get_client(&self) -> Result<Client, Error> {
        let mut clients = self.clients.lock().unwrap();

        if let Some((client, _)) = clients.pop() {
            Ok(client)
        } else {
            let client = self.config.connect(NoTls).await?;
            Ok(client)
        }
    }

    pub async fn release_client(&self, client: Client) {
        let mut clients = self.clients.lock().unwrap();

        if clients.len() < self.max_size {
            clients.push((client.clone(), client));
        }
    }
}