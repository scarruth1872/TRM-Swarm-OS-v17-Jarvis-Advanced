//! Autonomous Healing strips: ChaosSimulator (fault injection), StarMatrix (lattice adjacency),
//! DynamicRouter (least-latency routing).

/// Inject a single-bit fault into the payload at a deterministic position.
pub fn chaos_simulator(input: &str) -> String {
    if input.is_empty() {
        return r#"{"agent":"ChaosSimulator","status":"error","error":"input = payload to perturb"}"#.to_string();
    }
    let mut bytes = input.as_bytes().to_vec();
    let idx = (crate::fnv1a(input) as usize) % bytes.len();
    bytes[idx] ^= 0x01;
    format!(r#"{{"agent":"ChaosSimulator","status":"ok","injected_bit_at":{idx},"kind":"derived"}}"#)
}

/// Build an order-n path lattice adjacency matrix (diagonal + off-diagonal 1) and report its trace.
pub fn star_matrix(input: &str) -> String {
    let n: usize = input.trim().parse().unwrap_or(0);
    if n == 0 || n > 32 {
        return r#"{"agent":"StarMatrix","status":"error","error":"input = lattice order n (1..32)"}"#.to_string();
    }
    let mut adj = vec![vec![0u8; n]; n];
    for i in 0..n {
        for j in 0..n {
            if i == j || (i as isize - j as isize).abs() == 1 {
                adj[i][j] = 1;
            }
        }
    }
    let trace: u32 = (0..n).map(|i| adj[i][i] as u32).sum();
    format!(r#"{{"agent":"StarMatrix","status":"ok","order":{n},"trace":{trace},"kind":"derived"}}"#)
}

/// Least-latency weighted routing. Input: name:latency,...
pub fn dynamic_router(input: &str) -> String {
    let nodes: Vec<(&str, f64)> = input
        .split(',')
        .filter_map(|p| {
            let mut it = p.split(':');
            let name = it.next()?.trim();
            let lat = it.next()?.trim().parse::<f64>().ok()?;
            Some((name, lat))
        })
        .collect();
    if nodes.is_empty() {
        return r#"{"agent":"DynamicRouter","status":"error","error":"input = name:latency,..."}"#.to_string();
    }
    let best = nodes.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
    format!(
        r#"{{"agent":"DynamicRouter","status":"ok","best_node":"{}","latency":{:.3},"kind":"derived"}}"#,
        best.0, best.1
    )
}
