use std::str::FromStr;

use serde::Deserialize;

use crate::error::Error;

/// Winstons are a sub unit of the native Arweave network token, AR. There are 10<sup>12</sup> Winstons per AR.
pub const WINSTONS_PER_AR: u64 = 1_000_000_000_000;

#[derive(Deserialize, Debug, Default, PartialEq)]
pub struct Currency {
    arweave: u64, //integer
    winston: u64, //decimal
}

impl From<u64> for Currency {
    fn from(u: u64) -> Self {
        let s = u.to_string();
        let mut arweave: u64 = 0;
        let mut winston: u64 = 0;
        if s.len() <= 12 {
            winston = u as u64;
        } else {
            let d = s.split_at(s.len() - 12);
            winston = (u % WINSTONS_PER_AR) as u64;
            arweave = u64::from_str_radix(d.0, 10).unwrap();
        }

        Self { arweave, winston }
    }
}

impl FromStr for Currency {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(".").collect();
        if split.len() == 2 {
            Ok(Currency {
                arweave: u64::from_str_radix(split[0], 10).expect("Could not parse arweave value"),
                winston: u64::from_str_radix(split[1], 10).expect("Could not parse winston value"),
            })
        } else {
            Ok(Currency {
                winston: u64::from_str_radix(split[0], 10).expect("Could not parse winston value"),
                ..Currency::default()
            })
        }
    }
}

impl Currency {
    pub fn to_json_string(&self) -> String {
        let decimal = format!("{:#012}", self.winston);
        if self.arweave == 0 {
            decimal.trim_start_matches('0').to_string()
        } else {
            self.arweave.to_string() + &decimal
        }
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
        assert_eq!(curr.to_json_string(), "1000000000000");

        let curr = Currency::from_str("10.000000000000").unwrap();
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 10);
        assert_eq!(curr.to_json_string(), "10000000000000");

        let curr = Currency::from_str("999.000000000000").unwrap();
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 999);
        assert_eq!(curr.to_json_string(), "999000000000000");

        let curr = Currency::from_str("999.123123123123").unwrap();
        assert_eq!(curr.winston, 123123123123);
        assert_eq!(curr.arweave, 999);
        assert_eq!(curr.to_json_string(), "999123123123123");

        let curr = Currency::from_str("123123123123").unwrap();
        assert_eq!(curr.winston, 123123123123);
        assert_eq!(curr.arweave, 0);
        assert_eq!(curr.to_json_string(), "123123123123");

        let curr = Currency::from_str("10000").unwrap();
        assert_eq!(curr.winston, 10000);
        assert_eq!(curr.arweave, 0);
        assert_eq!(curr.to_json_string(), "10000");
    }

    #[test]
    fn test_u64_format() {
        let curr = Currency::from(1_000_000_000_000);
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 1);
        assert_eq!(curr.to_json_string(), "1000000000000");

        let curr = Currency::from(10_000_000_000_000);
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 10);
        assert_eq!(curr.to_json_string(), "10000000000000");

        let curr = Currency::from(999_000_000_000_000);
        assert_eq!(curr.winston, 0);
        assert_eq!(curr.arweave, 999);
        assert_eq!(curr.to_json_string(), "999000000000000");

        let curr = Currency::from(999_123_123_123_123);
        assert_eq!(curr.winston, 123123123123);
        assert_eq!(curr.arweave, 999);
        assert_eq!(curr.to_json_string(), "999123123123123");

        let curr = Currency::from(123_123_123_123);
        assert_eq!(curr.winston, 123123123123);
        assert_eq!(curr.arweave, 0);
        assert_eq!(curr.to_json_string(), "123123123123");

        let curr = Currency::from(10000);
        assert_eq!(curr.winston, 10000);
        assert_eq!(curr.arweave, 0);
        assert_eq!(curr.to_json_string(), "10000");
    }
}
