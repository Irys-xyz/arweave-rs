use std::net::IpAddr;

use pretend::{pretend, resolver::UrlResolver, JsonResult, Pretend, Url};
use pretend_reqwest::Client as HttpClient;

use crate::{
    error::Error,
    types::{BlockInfo, NetworkInfo},
};

#[pretend]
trait NetworkInfoFetch {
    #[request(method = "GET", path = "/info")]
    async fn network_info(&self) -> pretend::Result<JsonResult<NetworkInfo, Error>>;

    #[request(method = "GET", path = "/peers")]
    async fn peer_info(&self) -> pretend::Result<JsonResult<Vec<String>, Error>>;

    #[request(method = "GET", path = "/block/hash/{id}")]
    async fn block(&self, id: &str) -> pretend::Result<JsonResult<BlockInfo, Error>>;
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

    pub async fn block(&self, id: &str) -> Result<BlockInfo, Error> {
        let response = self.0.block(id).await.expect("Error getting block info");
        match response {
            JsonResult::Ok(n) => Ok(n),
            JsonResult::Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{network::NetworkInfoClient, ARWEAVE_BASE_URL};
    use pretend::Url;
    use tokio_test::block_on;

    #[test]
    fn test_network_info() {
        let url = Url::parse(ARWEAVE_BASE_URL).unwrap();
        let client = NetworkInfoClient::new(url);
        let network_info = block_on(client.network_info()).unwrap();

        assert_eq!(network_info.network, "arweave.N.1".to_string());
    }

    #[test]
    fn test_peer_info() {
        let url = Url::parse(ARWEAVE_BASE_URL).unwrap();
        let client = NetworkInfoClient::new(url);
        let peer_info = block_on(client.peer_info()).unwrap();

        assert!(peer_info.len() > 0);
    }

    #[test]
    fn test_block_info() {
        let block_hash = "g2iYhOVi2FmFvg8MnV5yry6MLi_kMvedk9HwHerIz01PgcWar7tD10JKn2Se6kwR";
        let url = Url::parse(ARWEAVE_BASE_URL).unwrap();
        let client = NetworkInfoClient::new(url);
        let block_info = block_on(client.block(block_hash)).unwrap();

        assert!(block_info.timestamp == 1667863185);
        assert_eq!(block_info.indep_hash.to_string(), block_hash);
    }
}
