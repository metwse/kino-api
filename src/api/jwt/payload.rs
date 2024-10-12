use serde::{Serialize, Deserialize};


/// Kino JWT claims.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KinoIdToken {
    pub sub: i64,
    pub scope: Vec<KinoTokenScope>,
    pub google_id: String,
    pub email: String,
    pub username: Option<String>,
    pub exp: u64,
}

/// Scopes to that token capable to do.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum KinoTokenScope {
    Auth,
    // TODO: DeleteAccount,
    // TODO: ChangeEmail,
}
