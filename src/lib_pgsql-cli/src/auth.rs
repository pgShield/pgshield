use bytes::{BytesMut, BufMut, Buf};
use md5::{Md5, Digest};
use ldap3::LdapConn;
use url::Url;

use crate::error::PostgresError;
use crate::connection::Connection;
use crate::config::ConnectionConfig;

#[derive(Clone)]
pub enum AuthMethod {
    Password,
    MD5,
    LDAP(LDAPConfig),
}

#[derive(Clone)]
pub struct LDAPConfig {
    pub ldap_server: String,
    pub bind_dn: String,
}

pub async fn handle_authentication(conn: &mut Connection, data: &[u8], config: &ConnectionConfig) -> Result<(), PostgresError> {
    let auth_type = (&data[..4]).get_u32_le();
    match auth_type {
        0 => Ok(()), // AuthenticationOk
        3 => handle_cleartext_password(conn, config).await,
        5 => handle_md5_password(conn, config, &data[4..]).await,
        10 => handle_sasl_authentication(conn, config).await,
        _ => Err(PostgresError::Auth(format!("Unsupported authentication type: {}", auth_type))),
    }
}

async fn handle_cleartext_password(conn: &mut Connection, config: &ConnectionConfig) -> Result<(), PostgresError> {
    let mut buf = BytesMut::with_capacity(config.password.len() + 5);
    buf.put_u8(b'p');
    buf.put_i32((config.password.len() + 5) as i32);
    buf.put_slice(config.password.as_bytes());
    buf.put_u8(0);
    conn.write_message(None, &buf).await
}

async fn handle_md5_password(conn: &mut Connection, config: &ConnectionConfig, salt: &[u8]) -> Result<(), PostgresError> {
    let mut hasher = Md5::new();
    hasher.update(config.password.as_bytes());
    hasher.update(config.user.as_bytes());
    let result = hasher.finalize();

    let mut hasher = Md5::new();
    hasher.update(hex::encode(result));
    hasher.update(salt);
    let result = hasher.finalize();

    let pwd = format!("md5{}", hex::encode(result));
    
    let mut buf = BytesMut::with_capacity(pwd.len() + 5);
    buf.put_u8(b'p');
    buf.put_i32((pwd.len() + 5) as i32);
    buf.put_slice(pwd.as_bytes());
    buf.put_u8(0);
    conn.write_message(None, &buf).await
}

async fn handle_sasl_authentication(conn: &mut Connection, config: &ConnectionConfig) -> Result<(), PostgresError> {
    match &config.auth_method {
        AuthMethod::LDAP(ldap_config) => handle_ldap(conn, ldap_config, &config.password).await,
        _ => Err(PostgresError::Auth("Unsupported SASL authentication method".into())),
    }
}

async fn handle_ldap(conn: &mut Connection, ldap_config: &LDAPConfig, password: &str) -> Result<(), PostgresError> {
    let mut ldap = LdapConn::new(&ldap_config.ldap_server)
        .map_err(|e| PostgresError::Auth(format!("LDAP connection failed: {}", e)))?;
    
    ldap.simple_bind(&ldap_config.bind_dn, password)
        .map_err(|e| PostgresError::Auth(format!("LDAP authentication failed: {}", e)))?;

    // If LDAP authentication succeeds, send an empty password message to PostgreSQL
    let mut buf = BytesMut::with_capacity(5);
    buf.put_u8(b'p');
    buf.put_i32(5);
    buf.put_u8(0);
    conn.write_message(None, &buf).await
}

pub fn parse_auth_method(url: &Url) -> Result<AuthMethod, PostgresError> {
    let auth_method = url.query_pairs()
        .find(|(key, _)| key == "auth_method")
        .map(|(_, value)| value.into_owned())
        .unwrap_or_else(|| "password".to_string());

    match auth_method.as_str() {
        "password" => Ok(AuthMethod::Password),
        "md5" => Ok(AuthMethod::MD5),
        "ldap" => {
            let ldap_server = get_url_param(url, "ldap_server")?;
            let bind_dn = get_url_param(url, "ldap_bind_dn")?;
            Ok(AuthMethod::LDAP(LDAPConfig {
                ldap_server,
                bind_dn,
            }))
        },
        _ => Err(PostgresError::Parse(format!("Unsupported authentication method: {}", auth_method))),
    }
}

pub fn get_url_param(url: &Url, param: &str) -> Result<String, PostgresError> {
    url.query_pairs()
        .find(|(key, _)| key == param)
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| PostgresError::Parse(format!("Missing parameter: {}", param)))
}
