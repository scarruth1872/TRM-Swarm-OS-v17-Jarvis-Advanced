//! TRM Swarm OS - swarm_performance_insights (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/swarm_performance_insights.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  MetricPoint 
#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f64,
    pub metadata: u64,
}

impl MetricPoint {
    pub fn new() -> Self {
        MetricPoint {
            timestamp: 0,
            value: 0.0,
            metadata: 0,
        }
    }
}

//  PerformanceMetric 
#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    pub metric_type: u64,
    pub current_value: f64,
    pub previous_value: f64,
    pub change_percent: f64,
    pub trend: String,
    pub data_points: Vec<u64>,
}

impl PerformanceMetric {
    pub fn new() -> Self {
        PerformanceMetric {
            metric_type: 0,
            current_value: 0.0,
            previous_value: 0.0,
            change_percent: 0.0,
            trend: String::new(),
            data_points: vec![],
        }
    }
}

//  SwarmIntelligenceReport 
#[derive(Debug, Clone)]
pub struct SwarmIntelligenceReport {
    pub report_id: String,
    pub period: u64,
    pub generated_at: u64,
    pub start_date: u64,
    pub end_date: u64,
    pub metrics: Vec<(String, f64)>,
    pub overall_health_score: f64,
    pub efficiency_score: f64,
    pub growth_score: f64,
    pub top_performers: Vec<u64>,
    pub improvement_areas: Vec<String>,
    pub recommendations: Vec<String>,
    pub total_tasks_processed: i64,
    pub total_errors: i64,
    pub uptime_hours: f64,
}

impl SwarmIntelligenceReport {
    pub fn new() -> Self {
        SwarmIntelligenceReport {
            report_id: String::new(),
            period: 0,
            generated_at: 0,
            start_date: 0,
            end_date: 0,
            metrics: vec![],
            overall_health_score: 0.0,
            efficiency_score: 0.0,
            growth_score: 0.0,
            top_performers: vec![],
            improvement_areas: vec![],
            recommendations: vec![],
            total_tasks_processed: 0,
            total_errors: 0,
            uptime_hours: 0.0,
        }
    }
}

