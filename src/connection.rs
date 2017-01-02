use std::io::{Read, Write};
use std::net;
use std::time::Duration;
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
    ready_for_query: bool,
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
        let mut socket = try!(net::TcpStream::connect((host.as_str(), port)));
        socket.set_read_timeout(Some(Duration::new(2, 0)));

        let mut buf: Vec<u8> = Vec::with_capacity(1024);
        let startup = StartupMessage {
            user: &user,
            database: Some(&database),
            params: vec!(),
        };
        println!("Startup Message: {:?}", startup.to_bytes());
        try!(socket.write_all(&startup.to_bytes()));
        try!(socket.flush());
        try!(socket.read_to_end(&mut buf));
        println!("{:?}", buf);
        let mut remainder = &buf[..];
        let mut authorized = false;
        let mut ready_for_query = false;
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
                ServerMsg::ReadyForQuery => ready_for_query = true,
                _ => {},
            };
        }
        if authorized == true {
            Ok(Connection { 
                user: user.clone(),
                database: database.clone(),
                host: host,
                port: port,
                socket: socket,
                ready_for_query: ready_for_query,
            })
        } else {
            Err(PgError::Unauthenticated)
        }
    }

    pub fn query(&mut self, sql: &str) -> Result<Vec<Vec<String>>> {
        let query = Query { query: sql.to_string() };
        try!(self.socket.write_all(&query.to_bytes()));
        self.ready_for_query = false;
        let mut buf: Vec<u8> = vec!();
        try!(self.socket.read_to_end(&mut buf));
        let mut remainder = &buf[..];
        let mut data = vec![];
        let mut complete = None;
            
        while remainder.len() > 0 {
            let (bytes, excess) = try!(take_msg(remainder));
            let msg = try!(ServerMsg::from_slice(bytes));
            remainder = excess;
            match msg {
                ServerMsg::DataRow(vec) => {
                    let mut row = vec![];
                    println!("DataRow: {:?}", vec);
                    for each in vec {
                        row.push(each.to_string());
                    }
                    data.push(row)
                },
                ServerMsg::CommandComplete(command_tag) => complete = Some(command_tag),
                ServerMsg::ReadyForQuery => {
                    if remainder.len() > 0 {
                        return Err(PgError::Error(
                            format!("Received data after ReadyForQuery: {:?}", remainder)
                        ));
                    };
                    self.ready_for_query = true;
                },
                _ => {}
            }
        }
        Ok(data)
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
    use std::env;
    use super::Connection;

    #[test]
    fn test_connect() {
        let user_string = env::var("USER").unwrap();
        let user = user_string.as_ref();
        println!("{:?}", user);
        let host = "127.0.0.1";
        let database = Some(user);
        let conn = Connection::new(user, host, database);
        println!("{:?}", conn);
        assert!(conn.is_ok());
    }

    #[test]
    fn test_query() {
        let mut conn = Connection::new("cliff", "127.0.0.1", Some("cliff")).unwrap();
        let data = conn.query("SELECT VERSION()").unwrap();
        assert_eq!(data.len(), 5);
    }
}
