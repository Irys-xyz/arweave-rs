use std::{str::FromStr, thread::sleep, time::Duration};

use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client,
};

use crate::{
    consts::{ARWEAVE_BASE_URL, CHUNKS_RETRIES, CHUNKS_RETRY_SLEEP},
    error::Error,
    types::Chunk,
};

pub struct Uploader {
    url: url::Url,
}

impl Default for Uploader {
    fn default() -> Self {
        let url = url::Url::from_str(ARWEAVE_BASE_URL).unwrap();
        Self { url }
    }
}

impl Uploader {
    pub fn new(url: url::Url) -> Self {
        Uploader { url }
    }

    pub async fn post_chunk_with_retries(
        &self,
        chunk: Chunk,
        client: Client,
    ) -> Result<usize, Error> {
        let mut retries = 0;
        let mut resp = self.post_chunk(&chunk, &client).await;

        while retries < CHUNKS_RETRIES {
            match resp {
                Ok(offset) => return Ok(offset),
                Err(e) => {
                    dbg!("post_chunk_with_retries: {:?}", e);
                    sleep(Duration::from_secs(CHUNKS_RETRY_SLEEP));
                    retries += 1;
                    resp = self.post_chunk(&chunk, &client).await;
                }
            }
        }
        resp
    }

    pub async fn post_chunk(&self, chunk: &Chunk, client: &Client) -> Result<usize, Error> {
        let url = self.url.join("chunk").expect("Could not join url");
        // let client = reqwest::Client::new();

        let resp = client
            .post(url)
            .json(&chunk)
            .header(&ACCEPT, "application/json")
            .header(&CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(|e| Error::PostChunkError(e.to_string()))?;

        match resp.status() {
            reqwest::StatusCode::OK => Ok(chunk.offset),
            _ => Err(Error::StatusCodeNotOk),
        }
    }
}
