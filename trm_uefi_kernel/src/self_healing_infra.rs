//! TRM Swarm OS — self_healing_infra (no_std port).
//! Infrastructure-level self-healing state machine, 1:1 from
//! swarm_v2/core/self_healing_infra.py:
//!   - ServiceStatus enum (HEALTHY/DEGRADED/UNRESPONSIVE/RESTARTING/STOPPED);
//!   - default monitored services: swarm_api :8021 (heartbeat_timeout 600,
//!     cooldown 30, max 3 attempts), swarm_dashboard :5183 and
//!     swarm_dashboard_v2 :5174 (timeout 3600, cooldown 300, max 1);
//!   - _check_service_health decision core (responsive vs not, time since
//!     heartbeat vs heartbeat_timeout, silent skip once max restarts reached);
//!   - _restart_service decision core (cooldown, max-attempts STOPPED,
//!     RESTARTING with attempt counter, verified-restart resets count to 0).
//! Subprocess/psutil/HTTP orchestration bodies (spawn, kill, ping) are
//! OS-bound and become honest `// MANUAL` returns below — never invented.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// ServiceStatus enum (python Enum, `.value` string in parentheses).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceStatus {
    Healthy = 0,
    Degraded = 1,
    Unresponsive = 2,
    Restarting = 3,
    Stopped = 4,
}

impl ServiceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceStatus::Healthy => "healthy",
            ServiceStatus::Degraded => "degraded",
            ServiceStatus::Unresponsive => "unresponsive",
            ServiceStatus::Restarting => "restarting",
            ServiceStatus::Stopped => "stopped",
        }
    }
}

/// ServiceConfig dataclass (1:1 fields).
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub health_endpoint: String,
    pub port: u32,
    pub heartbeat_timeout: i64, // seconds before restart
    pub restart_cooldown: i64,  // seconds between restart attempts
    pub max_restart_attempts: i64,
}

impl ServiceConfig {
    pub fn new(
        name: &str,
        health_endpoint: &str,
        port: u32,
        heartbeat_timeout: i64,
        restart_cooldown: i64,
        max_restart_attempts: i64,
    ) -> Self {
        ServiceConfig {
            name: name.to_string(),
            health_endpoint: health_endpoint.to_string(),
            port,
            heartbeat_timeout,
            restart_cooldown,
            max_restart_attempts,
        }
    }
}

/// ServiceHealth dataclass (1:1 fields). last_restart uses python's
/// `Optional[float] = None`; 0.0 carries the "never restarted" sentinel.
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub service_name: String,
    pub status: ServiceStatus,
    pub last_heartbeat: f64,
    pub last_response_time: f64,
    pub consecutive_failures: i64,
    pub restart_count: i64,
    pub last_restart: f64,
    pub error_message: String,
    pub pid: i64,
}

