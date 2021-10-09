use std::{io::Cursor, net::IpAddr};

use crate::AsyncBody;

use super::Service;
use anyhow::Result;
use async_trait::async_trait;
use hyper::{Body, Request, Response};
use router::Router;

#[async_trait]
impl Service for Router {
    #[tracing::instrument]
    async fn handle(
        &self,
        remote_addr: IpAddr,
        req: Request<Body>,
    ) -> Result<hyper::Response<AsyncBody>> {
        let uri = req.uri().clone();
        let uri_string = if uri.host().is_none() {
            format!("http://localhost{}", uri.to_string())
        } else {
            uri.to_string()
        };

        let resp = match self.handle(uri_string.parse()?, remote_addr).await? {
            router::Response::Success(mime, data) => Response::builder()
                .status(200)
                .header("Content-Type", mime)
                .body(super::AsyncBody::new(data))?,
            router::Response::NotFound => Response::builder()
                .status(404)
                .body(AsyncBody::new(Cursor::new(b"File not Found")))?,
            router::Response::ServerError => Response::builder()
                .status(500)
                .body(AsyncBody::new(Cursor::new(b"Internal Server Error")))?,
        };
        Ok(resp)
    }
}
