use pretend::{interceptor::NoopRequestInterceptor, pretend, resolver::UrlResolver, Pretend, Url};

use crate::{client::Client, error::Error};

#[pretend]
trait TransactionInfoFetch {
    #[request(method = "GET", path = "/wallet/{address}/balance")]
    async fn wallet_balance(&self, address: &str) -> pretend::Result<String>;

    #[request(method = "GET", path = "/wallet/{address}/last_tx")]
    async fn wallet_last_tx_id(&self, address: &str) -> pretend::Result<String>;
}

pub struct WalletInfoClient(Pretend<Client, UrlResolver, NoopRequestInterceptor>);

impl WalletInfoClient {
    pub fn new(url: Url) -> Self {
        let client = Client::default();
        let pretend = Pretend::for_client(client).with_url(url);
        Self(pretend)
    }

    pub async fn balance(&self, address: &str) -> Result<String, Error> {
        self.0
            .wallet_balance(address)
            .await
            .map_err(|op| Error::WalletError(op.to_string()))
    }

    pub async fn last_tx_id(&self, address: &str) -> Result<String, Error> {
        self.0
            .wallet_last_tx_id(address)
            .await
            .map_err(|op| Error::WalletError(op.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use httpmock::{Method::GET, MockServer};
    use pretend::Url;
    use tokio_test::block_on;

    use crate::wallet::WalletInfoClient;

    #[test]
    fn test_balance() {
        let address = "address";
        let server = MockServer::start();
        let server_url = server.url("");
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/wallet/{}/balance", address));
            then.status(200)
                .header("Content-Type", "application/json")
                .body("123123");
        });

        let url = Url::parse(&server_url).unwrap();
        let client = WalletInfoClient::new(url);
        let tx_info = block_on(client.balance(address)).unwrap();

        mock.assert();
        assert_eq!(tx_info, "123123".to_string());
    }

    #[test]
    fn test_last_tx() {
        let address = "address";
        let server = MockServer::start();
        let server_url = server.url("");
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/wallet/{}/last_tx", address));
            then.status(200)
                .header("Content-Type", "application/json")
                .body("last_tx");
        });

        let url = Url::parse(&server_url).unwrap();
        let client = WalletInfoClient::new(url);
        let tx_info = block_on(client.last_tx_id(address)).unwrap();

        mock.assert();
        assert_eq!(tx_info, "last_tx".to_string());
    }
}
