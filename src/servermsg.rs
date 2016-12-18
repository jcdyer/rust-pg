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

#[derive(Debug, PartialEq, Eq)]
pub enum ServerMsg<'a> {
    Error(&'a[u8]),
    Notification(&'a[u8]),
    Auth(AuthMsg<'a>),
    ReadyForQuery,
    ParamStatus(&'a[u8], &'a[u8]),
    BackendKeyData(u32, u32),
    RowDescription(&'a[u8]),
    RowData(&'a[u8]),
    Unknown(&'a[u8]),
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
        let (_, extra) = message.split_at(5);
        if identifier == &b'R' {
            AuthMsg::from_slice(message).map(ServerMsg::Auth)
        } else if identifier == &b'S' {  // Parameter Status
            let mut param_iter = extra.split(|c| c == &0); // split on nulls
            let name = param_iter.next().unwrap();
            let value = param_iter.next().unwrap();
            let nothing = param_iter.next().unwrap(); // The second null is the terminator
            if nothing != [] {
                Err(PgError::Other)
            } else {
                Ok(ServerMsg::ParamStatus(name, value))
            }
        } else if identifier == &b'K' {  // Row Description
            Ok(ServerMsg::Unknown(extra))  // TBD
        } else if identifier == &b'T' {  // Row Description
            Ok(ServerMsg::RowDescription(extra))  // TBD
        } else if identifier == &b'D' {  // Data Row
            Ok(ServerMsg::RowData(extra))  // TBD
        } else if identifier == &b'Z' {  // ReadyForQuery
            Ok(ServerMsg::ReadyForQuery)
        } else {
            Err(PgError::Other)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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
    if input.len() < 5 {
        Err(PgError::Other)
    } else {
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

    #[test]
    fn test_server_response_parse() {
        /// Test a realistic example of a server response.  Need this buffer in bytes, but it's not all utf-8

        let buffer = b"R\x00\x00\x00\x08\x00\x00\x00\x00S\x00\x00\x00\x16application_name\x00\x00S\x00\x00\x00\x19client_encoding\x00UTF8\x00S\x00\x00\x00\x17DateStyle\x00ISO, MDY\x00S\x00\x00\x00\x19integer_datetimes\x00on\x00S\x00\x00\x00\x1bIntervalStyle\x00postgres\x00S\x00\x00\x00\x15is_superuser\x00off\x00S\x00\x00\x00\x19server_encoding\x00UTF8\x00S\x00\x00\x00\x19server_version\x009.6.1\x00S\x00\x00\x00 session_authorization\x00cliff\x00S\x00\x00\x00#standard_conforming_strings\x00on\x00S\x00\x00\x00\x18TimeZone\x00US/Eastern\x00K\x00\x00\x00\x0c\x00\x00\x17\xbb\x15b\xfb1Z\x00\x00\x00\x05I";
        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::Auth(AuthMsg::AuthOk));
        assert_eq!(buffer.len(), 314);
        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("application_name".as_bytes(), "".as_bytes()));
        assert_eq!(buffer.len(), 291);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("client_encoding".as_bytes(), "UTF8".as_bytes()));
        assert_eq!(buffer.len(), 265);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("DateStyle".as_bytes(), "ISO, MDY".as_bytes()));
        assert_eq!(buffer.len(), 241);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("integer_datetimes".as_bytes(), "on".as_bytes()));
        assert_eq!(buffer.len(), 215);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("IntervalStyle".as_bytes(), "postgres".as_bytes()));
        assert_eq!(buffer.len(), 187);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("is_superuser".as_bytes(), "off".as_bytes()));
        assert_eq!(buffer.len(), 165);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("server_encoding".as_bytes(), "UTF8".as_bytes()));
        assert_eq!(buffer.len(), 139);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("server_version".as_bytes(), "9.6.1".as_bytes()));
        assert_eq!(buffer.len(), 113);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(
            msg, 
            ServerMsg::ParamStatus("session_authorization".as_bytes(), "cliff".as_bytes())
        );
        assert_eq!(buffer.len(), 80);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(
            msg,
            ServerMsg::ParamStatus("standard_conforming_strings".as_bytes(), "on".as_bytes()));
        assert_eq!(buffer.len(), 44);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("TimeZone".as_bytes(), "US/Eastern".as_bytes()));
        assert_eq!(buffer.len(), 19);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert!(
            match msg {
                ServerMsg::Unknown(x) => {
                    assert_eq!(x.len(), 8);
                    true
                },
                _ => false,
            }
        );
        assert_eq!(buffer.len(), 6);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ReadyForQuery);
        assert_eq!(buffer.len(), 0);

        assert!(take_msg(buffer).is_err())
    }
}
