use avro_rs::{from_avro_datum, to_avro_datum, Schema};
use bytes::Bytes;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{crypto::hash::DeepHashChunk, error::Error};

use super::ToItems;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Tag {
    name: String,
    value: String,
}

impl Tag {
    pub fn new(name: String, value: String) -> Self {
        Tag { name, value }
    }
}

impl<'a> ToItems<'a, Tag> for Tag {
    fn to_deep_hash_chunk(&'a self) -> Result<DeepHashChunk, Error> {
        Ok(DeepHashChunk::Chunks(vec![
            DeepHashChunk::Chunk(Bytes::copy_from_slice(&self.name.as_bytes().to_vec())),
            DeepHashChunk::Chunk(Bytes::copy_from_slice(&self.value.as_bytes().to_vec())),
        ]))
    }
}

impl<'a> ToItems<'a, Vec<Tag>> for Vec<Tag> {
    fn to_deep_hash_chunk(&'a self) -> Result<DeepHashChunk, Error> {
        if self.len() > 0 {
            Ok(DeepHashChunk::Chunks(
                self.iter()
                    .map(|t| t.to_deep_hash_chunk().unwrap())
                    .collect(),
            ))
        } else {
            Ok(DeepHashChunk::Chunk(Bytes::new()))
        }
    }
}

pub trait AvroEncode {
    fn encode(&self) -> Result<Bytes, Error>;
}

pub trait AvroDecode {
    fn decode(&mut self) -> Result<Vec<Tag>, Error>;
}

const SCHEMA_STR: &str = r##"{
    "type": "array",
    "items": {
        "type": "record",
        "name": "Tag",
        "fields": [
            { "name": "name", "type": "string" },
            { "name": "value", "type": "string" }
        ]
    }
}"##;

lazy_static! {
    pub static ref TAGS_SCHEMA: Schema = Schema::parse_str(SCHEMA_STR).unwrap();
}

impl AvroEncode for Vec<Tag> {
    fn encode(&self) -> Result<Bytes, Error> {
        let v = avro_rs::to_value(self).unwrap();
        to_avro_datum(&TAGS_SCHEMA, v)
            .map(|v| v.into())
            .map_err(|_| Error::NoBytesLeft)
    }
}

impl AvroDecode for &mut [u8] {
    fn decode(&mut self) -> Result<Vec<Tag>, Error> {
        let x = self.to_vec();
        let v = from_avro_datum(&TAGS_SCHEMA, &mut x.as_slice(), Some(&TAGS_SCHEMA))
            .map_err(|_| Error::InvalidTagEncoding)?;
        avro_rs::from_value(&v).map_err(|_| Error::InvalidTagEncoding)
    }
}

impl From<avro_rs::DeError> for Error {
    fn from(_: avro_rs::DeError) -> Self {
        Error::InvalidTagEncoding
    }
}

#[cfg(test)]
mod tests {

    use crate::transaction::tags::{AvroDecode, AvroEncode};

    use super::Tag;

    #[test]
    fn test_bytes() {
        let b = &[2u8, 8, 110, 97, 109, 101, 10, 118, 97, 108, 117, 101, 0];

        let mut sli = &mut b.clone()[..];

        dbg!((sli).decode()).unwrap();
    }

    #[test]
    fn test_tags() {
        let tags = vec![Tag {
            name: "name".to_string(),
            value: "value".to_string(),
        }];

        dbg!(tags.encode().unwrap().to_vec());
    }
}
