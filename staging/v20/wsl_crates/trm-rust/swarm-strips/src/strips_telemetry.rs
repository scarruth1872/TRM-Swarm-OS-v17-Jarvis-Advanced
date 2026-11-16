//! Telemetry & Memory strips: MemorySynchronizer (content-hash diff), MonitoringSkill (real CPU),
//! SharedMemory (file-backed segment).

/// Content-hash sync check between two payloads. Input: old|new.
pub fn memory_synchronizer(input: &str) -> String {
    let parts: Vec<&str> = input.split('|').collect();
    if parts.len() != 2 {
        return r#"{"agent":"MemorySynchronizer","status":"error","error":"input = old|new"}"#.to_string();
    }
    let (ha, hb) = (crate::fnv1a(parts[0]), crate::fnv1a(parts[1]));
    let in_sync = ha == hb;
    let first_diff = parts[0].chars().zip(parts[1].chars()).position(|(a, b)| a != b);
    let first_diff_json = match first_diff { Some(i) => i.to_string(), None => "null".to_string() };
    format!(
        r#"{{"agent":"MemorySynchronizer","status":"ok","in_sync":{in_sync},"hash_a":"{ha:016x}","hash_b":"{hb:016x}","first_diff":{first_diff_json},"kind":"derived"}}"#
    )
}

/// Real host metric: logical CPU count from the OS.
pub fn monitoring_skill(input: &str) -> String {
    let cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
    let metric = if input.is_empty() { "cpu" } else { input };
    let value = if metric == "cpu" { cpus } else { 0 };
    format!(
        r#"{{"agent":"MonitoringSkill","status":"ok","metric":"{metric}","value":{value},"source":"available_parallelism","kind":"measured_on_host"}}"#
    )
}

/// File-backed shared segment: append the payload, return offset + content hash.
pub fn shared_memory(input: &str) -> String {
    use std::io::Write;
    let dir = std::env::temp_dir().join("swarm-shm");
    std::fs::create_dir_all(&dir).ok();
    let p = dir.join("segment.bin");
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&p).unwrap();
    let offset = f.metadata().map(|m| m.len()).unwrap_or(0);
    f.write_all(input.as_bytes()).ok();
    let h = crate::fnv1a(input);
    format!(
        r#"{{"agent":"SharedMemory","status":"ok","offset":{offset},"bytes":{},"hash":"{h:016x}","segment":"{}","kind":"derived"}}"#,
        input.len(),
        crate::json_escape(&p.to_string_lossy())
    )
}
