use crate::wallet::load::load_from_file;
use jsonwebkey as jwk;

use self::{
    hash::{Hasher, RingHasher},
    sign::{RsaSigner, Signer},
    verify::{RsaVerifier, Verifier},
};

pub mod base64;
pub mod deep_hash;
pub mod hash;
pub mod merkle;
pub mod sign;
pub mod verify;

pub trait Provider {
    fn get_signer(&self) -> &dyn Signer;
    fn get_verifier(&self) -> &dyn Verifier;
    fn get_hasher(&self) -> &dyn Hasher;
}

//TODO: implement Signer & Verifier using Ring only
pub struct RingProvider {
    pub signer: Box<RsaSigner>,
    pub verifier: Box<RsaVerifier>,
    pub hasher: Box<RingHasher>,
}

impl Default for RingProvider {
    fn default() -> Self {
        let jwk: jwk::JsonWebKey =
            load_from_file("res/test_wallet.json").expect("Error loading wallet");
        Self {
            signer: Box::new(RsaSigner::from_jwk(jwk)),
            verifier: Default::default(),
            hasher: Default::default(),
        }
    }
}

impl<'a> RingProvider {
    pub fn new(
        signer: Box<RsaSigner>,
        verifier: Box<RsaVerifier>,
        hasher: Box<RingHasher>,
    ) -> Self {
        RingProvider {
            signer,
            verifier,
            hasher,
        }
    }
}

impl Provider for RingProvider {
    fn get_signer(&self) -> &dyn Signer {
        &*self.signer
    }

    fn get_verifier(&self) -> &dyn Verifier {
        &*self.verifier
    }

    fn get_hasher(&self) -> &dyn Hasher {
        &*self.hasher
    }
}
