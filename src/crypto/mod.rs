use std::{fs, path::PathBuf};

use crate::wallet::load::load_from_file;
use bytes::Bytes;
use jsonwebkey as jwk;
use jwk::JsonWebKey;

use self::{
    deep_hash::DeepHashItem,
    hash::{Hasher, RingHasher},
    sign::{RsaSigner, Signer},
    verify::RsaVerifier,
};

pub mod base64;
pub mod deep_hash;
pub mod hash;
pub mod merkle;
pub mod sign;
pub mod verify;

pub trait Provider {
    fn deep_hash(&self, deep_hash_item: DeepHashItem) -> [u8; 48];
    fn sign(&self, message: &[u8]) -> Vec<u8>;
    fn hash_sha256(&self, message: &[u8]) -> [u8; 32];
    fn pub_key(&self) -> Bytes;
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
    pub fn from_keypair_path(keypair_path: PathBuf) -> Self {
        let data = fs::read_to_string(keypair_path).expect("Valid file path");
        let jwk_parsed: JsonWebKey = data.parse().unwrap();
        let signer = RsaSigner::from_jwk(jwk_parsed);
        RingProvider::new(
            Box::new(signer),
            Box::new(RsaVerifier::default()),
            Box::new(RingHasher::default()),
        )
    }

    pub fn new(
        signer: Box<RsaSigner>,
        verifier: Box<RsaVerifier>,
        hasher: Box<RingHasher>,
    ) -> Self {
        RingProvider {
            signer,
            verifier,
            hasher,
            ..Default::default()
        }
    }
}

impl Provider for RingProvider {
    fn get_hasher(&self) -> &dyn Hasher {
        &*self.hasher
    }

    fn deep_hash(&self, deep_hash_item: DeepHashItem) -> [u8; 48] {
        deep_hash::deep_hash(self.get_hasher(), deep_hash_item)
    }

    fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signer
            .sign(Bytes::copy_from_slice(message))
            .expect("Valid message")
            .to_vec()
    }

    fn hash_sha256(&self, message: &[u8]) -> [u8; 32] {
        self.hasher.hash_sha256(message)
    }

    fn pub_key(&self) -> Bytes {
        self.signer.pub_key()
    }
}
