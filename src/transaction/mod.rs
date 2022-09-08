use serde::{Deserialize, Serialize};

use crate::{
    crypto::base64::Base64, crypto::hash::DeepHashChunk, error::Error, transaction::tags::Tag,
};

pub mod get;
pub mod tags;

pub trait ToItems<'a, T> {
    fn to_deep_hash_chunk(&'a self) -> Result<DeepHashChunk, Error>;
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Transaction {
    pub format: u8,
    pub id: Base64,
    pub last_tx: Base64,
    pub owner: Base64,
    pub tags: Vec<Tag>,
    pub target: Base64,
    pub quantity: u64,
    pub data: Base64,
    pub data_root: Base64,
    pub data_size: u64,
    pub reward: u64,
    pub signature: Base64,
}

impl<'a> ToItems<'a, Transaction> for Transaction {
    fn to_deep_hash_chunk(&'a self) -> Result<DeepHashChunk, Error> {
        match &self.format {
            1 => {
                let quantity = Base64::from_utf8_str(&self.quantity.to_string()).unwrap();
                let reward = Base64::from_utf8_str(&self.reward.to_string()).unwrap();
                let mut children: Vec<DeepHashChunk> = vec![
                    &self.owner,
                    &self.target,
                    &self.data,
                    &quantity,
                    &reward,
                    &self.last_tx,
                ]
                .into_iter()
                .map(|op| DeepHashChunk::from_item(&op.0))
                .collect();
                children.push(self.tags.to_deep_hash_chunk()?);

                Ok(DeepHashChunk::from_children(children))
            }
            2 => {
                let mut children: Vec<DeepHashChunk> = vec![
                    self.format.to_string().as_bytes(),
                    &self.owner.0,
                    &self.target.0,
                    self.quantity.to_string().as_bytes(),
                    self.reward.to_string().as_bytes(),
                    &self.last_tx.0,
                ]
                .into_iter()
                .map(DeepHashChunk::from_item)
                .collect();
                children.push(self.tags.to_deep_hash_chunk()?);
                children.push(DeepHashChunk::from_item(
                    self.data_size.to_string().as_bytes(),
                ));
                children.push(DeepHashChunk::from_item(&self.data_root.0));

                Ok(DeepHashChunk::from_children(children))
            }
            _ => unreachable!(),
        }
    }
}

impl Transaction {
    pub fn new(
        data: Vec<u8>,
        other_tags: Option<Vec<Tag>>,
        last_tx: Option<Base64>,
        price_terms: (u64, u64),
        auto_content_tag: bool,
    ) -> Self {
        todo!()
    }
}
