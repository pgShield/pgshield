use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use bytes::{BytesMut, BufMut, Buf};
use tokio_native_tls::{TlsConnector, TlsStream};
use native_tls::TlsConnector as NativeTlsConnector;

use crate::config::{ConnectionConfig, SslMode};
use crate::error::PostgresError;
use crate::auth::{handle_authentication, AuthMethod};

const PROTOCOL_VERSION: i32 = 196608; // 3.0

pub enum Connection {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl Connection {
    pub async fn new(config: &ConnectionConfig) -> Result<Self, PostgresError> {
        let stream = TcpStream::connect((config.host.as_str(), config.port)).await?;
        
        let mut connection = match config.ssl_mode {
            SslMode::Disable => Connection::Plain(stream),
            SslMode::Prefer | SslMode::Require => {
                match Self::try_ssl_connection(stream, &config.host, &config.ssl_mode).await {
                    Ok(tls_stream) => Connection::Tls(tls_stream),
                    Err(e) if config.ssl_mode == SslMode::Prefer => {
                        eprintln!("SSL connection failed, falling back to plain: {}", e);
                        Connection::Plain(TcpStream::connect((config.host.as_str(), config.port)).await?)
                    },
                    Err(e) => return Err(e),
                }
            }
        };

        connection.startup(config).await?;
        Ok(connection)
    }

    async fn try_ssl_connection(mut stream: TcpStream, host: &str, ssl_mode: &SslMode) -> Result<TlsStream<TcpStream>, PostgresError> {
        // Send SSL request
        stream.write_all(&[b'S', 0, 0, 0, 8, 4, 210, 22, 47]).await?;
        
        // Read server response
        let mut response = [0u8; 1];
        stream.read_exact(&mut response).await?;

        match response[0] {
            b'S' => {
                let connector = TlsConnector::from(NativeTlsConnector::builder()
                    .danger_accept_invalid_certs(*ssl_mode == SslMode::Prefer)
                    .build()?);
                Ok(connector.connect(host, stream).await?)
            },
            b'N' => Err(PostgresError::Protocol("Server does not support SSL".into())),
            _ => Err(PostgresError::Protocol("Unexpected response to SSL request".into())),
        }
    }

    async fn startup(&mut self, config: &ConnectionConfig) -> Result<(), PostgresError> {
        let mut buf = BytesMut::with_capacity(1024);
        buf.put_i32(PROTOCOL_VERSION);
        buf.put_slice(b"user\0");
        buf.put_slice(config.user.as_bytes());
        buf.put_u8(0);
        buf.put_slice(b"database\0");
        buf.put_slice(config.database.as_bytes());
        buf.put_u8(0);
        buf.put_u8(0);

        self.write_message(None, &buf).await?;

        loop {
            let (message_type, data) = self.read_message().await?;
            match message_type {
                Some(b'R') => handle_authentication(self, &data, config).await?,
                Some(b'Z') => break, // ReadyForQuery
                Some(b'E') => return Err(PostgresError::Protocol("Error during startup".into())),
                _ => {} // Ignore other messages
            }
        }

        Ok(())
    }

    pub async fn write_message(&mut self, message_type: Option<u8>, data: &[u8]) -> Result<(), PostgresError> {
        let mut buf = BytesMut::with_capacity(5 + data.len());
        if let Some(mt) = message_type {
            buf.put_u8(mt);
        }
        buf.put_u32((data.len() + 4) as u32);
        buf.put_slice(data);

        match self {
            Connection::Plain(stream) => stream.write_all(&buf).await?,
            Connection::Tls(stream) => stream.write_all(&buf).await?,
        }

        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<(Option<u8>, BytesMut), PostgresError> {
        let mut header = [0u8; 5];
        match self {
            Connection::Plain(stream) => { stream.read_exact(&mut header).await?; },
            Connection::Tls(stream) => { stream.read_exact(&mut header).await?; },
        }

        let message_type = header[0];
        let length = (&header[1..5]).get_u32() as usize - 4;
        let mut data = BytesMut::with_capacity(length);
        data.resize(length, 0);

        match self {
            Connection::Plain(stream) => { stream.read_exact(&mut data).await?; },
            Connection::Tls(stream) => { stream.read_exact(&mut data).await?; },
        }

        Ok((Some(message_type), data))
    }
}

