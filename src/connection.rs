use std::io::{Read, Write};
use std::net;
use Result;
use error::PgError;
use message::{Message, StartupMessage, Query, Terminate};
use servermsg::{take_msg, ServerMsg, AuthMsg};

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
        let mut conn: Option<Result<Connection>> = None;
        let mut remainder = &buf[..];
        let mut authorized = false;
        while remainder.len() > 0 {
            println!("Authorized? {:?}", authorized);
            let (bytes, excess) = try!(take_msg(remainder));
            println!("B: {:?}, E: {:?}", bytes, excess);
            let msg = try!(ServerMsg::from_slice(bytes));
            println!("{:?}", msg);
            remainder = excess;

            match msg {
                ServerMsg::Auth(AuthMsg::Ok) => {
                    println!("Authorized");
                    authorized = true;
                },
                ServerMsg::Auth(method) => {
                    println!("Unimplemented authentication method, {:?}", method);
                    return Err(PgError::Other);
                },
                ServerMsg::ReadyForQuery => break,
                _ => {},
            };
        }
        if authorized == true {
            Ok(Connection { 
                user: user.clone(),
                database: database.clone(),
                host: host.clone(),
                port: port,
                socket: socket,
            })
        } else {
            Err(PgError::Unauthenticated)
        }
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
        println!("{:?}", conn);
        assert!(conn.is_ok());
    }
}
