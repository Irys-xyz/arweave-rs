use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc, Mutex,
};

use crate::{
    consts::{CHUNK_SIZE, DEFAULT_CONCURRENCY_LEVEL, DEFAULT_RETRIES_PER_CHUNK},
    crypto::base64::Base64,
    dynamic_async_queue::DynamicAsyncQueue,
    error::Error,
};
use data_encoding::BASE64URL_NOPAD;
use futures::{pin_mut, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt, StreamExt};
use log::{debug, error};
use pretend::{resolver::UrlResolver, Pretend, Url};
use pretend_reqwest::Client as HttpClient;
use reqwest::header::CONTENT_LENGTH;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct LilChunk<'a> {
    pub chunk: &'a [u8],
}

#[derive(Clone, Debug, Deserialize)]
pub struct Offset {
    pub offset: u64,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub(super) struct DataChunk {
    pub seed_offset: u64,
    pub file_offset: u64,
    pub size: u64,
    pub node: Option<Url>,
    pub chunk: Vec<u8>,
}

pub struct TransactionDataClient(Pretend<HttpClient, UrlResolver>);

impl TransactionDataClient {
    pub fn new(url: Url) -> Self {
        let client = HttpClient::default();
        let pretend = Pretend::for_client(client).with_url(url);
        Self(pretend)
    }

    pub async fn download_tx_data<Output>(
        &self,
        tx: Base64,
        output: &mut Output,
        peers: Vec<Url>,
    ) -> Result<(), Error>
    where
        Output: AsyncWrite + AsyncSeek + Unpin,
    {
        let chunks_indexes = self.index_chunks(peers, tx).await.unwrap();
        let concurrency_level = DEFAULT_CONCURRENCY_LEVEL;

        self.download_chunks(chunks_indexes, output, concurrency_level)
            .await
    }

    async fn get_offset(&self, peer: &Url, tx_id: String) -> Result<Offset, Error> {
        let url = peer
            .join(&format!("/tx/{}/offset", &tx_id))
            .map_err(|err| {
                error!("Failed to build request Url: {:?}", err);
                Error::UrlError
            })
            .expect("Could not join url");

        let res: reqwest::Response = reqwest::get(url).await.unwrap();
        res.json()
            .await
            .map_err(|err| {
                error!("Failed to parse response for offset data: {:?}", err);
                Error::UnknownError
            })
            .map(|Offset { offset, size }| Offset { offset, size })
    }

    async fn index_chunks(&self, peers: Vec<Url>, tx: Base64) -> Result<Vec<DataChunk>, Error> {
        let url = peers.first().expect("Empty peer");

        // TODO: get each chunk respective seeded node
        let Offset { offset, size } = self
            .get_offset(url, tx.to_string())
            .await
            .expect("Could not get offset");

        debug!("Transaction offset={}, size={}", offset, size);
        let start_offset = offset - size + 1;

        let mut file_offset = 0;
        let mut chunks_indexes = Vec::<DataChunk>::new();

        while file_offset < size + 1 && file_offset + CHUNK_SIZE < size {
            let chunk = DataChunk {
                seed_offset: start_offset + file_offset,
                file_offset,
                size: CHUNK_SIZE,
                node: None,
                chunk: vec![],
            };
            debug!("Expecting {:?}", &chunk);
            chunks_indexes.push(chunk);
            file_offset += CHUNK_SIZE;
        }
        let chunk = DataChunk {
            seed_offset: start_offset + file_offset,
            file_offset,
            size: size % CHUNK_SIZE,
            node: None,
            chunk: vec![],
        };
        debug!("Expecting {:?}", &chunk);
        chunks_indexes.push(chunk);
        Ok(chunks_indexes)
    }

