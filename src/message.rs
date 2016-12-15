trait Message {
    fn get_id(&self) -> Option<char>;
    fn serialize(&self) -> Vec<u8>;
}

enum TransactionStatus {
    Idle,
    Transaction,
    Failed,
}


// Message types

struct Startup {
    version: (u8, u8),
    user: &str,
    database: Option<&str>,
    params: Vec<(&str, &str)>,
}

impl Message for Startup {
    fn get_id(&self) -> Option<char> {
        None
    }
}

struct Terminate;

impl Message for Terminate {
    fn get_id(&self) -> Option<char> {
        Some('X')
    }
}

struct Query {
    query: &str,
}

impl Message for Query {
    fn get_id(&self) -> Option<char> {
        Some('Q')
    }
}

struct ReadyForQuery {
    status: TransactionStatus
}

impl Message for ReadyForQuery {
    fn get_id(&self) -> Option<char> {
        Some('Z')
    }
}

struct ParameterStatus {
    param: &str,
    value: &str,
}

impl Message for ParameterStatus {
    fn get_id(&self) -> Option<char> {
        Some('S')
    }
}
