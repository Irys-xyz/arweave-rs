use serde::{Deserialize, Serialize};

use crate::{
    crypto::{base64::Base64, Provider},
    crypto::{
        hash::{DeepHashItem, ToItems},
        merkle::{generate_data_root, generate_leaves, resolve_proofs, Node, Proof},
    },
    currency::Currency,
    error::Error,
    transaction::tags::Tag,
    VERSION,
};

use self::tags::FromUtf8Strs;

pub mod client;
pub mod parser;
pub mod tags;

#[derive(Deserialize, Debug, Default, PartialEq)]
struct JsonTx {
    pub format: u8,
    pub id: Base64,
    pub last_tx: Base64,
    pub owner: Base64,
    pub tags: Vec<Tag<Base64>>,
    pub target: Base64,
    pub quantity: String,
    pub data_root: Base64,
    pub data: Base64,
    pub data_size: String,
    pub reward: String,
    pub signature: Base64,
}
#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct Tx {
    /* Fields required for signing */
    pub format: u8,
    pub id: Base64,
    pub last_tx: Base64,
    pub owner: Base64,
    pub tags: Vec<Tag<Base64>>,
    pub target: Base64,
    pub quantity: Currency,
    pub data_root: Base64,
    pub data: Base64,
    pub data_size: u64,
    pub reward: u64,
    pub signature: Base64,
    #[serde(skip)]
    pub chunks: Vec<Node>,
    #[serde(skip)]
    pub proofs: Vec<Proof>,
}

/// Chunk data structure per [Arweave chunk spec](https://docs.arweave.org/developers/server/http-api#upload-chunks).
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Chunk {
    data_root: Base64,
    data_size: u64,
    data_path: Base64,
    pub offset: usize,
    chunk: Base64,
}

impl<'a> ToItems<'a, Tx> for Tx {
    fn to_deep_hash_item(&'a self) -> Result<DeepHashItem, Error> {
        match &self.format {
            1 => {
                let quantity = Base64::from_utf8_str(&self.quantity.to_string()).unwrap();
                let reward = Base64::from_utf8_str(&self.reward.to_string()).unwrap();
                let mut children: Vec<DeepHashItem> = vec![
                    &self.owner,
                    &self.target,
                    &self.data,
                    &quantity,
                    &reward,
                    &self.last_tx,
                ]
                .into_iter()
                .map(|op| DeepHashItem::from_item(&op.0))
                .collect();
                children.push(self.tags.to_deep_hash_item()?);

                Ok(DeepHashItem::from_children(children))
            }
            2 => {
                let mut children: Vec<DeepHashItem> = vec![
                    self.format.to_string().as_bytes(),
                    &self.owner.0,
                    &self.target.0,
                    self.quantity.to_string().as_bytes(),
                    self.reward.to_string().as_bytes(),
                    &self.last_tx.0,
                ]
                .into_iter()
                .map(DeepHashItem::from_item)
                .collect();
                children.push(self.tags.to_deep_hash_item().unwrap());
                children.push(DeepHashItem::from_item(
                    self.data_size.to_string().as_bytes(),
                ));
                children.push(DeepHashItem::from_item(&self.data_root.0));

                Ok(DeepHashItem::from_children(children))
            }
            _ => unreachable!(),
        }
    }
}

impl Tx {
    fn base_tag() -> Tag<Base64> {
        Tag::<Base64>::from_utf8_strs("User-Agent", &format!("arweave-rs/{}", VERSION)).unwrap()
    }

    fn generate_merkle(data: Vec<u8>) -> Result<Tx, Error> {
        if data.is_empty() {
            let empty = Base64(vec![]);
            Ok(Tx {
                format: 2,
                data_size: 0,
                data: empty.clone(),
                data_root: empty,
                chunks: vec![],
                proofs: vec![],
                ..Default::default()
            })
        } else {
            let mut chunks = generate_leaves(data.clone()).unwrap();
            let root = generate_data_root(chunks.clone()).unwrap();
            let data_root = Base64(root.id.clone().into_iter().collect());
            let mut proofs = resolve_proofs(root, None).unwrap();

            // Discard the last chunk & proof if it's zero length.
            let last_chunk = chunks.last().unwrap();
            if last_chunk.max_byte_range == last_chunk.min_byte_range {
                chunks.pop();
                proofs.pop();
            }

            Ok(Tx {
                format: 2,
                data_size: data.len() as u64,
                data: Base64(data),
                data_root,
                chunks,
                proofs,
                ..Default::default()
            })
        }
    }
}

impl Tx {
    pub fn new(
        crypto: &Provider,
        target: Base64,
        data: Vec<u8>,
        quantity: u128,
        fee: u64,
        last_tx: Base64,
        other_tags: Vec<Tag<Base64>>,
        auto_content_tag: bool,
    ) -> Result<Self, Error> {
        if quantity.lt(&0) {
            return Err(Error::InvalidValueForTx);
        }

        let mut transaction = Tx::generate_merkle(data).unwrap();
        transaction.owner = crypto.keypair_modulus();

        let mut tags = vec![Tx::base_tag()];

        // Get content type from [magic numbers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types)
        // and include additional tags if any.
        if auto_content_tag {
            let content_type = if let Some(kind) = infer::get(&transaction.data.0) {
                kind.mime_type()
            } else {
                "application/octet-stream"
            };

            tags.push(Tag::<Base64>::from_utf8_strs("Content-Type", content_type)?)
        }

        // Add other tags if provided.
        tags.extend(other_tags);
        transaction.tags = tags;

        // Fetch and set last_tx if not provided (primarily for testing).
        transaction.last_tx = last_tx;

        transaction.reward = fee;
        transaction.quantity = Currency::from(quantity);
        transaction.target = target;

        Ok(transaction)
    }

    pub fn clone_with_no_data(&self) -> Result<Self, Error> {
        Ok(Self {
            format: self.format,
            id: self.id.clone(),
            last_tx: self.last_tx.clone(),
            owner: self.owner.clone(),
            tags: self.tags.clone(),
            target: self.target.clone(),
            quantity: self.quantity,
            data_root: self.data_root.clone(),
            data: Base64::default(),
            data_size: self.data_size,
            reward: self.reward,
            signature: self.signature.clone(),
            chunks: Vec::new(),
            proofs: Vec::new(),
        })
    }

    pub fn get_chunk(&self, idx: usize) -> Result<Chunk, Error> {
        Ok(Chunk {
            data_root: self.data_root.clone(),
            data_size: self.data_size,
            data_path: Base64(self.proofs[idx].proof.clone()),
            offset: self.proofs[idx].offset,
            chunk: Base64(
                self.data.0[self.chunks[idx].min_byte_range..self.chunks[idx].max_byte_range]
                    .to_vec(),
            ),
        })
    }
}
