use serde::{Deserialize, Serialize};

use crate::{
    crypto::{base64::Base64, deep_hash::DeepHashItem},
    error::Error,
};

use super::ToItems;

/// Transaction tag.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tag<T> {
    pub name: T,
    pub value: T,
}

/// Implemented to create [`Tag`]s from utf-8 strings.
pub trait FromUtf8Strs<T> {
    fn from_utf8_strs(name: &str, value: &str) -> Result<T, Error>;
}

impl FromUtf8Strs<Tag<Base64>> for Tag<Base64> {
    fn from_utf8_strs(name: &str, value: &str) -> Result<Self, Error> {
        let b64_name = Base64::from_utf8_str(name)?;
        let b64_value = Base64::from_utf8_str(value)?;

        Ok(Self {
            name: b64_name,
            value: b64_value,
        })
    }
}

impl FromUtf8Strs<Tag<String>> for Tag<String> {
    fn from_utf8_strs(name: &str, value: &str) -> Result<Self, Error> {
        let name = String::from(name);
        let value = String::from(value);

        Ok(Self { name, value })
    }
}

impl<'a> ToItems<'a, Vec<Tag<Base64>>> for Vec<Tag<Base64>> {
    fn to_deep_hash_item(&'a self) -> Result<DeepHashItem, Error> {
        if self.len() > 0 {
            Ok(DeepHashItem::List(
                self.iter()
                    .map(|t| t.to_deep_hash_item().unwrap())
                    .collect(),
            ))
        } else {
            Ok(DeepHashItem::Blob(Vec::<u8>::new()))
        }
    }
}

impl<'a> ToItems<'a, Tag<Base64>> for Tag<Base64> {
    fn to_deep_hash_item(&'a self) -> Result<DeepHashItem, Error> {
        Ok(DeepHashItem::List(vec![
            DeepHashItem::Blob(self.name.0.to_vec()),
            DeepHashItem::Blob(self.value.0.to_vec()),
        ]))
    }
}
