use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use proto_stream::{quic::QuicHandler, tcp::TcpHandler, tls::TlsHandler, ConnectionHandler};
use quinn::Endpoint;
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
pub async fn run_gemini(service: Arc<Router>) -> Result<()> {
    info!("Starting up gemini server...");
    let server = Arc::new(proto_gemini::Server::new(service));
    tokio::select!(
        tls = tokio::spawn(run_gemini_tls(Arc::clone(&server))) => {tls?}
        quic = tokio::spawn(run_gemini_quic(Arc::clone(&server))) => {quic?}
    )?;
    Ok(())
}

#[tracing::instrument(skip(handler))]
async fn run_gemini_tls<C>(handler: C) -> Result<()>
where
    C: ConnectionHandler + Send + Sync + 'static,
{
    let (privkey, certs) = crate::ssl::async_read_keys().await?;
    let mut server_config = ServerConfig::with_ciphersuites(NoClientAuth::new(), &CIPHERSUITES);

    server_config.set_persistence(ServerSessionMemoryCache::new(128));
    server_config.set_single_cert(certs, privkey)?;
    server_config.set_protocols(&[b"gemini".to_vec()]);

    info!("Listening for gemini connections on port 1965");
    Arc::new(
        TcpHandler::new(
            "0.0.0.0:1965",
            TlsHandler::new(Arc::new(server_config), handler).await?,
        )
        .await?,
    )
    .listen()
    .await?;

    Ok(())
}

#[tracing::instrument(skip(handler))]
async fn run_gemini_quic<C>(handler: C) -> Result<()>
where
    C: ConnectionHandler + Send + Sync + 'static,
{
    let (privkey, certs) = crate::ssl::async_read_keys().await?;
    let socket_addr: SocketAddr = "0.0.0.0:1965".parse()?;

    info!("Listening for QUIC gemini connections on port 1965");
    let mut server = quinn::ServerConfig::default();
    server.certificate(certs.into(), quinn::PrivateKey::from_der(&privkey.0)?)?;

    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.listen(server);
    let (_endpoint, incoming) = endpoint_builder.bind(&socket_addr)?;

    Arc::new(QuicHandler::new(incoming, handler).await?)
        .listen()
        .await?;

    Ok(())
}
