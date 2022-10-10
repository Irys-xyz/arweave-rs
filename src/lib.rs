use std::{path::PathBuf, str::FromStr};

use crypto::base64::Base64;
use error::Error;
use pretend::StatusCode;
use serde::{Deserialize, Serialize};
use signer::ArweaveSigner;
use transaction::{
    client::{TxClient, TxStatus},
    tags::Tag,
    Tx,
};

pub mod client;
pub mod crypto;
pub mod currency;
pub mod error;
pub mod network;
pub mod signer;
pub mod transaction;
pub mod wallet;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Block size used for pricing calculations = 256 KB
pub const BLOCK_SIZE: u64 = 1024 * 256;

/// Maximum data size to send to `tx/` endpoint. Sent to `chunk/` endpoint above this.
pub const MAX_TX_DATA: u64 = 10_000_000;

/// Multiplier applied to the buffer argument from the cli to determine the maximum number
/// of simultaneous request to the `chunk/ endpoint`.
pub const CHUNKS_BUFFER_FACTOR: usize = 20;

/// Number of times to retry posting chunks if not successful.
pub const CHUNKS_RETRIES: u16 = 10;

/// Number of seconds to wait between retying to post a failed chunk.
pub const CHUNKS_RETRY_SLEEP: u64 = 1;

const ARWEAVE_BASE_URL: &str = "https://arweave.net/";

#[derive(Serialize, Deserialize, Debug)]
pub struct OraclePrice {
    pub arweave: OraclePricePair,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OraclePricePair {
    pub usd: f32,
}

pub struct Arweave {
    name: String,
    units: String,
    pub base_url: url::Url,
    pub signer: ArweaveSigner,
    tx_client: TxClient,
}

impl Default for Arweave {
    fn default() -> Self {
        let arweave_url = url::Url::from_str(ARWEAVE_BASE_URL).unwrap();
        Self {
            name: Default::default(),
            units: Default::default(),
            base_url: arweave_url.clone(),
            signer: Default::default(),
            tx_client: TxClient::default(),
        }
    }
}

impl Arweave {
    pub fn from_keypair_path(keypair_path: PathBuf, base_url: url::Url) -> Result<Arweave, Error> {
        let signer =
            ArweaveSigner::from_keypair_path(keypair_path).expect("Could not create signer");
        let tx_client = TxClient::new(reqwest::Client::new(), base_url.clone())
            .expect("Could not create TxClient");
        let arweave = Arweave {
            base_url,
            signer,
            tx_client,
            ..Default::default()
        };
        Ok(arweave)
    }

    pub async fn create_transaction(
        &self,
        target: Base64,
        other_tags: Vec<Tag<Base64>>,
        data: Vec<u8>,
        quantity: u128,
        fee: u64,
        auto_content_tag: bool,
    ) -> Result<Tx, Error> {
        let last_tx = self.get_last_tx().await;
        Tx::new(
            self.signer.get_provider(),
            target,
            data,
            quantity,
            fee,
            last_tx,
            other_tags,
            auto_content_tag,
        )
    }

    pub fn sign_transaction(&self, transaction: Tx) -> Result<Tx, Error> {
        self.signer.sign_transaction(transaction)
    }

    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        self.signer.sign_message(message)
    }

    pub fn verify_transaction(&self, transaction: &Tx) -> Result<(), Error> {
        self.signer.verify_transaction(transaction)
    }

    pub async fn post_transaction(&self, signed_transaction: &Tx) -> Result<(String, u64), Error> {
        self.tx_client
            .post_transaction(signed_transaction)
            .await
            .map(|(id, reward)| (id.to_string(), reward))
    }

    async fn get_last_tx(&self) -> Base64 {
        self.tx_client.get_last_tx().await
    }

    pub async fn get_fee(&self, target: Base64) -> Result<u64, Error> {
        self.tx_client.get_fee(target).await
    }

    pub async fn get_tx(&self, id: Base64) -> Result<(StatusCode, Option<Tx>), Error> {
        self.tx_client.get_tx(id).await
    }

    pub async fn get_tx_status(&self, id: Base64) -> Result<(StatusCode, Option<TxStatus>), Error> {
        self.tx_client.get_tx_status(id).await
    }

    pub fn get_pub_key(&self) -> String {
        self.signer.keypair_modulus().to_string()
    }

    pub fn get_wallet_address(&self) -> String {
        self.signer.wallet_address().to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, path::PathBuf, str::FromStr};

    use pretend::Url;

    use crate::{error::Error, transaction::Tx, Arweave, ARWEAVE_BASE_URL};

    #[test]
    pub fn should_parse_and_verify_valid_tx() -> Result<(), Error> {
        let mut file = File::open("res/sample_tx.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        let tx = Tx::from_str(&data).unwrap();

        let path = PathBuf::from_str("res/test_wallet.json").unwrap();
        let arweave =
            Arweave::from_keypair_path(path, Url::from_str(ARWEAVE_BASE_URL).unwrap()).unwrap();

        //TODO: verification
        //arweave.verify_transaction(&tx)
        Ok(())
    }
}
