use super::GoogleIdToken;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    DecodingKey, Validation,
};

type DecodingKeyList = Arc<Mutex<Option<Vec<(DecodingKey, String)>>>>;

/// Google ID Token validator.
pub struct GoogleClient {
    audiences: Vec<String>,
    allowed_hosted_domains: Vec<String>,
    validate_hosted_domains: bool,
    decoding_keys: DecodingKeyList,
}

impl GoogleClient {
    pub fn new(audiences: Vec<String>, allowed_hosted_domains: Vec<String>) -> Self {
        Self {
            audiences,
            allowed_hosted_domains,
            validate_hosted_domains: true,
            decoding_keys: Arc::new(Mutex::new(None)),
        }
    }

    /// Spawns a background task and updates `decoding_keys` regularly.
    pub async fn init(&mut self) {
        let (tx, mut rx) = tokio::sync::oneshot::channel::<u8>();

        let decoding_keys = Arc::clone(&self.decoding_keys);
        tokio::spawn(async move {
            let mut tx = Some(tx);
            loop {
                if let Ok(request) = reqwest::get(super::CERTS_URI).await {
                    // parse Cache-Control header as std::time::Duration
                    let max_age = if let Some(cache_control_value) = {
                        if let Some(header) = request.headers().get("Cache-Control") {
                            header.to_str().ok()
                        } else {
                            None
                        }
                    } {
                        if let Some(cache_control) =
                            cache_control::CacheControl::from_value(cache_control_value)
                        {
                            cache_control.max_age
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Ok(text) = request.text().await {
                        let jwks: JwkSet =
                            serde_json::from_str(&text[..]).expect("Invalid certs URI.");

                        *decoding_keys.lock().unwrap() = Some(
                            jwks.keys
                                .iter()
                                .map(|jwk| {
                                    (
                                        match &jwk.algorithm {
                                            AlgorithmParameters::RSA(rsa) => {
                                                DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                                                    .unwrap()
                                            }
                                            _ => unreachable!("Algorithm should be RSA."),
                                        },
                                        jwk.common.key_id.clone().unwrap_or_else(|| {
                                            unreachable!("Key id must be present.")
                                        }),
                                    )
                                })
                                .collect::<Vec<(DecodingKey, String)>>(),
                        );

                        // notify main thread when first certs received
                        if let Some(tx) = tx.take() {
                            tracing::info!("first JWK certs received");
                            tx.send(1).unwrap();
                        } else {
                            tracing::info!("JWK certs reloaded");
                        }

                        tokio::time::sleep(max_age.unwrap_or(Duration::from_secs(10))).await;
                    }
                } else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        });

        // wait until first certificates
        while rx.try_recv().is_err() {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    /// Validates Google's JWT
    pub fn validate(&self, token: &str) -> Result<GoogleIdToken, super::Error> {
        let Ok(header) = decode_header(token) else {
            return Err(super::Error::InvalidHeader);
        };

        // token must include key_id
        let Some(token_key_id) = header.kid else {
            return Err(super::Error::MissingKeyId);
        };

        let Some(ref decoding_keys) = *self.decoding_keys.lock().unwrap() else {
            return Err(super::Error::ClientNotInitialized);
        };

        let decoding_key = 'decoding_key: {
            for (decoding_key, key_id) in decoding_keys {
                if *key_id == token_key_id {
                    break 'decoding_key decoding_key;
                }
            }
            return Err(super::Error::KeyIdNotFound);
        };

        let validation = {
            let mut validation = Validation::new(header.alg);
            validation.set_audience(&self.audiences);
            validation.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);
            validation
        };

        let decoded_token = match decode::<GoogleIdToken>(token, decoding_key, &validation) {
            Ok(decoded_token) => {
                let decoded_token = decoded_token.claims;
                if let Some(ref hosted_domain) = decoded_token.hd {
                    if self.validate_hosted_domains
                        && !self.allowed_hosted_domains.contains(hosted_domain)
                    {
                        return Err(super::Error::InvalidHostedDomain);
                    }
                }
                decoded_token
            }
            Err(validation_error) => return Err(super::Error::ValidationError(validation_error)),
        };

        Ok(decoded_token)
    }
}
