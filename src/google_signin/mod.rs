mod client;
mod payload;
mod error;

pub use payload::GoogleIdToken;
pub use client::GoogleClient;
pub use error::Error;

const CERTS_URI: &str = "https://www.googleapis.com/oauth2/v3/certs";
