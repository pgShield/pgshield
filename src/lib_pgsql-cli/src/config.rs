use url::Url;
use crate::error::PostgresError;
use crate::auth::{AuthMethod, parse_auth_method};

#[derive(Clone)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
    pub ssl_mode: SslMode,
    pub auth_method: AuthMethod,
}

#[derive(Clone, PartialEq)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

impl ConnectionConfig {
    pub fn from_connection_string(connection_string: &str) -> Result<Self, PostgresError> {
        let url = Url::parse(connection_string)
            .map_err(|e| PostgresError::Parse(format!("Invalid connection string: {}", e)))?;

        if url.scheme() != "postgresql" && url.scheme() != "postgres" {
            return Err(PostgresError::Parse("Invalid scheme, expected 'postgresql' or 'postgres'".into()));
        }

        let host = url.host_str()
            .ok_or_else(|| PostgresError::Parse("Missing host".into()))?
            .to_string();
        let port = url.port().unwrap_or(5432);
        let database = url.path().trim_start_matches('/').to_string();
        let user = url.username().to_string();
        let password = url.password().unwrap_or("").to_string();

        let ssl_mode = url.query_pairs()
            .find(|(key, _)| key == "sslmode")
            .map(|(_, value)| match value.as_ref() {
                "disable" => SslMode::Disable,
                "prefer" => SslMode::Prefer,
                "require" => SslMode::Require,
                _ => SslMode::Prefer,
            })
            .unwrap_or(SslMode::Prefer);

        let auth_method = parse_auth_method(&url)?;

        Ok(Self {
            host,
            port,
            database,
            user,
            password,
            ssl_mode,
            auth_method,
        })
    }
}