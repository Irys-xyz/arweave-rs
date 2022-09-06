use pretend::{pretend, Result, JsonResult};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NetworkInfo {
    pub network: String,
    pub height: usize,
    pub blocks: usize,
    pub version: String,
    pub release: usize,
    pub current: String,
    pub peers: usize,
    pub queue_length: usize,
    pub node_state_latency: usize,
}

#[pretend]
trait NetworkInfoFetch {
    // Network
    #[request(method = "GET", path = "/info")]
    async fn network_info(&self) -> Result<JsonResult<NetworkInfo, ()>>;

    #[request(method = "GET", path = "/peers")]
    async fn peer_info(&self) -> Result<JsonResult<Vec<String>, ()>>;
}