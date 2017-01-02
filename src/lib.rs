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
    use std::str::Utf8Error;
    use std::error::Error;

    #[derive(Debug)]
    pub enum PgError {
        Io(io::Error),
        IP(net::AddrParseError),
        Utf8(Utf8Error),
        Error(String),
        Unauthenticated,
        Other,
    }

    impl fmt::Display for PgError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                PgError::Io(ref err) => err.fmt(f),
                PgError::IP(ref err) => err.fmt(f),
                PgError::Utf8(ref err) => err.fmt(f),
                PgError::Error(ref string) => write!(f, "Error: {:?}", string),
                PgError::Unauthenticated => write!(f, "Unauthenticated"),
                PgError::Other => write!(f, "An unknown error occured"),
            }
        }
    }

    impl Error for PgError {
        fn description(&self) -> &str {
            match *self {
                PgError::Io(ref err) => err.description(),
                PgError::IP(ref err) => err.description(),
                PgError::Utf8(ref err) => err.description(),
                PgError::Error(ref string) => string,
                PgError::Unauthenticated => "Unauthenticated",
                PgError::Other => "An error occurred",
            }
        }

        fn cause(&self) -> Option<&Error> {
            match *self {
                PgError::Io(ref err) => Some(err),
                PgError::IP(ref err) => Some(err),
                PgError::Utf8(ref err) => Some(err),
                PgError::Error(..) => None,
                PgError::Unauthenticated => None,
                PgError::Other => None,
            }
        }
    }

    impl From<io::Error> for PgError {
        fn from(err: io::Error) -> PgError {
            PgError::Io(err)
        }
    }

}

pub type Result<T> = result::Result<T, error::PgError>;
