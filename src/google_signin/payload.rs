use serde::Deserialize;

/// Google ID Token
#[derive(Deserialize, Debug)]
pub struct GoogleIdToken {
    // These six fields are incuded in all Google ID tokens.
    /// issuer: Should equal to `accounts.google.com` or `https://accounts.google.com`.
    pub iss: String,
    /// subject: Unique account id.
    pub sub: String, 
    /// audience: Should equal to one of your app's client IDs
    pub aud: String,
    /// issued at: Unix timestamp when issued.
    pub iat: u64,
    /// expiration time: Unix timestamp that token expires
    pub exp: u64,

    /// hosted domain: Represents a Google Workspace or Cloud organization account.
    pub hd: Option<String>,

    
    // These seven fields are only included when the user has granted the "profile" and
    // "email" OAuth scopes to the application.
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub locale: Option<String>,
}
