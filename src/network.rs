use pretend::{pretend, resolver::UrlResolver, JsonResult, Pretend, Url};
use pretend_reqwest::Client as HttpClient;
use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize, Debug)]
pub struct NetworkInfo {
    pub network: String,
    pub version: usize,
    pub release: usize,
    pub height: usize,
    pub current: String,
    pub blocks: usize,
    pub peers: usize,
    pub queue_length: usize,
    pub node_state_latency: usize,
}

#[pretend]
trait NetworkInfoFetch {
    #[request(method = "GET", path = "/info")]
    async fn network_info(&self) -> pretend::Result<JsonResult<NetworkInfo, Error>>;

    #[request(method = "GET", path = "/peers")]
    async fn peer_info(&self) -> pretend::Result<JsonResult<Vec<String>, Error>>;
}

pub struct NetworkInfoClient(Pretend<HttpClient, UrlResolver>);

impl NetworkInfoClient {
    pub fn new(url: Url) -> Self {
        let client = HttpClient::default();
        let pretend = Pretend::for_client(client).with_url(url);
        Self(pretend)
    }

    pub async fn network_info(&self) -> Result<NetworkInfo, Error> {
        let response = self
            .0
            .network_info()
            .await
            .expect("Error getting network info");
        match response {
            JsonResult::Ok(n) => Ok(n),
            JsonResult::Err(err) => Err(err),
        }
    }

    pub async fn peer_info(&self) -> Result<Vec<String>, Error> {
        let response = self.0.peer_info().await.expect("Error getting peer info");
        match response {
            JsonResult::Ok(n) => Ok(n),
            JsonResult::Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::network::NetworkInfoClient;
    use pretend::Url;
    use tokio_test::block_on;

    #[test]
    fn test_network_info() {
        let url = Url::parse("https://arweave.net/").unwrap();
        let client = NetworkInfoClient::new(url);
        let network_info = block_on(client.network_info()).unwrap();

        assert_eq!(network_info.network, "arweave.N.1".to_string());
    }

    #[test]
    fn test_peer_info() {
        let url = Url::parse("https://arweave.net/").unwrap();
        let client = NetworkInfoClient::new(url);
        let peer_info = block_on(client.peer_info()).unwrap();

        assert!(peer_info.len() > 0);
    }
}
