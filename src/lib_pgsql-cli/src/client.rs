use crate::connection::Connection;
use crate::config::ConnectionConfig;
use crate::error::PostgresError;

pub struct PostgresClient {
    connection: Connection,
}

impl PostgresClient {
    pub async fn connect(connection_string: &str) -> Result<Self, PostgresError> {
        let config = ConnectionConfig::from_connection_string(connection_string)?;
        let connection = Connection::new(&config).await?;
        Ok(Self { connection })
    }

    
}