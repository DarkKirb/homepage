//! HTTP Protocol support

use std::{
    convert::Infallible,
    io::Cursor,
    net::IpAddr,
    pin::Pin,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
};

use anyhow::Result;
use async_trait::async_trait;
use hyper::{
    body::{Bytes, HttpBody},
    server::conn::Http,
    service::service_fn,
    Body, Request, Response,
};
use pin_project::pin_project;
use proto_stream::ConnectionHandler;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tracing::{error, info};

mod router;

/// type-erased [`AsyncRead`] to [`HttpBody`] bridge
#[pin_project]
pub struct AsyncBody {
    #[pin]
    inner: Box<dyn AsyncRead + Unpin + Send + Sync + 'static>,
    read_buf_owned: Vec<u8>,
}

impl std::fmt::Debug for AsyncBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncBody")
            .field("inner", &"opaque")
            .field("read_buf_owned", &self.read_buf_owned)
            .finish()
    }
}

impl AsyncBody {
    /// Creates a new async body from an async reader
    pub fn new(inner: impl AsyncRead + Unpin + Send + Sync + 'static) -> Self {
        Self {
            inner: Box::new(inner),
            read_buf_owned: vec![0_u8; 65536],
        }
    }
}

impl HttpBody for AsyncBody {
    type Data = Bytes;

    type Error = anyhow::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();
        let mut read_buf = ReadBuf::new(this.read_buf_owned);
        match this.inner.poll_read(cx, &mut read_buf) {
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e.into()))),
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => {
                let filled = read_buf.filled();
                if filled.is_empty() {
                    return Poll::Ready(None);
                }
                Poll::Ready(Some(Ok(Bytes::copy_from_slice(filled))))
            }
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<hyper::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

/// HTTP Service
#[async_trait]
pub trait Service {
    /// Handle HTTP request
    ///
    /// # Arguments
    /// - `remote_addr` - IP Address of Peer, with proxy headers fully resolved
    /// - `request` - The request header and body
    async fn handle(
        &self,
        remote_addr: IpAddr,
        request: Request<Body>,
    ) -> Result<Response<AsyncBody>>;
}

#[async_trait]
impl<T> Service for Arc<T>
where
    T: Service + Send + Sync,
{
    async fn handle(
        &self,
        remote_addr: IpAddr,
        request: Request<Body>,
    ) -> Result<Response<AsyncBody>> {
        (**self).handle(remote_addr, request).await
    }
}

/// HTTP Server
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Server<S>
where
    S: Service,
{
    inner: Arc<S>,
}

impl<S> Server<S>
where
    S: Service,
{
    /// Create a new HTTP server
    ///
    /// # Arguments
    /// - `service` - Service to service requests with
    pub fn new(service: S) -> Self {
        Self {
            inner: Arc::new(service),
        }
    }
}

fn get_req_ip_addr<B: HttpBody>(request: &Request<B>) -> Option<IpAddr> {
    let connecting_ip_str = request
        .headers()
        .get("CF-Connecting-IP")
        .or_else(|| request.headers().get("X-Forwarded-For"));
    IpAddr::from_str(connecting_ip_str?.to_str().ok()?.split(',').next()?).ok()
}

#[async_trait]
impl<S> ConnectionHandler for Server<S>
where
    S: Service + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self, stream))]
    async fn handle_connection<RW>(
        &self,
        stream: RW,
        remote_addr: IpAddr,
        alpn: Option<Vec<u8>>,
    ) -> Result<()>
    where
        RW: AsyncRead + AsyncReadExt + AsyncWrite + AsyncWriteExt + Unpin + Send + Sync + 'static,
    {
        let alpn = alpn.unwrap_or_else(|| b"http/1.1".to_vec());
        let mut http = Http::new();
        if alpn == b"h2" {
            http.http2_only(true);
        } else {
            http.http1_only(true).http1_keep_alive(true);
        }
        let service = Arc::clone(&self.inner);
        http.serve_connection(
            stream,
            service_fn(move |req| {
                let service = Arc::clone(&service);
                async move {
                    let remote_addr = get_req_ip_addr(&req).unwrap_or(remote_addr);
                    info!("Handling Request {} for {}", req.uri(), remote_addr);
                    match service.handle(remote_addr, req).await {
                        Ok(v) => Ok::<_, Infallible>(v),
                        Err(e) => {
                            sentry_anyhow::capture_anyhow(&e);
                            error!("{:?}", e);
                            Ok::<_, Infallible>(
                                Response::builder()
                                    .status(500)
                                    .body(AsyncBody::new(Cursor::new(
                                        b"Internal Server Error".to_vec(),
                                    )))
                                    .unwrap(),
                            )
                        }
                    }
                }
            }),
        )
        .await?;
        Ok(())
    }
}
