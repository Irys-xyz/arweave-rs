use rsa::RsaPrivateKey;

use self::{hash::Hasher, sign::Signer, verify::Verifier};

pub mod base64;
pub mod deep_hash;
pub mod hash;
pub mod merkle;
pub mod sign;
pub mod verify;

pub struct Provider {
    signer: Signer,
    verifier: Verifier,
    hasher: Hasher,
}
