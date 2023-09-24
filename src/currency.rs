use std::str::FromStr;

use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

use crate::error::Error;

/// Winstons are a sub unit of the native Arweave network token, AR. There are 10<sup>12</sup> Winstons per AR.
pub const WINSTONS_PER_AR: u64 = 1_000_000_000_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Currency {
    arweave: u64, //integer
    winston: u64, //decimal
}

impl From<u128> for Currency {
    fn from(u: u128) -> Self {
        let s = u.to_string();
        let mut arweave: u64 = 0;
        let winston: u64;
        if s.len() <= 12 {
            winston = u as u64;
        } else {
            let d = s.split_at(s.len() - 12);
            winston = (u % (WINSTONS_PER_AR as u128)) as u64;
            arweave = d.0.parse::<u64>().unwrap();
        }

        Self { arweave, winston }
    }
}

impl FromStr for Currency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let split: Vec<&str> = s.split('.').collect();
        if split.len() == 2 {
            Ok(Currency {
                arweave: split[0].parse::<u64>().map_err(Error::ParseIntError)?,
                winston: split[1].parse::<u64>().map_err(Error::ParseIntError)?,
            })
        } else {
            Ok(Currency {
                winston: split[0].parse::<u64>().map_err(Error::ParseIntError)?,
                ..Currency::default()
            })
        }
    }
}

impl ToString for Currency {
    fn to_string(&self) -> String {
        let decimal = format!("{:#012}", self.winston);
        if self.arweave == 0 && self.winston == 0 {
            '0'.to_string()
        } else if self.arweave == 0 {
            decimal.trim_start_matches('0').to_string()
        } else {
            self.arweave.to_string() + &decimal
        }
    }
}

//TODO: remove unwraps
impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Value::deserialize(deserializer)? {
            Value::String(s) => Currency::from_str(&s).expect("Could not deserialize"),
            Value::Number(num) => {
                Currency::from(num.as_u64().expect("Could not deserialize") as u128)
            }
            _ => return Err(de::Error::custom("Wrong type")),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Currency;

    #[test]
    fn test_str_parse() {
        let curr = Currency::from_str("1.000000000000").unwrap();
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 1);
        assert_eq!(curr.to_string(), "1000000000000");

        let curr = Currency::from_str("10.000000000000").unwrap();
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 10);
        assert_eq!(curr.to_string(), "10000000000000");

        let curr = Currency::from_str("999.000000000000").unwrap();
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 999);
        assert_eq!(curr.to_string(), "999000000000000");

        let curr = Currency::from_str("999.123123123123").unwrap();
        assert_eq!(curr.winston, 123123123123);
        assert_eq!(curr.arweave, 999);
        assert_eq!(curr.to_string(), "999123123123123");

        let curr = Currency::from_str("123123123123").unwrap();
        assert_eq!(curr.winston, 123123123123);
        assert_eq!(curr.arweave, 0);
        assert_eq!(curr.to_string(), "123123123123");

        let curr = Currency::from_str("10000").unwrap();
        assert_eq!(curr.winston, 10000);
        assert_eq!(curr.arweave, 0);
        assert_eq!(curr.to_string(), "10000");
    }

    #[test]
    fn test_u64_format() {
        let curr = Currency::from(1_000_000_000_000);
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 1);
        assert_eq!(curr.to_string(), "1000000000000");

        let curr = Currency::from(10_000_000_000_000);
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 10);
        assert_eq!(curr.to_string(), "10000000000000");

        let curr = Currency::from(999_000_000_000_000);
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 999);
        assert_eq!(curr.to_string(), "999000000000000");

        let curr = Currency::from(999_123_123_123_123);
        assert_eq!(curr.winston, 123123123123);
        assert_eq!(curr.arweave, 999);
        assert_eq!(curr.to_string(), "999123123123123");

        let curr = Currency::from(123_123_123_123);
        assert_eq!(curr.winston, 123123123123);
        assert_eq!(curr.arweave, 0);
        assert_eq!(curr.to_string(), "123123123123");

        let curr = Currency::from(10000);
        assert_eq!(curr.winston, 10000);
        assert_eq!(curr.arweave, 0);
        assert_eq!(curr.to_string(), "10000");
    }
}