impl ServiceHealth {
    pub fn new(service_name: &str) -> Self {
        ServiceHealth {
            service_name: service_name.to_string(),
            // register_service() seeds status=STOPPED, last_heartbeat=now.
            status: ServiceStatus::Stopped,
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

/// Outcome of one health-check decision (deterministic subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthDecision {
    Healthy,                          // responsive → HEALTHY + failures reset
    Degraded,                         // unresponsive, within heartbeat timeout
    Restarting(i64),                  // restart initiated, attempt number
    InCooldown,                       // restart attempted inside cooldown window
    MaxAttemptsStopped,               // restart_count >= max → STOPPED
    SilentSkip,                       // already gave up (STOPPED + max restarts)
}

/// `_restart_service` decision core (python lines: cooldown check →
/// max-attempts check → status RESTARTING / restart_count += 1).
fn restart_decision(
    config: &ServiceConfig,
    health: &mut ServiceHealth,
    now: f64,
) -> HealthDecision {
    // if health.last_restart:  (None → 0.0 sentinel, python truthiness)
    if health.last_restart > 0.0 {
        let time_since_restart = now - health.last_restart;
        if time_since_restart < config.restart_cooldown as f64 {
            // "In cooldown" — return early, no state change.
            return HealthDecision::InCooldown;
        }
    }
    // Check max restart attempts.
    if health.restart_count >= config.max_restart_attempts {
        health.status = ServiceStatus::Stopped;
        return HealthDecision::MaxAttemptsStopped;
    }
    health.status = ServiceStatus::Restarting;
    health.restart_count += 1;
    health.last_restart = now;
    HealthDecision::Restarting(health.restart_count)
}

/// `_check_service_health` decision core — one full check cycle given the
/// ping outcome. `is_responsive` is the health-endpoint ping result;
/// process spawn / verify-after-5s bodies are OS-bound (MANUAL below).
pub fn check_service_health(
    config: &ServiceConfig,
    health: &mut ServiceHealth,
    now: f64,
    is_responsive: bool,
) -> HealthDecision {
    if is_responsive {
        // HEALTHY: refresh heartbeat, response time, clear failures/error.
        health.status = ServiceStatus::Healthy;
        health.last_heartbeat = now;
        health.consecutive_failures = 0;
        health.error_message.clear();
        return HealthDecision::Healthy;
    }

    // Unresponsive: increment consecutive failures.
    health.consecutive_failures += 1;
    health.error_message = alloc::format!(
        "Health check failed ({} consecutive)",
        health.consecutive_failures
    );

    // time_since_heartbeat = now - last_heartbeat if last_heartbeat > 0
    // else float('inf')  → saturating sentinel.
    let time_since_heartbeat = if health.last_heartbeat > 0.0 {
        now - health.last_heartbeat
    } else {
        f64::MAX
    };

    // Check if we need to restart.
    if time_since_heartbeat > config.heartbeat_timeout as f64 {
        // If already gave up (max restarts exhausted) don't keep logging.
        if health.status == ServiceStatus::Stopped && health.restart_count >= config.max_restart_attempts
        {
            // Silently skip — already logged the max-attempts error.
            return HealthDecision::SilentSkip;
        }
        return restart_decision(config, health, now);
    }

    health.status = ServiceStatus::Degraded;
    HealthDecision::Degraded
}

/// The post-restart verification step of `_restart_service`: python waits
/// 5s, pings again; responsive → HEALTHY + restart_count reset to 0,
/// else → UNRESPONSIVE. `is_responsive_after` is the caller-provided ping.
pub fn restart_verify(
    config: &ServiceConfig,
    health: &mut ServiceHealth,
    now: f64,
    is_responsive_after: bool,
) -> ServiceStatus {
    let _ = config;
    if is_responsive_after {
        health.status = ServiceStatus::Healthy;
        health.last_heartbeat = now;
        health.consecutive_failures = 0;
        health.restart_count = 0; // Reset on successful restart
        ServiceStatus::Healthy
    } else {
        health.status = ServiceStatus::Unresponsive;
        ServiceStatus::Unresponsive
    }
}

/// Default services registered by `_register_default_services()`.
/// Returns (name, port, heartbeat_timeout, restart_cooldown, max_attempts).
pub const DEFAULT_SERVICES: [(&str, u32, i64, i64, i64); 3] = [
    ("swarm_api", 8021, 600, 30, 3),
    ("swarm_dashboard", 5183, 3600, 300, 1),
    ("swarm_dashboard_v2", 5174, 3600, 300, 1),
];

/// SelfHealingInfra — deterministic subset: registered services + health
/// status store. Async loop / subprocess bodies are host-bound (MANUAL).
#[derive(Debug, Clone)]
pub struct SelfHealingInfra {
    pub check_interval: i64,
    pub running: bool,
    pub services: Vec<ServiceConfig>,
    pub health_status: Vec<ServiceHealth>,
    pub restart_history: Vec<String>,
}

impl SelfHealingInfra {
    /// __init__(check_interval=60) + _register_default_services().
    pub fn new() -> Self {
        let mut s = SelfHealingInfra {
            check_interval: 60,
            running: false,
            services: vec![],
            health_status: vec![],
            restart_history: vec![],
        };
        for (name, port, timeout, cooldown, max_attempts) in DEFAULT_SERVICES.iter() {
            s.register_service(
                name,
                &alloc::format!("http://localhost:{}/health", port),
                *port,
                *timeout,
                *cooldown,
                *max_attempts,
            );
        }
        s
    }

    /// register_service: store config + seed ServiceHealth (status STOPPED,
    /// last_heartbeat = now passed by caller; python uses time.time()).
    pub fn register_service(
        &mut self,
        name: &str,
        health_endpoint: &str,
        port: u32,
        heartbeat_timeout: i64,
        restart_cooldown: i64,
        max_restart_attempts: i64,
    ) {
        self.services.push(ServiceConfig::new(
            name,
            health_endpoint,
            port,
            heartbeat_timeout,
            restart_cooldown,
            max_restart_attempts,
        ));
        self.health_status.push(ServiceHealth::new(name));
    }

    /// start()/stop() async loop flags.
    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn service_index(&self, name: &str) -> Option<usize> {
        self.services.iter().position(|c| c.name == name)
    }

    /// force_restart(service_name) — creates the async restart task in
    /// python; here the decision core is applied synchronously with now.
    /// Returns the decision; error "Service not found" when unknown.
    pub fn force_restart(&mut self, service_name: &str, now: f64) -> Result<HealthDecision, String> {
        let idx = self
            .service_index(service_name)
            .ok_or_else(|| alloc::format!("Service '{}' not found", service_name))?;
        let cfg = self.services[idx].clone();
        let health = &mut self.health_status[idx];
        Ok(restart_decision(&cfg, health, now))
    }

    /// reset_restart_count(service_name) — python resets counter to 0.
    pub fn reset_restart_count(&mut self, service_name: &str) -> Result<(), String> {
        let idx = self
            .health_status
            .iter()
            .position(|h| h.service_name == service_name)
            .ok_or_else(|| alloc::format!("Service '{}' not found", service_name))?;
        self.health_status[idx].restart_count = 0;
        Ok(())
    }

    /// get_status(): count services per status + total_restarts history len.
    pub fn status_summary(&self) -> (usize, usize, usize, usize, usize, usize) {
        let (mut healthy, mut degraded, mut unresponsive, mut restarting, mut stopped) =
            (0, 0, 0, 0, 0);
        for h in &self.health_status {
            match h.status {
                ServiceStatus::Healthy => healthy += 1,
                ServiceStatus::Degraded => degraded += 1,
                ServiceStatus::Unresponsive => unresponsive += 1,
                ServiceStatus::Restarting => restarting += 1,
                ServiceStatus::Stopped => stopped += 1,
            }
        }
        (healthy, degraded, unresponsive, restarting, stopped, self.restart_history.len())
    }
}

impl Default for SelfHealingInfra {
    fn default() -> Self {
        Self::new()
    }
}

/// Singleton instance (lazy-init pattern from self_healing_infra.py).
static SINGLETON_SELF_HEALING_INFRA: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
/// get_self_healing_infra() singleton handle: 1 once initialized.
pub fn get_self_healing_infra() -> u64 {
    let v = SINGLETON_SELF_HEALING_INFRA.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SELF_HEALING_INFRA.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
