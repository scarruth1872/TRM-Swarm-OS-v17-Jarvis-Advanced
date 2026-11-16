//! swarm-server — the trm:// QUIC endpoint.
//!
//! Replaces HTTP/WebSocket: QUIC (UDP) with the K3 ternary framing from
//! swarm-proto. Zero JSON, zero headers, zero HTTP.
//!
//! Phase 5 Step 1: a working trm:// server that accepts a client,
//! exchanges a K3 greeting, answers with a K3 verdict, and measures
//! round-trip latency. Self-signed cert (dev auth) — the tHSM SRAM
//! pattern key exchange is Step 2.

use futures::StreamExt;
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use swarm_fit::NandFabric;
use swarm_proto::{decode, encode, text_to_trits, trits_to_text, Kind};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// NAND kernel for the fit service (byte-verified persistence).
const CHAT_K3: &str = "/home/shawn/.hermes/cache/chat_fabric.k3";
const ABX_K3: &str = "/home/shawn/.hermes/cache/abx_2026.k3";
const JARVIS_K3: &str = "/home/shawn/.hermes/cache/jarvis_fabric.k3";
const SVC_K3: &str = "/home/shawn/.hermes/cache/ecosystem.k3";
const FIT_K3: &str = "/home/shawn/.hermes/cache/inspired_fitness.k3";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = std::env::var("TRM_BIND")
        .unwrap_or_else(|_| "0.0.0.0:5184".into())
        .parse()?;
    let (cert, key) = make_cert()?;

    let server_config = ServerConfig::with_single_cert(vec![cert], key)?;
    let mut endpoint = Endpoint::server(server_config, addr)?;
    println!("[trm://] QUIC server on {addr} (UDP, no HTTP)");

    loop {
        // quinn 0.11: accept() is a Future yielding the next connection
        let conn = match endpoint.accept().await {
            Some(c) => c,
            None => continue,
        };
        let conn = match conn.await {
            Ok(c) => c,
            Err(e) => {
                println!("[trm://] handshake failed: {e}");
                continue;
            }
        };
        println!("[trm://] connection from {}", conn.remote_address());
        tokio::spawn(handle_conn(conn));
    }
}

