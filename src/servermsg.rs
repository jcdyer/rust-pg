use std::str::from_utf8;
use super::Result;
use super::error::PgError;

fn slice_to_u32(input: &[u8]) -> u32 {
    if input.len() != 4 { 
        panic!("Expected four bytes");
    }
    let mut value: u32 = 0;
    for x in input {
        value <<= 8;
        value += *x as u32;
    }
    value
}

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
    ParamStatus(&'a str, &'a str),
    BackendKeyData(u32, u32),
    RowDescription(&'a[u8]),  // TBD
    RowData(&'a[u8]),  // TBD
    Unknown(&'a[u8]),  // TBD
}

impl <'a> ServerMsg<'a> {
    pub fn from_slice(message: &[u8]) -> Result<ServerMsg> {
        let length = 1 + slice_to_u32(&message[1 .. 5]) as usize;
        if message.len() != length {
            return Err(PgError::Other)
        }
        let identifier = message.get(0).unwrap();
        let (_, extra) = message.split_at(5);
        if identifier == &b'R' {
            AuthMsg::from_slice(message).map(ServerMsg::Auth)
        } else if identifier == &b'S' {  // Parameter Status
            let mut param_iter = extra.split(|c| c == &0); // split on nulls
            let name = from_utf8(param_iter.next().unwrap()).unwrap();
            let value = from_utf8(param_iter.next().unwrap()).unwrap();
            let nothing = param_iter.next().unwrap(); // The second null is the terminator
            if nothing != [] {
                Err(PgError::Other)
            } else {
                Ok(ServerMsg::ParamStatus(name, value))
            }
        } else if identifier == &b'K' {  // BackendKeyData
            let pid = slice_to_u32(&extra[..4]);
            let key = slice_to_u32(&extra[4..]);
            Ok(ServerMsg::BackendKeyData(pid, key))  
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
    Ok,
    Kerberos,
    Cleartext,
    Md5(&'a[u8]),
    ScmCredential,
    Gss,
    Sspi,
    GssContinue(&'a[u8]),
    Unknown,
}


impl <'a> AuthMsg<'a> {
    pub fn from_slice(message: &'a [u8]) -> Result<AuthMsg> {
        let identifier = message.get(0).expect("input not long enough");
        if identifier != &_ascii_byte("R") {
            Err(PgError::Other)
        } else {
            Ok(match slice_to_u32(&message[5..9]) {
                0 => AuthMsg::Ok,
                2 => AuthMsg::Kerberos,
                3 => AuthMsg::Cleartext,
                5 => {
                    let salt = &message[9 .. 13];
                    AuthMsg::Md5(salt)
                },
                6 => AuthMsg::ScmCredential,
                7 => AuthMsg::Gss,
                8 => {
                    let gss_data = &message[9 .. 13];
                    AuthMsg::GssContinue(gss_data)
                },
                9 => AuthMsg::Sspi,
                _ => AuthMsg::Unknown,
            })
        }
    }
}

pub fn take_msg(input: &[u8]) -> Result<(&[u8], &[u8])> {
    if input.len() < 5 {
        Err(PgError::Other)
    } else {
        let length = 1 + slice_to_u32(&input[1 .. 5]) as usize;
        if input.len() < length {
            Err(PgError::Other)
        } else {
            Ok(input.split_at(length))
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
        let buffer = b"R\x00\x00\x00\x08\x00\x00\x00\x00S\x00\x00\x00\x16application_name\x00\x00S\x00\x00\x00\x19client_encoding\x00UTF8\x00S\x00\x00\x00\x17DateStyle\x00ISO, MDY\x00S\x00\x00\x00\x19integer_datetimes\x00on\x00S\x00\x00\x00\x1bIntervalStyle\x00postgres\x00S\x00\x00\x00\x15is_superuser\x00off\x00S\x00\x00\x00\x19server_encoding\x00UTF8\x00S\x00\x00\x00\x19server_version\x009.6.1\x00S\x00\x00\x00 session_authorization\x00cliff\x00S\x00\x00\x00#standard_conforming_strings\x00on\x00S\x00\x00\x00\x18TimeZone\x00US/Eastern\x00K\x00\x00\x00\x0c\x00\x00\x17\xbb\x15b\xfb1Z\x00\x00\x00\x05I";
        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::Auth(AuthMsg::Ok));
        assert_eq!(buffer.len(), 314);
        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("application_name", ""));
        assert_eq!(buffer.len(), 291);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("client_encoding", "UTF8"));
        assert_eq!(buffer.len(), 265);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("DateStyle", "ISO, MDY"));
        assert_eq!(buffer.len(), 241);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("integer_datetimes", "on"));
        assert_eq!(buffer.len(), 215);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("IntervalStyle", "postgres"));
        assert_eq!(buffer.len(), 187);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("is_superuser", "off"));
        assert_eq!(buffer.len(), 165);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("server_encoding", "UTF8"));
        assert_eq!(buffer.len(), 139);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("server_version", "9.6.1"));
        assert_eq!(buffer.len(), 113);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("session_authorization", "cliff"));
        assert_eq!(buffer.len(), 80);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("standard_conforming_strings", "on"));
        assert_eq!(buffer.len(), 44);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ParamStatus("TimeZone", "US/Eastern"));
        assert_eq!(buffer.len(), 19);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert!(
            match msg {
                ServerMsg::BackendKeyData(..) => true,
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
