use serde::{Deserialize, Serialize};

use crate::{crypto::base64::Base64, transaction::tags::Tag};

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkInfo {
    pub network: String,
    pub version: usize,
    pub release: usize,
    pub height: u128,
    pub current: Base64,
    pub blocks: usize,
    pub peers: usize,
    pub queue_length: usize,
    pub node_state_latency: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProofOfAccess {
    pub option: String,
    pub tx_path: Base64,
    pub data_path: Base64,
    pub chunk: Base64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockInfo {
    pub hash: Base64,
    pub nonce: Base64,
    pub previous_block: Base64,
    pub timestamp: u64,
    pub height: u64,
    pub usd_to_ar_rate: Vec<String>, // pair of numbers
    pub scheduled_usd_to_ar_rate: Vec<String>,
    pub last_retarget: u64,
    pub diff: String,
    pub indep_hash: Base64,
    pub txs: Vec<Base64>,
    pub tx_root: Base64,
    pub wallet_list: Base64,
    pub reward_addr: Base64,
    pub tags: Vec<Tag<Base64>>,
    pub reward_pool: String,
    pub weave_size: String,
    pub block_size: String,
    pub cumulative_diff: String,
    pub hash_list_merkle: Base64,
    pub poa: ProofOfAccess,
}
