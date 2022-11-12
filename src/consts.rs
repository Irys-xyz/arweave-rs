pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const ARWEAVE_BASE_URL: &str = "https://arweave.net/";

/// Block size used for pricing calculations = 256 KB
pub const BLOCK_SIZE: u64 = 1024 * 256;

/// Chunk of data to download
pub const CHUNK_SIZE: u64 = 1024 * 256; //256kb

/// Maximum data size to send to `tx/` endpoint. Sent to `chunk/` endpoint above this.
pub const MAX_TX_DATA: u64 = 10_000_000;

/// Multiplier applied to the buffer argument from the cli to determine the maximum number
/// of simultaneous request to the `chunk/ endpoint`.
pub const CHUNKS_BUFFER_FACTOR: usize = 20;

/// Number of times to retry posting chunks if not successful.
pub const CHUNKS_RETRIES: u16 = 10;

/// Number of seconds to wait between retying to post a failed chunk.
pub const CHUNKS_RETRY_SLEEP: u64 = 1;

// First block to use V2 block format
pub const V2_BLOCK_HEIGHT: u32 = 269510;

// First block to use V3 block format
pub const V3_BLOCK_HEIGHT: u32 = 422250;

pub const DEFAULT_RETRIES_PER_CHUNK: u16 = 3;

pub const CONFIRMATION_THRESHOLD: u64 = 15;

pub const DEFAULT_CONCURRENCY_LEVEL: u16 = 100;
