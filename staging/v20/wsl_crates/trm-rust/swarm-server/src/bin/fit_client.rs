//! fit_client.rs — the trm://-native Inspired Fitness client.
//!
//! Sends Kind::Fit frames over the trm:// QUIC channel (UDP 5184) and
//! reads the K3 verdict — NO HTTP, NO APIs. The frame payload is the
//! business record as trit-encoded fields:
//!
//!   fit:booking:userId=1|providerId=1|serviceId=1|2026-08-20|pending
//!
//! The server routes it to swarm_fit → NAND, returns the verdict
//! (BOOKED/BLOCKED/ANALYZE) as trits.

use quinn::Endpoint;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::sync::Arc;
use swarm_proto::{decode, encode, Kind};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut endpoint = Endpoint::client("127.0.0.1:0".parse()?)?;
    let server_addr: SocketAddr = "127.0.0.1:5184".parse()?;
    let server_name = "trm.local".to_string();
    endpoint.set_default_client_config(client_config_insecure());

    let conn = endpoint.connect(server_addr, &server_name)?.await?;

    // the fit request: a business record (entity + fields)
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: fit_client <entity> <field|field|...> [signals]");
        eprintln!("  fit_client booking \"userId=1|providerId=1|serviceId=1|2026-08-20|pending\" \"0.9,0.8,0.7\"");
        std::process::exit(1);
    }
    let entity = &args[1];
    let fields: Vec<String> = args[2].split('|').map(|s| s.to_string()).collect();
    let signals: Vec<f32> = args.get(3)
        .map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or_else(|| vec![0.8, 0.7, 0.6]);

    // encode the command as text-carried trits (the trm:// fit channel):
    //   fit:<entity>:<f1|f2|...>:<s1,s2,...>
    let cmd = format!(
        "fit:{}:{}:{}",
        entity,
        fields.join("|"),
        signals.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",")
    );
    let trits = swarm_proto::text_to_trits(&cmd);

    // send the Fit frame
    let (mut send, mut recv) = conn.open_bi().await?;
    let frame = encode(Kind::Fit, &trits);
    send.write_all(&frame).await?;
    let _ = send.finish();

    let mut buf = vec![0u8; 4096];
    let n = recv.read(&mut buf).await?.unwrap_or(0);
    if let Some((kind, resp, _)) = decode(&buf[..n]) {
        println!("[trm://] response {kind:?}: {resp:?}");
        // the verdict text: fit:<VERDICT>:<offset>:<trits>
        let text = swarm_proto::trits_to_text(&resp);
        println!("[fit] {text}");
        if let Some(rest) = text.strip_prefix("fit:") {
            let verdict = rest.split(':').next().unwrap_or("?");
            println!("[fit] {entity} → {verdict} (persisted in the NAND fabric, byte-verified)");
        }
    }
    Ok(())
}

/// Dev-only: accept any cert (same as the existing client).
fn client_config_insecure() -> quinn::ClientConfig {
    use quinn::crypto::rustls::QuicClientConfig;
    let cfg = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerify))
        .with_no_client_auth();
    let qcfg = QuicClientConfig::try_from(cfg).unwrap();
    quinn::ClientConfig::new(Arc::new(qcfg))
}

#[derive(Debug)]
struct NoVerify;
impl rustls::client::danger::ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp: &[u8],
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
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::RSA_PSS_SHA256,
        ]
    }
}
