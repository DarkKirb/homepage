//! TLS Support

use std::{net::IpAddr, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};

use super::ConnectionHandler;

pub struct TlsHandler<H>
where
    H: ConnectionHandler + ?Sized + Send + Sync,
{
    acceptor: TlsAcceptor,
    inner: H,
}

impl<H> TlsHandler<H>
where
    H: ConnectionHandler + Send + Sync,
{
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
    async fn handle_connection<RW>(&self, stream: RW, remote_addr: IpAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncReadExt + AsyncWrite + AsyncWriteExt + Unpin + Send + Sync,
    {
        let acceptor = self.acceptor.accept(stream).await?;
        self.inner.handle_connection(acceptor, remote_addr).await
    }
}
