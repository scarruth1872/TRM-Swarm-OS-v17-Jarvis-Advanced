//! svc_client.rs — the trm://-native ecosystem client (all platforms).
//!
//! Sends Kind::Fit frames with `svc:<platform>:<fields>:<signals>`
//! commands over the trm:// QUIC channel — the whole family absorbed:
//! jarvis, agentq, aura, hera7, voicebox, bonsai, embed.
//!
//! usage: svc_client <platform> "<f1|f2|...>" "<s1,s2,...>"

use quinn::Endpoint;
use rustls::pki_types::CertificateDer;
use std::net::SocketAddr;
use std::sync::Arc;
use swarm_proto::{decode, encode, Kind, text_to_trits, trits_to_text};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut endpoint = Endpoint::client("127.0.0.1:0".parse()?)?;
    let server_addr: SocketAddr = "127.0.0.1:5184".parse()?;
    endpoint.set_default_client_config(client_config_insecure());

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: svc_client <platform> \"<f1|f2|...>\" \"<s1,s2,...>\"");
        eprintln!("  svc_client jarvis \"hand=monitor|status=ran\" \"0.9,0.8,0.7\"");
        std::process::exit(1);
    }
    let platform = &args[1];
    let fields = &args[2];
    let signals = &args[3];

    // the query form: `jarvis:query:N`, `recall:...`, `vitals` → bare command
    let cmd = if platform.starts_with("be:")
        || platform.starts_with("chat:")
        || platform.starts_with("jarvis:query:")
        || platform.starts_with("recall:")
        || platform.starts_with("vitals") {
        format!("{platform}")
    } else {
        format!("svc:{platform}:{fields}:{signals}")
    };
    let trits = text_to_trits(&cmd);

    let conn = endpoint.connect(server_addr, "trm.local")?.await?;
    let (mut send, mut recv) = conn.open_bi().await?;
    let frame = encode(Kind::Fit, &trits);
    send.write_all(&frame).await?;
    let _ = send.finish();

    let mut buf = vec![0u8; 4096];
    let n = recv.read(&mut buf).await?.unwrap_or(0);
    if let Some((kind, resp, _)) = decode(&buf[..n]) {
        println!("[trm://] response {kind:?}");
        let text = trits_to_text(&resp);
        println!("[svc] {text}");
        if let Some(rest) = text.strip_prefix("svc:") {
            let verdict = rest.split(':').next().unwrap_or("?");
            println!("[svc] {platform} → {verdict} (persisted in the fabric, byte-verified)");
        } else if text.starts_with("be:") || text.starts_with("jarvis:recall:")
            || text.starts_with("vitals:") || text.starts_with("recall:") {
            println!("[svc] {text}");
        }
    }
    Ok(())
}

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
