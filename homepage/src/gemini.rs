use std::{
    fs::File,
    io::{BufReader, Cursor, Seek, SeekFrom},
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use anyhow::Result;
use async_trait::async_trait;
use proto_gemini::{Response, Service};
use proto_stream::{quic::QuicHandler, tcp::TcpHandler, tls::TlsHandler, ConnectionHandler};
use quinn::Endpoint;
use rustls::{
    ciphersuite::{
        TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
    },
    Certificate, NoClientAuth, PrivateKey, ServerConfig, ServerSessionMemoryCache,
    SupportedCipherSuite,
};
use rustls_pemfile::{pkcs8_private_keys, rsa_private_keys};
use tracing::info;
use url::Url;

static CIPHERSUITES: [&SupportedCipherSuite; 3] = [
    &TLS13_CHACHA20_POLY1305_SHA256,
    &TLS13_AES_256_GCM_SHA384,
    &TLS13_AES_128_GCM_SHA256,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct GeminiTest;

#[async_trait]
impl Service<Cursor<Vec<u8>>> for GeminiTest {
    #[tracing::instrument]
    async fn handle(&self, _: Url, _: IpAddr) -> Result<Response<Cursor<Vec<u8>>>> {
        Ok(Response::Success(
            "text/gemini".to_string(),
            Cursor::new(b"Hewwo!!!\n".to_vec()),
        ))
    }
}

#[tracing::instrument]
pub async fn run_gemini() -> Result<()> {
    info!("Starting up gemini server...");
    let service = GeminiTest;
    let server = Arc::new(proto_gemini::Server::new(service));
    let tls_server = tokio::spawn(run_gemini_tls(Arc::clone(&server)));
    let quic_server = tokio::spawn(run_gemini_quic(Arc::clone(&server)));
    let (tls_server, quic_server) = tokio::try_join!(tls_server, quic_server)?;
    tls_server?;
    quic_server?;
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
async fn run_gemini_tls<C>(handler: C) -> Result<()>
where
    C: ConnectionHandler + Send + Sync + 'static,
{
    let (privkey, certs) = async_read_keys().await?;
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
    let (privkey, certs) = async_read_keys().await?;
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
