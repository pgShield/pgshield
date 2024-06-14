use postgres::{Client, Error, types::ToSql};
use serde::de::DeserializeOwned;
use std::error::Error as StdError;

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

    pub fn bind<T: ToSql + Sync + 'static>(&mut self, value: T) -> &mut Self {
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
    pub fn execute(&self, client: &mut Client) -> Result<u64, Error> {
        let params: Vec<&(dyn ToSql + Sync)> = self.params.iter().map(|p| p.as_ref()).collect();
        client.execute(&self.sql, &params)
    }

    pub fn query<T: DeserializeOwned>(&self, client: &mut Client) -> Result<Vec<T>, Error> {
        let params: Vec<&(dyn ToSql + Sync)> = self.params.iter().map(|p| p.as_ref()).collect();
        let rows = client.query(&self.sql, &params)?;

        let result: Result<Vec<T>, Box<dyn StdError>> = rows.iter().map(|row| {
            let mut map = serde_json::Map::new();
            for column in row.columns() {
                let value: serde_json::Value = match row.try_get(column.name()) {
                    Ok(v) => json!(v),
                    Err(_) => serde_json::Value::Null,
                };
                map.insert(column.name().to_string(), value);
            }
            serde_json::from_value(serde_json::Value::Object(map)).map_err(|e| Box::new(e) as Box<dyn StdError>)
        }).collect();

        result.map_err(|e| Error::from(*e.downcast::<postgres::Error>().unwrap()))
    }
}
