use std::net::IpAddr;

use crate::Response;

use super::Service;
use anyhow::Result;
use async_trait::async_trait;
use router::Router;
use tokio::io::AsyncRead;
use url::Url;

#[async_trait]
impl Service<Box<dyn AsyncRead + Unpin + Send + Sync + 'static>> for Router {
    #[tracing::instrument]
    async fn handle(
        &self,
        url: Url,
        remote_addr: IpAddr,
    ) -> Result<Response<Box<dyn AsyncRead + Unpin + Send + Sync + 'static>>> {
        match self.handle(url, remote_addr).await? {
            router::Response::Success(mime, res) => Ok(Response::Success(mime, res)),
            router::Response::NotFound => Ok(Response::NotFound),
            _ => Ok(Response::CGIError("Internal Server Error".to_string())),
        }
    }
}
