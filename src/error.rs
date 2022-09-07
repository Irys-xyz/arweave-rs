use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error, Deserialize)]
pub enum ArweaveError {
    #[error("Error getting transaction info: {0}")]
    TransactionInfoError(String),

    #[error("Error getting network info: {0}")]
    NetworkInfoError(String),

    #[error("Unknown Error.")]
    UnknownError,
}
