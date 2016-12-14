
#[derive(Debug, Eq, PartialEq)]
pub struct Connection {
    url: String,
}

impl Connection {
    pub fn new(url: &str) -> Connection {
        Connection { url: url.to_string() }
    }
}
