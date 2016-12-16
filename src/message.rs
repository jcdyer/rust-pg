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
pub struct StartupMessage {
    pub version: (u8, u8),
    pub user: String,
    pub database: Option<String>,
    pub params: Vec<(String, String)>,
}

impl Message for StartupMessage {
    fn get_id(&self) -> Option<u8> {
        None
    }
    fn get_body(&self) -> Vec<u8> {
        let mut body: Vec<u8> = vec!();
        body.extend(&[0, 0x3, 0, 0]);
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

#[derive(Debug, Eq, PartialEq)]
pub struct ReadyForQuery {
    status: TransactionStatus
}

impl Message for ReadyForQuery {
    fn get_id(&self) -> Option<u8> {
        Some(0x5a)  //'Z'
    }

    fn get_body(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(1);
        v.extend(
            match self.status {
                TransactionStatus::Idle => "I".as_bytes(),
                TransactionStatus::Transaction => "T".as_bytes(),
                TransactionStatus::Failed => "E".as_bytes(),
            }
        );
        v
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParameterStatus {
    param: String,
    value: String,
}

impl Message for ParameterStatus {
    fn get_id(&self) -> Option<u8> {
        Some(0x53)  // 'S'
    }
    fn get_body(&self) -> Vec<u8> {
        let param_bytes = self.param.as_bytes();
        let value_bytes = self.value.as_bytes();
        let mut v = Vec::with_capacity(param_bytes.len() + value_bytes.len() + 2);
        v.extend(param_bytes);
        v.push(0);
        v.extend(value_bytes);
        v.push(0);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminate() {
        assert_eq!(
            Terminate.to_bytes(),
            [0x58, 0x0, 0x0, 0x0, 0x4u8]
        )
    }

    #[test]
    fn test_parameter_status() {
        let msg = ParameterStatus { param: "name".to_string(), value: "Theseus".to_string() };
        assert_eq!(
            msg.to_bytes(),
            [0x53, 0x0, 0x0, 0x0, 0x11, 0x6e, 0x61, 0x6d, 0x65, 0x0, 0x54, 0x68, 0x65, 0x73, 0x65, 0x75, 0x73, 0x0]
        );
    }
}
