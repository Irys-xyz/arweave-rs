use std::str::FromStr;

use num::{BigRational, BigUint, ToPrimitive, Zero};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use crate::{
    crypto::base64::Base64,
    crypto::{
        self,
        deep_hash::{DeepHashItem, ToItems},
        hash::Hasher,
        merkle::{generate_data_root, generate_leaves, resolve_proofs, Node, Proof},
    },
    currency::Currency,
    error::Error,
    transaction::tags::Tag,
    VERSION,
};

use self::tags::FromUtf8Strs;

pub mod tags;

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

impl Serialize for Tx {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Tx", 12)?;
        s.serialize_field("format", &self.format)?;
        s.serialize_field("owner", &self.owner.to_string())?;
        s.serialize_field("target", &self.target.to_string())?;
        s.serialize_field("data_root", &self.data_root.to_string())?;
        s.serialize_field("data_size", &self.data_size.to_string())?;
        s.serialize_field("quantity", &self.quantity.to_formatted_string())?;
        s.serialize_field("reward", &self.reward.to_string())?;
        s.serialize_field("last_tx", &self.last_tx.to_string())?;
        s.serialize_field("tags", &self.tags)?;
        s.serialize_field("id", &self.id.to_string())?;
        s.serialize_field("data", &self.data.to_string())?;
        s.serialize_field("signature", &self.signature.to_string())?;

        s.end()
    }
}

pub trait Generator {
    fn new_w2w_tx(
        &self,
        crypto: &dyn crypto::Provider,
        target: Base64,
        data: Vec<u8>,
        quantity: u64,
        price_terms: (u64, u64),
        last_tx: Base64,
        other_tags: Vec<Tag<Base64>>,
        auto_content_tag: bool,
    ) -> Result<Tx, Error>;
}

impl<'a> ToItems<'a, Tx> for Tx {
    fn to_deep_hash_item(&'a self) -> Result<DeepHashItem, Error> {
        match &self.format {
            1 => {
                let quantity = Base64::from_utf8_str(&self.quantity.to_formatted_string()).unwrap();
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
                    self.quantity.to_formatted_string().as_bytes(),
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

    fn generate_merkle(hasher: &dyn Hasher, data: Vec<u8>) -> Result<Tx, Error> {
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
            let mut chunks = generate_leaves(&*hasher, data.clone()).unwrap();
            let root = generate_data_root(&*hasher, chunks.clone()).unwrap();
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

impl Generator for Tx {
    fn new_w2w_tx(
        &self,
        crypto: &dyn crypto::Provider,
        target: Base64,
        data: Vec<u8>,
        quantity: u64,
        price_terms: (u64, u64),
        last_tx: Base64,
        other_tags: Vec<Tag<Base64>>,
        auto_content_tag: bool,
    ) -> Result<Self, Error> {
        if quantity <= Zero::zero() {
            return Err(Error::InvalidValueForTx);
        }

        let mut transaction = Tx::generate_merkle(crypto.get_hasher(), data).unwrap();
        transaction.owner = crypto.keypair_modulus();

        let mut tags = vec![/* Tx::base_tag() */];

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
        transaction.last_tx =
            Base64::from_str("5jXeTrl978sxUBvODU2_18_eoXY29m8VII2ghDdP7SPBdAQMnshNkjqffZXAI9kp")
                .unwrap();

        transaction.reward = price_terms.0;
        transaction.quantity = Currency::from(quantity);
        transaction.target = target;

        Ok(transaction)
    }
}
