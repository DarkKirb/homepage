//! Stream Prototype Crate
//!
//! This crate exposes an unified interface to stream-like interfaces. It also implements that interface for the TCP, TLS and QUIC protocols.
//!
//! The most important trait is the [`ConnectionHandler`] trait, which is used by the protocol implementations to receive connections.
//!
//! ```rust
//! use std::{net::IpAddr};
//!
//! use anyhow::Result;
//! use async_trait::async_trait;
//! use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
//!
//! use proto_stream::{ConnectionHandler};
//!
//! pub struct HelloWorldHandler;
//!
//! #[async_trait]
//! impl ConnectionHandler for HelloWorldHandler {
//!     async fn handle_connection<RW>(
//!         &self,
//!         mut stream: RW,
//!         remote_addr: IpAddr,
//!         alpn: Option<Vec<u8>>,
//!     ) -> Result<()>
//!     where
//!         RW: AsyncRead + AsyncReadExt + AsyncWrite + AsyncWriteExt + Unpin + Send + Sync + 'static,
//!     {
//!         let msg = format!("Hello {}!\n\n", remote_addr);
//!         stream.write_all(msg.as_bytes()).await?;
//!         Ok(())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let handler = HelloWorldHandler;
//!     let (mut client, server) = tokio::io::duplex(64);
//!     handler.handle_connection(server, IpAddr::from([127, 0, 0, 1]), None).await?;
//!     let mut out_buf = Vec::new();
//!     client.read_to_end(&mut out_buf).await?;
//!     assert_eq!(out_buf, b"Hello 127.0.0.1!\n\n");
//!     Ok(())
//! }
//! ```

use std::{net::IpAddr, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub mod quic;
pub mod tcp;
pub mod tls;

/// Connection Handler Trait for
///
/// Each stream takes a Connection Handler to handle bidirectional streaming connections. The ConnectionHandler's job is to parse and handle the inner protocol.
#[async_trait]
pub trait ConnectionHandler {
    /// Handle a new Connection
    ///
    /// # Arguments
    /// - `stream` - Bidirectional stream, representing a socket.
    /// - `remote_addr` - IP Address of the peer. 127.0.0.1 in unix sockets.
    /// - `alpn` - Application-Level Protocol Negotiation. Some Stream providers (like TLS or QUIC) may offer a mechanism to negotiate the protocol at an application level. The result of this negotiation is exposed here
    ///
    /// # Type Arguments
    /// `RW` - Type of the stream
    ///
    /// # Errors
    /// This function may return an error. If the protocols allows for error reporting to the client, the Connection Handler should instead do error handling by itself. An example of that would be HTTP Status Code 500.
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
