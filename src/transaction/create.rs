use crate::{crypto::hash::DeepHashItem, error::Error};

use super::{Base64, Tag, Transaction};

impl Transaction {
    pub fn new(
        data: Vec<u8>,
        other_tags: Option<Vec<Tag<Base64>>>,
        last_tx: Option<Base64>,
        price_terms: (u64, u64),
        auto_content_tag: bool,
    ) -> Self {
        todo!()
    }

    pub fn to_deep_hash_item(&self) -> Result<DeepHashItem, Error> {
        todo!()
    }
}
