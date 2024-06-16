use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio_postgres::{Client, Config, Error, NoTls, Row};
use tokio_postgres::types::ToSql;
use log;
use std::error::Error as StdError;
use std::fmt;
 
#[derive(Debug)]
pub enum MyError {
    Postgres(tokio_postgres::Error),
    Serde(serde_json::Error),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::Postgres(e) => write!(f, "Postgres error: {}", e),
            MyError::Serde(e) => write!(f, "Serde error: {}", e),
        }
    }
}

impl From<tokio_postgres::Error> for MyError {
    fn from(e: tokio_postgres::Error) -> Self {
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
    params: Vec<Box<dyn ToSql + Sync>>,
}

impl QueryBuilder {
    pub fn new(sql: &str) -> Self {
        QueryBuilder {
            sql: sql.to_string(),
            params: Vec::new(),
        }
    }

    pub fn bind<T: ToSql + Sync +'static>(&mut self, value: T) -> &mut Self {
        self.params.push(Box::new(value));
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
    params: Vec<Box<dyn ToSql + Sync>>,
}

impl Query {
    pub async fn execute(&self, client: &mut Client) -> Result<u64, Error> {
        let params: Vec<&(dyn ToSql + Sync)> = self.params.iter().map(|p| p.as_ref()).collect();
        client.execute(&self.sql, &params).await
    }
    
    pub async fn query<T: DeserializeOwned>(&self, client: &mut Client) -> Result<Vec<T>, MyError> {
        let mut map = serde_json::Map::new();  // Reuse a single Map
    
        let params: Vec<&(dyn ToSql + Sync)> = self.params.iter().map(|p| p.as_ref()).collect();
        let rows = client.query(&self.sql, &params).await.map_err(MyError::from)?;
    
        let mut result = Vec::new();
        for row in rows {
            map.clear();  // Clear the map before processing each row
            for column in row.columns() {
                let value: Value = match row.try_get::<&str, Option<String>>(column.name()) {
                    Ok(Some(val)) => serde_json::from_str(&val).unwrap_or(Value::Null),
                    Ok(None) => Value::Null,
                    Err(e) => return Err(e.into()),
                };
                map.insert(column.name().to_string(), value);
            }
            let value: Value = Value::Object(map.clone());
            
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