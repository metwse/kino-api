mod client;
mod payload;
mod error;

pub use payload::IdToken;
pub use client::Client;
pub use error::Error;

const CERTS_URI: &str = "https://www.googleapis.com/oauth2/v3/certs";
