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

pub struct Provider {
    pub signer: Box<Signer>,
    pub hasher: Box<RingHasher>,
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            signer: Box::new(Signer::default()),
            hasher: Default::default(),
        }
    }
}

impl<'a> Provider {
    pub fn from_keypair_path(keypair_path: PathBuf) -> Self {
        let signer = Signer::from_keypair_path(keypair_path)
            .expect("Could not create signer from keypair_path");
        Provider::new(Box::new(signer), Box::new(RingHasher::default()))
    }

    pub fn new(signer: Box<Signer>, hasher: Box<RingHasher>) -> Self {
        Provider {
            signer,
            hasher,
            ..Default::default()
        }
    }
}

impl Provider {
    pub fn get_hasher(&self) -> &dyn Hasher {
        &*self.hasher
    }

    pub fn deep_hash(&self, deep_hash_item: DeepHashItem) -> [u8; 48] {
        deep_hash::deep_hash(self.get_hasher(), deep_hash_item)
    }

    pub fn sign(&self, message: &[u8]) -> Base64 {
        self.signer.sign(message).expect("Valid message")
    }

    pub fn verify(&self, pub_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        match self.signer.verify(pub_key, message, signature) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn hash_sha256(&self, message: &[u8]) -> [u8; 32] {
        self.hasher.hash_sha256(message)
    }

    pub fn keypair_modulus(&self) -> Base64 {
        self.signer
            .keypair_modulus()
            .expect("Could not get keypair_modulus")
    }

    pub fn wallet_address(&self) -> Base64 {
        self.signer.wallet_address().expect("Could not get pub key")
    }

    pub fn public_key(&self) -> Base64 {
        self.signer.public_key()
    }
}

#[cfg(test)]
mod tests {
    use super::{base64::Base64, Provider};

    #[test]
    fn test_sign_verify() {
        let message = Base64(
            [
                9, 214, 233, 210, 242, 45, 194, 247, 28, 234, 14, 86, 105, 40, 41, 251, 52, 39,
                236, 214, 54, 13, 53, 254, 179, 53, 220, 205, 129, 37, 244, 142, 230, 32, 209, 103,
                68, 75, 39, 178, 10, 186, 24, 160, 179, 143, 211, 151,
            ]
            .to_vec(),
        );
        let provider = Provider::default();
        let signature = provider.sign(&message.0);
        let pubk = provider.public_key();
        assert!(provider.verify(&pubk.0, &message.0, &signature.0))
    }
}
