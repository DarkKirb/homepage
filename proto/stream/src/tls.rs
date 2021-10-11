//! TLS Support
//!
//! This module exposes TLS support for the Streams subsystem and is based on the [rustls](https://crates.io/crate/rustls) crate.
//!
//! The [`TlsHandler`] struct can be nested within any Stream, and it is intended that it is used together with [`TcpHandler`].
//!
//! [`TcpHandler`]: ../tcp/struct.TcpHandler.html

use std::{borrow::ToOwned, net::IpAddr, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_rustls::{
    rustls::{ServerConfig, Session},
    TlsAcceptor,
};

use super::ConnectionHandler;

/// [`ConnectionHandler`] for TLS
#[derive(Clone)]
pub struct TlsHandler<H>
where
    H: ConnectionHandler + ?Sized + Send + Sync,
{
    acceptor: TlsAcceptor,
    inner: H,
}

impl<H: ConnectionHandler + Send + Sync + std::fmt::Debug> std::fmt::Debug for TlsHandler<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsHandler")
            .field("acceptor", &"{opaque}")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<H> TlsHandler<H>
where
    H: ConnectionHandler + Send + Sync,
{
    /// Create a new `TlsHandler` from a [`ServerConfig`]
    ///
    /// # Errors
    /// This function may return an error if the state in the [`ServerConfig`] is unsupported. Currently no checks are done.
    pub async fn new(config: Arc<ServerConfig>, inner: H) -> Result<Self> {
        Ok(Self {
            acceptor: TlsAcceptor::from(config),
            inner,
        })
    }
}

#[async_trait]
impl<H> ConnectionHandler for TlsHandler<H>
where
    H: ConnectionHandler + ?Sized + Send + Sync,
{
    #[tracing::instrument(skip(self, stream))]
    async fn handle_connection<RW>(
        &self,
        stream: RW,
        remote_addr: IpAddr,
        orig_proto: Option<Vec<u8>>,
    ) -> Result<()>
    where
        RW: AsyncRead + AsyncReadExt + AsyncWrite + AsyncWriteExt + Unpin + Send + Sync + 'static,
    {
        let acceptor = self.acceptor.accept(stream).await?;
        let server_session = acceptor.get_ref().1;
        let protocol = server_session
            .get_alpn_protocol()
            .map(ToOwned::to_owned)
            .or(orig_proto);
        self.inner
            .handle_connection(acceptor, remote_addr, protocol)
            .await
    }
}
