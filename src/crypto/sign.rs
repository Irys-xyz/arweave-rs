//! Functionality for creating and verifying signatures and hashing.

use crate::{error::Error, wallet::load::load_from_file};
use data_encoding::BASE64URL;
use jsonwebkey as jwk;
use jsonwebkey::JsonWebKey;
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
        //TODO: implement new key generation
        let jwk_parsed = load_from_file("res/test_wallet.json").expect("Valid wallet file");
        Self {
            keypair: signature::RsaKeyPair::from_pkcs8(&jwk_parsed.key.as_ref().to_der()).unwrap(),
            sr: rand::SystemRandom::new(),
        }
    }
}

impl Signer {
    pub async fn from_keypair_path(keypair_path: PathBuf) -> Result<Signer, Error> {
        let data = fs::read_to_string(keypair_path)
            .await
            .expect("Could not open file");

        let jwk_parsed: JsonWebKey = data.parse().expect("Could not parse key");
        Ok(Self {
            keypair: signature::RsaKeyPair::from_pkcs8(&jwk_parsed.key.as_ref().to_der()).unwrap(),
            sr: rand::SystemRandom::new(),
        })
    }

    pub fn from_keypair_path_sync(keypair_path: PathBuf) -> Result<Signer, Error> {
        let data = fsSync::read_to_string(keypair_path).expect("Could not open file");

        let jwk_parsed: JsonWebKey = data.parse().expect("Could not parse key");
        Ok(Self {
            keypair: signature::RsaKeyPair::from_pkcs8(&jwk_parsed.key.as_ref().to_der()).unwrap(),
            sr: rand::SystemRandom::new(),
        })
    }

    pub fn public_key(&self) -> &[u8] {
        self.keypair.public_key().as_ref()
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

    pub fn verify(&self, pubk: &[u8], signature: &[u8], message: &[u8]) -> Result<(), Error> {
        let public_key =
            signature::UnparsedPublicKey::new(&signature::RSA_PSS_2048_8192_SHA256, pubk);
        match public_key.verify(message, signature) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::InvalidSignature),
        }
    }

    pub fn fill_rand(&self, dest: &mut [u8]) -> Result<(), Error> {
        let rand_bytes = self.sr.fill(dest).unwrap();
        Ok(rand_bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::{crypto::sign::Signer, error};

    #[test]
    fn test_default_keypair() {
        let path = PathBuf::from_str("res/test_wallet.json").unwrap();
        let provider = Signer::from_keypair_path_sync(path).expect("Valid wallet file");
        assert_eq!(
            provider.wallet_address().unwrap().to_string(),
            "ggHWyKn0I_CTtsyyt2OR85sPYz9OvKLd9DYIvRQ2ET4"
        );
    }

    #[test]
    fn test_sign_verify() -> Result<(), error::Error> {
        let message = b"hello";
        let provider = Signer::default();
        let signature = provider.sign(b"hello").unwrap();
        let pubk = provider.public_key();
        provider.verify(pubk, &signature, message)
    }
}
