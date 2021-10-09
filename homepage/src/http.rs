use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
    sync::Arc,
};

use anyhow::Result;
use proto_stream::{tcp::TcpHandler, tls::TlsHandler, ConnectionHandler};
use router::Router;
use rustls::{
    ciphersuite::{
        TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
    },
    Certificate, NoClientAuth, PrivateKey, ServerConfig, ServerSessionMemoryCache,
    SupportedCipherSuite,
};
use rustls_pemfile::{pkcs8_private_keys, rsa_private_keys};
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
    let tcp_server = tokio::spawn(run_http_tcp(Arc::clone(&server)));
    let tls_server = tokio::spawn(run_http_tls(Arc::clone(&server)));
    let (tcp_server, tls_server) = tokio::try_join!(tcp_server, tls_server)?;
    tcp_server?;
    tls_server?;
    Ok(())
}

#[tracing::instrument]
fn read_keys() -> Result<(PrivateKey, Vec<Certificate>)> {
    let privkey_path = std::env::var("PRIVKEY_PATH")?;
    let mut privkey_file = BufReader::new(File::open(privkey_path)?);
    let privkey = pkcs8_private_keys(&mut privkey_file)
        .or_else(|_| {
            privkey_file.seek(SeekFrom::Start(0))?;
            rsa_private_keys(&mut privkey_file)
        })?
        .pop()
        .unwrap();
    let cert_path = std::env::var("CERT_PATH")?;
    let mut certs_file = BufReader::new(File::open(cert_path)?);
    let certs = rustls_pemfile::certs(&mut certs_file)?;

    Ok((
        PrivateKey(privkey),
        certs.into_iter().map(Certificate).collect(),
    ))
}

#[tracing::instrument]
async fn async_read_keys() -> Result<(PrivateKey, Vec<Certificate>)> {
    tokio::task::spawn_blocking(read_keys).await?
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
    let (privkey, certs) = async_read_keys().await?;
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
