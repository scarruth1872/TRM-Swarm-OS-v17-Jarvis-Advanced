//! TRM Swarm OS - swarm_performance_insights (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/swarm_performance_insights.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
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
