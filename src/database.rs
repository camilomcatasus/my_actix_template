use std::fmt::Display;
use libsql_client::{client::Client, Value};
use bb8::ManageConnection;
use async_trait::async_trait;

pub struct LibSqlConnectionManager;

#[derive(Debug)]
pub enum LibSqlPoolError {
    ConnectionError,
    QueryError

}

impl Display for LibSqlPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        match self {
            Self::ConnectionError => write!(f, "Error establishing Connection"),
            Self::QueryError => write!(f, "Error when making query"),
        }
    }
}

#[async_trait]
impl ManageConnection for LibSqlConnectionManager {
    type Connection = Client;
    type Error = LibSqlPoolError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(libsql_client::Client::from_env().await.map_err(|_| LibSqlPoolError::ConnectionError)?)
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        let query_result = conn.execute("SELECT 1").await;
        match query_result {
            Ok(val) => {
                let first_row = val.rows.first().ok_or(LibSqlPoolError::ConnectionError)?;
                let first_val = &first_row.values[0];
                match first_val {
                    Value::Integer { value } => {
                        if *value != 1i64 {
                            return Err(LibSqlPoolError::QueryError)
                        }
                    }
                    _ => ()
                }
            },
            Err(err) => return Err(LibSqlPoolError::ConnectionError)
        }
        Ok(())
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        false
    }
}
