use std::{path::PathBuf, str::FromStr, time::Duration};

use crypto::{base64::Base64, deep_hash::ToItems, Provider, RingProvider};
use error::Error;
use futures::future::try_join;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use tokio::time::sleep;
use transaction::{get::TransactionInfoClient, tags::Tag, Tx};

pub mod client;
pub mod crypto;
pub mod error;
pub mod network;
pub mod transaction;
pub mod wallet;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Winstons are a sub unit of the native Arweave network token, AR. There are 10<sup>12</sup> Winstons per AR.
pub const WINSTONS_PER_AR: u64 = 1_000_000_000_000;

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

pub struct Arweave {
    name: String,
    units: String,
    pub base_url: url::Url,
    pub crypto: Box<dyn crypto::Provider>,
    tx_generator: Box<dyn transaction::Generator>,
    tx_client: TransactionInfoClient,
}

impl Default for Arweave {
    fn default() -> Self {
        let arweave_url = url::Url::from_str("https://arweave.net/").unwrap();
        Self {
            name: Default::default(),
            units: Default::default(),
            base_url: arweave_url.clone(),
            crypto: Box::new(RingProvider::default()),
            tx_generator: Box::new(Tx::default()),
            tx_client: TransactionInfoClient::new(arweave_url),
        }
    }
}

impl Arweave {
    pub fn from_keypair_path(keypair_path: PathBuf, base_url: url::Url) -> Result<Arweave, Error> {
        let crypto = RingProvider::from_keypair_path(keypair_path);
        let arweave = Arweave {
            base_url,
            crypto: Box::new(crypto),
            ..Default::default()
        };
        Ok(arweave)
    }

    pub fn create_w2w_transaction(
        &self,
        target: Base64,
        other_tags: Vec<Tag<Base64>>,
        last_tx: Base64,
        quantity: u64,
        reward: u64,
        auto_content_tag: bool,
    ) -> Result<Tx, Error> {
        let owner = Base64(self.crypto.pub_key().to_vec());
        self.tx_generator.new_w2w_tx(
            &*self.crypto,
            owner,
            target,
            vec![],
            quantity,
            reward,
            last_tx,
            other_tags,
            auto_content_tag,
        )
    }

    /// Gets deep hash, signs and sets signature and id.
    pub fn sign_transaction(&self, mut transaction: Tx) -> Result<Tx, Error> {
        let deep_hash_item = transaction.to_deep_hash_item()?;
        let deep_hash = self.crypto.deep_hash(deep_hash_item);
        let signature = self.crypto.sign(&deep_hash);
        let id = self.crypto.hash_sha256(&signature);
        transaction.signature = Base64(signature);
        transaction.id = Base64(id.to_vec());
        Ok(transaction)
    }

    pub async fn post_transaction(&self, signed_transaction: &Tx) -> Result<(Base64, u64), Error> {
        if signed_transaction.id.0.is_empty() {
            return Err(Error::UnsignedTransaction);
        }

        let mut retries = 0;
        let mut status = reqwest::StatusCode::NOT_FOUND;
        let url = self.base_url.join("tx").expect("Valid url joining");
        let client = reqwest::Client::new();

        while (retries < CHUNKS_RETRIES) & (status != reqwest::StatusCode::OK) {
            status = client
                .post(url.clone())
                .json(&signed_transaction)
                .header(&ACCEPT, "application/json")
                .header(&CONTENT_TYPE, "application/json")
                .send()
                .await
                .unwrap()
                .status();
            if status == reqwest::StatusCode::OK {
                return Ok((signed_transaction.id.clone(), signed_transaction.reward));
            }
            dbg!("post_transaction: {:?}", status);
            sleep(Duration::from_secs(CHUNKS_RETRY_SLEEP)).await;
            retries += 1;
        }

        Err(Error::StatusCodeNotOk)
    }
}
