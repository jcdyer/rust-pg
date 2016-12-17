use std::str::FromStr;
use super::Result;
use super::error::PgError;

pub enum AuthMsg {
    AuthOk,
    AuthCleartext,
    AuthMd5(Vec<u8>),
    UnknownAuth,
}

impl FromStr for AuthMsg {
    type Err = PgError;
    fn from_str(input: &str) -> Result<AuthMsg> {
        if input[0] != 'R' {
            return PgError::Other;
        }
        Ok(match (input[5], input[6], input[7], input[8]) {
            (0, 0, 0, 0) => AuthMsg::AuthOk,
            (0, 0, 0, 3) => AuthMsg::AuthCleartext,
            (0, 0, 0, 5) => {
                let salt = Vec::with_capacity(4);
                salt.extend(&input[9 .. 13]);
                AuthMsg::AuthMd5(salt)
            },
            _ => AuthMsg::UnknownAuth,
        })
    }
}

/* Note: Authmsg source is not a str. It's a Vec<u8>.  Rewrite accordingly. */
