use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error, Deserialize)]
pub enum Error {
    #[error("Error getting network info: {0}")]
    NetworkInfoError(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Error getting transaction info: {0}")]
    TransactionInfoError(String),

    #[error("Unknown Error.")]
    UnknownError,

    #[error("Error getting wallet: {0}")]
    WalletError(String),
}
