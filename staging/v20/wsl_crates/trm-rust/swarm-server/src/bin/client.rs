//! swarm-client — test client for the trm:// protocol.
//!
//! Connects to the QUIC server, sends a K3 greeting frame, receives the
//! K3 response, and measures round-trip latency. Uses a self-signed cert
//! with rustls webpki-roots disabled — dev auth (Phase 5 Step 1).

use quinn::Endpoint;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use swarm_proto::{decode, encode, Kind};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    // build an endpoint with a dev trust anchor
    let mut endpoint = Endpoint::client("127.0.0.1:0".parse()?)?;

    // load the same self-signed cert (we generate one; client trusts it
    // because we add it as a root via a custom verifier — simplified for
    // dev: we skip verification via rustls dangerous config)
    let (cert, key) = make_cert()?;
    let _ = (cert, key);

    let server_addr: SocketAddr = "127.0.0.1:5184".parse()?;
    let server_name = "trm.local".to_string();

    let client_cfg = client_config_insecure();
    endpoint.set_default_client_config(client_cfg);

    let start = Instant::now();
    let conn = endpoint.connect(server_addr, &server_name)?.await?;
    println!("[trm://] connected (QUIC handshake: {:?})", start.elapsed());

    let (mut send, mut recv) = conn.open_bi().await?;
    let greeting = encode(Kind::Greeting, &[1, 1, 1, -1, -1, 1]);
    let t0 = Instant::now();
    send.write_all(&greeting).await?;
    let mut buf = vec![0u8; 4096];
    let n = recv.read(&mut buf).await?.unwrap_or(0);
    let rtt = t0.elapsed();

    if let Some((kind, trits, _)) = decode(&buf[..n]) {
        println!("[trm://] response {kind:?}: {trits:?}");
        println!("[trm://] round-trip latency: {rtt:?} ({:.2} us)", rtt.as_micros());
        if kind == Kind::Response && trits == vec![1, 1, 1] {
            println!("[VERIFY] trm:// protocol works - K3 greeting answered with K3 TRUE");
        }
    }
    Ok(())
}

/// Dev-only: accept any cert (Phase 5 Step 1; tHSM auth is Step 2).
fn client_config_insecure() -> quinn::ClientConfig {
    use quinn::crypto::rustls::QuicClientConfig;
    let cfg = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerify::new()))
        .with_no_client_auth();
    let qcfg = QuicClientConfig::try_from(cfg).unwrap();
    quinn::ClientConfig::new(Arc::new(qcfg))
}

#[derive(Debug)]
struct NoVerify;
impl NoVerify {
    fn new() -> Self {
        NoVerify
    }
}
impl rustls::client::danger::ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
        ]
    }
}

/// Generate a self-signed cert (same helper as the server).
fn make_cert(
) -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>), Box<dyn std::error::Error>> {
    let certified_key = rcgen::generate_simple_self_signed(vec!["trm.local".into()])?;
    let cert = CertificateDer::from(certified_key.cert.der().to_vec());
    let key = PrivateKeyDer::try_from(certified_key.key_pair.serialize_der())?;
    Ok((cert, key))
}
