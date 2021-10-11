//! TCP Support
//!
//! This module exposes TCP connections and is a thin wrapper around [`TcpListener`]

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

/// [`ConnectionHandler`] for TCP
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
    /// Creates a new `TcpListener` instance
    ///
    /// # Arguments
    /// - `addrs` - Socket Address to listen on
    /// - `inner` - [`ConnectionHandler`] to hand connections to
    ///
    /// # Errors
    /// This function will return an error when [`TcpListener::bind`] returns an error.
    pub async fn new(addrs: impl ToSocketAddrs + Send + Sync, inner: H) -> Result<Self> {
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
    /// Accepts a single TCP connection and spawns a [`ConnectionHandler`] task
    ///
    /// # Errors
    /// This function will return an error if accepting a connection failed.
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

    /// Listens on the TCP Socket indefinitely
    ///
    /// # Errors
    /// This function will return an error if accepting a connection failed
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