    async fn download_chunks<Output>(
        &self,
        chunks_indexes: Vec<DataChunk>,
        output: &mut Output,
        concurrency_level: u16,
    ) -> Result<(), Error>
    where
        Output: AsyncWrite + AsyncSeek + Unpin,
    {
        let expected_chunk_amount = chunks_indexes.len();
        let chunks_indexes = DynamicAsyncQueue::new(chunks_indexes);
        let busy_jobs = Arc::new(AtomicU16::new(0));
        let notifier = chunks_indexes.clone();
        let retries_per_chunk = DEFAULT_RETRIES_PER_CHUNK;

        pin_mut!(busy_jobs);

        let output = Arc::new(Mutex::new(output));
        let chunks = chunks_indexes
            .map(|chunk| {
                busy_jobs.fetch_add(1, Ordering::Relaxed);
                async move {
                    let base_url = chunk.node.clone().expect("No url present for chunk");

                    let url = base_url
                        .join(&format!("/chunk/{}", chunk.seed_offset))
                        .map_err(|err| {
                            error!("Failed to build request Url: {:?}", err);
                            Error::UrlError
                        })
                        .expect("Unable to build url");

                    let mut res = reqwest::get(url.clone()).await;
                    let mut retries = 0;
                    while retries < retries_per_chunk {
                        if let Err(err) = res {
                            error!("Request error: {}", err);
                            debug!("Retrying request, attempt {}", retries);
                            res = reqwest::get(url.clone()).await;
                        } else {
                            break;
                        }
                        retries += 1;
                    }

                    if let Err(err) = res {
                        error!("Request error: {}", err);
                        return None;
                    }

                    let res = res.unwrap();
                    if res.status() == 404 {
                        error!("Chunk {} not found in this peer", chunk.seed_offset);
                    }
                    Some((chunk, res))
                }
            })
            .buffer_unordered(concurrency_level.into())
            .map(|res| {
                let notifier = notifier.clone();
                let busy_jobs = busy_jobs.clone();
                let mut retries = 1;
                async move {
                    res.as_ref()?;
                    let (expected_chunk, mut res) = res.unwrap();
                    let mut fetched_chunk: Option<DataChunk> = None;
                    while fetched_chunk.is_none() && retries <= retries_per_chunk {
                        let content_length: u64 = res
                            .headers()
                            .get(CONTENT_LENGTH)
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .ok_or_else(|| {
                                error!("Could not read chunk size, missing Content-Length header");
                                Error::RequestFailed("Could not read chunk size".to_owned())
                            })
                            .unwrap();

                        // Getting contents
                        let mut buf = Vec::with_capacity(content_length as usize);
                        while let Some(chunk) = match res.chunk().await {
                            Ok(chunk) => chunk,
                            Err(err) => {
                                error!("Failed to read chunk {:?}: {:?}", expected_chunk, err);
                                None
                            }
                        } {
                            buf.write_all(&chunk)
                                .await
                                .map_err(|err| {
                                    error!(
                                        "Failed to write chunk {:?} data to output: {:?}",
                                        expected_chunk, err
                                    );
                                    Error::RequestFailed(err.to_string())
                                })
                                .unwrap();
                        }
                        let chunk: LilChunk = match serde_json::from_slice(buf.as_slice()) {
                            Ok(chunk) => chunk,
                            Err(err) => {
                                error!("Failed to read chunk {:?}: {:?}", expected_chunk, err);
                                LilChunk { chunk: &[] }
                            }
                        };
                        let chunk = BASE64URL_NOPAD.decode(chunk.chunk).unwrap();

                        if chunk.len() == expected_chunk.size as usize {
                            debug!("Got chunk {:?} attempt={}", expected_chunk, retries);
                            fetched_chunk = Some(DataChunk {
                                chunk: vec![],
                                node: None,
                                ..expected_chunk
                            });
                        } else {
                            error!("Err chunk {:?} attempt={}", expected_chunk, retries);
                        }
                        retries += 1;
                    }

                    let busy_jobs = busy_jobs.fetch_sub(1, Ordering::Relaxed);
                    if busy_jobs < 2 {
                        notifier.all_pending_work_done();
                    }
                    fetched_chunk
                }
            })
            .buffer_unordered(concurrency_level.into())
            .filter_map(|n| async {
                if let Some(chunk) = n.clone() {
                    let mut mut_output = output.lock().expect("Failed to acquire lock");
                    let res = mut_output
                        .seek(std::io::SeekFrom::Start(chunk.file_offset))
                        .await
                        .map_err(|err| {
                            error!(
                                "Failed to seek position {}: in file: {}",
                                chunk.file_offset, err
                            );
                            Error::RequestFailed(err.to_string())
                        });
                    if let Err(err) = res {
                        error!("Failed to write chunk {:?}: {}", chunk, err);
                        return None;
                    } else {
                        let _w = mut_output
                            .write_all(&chunk.chunk)
                            .await
                            .map_err(|err| {
                                error!("Failed to write chunk {:?}: {}", chunk, err);
                                Error::UnknownError
                            })
                            .map(|_| {
                                debug!("Wrote chunk {:?}", chunk);
                            });
                        let _f = mut_output.flush();
                    }
                }
                n
            })
            .collect::<Vec<DataChunk>>()
            .await;

        if chunks.len() != expected_chunk_amount {
            error!("{}/{} chunks fetched", chunks.len(), expected_chunk_amount);
            Err(Error::MissingChunks)
        } else {
            debug!("{}/{} chunks fetched", chunks.len(), expected_chunk_amount);
            Ok(())
        }
    }
}
