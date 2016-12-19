use std::io::{Read, Write};
use std::net;
use super::Result;
use super::message::{Message, StartupMessage, Query, Terminate};

#[derive(Debug)]
pub struct Connection {
    user: String,
    database: String,
    host: String,
    port: u16,
    socket: net::TcpStream,
}

impl Connection {
    pub fn new(user: &str, host: &str, database: Option<&str>) -> Result<Connection> {
        let database = match database {
            Some(db) => db.to_string(),
            None => user.to_string(),
        };
        let user = user.to_string();
        let host = host.to_string();
        let port = 5432u16;
        let mut socket = net::TcpStream::connect((host.as_str(), port)).unwrap();

        let mut buf: Vec<u8> = vec!();
        let startup = StartupMessage {
            user: user.clone(),
            database: Some(database.clone()),
            params: vec!(),
        };

        socket.write_all(&startup.to_bytes()).unwrap();
        socket.read_to_end(&mut buf).unwrap();
        println!("{:?}", buf);

        Ok(Connection { 
            user: user,
            database: database,
            host: host,
            port: port,
            socket: socket,
        })
    }
    /*
    pub fn close(self) -> Result<()> {
        self.socket.write_all(&Terminate.to_bytes()).unwrap();
        Ok(())
    }
    */
}

#[cfg(test)]
mod tests {
    use super::Connection;

    #[test]
    fn test_connect() {
        let url = "postgres://test.url";
        let user = "cliff";
        let host = "localhost";
        let database = Some("db");
        let conn = Connection::new(user, host, database);
        assert!(conn.is_ok());
    }
}
