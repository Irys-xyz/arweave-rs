//! Functionality for creating and verifying signatures and hashing.

use crate::{error::Error, wallet::load::load_from_file};
use jsonwebkey::JsonWebKey;
use pretend::Json;
use ring::{
    digest::{Context, SHA256},
    rand::{self, SecureRandom},
    signature::{self, KeyPair, RsaKeyPair},
};
use std::fs as fsSync;
use std::path::PathBuf;
use tokio::fs;

use super::base64::Base64;

/// Struct for for crypto methods.
pub struct Signer {
    pub keypair: RsaKeyPair,
    pub sr: rand::SystemRandom,
}

impl Default for Signer {
    fn default() -> Self {
        let jwk_parsed = load_from_file(".wallet.json").expect("Valid wallet file");
        Self {
            keypair: signature::RsaKeyPair::from_pkcs8(&jwk_parsed.key.as_ref().to_der()).unwrap(),
            sr: rand::SystemRandom::new(),
        }
    }
}

impl Signer {
    pub async fn from_keypair_path(keypair_path: PathBuf) -> Result<Signer, Error> {
        let data = fs::read_to_string(keypair_path).await.unwrap();

        let jwk_parsed: JsonWebKey = data.parse().unwrap();
        Ok(Self {
            keypair: signature::RsaKeyPair::from_pkcs8(&jwk_parsed.key.as_ref().to_der()).unwrap(),
            sr: rand::SystemRandom::new(),
        })
    }

    pub fn from_keypair_path_sync(keypair_path: PathBuf) -> Result<Signer, Error> {
        let data = fsSync::read_to_string(keypair_path).unwrap();

        let jwk_parsed: JsonWebKey = data.parse().unwrap();
        Ok(Self {
            keypair: signature::RsaKeyPair::from_pkcs8(&jwk_parsed.key.as_ref().to_der()).unwrap(),
            sr: rand::SystemRandom::new(),
        })
    }

    pub fn keypair_modulus(&self) -> Result<Base64, Error> {
        let modulus = self
            .keypair
            .public_key()
            .modulus()
            .big_endian_without_leading_zero();
        Ok(Base64(modulus.to_vec()))
    }

    pub fn wallet_address(&self) -> Result<Base64, Error> {
        let mut context = Context::new(&SHA256);
        context.update(&self.keypair_modulus()?.0[..]);
        let wallet_address = Base64(context.finish().as_ref().to_vec());
        Ok(wallet_address)
    }

    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
        let mut signature = vec![0; self.keypair.public_modulus_len()];
        self.keypair
            .sign(
                &signature::RSA_PSS_SHA256,
                &self.sr,
                message,
                &mut signature,
            )
            .unwrap();
        Ok(signature)
    }

    pub fn verify(&self, signature: &[u8], message: &[u8]) -> Result<(), Error> {
        let public_key = signature::UnparsedPublicKey::new(
            &signature::RSA_PSS_2048_8192_SHA256,
            self.keypair.public_key().as_ref(),
        );
        public_key.verify(message, signature).unwrap();
        Ok(())
    }

    pub fn fill_rand(&self, dest: &mut [u8]) -> Result<(), Error> {
        let rand_bytes = self.sr.fill(dest).unwrap();
        Ok(rand_bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::sign::Signer;

    #[test]
    fn test_default_keypair() {
        let provider = Signer::default();
        assert_eq!(
            provider.wallet_address().unwrap().to_string(),
            "MKp3hwQJrL8gVIdOTkoZw-dOnALh4UiRKrA8vyTcfH8"
        );
    }
}
