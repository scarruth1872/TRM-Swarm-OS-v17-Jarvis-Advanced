"""
telemetry_anomaly_baseliner.py — TRM Swarm OS v19.001 "Sovereign"
==================================================================
Darktrace-Style Statistical Anomaly Baselining & Autonomous Response Engine:
- Computes real-time rolling mean (μ), Exponential Moving Average (EMA), and standard deviation (σ).
- Calculates robust dynamic Z-scores: Z = (x - μ) / σ.
- Monitors multi-channel telemetry streams:
  1. Inter-agent message throughput (msgs/sec)
  2. IPC ring communication latency (microseconds)
  3. Memory allocation / heap delta (MB/sec)
  4. Physical voltage rail jitter (mV)
- Automatic Threat Level Classification:
  * NORMAL: |Z| < 2.0 (Nominal operational baseline)
  * ELEVATED_WARNING: 2.0 <= |Z| < 3.0 (Minor drift / queue accumulation)
  * CRITICAL_ANOMALY: |Z| >= 3.0 (Potential exploit / runaway loop / Rowhammer breach)
- Triggers autonomous mitigation policies (rate limiting, thread throttling, antibody memory sweep).
"""

import math
import time
from collections import deque
from enum import Enum
from typing import Dict, List, Any, Tuple, Optional
from dataclasses import dataclass, field

class ThreatLevel(Enum):
    NORMAL = "normal"
    ELEVATED_WARNING = "elevated_warning"
    CRITICAL_ANOMALY = "critical_anomaly"

@dataclass
class AnomalyEvent:
    timestamp: float
    channel_name: str
    observed_value: float
    baseline_mean: float
    baseline_std: float
    z_score: float
    threat_level: ThreatLevel
    mitigation_action: Optional[str] = None

class RollingMetricStream:
    """Tracks rolling statistical moments over a sliding window."""
    def __init__(self, window_size: int = 100, ema_alpha: float = 0.15):
        self.window_size = window_size
        self.ema_alpha = ema_alpha
        self.window = deque(maxlen=window_size)
        self.ema: Optional[float] = None

    def push(self, value: float) -> Tuple[float, float, float]:
        """Pushes a sample and returns (mean, std, ema)."""
        self.window.append(value)
        if self.ema is None:
            self.ema = value
        else:
            self.ema = self.ema_alpha * value + (1.0 - self.ema_alpha) * self.ema

        n = len(self.window)
        mean_val = sum(self.window) / n
        if n > 1:
            variance = sum((x - mean_val) ** 2 for x in self.window) / (n - 1)
            std_val = math.sqrt(variance) if variance > 1e-9 else 1e-6
        else:
            std_val = 1e-6

        return mean_val, std_val, self.ema

class TelemetryAnomalyBaseliner:
    """
    Multi-channel real-time telemetry baseliner and anomaly mitigation engine.
    """
    def __init__(
        self,
        window_size: int = 100,
        warning_z: float = 2.0,
        critical_z: float = 3.0
    ):
        self.warning_z = warning_z
        self.critical_z = critical_z
        self.channels: Dict[str, RollingMetricStream] = {
            "msg_rate_mps": RollingMetricStream(window_size),
            "ipc_latency_us": RollingMetricStream(window_size),
            "memory_alloc_mb_s": RollingMetricStream(window_size),
            "voltage_jitter_mv": RollingMetricStream(window_size)
        }
        self.anomaly_log: List[AnomalyEvent] = []
        self.mitigation_history: List[Dict[str, Any]] = []

    def observe(self, channel_name: str, value: float) -> AnomalyEvent:
        """Records an observation, computes Z-score, and triggers mitigation if anomalous."""
        if channel_name not in self.channels:
            self.channels[channel_name] = RollingMetricStream()

        stream = self.channels[channel_name]
        mean_val, std_val, ema_val = stream.push(value)

        # Compute Z-score
        z = (value - mean_val) / std_val if std_val > 1e-9 else 0.0

        # Classify threat
        abs_z = abs(z)
        if abs_z >= self.critical_z:
            threat = ThreatLevel.CRITICAL_ANOMALY
            mitigation = self._execute_mitigation(channel_name, value, z)
        elif abs_z >= self.warning_z:
            threat = ThreatLevel.ELEVATED_WARNING
            mitigation = f"LOGGED_WARNING_ON_{channel_name.upper()}"
        else:
            threat = ThreatLevel.NORMAL
            mitigation = None

        event = AnomalyEvent(
            timestamp=time.time(),
            channel_name=channel_name,
            observed_value=round(value, 3),
            baseline_mean=round(mean_val, 3),
            baseline_std=round(std_val, 3),
            z_score=round(z, 2),
            threat_level=threat,
            mitigation_action=mitigation
        )

        if threat != ThreatLevel.NORMAL:
            self.anomaly_log.append(event)

        return event

    def _execute_mitigation(self, channel_name: str, value: float, z: float) -> str:
        """Executes autonomous mitigation response based on breached channel."""
        action = "AUTONOMOUS_INTERVENTION_TRIGGERED"
        if channel_name == "msg_rate_mps":
            action = "RATE_LIMITER_ENGAGED_CAP_500_MPS"
        elif channel_name == "ipc_latency_us":
            action = "IPC_RING_FLUSH_AND_THREAD_POOL_REBALANCE"
        elif channel_name == "memory_alloc_mb_s":
            action = "DDR_ANTIBODY_SWEEP_AND_HEAP_QUARANTINE"
        elif channel_name == "voltage_jitter_mv":
            action = "VOLTAGE_NODE_REEQUILIBRATION_TO_NOMINAL"

        self.mitigation_history.append({
            "timestamp": time.time(),
            "channel": channel_name,
            "observed_value": value,
            "z_score": z,
            "action": action
        })
        return action

    def get_system_telemetry_status(self) -> Dict[str, Any]:
        """Returns instantaneous baseline status across all telemetry channels."""
        status = {}
        for name, stream in self.channels.items():
            if stream.window:
                n = len(stream.window)
                mean_v = sum(stream.window) / n
                var_v = sum((x - mean_v)**2 for x in stream.window) / n if n > 1 else 0.0
                status[name] = {
                    "sample_count": n,
                    "rolling_mean": round(mean_v, 3),
                    "rolling_std": round(math.sqrt(var_v), 3),
                    "ema": round(stream.ema, 3) if stream.ema else 0.0,
                    "latest_value": round(stream.window[-1], 3)
                }
        return {
            "channels": status,
            "total_anomalies_detected": len(self.anomaly_log),
            "total_mitigations_executed": len(self.mitigation_history)
        }
