use pretend::{pretend, Result, JsonResult};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionData {
    pub format: usize,
    pub id: String,
    pub last_tx: String,
    pub owner: String,
    pub tags: Vec<Tag>,
    pub target: String,
    pub quantity: String,
    pub data: Vec<u8>,
    pub reward: String,
    pub signature: String,
    pub data_size: String,
    pub data_root: String,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct TransactionConfirmedData {
    block_indep_hash: String,
    block_height: usize,
    number_of_confirmations: usize,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct TransactionStatusResponse {
    status: usize,
    confirmed: Option<TransactionConfirmedData>,
}
#[pretend]
trait TransactionInfoFetch {
    #[request(method = "GET", path = "/price/{byte_size}")]
    async fn tx_get_price(&self, byte_size: &str) -> Result<String>;

    #[request(method = "GET", path = "/tx/{id}")]
    async fn tx_get(&self, id: &str) -> Result<JsonResult<TransactionData, ()>>;

    #[request(method = "GET", path = "/tx/{id}/status")]
    async fn tx_status(&self, id: &str) -> Result<JsonResult<TransactionStatusResponse, ()>>;
}