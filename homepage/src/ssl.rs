use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
};

use anyhow::Result;
use rustls::{Certificate, PrivateKey};
use rustls_pemfile::{pkcs8_private_keys, rsa_private_keys};

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
pub async fn async_read_keys() -> Result<(PrivateKey, Vec<Certificate>)> {
    tokio::task::spawn_blocking(read_keys).await?
}
