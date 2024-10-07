use std::fmt;

/// Validation error.
#[derive(Debug)]
pub enum Error {
    InvalidToken,
    KeyIdNotFound,
    MissingKeyId,
    InvalidHostedDomain,
    ClientNotInitialized,
    ValidationError(jsonwebtoken::errors::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToken => f.write_str("Invalid token signature."),
            Self::MissingKeyId => f.write_str("Header not included key id."),
            Self::KeyIdNotFound => f.write_str("Key id that provided by token is not found in decoding keys."),
            Self::InvalidHostedDomain => f.write_str("User is not on a permitted restricted domainuser is on permitted hosted domain."),
            Self::ClientNotInitialized => f.write_str("Decoding keys not initialized."),
            Self::ValidationError(validation_error) => validation_error.fmt(f),
        }
    }
}
