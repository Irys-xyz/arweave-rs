// Mostly duplicated pretend_reqwest, because pretend_reqwest causes
// request to pull defeault features that forces us to pull openssl
// and we want to use rustls-tls instead of native-tls.

use std::mem;

use async_trait::async_trait;
use pretend::{client::Bytes, Error, HeaderMap, Response, Result};
use reqwest::Method;
use url::Url;

#[derive(Default)]
pub struct Client(reqwest::Client);

#[async_trait]
impl pretend::client::Client for Client {
    async fn execute(
        &self,
        method: Method,
        url: Url,
        headers: HeaderMap,
        body: Option<Bytes>,
    ) -> Result<Response<Bytes>> {
        let mut builder = self.0.request(method, url).headers(headers);
        if let Some(body) = body {
            builder = builder.body(body);
        }
        let response = builder.send().await;
        let mut response = response.map_err(Error::response)?;

        let status = response.status();
        let headers = mem::take(response.headers_mut());

        let bytes = response.bytes().await;
        let bytes = bytes.map_err(Error::body)?;

        Ok(Response::new(status, headers, bytes))
    }
}
