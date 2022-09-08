use rsa::RsaPrivateKey;

pub mod sign;
pub mod verify;

pub struct ArweaveSigner {
    priv_key: RsaPrivateKey,
}
