use std::string::FromUtf8Error;

use thiserror::Error;
use url::ParseError;

#[derive(Debug, Error)]
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

    #[error("Error signing: {0}")]
    SigningError(String),

    #[error("No field present: {0}")]
    NoneError(String), //TODO: add option::NoneError implementation when released

    #[error("Io Error")]
    IoError(std::io::Error),

    #[error("ParseIntError")]
    ParseIntError(std::num::ParseIntError),

    #[error("UrlParseError")]
    UrlParseError(ParseError),

    #[error("FromUtf8Error")]
    FromUtf8Error(FromUtf8Error),

    #[error("FromUtf8Error")]
    JsonWebKeyError(jsonwebkey::Error),

    #[error("ReqwestError")]
    ReqwestError(reqwest::Error),

    #[error("DecodeError")]
    Base64DecodeError(base64::DecodeError),

    #[error("SerdeJsonError")]
    SerdeJsonError(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
