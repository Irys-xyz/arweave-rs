use pretend::{Pretend, resolver::UrlResolver, Url};
use pretend_reqwest::Client as HttpClient;

pub struct Client(Pretend<HttpClient, UrlResolver>);

impl Client {
    pub fn new(url: Url) -> Self {
        let client = HttpClient::default();
        let pretend = Pretend::for_client(client).with_url(url);
        Self(pretend)
    }
}