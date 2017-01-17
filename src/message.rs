use std::mem::transmute;

pub trait Message {
    fn get_id(&self) -> Option<u8>;
    fn get_body(&self) -> Vec<u8> {
        vec!()
    }
    fn to_bytes(&self) -> Vec<u8> {
        let body = self.get_body();
        let length = (body.len() + 4) as u32;
        let length_bytes: [u8; 4] = unsafe { transmute(length.to_be()) };

        let mut bytes = Vec::with_capacity(5 + body.len());
        if let Some(id) = self.get_id() {
            bytes.push(id);
        }
        bytes.extend(length_bytes.iter());
        bytes.extend(body);
        bytes
    }
}

// Message types

#[derive(Debug, Eq, PartialEq)]
pub struct StartupMessage<'a> {
    pub user: &'a str,
    pub database: Option<&'a str>,
    pub params: Vec<(String, String)>,
}

fn extend_string(body: &mut Vec<u8>, s: &str) {
    body.extend(s.as_bytes());
    body.push(0);

}

impl <'a> Message for StartupMessage<'a> {
    fn get_id(&self) -> Option<u8> {
        None
    }
    fn get_body(&self) -> Vec<u8> {
        let mut body: Vec<u8> = Vec::with_capacity(256);
        body.extend(&[0, 3, 0, 0]);
        extend_string(&mut body, "user");
        extend_string(&mut body, self.user);
        if let Some(ref db) = self.database {
            extend_string(&mut body, "database");
            extend_string(&mut body, db);
        }
        for &(ref param, ref value) in &self.params {
            extend_string(&mut body, param);
            extend_string(&mut body, value);
        }
        body.push(0);
        body
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PasswordMessage<'a> {
    pub hash: &'a str,
}

impl <'a> Message for PasswordMessage<'a> {
    fn get_id(&self) -> Option<u8> {
        Some(0x70) // 'p'
    }

    fn get_body(&self) -> Vec<u8> {
        let mut body = vec!();
        extend_string(&mut body, self.hash);
        body
    }
}
        
#[derive(Debug, Eq, PartialEq)]
pub struct Terminate;

impl Message for Terminate {
    fn get_id(&self) -> Option<u8> {
        Some(0x58)  // 'X'
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
    fn get_body(&self) -> Vec<u8> {
        let mut body = Vec::with_capacity(self.query.as_bytes().len() + 1);
        extend_string(&mut body, &self.query);
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminate() {
        assert_eq!(
            Terminate.to_bytes(),
            b"\x58\0\0\0\x04".to_vec()
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
            msg.to_bytes(),
            b"\0\0\0\x2d\x00\x03\x00\x00user\0cliff\0name\0Theseus\0vessel\0ship\0\0".to_vec()
        );
    }

    #[test]
    fn test_password_message() {
        let msg = PasswordMessage {
            hash: "open sesame",
        };
        assert_eq!(
            msg.to_bytes(),
            b"p\0\0\0\x10open sesame\0".to_vec()
        );
    }

    #[test]
    fn test_query_message() {
        let msg = Query {
            query: "SELECT 1".to_string(),
        };
        assert_eq!(
            msg.to_bytes(),
            b"Q\0\0\0\x0dSELECT 1\0".to_vec()
        );
    }
}
