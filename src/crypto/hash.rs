use sha2::Digest;

use crate::error::Error;

use super::utils::concat_u8_48;

pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut context = sha2::Sha256::new();
    context.update(message);
    let mut result: [u8; 32] = [0; 32];
    result.copy_from_slice(context.finalize().as_ref());
    result
}

pub fn sha384(message: &[u8]) -> [u8; 48] {
    let mut context = sha2::Sha384::new();
    context.update(message);
    let mut result: [u8; 48] = [0; 48];
    result.copy_from_slice(context.finalize().as_ref());
    result
}

/// Returns a SHA256 hash of the the concatenated SHA256 hashes of a vector of messages.
pub fn hash_all_sha256(messages: Vec<&[u8]>) -> [u8; 32] {
    let hash: Vec<u8> = messages
        .into_iter()
        .map(sha256)
        .into_iter()
        .flatten()
        .collect();
    sha256(&hash)
}

/// Returns a SHA384 hash of the the concatenated SHA384 hashes of a vector messages.
pub fn hash_all_sha384(messages: Vec<&[u8]>) -> [u8; 48] {
    let hash: Vec<u8> = messages
        .into_iter()
        .map(sha384)
        .into_iter()
        .flatten()
        .collect();
    sha384(&hash)
}

#[derive(Debug)]
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
pub fn deep_hash(deep_hash_item: DeepHashItem) -> [u8; 48] {
    let hash = match deep_hash_item {
        DeepHashItem::Blob(blob) => {
            let blob_tag = format!("blob{}", blob.len());
            hash_all_sha384(vec![blob_tag.as_bytes(), &blob])
        }
        DeepHashItem::List(list) => {
            let list_tag = format!("list{}", list.len());
            let mut hash = sha384(list_tag.as_bytes());

            for child in list.into_iter() {
                let child_hash = deep_hash(child);
                hash = sha384(&concat_u8_48(hash, child_hash));
            }
            hash
        }
    };
    hash
}
#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, str::FromStr};

    use crate::{
        crypto::hash::{deep_hash, ToItems},
        error::Error,
        transaction::Tx,
    };

    #[tokio::test]
    async fn test_deep_hash() -> Result<(), Error> {
        let mut file = File::open("res/sample_tx.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let tx = Tx::from_str(&data).unwrap();

        let actual_hash = deep_hash(tx.to_deep_hash_item().unwrap());
        let correct_hash: [u8; 48] = [
            74, 15, 74, 255, 248, 205, 47, 229, 107, 195, 69, 76, 215, 249, 34, 186, 197, 31, 178,
            163, 72, 54, 78, 179, 19, 178, 1, 132, 183, 231, 131, 213, 146, 203, 6, 99, 106, 231,
            215, 199, 181, 171, 52, 255, 205, 55, 203, 117,
        ];
        assert_eq!(actual_hash, correct_hash);

        Ok(())
    }
}
