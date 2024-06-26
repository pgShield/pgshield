use std::fmt;

#[derive(Debug)]
pub enum PostgresError {
    Io(std::io::Error),
    Protocol(String),
    Auth(String),
    Parse(String),
    Tls(native_tls::Error),
}

impl fmt::Display for PostgresError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PostgresError::Io(err) => write!(f, "I/O error: {}", err),
            PostgresError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            PostgresError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            PostgresError::Parse(msg) => write!(f, "Parse error: {}", msg),
            PostgresError::Tls(err) => write!(f, "TLS error: {}", err),
        }
    }
}

impl From<std::io::Error> for PostgresError {
    fn from(err: std::io::Error) -> Self {
        PostgresError::Io(err)
    }
}

impl From<native_tls::Error> for PostgresError {
    fn from(err: native_tls::Error) -> Self {
        PostgresError::Tls(err)
    }
}