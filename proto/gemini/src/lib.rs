//! Gemini Protocol Implementation
use std::{marker::PhantomData, net::IpAddr, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use proto_stream::ConnectionHandler;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::{error, info, warn};
use url::Url;

mod router;

fn is_valid_gemini_url(url: &Url) -> bool {
    if url.scheme() != "gemini" {
        return false;
    }
    if !url.has_authority() {
        return false;
    }
    if url.username() != "" {
        return false;
    }
    if url.password().is_some() {
        return false;
    }

    true
}

/// Gemini Service Trait
#[async_trait]
pub trait Service<R>
where
    R: AsyncRead + AsyncReadExt + Unpin,
{
    /// Handle a Gemini Service Request
    ///
    /// # Arguments
    /// - `url` - Requested URL
    /// - `remote_addr` - Address of peer
    ///
    /// # Errors
    /// This function returns an error if handling a request fails
    async fn handle(&self, url: Url, remote_addr: IpAddr) -> Result<Response<R>>;
}

#[async_trait]
impl<R, T> Service<R> for Arc<T>
where
    R: AsyncRead + AsyncReadExt + Unpin,
    T: Service<R> + Send + Sync,
{
    async fn handle(&self, url: Url, remote_addr: IpAddr) -> Result<Response<R>> {
        (**self).handle(url, remote_addr).await
    }
}

/// Gemini Server
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Server<S, R>
where
    S: Service<R>,
    R: AsyncRead + AsyncReadExt + Unpin,
{
    _phantom: PhantomData<R>,
    inner: S,
}

impl<S, R> Server<S, R>
where
    S: Service<R>,
    R: AsyncRead + AsyncReadExt + Unpin + Send + Sync,
{
    /// Create a new Gemini Server from a [`Service`]
    pub fn new(service: S) -> Self {
        Self {
            _phantom: PhantomData,
            inner: service,
        }
    }
}

fn rstrip(mut buf: &[u8]) -> &[u8] {
    while buf[buf.len() - 1].is_ascii_whitespace() {
        buf = &buf[..(buf.len() - 1)];
    }
    buf
}

#[async_trait]
impl<S, R> ConnectionHandler for Server<S, R>
where
    S: Service<R> + Send + Sync,
    R: AsyncRead + AsyncReadExt + Unpin + Send + Sync,
{
    #[tracing::instrument(skip(self, stream))]
    async fn handle_connection<RW>(
        &self,
        mut stream: RW,
        remote_addr: IpAddr,
        _: Option<Vec<u8>>,
    ) -> Result<()>
    where
        RW: AsyncRead + AsyncReadExt + AsyncWrite + AsyncWriteExt + Unpin + Send + Sync,
    {
        let mut buffer = vec![0_u8; 1026];
        let req_len = stream.read(&mut buffer).await?;
        let req = rstrip(&buffer[..req_len]);
        let url_result = (|| Ok(Url::parse(std::str::from_utf8(req)?)?))();
        match url_result {
            Ok(url) => {
                let response = self.handle(url, remote_addr).await?;
                response.write(stream).await?;
            }
            Err(e) => {
                sentry_anyhow::capture_anyhow(&e);
                error!("{:?}", e);
                Response::<R>::BadRequest.write(stream).await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<S, R> Service<R> for Server<S, R>
where
    S: Service<R> + Send + Sync,
    R: AsyncRead + AsyncReadExt + Unpin + Send + Sync,
{
    #[tracing::instrument(skip(self))]
    async fn handle(&self, url: Url, remote_addr: IpAddr) -> Result<Response<R>> {
        info!("Handling Request {} for {}", url, remote_addr);
        if !is_valid_gemini_url(&url) {
            warn!("Malformed gemini url {} found!", url);
            return Ok(Response::BadRequest);
        }
        Ok(self
            .inner
            .handle(url, remote_addr)
            .await
            .unwrap_or_else(|e| {
                sentry_anyhow::capture_anyhow(&e);
                error!("{:?}", e);
                Response::CGIError("Internal Server Error".to_string())
            }))
    }
}

/// Gemini Response
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Response<R>
where
    R: AsyncRead + Unpin + AsyncReadExt,
{
    /// An Input is required from the user. String contains the prompt
    Input(String),
    /// A Password is required from the. String contains the prompt
    SensitiveInput(String),
    /// Success. The String contains the mime type, and the Reader contains the contents
    Success(String, R),
    /// Temporary Redirect
    TemporaryRedirect(Url),
    /// Permanent Redirect
    PermanentRedirect(Url),
    /// Temporary Failure
    TemporaryFailure(String),
    /// Server Unavailable
    ServerUnavailable(String),
    /// CGI error
    CGIError(String),
    /// Proxy error
    ProxyError(String),
    /// Slow Down
    SlowDown(u32),
    /// Permanent Failure
    PermanentFailure(String),
    /// Not Found
    NotFound,
    /// Gone
    Gone,
    /// Proxy Service Refused
    ProxyServiceRefused,
    /// Client made a bad request
    BadRequest,
}

impl<R> Response<R>
where
    R: AsyncRead + Unpin + AsyncReadExt + Send,
{
    /// Write a gemini response to an Async writer
    ///
    /// # Errors
    /// This function returns an error if writing to the writer failed
    pub async fn write<W>(self, mut writer: W) -> Result<()>
    where
        W: AsyncWrite + AsyncWriteExt + Unpin + Send,
    {
        match self {
            Response::Input(prompt) => {
                writer.write_all(b"10 ").await?;
                writer.write_all(prompt.as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }
            Response::SensitiveInput(prompt) => {
                writer.write_all(b"11 ").await?;
                writer.write_all(prompt.as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }
            Response::Success(mime, mut reader) => {
                writer.write_all(b"20 ").await?;
                writer.write_all(mime.as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
                let mut buf = vec![0; 65536];
                loop {
                    let length = reader.read(&mut buf).await?;
                    writer.write_all(&buf[..length]).await?;
                    if length != 65536 {
                        break;
                    }
                }
            }
            Response::TemporaryRedirect(dest) => {
                writer.write_all(b"30 ").await?;
                writer.write_all(dest.as_str().as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }
            Response::PermanentRedirect(dest) => {
                writer.write_all(b"31 ").await?;
                writer.write_all(dest.as_str().as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }
            Response::TemporaryFailure(msg) => {
                writer.write_all(b"40 ").await?;
                writer.write_all(msg.as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }
            Response::ServerUnavailable(msg) => {
                writer.write_all(b"41 ").await?;
                writer.write_all(msg.as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }
            Response::CGIError(msg) => {
                writer.write_all(b"42 ").await?;
                writer.write_all(msg.as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }
            Response::ProxyError(msg) => {
                writer.write_all(b"43 ").await?;
                writer.write_all(msg.as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }
            Response::SlowDown(msg) => {
                writer.write_all(b"44 ").await?;
                writer.write_all(format!("{}", msg).as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }
            Response::PermanentFailure(msg) => {
                writer.write_all(b"50 ").await?;
                writer.write_all(msg.as_bytes()).await?;
                writer.write_all(b"\r\n").await?;
            }
            Response::NotFound => {
                writer.write_all(b"51 Not Found\r\n").await?;
            }
            Response::Gone => {
                writer.write_all(b"52 Gone\r\n").await?;
            }
            Response::ProxyServiceRefused => {
                writer.write_all(b"53 Proxy Request Refused\r\n").await?;
            }
            Response::BadRequest => {
                writer.write_all(b"59 Bad Request\r\n").await?;
            }
        };
        Ok(())
    }
}
