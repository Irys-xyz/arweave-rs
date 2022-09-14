use std::str::FromStr;

use crypto::RingProvider;
use error::Error;
use transaction::Tx;

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

#[derive(Default)]
pub enum CryptoProvider {
    #[default]
    Ring,
}

pub struct Arweave {
    name: String,
    units: String,
    pub base_url: url::Url,
    pub crypto: Box<dyn crypto::Provider>,
    tx_generator: Box<dyn transaction::Generator>,
}

impl Default for Arweave {
    fn default() -> Self {
        Self {
            name: Default::default(),
            units: Default::default(),
            base_url: url::Url::from_str("https://arweave.net/").unwrap(),
            crypto: Box::new(RingProvider::default()),
            tx_generator: Box::new(Tx::default()),
        }
    }
}

impl<'a> Arweave {
    pub async fn new(
        base_url: url::Url,
        crypto: Box<dyn crypto::Provider>,
        tx_generator: Box<dyn transaction::Generator>,
    ) -> Result<Arweave, Error> {
        let arweave = Arweave {
            name: String::from("arweave"),
            units: String::from("winstons"),
            base_url,
            crypto,
            tx_generator,
        };
        Ok(arweave)
    }
}
