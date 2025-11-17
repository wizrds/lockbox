use thiserror::Error;
use sea_orm::{DbErr, SqlErr};


#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbErr),
    #[error("SQL error: {0}")]
    SqlError(#[from] SqlErr),
    #[error("Transaction error: {0}")]
    TransactionError(#[from] std::io::Error),
    #[error("Missing primary DSN")]
    MissingPrimaryDsn,
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}


pub type DatabaseResult<T> = Result<T, DatabaseError>;
