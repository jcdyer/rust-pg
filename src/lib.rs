use connection::Connection;

pub mod connection;
pub mod message;


mod error {
    use std::fmt;
    use std::io;
    use std::error::Error;

    #[derive(Debug)]
    pub enum PgError {
        Other,
        Io(io::Error),
    }

    impl fmt::Display for PgError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                PgError::Io(ref err) => err.fmt(f),
                PgError::Other => write!(f, "Unknown error"),
            }
        }
    }

    impl Error for PgError {
        fn description(&self) -> &str {
            match *self {
                PgError::Io(ref err) => err.description(),
                PgError::Other => "An error occurred",
            }
        }

        fn cause(&self) -> Option<&Error> {
            match *self {
                PgError::Io(ref err) => Some(err),
                PgError::Other => None,
            }
        }
    }
}

pub type PGResult<T> = Result<T, error::PgError>;

/// This function takes a connection URL, and returns a PGResult with a
/// Connection object if the connection can be made, or a PGErr otherwise.
///
/// ```
/// use pg::connect;
/// use pg::connection::Connection;
/// assert!(connect("postgres://server.com/db").is_ok())
/// ```
pub fn connect(url: &str) -> PGResult<Connection> {
    Ok(Connection::new(url))
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::connection::Connection;

    #[test]
    fn test_connect() {
        let url = "postgres://test.url";
        assert_eq!(connect(&url).unwrap(), Connection::new(url));
    }
}
