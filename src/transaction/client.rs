use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    StatusCode,
};
use serde_json::json;
use std::{str::FromStr, thread::sleep, time::Duration};

use crate::{
    consts::{ARWEAVE_BASE_URL, CHUNKS_RETRIES, CHUNKS_RETRY_SLEEP},
    crypto::base64::Base64,
    error::Error,
    types::TxStatus,
};

use super::Tx;

pub struct TxClient {
    client: reqwest::Client,
    base_url: url::Url,
}

impl Default for TxClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: url::Url::from_str(ARWEAVE_BASE_URL).unwrap(),
        }
    }
}

impl TxClient {
    pub fn new(client: reqwest::Client, base_url: url::Url) -> Result<Self, Error> {
        Ok(Self { client, base_url })
    }

    pub async fn post_transaction(&self, signed_transaction: &Tx) -> Result<(Base64, u64), Error> {
        if signed_transaction.id.0.is_empty() {
            return Err(Error::UnsignedTransaction);
        }

        let mut retries = 0;
        let mut status = reqwest::StatusCode::NOT_FOUND;
        let url = self.base_url.join("tx").map_err(Error::UrlParseError)?;

        dbg!(json!(signed_transaction));
        while (retries < CHUNKS_RETRIES) & (status != reqwest::StatusCode::OK) {
            let res = self
                .client
                .post(url.clone())
                .json(&signed_transaction)
                .header(&ACCEPT, "application/json")
                .header(&CONTENT_TYPE, "application/json")
                .send()
                .await
                .map_err(Error::ReqwestError)?;
            status = res.status();
            dbg!(status);
            if status == reqwest::StatusCode::OK {
                return Ok((signed_transaction.id.clone(), signed_transaction.reward));
            }
            sleep(Duration::from_secs(CHUNKS_RETRY_SLEEP));
            retries += 1;
        }

        Err(Error::StatusCodeNotOk)
    }

    pub async fn get_last_tx(&self) -> Result<Base64, Error> {
        let resp = self
            .client
            .get(
                self.base_url
                    .join("tx_anchor")
                    .map_err(Error::UrlParseError)?,
            )
            .send()
            .await
            .map_err(Error::ReqwestError)?;
        let last_tx_str = resp.text().await.unwrap();
        Base64::from_str(&last_tx_str).map_err(Error::Base64DecodeError)
    }

    pub async fn get_fee(&self, target: Base64, data: Vec<u8>) -> Result<u64, Error> {
        let url = self
            .base_url
            .join(&format!("price/{}/{}", data.len(), target))
            .map_err(Error::UrlParseError)?;
        let winstons_per_bytes = reqwest::get(url)
            .await
            .map_err(|e| Error::GetPriceError(e.to_string()))?
            .json::<u64>()
            .await
            .map_err(Error::ReqwestError)?;

        Ok(winstons_per_bytes)
    }

    pub async fn get_tx(&self, id: Base64) -> Result<(StatusCode, Option<Tx>), Error> {
        let res = self
            .client
            .get(
                self.base_url
                    .join(&format!("tx/{}", id))
                    .map_err(Error::UrlParseError)?,
            )
            .send()
            .await
            .map_err(Error::ReqwestError)?;

        if res.status() == StatusCode::OK {
            let text = res.text().await.map_err(Error::ReqwestError)?;
            let tx = Tx::from_str(&text)?;
            return Ok((StatusCode::OK, Some(tx)));
        } else if res.status() == StatusCode::ACCEPTED {
            //Tx is pending
            return Ok((StatusCode::ACCEPTED, None));
        }

        Err(Error::TransactionInfoError(res.status().to_string()))
    }

    pub async fn get_tx_status(&self, id: Base64) -> Result<(StatusCode, Option<TxStatus>), Error> {
        let res = self
            .client
            .get(
                self.base_url
                    .join(&format!("tx/{}/status", id))
                    .map_err(Error::UrlParseError)?,
            )
            .send()
            .await
            .map_err(Error::ReqwestError)?;

        if res.status() == StatusCode::OK {
            let status = res
                .json::<TxStatus>()
                .await
                .map_err(|err| Error::TransactionInfoError(err.to_string()))?;

            Ok((StatusCode::OK, Some(status)))
        } else if res.status() == StatusCode::ACCEPTED {
            Ok((StatusCode::ACCEPTED, None))
        } else {
            Err(Error::TransactionInfoError(res.status().to_string()))
        }
    }
}
