extern crate crypto;
use std::result;
pub use connection::Connection;

pub mod connection;
pub mod error;
pub mod message;
pub mod servermsg;
pub mod auth;

pub type Result<T> = result::Result<T, error::PgError>;
