use crate::{
    crypto::{self, base64::Base64},
    error::Error,
};

use super::{tags::Tag, Tx};

pub trait Generator {
    fn new_tx(
        &self,
        crypto: &dyn crypto::Provider,
        target: Base64,
        data: Vec<u8>,
        quantity: u64,
        price_terms: u64,
        last_tx: Base64,
        other_tags: Vec<Tag<Base64>>,
        auto_content_tag: bool,
    ) -> Result<Tx, Error>;
}
