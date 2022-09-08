use std::pin::Pin;

use async_recursion::async_recursion;
use bytes::Bytes;
use sha2::{Digest, Sha384};

use futures::{Stream, TryStreamExt};

use crate::error::Error;

const LIST_AS_BUFFER: &[u8] = "list".as_bytes();
const BLOB_AS_BUFFER: &[u8] = "blob".as_bytes();
pub const DATAITEM_AS_BUFFER: &[u8] = "dataitem".as_bytes();
pub const ONE_AS_BUFFER: &[u8] = "1".as_bytes();

pub enum DeepHashChunk {
    Chunk(Bytes),
    Stream(Pin<Box<dyn Stream<Item = anyhow::Result<Bytes>>>>),
    Chunks(Vec<DeepHashChunk>),
}

impl DeepHashChunk {
    pub fn from_item(item: &[u8]) -> DeepHashChunk {
        Self::Chunk(Bytes::copy_from_slice(item))
    }
    pub fn from_children(children: Vec<DeepHashChunk>) -> DeepHashChunk {
        Self::Chunks(children)
    }
}

pub async fn deep_hash(chunk: DeepHashChunk) -> Result<Bytes, Error> {
    match chunk {
        DeepHashChunk::Chunk(b) => {
            let tag = [BLOB_AS_BUFFER, b.len().to_string().as_bytes()].concat();
            let c = [sha384_hash(tag.into()), sha384_hash(b)].concat();
            Ok(Bytes::copy_from_slice(&sha384_hash(c.into())))
        }
        DeepHashChunk::Stream(mut s) => {
            let mut hasher = Sha384::new();
            let mut length = 0;
            while let Some(chunk) = s
                .as_mut()
                .try_next()
                .await
                .map_err(|_| Error::NoBytesLeft)?
            {
                length += chunk.len();
                hasher.update(&chunk);
            }

            let tag = [BLOB_AS_BUFFER, length.to_string().as_bytes()].concat();

            let tagged_hash = [
                sha384_hash(tag.into()),
                Bytes::copy_from_slice(&hasher.finalize()),
            ]
            .concat();

            Ok(sha384_hash(tagged_hash.into()))
        }
        DeepHashChunk::Chunks(chunks) => {
            // Be careful of truncation
            let len = chunks.len() as f64;
            let tag = [LIST_AS_BUFFER, len.to_string().as_bytes()].concat();

            let acc = sha384_hash(tag.into());

            deep_hash_chunks(chunks, acc).await
        }
    }
}

#[async_recursion(?Send)]
pub async fn deep_hash_chunks(mut chunks: Vec<DeepHashChunk>, acc: Bytes) -> Result<Bytes, Error> {
    if chunks.is_empty() {
        return Ok(acc);
    };

    let acc = Bytes::copy_from_slice(&acc);

    let hash_pair = [acc, deep_hash(chunks.remove(0)).await?].concat();
    let new_acc = sha384_hash(hash_pair.into());
    deep_hash_chunks(chunks, new_acc).await
}

fn sha384_hash(b: Bytes) -> Bytes {
    let mut hasher = Sha384::new();
    hasher.update(&b);
    Bytes::copy_from_slice(&hasher.finalize())
}
