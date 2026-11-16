//! TRM Swarm OS - host supervisor (Rust).
//!
//! Replaces swarm_supervisor.py, the last Python process in the runtime control path.
//! Ported 1:1 from the Python contract, including the two rules that made it correct:
//!
//!   1. A service declared with an EMPTY cmd is "defined but not started" -- Aura Studio and
//!      the retired Python shard are in this class. It is reported, never resurrected. The
//!      Python original carried an explicit fix for this: an adopted service whose process
//!      handle is None could never satisfy the crash branch, so it warned once and went
//!      OFFLINE forever. Resurrecting a cmd=[] service would be worse than leaving it down.
//!   2. A restart is capped: restart_count < max_restarts. Unbounded restarts turn a
//!      permanently broken service into a fork bomb.
//!
//! Config is pipe-delimited so the crate needs no JSON dependency and stays buildable with a
//! single rustc invocation on any host -- the same property the voltage strips rely on.
//!
//! Grammar (one service per line, '#' comments, blank lines ignored):
//!   key | name | port | health_url | cwd | argv (space separated; EMPTY = never started)

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// One supervised service. Deliberately not Clone: it owns a Child handle, and duplicating a
/// process handle would let two code paths believe they own the same child.
pub struct Service {
    pub key: String,
    pub name: String,
    pub port: u16,
    pub health_url: String,
    pub cwd: String,
    pub argv: Vec<String>,
    pub restart_delay_ms: u64,
    pub max_restarts: u32,
    pub process: Option<Child>,
    pub restart_count: u32,
    pub status: String,
    pub last_ping_ms: u128,
    /// When this service was last restarted, in epoch ms. 0 = never restarted by this supervisor.
    /// Recorded so a reader can measure the cooldown window instead of assuming it has elapsed.
    pub last_restart_ms: u128,
}

impl Service {
    /// True when this service has a command to run. A service with argv == [] is DEFINED BUT
    /// NOT STARTED by design and must never be spawned, no matter how long its port stays dark.
    pub fn is_startable(&self) -> bool {
        !self.argv.is_empty()
    }
}

pub fn parse_config(text: &str) -> Result<Vec<Service>, String> {
    let mut out = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if f.len() < 6 {
            return Err(format!("line {}: expected 6 pipe-separated fields, got {}", n + 1, f.len()));
        }
        let port: u16 = f[2].parse().map_err(|_| format!("line {}: port '{}' is not a u16", n + 1, f[2]))?;
        let argv: Vec<String> = if f[5].is_empty() {
            Vec::new()
        } else {
            f[5].split(' ').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect()
        };
        out.push(Service {
            key: f[0].to_string(),
            name: f[1].to_string(),
            port,
            health_url: f[3].to_string(),
            cwd: f[4].to_string(),
            argv,
            restart_delay_ms: 3000,
            max_restarts: 10,
            process: None,
            restart_count: 0,
            status: "STOPPED".to_string(),
            last_ping_ms: 0,
            last_restart_ms: 0,
        });
    }
    Ok(out)
}

pub fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Is something listening on this port? A TCP connect is the honest test: it says "a process is
/// accepting connections here" without assuming what protocol it speaks.
pub fn port_active(port: u16) -> bool {
    match format!("127.0.0.1:{}", port).parse::<SocketAddr>() {
        Ok(addr) => TcpStream::connect_timeout(&addr, Duration::from_millis(900)).is_ok(),
        Err(_) => false,
    }
}

/// HTTP GET over a raw socket for services that expose a health route. Returns the status code
/// when one is parseable. Deliberately minimal: no redirects, no chunked handling, no TLS.
pub fn http_status(url: &str) -> Option<u16> {
    let rest = url.strip_prefix("http://")?;
    let (hostport, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let addr: SocketAddr = hostport.parse().ok()?;
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_millis(900)).ok()?;
    s.set_read_timeout(Some(Duration::from_millis(1200))).ok()?;
    let req = format!("GET {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\n\r\n", path, hostport);
    s.write_all(req.as_bytes()).ok()?;
    let mut buf = String::new();
    let _ = s.take(4096).read_to_string(&mut buf);
    buf.lines().next()?.split_whitespace().nth(1)?.parse().ok()
}

/// Start a service. Only ever called for is_startable() services -- never for cmd=[] ones.
pub fn start(svc: &mut Service, log: &mut dyn Write) {
    let (prog, args) = match svc.argv.split_first() {
        Some((p, a)) => (p.clone(), a.to_vec()),
        None => return,
    };
    let mut c = Command::new(&prog);
    c.args(&args).stdout(Stdio::null()).stderr(Stdio::null());
    if !svc.cwd.is_empty() {
        c.current_dir(PathBuf::from(&svc.cwd));
    }
    match c.spawn() {
        Ok(child) => {
            let _ = writeln!(log, "    started {} (pid {})", svc.name, child.id());
            svc.process = Some(child);
            svc.status = "STARTING".to_string();
        }
        Err(e) => {
            let _ = writeln!(log, "    FAILED to start {}: {}", svc.name, e);
            svc.status = "FAILED".to_string();
        }
    }
}

