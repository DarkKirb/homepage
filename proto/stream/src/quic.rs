use std::{
    io::IoSlice,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use pin_project::pin_project;
use quinn::{Connecting, Incoming};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf},
    sync::Mutex,
    task::JoinHandle,
};
use tracing::error;

use super::ConnectionHandler;

#[pin_project]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct MergedStream<R, W>
where
    R: AsyncRead + Unpin + Send + Sync,
    W: AsyncWrite + Unpin + Send + Sync,
{
    #[pin]
    r: R,
    #[pin]
    w: W,
}

impl<R, W> AsyncRead for MergedStream<R, W>
where
    R: AsyncRead + Unpin + Send + Sync,
    W: AsyncWrite + Unpin + Send + Sync,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.project().r.poll_read(cx, buf)
    }
}

impl<R, W> AsyncWrite for MergedStream<R, W>
where
    R: AsyncRead + Unpin + Send + Sync,
    W: AsyncWrite + Unpin + Send + Sync,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        self.project().w.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        self.project().w.poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.project().w.poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize, std::io::Error>> {
        self.project().w.poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.w.is_write_vectored()
    }
}

#[derive(Debug)]
pub struct QuicHandler<H>
where
    H: ConnectionHandler + Send + Sync + ?Sized,
{
    incoming: Mutex<Incoming>,
    inner: H,
}

impl<H> QuicHandler<H>
where
    H: ConnectionHandler + Send + Sync,
{
    pub async fn new(incoming: Incoming, inner: H) -> Result<Self> {
        Ok(Self {
            incoming: Mutex::new(incoming),
            inner,
        })
    }
}

impl<H> QuicHandler<H>
where
    H: ConnectionHandler + Send + Sync + ?Sized + 'static,
{
    async fn accept<RW>(
        self: Arc<Self>,
        stream: RW,
        remote_addr: SocketAddr,
        protocol: Option<Vec<u8>>,
    ) -> Result<JoinHandle<()>>
    where
        RW: AsyncRead + AsyncReadExt + AsyncWrite + AsyncWriteExt + Unpin + Send + Sync + 'static,
    {
        Ok(tokio::spawn(async move {
            let res = || async {
                self.handle_connection(stream, remote_addr.ip(), protocol)
                    .await?;
                Ok(())
            };
            if let Err(e) = res().await {
                sentry_anyhow::capture_anyhow(&e);
                error!("{:?}", e);
            }
        }))
    }
    async fn handle_quic_connection(
        self: Arc<Self>,
        mut connecting: Connecting,
    ) -> Result<JoinHandle<()>> {
        let remote_addr = connecting.remote_address();
        let protocol = connecting.handshake_data().await?.protocol;
        Ok(tokio::spawn(async move {
            let res = || async {
                let mut conn = connecting.await?;

                while let Some(item) = conn.bi_streams.next().await {
                    let item = item?;
                    Arc::clone(&self)
                        .accept(
                            MergedStream {
                                r: item.1,
                                w: item.0,
                            },
                            remote_addr,
                            protocol.clone(),
                        )
                        .await?;
                }

                Ok(())
            };
            if let Err(e) = res().await {
                sentry_anyhow::capture_anyhow(&e);
                error!("{:?}", e);
            }
        }))
    }
    pub async fn listen(self: Arc<Self>) -> Result<()> {
        while let Some(item) = self.incoming.lock().await.next().await {
            Arc::clone(&self).handle_quic_connection(item).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<H> ConnectionHandler for QuicHandler<H>
where
    H: ConnectionHandler + Send + Sync + ?Sized,
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
