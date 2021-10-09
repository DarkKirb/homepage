//! Streaming Connections (TCP, TLS, QUIC)

use std::{net::IpAddr, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub mod quic;
pub mod tcp;
pub mod tls;

#[async_trait]
pub trait ConnectionHandler {
    async fn handle_connection<RW>(
        &self,
        stream: RW,
        remote_addr: IpAddr,
        alpn: Option<Vec<u8>>,
    ) -> Result<()>
    where
        RW: AsyncRead + AsyncReadExt + AsyncWrite + AsyncWriteExt + Unpin + Send + Sync + 'static;
}

#[async_trait]
impl<C> ConnectionHandler for Arc<C>
where
    C: ConnectionHandler + Send + Sync,
{
    async fn handle_connection<RW>(
        &self,
        stream: RW,
        remote_addr: IpAddr,
        alpn: Option<Vec<u8>>,
    ) -> Result<()>
    where
        RW: AsyncRead + AsyncReadExt + AsyncWrite + AsyncWriteExt + Unpin + Send + Sync + 'static,
    {
        (**self).handle_connection(stream, remote_addr, alpn).await
    }
}
