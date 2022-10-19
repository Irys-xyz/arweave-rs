use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error, Deserialize)]
pub enum Error {
    #[error("Error getting oracle price: {0}")]
    OracleGetPriceError(String),

    #[error("Getting Arweave price from oracle: {0}")]
    GetPriceError(String),

    #[error("Status code not Ok")]
    StatusCodeNotOk,

    #[error("Unsigned transaction")]
    UnsignedTransaction,

    #[error("Invalid proof")]
    InvalidProof,

    #[error("Slice error")]
    SliceError,

    #[error("Invalid tag encoding.")]
    InvalidValueForTx,

    #[error("Invalid tag encoding.")]
    InvalidTagEncoding,

    #[error("Error getting network info: {0}")]
    NetworkInfoError(String),

    #[error("No bytes left.")]
    NoBytesLeft,

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Error getting transaction info: {0}")]
    TransactionInfoError(String),

    #[error("Unknown Error.")]
    UnknownError,

    #[error("Error getting wallet: {0}")]
    WalletError(String),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Error posting chunk: {0}")]
    PostChunkError(String),

    #[error("Error signin: {0}")]
    SigningError(String),
}
