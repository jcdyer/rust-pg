use std::str::FromStr;
use super::Result;
use super::error::PgError;

#[inline]
fn _ascii_byte(input: &str) -> u8 {
    let bytes = input.as_bytes();
    if bytes.len() == 1 {
        bytes[0]
    } else {
        panic!("Not an ascii byte")
    }
}

pub enum ServerMsg<'a> {
    Auth(AuthMsg<'a>),
    ReadyForQuery,
    ParamStatus(&'a[u8], &'a[u8]),
    Error(&'a[u8]),
    Notification(&'a[u8]),
    Row(&'a[u8]),
}

impl <'a> ServerMsg<'a> {
    pub fn from_slice(message: &[u8]) -> Result<ServerMsg> {
        let length: usize = 
            ((message[1] as usize) << 24) + 
            ((message[2] as usize) << 16) + 
            ((message[3] as usize) << 8) +
            (message[4] as usize);
        if message.len() != length + 1 {
            return Err(PgError::Other)
        }
        let identifier = message.get(0).unwrap();
        if identifier == &_ascii_byte("R") {
            AuthMsg::from_slice(message).map(ServerMsg::Auth)
        } else if identifier == &_ascii_byte("S") {
            let (_, param_slice) = message.split_at(5); // strip off the identifier and length
            let mut param_iter = param_slice.split(|c| c == &0); // split on nulls
            let name = param_iter.next().unwrap();
            let value = param_iter.next().unwrap();
            let nothing = param_iter.next().unwrap(); // The second null is the terminator
            if nothing != [] {
                Err(PgError::Other)
            } else {
                Ok(ServerMsg::ParamStatus(name, value))
            }
        } else {
            Err(PgError::Other)
        }
    }
}

pub enum AuthMsg<'a> {
    AuthOk,
    AuthCleartext,
    AuthMd5(&'a[u8]),
    UnknownAuth,
}


impl <'a> AuthMsg<'a> {
    pub fn from_slice(message: &'a [u8]) -> Result<AuthMsg> {
        let identifier = message.get(0).expect("input not long enough");
        if identifier != &_ascii_byte("R") {
            Err(PgError::Other)
        } else {
            Ok(match (message[5], message[6], message[7], message[8]) {
                (0, 0, 0, 0) => AuthMsg::AuthOk,
                (0, 0, 0, 3) => AuthMsg::AuthCleartext,
                (0, 0, 0, 5) => {
                    let salt = &message[9 .. 13];
                    AuthMsg::AuthMd5(salt)
                },
                _ => AuthMsg::UnknownAuth,
            })
        }
    }
}

pub fn take_msg(input: &[u8]) -> Result<(&[u8], &[u8])> {
    let length: usize = 
        ((input[1] as usize) << 24) + 
        ((input[2] as usize) << 16) + 
        ((input[3] as usize) << 8) +
        (input[4] as usize);
    if input.len() < length + 1 {
        Err(PgError::Other)
    } else {
        Ok(input.split_at(length + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_msg() {
        let buffer = [69, 0, 0, 0, 5, 1, 72, 0, 0, 0, 12, 1, 2, 3, 4, 5, 6, 7, 8];
        let (first, rest) = take_msg(&buffer).unwrap();
        assert_eq!(first, [69, 0, 0, 0, 5, 1]);
        let (second, nothing) = take_msg(rest).unwrap();
        assert_eq!(second, [72, 0, 0, 0, 12, 1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(nothing, []);
    }
}
