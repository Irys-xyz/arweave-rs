use serde::{Deserialize, Serialize};

pub mod create;
pub mod get;

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub format: usize,
    pub id: String,
    pub last_tx: String,
    pub owner: String,
    pub tags: Vec<Tag>,
    pub target: String,
    pub quantity: String,
    pub data_root: String,
    pub data_size: String,
    pub data: Vec<u8>,
    pub reward: String,
    pub signature: String,
}

pub struct UnsignedTransaction {
    pub format: usize,
    pub owner: String,
    pub target: String,
    pub data_root: String,
    pub data_size: String,
    pub quantity: String,
    pub reward: String,
    pub last_tx: String,
    pub tags: Vec<Tag>,
}
