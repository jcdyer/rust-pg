use std::io;
use std::result;
use connection::Connection;

pub mod connection;
pub mod message;
pub mod servermsg;

pub mod error {
    use std::fmt;
    use std::io;
    use std::net;
    use std::error::Error;

    #[derive(Debug)]
    pub enum PgError {
        Other,
        Io(io::Error),
        IP(net::AddrParseError),
    }

    impl fmt::Display for PgError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                PgError::Io(ref err) => err.fmt(f),
                PgError::IP(ref err) => err.fmt(f),
                PgError::Other => write!(f, "Unknown error"),
            }
        }
    }

    impl Error for PgError {
        fn description(&self) -> &str {
            match *self {
                PgError::Io(ref err) => err.description(),
                PgError::IP(ref err) => err.description(),
                PgError::Other => "An error occurred",
            }
        }

        fn cause(&self) -> Option<&Error> {
            match *self {
                PgError::Io(ref err) => Some(err),
                PgError::IP(ref err) => Some(err),
                PgError::Other => None,
            }
        }
    }
}

pub type Result<T> = result::Result<T, error::PgError>;



