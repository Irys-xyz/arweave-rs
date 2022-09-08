use crate::{
    crypto::{hash::DeepHashItem, ArweaveSigner},
    error::ArweaveError,
};

use super::{Transaction, UnsignedTransaction};

impl Transaction {
    pub fn from_unsigned_tx(signer: ArweaveSigner, utx: UnsignedTransaction) -> Self {
        todo!()
    }

    pub fn to_deep_hash_item(&self) -> Result<DeepHashItem, ArweaveError> {
        todo!()
    }
}
