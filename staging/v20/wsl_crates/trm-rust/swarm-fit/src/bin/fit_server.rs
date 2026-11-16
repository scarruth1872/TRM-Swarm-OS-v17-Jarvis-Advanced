//! fit_server — minimal std-only HTTP server for the swarm-fit service.
//! Binds 0.0.0.0:5281, serves /health + five /fit/* POST endpoints,
//! NAND-persists every record via swarm_fit::serve. No external deps.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use swarm_fit::serve;
use swarm_fit::NandFabric;

const KERNEL: &str = "/home/shawn/.hermes/cache/inspired_fitness.k3";
const PORT: u16 = 5281;

fn main() {
    let _ = std::fs::create_dir_all("/home/shawn/.hermes/cache");
    let listener = TcpListener::bind(("0.0.0.0", PORT)).expect("bind 0.0.0.0:5281");
    eprintln!("fit_server listening on 0.0.0.0:{PORT}");
    for stream in listener.incoming() {
        if let Ok(s) = stream {
            let _ = handle(s);
        }
    }
}

fn handle(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    // Read until the header terminator is present.
    let mut header_end = None;
    while header_end.is_none() {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        header_end = buf.windows(4).position(|w| w == b"\r\n\r\n");
    }
    let header_end = match header_end {
        Some(e) => e,
        None => {
            let _ = respond(&mut stream, 400, "{\"error\":\"bad_request\"}");
            return Ok(());
        }
    };
    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    // Content-Length (case-insensitive).
    let mut clen = 0usize;
    for line in head.lines().skip(1) {
        let l = line.trim();
        if let Some(v) = l
            .to_ascii_lowercase()
            .strip_prefix("content-length:")
        {
            clen = v.trim().parse().unwrap_or(0);
        }
    }
    // Read the remaining body.
    let body_start = header_end + 4;
    while buf.len() < body_start + clen {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
    }
    let body = String::from_utf8_lossy(&buf[body_start..body_start + clen]).to_string();

    let req_line = head.lines().next().unwrap_or("").to_string();
    let mut parts = req_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();

    if method == "GET" && path == "/health" {
        return respond(&mut stream, 200, "{\"status\":\"fit-live\"}");
    }
    let entity = match (method.as_str(), path.as_str()) {
        ("POST", "/fit/booking") => "booking",
        ("POST", "/fit/client") => "client",
        ("POST", "/fit/wellness") => "wellness",
        ("POST", "/fit/grant") => "grant",
        ("POST", "/fit/proof") => "proof",
        _ => {
            return respond(&mut stream, 404, "{\"error\":\"not_found\"}");
        }
    };
    let (fields, signals) = match parse_body(&body) {
        Some(x) => x,
        None => {
            return respond(&mut stream, 400, "{\"error\":\"bad_request\"}");
        }
    };
    let field_refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
    let mut fab = match NandFabric::open(KERNEL) {
        Ok(f) => f,
        Err(_) => {
            return respond(&mut stream, 500, "{\"error\":\"fabric_error\"}");
        }
    };
    let (verdict, offset, trits) = serve(&mut fab, entity, &field_refs, &signals);
    let out = format!(
        "{{\"entity\":\"{entity}\",\"verdict\":\"{}\",\"offset\":{offset},\"trits\":{}}}",
        verdict.as_str(),
        trits.len()
    );
    respond(&mut stream, 200, &out)
}

fn respond(stream: &mut TcpStream, code: u16, body: &str) -> std::io::Result<()> {
    let reason = match code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Error",
    };
    let head = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body.as_bytes())?;
    stream.flush()
}

/// Parse `{"fields":["a","b"],"signals":[0.9,0.8]}` by hand (no serde).
fn parse_body(body: &str) -> Option<(Vec<String>, Vec<f32>)> {
    let fields = string_array(body, "fields")?;
    let signals = float_array(body, "signals")?;
    Some((fields, signals))
}

fn key_region<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    let idx = body.find(&format!("\"{key}\""))?;
    let rest = &body[idx..];
    let open = rest.find('[')?;
    let close = rest[open..].find(']')?;
    Some(&rest[open + 1..open + close])
}

fn string_array(body: &str, key: &str) -> Option<Vec<String>> {
    let inner = key_region(body, key)?;
    let bytes = inner.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'"' {
                if bytes[j] == b'\\' {
                    j += 1;
                }
                j += 1;
            }
            if j >= bytes.len() {
                return None;
            }
            out.push(inner[i + 1..j].to_string());
            i = j + 1;
        } else {
            i += 1;
        }
    }
    Some(out)
}

fn float_array(body: &str, key: &str) -> Option<Vec<f32>> {
    let inner = key_region(body, key)?;
    let mut out = Vec::new();
    for tok in inner.split(',') {
        let t = tok.trim();
        if !t.is_empty() {
            out.push(t.parse::<f32>().ok()?);
        }
    }
    Some(out)
}
