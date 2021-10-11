use std::sync::Arc;

use anyhow::Result;
use proto_stream::{tcp::TcpHandler, tls::TlsHandler, ConnectionHandler};
use router::Router;
use rustls::{
    ciphersuite::{
        TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
    },
    NoClientAuth, ServerConfig, ServerSessionMemoryCache, SupportedCipherSuite,
};
use tracing::info;

static CIPHERSUITES: [&SupportedCipherSuite; 3] = [
    &TLS13_CHACHA20_POLY1305_SHA256,
    &TLS13_AES_256_GCM_SHA384,
    &TLS13_AES_128_GCM_SHA256,
];

#[tracing::instrument]
pub async fn run_http(service: Arc<Router>) -> Result<()> {
    info!("Starting up HTTP server...");
    let server = Arc::new(proto_http::Server::new(service));
    tokio::select! {
        tcp = tokio::spawn(run_http_tcp(Arc::clone(&server))) => {tcp??}
        tls = tokio::spawn(run_http_tls(Arc::clone(&server))) => {tls??}
    };
    Ok(())
}

#[tracing::instrument(skip(handler))]
async fn run_http_tcp<C>(handler: C) -> Result<()>
where
    C: ConnectionHandler + Send + Sync + 'static,
{
    info!("Listening for HTTP connections on port 3002");
    Arc::new(TcpHandler::new("0.0.0.0:3002", handler).await?)
        .listen()
        .await?;
    Ok(())
}

#[tracing::instrument(skip(handler))]
async fn run_http_tls<C>(handler: C) -> Result<()>
where
    C: ConnectionHandler + Send + Sync + 'static,
{
    let (privkey, certs) = crate::ssl::async_read_keys().await?;
    let mut server_config = ServerConfig::with_ciphersuites(NoClientAuth::new(), &CIPHERSUITES);

    server_config.set_persistence(ServerSessionMemoryCache::new(128));
    server_config.set_single_cert(certs, privkey)?;
    server_config.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);

    info!("Listening for HTTPS connections on port 3003");
    Arc::new(
        TcpHandler::new(
            "0.0.0.0:3003",
            TlsHandler::new(Arc::new(server_config), handler).await?,
        )
        .await?,
    )
    .listen()
    .await?;

    Ok(())
}
