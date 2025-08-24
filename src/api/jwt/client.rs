use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

/// Kino JWT manager for creating and validating JWT's.
pub struct KinoClient {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

pub use super::KinoIdToken;

impl KinoClient {
    /// Creates a [`KinoClient`]
    pub fn new(secret: &str) -> Self {
        let secret = secret.as_bytes();
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    /// Encodes [`KinoIdToken`] to `jwt`
    pub fn encode(&self, payload: KinoIdToken) -> Option<String> {
        encode(&Header::default(), &payload, &self.encoding_key).ok()
    }

    /// Decodes [`KinoIdToken`] from `jwt`
    pub fn decode(&self, payload: &str) -> Option<KinoIdToken> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        if let Ok(token) = decode::<KinoIdToken>(payload, &self.decoding_key, &validation) {
            Some(token.claims)
        } else {
            None
        }
    }
}
