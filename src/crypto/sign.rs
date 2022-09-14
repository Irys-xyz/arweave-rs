use bytes::Bytes;
use jsonwebkey as jwk;
use rand::thread_rng;
use rsa::{pkcs8::DecodePrivateKey, PaddingScheme, PublicKeyParts, RsaPrivateKey};
use sha2::Digest;

use crate::error::Error;

const SIG_LENGTH: u16 = 512;
const PUB_LENGTH: u16 = 512;

pub trait Signer {
    fn sign(&self, message: Bytes) -> Result<Bytes, Error>;
    fn get_sig_length(&self) -> u16;
    fn get_pub_length(&self) -> u16;
    fn pub_key(&self) -> Bytes;
}

pub struct RsaSigner {
    priv_key: RsaPrivateKey,
}

impl RsaSigner {
    pub fn new(priv_key: RsaPrivateKey) -> Self {
        Self { priv_key }
    }

    pub fn from_jwk(jwk: jwk::JsonWebKey) -> Self {
        let pem = jwk.key.to_pem();
        let priv_key = RsaPrivateKey::from_pkcs8_pem(&pem).unwrap();

        Self::new(priv_key)
    }
}

#[allow(unused)]
impl Signer for RsaSigner {
    fn sign(&self, message: Bytes) -> Result<Bytes, Error> {
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
            .map_err(|e| Error::CryptoError(e.to_string()))?;

        Ok(signature.into())
    }

    fn pub_key(&self) -> Bytes {
        self.priv_key.to_public_key().n().to_bytes_be().into()
    }

    fn get_sig_length(&self) -> u16 {
        SIG_LENGTH
    }

    fn get_pub_length(&self) -> u16 {
        PUB_LENGTH
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use jsonwebkey as jwk;

    use crate::{
        crypto::{
            sign::{RsaSigner, Signer},
            verify::{RsaVerifier, Verifier},
        },
        wallet::load::load_from_file,
    };

    #[test]
    fn should_sign_and_verify() {
        let msg = Bytes::copy_from_slice(b"Hello, Arweave!");
        let jwk: jwk::JsonWebKey =
            load_from_file("res/test_wallet.json").expect("Error loading wallet");
        let signer = RsaSigner::from_jwk(jwk);

        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();

        let verifier = RsaVerifier::default();
        assert!(verifier.verify(pub_key, msg.clone(), sig).is_ok());
    }
}
