mod client;
mod error;
mod payload;

pub use client::GoogleClient;
pub use error::Error;
pub use payload::GoogleIdToken;

const CERTS_URI: &str = "https://www.googleapis.com/oauth2/v3/certs";
