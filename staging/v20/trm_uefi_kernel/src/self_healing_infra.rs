//! TRM Swarm OS - self_healing_infra (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/self_healing_infra.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  ServiceConfig 
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub health_endpoint: String,
    pub restart_command: Vec<String>,
    pub port: i64,
    pub heartbeat_timeout: i64,
    pub restart_cooldown: i64,
    pub max_restart_attempts: i64,
}

impl ServiceConfig {
    /// The oracle's `ServiceConfig` declares `heartbeat_timeout: int = 60`,
    /// `restart_cooldown: int = 30` and `max_restart_attempts: int = 3`. This constructor was zeroing
    /// all three, which is not a neutral default: with a timeout of 0 every service is unresponsive the
    /// moment `now` exceeds its heartbeat by anything at all, and with `max_restart_attempts: 0` every
    /// service is immediately STOPPED (code 4) on the evaluator's first clause.
    pub fn new() -> Self {
        ServiceConfig {
            name: String::new(),
            health_endpoint: String::new(),
            restart_command: vec![],
            port: 0,
            heartbeat_timeout: 60,
            restart_cooldown: 30,
            max_restart_attempts: 3,
        }
    }
}

//  ServiceHealth 
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub service_name: String,
    pub status: u64,
    pub last_heartbeat: f64,
    pub last_response_time: f64,
    pub consecutive_failures: i64,
    pub restart_count: i64,
    pub last_restart: f64,
    pub error_message: String,
    pub pid: i64,
}

impl ServiceHealth {
    pub fn new() -> Self {
        ServiceHealth {
            service_name: String::new(),
            status: 0,
            last_heartbeat: 0.0,
            last_response_time: 0.0,
            consecutive_failures: 0,
            restart_count: 0,
            last_restart: 0.0,
            error_message: String::new(),
            pid: 0,
        }
    }
}

//  get_self_healing_infra() 
static SINGLETON_SELF_HEALING_INFRA: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_self_healing_infra() -> u64 {
    // Singleton handle (lazy-init pattern from self_healing_infra.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_SELF_HEALING_INFRA.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SELF_HEALING_INFRA.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ── The oracle's model (swarm_v2/core/self_healing_infra.py + strip_self_healing_infra.rs) ──────
//
// `native_evaluate_services_health` is the module's native evaluator -- the strip's own kernel, with
// f64 heartbeat/timeout/last-restart/cooldown arrays and u32 failure/max-attempt/restart-count arrays:
//
//     status = 4 (STOPPED)        if restart_count >= max_attempts
//            = 2 (UNRESPONSIVE)   elif now - last_heartbeat > heartbeat_timeout
//            = 1 (DEGRADED)       elif consecutive_failures > 2
//            = 0 (HEALTHY)        otherwise
//     eligible_for_restart = (status == 2) && (now - last_restart > cooldown) && (restart_count < max)
//
// Three boundaries are STRICT and are what the port is checked on: `elapsed > timeout` (exactly the
// timeout is NOT unresponsive), `failures > 2` (exactly 2 is NOT degraded), and `rst_elapsed > cooldown`.
//
// Two things worth recording rather than "fixing":
//   * the evaluator only ever emits codes 0, 1, 2 and 4 -- RESTARTING (3) is assigned elsewhere, in the
//     restart path itself, so a status of 3 never comes from this function;
//   * `eligible`'s third clause `restart_count < max_attempts` can never be false when the status is 2,
//     because `restart_count >= max_attempts` would already have made the status 4. It is dead in the
//     oracle's own expression, reproduced here as written.

/// `ServiceStatus`'s numeric codes, as the native evaluator emits them.
pub const SERVICE_HEALTHY: u8 = 0;
pub const SERVICE_DEGRADED: u8 = 1;
pub const SERVICE_UNRESPONSIVE: u8 = 2;
pub const SERVICE_RESTARTING: u8 = 3;
pub const SERVICE_STOPPED: u8 = 4;

/// `native_evaluate_services_health`: (statuses, eligible-for-restart flags).
pub fn evaluate_services_health(
    heartbeat: &[f64],
    timeout: &[f64],
    failures: &[u32],
    last_restart: &[f64],
    cooldown: &[f64],
    max_attempts: &[u32],
    restart_count: &[u32],
    now: f64,
) -> (Vec<u8>, Vec<u8>) {
    let n = heartbeat.len();
    let mut statuses: Vec<u8> = Vec::with_capacity(n);
    let mut eligible: Vec<u8> = Vec::with_capacity(n);
    for i in 0..n {
        let elapsed = now - heartbeat[i];
        let f = failures[i];
        let c = restart_count[i];
        let ma = max_attempts[i];
        let rst_elapsed = now - last_restart[i];
        let st = if c >= ma {
            SERVICE_STOPPED
        } else if elapsed > timeout[i] {
            SERVICE_UNRESPONSIVE
        } else if f > 2 {
            SERVICE_DEGRADED
        } else {
            SERVICE_HEALTHY
        };
        let el = st == SERVICE_UNRESPONSIVE && rst_elapsed > cooldown[i] && c < ma;
        statuses.push(st);
        eligible.push(if el { 1 } else { 0 });
    }
    (statuses, eligible)
}

/// `ServiceStatus`'s name for a code, for reporting.
pub fn status_name(code: u8) -> &'static str {
    match code {
        SERVICE_HEALTHY => "healthy",
        SERVICE_DEGRADED => "degraded",
        SERVICE_UNRESPONSIVE => "unresponsive",
        SERVICE_RESTARTING => "restarting",
        SERVICE_STOPPED => "stopped",
        _ => "unknown",
    }
}

