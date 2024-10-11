use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KinoToken {
    pub sub: i64,
    pub scope: Vec<KinoScope>,
    pub google_id: String,
    pub email: String,
    pub username: Option<String>,
    pub exp: u64,
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum KinoScope {
    Auth,
}
