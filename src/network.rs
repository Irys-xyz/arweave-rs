use pretend::{pretend, resolver::UrlResolver, JsonResult, Pretend, Url};
use pretend_reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::{
    block::BlockInfoFull,
    error::Error,
    nodes::Node,
    types::{BlockInfo, NetworkInfo},
};
#[derive(Serialize, Deserialize, Debug)]
struct HeightInfo {
    height: u64,
}
#[pretend]
trait NetworkInfoFetch {
    #[request(method = "GET", path = "/info")]
    async fn network_info(&self) -> pretend::Result<JsonResult<NetworkInfo, Error>>;

    #[request(method = "GET", path = "/peers")]
    async fn peers(&self) -> pretend::Result<JsonResult<Vec<String>, Error>>;

    #[request(method = "GET", path = "/block/hash/{id}")]
    async fn block_by_hash(&self, id: &str) -> pretend::Result<JsonResult<BlockInfoFull, Error>>;

    #[request(method = "GET", path = "/block/height/{height}")]
    async fn block_by_height(
        &self,
        height: u64,
    ) -> pretend::Result<JsonResult<BlockInfoFull, Error>>;
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

    pub async fn peers(&self, url: Option<Url>) -> Result<Vec<Node>, Error> {
        match url {
            Some(url) => {
                let client = self.0.clone().with_url(url);
                let response = client.peers().await;
                match response {
                    Ok(res) => match res {
                        JsonResult::Ok(n) => Ok(n.iter().map(|n| Node(n.to_string())).collect()),
                        JsonResult::Err(err) => Err(err),
                    },
                    Err(err) => Err(Error::RequestFailed(err.to_string())),
                }
            }
            None => {
                let response = self.0.peers().await;
                match response {
                    Ok(res) => match res {
                        JsonResult::Ok(n) => Ok(n.iter().map(|n| Node(n.to_string())).collect()),
                        JsonResult::Err(err) => Err(err),
                    },
                    Err(err) => Err(Error::RequestFailed(err.to_string())),
                }
            }
        }
    }

    pub async fn block_by_hash(&self, id: &str) -> Result<BlockInfo, Error> {
        let response = self
            .0
            .block_by_hash(id)
            .await
            .expect("Error getting block info");
        match response {
            JsonResult::Ok(n) => Ok(BlockInfo::from(n)),
            JsonResult::Err(err) => Err(err),
        }
    }

    pub async fn block_by_height(&self, id: &str) -> Result<BlockInfo, Error> {
        let response = self
            .0
            .block_by_hash(id)
            .await
            .expect("Error getting block info");
        match response {
            JsonResult::Ok(n) => Ok(BlockInfo::from(n)),
            JsonResult::Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{crypto::base64::Base64, network::NetworkInfoClient, ARWEAVE_BASE_URL};
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
        let peer_info = block_on(client.peers(None)).unwrap();

        assert!(!peer_info.is_empty());
    }

    #[test]
    fn test_block_info() {
        let url = Url::parse(ARWEAVE_BASE_URL).unwrap();
        let client = NetworkInfoClient::new(url);

        let block_hash_v1 = "ngFDAB2KRhJgJRysuhpp1u65FjBf5WZk99_NyoMx8w6uP0IVjzb93EVkYxmcErdZ";
        let block_info_v1 = match block_on(client.block_by_hash(block_hash_v1)).unwrap() {
            crate::types::BlockInfo::V1(b) => b,
            _ => panic!(),
        };
        assert_eq!(block_info_v1.nonce, Base64::from_str("AAEBAAABAQAAAQAAAQEBAAEAAAABAQABAQABAAEAAAEBAAAAAQAAAAAAAQAAAQEBAAEBAAEBAQEBAQEAAQEBAAABAQEAAQAAAQABAAABAAAAAAEBAQEBAAABAQEAAAAAAAABAQAAAQAAAQEAAQABAQABAQEAAAABAAABAQABAQEAAAEBAQABAQEBAQEBAAABAQEAAAABAQABAAABAAEAAQEBAQAAAAABAQABAQAAAAAAAAABAQABAAEBAAEAAQABAQABAAEBAQEBAAEAAQABAAABAQEBAQAAAQABAQEBAAEBAQAAAQEBAQABAAEBAQEBAAAAAAABAAEAAAEAAAEAAAEBAAAAAAEAAQABAAAAAAABAQABAQAAAAEBAQAAAAABAAABAAEBAQEAAAAAAQAAAQABAQABAAEAAQABAQAAAAEBAQAAAQAAAAEBAAEBAAEBAQEAAAEBAQAAAQAAAAABAAEAAQEAAQ").unwrap());
        assert_eq!(
            block_info_v1.previous_block,
            Base64::from_str("V6YjG8G3he0JIIwRtzTccX39rS0jH-jOqUJy6rxrVAHY0RT0AVhG8K22wCDxy1A0")
                .unwrap()
        );
        assert_eq!(block_info_v1.timestamp, 1528500720);
        assert_eq!(block_info_v1.last_retarget, 1528500720);
        assert_eq!(block_info_v1.diff, 31);
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

        let block_hash_v2 = "5H-hJycMS_PnPOpobXu2CNobRlgqmw4yEMQSc5LeBfS7We63l8HjS-Ek3QaxK8ug";
        let block_info_v2 = match block_on(client.block_by_hash(block_hash_v2)).unwrap() {
            crate::types::BlockInfo::V2(b) => b,
            _ => panic!(),
        };
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
        assert_eq!(block_info_v2.cumulative_diff, "616416144");
        assert_eq!(
            block_info_v2.hash_list_merkle,
            Base64::from_str("1QVbbLwZHpNMJd8ZghRb13HZfrRu-aIIfzY29r64_yBJAcYv-Kfblv_c2pfKbQBP")
                .unwrap()
        );

        let block_hash_v3 = "5VTARz7bwDO4GqviCSI9JXm8_JOtoQwF-QCZm0Gt2gVgwdzSY3brOtOD46bjMz09";
        let block_info_v3 = match block_on(client.block_by_hash(block_hash_v3)).unwrap() {
            crate::types::BlockInfo::V3(b) => b,
            _ => panic!(),
        };
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
        assert_eq!(block_info_v3.cumulative_diff, "99416580392277");
        assert_eq!(
            block_info_v3.hash_list_merkle,
            Base64::from_str("akSjDrBKPuepJMOhO_S9C-iFp5zn9Glv57HGdN_WPqEToWC0Ukb37Gzs4PDA7oLU")
                .unwrap()
        );
        assert_eq!(block_info_v3.poa.option, "1");
        assert_eq!(block_info_v3.poa.tx_path.0.len(), 640);
        assert_eq!(block_info_v3.poa.data_path.0.len(), 352);
        assert_eq!(block_info_v3.poa.chunk.0.len(), 262144);
    }
}
