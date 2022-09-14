use serde::{Deserialize, Serialize};

use crate::{
    crypto::base64::Base64,
    crypto::{
        deep_hash::DeepHashItem,
        merkle::{generate_data_root, generate_leaves, resolve_proofs, Node, Proof},
    },
    error::Error,
    transaction::tags::Tag,
    BLOCK_SIZE, VERSION,
};

use self::tags::FromUtf8Strs;

pub mod get;
pub mod tags;

pub trait ToItems<'a, T> {
    fn to_deep_hash_item(&'a self) -> Result<DeepHashItem, Error>;
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Tx {
    /* Fields required for signing */
    pub format: u8,
    pub owner: Base64,
    pub target: Base64,
    pub data_root: Base64,
    pub data_size: u64,
    pub quantity: u64,
    pub reward: u64,
    pub last_tx: Base64,
    pub tags: Vec<Tag<Base64>>,

    /* Fields generated after signing */
    pub id: Base64,
    pub data: Base64,
    pub signature: Base64,
    #[serde(skip)]
    pub chunks: Vec<Node>,
    #[serde(skip)]
    pub proofs: Vec<Proof>,
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
            let empty_string = Base64::from_utf8_str("").expect("Empty string");
            Ok(Tx {
                format: 2,
                data_size: 0,
                data: empty_string.clone(),
                data_root: empty_string,
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

    pub fn new(
        owner: Base64,
        data: Vec<u8>,
        other_tags: Vec<Tag<Base64>>,
        last_tx: Base64,
        price_terms: (u64, u64),
        auto_content_tag: bool,
    ) -> Result<Self, Error> {
        let mut transaction = Tx {
            owner,
            last_tx,
            ..Self::generate_merkle(data).expect("Valid data")
        };

        let mut tags = vec![Self::base_tag()];
        if auto_content_tag {
            let content_type = if let Some(kind) = infer::get(&transaction.data.0) {
                kind.mime_type()
            } else {
                "application/octet-stream"
            };

            tags.push(
                Tag::<Base64>::from_utf8_strs("Content-Type", content_type)
                    .expect("Valid tag data"),
            )
        };
        tags.extend(other_tags);
        transaction.tags = tags;

        let blocks_len =
            transaction.data_size / BLOCK_SIZE + (transaction.data_size % BLOCK_SIZE != 0) as u64;
        let reward = price_terms.0 + price_terms.1 * (blocks_len - 1);
        transaction.reward = reward;

        Ok(transaction)
    }
}
