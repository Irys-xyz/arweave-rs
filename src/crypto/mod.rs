use std::{fs, path::PathBuf};

use self::{
    base64::Base64,
    deep_hash::DeepHashItem,
    hash::{Hasher, RingHasher},
    sign::Signer,
};

pub mod base64;
pub mod deep_hash;
pub mod hash;
pub mod merkle;
pub mod sign;

pub trait Provider {
    fn deep_hash(&self, deep_hash_item: DeepHashItem) -> [u8; 48];
    fn sign(&self, message: &[u8]) -> Vec<u8>;
    fn hash_sha256(&self, message: &[u8]) -> [u8; 32];
    fn pub_key(&self) -> Base64;
    fn get_hasher(&self) -> &dyn Hasher;
}

//TODO: implement Signer & Verifier using Ring only
pub struct RingProvider {
    pub signer: Box<Signer>,
    pub hasher: Box<RingHasher>,
}

impl Default for RingProvider {
    fn default() -> Self {
        let keypair_path = PathBuf::from(".wallet.json");
        Self {
            signer: Box::new(Signer::from_keypair_path_sync(keypair_path).unwrap()),
            hasher: Default::default(),
        }
    }
}

impl<'a> RingProvider {
    pub fn from_keypair_path(keypair_path: PathBuf) -> Self {
        let signer = Signer::from_keypair_path_sync(keypair_path).unwrap();
        RingProvider::new(Box::new(signer), Box::new(RingHasher::default()))
    }

    pub fn new(signer: Box<Signer>, hasher: Box<RingHasher>) -> Self {
        RingProvider {
            signer,
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
        self.signer.sign(message).expect("Valid message").to_vec()
    }

    fn hash_sha256(&self, message: &[u8]) -> [u8; 32] {
        self.hasher.hash_sha256(message)
    }

    fn pub_key(&self) -> Base64 {
        self.signer.keypair_modulus().unwrap()
    }
}
