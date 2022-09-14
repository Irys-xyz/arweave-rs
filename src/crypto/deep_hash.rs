use crate::error::Error;

use super::hash::Hasher;

pub enum DeepHashItem {
    Blob(Vec<u8>),
    List(Vec<DeepHashItem>),
}

impl DeepHashItem {
    pub fn from_item(item: &[u8]) -> DeepHashItem {
        Self::Blob(item.to_vec())
    }
    pub fn from_children(children: Vec<DeepHashItem>) -> DeepHashItem {
        Self::List(children)
    }
}

pub trait ToItems<'a, T> {
    fn to_deep_hash_item(&'a self) -> Result<DeepHashItem, Error>;
}

/// Calculates data root of transaction in accordance with implementation in [arweave-js](https://github.com/ArweaveTeam/arweave-js/blob/master/src/common/lib/deepHash.ts).
/// [`DeepHashItem`] is a recursive Enum that allows the function to be applied to
/// nested [`Vec<u8>`] of arbitrary depth.
pub fn deep_hash(hasher: &impl Hasher, deep_hash_item: DeepHashItem) -> [u8; 48] {
    let hash = match deep_hash_item {
        DeepHashItem::Blob(blob) => {
            let blob_tag = format!("blob{}", blob.len());
            hasher.hash_all_sha384(vec![blob_tag.as_bytes(), &blob])
        }
        DeepHashItem::List(list) => {
            let list_tag = format!("list{}", list.len());
            let mut hash = hasher.hash_sha384(list_tag.as_bytes());

            for child in list.into_iter() {
                let child_hash = deep_hash(hasher, child);
                hash = hasher.hash_sha384(&hasher.concat_u8_48(hash, child_hash));
            }
            hash
        }
    };
    hash
}

#[cfg(test)]
mod tests {
    use crate::{
        crypto::{
            deep_hash::{deep_hash, ToItems},
            hash::RingHasher,
        },
        error::Error,
        transaction::Tx,
    };

    #[tokio::test]
    async fn test_deep_hash() -> Result<(), Error> {
        let transaction = Tx {
            format: 2,
            ..Tx::default()
        };
        let hasher = RingHasher::new();
        let deep_hash = deep_hash(&hasher, transaction.to_deep_hash_item().unwrap());

        let correct_hash: [u8; 48] = [
            72, 43, 204, 204, 122, 20, 48, 138, 114, 252, 43, 128, 87, 244, 105, 231, 189, 246, 94,
            44, 150, 163, 165, 136, 133, 204, 158, 192, 28, 46, 222, 95, 55, 159, 23, 15, 3, 169,
            32, 27, 222, 153, 54, 137, 100, 159, 17, 247,
        ];

        assert_eq!(deep_hash, correct_hash);

        Ok(())
    }
}
