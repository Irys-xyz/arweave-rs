//! Functionality for creating and verifying signatures and hashing.

use crate::error::Error;
use jsonwebkey as jwk;
use rand::thread_rng;
use rsa::{pkcs8::DecodePrivateKey, PaddingScheme, PublicKeyParts, RsaPrivateKey};
use sha2::Digest;
use std::{fs, path::PathBuf};

use super::base64::Base64;

/// Struct for for crypto methods.
pub struct Signer {
    priv_key: RsaPrivateKey,
}

impl Signer {
    fn new(priv_key: RsaPrivateKey) -> Self {
        Self { priv_key }
    }

    pub fn from_jwk(jwk: jwk::JsonWebKey) -> Self {
        let pem = jwk.key.to_pem();
        let priv_key = RsaPrivateKey::from_pkcs8_pem(&pem).unwrap();

        Self::new(priv_key)
    }

    pub fn from_keypair_path(keypair_path: PathBuf) -> Result<Self, Error> {
        let data = fs::read_to_string(keypair_path).expect("Could not open file");
        let jwk_parsed: jwk::JsonWebKey = data.parse().expect("Could not parse key");

        Ok(Self::from_jwk(jwk_parsed))
    }

    pub fn public_key(&self) -> Base64 {
        Base64(self.priv_key.to_public_key().n().to_bytes_be())
    }

    pub fn keypair_modulus(&self) -> Result<Base64, Error> {
        let modulus = self.priv_key.to_public_key().n().to_bytes_be();
        Ok(Base64(modulus.to_vec()))
    }

    pub fn wallet_address(&self) -> Result<Base64, Error> {
        let mut context = sha2::Sha256::new();
        context.update(&self.keypair_modulus()?.0[..]);
        let wallet_address = Base64(context.finalize().to_vec());
        Ok(wallet_address)
    }

    pub fn sign(&self, message: &[u8]) -> Result<Base64, Error> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&message);
        let hashed = hasher.finalize();

        let rng = thread_rng();
        let padding = PaddingScheme::PSS {
            salt_rng: Box::new(rng),
            digest: Box::new(sha2::Sha256::new()),
            salt_len: None,
        };

        let signature = self
            .priv_key
            .sign(padding, &hashed)
            .map_err(|e| Error::SigningError(e.to_string()))?;

        Ok(Base64(signature))
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::{
        crypto::{base64::Base64, sign::Signer},
        error,
    };

    const DEFAULT_WALLET_PATH: &str = "res/test_wallet.json";

    impl Default for Signer {
        fn default() -> Self {
            let path = PathBuf::from_str(DEFAULT_WALLET_PATH).unwrap();
            Self::from_keypair_path(path).expect("Could not create signer")
        }
    }

    #[test]
    fn test_default_keypair() {
        let path = PathBuf::from_str(DEFAULT_WALLET_PATH).unwrap();
        let provider = Signer::from_keypair_path(path).expect("Valid wallet file");
        assert_eq!(
            provider.wallet_address().unwrap().to_string(),
            "ggHWyKn0I_CTtsyyt2OR85sPYz9OvKLd9DYIvRQ2ET4"
        );
    }

    #[test]
    fn test_sign_verify() -> Result<(), error::Error> {
        let message = Base64(
            [
                74, 15, 74, 255, 248, 205, 47, 229, 107, 195, 69, 76, 215, 249, 34, 186, 197, 31,
                178, 163, 72, 54, 78, 179, 19, 178, 1, 132, 183, 231, 131, 213, 146, 203, 6, 99,
                106, 231, 215, 199, 181, 171, 52, 255, 205, 55, 203, 117,
            ]
            .to_vec(),
        );
        let provider = Signer::default();
        let signature = provider.sign(&message.0).unwrap();
        let pubk = provider.public_key();
        println!("pubk: {}", &pubk.to_string());
        println!("message: {}", &message.to_string());
        println!("sig: {}", &signature.to_string());

        //TODO: implement verification
        //provider.verify(&pubk.0, &message.0, &signature.0)
        Ok(())
    }
}
