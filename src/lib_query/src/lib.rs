use serde::de::DeserializeOwned;
use serde_json::Value;
use log;
use std::error::Error as StdError;
use std::fmt;

use lib_pgsqlcli::{client::PostgresClient, error::PostgresError, connection::PostgresValue};  // Adjusted the imports

#[derive(Debug)]
pub enum MyError {
    Postgres(PostgresError),
    Serde(serde_json::Error),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Postgres(e) => write!(f, "Postgres error: {}", e),
            MyError::Serde(e) => write!(f, "Serde error: {}", e),
        }
    }
}

impl StdError for MyError {}

impl From<PostgresError> for MyError {
    fn from(e: PostgresError) -> Self {
        MyError::Postgres(e)
    }
}

impl From<serde_json::Error> for MyError {
    fn from(e: serde_json::Error) -> Self {
        MyError::Serde(e)
    }
}

pub struct QueryBuilder {
    sql: String,
    params: Vec<PostgresValue>,
}

impl QueryBuilder {
    pub fn new(sql: &str) -> Self {
        QueryBuilder {
            sql: sql.to_string(),
            params: Vec::new(),
        }
    }

    pub fn bind(&mut self, value: PostgresValue) -> &mut Self {
        self.params.push(value);
        self
    }

    pub fn build(self) -> Query {
        Query {
            sql: self.sql,
            params: self.params,
        }
    }
}

pub struct Query {
    sql: String,
    params: Vec<PostgresValue>,
}

impl Query {
    pub async fn execute(&self, client: &mut PostgresClient) -> Result<u64, PostgresError> {
        client.execute(&self.sql).await.map(|_| 0)  // Adjusted the method call
    }

    pub async fn query<T: DeserializeOwned>(&self, client: &mut PostgresClient) -> Result<Vec<T>, MyError> {
        let mut map = serde_json::Map::new();  // Reuse a single Map

        let rows = client.query(&self.sql).await.map_err(MyError::from)?;  // Adjusted the method call

        let mut result = Vec::new();
        for row in rows {
            map.clear();  // Clear the map before processing each row
            for (column, value) in row {
                let json_value = match value {
                    PostgresValue::Null => Value::Null,
                    PostgresValue::Boolean(b) => Value::Bool(b),
                    PostgresValue::Int16(i) => Value::Number(i.into()),
                    PostgresValue::Int32(i) => Value::Number(i.into()),
                    PostgresValue::Int64(i) => Value::Number(i.into()),
                    PostgresValue::Float32(f) => Value::Number(serde_json::Number::from_f64(f as f64).unwrap()),
                    PostgresValue::Float64(f) => Value::Number(serde_json::Number::from_f64(f).unwrap()),
                    PostgresValue::String(s) => Value::String(s),
                    PostgresValue::Bytes(b) => Value::String(hex::encode(b)),
                };
                map.insert(column, json_value);
            }
            let value = Value::Object(map.clone());

            let deserialized: Result<T, _> = serde_json::from_value(value);
            match deserialized {
                Ok(data) => result.push(data),
                Err(e) => {
                    log::error!("Failed to deserialize row: {}", e);
                    return Err(MyError::from(e));
                }
            }
        }
        Ok(result)
    }
}
