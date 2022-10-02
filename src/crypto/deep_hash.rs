use crate::error::Error;

use super::hash::Hasher;

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
pub fn deep_hash(hasher: &dyn Hasher, deep_hash_item: DeepHashItem) -> [u8; 48] {
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
    use std::str::FromStr;

    use crate::{
        crypto::{
            base64::Base64,
            deep_hash::{deep_hash, ToItems},
            hash::RingHasher,
        },
        error::Error,
        transaction::Tx,
    };
    use serde_json::json;

    #[tokio::test]
    async fn test_deep_hash() -> Result<(), Error> {
        let hasher = RingHasher::new();

        let expected_tx = Tx {
            format: 2,
            id: Base64::from_str("Icwx2k4O3tZ4KcVQ77ARmLcANiCrpoeakXvaYtQsrUI").unwrap(),
            last_tx: Base64::from_str("5jXeTrl978sxUBvODU2_18_eoXY29m8VII2ghDdP7SPBdAQMnshNkjqffZXAI9kp").unwrap(),
            owner: Base64::from_str("pjdss8ZaDfEH6K6U7GeW2nxDqR4IP049fk1fK0lndimbMMVBdPv_hSpm8T8EtBDxrUdi1OHZfMhUixGaut-3nQ4GG9nM249oxhCtxqqNvEXrmQRGqczyLxuh-fKn9Fg--hS9UpazHpfVAFnB5aCfXoNhPuI8oByyFKMKaOVgHNqP5NBEqabiLftZD3W_lsFCPGuzr4Vp0YS7zS2hDYScC2oOMu4rGU1LcMZf39p3153Cq7bS2Xh6Y-vw5pwzFYZdjQxDn8x8BG3fJ6j8TGLXQsbKH1218_HcUJRvMwdpbUQG5nvA2GXVqLqdwp054Lzk9_B_f1lVrmOKuHjTNHq48w").unwrap(),
            tags: vec![],
            target: Base64::from_str("PAgdonEn9f5xd-UbYdCX40Sj28eltQVnxz6bbUijeVY").unwrap(),
            quantity: 100000,
            data_root: Base64::from_str("").unwrap(),
            data: Base64([].to_vec()),
            data_size: 0,
            reward: 1005003731,
            signature: Base64::from_str("kdvYjZy_gYrLuTcJwbtZsLjRsLIz1NmdqEYg-lMuX9q3eaBeB4gej0nWXQB16xtwikalnyyGagE8KM9HmItW_X9NbvRlwgF4QxVZeQKvn6hkBF-oThX57yO3MnAbo5euXgo3XJVdd8-40Yh6EwVAGJy-X80ZEBvTEvbD8zjy5YPR2YAQ9f63KtY7o5aZ-7w5prCWQNfAd_KB1E7E5EFqyD51X98a5IaacOB2wPRuTYXhDKkOg8VTx-4C6i3qrzvO_opVcitNoRMSOZLjPNPJ-xf4vh-XoCjH5kSlMWGvn3i67DPR9iX8Sdlp-t3NUsbuJMnlqBjG700HK6cZzBohXA").unwrap(),
            chunks: vec![],
            proofs: vec![],
        };

        let actual_hash = deep_hash(&hasher, expected_tx.to_deep_hash_item().unwrap());
        let correct_hash: [u8; 48] = [
            92, 69, 51, 135, 69, 123, 91, 178, 182, 70, 62, 91, 146, 71, 247, 59, 33, 208, 26, 136,
            141, 219, 43, 36, 129, 117, 174, 201, 197, 237, 248, 151, 36, 33, 151, 26, 203, 201,
            172, 245, 161, 182, 207, 56, 96, 119, 195, 102,
        ];

        assert_eq!(actual_hash, correct_hash);

        Ok(())
    }
}
