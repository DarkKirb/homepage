//! TCP Connection Handling code

use std::{net::IpAddr, sync::Arc};

use super::ConnectionHandler;
use anyhow::Result;
use async_trait::async_trait;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, ToSocketAddrs},
    task::JoinHandle,
};
use tracing::error;

#[derive(Debug)]
pub struct TcpHandler<H>
where
    H: ConnectionHandler + ?Sized + Send + Sync,
{
    listener: TcpListener,
    inner: H,
}

impl<H> TcpHandler<H>
where
    H: ConnectionHandler + Send + Sync,
{
    pub async fn new(addrs: impl ToSocketAddrs, inner: H) -> Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addrs).await?,
            inner,
        })
    }
}

impl<H> TcpHandler<H>
where
    H: ConnectionHandler + Send + Sync + ?Sized + 'static,
{
    #[tracing::instrument(skip(self))]
    pub async fn accept(self: Arc<Self>) -> Result<JoinHandle<()>> {
        let (socket, addr) = self.listener.accept().await?;
        Ok(tokio::spawn(async move {
            if let Err(e) = self.handle_connection(socket, addr.ip(), None).await {
                sentry_anyhow::capture_anyhow(&e);
                error!("{:?}", e);
            }
        }))
    }

    pub async fn listen(self: Arc<Self>) -> Result<()> {
        loop {
            Arc::clone(&self).accept().await?;
        }
    }
}

#[async_trait]
impl<H> ConnectionHandler for TcpHandler<H>
where
    H: ConnectionHandler + ?Sized + Send + Sync,
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
        self.inner
            .handle_connection(stream, remote_addr, alpn)
            .await
    }
}
