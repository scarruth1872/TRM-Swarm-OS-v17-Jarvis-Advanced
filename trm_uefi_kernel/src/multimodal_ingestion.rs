//! TRM Swarm OS - multimodal_ingestion (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/multimodal_ingestion.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  SensorTelemetryData 
#[derive(Debug, Clone)]
pub struct SensorTelemetryData {
    pub sensor_id: String,
    pub temperature_celsius: f64,
    pub vibration_hz: f64,
    pub power_draw_watts: f64,
    pub pressure_psi: f64,
    pub timestamp: f64,
}

impl SensorTelemetryData {
    pub fn new() -> Self {
        SensorTelemetryData {
            sensor_id: String::new(),
            temperature_celsius: 0.0,
            vibration_hz: 0.0,
            power_draw_watts: 0.0,
            pressure_psi: 0.0,
            timestamp: 0.0,
        }
    }
}

//  VisualFrameData 
#[derive(Debug, Clone)]
pub struct VisualFrameData {
    pub frame_id: String,
    pub camera_id: String,
    pub thermal_hotspot_intensity: f64,
    pub optical_flow_energy: f64,
    pub smoke_particle_density: f64,
    pub bounding_box_count: i64,
    pub timestamp: f64,
}

impl VisualFrameData {
    pub fn new() -> Self {
        VisualFrameData {
            frame_id: String::new(),
            camera_id: String::new(),
            thermal_hotspot_intensity: 0.0,
            optical_flow_energy: 0.0,
            smoke_particle_density: 0.0,
            bounding_box_count: 0,
            timestamp: 0.0,
        }
    }
}

//  FeatureVector 
#[derive(Debug, Clone)]
pub struct FeatureVector {
    pub modality: String,
    pub source_id: String,
    pub features: Vec<(String, f64)>,
    pub normalized_score: f64,
    pub timestamp: f64,
}

impl FeatureVector {
    pub fn new() -> Self {
        FeatureVector {
            modality: String::new(),
            source_id: String::new(),
            features: vec![],
            normalized_score: 0.0,
            timestamp: 0.0,
        }
    }
}

//  get_multimodal_ingestion() 
static SINGLETON_MULTIMODAL_INGESTION: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_multimodal_ingestion() -> u64 {
    // Singleton handle (lazy-init pattern from multimodal_ingestion.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_MULTIMODAL_INGESTION.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_MULTIMODAL_INGESTION.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
