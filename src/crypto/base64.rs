use std::str::FromStr;

use num::BigUint;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Base64(pub Vec<u8>);

impl Default for Base64 {
    fn default() -> Self {
        Base64(vec![])
    }
}

impl std::fmt::Display for Base64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string = &base64::display::Base64Display::with_config(&self.0, base64::URL_SAFE_NO_PAD);
        write!(f, "{}", string)
    }
}

impl From<&[u8]> for Base64 {
    fn from(u: &[u8]) -> Self {
        Base64(u.to_vec())
    }
}

impl FromStr for Base64 {
    type Err = base64::DecodeError;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let result = base64::decode_config(str, base64::URL_SAFE_NO_PAD)?;
        Ok(Self(result))
    }
}

impl Base64 {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn from_utf8_str(str: &str) -> Result<Self, Error> {
        Ok(Self(str.as_bytes().to_vec()))
    }
    pub fn to_utf8_string(&self) -> Result<String, Error> {
        Ok(String::from_utf8(self.0.clone()).expect("Could not convert from utf8"))
    }
}

impl Serialize for Base64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&format!("{}", &self))
    }
}

impl<'de> Deserialize<'de> for Base64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Vis;
        impl serde::de::Visitor<'_> for Vis {
            type Value = Base64;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a base64 string")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                base64::decode_config(v, base64::URL_SAFE_NO_PAD)
                    .map(Base64)
                    .map_err(|_| de::Error::custom("failed to decode base64 string"))
            }
        }
        deserializer.deserialize_str(Vis)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::crypto::base64::Base64;

    #[test]
    fn test_deserialize_base64() {
        let base_64 = Base64(vec![44; 7]);
        assert_eq!(base_64.0, vec![44; 7]);
        assert_eq!(format!("{}", base_64), "LCwsLCwsLA");

        let base_64: Base64 = serde_json::from_str("\"LCwsLCwsLA\"").unwrap();
        assert_eq!(base_64.0, vec![44; 7]);
        assert_eq!(format!("{}", base_64), "LCwsLCwsLA");
    }

    #[test]
    fn test_base64_convert_utf8() {
        let foo_b64 = Base64::from_utf8_str("foo").unwrap();
        assert_eq!(foo_b64.0, vec![102, 111, 111]);

        let foo_b64 = Base64(vec![102, 111, 111]);
        assert_eq!(foo_b64.to_utf8_string().unwrap(), "foo".to_string());
    }

    #[test]
    fn test_base64_convert_string() {
        let foo_b64 = Base64::from_str("LCwsLCwsLA").unwrap();
        assert_eq!(foo_b64.0, vec![44; 7]);

        let foo_b64 = Base64(vec![44; 7]);
        assert_eq!(foo_b64.to_string(), "LCwsLCwsLA".to_string());
    }
}
