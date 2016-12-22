use std::str::from_utf8;
use Result;
use error::PgError;


#[derive(Debug, Eq, PartialEq)]
pub enum FieldFormat {
    Text,
    Binary
}

#[derive(Debug, Eq, PartialEq)]
pub struct FieldDescription<'a> {
    field_name: &'a str,
    format: FieldFormat
}

fn find_first<T: Eq>(input: &[T], matched: T) -> Option<usize> {
    let mut x = input.len();
    for i in 0..input.len() {
        if input[i] == matched {
            x = i;
            break
        }
    }
    if x == input.len() {
        None
    } else {
        Some(x)
    }
}

fn take_cstring_plus_fixed<'a>(input: &'a[u8], fixed: usize) -> Result<(&'a str, &'a[u8], &'a[u8])> {
    let strlen = find_first(input, b'\0');
    match strlen {
        Some(strlen) => {
            let string = from_utf8(&input[..strlen]).unwrap();
            let fixed = &input[strlen+1..strlen+1+fixed];
            let extra = &input[strlen+1+fixed..];
            Ok((string, fixed, extra))
        },
        None => Err(PgError::Other)
    }
}

impl <'a> FieldDescription<'a> {
    fn take_field(input: &'a[u8]) -> Result<(&'a str, &'a[u8], &'a[u8])> {
        take_cstring_plus_fixed(input, 18)
    }

    fn from_slice(input: &[u8]) -> Result<FieldDescription> {
        let mut input_iter = input.splitn(2, |c| c == &b'\0');
        let field_name = from_utf8(input_iter.next().unwrap()).unwrap();
        let remainder = input_iter.next().unwrap();
        let format = match slice_to_u16(&remainder[16..18]) {
            0 => FieldFormat::Text,
            1 => FieldFormat::Binary,
            _ => return Err(PgError::Other),
        };
        Ok(FieldDescription {
            field_name: field_name,
            format: format,
        })
    }
}

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

fn slice_to_u16(input: &[u8]) -> u16 {
    if input.len() != 2 {
        panic!("Expected two bytes");
    }
    let mut value: u16 = 0;
    for x in input {
        value <<= 8;
        value += *x as u16;
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
pub struct FieldData<'a> {
    data: &'a[u8],
}

#[derive(Debug, PartialEq, Eq)]
pub enum ServerMsg<'a> {
    Error(&'a[u8]),
    Notification(&'a[u8]),
    Auth(AuthMsg<'a>),
    ReadyForQuery,
    CommandComplete(&'a str),
    ParamStatus(&'a str, &'a str),
    BackendKeyData(u32, u32),
    RowDescription(Vec<FieldDescription<'a>>),  // TBD
    DataRow(Vec<FieldData<'a>>),  // TBD
    Unknown(&'a[u8]),  // TBD
}

impl <'a> ServerMsg<'a> {
    pub fn from_slice(message: &[u8]) -> Result<ServerMsg> {
        let length = 1 + slice_to_u32(&message[1 .. 5]) as usize;
        if message.len() != length {
            return Err(PgError::Other)
        }
        let identifier = message.get(0).unwrap();
        let (_, mut extra) = message.split_at(5);
        if identifier == &b'R' {
            AuthMsg::from_slice(extra).map(ServerMsg::Auth)
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
            let field_count = slice_to_u16(&extra[..2]);
            let mut extra = &extra[2..];
            let mut fields = vec![];

            for i in [..field_count].iter() {
                let (fd, rem) = FieldDescription::take_field(extra).unwrap();
                let fd = FieldDescription::from_slice(fd).unwrap();
                fields.push(fd);
                extra = rem;
            }
            Ok(ServerMsg::RowDescription(fields))  // TBD
        } else if identifier == &b'D' {  // Data Row
            // TBD
            Ok(ServerMsg::DataRow(vec![FieldData { data: extra }]))  // TBD
        } else if identifier == &b'C' {  // Row Description
            let command_tag = from_utf8(&extra).unwrap();
            Ok(ServerMsg::CommandComplete(command_tag))
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
    pub fn from_slice(extra: &'a [u8]) -> Result<AuthMsg> {
        match slice_to_u32(&extra[0..4]) {
            0 => Ok(AuthMsg::Ok),
            2 => Ok(AuthMsg::Kerberos),
            3 => Ok(AuthMsg::Cleartext),
            5 => {
                let salt = &extra[4..8];
                Ok(AuthMsg::Md5(salt))
            },
            6 => Ok(AuthMsg::ScmCredential),
            7 => Ok(AuthMsg::Gss),
            8 => {
                let gss_data = &extra[4..8];
                Ok(AuthMsg::GssContinue(gss_data))
            },
            9 => Ok(AuthMsg::Sspi),
            1|4|10...255 => Ok(AuthMsg::Unknown),
            _ => Err(PgError::Other)
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
    fn test_server_startup_response_parsing() {
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

    #[test]
    fn test_server_query_response_parsing() {
        let buffer = b"T\x00\x00\x00 \x00\x01version\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x19\xff\xff\xff\xff\xff\xff\x00\x00D\x00\x00\x00_\x00\x01\x00\x00\x00UPostgreSQL 9.6.1 on x86_64-pc-linux-gnu, compiled by gcc (GCC) 6.2.1 20160830, 64-bitC\x00\x00\x00\rSELECT 1\x00Z\x00\x00\x00\x05I";

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(
            msg,
            ServerMsg::RowDescription(
                vec![FieldDescription { 
                    field_name: "version",
                    format: FieldFormat::Text,
                }]
            )
        );
        assert_eq!(buffer.len(), 116);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::DataRow(vec![FieldData { data: b"PostgreSQL 9.6.1 on x86_64-pc-linux-gnu, compiled by gcc (GCC) 6.2.1 20160830, 64-bit" }]));
        assert_eq!(buffer.len(), 20);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::CommandComplete("SELECT 1"));
        assert_eq!(buffer.len(), 0);

        let (next, buffer) = take_msg(buffer).unwrap();
        let msg = ServerMsg::from_slice(next).unwrap();
        assert_eq!(msg, ServerMsg::ReadyForQuery);
        assert_eq!(buffer.len(), 0);
    }
}
