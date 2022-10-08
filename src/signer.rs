use std::path::PathBuf;

use crate::{
    crypto::{
        self,
        base64::Base64,
        deep_hash::ToItems,
        RingProvider,
    },
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
    pub fn from_keypair_path(keypair_path: PathBuf) -> Result<ArweaveSigner, Error> {
        let crypto = RingProvider::from_keypair_path(keypair_path);
        let signer = ArweaveSigner {
            crypto: Box::new(crypto),
        };
        Ok(signer)
    }

    pub fn sign_transaction(&self, mut transaction: Tx) -> Result<Tx, Error> {
        let deep_hash_item = transaction.to_deep_hash_item().unwrap();
        let signature_data = self.crypto.deep_hash(deep_hash_item);
        let signature = self.crypto.sign(&signature_data);
        let id = self.crypto.hash_sha256(&signature);
        transaction.signature = Base64(signature);
        transaction.id = Base64(id.to_vec());
        Ok(transaction)
    }

    pub fn verify_transaction(&self, transaction: &Tx) -> Result<(), Error> {
        todo!();
        if transaction.signature.is_empty() {
            return Err(Error::UnsignedTransaction);
        }

        let deep_hash_item = transaction.to_deep_hash_item().unwrap();
        let data_to_sign = self.crypto.deep_hash(deep_hash_item);
        let signature = &transaction.signature.to_string();
        let sig_bytes = signature.as_bytes();
        let pubk = &transaction.owner;
        if self
            .crypto
            .verify(pubk.to_string().as_bytes(), sig_bytes, &data_to_sign)
        {
            Ok(())
        } else {
            Err(Error::InvalidSignature)
        }
    }

    fn hash_sha256(&self, message: &[u8]) -> [u8; 32] {
        self.crypto.hash_sha256(message)
    }

    pub fn keypair_modulus(&self) -> Base64 {
        self.crypto.keypair_modulus()
    }

    pub fn get_provider(&self) -> &dyn crypto::Provider {
        &*self.crypto
    }
}
