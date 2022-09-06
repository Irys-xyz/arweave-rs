use pretend::{pretend, Result};

#[pretend]
trait TransactionInfoFetch {
    #[request(method = "GET", path = "/wallet/{address}/balance")]
    async fn wallet_balance(&self, address: &str) -> Result<String>;

    #[request(method = "GET", path = "/wallet/{address}/last_tx")]
    async fn wallet_last_tx_id(&self, address: &str) -> Result<String>;
}