use pretend::{pretend, resolver::UrlResolver, JsonResult, Pretend, Url};
use pretend_reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::error::Error;

use super::Tx;

#[allow(unused)]
#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionConfirmedData {
    block_indep_hash: String,
    block_height: usize,
    number_of_confirmations: usize,
}

#[allow(unused)]
#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionStatusResponse {
    status: usize,
    confirmed: Option<TransactionConfirmedData>,
}
#[pretend]
trait TransactionInfoFetch {
    #[request(method = "GET", path = "/price/{byte_size}")]
    async fn tx_get_price(&self, byte_size: &str) -> pretend::Result<String>;

    #[request(method = "GET", path = "/tx/{id}")]
    async fn tx_get(&self, id: &str) -> pretend::Result<JsonResult<Tx, Error>>;

    #[request(method = "GET", path = "/tx/{id}/status")]
    async fn tx_status(
        &self,
        id: &str,
    ) -> pretend::Result<JsonResult<TransactionStatusResponse, Error>>;
}

pub struct TransactionInfoClient(Pretend<HttpClient, UrlResolver>);

impl TransactionInfoClient {
    pub fn new(url: Url) -> Self {
        let client = HttpClient::default();
        let pretend = Pretend::for_client(client).with_url(url);
        Self(pretend)
    }

    pub async fn get_price(&self, byte_size: &str) -> Result<String, Error> {
        self.0
            .tx_get_price(byte_size)
            .await
            .map_err(|err| Error::TransactionInfoError(err.to_string()))
    }

    pub async fn get(&self, id: &str) -> Result<Tx, Error> {
        self.0
            .tx_get(id)
            .await
            .map(|op| match op {
                JsonResult::Ok(op) => op,
                JsonResult::Err(err) => panic!("Error parsing info {}", err),
            })
            .map_err(|op| Error::TransactionInfoError(op.to_string()))
    }

    pub async fn get_status(&self, id: &str) -> Result<TransactionStatusResponse, Error> {
        let response = self.0.tx_status(id).await.expect("Error getting tx status");
        match response {
            JsonResult::Ok(n) => Ok(n),
            JsonResult::Err(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use httpmock::{Method::GET, MockServer};
    use pretend::Url;
    use tokio_test::block_on;

    use crate::{
        crypto::base64::Base64,
        transaction::get::{
            TransactionConfirmedData, TransactionInfoClient, TransactionStatusResponse, Tx,
        },
    };

    #[test]
    fn test_price() {
        let byte_size = "156";
        let server = MockServer::start();
        let server_url = server.url("");
        let mock = server.mock(|when, then| {
            when.method(GET).path(format!("/price/{}", byte_size));
            then.status(200)
                .header("Content-Type", "application/json")
                .body("123123");
        });

        let url = Url::parse(&server_url).unwrap();
        let client = TransactionInfoClient::new(url);
        let tx_info = block_on(client.get_price(byte_size)).unwrap();

        mock.assert();
        assert_eq!(tx_info, "123123".to_string());
    }

    #[test]
    fn test_get() {
        let id = "id";
        let base64_id = Base64::from_utf8_str(id).unwrap();
        let tx_info_mock = Tx {
            id: base64_id.clone(),
            ..Tx::default()
        };

        let server = MockServer::start();
        let server_url = server.url("");
        let mock = server.mock(|when, then| {
            when.method(GET).path(format!("/tx/{}", id));
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&tx_info_mock).unwrap());
        });

        let url = Url::parse(&server_url).unwrap();
        let client = TransactionInfoClient::new(url);
        let tx_info = block_on(client.get(id)).unwrap();

        mock.assert();
        assert_eq!(tx_info.id, base64_id);
    }

    #[test]
    fn test_get_status() {
        let id = "arweave_tx_id";
        let tx_data_mock = TransactionConfirmedData {
            block_indep_hash: "block_indep_hash".to_string(),
            block_height: 10,
            number_of_confirmations: 10,
        };
        let tx_status_mock = TransactionStatusResponse {
            status: 1,
            confirmed: Some(tx_data_mock),
        };

        let server = MockServer::start();
        let server_url = server.url("");
        let mock = server.mock(|when, then| {
            when.method(GET).path(format!("/tx/{}/status", id));
            then.status(200)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&tx_status_mock).unwrap());
        });

        let url = Url::parse(&server_url).unwrap();
        let client = TransactionInfoClient::new(url);
        let tx_info = block_on(client.get_status(id)).unwrap();

        mock.assert();
        assert_eq!(tx_info.status, 1);
    }
}
