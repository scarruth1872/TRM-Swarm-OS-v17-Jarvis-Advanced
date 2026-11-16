//! swarm-bench — concurrent-user load bench for the trm:// QUIC core.
//!
//! Opens N concurrent QUIC connections, each sending a K3 greeting and
//! receiving the K3 TRUE response, then measures:
//!   - total wall time, aggregate connections/s
//!   - per-request RTT min/avg/p99/max
//!   - error count (honest — errors reported, not hidden)
//!
//! Usage: swarm-bench [connections]

use quinn::Endpoint;
use rustls::pki_types::CertificateDer;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use swarm_proto::{decode, encode, Kind};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n_conns: usize = std::env::args().nth(1).map(|s| s.parse().unwrap_or(1000)).unwrap_or(1000);
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_bench(n_conns))
}

async fn run_bench(n_conns: usize) -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let mut endpoint = Endpoint::client("127.0.0.1:0".parse()?)?;
    endpoint.set_default_client_config(client_config_insecure());
    let server_addr: SocketAddr = std::env::var("TRM_SERVER").unwrap_or_else(|_| "127.0.0.1:5184".into()).parse()?;

    println!("[bench] trm:// concurrent bench: {n_conns} connections");
    let t0 = Instant::now();

    // open all connections concurrently
    let mut conns = Vec::with_capacity(n_conns);
    for _ in 0..n_conns {
        let conn = endpoint.connect(server_addr, "trm.local")?.await?;
        conns.push(conn);
    }
    let connect_time = t0.elapsed();
    println!("[bench] {n_conns} QUIC handshakes: {connect_time:?} ({:.0}/s)",
             n_conns as f64 / connect_time.as_secs_f64());

    // fire one K3 greeting per connection, measure each RTT
    let t1 = Instant::now();
    let mut rtts: Vec<f64> = Vec::with_capacity(n_conns);
    let mut errors = 0usize;
    let mut handles = Vec::with_capacity(n_conns);
    for conn in conns {
        handles.push(tokio::spawn(async move {
            let t = Instant::now();
            let mut out = Err("no stream");
            if let Ok((mut send, mut recv)) = conn.open_bi().await {
                let greeting = encode(Kind::Greeting, &[1, 1, 1, -1, -1, 1]);
                if send.write_all(&greeting).await.is_ok() {
                    let mut buf = vec![0u8; 4096];
                    if let Ok(Some(n)) = recv.read(&mut buf).await {
                        if let Some((kind, trits, _)) = decode(&buf[..n]) {
                            if kind == Kind::Response && trits == vec![1, 1, 1] {
                                out = Ok(());
                            }
                        }
                    }
                }
            }
            (t.elapsed().as_secs_f64() * 1e6, out)
        }));
    }
    for h in handles {
        match h.await {
            Ok((rtt_us, Ok(()))) => rtts.push(rtt_us),
            _ => errors += 1,
        }
    }
    let total = t1.elapsed();

    rtts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg = rtts.iter().sum::<f64>() / rtts.len().max(1) as f64;
    let p99 = rtts[(rtts.len() as f64 * 0.99) as usize].min(*rtts.last().unwrap_or(&0.0));
    let max = *rtts.last().unwrap_or(&0.0);
    let min = *rtts.first().unwrap_or(&0.0);

    println!("[bench] {} requests, {} errors ({:.1}%)",
             rtts.len(), errors, errors as f64 / n_conns as f64 * 100.0);
    println!("[bench] total: {total:?} | {:.0} req/s",
             rtts.len() as f64 / total.as_secs_f64());
    println!("[bench] RTT us: min {min:.0} | avg {avg:.0} | p99 {p99:.0} | max {max:.0}");

    if errors == 0 && rtts.len() == n_conns {
        println!("[VERIFY] trm:// concurrent bench PASSED - {n_conns} connections served, zero errors");
    } else {
        println!("[VERIFY] bench had errors - honest report above");
    }
    Ok(())
}

/// Dev-only: accept any cert (same as client).
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
