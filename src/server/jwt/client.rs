use jsonwebtoken::{
    EncodingKey, DecodingKey,
    Header, Algorithm,
    Validation,
    encode, decode,
};

pub struct JWTClient {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey
}

use super::payload::KinoToken;

impl JWTClient {
    pub fn new(secret: &str) -> Self {
        let secret = secret.as_bytes();
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    pub fn encode(&self, payload: KinoToken) -> Option<String> {
        encode(&Header::default(), &payload, &self.encoding_key).ok()
    }

    pub fn decode(&self, payload: &str) -> Option<KinoToken> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        if let Some(token) = decode::<KinoToken>(&payload, &self.decoding_key, &validation).ok() {
            Some(token.claims)
        } else {
            None
        }
    }
}
