use crate::{
    client::Client,
    types::{BlockInfo, NetworkInfo},
};
use pretend::{
    interceptor::NoopRequestInterceptor, pretend, resolver::UrlResolver, JsonResult, Pretend, Url,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
struct HeightInfo {
    height: u64,
}

#[derive(Debug, Error, Deserialize)]

pub enum ResponseError {
    #[error("Internal error")]
    InternalError(String),

    #[error("Unknown error")]
    UnknownError(String),
}

#[pretend]
trait NetworkInfoFetch {
    #[request(method = "GET", path = "/info")]
    async fn network_info(&self) -> pretend::Result<JsonResult<NetworkInfo, ResponseError>>;

    #[request(method = "GET", path = "/peers")]
    async fn peer_info(&self) -> pretend::Result<JsonResult<Vec<String>, ResponseError>>;

    #[request(method = "GET", path = "/block/hash/{id}")]
    async fn block_by_hash(
        &self,
        id: &str,
    ) -> pretend::Result<JsonResult<BlockInfo, ResponseError>>;

    #[request(method = "GET", path = "/block/height/{height}")]
    async fn block_by_height(
        &self,
        height: u64,
    ) -> pretend::Result<JsonResult<BlockInfo, ResponseError>>;
}

pub struct NetworkInfoClient(Pretend<Client, UrlResolver, NoopRequestInterceptor>);

impl NetworkInfoClient {
    pub fn new(url: Url) -> Self {
        let client = Client::default();
        let pretend = Pretend::for_client(client).with_url(url);
        Self(pretend)
    }

    pub async fn network_info(&self) -> Result<NetworkInfo, ResponseError> {
        let response = self
            .0
            .network_info()
            .await
            .map_err(|err| ResponseError::InternalError(err.to_string()))?;
        match response {
            JsonResult::Ok(n) => Ok(n),
            JsonResult::Err(err) => Err(err),
        }
    }

    pub async fn peer_info(&self) -> Result<Vec<String>, ResponseError> {
        let response = self
            .0
            .peer_info()
            .await
            .map_err(|err| ResponseError::InternalError(err.to_string()))?;
        match response {
            JsonResult::Ok(n) => Ok(n),
            JsonResult::Err(err) => Err(err),
        }
    }

    pub async fn block_by_hash(&self, id: &str) -> Result<BlockInfo, ResponseError> {
        let response = self
            .0
            .block_by_hash(id)
            .await
            .map_err(|err| ResponseError::InternalError(err.to_string()))?;
        match response {
            JsonResult::Ok(n) => Ok(n),
            JsonResult::Err(err) => Err(err),
        }
    }

    pub async fn block_by_height(&self, id: &str) -> Result<BlockInfo, ResponseError> {
        let response = self
            .0
            .block_by_hash(id)
            .await
            .map_err(|err| ResponseError::InternalError(err.to_string()))?;
        match response {
            JsonResult::Ok(n) => Ok(n),
            JsonResult::Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{consts::ARWEAVE_BASE_URL, crypto::base64::Base64, network::NetworkInfoClient};
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

        assert!(!peer_info.is_empty());
    }

    #[test]
    fn test_block_info() {
        let url = Url::parse(ARWEAVE_BASE_URL).unwrap();
        let client = NetworkInfoClient::new(url);

        let block_hash_v1 = "ngFDAB2KRhJgJRysuhpp1u65FjBf5WZk99_NyoMx8w6uP0IVjzb93EVkYxmcErdZ";
        let block_info_v1 = block_on(client.block_by_hash(block_hash_v1)).unwrap();
        assert_eq!(block_info_v1.nonce, Base64::from_str("AAEBAAABAQAAAQAAAQEBAAEAAAABAQABAQABAAEAAAEBAAAAAQAAAAAAAQAAAQEBAAEBAAEBAQEBAQEAAQEBAAABAQEAAQAAAQABAAABAAAAAAEBAQEBAAABAQEAAAAAAAABAQAAAQAAAQEAAQABAQABAQEAAAABAAABAQABAQEAAAEBAQABAQEBAQEBAAABAQEAAAABAQABAAABAAEAAQEBAQAAAAABAQABAQAAAAAAAAABAQABAAEBAAEAAQABAQABAAEBAQEBAAEAAQABAAABAQEBAQAAAQABAQEBAAEBAQAAAQEBAQABAAEBAQEBAAAAAAABAAEAAAEAAAEAAAEBAAAAAAEAAQABAAAAAAABAQABAQAAAAEBAQAAAAABAAABAAEBAQEAAAAAAQAAAQABAQABAAEAAQABAQAAAAEBAQAAAQAAAAEBAAEBAAEBAQEAAAEBAQAAAQAAAAABAAEAAQEAAQ").unwrap());
        assert_eq!(
            block_info_v1.previous_block,
            Base64::from_str("V6YjG8G3he0JIIwRtzTccX39rS0jH-jOqUJy6rxrVAHY0RT0AVhG8K22wCDxy1A0")
                .unwrap()
        );
        assert_eq!(block_info_v1.timestamp, 1528500720);
        assert_eq!(block_info_v1.last_retarget, 1528500720);
        assert_eq!(block_info_v1.diff, "31");
        assert_eq!(block_info_v1.height, 100);
        assert_eq!(
            block_info_v1.hash,
            Base64::from_str("AAAAANsEvzGbICpfAj3NN41_ox--2cNxkEhAo0aggpDPkY7zru29g24uMWUP9hTa")
                .unwrap()
        );
        assert_eq!(
            block_info_v1.indep_hash,
            Base64::from_str(block_hash_v1).unwrap()
        );
        assert_eq!(block_info_v1.txs.len(), 1);
        assert_eq!(block_info_v1.tx_root, Base64::default());
        assert_eq!(block_info_v1.tx_tree.len(), 0);
        assert_eq!(
            block_info_v1.wallet_list,
            Base64::from_str("ph2FDDuQjNbca34tz7vP9X5Xve2EGJi2ZgFqhMITAdw").unwrap()
        );
        assert_eq!(
            block_info_v1.reward_addr,
            Base64::from_str("em8MfGRInwWEAQnE6b50ENaFOf-0to4Pbygng1ilWGQ").unwrap()
        );
        assert_eq!(block_info_v1.tags, []);
        assert_eq!(block_info_v1.reward_pool, 60770606104);
        assert_eq!(block_info_v1.weave_size, 599058);
        assert_eq!(block_info_v1.block_size, 0);
        assert_eq!(block_info_v1.poa.option, "1");
        assert_eq!(block_info_v1.poa.tx_path, Base64::default());
        assert_eq!(block_info_v1.poa.data_path, Base64::default());
        assert_eq!(block_info_v1.poa.chunk, Base64::default());

        let block_hash_v2 = "5H-hJycMS_PnPOpobXu2CNobRlgqmw4yEMQSc5LeBfS7We63l8HjS-Ek3QaxK8ug";
        let block_info_v2 = block_on(client.block_by_hash(block_hash_v2)).unwrap();
        assert_eq!(
            block_info_v2.nonce,
            Base64::from_str("O3IQWXYmxLN_b0w7QyT2GTruaVIGsl-Ybhc6Pl2V20U").unwrap()
        );
        assert_eq!(
            block_info_v2.previous_block,
            Base64::from_str("VRVYubqppWUVAeCWlzHR-38dQoWcFAKbGculkVZThfj-hNMX4QVZjqkC6-PkiNGE")
                .unwrap()
        );
        assert_eq!(block_info_v2.timestamp, 1567052949);
        assert_eq!(block_info_v2.last_retarget, 1567052114);
        assert_eq!(
            block_info_v2.diff,
            "115792088374597902074750511579343425068641803109251942518159264612597601665024"
        );
        assert_eq!(block_info_v2.height, 269512);
        assert_eq!(
            block_info_v2.hash,
            Base64::from_str("____47liyh_OZdYUP4EzBoLl7JOPge9VsWPQ3b5kiU8").unwrap()
        );
        assert_eq!(
            block_info_v2.indep_hash,
            Base64::from_str(block_hash_v2).unwrap()
        );
        assert_eq!(block_info_v2.txs.len(), 2);
        assert_eq!(block_info_v2.tx_root, Base64::default());
        assert_eq!(block_info_v2.tx_tree.len(), 0);
        assert_eq!(
            block_info_v2.wallet_list,
            Base64::from_str("6haahtRP5WVchxPbqtLCqDsFWidhebYJpU5PVB4zQhE").unwrap()
        );
        assert_eq!(
            block_info_v2.reward_addr,
            Base64::from_str("aE1AjkBoXBfF-PRP2dzRrbYY8cY2OYzeH551nSPRU5M").unwrap()
        );
        assert_eq!(block_info_v2.tags, []);
        assert_eq!(block_info_v2.reward_pool, 0);
        assert_eq!(block_info_v2.weave_size, 21080508475);
        assert_eq!(block_info_v2.block_size, 991723);
        assert!(block_info_v2.cumulative_diff.is_some());
        assert_eq!(block_info_v2.cumulative_diff.unwrap(), "616416144");
        assert!(block_info_v2.hash_list_merkle.is_some());
        assert_eq!(
            block_info_v2.hash_list_merkle.unwrap(),
            Base64::from_str("1QVbbLwZHpNMJd8ZghRb13HZfrRu-aIIfzY29r64_yBJAcYv-Kfblv_c2pfKbQBP")
                .unwrap()
        );
        assert_eq!(block_info_v2.poa.option, "1");
        assert_eq!(block_info_v2.poa.tx_path, Base64::default());
        assert_eq!(block_info_v2.poa.data_path, Base64::default());
        assert_eq!(block_info_v2.poa.chunk, Base64::default());

        let block_hash_v3 = "5VTARz7bwDO4GqviCSI9JXm8_JOtoQwF-QCZm0Gt2gVgwdzSY3brOtOD46bjMz09";
        let block_info_v3 = block_on(client.block_by_hash(block_hash_v3)).unwrap();
        assert_eq!(
            block_info_v3.nonce,
            Base64::from_str("W3Jy4wp2LVbDFhGX_hUjRQZCkTdEbKxz45E5OVe52Lo").unwrap()
        );
        assert_eq!(
            block_info_v3.previous_block,
            Base64::from_str("YuTyalVBTNB9t5KhuRezcIgxVz9PbQsbrcY4Tpkiu8XBPgglGM_Yql5qZd0c9PVG")
                .unwrap()
        );
        assert_eq!(block_info_v3.timestamp, 1586440919);
        assert_eq!(block_info_v3.last_retarget, 1586440919);
        assert_eq!(
            block_info_v3.diff,
            "115792089039110416381168389782714091630053560834545856346499935466490404274176"
        );
        assert_eq!(block_info_v3.height, 422250);
        assert_eq!(
            block_info_v3.hash,
            Base64::from_str("_____8422fLZnBsEsxtwEdpi8GZDHVT-aFlqroQDG44").unwrap()
        );
        assert_eq!(
            block_info_v3.indep_hash,
            Base64::from_str(block_hash_v3).unwrap()
        );
        assert_eq!(block_info_v3.txs.len(), 1);
        assert_eq!(
            block_info_v3.tx_root,
            Base64::from_str("lsoo-p3Tj7oblZ-54WVPHoVguqgw5rA9Jf3lLH6H8zY").unwrap()
        );
        assert_eq!(block_info_v3.tx_tree.len(), 0);
        assert_eq!(
            block_info_v3.wallet_list,
            Base64::from_str("N5NJtXhgH9bPmXoSopehcr_zqwyPjjg3igel0V8G1DdLk_BYdoRVIBsqjVA9JmFc")
                .unwrap()
        );
        assert_eq!(
            block_info_v3.reward_addr,
            Base64::from_str("Oox7m4HIcVhUtMd6AUuGtlaOoSCmREUNPyyKQCbz4d4").unwrap()
        );
        assert_eq!(block_info_v3.tags, []);
        assert_eq!(block_info_v3.reward_pool, 3026104059201252);
        assert_eq!(block_info_v3.weave_size, 407672420044);
        assert_eq!(block_info_v3.block_size, 937455);
        assert!(block_info_v3.cumulative_diff.is_some());
        assert_eq!(block_info_v3.cumulative_diff.unwrap(), "99416580392277");
        assert!(block_info_v3.hash_list_merkle.is_some());
        assert_eq!(
            block_info_v3.hash_list_merkle.unwrap(),
            Base64::from_str("akSjDrBKPuepJMOhO_S9C-iFp5zn9Glv57HGdN_WPqEToWC0Ukb37Gzs4PDA7oLU")
                .unwrap()
        );
        assert_eq!(block_info_v3.poa.option, "1");
        assert_eq!(block_info_v3.poa.tx_path.0.len(), 640);
        assert_eq!(block_info_v3.poa.data_path.0.len(), 352);
        assert_eq!(block_info_v3.poa.chunk.0.len(), 262144);
    }
}
