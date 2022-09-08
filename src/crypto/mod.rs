use rsa::RsaPrivateKey;

pub mod hash;
pub mod merkle;
pub mod sign;
pub mod verify;

pub struct ArweaveSigner {
    priv_key: RsaPrivateKey,
}