/// One supervision pass, mirroring the Python contract's two-branch heal:
///   - a service WITH a command that is not answering gets restarted, capped by max_restarts;
///   - a service WITHOUT a command is reported and left alone, permanently. It is not broken;
///     it is declared. Resurrecting it is the bug the Python original carried a fix for.
///
/// `dry` reports what a pass WOULD do without spawning anything. A diagnostic that starts
/// services is not a diagnostic: `--once` spawned a duplicate kernel during a live swap
/// precisely because it still ran the start branch.
pub fn tick(services: &mut [Service], log: &mut dyn Write, dry: bool) {
    for svc in services.iter_mut() {
        if port_active(svc.port) {
            svc.status = "ONLINE".to_string();
            svc.last_ping_ms = now_ms();
            continue;
        }
        if !svc.is_startable() {
            if svc.status != "OFFLINE" {
                let _ = writeln!(log, "    {} :{} dark -- declared, not started (cmd empty), never resurrected",
                                 svc.name, svc.port);
            }
            svc.status = "OFFLINE".to_string();
            continue;
        }
        let died = match svc.process.as_mut() {
            Some(ch) => matches!(ch.try_wait(), Ok(Some(_))),
            None => true,
        };
        // Only a process that has actually EXITED is a crash. A live child whose
        // port is not up yet (still loading the fabric) is STALE -- restarting it
        // spawns a duplicate that panics on the bind ("address already in use").
        if !died {
            svc.status = "STALE".to_string();
            continue;
        }
        svc.status = "CRASHED".to_string();
        if dry {
            let _ = writeln!(log, "    {} :{} {} -- dry run, NOT started", svc.name, svc.port, svc.status);
            continue;
        }
        if svc.restart_count < svc.max_restarts {
            svc.restart_count += 1;
            let _ = writeln!(log, "    restarting {} (attempt {}/{})", svc.name, svc.restart_count, svc.max_restarts);
            svc.last_restart_ms = now_ms();
            std::thread::sleep(Duration::from_millis(svc.restart_delay_ms));
            start(svc, log);
        } else {
            let _ = writeln!(log, "    {} exhausted {} restarts; leaving it down", svc.name, svc.max_restarts);
        }
    }
}

/// Status as JSON, written by hand. The dashboard and the kernel read this file. There is no HTTP
/// server here, deliberately -- the Python original had none either.
pub fn status_json(services: &[Service], interval_s: u64, running: bool) -> String {
    let items: Vec<String> = services
        .iter()
        .map(|s| {
            format!(
                "{{\"key\":\"{}\",\"name\":\"{}\",\"port\":{},\"status\":\"{}\",\"restarts\":{},\"startable\":{},\"last_ping_ms\":{},\"last_restart_ms\":{},\"max_restarts\":{}}}",
                s.key, s.name, s.port, s.status, s.restart_count, s.is_startable(),
                s.last_ping_ms, s.last_restart_ms, s.max_restarts
            )
        })
        .collect();
    format!(
        "{{\"services\":[{}],\"checked_ms\":{},\"interval_s\":{},\"running\":{}}}",
        items.join(","),
        now_ms(),
        interval_s,
        running
    )
}

/// Apply one control action written by the kernel to `supervisor.control.json`, then consume the
/// file so the action runs exactly once. The kernel writes a single-action JSON object:
///   {"action":"pause"}   {"action":"resume"}   {"action":"reset"}
///   {"action":"restart","key":"rust_kernel"}
/// The crate stays dependency-free, so the action is recognised by substring rather than parsed.
fn apply_control(path: &str, services: &mut [Service], paused: &mut bool, log: &mut dyn Write) {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return, // no control file = no pending action
    };
    if text.contains("\"pause\"") {
        *paused = true;
        let _ = writeln!(log, "[control] watchdog paused (heal loop suspended)");
    } else if text.contains("\"resume\"") {
        *paused = false;
        let _ = writeln!(log, "[control] watchdog resumed");
    } else if text.contains("\"reset\"") {
        for s in services.iter_mut() {
            s.restart_count = 0;
        }
        let _ = writeln!(log, "[control] restart counters reset to 0");
    } else if text.contains("\"restart\"") {
        let key = extract_json_string(&text, "key");
        if let Some(svc) = services.iter_mut().find(|s| s.key == key) {
            if let Some(child) = svc.process.as_mut() {
                let _ = child.kill();
                let _ = child.wait();
            }
            svc.process = None;
            svc.status = "CRASHED".to_string();
            // A manual restart is not a failure: do not let it eat the restart budget.
            svc.restart_count = svc.restart_count.saturating_sub(1);
            let _ = writeln!(log, "[control] force-restart requested for {} ({})", svc.name, key);
        } else {
            let _ = writeln!(log, "[control] restart requested for unknown key '{}'", key);
        }
    } else {
        let _ = writeln!(log, "[control] unrecognised control file content; ignoring");
    }
    let _ = std::fs::remove_file(path);
}

