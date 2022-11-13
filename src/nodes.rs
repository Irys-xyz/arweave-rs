use std::{
    collections::HashMap,
    str::FromStr,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use crate::{dynamic_async_queue::DynamicAsyncQueue, error::Error, network::NetworkInfoClient};
use futures::{pin_mut, FutureExt, StreamExt};
use log::{debug, error};
use pretend::{resolver::UrlResolver, Pretend, Url};
use pretend_reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum PeerProcessingStatus {
    Pending,
    Ok,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Node(pub String);

pub struct NodeClient(Pretend<HttpClient, UrlResolver>, Url);

impl NodeClient {
    pub fn new(url: Url) -> Self {
        let client = HttpClient::default();
        let pretend = Pretend::for_client(client).with_url(url.clone());
        Self(pretend, url)
    }

    pub async fn find_nodes(
        &self,
        concurrency_level: u16,
        timeout: Duration,
        max_depth: Option<usize>,
        max_count: Option<usize>,
    ) -> Result<Vec<Node>, Error> {
        let max_count = max_count.unwrap_or(100);
        let max_depth = max_depth.unwrap_or(3);
        let concurrency_level = concurrency_level as usize;

        debug!(
            "Find nodes, max_depth={}, max_count={}, req_timeout={:?}, concurrency_level={}",
            max_depth, max_count, timeout, concurrency_level
        );

        let cache: Arc<Mutex<HashMap<Node, PeerProcessingStatus>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let gateway_node = match (self.1.domain(), self.1.port()) {
            (Some(domain), Some(port)) => Ok(Node(format!("{}:{}", domain, port))),
            (Some(domain), None) => Ok(Node(domain.to_string())),
            (None, _) => {
                error!("Domain for Arweave gateway's URL cannot be empty");
                Err(Error::UnknownError)
            }
        }?;

        let network_info_client = NetworkInfoClient::new(self.1.clone());
        let unchecked_nodes: Vec<(Node, usize)> = network_info_client
            .peers(None)
            .await
            .map(|peers| peers.into_iter().map(|peer| (peer, 0)).collect())?;

        {
            let mut cache = cache.lock().expect("Failed to acquire lock");

            cache.insert(gateway_node.clone(), PeerProcessingStatus::Ok);

            unchecked_nodes.iter().for_each(|(node, _)| {
                cache.insert(node.clone(), PeerProcessingStatus::Pending);
            });
        }

        let unchecked_nodes = DynamicAsyncQueue::new(unchecked_nodes);
        let busy_jobs = Arc::new(AtomicU16::new(0));
        let unchecked_nodes_notifier = unchecked_nodes.clone();

        let good_nodes = {
            let cache = cache.clone();
            pin_mut!(unchecked_nodes_notifier);
            pin_mut!(busy_jobs);
            pin_mut!(cache);

            unchecked_nodes
                .map(|(node, depth)| {
                    let url = Url::from_str(&format!("http://{}", node.0)).expect("Invalid node url");
                    busy_jobs.fetch_add(1, Ordering::Relaxed);
                    debug!("Fetch peers for node={:?}, depth={}", node, depth,);
                    network_info_client
                        .peers(Some(url))
                        .map(move |res| (node, depth, res))
                })
                .buffer_unordered(concurrency_level)
                .filter_map(|(node, depth, res)| {
                    let cache = cache.clone();
                    let unchecked_nodes_notifier = unchecked_nodes_notifier.clone();
                    let busy_jobs = busy_jobs.clone();
                    async move {
                        let ret = match res {
                            Ok(peers) => {
                                let mut cache = cache.lock().expect("Failed to acquire lock");

                                *cache
                                    .get_mut(&node)
                                    .expect("Failed to find node from cache") =
                                    PeerProcessingStatus::Ok;

                                if depth < max_depth {
                                    let new_nodes: Vec<(Node, usize)> = peers
                                        .iter()
                                        .filter(|peer| !cache.contains_key(peer))
                                        .cloned()
                                        .map(|node| (node, depth + 1))
                                        .collect();

                                    new_nodes.iter().for_each(|(node, _)| {
                                        cache.insert(node.clone(), PeerProcessingStatus::Pending);
                                    });

                                    debug!(
                                        "Found good node {:?}, with {} peers, {} new",
                                        node,
                                        peers.len(),
                                        new_nodes.len()
                                    );

                                    unchecked_nodes_notifier.add_items(new_nodes);
                                } else {
                                    debug!(
                                        "Found good node {:?}, with {} peers, maximum depth reached",
                                        node,
                                        peers.len()
                                    );
                                }
                                Some((node, peers))
                            }
                            Err(_err) => {
                                let mut status = cache.lock().expect("Failed to acquire lock");
                                *status
                                    .get_mut(&node)
                                    .expect("Failed to find node from cache") =
                                    PeerProcessingStatus::Failed;
                                None
                            }
                        };
                        let busy_jobs = busy_jobs.fetch_sub(1, Ordering::Relaxed);
                        // busy_jobs here is the previous value before decrementing
                        if busy_jobs < 2 {
                            unchecked_nodes_notifier.all_pending_work_done();
                        }
                        ret
                    }
                })
                .take(max_count)
                .collect::<Vec<(Node, Vec<Node>)>>()
                .await
        };

        let mut cache = Arc::try_unwrap(cache)
            .expect("Cache not freed yet, while should be")
            .into_inner()
            .expect("Failed to unwrap mutex");

        // Remove gateway from the returned results
        cache.remove(&gateway_node);

        debug!(
            "found {} nodes, {} good",
            cache.keys().len(),
            good_nodes.len()
        );

        Ok(good_nodes.into_iter().fold(vec![], |mut acc, (node, _)| {
            acc.push(node);
            acc
        }))
    }
}
