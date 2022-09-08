use rsa::RsaPrivateKey;

pub mod base64;
pub mod hash;
pub mod merkle;
pub mod sign;
pub mod verify;

pub struct ArweaveSigner {
    priv_key: RsaPrivateKey,
}