/// Extract the string value of a JSON key (e.g. `"key":"rust_kernel"` -> `rust_kernel`).
fn extract_json_string(text: &str, key: &str) -> String {
    let needle = format!("\"{}\":\"", key);
    let Some(i) = text.find(&needle) else {
        return String::new();
    };
    let rest = &text[i + needle.len()..];
    match rest.find('"') {
        Some(j) => rest[..j].to_string(),
        None => String::new(),
    }
}

fn arg(args: &[String], flag: &str) -> Option<String> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cfg_path = arg(&args, "--config").unwrap_or_else(|| "supervisor.services.conf".to_string());
    let once = args.iter().any(|a| a == "--once");
    let interval = arg(&args, "--interval").and_then(|v| v.parse::<u64>().ok()).unwrap_or(10);

    // Single-instance guard. Two supervisors both seeing a service's port dark will both spawn it --
    // which is exactly how a duplicate kernel process appeared twice during live swaps. Holding a
    // bound socket is the guard, and it beats a pid file: the OS releases the socket when this
    // process dies, so there is never a stale lock to clean up after a crash. Only the continuous
    // mode takes it, so a --once diagnostic can still run against a supervised platform.
    let _instance = if once {
        None
    } else {
        let lock_port = arg(&args, "--lock-port")
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8023);
        match std::net::TcpListener::bind(("127.0.0.1", lock_port)) {
            Ok(l) => Some(l),
            Err(_) => {
                eprintln!(
                    "supervisor-rs: another supervisor already holds :{}; refusing to run a second. \
                     Two supervisors race and spawn duplicate services -- stop the running one first.",
                    lock_port
                );
                std::process::exit(2);
            }
        }
    };

    let text = match std::fs::read_to_string(&cfg_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("supervisor-rs: cannot read {}: {}", cfg_path, e);
            std::process::exit(2);
        }
    };
    let mut services = match parse_config(&text) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("supervisor-rs: bad config: {}", e);
            std::process::exit(2);
        }
    };

    // A supervisor that dies because its log directory is missing is not a supervisor. Create the
    // parent, and if the log still cannot be opened, say so on stderr and carry on with stdout
    // rather than panicking -- losing the log is survivable, losing the supervisor is not.
    let mut log: Box<dyn Write> = match arg(&args, "--log") {
        Some(p) => {
            if let Some(parent) = std::path::Path::new(&p).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            match std::fs::File::create(&p) {
                Ok(f) => Box::new(f),
                Err(e) => {
                    eprintln!("supervisor-rs: cannot open log {}: {} -- logging to stdout", p, e);
                    Box::new(std::io::stdout())
                }
            }
        }
        None => Box::new(std::io::stdout()),
    };

    let declared = services.iter().filter(|s| !s.is_startable()).count();

    // The status file is anchored to the CONFIG's directory, not to the process's cwd.
    //
    // A relative write went to whatever directory the supervisor happened to be launched from, so a
    // second, stale supervisor.status.json sat in the repo root while the live one was written
    // elsewhere -- and a reader could not tell which was current. That ambiguity cost a full
    // diagnostic cycle: a 50-minute-old file was mistaken for a hung supervisor, when the process was
    // ticking normally every interval. Anchoring to the config makes the location a property of the
    // DEPLOYMENT rather than of the shell that started it. --status overrides.
    let status_path = arg(&args, "--status").unwrap_or_else(|| {
        std::path::Path::new(&cfg_path)
            .parent()
            .filter(|d| !d.as_os_str().is_empty())
            .map(|d| d.join("supervisor.status.json"))
            .unwrap_or_else(|| std::path::PathBuf::from("supervisor.status.json"))
            .to_string_lossy()
            .into_owned()
    });

    let _ = writeln!(log, "[supervisor-rs] {} services from {}", services.len(), cfg_path);
    let _ = writeln!(log, "[supervisor-rs] {} declared services (cmd empty) will be reported, never spawned", declared);
    let _ = writeln!(log, "[supervisor-rs] status file: {}", status_path);

    // First pass adopts whatever is already listening, so a live platform is never double-started.
    tick(&mut services, &mut log, once);

    // The control file is anchored to the config's directory, same as the status file, so the kernel
    // writes its start/stop/restart signal where this supervisor is guaranteed to read it regardless
    // of how it was launched.
    let control_path = {
        let base = std::path::Path::new(&cfg_path).parent();
        match base {
            Some(d) if !d.as_os_str().is_empty() => {
                d.join("supervisor.control.json").to_string_lossy().into_owned()
            }
            _ => "supervisor.control.json".to_string(),
        }
    };

    let mut paused = false;

    loop {
        let json = status_json(&services, interval, !paused);
        // --once must not overwrite the live supervisor's status file. A diagnostic that mutates the
        // state it is diagnosing is not a diagnostic -- it reports to its own log and exits.
        if once {
            let _ = writeln!(log, "{}", json);
            break;
        }
        let _ = std::fs::write(&status_path, &json);
        std::thread::sleep(Duration::from_secs(interval));
        apply_control(&control_path, &mut services, &mut paused, &mut log);
        if !paused {
            tick(&mut services, &mut log, false);
        }
    }
}
