//! TRM Swarm OS — telemetry_fabric (no_std port).
//! High-Bandwidth Telemetry Fabric (HBTF), deterministic core from
//! swarm_v2/core/telemetry_fabric.py:
//!   - subscribe(channel): register a subscriber slot per channel;
//!   - publish(frame): total_frames_processed += 1; bounded frame buffer
//!     (max_buffer_size=1000, pop(0) when full then append); fan-out to the
//!     channel's subscribers (callback invocation counted — python callbacks
//!     are host callables, so the deterministic core counts delivery);
//!   - get_fabric_metrics: total processed, buffered frames, active channel
//!     count.
//! Subscriber callbacks (Callable[[TelemetryFrame], None]) and the push to
//! the Semantic Knowledge Graph are external bindings → honest `// MANUAL`
//! no-ops below, never invented.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Raw telemetry frame emitted by node sensors (TelemetryFrame).
#[derive(Debug, Clone)]
pub struct TelemetryFrame {
    pub frame_id: String,
    pub source_node: String,
    pub channel: String,
    /// payload Dict[str, Any] as numeric (key, value) pairs.
    pub payload: Vec<(String, f64)>,
    /// caller-supplied timestamp (python time.time() wall clock).
    pub timestamp: f64,
}

impl TelemetryFrame {
    pub fn new(
        frame_id: &str,
        source_node: &str,
        channel: &str,
        payload: Vec<(String, f64)>,
        timestamp: f64,
    ) -> Self {
        TelemetryFrame {
            frame_id: frame_id.to_string(),
            source_node: source_node.to_string(),
            channel: channel.to_string(),
            payload,
            timestamp,
        }
    }

    /// payload.get(key, default) — numeric dict subset.
    pub fn payload_get(&self, key: &str) -> Option<f64> {
        self.payload
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| *v)
    }
}

/// One subscription slot: channel → subscriber count.
#[derive(Debug, Clone)]
struct Subscription {
    channel: String,
    count: u64,
}

/// TelemetryFabric — bounded pub-sub fabric (python class).
#[derive(Debug, Clone)]
pub struct TelemetryFabric {
    subscribers: Vec<Subscription>,
    pub frame_buffer: Vec<TelemetryFrame>,
    pub max_buffer_size: usize,
    pub total_frames_processed: u64,
}

impl TelemetryFabric {
    pub fn new() -> Self {
        TelemetryFabric {
            subscribers: vec![],
            frame_buffer: vec![],
            max_buffer_size: 1000,
            total_frames_processed: 0,
        }
    }

    /// Constructor variant with a custom buffer cap (python sets the plain
    /// attribute `max_buffer_size`; kept for deterministic cap testing).
    pub fn with_max_buffer(max_buffer_size: usize) -> Self {
        TelemetryFabric {
            max_buffer_size,
            ..TelemetryFabric::new()
        }
    }

    /// subscribe(channel): if channel missing, create subscriber list, then
    /// append one callback slot. Returns the channel subscriber count.
    pub fn subscribe(&mut self, channel: &str) -> u64 {
        if let Some(sub) = self.subscribers.iter_mut().find(|s| s.channel == channel) {
            sub.count += 1;
            sub.count
        } else {
            self.subscribers.push(Subscription {
                channel: channel.to_string(),
                count: 1,
            });
            1
        }
    }

    /// publish(frame): python 1:1 — increment total; evict oldest frame when
    /// the buffer is full; append; fan out to channel subscribers (counted).
    /// Returns the number of subscriber deliveries (callback bodies MANUAL).
    pub fn publish(&mut self, frame: TelemetryFrame) -> u64 {
        self.total_frames_processed += 1;
        let channel = frame.channel.clone();

        // Buffer management: if len >= max, pop(0) then append.
        if self.frame_buffer.len() >= self.max_buffer_size {
            self.frame_buffer.remove(0);
        }
        self.frame_buffer.push(frame);

        // Notify subscribers of this frame's channel (delivery count; each
        // python callback body is host-bound → MANUAL).
        let delivered = self
            .subscribers
            .iter()
            .find(|s| s.channel == channel)
            .map(|s| s.count)
            .unwrap_or(0);

        // Push to Semantic Knowledge Graph: MANUAL (cross-module binding).

        delivered
    }

    /// get_fabric_metrics() deterministic subset:
    /// (total_frames_processed, buffered_frames, active_channels).
    pub fn get_fabric_metrics(&self) -> (u64, usize, usize) {
        (
            self.total_frames_processed,
            self.frame_buffer.len(),
            self.subscribers.len(),
        )
    }

    /// Oldest buffered frame id, if any (python buffer pop(0) order).
    pub fn oldest_frame_id(&self) -> Option<String> {
        self.frame_buffer.first().map(|f| f.frame_id.clone())
    }
}

impl Default for TelemetryFabric {
    fn default() -> Self {
        Self::new()
    }
}

/// Singleton instance (lazy-init pattern from telemetry_fabric.py).
static SINGLETON_TELEMETRY_FABRIC: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
/// get_telemetry_fabric() singleton handle: 1 once initialized.
pub fn get_telemetry_fabric() -> u64 {
    let v = SINGLETON_TELEMETRY_FABRIC.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_TELEMETRY_FABRIC.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// python module constant.
pub const MAX_BUFFER_SIZE_DEFAULT: usize = 1000;