/// Serve one connection: exchange K3 frames until the peer closes.
async fn handle_conn(conn: quinn::Connection) {
    loop {
        match conn.accept_bi().await {
            Ok((mut send, mut recv)) => {
                let mut buf = vec![0u8; 4096];
                let n = match recv.read(&mut buf).await {
                    Ok(Some(n)) => n,
                    _ => continue,
                };
                if let Some((kind, trits, _)) = decode(&buf[..n]) {
                    println!("[trm://] received {kind:?}: {trits:?}");
                    match kind {
                        Kind::Greeting => {
                            let resp = encode(Kind::Response, &[1, 1, 1]);
                            let _ = send.write_all(&resp).await;
                            let _ = send.finish();
                            println!("[trm://] greeted client (K3 TRUE)");
                        }
                        Kind::Ping => {
                            let pong = encode(Kind::Pong, &[1, 1, 1]);
                            let _ = send.write_all(&pong).await;
                            let _ = send.finish();
                        }
                        _ => {
                            // fit service: text commands (fit:<entity>:<fields>:<signals>)
                            // carried as trits over the same QUIC channel
                            let text = trits_to_text(&trits);
                            match serve_backend(&text).or_else(|| serve_fit(&text)).or_else(|| serve_jarvis_query(&text))
            .or_else(|| serve_recall(&text))
            .or_else(|| serve_chat(&text))
            .or_else(|| serve_svc(&text)).or_else(|| serve_vitals(&text)) {
                                Some(resp_text) => {
                                    let resp =
                                        encode(Kind::Verdict, &text_to_trits(&resp_text));
                                    let _ = send.write_all(&resp).await;
                                    let _ = send.finish();
                                    println!("[trm://] fit: {text} -> {resp_text}");
                                }
                                None => {
                                    let resp = encode(Kind::Response, &[0]);
                                    let _ = send.write_all(&resp).await;
                                    let _ = send.finish();
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => break, // peer closed
        }
    }
}

/// Fit service over trm://: parse `fit:<entity>:<f1|f2|...>:<s1,s2,...>`,
/// persist via swarm_fit::serve, answer `fit:<VERDICT>:<offset>:<trits>`.
fn serve_fit(cmd: &str) -> Option<String> {
    let rest = cmd.strip_prefix("fit:")?;
    // entity is the first field; fields and signals split from the RIGHT
    // (field values may contain ':' — e.g. ISO timestamps — so the last
    // ':' separates the signals from the fields)
    let (entity, tail) = rest.split_once(':')?;
    let (fields_part, signals_part) = tail.rsplit_once(':')?;
    match entity {
        "booking" | "client" | "wellness" | "grant" | "proof" => {}
        _ => return None,
    }
    let fields: Vec<&str> = fields_part.split('|').collect();
    let signals: Vec<f32> = signals_part
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().parse::<f32>().ok())
        .collect::<Option<_>>()?;
    if fields.is_empty() || signals.is_empty() {
        return None;
    }
    // NAND fabric opened per request; append + flush are the persistence guarantee
    let mut fabric = NandFabric::open(FIT_K3).ok()?;
    let (verdict, offset, trits) = swarm_fit::serve(&mut fabric, entity, &fields, &signals);
    Some(format!(
        "fit:{}:{}:{}",
        verdict.as_str(),
        offset,
        trits.len()
    ))
}

/// Ecosystem service over trm://: `svc:<platform>:<f1|f2|...>:<s1,s2,...>`
/// → swarm_services::serve → NAND, answers `svc:<VERDICT>:<offset>:<trits>`.
fn serve_svc(cmd: &str) -> Option<String> {
    let rest = cmd.strip_prefix("svc:")?;
    let (platform, tail) = rest.split_once(':')?;
    let (fields_part, signals_part) = tail.rsplit_once(':')?;
    if !swarm_services::PLATFORMS.contains(&platform) {
        return None;
    }
    let fields: Vec<&str> = fields_part.split('|').collect();
    let signals: Vec<f32> = signals_part
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().parse::<f32>().ok())
        .collect::<Option<_>>()?;
    if fields.is_empty() || signals.is_empty() {
        return None;
    }
    let mut fabric = NandFabric::open(SVC_K3).ok()?;
    let (verdict, offset, trits) = swarm_services::serve(&mut fabric, platform, &fields, &signals);
    Some(format!(
        "svc:{}:{}:{}",
        verdict.as_str(),
        offset,
        trits.len()
    ))
}

/// The live fabric dashboard over trm://: `vitals` → the One's state.
fn serve_vitals(cmd: &str) -> Option<String> {
    // accept both bare "vitals" and the svc-wrapped form
    let cmd = cmd.strip_prefix("svc:").unwrap_or(cmd);
    let base = cmd.split(':').next()?;
    if base.trim() != "vitals" {
        return None;
    }
    // the full 12-strand helix: live probes of every platform + kernel state
    let mut out = swarm_vitals::fused_view();
    let v = swarm_vitals::vitals();
    for f in &v {
        out.push_str(&format!("{}={},", f.name, f.trits));
    }
    out.push_str(&swarm_vitals::family_status(&v));
    Some(out)
}

/// Bidirectional Jarvis: `jarvis:query:<n>` → the last n jarvis-fabric
/// records, read back from the kernel (gap 3 closed).
fn serve_jarvis_query(cmd: &str) -> Option<String> {
    // accept both the bare form and the svc-wrapped form
    let cmd = cmd.strip_prefix("svc:jarvis:query:").or_else(|| cmd.strip_prefix("jarvis:query:"))?;
    let n: u64 = cmd.trim().parse().ok()?;
    let fab = NandFabric::open(JARVIS_K3).ok()?;
    let total = fab.trit_count();
    let start = total.saturating_sub(n * 22);
    let got = fab.read(start, (total - start) as usize).unwrap_or_default();
    let recs = (got.len() / 22) as u64;
    Some(format!("jarvis:recall:{recs}:{total}"))
}

/// Fabric recall: `recall:<fabric>:<n>` → the last n records from a
/// named fabric — the One answers "what did I decide?" natively.
fn serve_recall(cmd: &str) -> Option<String> {
    let rest = cmd.strip_prefix("recall:")?;
    let mut parts = rest.splitn(2, ':');
    let fabric = parts.next()?;
    let n: u64 = parts.next()?.trim().parse().ok()?;
    let path = match fabric {
        "ecosystem" => SVC_K3,
        "fitness" => FIT_K3,
        "jarvis" => JARVIS_K3,
        "abx" => ABX_K3,
        _ => return None,
    };
    let fab = NandFabric::open(path).ok()?;
    let total = fab.trit_count();
    let start_trit = total.saturating_sub(n * 22);
    // read() takes a BYTE offset (2 bits per trit, packed 4 trits/byte)
    let start_byte = start_trit / 4;
    let n_trits = (total - start_trit) as usize;
    let got = fab.read(start_byte, n_trits).unwrap_or_default();
    let recs = (got.len() / 22) as u64;
    Some(format!("recall:{fabric}:{recs}:{total}"))
}

/// The out-of-the-box chat: `chat:<message>` → the REAL intelligence
/// (route → council → persist). No hardcoded answers.
fn serve_chat(cmd: &str) -> Option<String> {
    let msg = cmd.strip_prefix("chat:")?;
    if msg.trim().is_empty() {
        return None;
    }
    let mut fabric = NandFabric::open(CHAT_K3).ok()?;
    let reply = swarm_chat::respond(&mut fabric, msg);
    Some(format!("chat:{reply}"))
}

/// The native backend (Python absorption): `be:<endpoint>[:<args>]`
/// → swarm_backend::serve (real kernel state, no Python interpreter).
fn serve_backend(cmd: &str) -> Option<String> {
    let reply = swarm_backend::serve(cmd)?;
    Some(format!("be:{}:{}:{}", reply.endpoint, reply.status, reply.detail))
}

/// Self-signed cert for the dev server (Phase 5 Step 1 auth).
fn make_cert(
) -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>), Box<dyn std::error::Error>> {
    let certified_key = rcgen::generate_simple_self_signed(vec!["trm.local".into()])?;
    let cert = CertificateDer::from(certified_key.cert.der().to_vec());
    let key = PrivateKeyDer::try_from(certified_key.key_pair.serialize_der())?;
    Ok((cert, key))
}
