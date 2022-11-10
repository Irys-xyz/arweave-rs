use crate::{
    consts::{V2_BLOCK_HEIGHT, V3_BLOCK_HEIGHT},
    crypto::base64::Base64,
    types::{self, BlockInfo, BlockInfoV1, BlockInfoV2, BlockInfoV3, Tag},
};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProofOfAccess {
    pub option: String,
    pub tx_path: Base64,
    pub data_path: Base64,
    pub chunk: Base64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockInfoFull {
    pub nonce: Base64,
    pub previous_block: Base64,
    pub timestamp: u64,
    pub last_retarget: u64,
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub diff: String,
    pub height: u64,
    pub hash: Base64,
    pub indep_hash: Base64,
    pub txs: Vec<Base64>,
    pub wallet_list: Base64,
    pub reward_addr: Base64,
    pub tags: Vec<Tag>,
    pub reward_pool: u64,
    pub weave_size: u64,
    pub block_size: u64,

    //V2 Stuff
    pub cumulative_diff: Option<String>,
    pub hash_list_merkle: Option<Base64>,

    // V3 stuff
    pub tx_root: Base64,
    pub tx_tree: Vec<Base64>,
    pub poa: ProofOfAccess,
}

impl From<BlockInfoFull> for BlockInfo {
    fn from(b: BlockInfoFull) -> Self {
        if b.height < V2_BLOCK_HEIGHT.into() {
            BlockInfo::V1(BlockInfoV1 {
                nonce: b.nonce,
                previous_block: b.previous_block,
                timestamp: b.timestamp,
                last_retarget: b.last_retarget,
                diff: b.diff.parse::<u64>().expect("Cannot parse diff"),
                height: b.height,
                hash: b.hash,
                indep_hash: b.indep_hash,
                txs: b.txs,
                wallet_list: b.wallet_list,
                reward_addr: b.reward_addr,
                tags: b.tags,
                reward_pool: b.reward_pool,
                weave_size: b.weave_size,
                block_size: b.block_size,
            })
        } else if b.height < V3_BLOCK_HEIGHT.into() {
            BlockInfo::V2(BlockInfoV2 {
                nonce: b.nonce,
                previous_block: b.previous_block,
                timestamp: b.timestamp,
                last_retarget: b.last_retarget,
                diff: b.diff,
                height: b.height,
                hash: b.hash,
                indep_hash: b.indep_hash,
                txs: b.txs,
                wallet_list: b.wallet_list,
                reward_addr: b.reward_addr,
                tags: b.tags,
                reward_pool: b.reward_pool,
                weave_size: b.weave_size,
                block_size: b.block_size,
                cumulative_diff: b.cumulative_diff.expect("No cumulative diff present"),
                hash_list_merkle: b.hash_list_merkle.expect("No hash list merkle present"),
            })
        } else {
            BlockInfo::V3(BlockInfoV3 {
                nonce: b.nonce,
                previous_block: b.previous_block,
                timestamp: b.timestamp,
                last_retarget: b.last_retarget,
                diff: b.diff,
                height: b.height,
                hash: b.hash,
                indep_hash: b.indep_hash,
                txs: b.txs,
                wallet_list: b.wallet_list,
                reward_addr: b.reward_addr,
                tags: b.tags,
                reward_pool: b.reward_pool,
                weave_size: b.weave_size,
                block_size: b.block_size,
                cumulative_diff: b.cumulative_diff.expect("No cumulative diff present"),
                hash_list_merkle: b.hash_list_merkle.expect("No hash list merkle present"),
                tx_root: b.tx_root,
                tx_tree: b.tx_tree,
                poa: types::ProofOfAccess {
                    option: b.poa.option,
                    tx_path: b.poa.tx_path,
                    data_path: b.poa.data_path,
                    chunk: b.poa.chunk,
                },
            })
        }
    }
}