//  get_performance_insights() 
static SINGLETON_PERFORMANCE_INSIGHTS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_performance_insights() -> u64 {
    // Singleton handle (lazy-init pattern from swarm_performance_insights.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_PERFORMANCE_INSIGHTS.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_PERFORMANCE_INSIGHTS.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PORTABLE RULE PORT — the decision logic of swarm_performance_insights.py.
//
// Ported 1:1 (names kept identical to the Python where the Python has a name):
//   _calculate_trend            (py 481-490)  trend classification, strict +-5%
//   _get_previous_metric_value  (py 492-499)  history[-2] / history[0] / 0.0
//   record_task                 (py 348-352)  session task/error counters
//   calculate_change_percent    (py 539)      ((cur - prev) / max(1, prev)) * 100
//   calculate_success_rate      (py 438/181)  (tasks - errors) / max(1, tasks) * 100
//   calculate_overall_scores    (py 553-559)  health/efficiency/growth tri-score
//   generate_improvement_areas  (py 576-581)  three thresholds, strict <
//   generate_recommendations    (py 584-593)  health>80 / skills<5 / threats>0
//   push_metric_history         (py 287/346)  deque(maxlen=N) semantics
//   tail_slice                  (py 131/644/332) python items[-limit:]
//   REPORT_METRIC_DEFAULTS      (py 529-535)  metric_mappings + .get() defaults
//
// NOT ported (host-side, all in the Python but not portable to a UEFI kernel):
//   file I/O — os.makedirs, _load_reports/_save_report, json.dump/load,
//   report_index.json, markdown export, reports_dir;
//   logging.basicConfig/logger; asyncio.monitor_loop/await asyncio.sleep(168h);
//   datetime.now/timedelta/isoformat/strftime and report_id timestamps;
//   the collect_metrics() cross-component imports (global_memory, task_arbiter,
//   agent_mesh, learning_engine, synthesizer, self_healing_infra, neural_wall,
//   resonance_engine, collaborative_reasoning) and psutil-style usage figures;
//   to_dict()/to_markdown() presentation including python round(x, 2) — python's
//   banker's rounding is a formatting rule, not a decision rule, and is not
//   reproduced here.
// The Phase-4 "native voltage strip" block (py 24-65) is injected telemetry code,
// never called by the class; its rule already exists in-tree as
// swarm_v2_artifacts/native_voltage_strips/strip_telemetry_trend.rs.
//
// no_std: libm is absent from this crate, so NO f64 floor/round/sqrt/exp/ln/powf
// is used or needed below — every rule is plain f64 arithmetic written in python's
// exact operand order (division before the *100.0 scaling), because float64 is not
// associative and reordering changes the last bits. Comparisons are python's
// (`min(a, b)` is ported as `if b < a { b } else { a }`, which is what python's
// min() does, including its false-when-NaN behavior).
// ═══════════════════════════════════════════════════════════════════════════════

/// `metrics_history_size=1000` ctor default (py 263).
pub const METRICS_HISTORY_SIZE: usize = 1000;
/// `MetricPoint`/`PerformanceMetric.to_dict()` keep only the last 10 points (py 131).
pub const DATA_POINTS_TAIL: usize = 10;
/// `_save_report()` keeps only the last 52 report-index entries (py 332).
pub const MAX_REPORT_INDEX_ENTRIES: usize = 52;
/// `monitor_loop(interval_hours=168)` default, i.e. weekly (py 601).
pub const MONITOR_INTERVAL_HOURS: u64 = 168;

/// metric_mappings defaults from generate_report() (py 529-535/537):
/// (report metric name, value used by `current_metrics.get(key, default)`).
/// `task_success_rate` defaults to 100 (an unmeasured swarm is assumed healthy);
/// every other metric defaults to 0.
pub const REPORT_METRIC_DEFAULTS: [(&str, f64); 5] = [
    ("memory_growth", 0.0),
    ("task_success_rate", 100.0),
    ("resource_utilization", 0.0),
    ("skill_acquisition", 0.0),
    ("mesh_health", 0.0),
];

/// python `min(a, b)` — `b` only when strictly smaller, so a NaN operand keeps `a`.
fn py_min(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}

/// `_native_telemetry_trend_analyzer` (py 42-64): per-series linear-regression slope and variance.
/// x_mean = (n-1)/2, x_var = Σ(i-x_mean)², slope = cov(x,y)/x_var, var = Σ(y-y_mean)²/n, all f64
/// (the oracle computes in Python float before casting to float32). Matrix is row-major with
/// `n_points` columns; `n_series = matrix.len() / n_points`. `n_points == 1` divides by zero, as
/// the oracle does.
pub fn telemetry_trend_analyzer(matrix: &[f64], n_points: usize) -> (Vec<f64>, Vec<f64>) {
    if n_points == 0 {
        return (Vec::new(), Vec::new());
    }
    let n_series = matrix.len() / n_points;
    let x_mean = (n_points as f64 - 1.0) / 2.0;
    let mut x_var = 0.0f64;
    for i in 0..n_points {
        let d = i as f64 - x_mean;
        x_var += d * d;
    }
    let mut slopes: Vec<f64> = Vec::with_capacity(n_series);
    let mut vars: Vec<f64> = Vec::with_capacity(n_series);
    for s in 0..n_series {
        let row = &matrix[s * n_points..(s + 1) * n_points];
        let mut y_mean = 0.0f64;
        for v in row {
            y_mean += *v;
        }
        y_mean /= n_points as f64;
        let mut cov = 0.0f64;
        for i in 0..n_points {
            cov += (i as f64 - x_mean) * (row[i] - y_mean);
        }
        slopes.push(cov / x_var);
        let mut var = 0.0f64;
        for v in row {
            let d = *v - y_mean;
            var += d * d;
        }
        vars.push(var / n_points as f64);
    }
    (slopes, vars)
}

/// `SwarmPerformanceInsights._calculate_trend` (py 481-490).
/// previous == 0.0 -> "stable" (covers -0.0, and NaN never equals 0 so NaN falls
/// through); change = (current - previous) / previous * 100; strict `> 5` ->
/// "improving", strict `< -5` -> "declining", otherwise "stable" (a NaN change is
/// therefore "stable").
pub fn _calculate_trend(current: f64, previous: f64) -> String {
    if previous == 0.0 {
        return String::from("stable");
    }
    let change = (current - previous) / previous * 100.0;
    if change > 5.0 {
        String::from("improving")
    } else if change < -5.0 {
        String::from("declining")
    } else {
        String::from("stable")
    }
}

/// `SwarmPerformanceInsights._get_previous_metric_value` (py 492-499) over that
/// metric's history values: >= 2 entries -> history[len - 2] (the second-to-last,
/// i.e. the last *previous* value); exactly 1 entry -> history[0] (the value is its
/// own baseline); empty -> 0.0.
pub fn _get_previous_metric_value(history: &[f64]) -> f64 {
    if history.len() >= 2 {
        history[history.len() - 2]
    } else if history.len() == 1 {
        history[0]
    } else {
        0.0
    }
}

/// `record_task` (py 348-352): every call counts a task, a failure also counts an error.
pub fn record_task(session_tasks: &mut i64, session_errors: &mut i64, success: bool) {
    *session_tasks += 1;
    if !success {
        *session_errors += 1;
    }
}

/// task success rate (py 438 in collect_metrics, py 181 in to_dict):
/// `(tasks - errors) / max(1, tasks) * 100` — max(1, .) only guards the zero-task
/// division, it does not clamp a negative task count.
pub fn calculate_success_rate(total_tasks: i64, total_errors: i64) -> f64 {
    let denom: i64 = if total_tasks > 1 { total_tasks } else { 1 };
    (total_tasks - total_errors) as f64 / denom as f64 * 100.0
}

/// change_percent from the metric_mappings loop (py 539):
/// `((current - previous) / max(1, previous)) * 100 if previous > 0 else 0`.
/// A non-positive baseline yields exactly 0.0 (never a division result), while a
/// positive baseline below 1.0 is divided by 1 (the `max(1, previous)` guard).
pub fn calculate_change_percent(current: f64, previous: f64) -> f64 {
    if previous > 0.0 {
        let denom = if previous > 1.0 { previous } else { 1.0 };
        (current - previous) / denom * 100.0
    } else {
        0.0
    }
}

/// Overall scores (py 553-559), returns
/// `(overall_health_score, efficiency_score, growth_score)`:
/// `cpu_efficiency = 100 - min(100, cpu_usage)`,
/// `memory_growth  = min(100, memory_total_entries / 10)`,
/// `overall_health = (task_success + cpu_efficiency + memory_growth) / 3`.
/// Callers supply the `current_metrics.get(key, default)` value — see
/// `REPORT_METRIC_DEFAULTS` (task_success_rate defaults to 100.0, others 0.0).
pub fn calculate_overall_scores(
    task_success_rate: f64,
    cpu_usage: f64,
    memory_total_entries: f64,
) -> (f64, f64, f64) {
    let cpu_efficiency = 100.0 - py_min(100.0, cpu_usage);
    let memory_growth = py_min(100.0, memory_total_entries / 10.0);
    let overall_health = (task_success_rate + cpu_efficiency + memory_growth) / 3.0;
    (overall_health, cpu_efficiency, memory_growth)
}

/// Improvement areas (py 576-581), in this fixed order, strict `<` on all three:
/// task_success < 90, cpu_efficiency < 50, memory_growth < 10.
pub fn generate_improvement_areas(
    task_success: f64,
    cpu_efficiency: f64,
    memory_growth: f64,
) -> Vec<String> {
    let mut areas: Vec<String> = vec![];
    if task_success < 90.0 {
        areas.push(String::from(
            "Task success rate below 90% - investigate error patterns",
        ));
    }
    if cpu_efficiency < 50.0 {
        areas.push(String::from(
            "High CPU utilization - consider load balancing",
        ));
    }
    if memory_growth < 10.0 {
        areas.push(String::from(
            "Low memory growth - increase knowledge acquisition",
        ));
    }
    areas
}

/// Recommendations (py 584-593), in this fixed order: the health verdict is always
/// present (strict `> 80`, else the review message), then skills `< 5`, then the
/// blocked-threat count when strictly `> 0`.
pub fn generate_recommendations(
    overall_health_score: f64,
    learned_skills: f64,
    threats_blocked: i64,
) -> Vec<String> {
    let mut recs: Vec<String> = vec![];
    if overall_health_score > 80.0 {
        recs.push(String::from(
            "Swarm is performing well - maintain current configuration",
        ));
    } else {
        recs.push(String::from(
            "Review agent task distribution for optimization",
        ));
    }
    if learned_skills < 5.0 {
        recs.push(String::from(
            "Increase skill acquisition through documentation ingestion",
        ));
    }
    if threats_blocked > 0 {
        // py 593: f"Review {threats_blocked} blocked threats for patterns";
        // threats_blocked is neural_wall.get_stats()["total_detections"], an int count.
        recs.push(format!("Review {} blocked threats for patterns", threats_blocked));
    }
    recs
}

/// `deque(maxlen=metrics_history_size)` + `record_metric` semantics (py 287, 346):
/// append, then drop from the front while the buffer exceeds maxlen. A maxlen of 0
/// discards the value immediately (python `deque(maxlen=0)` keeps nothing).
pub fn push_metric_history(history: &mut Vec<f64>, value: f64, maxlen: usize) {
    history.push(value);
    while history.len() > maxlen {
        history.remove(0);
    }
}

/// python's `items[-limit:]` as used by `data_points[-10:]` (py 131),
/// `get_metric_history(limit)` (py 644) and the 52-entry report-index cap (py 332).
/// NOTE: python's `items[-0:]` is `items[0:]`, i.e. the WHOLE list, so limit == 0
/// returns everything — not an empty slice.
pub fn tail_slice<T>(items: &[T], limit: usize) -> &[T] {
    if limit == 0 || limit >= items.len() {
        items
    } else {
        &items[items.len() - limit..]
    }
}
