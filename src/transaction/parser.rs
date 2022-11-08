use std::str::FromStr;

use serde::{ser::SerializeStruct, Serialize, Serializer};

use crate::{currency::Currency, error::Error};

use super::Tx;
use crate::types::Tx as JsonTx;

impl From<JsonTx> for Tx {
    fn from(json_tx: JsonTx) -> Self {
        Tx {
            quantity: Currency::from_str(&json_tx.quantity).unwrap(),
            format: json_tx.format,
            id: json_tx.id,
            last_tx: json_tx.last_tx,
            owner: json_tx.owner,
            tags: json_tx.tags,
            target: json_tx.target,
            data_root: json_tx.data_root,
            data: json_tx.data,
            data_size: u64::from_str(&json_tx.data_size).unwrap(),
            reward: u64::from_str(&json_tx.reward).unwrap(),
            signature: json_tx.signature,
            chunks: vec![],
            proofs: vec![],
        }
    }
}

impl FromStr for Tx {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let json_tx: JsonTx = serde_json::from_str(s).expect("Could not parse json");
        Ok(Tx::from(json_tx))
    }
}

impl Serialize for Tx {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Tx", 12)?;
        s.serialize_field("format", &self.format)?;
        s.serialize_field("id", &self.id.to_string())?;
        s.serialize_field("last_tx", &self.last_tx.to_string())?;
        s.serialize_field("owner", &self.owner.to_string())?;
        s.serialize_field("tags", &self.tags)?;
        s.serialize_field("target", &self.target.to_string())?;
        s.serialize_field("quantity", &self.quantity.to_string())?;
        s.serialize_field("data", &self.data.to_string())?;
        s.serialize_field("data_size", &self.data_size.to_string())?;
        s.serialize_field("data_root", &self.data_root.to_string())?;
        s.serialize_field("reward", &self.reward.to_string())?;
        s.serialize_field("signature", &self.signature.to_string())?;

        s.end()
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, str::FromStr};

    use crate::{
        crypto::base64::Base64,
        currency::Currency,
        transaction::{tags::Tag, Tx},
    };

    #[test]
    pub fn should_parse_correctly() {
        let mut file = File::open("res/sample_tx.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let actual_tx = Tx::from_str(&data).unwrap();
        let expected_tx = Tx {
            format: 2,
            id: Base64::from_str("t3K1b8IhvtGWxAGsipZE5NafmEGrtj3OAcYikJ0edeU").unwrap(),
            last_tx: Base64::from_str("ddvXNxatQmS3LeKi_x1RJn6g9G0esUaTEgT40a6f_WYyawZaSK3w8WC2czAuLgmT").unwrap(),
            owner: Base64::from_str("pjdss8ZaDfEH6K6U7GeW2nxDqR4IP049fk1fK0lndimbMMVBdPv_hSpm8T8EtBDxrUdi1OHZfMhUixGaut-3nQ4GG9nM249oxhCtxqqNvEXrmQRGqczyLxuh-fKn9Fg--hS9UpazHpfVAFnB5aCfXoNhPuI8oByyFKMKaOVgHNqP5NBEqabiLftZD3W_lsFCPGuzr4Vp0YS7zS2hDYScC2oOMu4rGU1LcMZf39p3153Cq7bS2Xh6Y-vw5pwzFYZdjQxDn8x8BG3fJ6j8TGLXQsbKH1218_HcUJRvMwdpbUQG5nvA2GXVqLqdwp054Lzk9_B_f1lVrmOKuHjTNHq48w").unwrap(),
            tags: vec![
                Tag { name: Base64(b"test".to_vec()), value: Base64(b"test".to_vec()) }
            ],
            target: Base64::from_str("PAgdonEn9f5xd-UbYdCX40Sj28eltQVnxz6bbUijeVY").unwrap(),
            quantity: Currency::from(100000),
            data_root: Base64(vec![]),
            data: Base64(vec![]),
            data_size: 0,
            reward: 600912,
            signature: Base64::from_str("EJQN0DpfPBm1aUo1qk6dCkrY_zKHMJBQx3v36UOzmodF39RvBI2rqx_gTgLzszNkHIWnf-zwzXCz6xF5wzlrHWkosgfSwfZOhm3aVE5KLGvqVqSlMTlIzkIcR6KKFRe9m7HyOxJHvXykAD8X1X_6RExnXAZX4B9mwR10lqCG2wkRMJxchVisOZph-O5OfgteC1lb5YFx0BNAtmVgtUlY7dQdV1vVYq2_sDJPkYpHK5YIMIjoRsqdGP31gOFXTmzuIHYhRyii-clx2uxrv0pjfnv9tl9WPViHu3FGLlW9tH5z3mXdt7PQx-o8MGK_MXz10LLlqsPdos2rI3D3MgPUqQ").unwrap(),
            chunks: vec![],
            proofs: vec![]
        };

        assert_eq!(actual_tx, expected_tx);
    }
}
