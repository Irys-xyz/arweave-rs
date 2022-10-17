use std::path::PathBuf;

use jsonwebkey::JsonWebKey;
use ring::signature;

use crate::{
    crypto::{self, base64::Base64, deep_hash::ToItems, Provider, RingProvider},
    error::Error,
    transaction::Tx,
};

pub struct ArweaveSigner {
    crypto: Box<dyn crypto::Provider + Send + Sync>,
}

impl Default for ArweaveSigner {
    fn default() -> Self {
        Self {
            crypto: Box::new(RingProvider::default()),
        }
    }
}

impl ArweaveSigner {
    pub fn verify(pub_key: &[u8], message: &[u8], signature: &[u8]) -> Result<(), Error> {
        let crypto = RingProvider::default();
        match crypto.verify(pub_key, message, signature) {
            true => Ok(()),
            false => Err(Error::InvalidSignature),
        }
    }

    pub fn from_keypair_path(keypair_path: PathBuf) -> Result<ArweaveSigner, Error> {
        let crypto = RingProvider::from_keypair_path(keypair_path);
        let signer = ArweaveSigner {
            crypto: Box::new(crypto),
        };
        Ok(signer)
    }

    pub fn sign_transaction(&self, mut transaction: Tx) -> Result<Tx, Error> {
        let deep_hash_item = transaction
            .to_deep_hash_item()
            .expect("Could not convert transaction into deep hash item");
        let signature_data = self.crypto.deep_hash(deep_hash_item);
        let signature = self.crypto.sign(&signature_data);
        let id = self.crypto.hash_sha256(&signature.0);
        transaction.signature = signature;
        transaction.id = Base64(id.to_vec());
        Ok(transaction)
    }

    pub fn sign(&self, message: &[u8]) -> Base64 {
        self.crypto.sign(message)
    }

    pub fn verify_transaction(transaction: &Tx) -> Result<(), Error> {
        if transaction.signature.is_empty() {
            return Err(Error::UnsignedTransaction);
        }

        let crypto = RingProvider::default();
        let deep_hash_item = transaction
            .to_deep_hash_item()
            .expect("Could not convert transaction into deep hash item");
        let message = crypto.deep_hash(deep_hash_item);
        let signature = &transaction.signature;
        let jwt_str = format!(
            "{{\"kty\":\"RSA\",\"e\":\"AQAB\",\"n\":\"{}\"}}",
            transaction.owner.to_string()
        );
        let jwk: JsonWebKey = jwt_str.parse()
            .expect("Could not parse JsonWebKey");
        let public_key =
            signature::UnparsedPublicKey::new(&signature::RSA_PKCS1_2048_8192_SHA256, transaction.owner.0.clone());

        println!("pubk: {:?}", transaction.owner.to_string());
        println!("message: {}", Base64(message.to_vec()).to_string());
        println!("sig: {}", &signature.to_string());
        public_key.verify(&message, &signature.0)
            .map_err(|_| Error::InvalidSignature)
    }

    fn hash_sha256(&self, message: &[u8]) -> [u8; 32] {
        self.crypto.hash_sha256(message)
    }

    pub fn wallet_address(&self) -> Base64 {
        self.crypto.wallet_address()
    }

    pub fn keypair_modulus(&self) -> Base64 {
        self.crypto.keypair_modulus()
    }

    pub fn get_provider(&self) -> &dyn crypto::Provider {
        &*self.crypto
    }

    pub fn get_public_key(&self) -> Base64 {
        self.crypto.public_key()
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    use super::{Base64, ArweaveSigner};

    #[test]
    fn test_sign_verify() -> Result<(), Error> {
        let message = Base64([74, 15, 74, 255, 248, 205, 47, 229, 107, 195, 69, 76, 215, 249, 34, 186, 197, 31, 178, 163, 72, 54, 78, 179, 19, 178, 1, 132, 183, 231, 131, 213, 146, 203, 6, 99, 106, 231, 215, 199, 181, 171, 52, 255, 205, 55, 203, 117].to_vec());
        let signer = ArweaveSigner::default();
        let signature = signer.sign(&message.0);
        let pubk = signer.get_public_key();
        ArweaveSigner::verify(&pubk.0, &message.0, &signature.0)
    }
}