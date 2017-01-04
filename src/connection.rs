use std::io::{Read, Write};
use std::net;
use std::thread::sleep;
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
        let port = 5432;
        let mut socket = try!(net::TcpStream::connect((host.as_str(), port)));
        try!(socket.set_read_timeout(Some(Duration::new(0, 1))));
        try!(socket.set_nodelay(true));

        let startup = StartupMessage {
            user: &user,
            database: Some(&database),
            params: vec!(),
        };
        let bytes_to_send = startup.to_bytes();
        println!("Startup Message: {:?}", bytes_to_send);
        let mut x = try!(socket.write_all(&bytes_to_send)); 
        let mut buf = Vec::with_capacity(1024);
        let mut z = 5;
        let x = socket.read_to_end(&mut buf);
        println!("msg read {:?}", &buf[..]);
        let mut remainder = &buf[..];
        let mut authorized = false;
        let mut ready_for_query = false;
        while remainder.len() > 0 {
            println!("Authorized? {:?}", authorized);
            let (bytes, excess) = try!(take_msg(remainder));
            println!("B: {:?}, E: {:?}", bytes, excess);
            let msg = try!(ServerMsg::from_slice(bytes));
            println!("Message: {:?}", msg);
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
                ServerMsg::ErrorResponse(err) => {
                    let message = err.get(3).unwrap();
                    return Err(PgError::Error(message.to_string()));
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
        println!("Sending query {:?}", query.to_bytes());
        try!(self.socket.write_all(&query.to_bytes()));
        self.ready_for_query = false;
        let mut buf: Vec<u8> = vec!();
        while buf.len() == 0 {
            match self.socket.read_to_end(&mut buf) {
                Ok(_) => break,
                Err(ioerr) => if let Some(11) = ioerr.raw_os_error() {
                    continue;
                } else {
                    try!(Err(ioerr));
                }
            }
        }
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
                    data.push(row);
                },
                ServerMsg::CommandComplete(command_tag) => {
                    complete = Some(command_tag);
                },
                ServerMsg::ReadyForQuery => {
                    if remainder.len() > 0 {
                        return Err(PgError::Error(
                            format!("Received data after ReadyForQuery: {:?}", remainder)
                        ));
                    };
                    self.ready_for_query = true;
                },
                other => {
                    println!("unexpected data: {:?}", other);
                },
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
        let host = "127.0.0.1";
        let database = Some(user);
        let mut conn = Connection::new(user, host, database);
        println!("Connection: {:?}", conn);
        assert!(conn.is_ok());
    }

    #[test]
    fn test_query_with_bad_creds() {
        let user = "notauser";
        let host = "127.0.0.1";
        let database = Some("notadb");
        let mut conn = Connection::new(user, host, database);
        assert!(conn.is_err());
    }

    #[test]
    fn test_query() {
        let user_string = env::var("USER").unwrap();
        let user = user_string.as_ref();
        let host = "127.0.0.1";
        let mut conn = Connection::new(user, host, Some(user)).expect("Could not establish connection");
        let data = conn.query("SELECT VERSION();").unwrap();
        assert_eq!(data.len(), 1);
        let ref result = data[0][0];
        let word = match result.find(' ') {
            Some(n) => data[0][0].split_at(n).0,
            None => &result,
        };  // First word of the version string
        assert_eq!(word, "PostgreSQL");
    }
}
