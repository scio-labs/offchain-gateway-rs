
pub struct Database;

/// Connects to database
pub async fn bootstrap() -> Database {
    Database {}
}

impl Database {
    pub async fn text(&self, _name: &str, _key: &str) -> String {
        unimplemented!()
    }

    pub async fn addr(&self, _name: &str, _coin_type: u64) -> String {
        unimplemented!()
    }
}
