use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArweaveError {
    #[error("Unknown Error.")]
    UnknownError,
}