/// `reset_restart_count`: false when the service is not registered (the oracle returns an error dict).
pub fn reset_restart_count(registered: bool, restart_count: &mut u32) -> bool {
    if !registered {
        return false;
    }
    *restart_count = 0;
    true
}

/// `force_restart`: false when the service is not registered, else the restart is initiated.
pub fn force_restart(registered: bool) -> bool {
    registered
}

// ============================================================================
// ORACLE-FAITHFUL ADDITIVE PORT — swarm_v2/core/self_healing_infra.py
// (class SelfHealingInfra, methods `_restart_service` / `_check_service_health`).
//
// `evaluate_services_health` above is the module's *vectorized* native evaluator
// (the strip's own kernel, already ported). What it does NOT carry is the
// sequential state machine the orchestrator actually drives: the pre-spawn
// restart gate and the per-check health transition. Those are ported here.
//
// `last_restart == 0.0` encodes the oracle's `Optional[float] = None` "never
// restarted": in Python `if health.last_restart:` is falsy for BOTH `None` and
// `0.0`, so a zero value means the cooldown check is skipped. Non-zero values are
// truthy and compare normally.
//
// Host-side and therefore NOT ported: the `_ping_service` urllib round-trip, the
// `_find_process_on_port` / `_kill_process_on_port` psutil walks, the
// `subprocess.Popen` spawn (including the PowerShell self-restart helper and
// `os._exit(0)`), `asyncio` loop/sleep, `time.time()`/`datetime.now()` wall
// clocks, `logging`, `os.getenv`/`os.name` default-service registration, and the
// `_self_healing_infra` singleton's process affinity.
//
// Status codes are the evaluator's numeric ones (SERVICE_* above): HEALTHY 0,
// DEGRADED 1, UNRESPONSIVE 2, RESTARTING 3, STOPPED 4. The oracle's
// `ServiceStatus` enum uses string values; the mapping is one-to-one.
// ============================================================================

/// `_restart_service`'s pre-spawn gate decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartDecision {
    Cooldown,
    Stopped,
    Proceed,
}

/// Result of the restart gate: the decision plus the fields the oracle mutates.
#[derive(Debug, Clone, Copy)]
pub struct RestartGate {
    pub decision: RestartDecision,
    pub status: u8,
    pub restart_count: u32,
    pub last_restart: f64,
}

/// `_restart_service`'s pre-spawn gating, verbatim:
///   * cooldown (`now - last_restart < cooldown`) returns first, changing nothing;
///   * max attempts (`restart_count >= max_attempts`) sets STOPPED, else
///   * RESTARTING: `restart_count += 1`, `last_restart = now`.
pub fn restart_gate(
    status: u8,
    restart_count: u32,
    max_attempts: u32,
    last_restart: f64,
    cooldown: f64,
    now: f64,
) -> RestartGate {
    if last_restart != 0.0 {
        let time_since_restart = now - last_restart;
        if time_since_restart < cooldown {
            return RestartGate {
                decision: RestartDecision::Cooldown,
                status,
                restart_count,
                last_restart,
            };
        }
    }
    if restart_count >= max_attempts {
        return RestartGate {
            decision: RestartDecision::Stopped,
            status: SERVICE_STOPPED,
            restart_count,
            last_restart,
        };
    }
    RestartGate {
        decision: RestartDecision::Proceed,
        status: SERVICE_RESTARTING,
        restart_count: restart_count + 1,
        last_restart: now,
    }
}

/// `_check_service_health`'s pure state transition (responsive reset + failure
/// branch). `needs_restart` is the oracle's `time_since_heartbeat > heartbeat_timeout`,
/// where `time_since_heartbeat` is `inf` whenever `last_heartbeat <= 0`.
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub status: u8,
    pub last_heartbeat: f64,
    pub last_response_time: f64,
    pub consecutive_failures: u32,
    pub error_message: String,
    pub needs_restart: bool,
}

/// `_check_service_health`'s pure core. On the restart path the status is left
/// unchanged (the restart gate then sets RESTARTING or STOPPED); otherwise a
/// non-responsive service degrades, and a responsive one resets to HEALTHY with
/// `last_heartbeat = now` and `consecutive_failures = 0`.
pub fn check_service_health(
    status: u8,
    last_heartbeat: f64,
    last_response_time: f64,
    consecutive_failures: u32,
    heartbeat_timeout: f64,
    is_responsive: bool,
    response_time: f64,
    now: f64,
) -> HealthCheck {
    if is_responsive {
        return HealthCheck {
            status: SERVICE_HEALTHY,
            last_heartbeat: now,
            last_response_time: response_time,
            consecutive_failures: 0,
            error_message: String::new(),
            needs_restart: false,
        };
    }
    let failures = consecutive_failures + 1;
    let error_message = format!("Health check failed ({failures} consecutive)");
    let time_since_heartbeat = if last_heartbeat > 0.0 {
        now - last_heartbeat
    } else {
        f64::INFINITY
    };
    let needs_restart = time_since_heartbeat > heartbeat_timeout;
    HealthCheck {
        status: if needs_restart { status } else { SERVICE_DEGRADED },
        last_heartbeat,
        last_response_time,
        consecutive_failures: failures,
        error_message,
        needs_restart,
    }
}
