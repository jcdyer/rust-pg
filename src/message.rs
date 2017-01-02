use std::mem::transmute;

pub trait Message {
    fn get_id(&self) -> Option<u8>;
    fn get_body(&self) -> Vec<u8> {
        vec!()
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        if let Some(id) = self.get_id() {
            bytes.push(id);
        }
        let body = self.get_body();
        let length = (body.len() + 4) as u32;
        let length_bytes: [u8; 4] = unsafe { transmute(length.to_be()) }; // or .to_le()
        bytes.extend(length_bytes.iter());
        bytes.extend(body);
        bytes
    }
}

#[derive(Debug, Eq, PartialEq)]
enum TransactionStatus {
    Idle,
    Transaction,
    Failed,
}

// Message types

#[derive(Debug, Eq, PartialEq)]
pub struct StartupMessage<'a> {
    pub user: &'a str,
    pub database: Option<&'a str>,
    pub params: Vec<(String, String)>,
}

impl <'a> Message for StartupMessage<'a> {
    fn get_id(&self) -> Option<u8> {
        None
    }
    fn get_body(&self) -> Vec<u8> {
        let mut body: Vec<u8> = Vec::with_capacity(256);
        body.extend(&[0, 3, 0, 0]);
        body.extend("user\0".as_bytes());
        body.extend(self.user.as_bytes());
        body.push(0);
        if let Some(ref db) = self.database {
            body.extend("database\0".as_bytes());
            body.extend(db.as_bytes());
            body.push(0);
        }
        for &(ref param, ref value) in &self.params {
            body.extend(param.as_bytes());
            body.push(0);
            body.extend(value.as_bytes());
            body.push(0);
        }
        body.push(0);
        body
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Terminate;

impl Message for Terminate {
    fn get_id(&self) -> Option<u8> {
        Some(0x58)  // 'X'
    }
    fn get_body(&self) -> Vec<u8> {
        vec!()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Query {
    pub query: String,
}

impl Message for Query {
    fn get_id(&self) -> Option<u8> {
        Some(0x51)  // 'Q'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminate() {
        assert_eq!(
            Terminate.to_bytes(),
            b"\x58\0\0\0\x04"
        );
    }

    #[test]
    fn test_startup_message() {
        let msg = StartupMessage {
            user: "cliff",
            database: None,
            params: vec![
                ("name".to_string(), "Theseus".to_string()), 
                ("vessel".to_string(), "ship".to_string()),
            ],
        };
        assert_eq!(
            &msg.to_bytes()[..],
            &b"\0\0\0\x2d\x00\x03\x00\x00user\0cliff\0name\0Theseus\0vessel\0ship\0\0"[..]
        );
    }
}
