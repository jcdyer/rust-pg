use std::result;
pub use connection::Connection;

pub mod connection;
pub mod error;
pub mod message;
pub mod servermsg;

pub type Result<T> = result::Result<T, error::PgError>;
