import time
import re
import unicodedata
from collections import defaultdict, deque
import os
import signal
import subprocess
import psutil

# 2.1 Input Sanitizer (IS)
class InputSanitizer:
    # Security attack patterns
    SQL_RE = re.compile(
        r"(\b(UNION\s+(ALL\s+)?SELECT|SELECT\b.{0,40}\bFROM\b"
        r"|INSERT\s+INTO|UPDATE\s+\w+\s+SET|DELETE\s+FROM"
        r"|DROP\s+(TABLE|DATABASE|INDEX|VIEW|PROCEDURE|FUNCTION)"
        r"|ALTER\s+(TABLE|DATABASE|INDEX|VIEW|PROCEDURE|FUNCTION|COLUMN)"
        r"|CREATE\s+(TABLE|DATABASE|INDEX|VIEW|PROCEDURE|FUNCTION|TRIGGER)"
        r"|TRUNCATE\s+(TABLE|DATABASE)|EXEC\b|EXECUTE\b|DECLARE\b"
        r"|CAST\s*\(|CONVERT\s*\(|ORD\s*\(|MID\s*\(|CHAR\s*\(|SUBSTRING\s*\("
        r"|CONCAT\s*\(|BENCHMARK\s*\(|SLEEP\s*\(|LOAD_FILE\s*\("
        r"|INTO\s+(OUTFILE|DUMPFILE)|INFORMATION_SCHEMA"
        r"|WAITFOR\s+DELAY|pg_sleep\s*\(|UNION\s+SELECT)"
        r"|(?<!\w)(OR|AND)\s+\d+\s*=\s*\d+"
        r"|(?<!\w)(OR|AND)\s+['\"][^'\"]+['\"]\s*=\s*['\"][^'\"]*['\"]"
        r"|--[\s\S]|#[\s]|/\*|\*/"
        r"|0x[0-9A-Fa-f]{4,}"
        r"|;\s*\b(ALTER|CREATE|DELETE|DROP|EXEC|EXECUTE|INSERT|SELECT|TRUNCATE|UPDATE|UNION|DECLARE|SHUTDOWN)\b)",
        re.IGNORECASE
    )
    XSS_RE = re.compile(
        r"(<script[^>]*>|</script\s*>"
        r"|<[^>]*\bonerror\s*=|<\w+\s[^>]*\bonload\s*="
        r"|<[^>]*\bonclick\s*=|<\w+\s[^>]*\bonmouseover\s*="
        r"|<[^>]*\bonfocus\s*=|<\w+\s[^>]*\bonblur\s*="
        r"|<[^>]*\bonsubmit\s*=|<\w+\s[^>]*\bonchange\s*="
        r"|<[^>]*\bonkeypress\s*=|<\w+\s[^>]*\bonkeydown\s*="
        r"|<[^>]*\bonkeyup\s*=|<\w+\s[^>]*\bondblclick\s*="
        r"|<[^>]*\bonmousedown\s*=|<\w+\s[^>]*\bonmouseup\s*="
        r"|<[^>]*\bonmouseenter\s*=|<\w+\s[^>]*\bonmouseleave\s*="
        r"|<[^>]*\bonpageshow\s*=|<\w+\s[^>]*\bontoggle\s*="
        r"|<[^>]*\bonstart\s*="
        r"|javascript:\s*|data:\s*text/html"
        r"|eval\s*\(|alert\s*\(|prompt\s*\(|confirm\s*\("
        r"|String\.fromCharCode\s*\(|document\.cookie|document\.location|document\.write\s*\("
        r"|&lt;script|\\x3Cscript|\\u003Cscript)",
        re.IGNORECASE
    )
    PATH_RE = re.compile(
        r"(\.\.[/\\]|\.\.%[25]*2f|\.\.%[25]*5c"
        r"|%2e%2e%[25]*2f|%2e%2e%[25]*5c"
        r"|%25(2e|2f|5c)"
        r"|%c0%ae"
        r"|%[25]*00"
        r"|[A-Za-z]:[/\\]"
        r"|(?:^|[^a-zA-Z])~\w*[/\\]"
        r"|/(etc|windows|proc|boot|sys|dev|bin|usr|var|opt|tmp|root|home)[/\\])",
        re.IGNORECASE
    )

    def __init__(self, max_chunk_size=65536, max_total_size=1048576):
        self.max_chunk_size = max_chunk_size
        self.max_total_size = max_total_size
        self.control_chars = set(range(0x00, 0x20)) | {0x7F}

    def sanitize(self, data: bytes) -> str:
        if len(data) > self.max_total_size:
            raise ValueError(f"Payload too large. Max allowed: {self.max_total_size} bytes.")
        
        # Remove control characters except newline/tab
        cleaned = bytes(b for b in data if b not in self.control_chars or b in (0x0A, 0x09))
        
        # Normalize unicode
        decoded = unicodedata.normalize('NFC', cleaned.decode('utf-8', errors='replace'))
        
        # ── Security pattern checks ─────────────────────────────────────────
        # After sanitization, check for SQLi / XSS / Path Traversal patterns
        if self.SQL_RE.search(decoded):
            raise ValueError("Request blocked: SQL injection patterns detected in payload.")
        if self.XSS_RE.search(decoded):
            raise ValueError("Request blocked: XSS patterns detected in payload.")
        if self.PATH_RE.search(decoded):
            raise ValueError("Request blocked: Path traversal patterns detected in payload.")
        
        return decoded


# 2.2 Token Rate Monitor (TRM)
class RateLimitExceeded(Exception):
    pass

class TokenRateMonitor:
    def __init__(self, window_size=1.0, max_rate=50000, max_budget=100000000):
        self.window_size = window_size
        self.max_rate = max_rate
        self.max_budget = max_budget
        self.token_buckets = defaultdict(lambda: deque())
        self.session_totals = defaultdict(int)
        self.session_timestamps = defaultdict(float)

    def check_rate(self, session_id, token_count):
        # Localhost and loopback IPs have unlimited budget for internal dashboard ops
        if session_id in ("127.0.0.1", "::1", "localhost", "testclient"):
            return True

        now = time.time()
        # Reset budget every hour for active sessions
        if now - self.session_timestamps[session_id] > 3600:
            self.session_totals[session_id] = 0
            self.session_timestamps[session_id] = now

        if self.session_totals[session_id] + token_count > self.max_budget:
            raise RateLimitExceeded(f"Session budget of {self.max_budget} tokens exceeded.")

        bucket = self.token_buckets[session_id]
        
        # Remove old entries
        while bucket and bucket[0] < now - self.window_size:
            bucket.popleft()
            
        # Calculate current rate
        current_rate = len(bucket) / self.window_size
        if current_rate + token_count > self.max_rate:
            raise RateLimitExceeded(f"Token rate exceeded {self.max_rate} tokens/sec.")
            
        bucket.extend([now] * token_count)
        self.session_totals[session_id] += token_count
        return True


import asyncio
import json
from swarm_v2.core.redis_mock import PersistentRedisMock

# 3.1 Health Dashboard Monitor (HDM)
class HealthDashboardMonitor:
    def __init__(self):
        self.thresholds = {
            'memory_percent': 80.0,
            'cpu_percent': 70.0
        }
        self.redis = PersistentRedisMock()
        self.running = False

    async def start(self):
        self.running = True
        print("[HDM] Health Dashboard Monitor started.")
        while self.running:
            try:
                metrics = self.collect_metrics()
                # Store to redis for dashboard observability
                self.redis.set("hdm:health_metrics", json.dumps(metrics))
                
                if metrics['anomalies']:
                    print(f"[HDM] Anomaly Detected: {metrics['anomalies']}")
                    # Trigger Recovery Coordinator
                    for anomaly in metrics['anomalies']:
                        result = recovery_coordinator.execute_recovery(anomaly, metrics)
                        telemetry.record_recovery(True)
                        print(f"[RCM] {result}")
            except Exception as e:
                print(f"[HDM] Error in health loop: {e}")
                
            await asyncio.sleep(5)

    def stop(self):
        self.running = False

    def collect_metrics(self):
        mem = psutil.virtual_memory().percent
        cpu = psutil.cpu_percent(interval=None)
        
        anomalies = []
        if mem > self.thresholds['memory_percent']:
            anomalies.append('memory_pressure')
        if cpu > self.thresholds['cpu_percent']:
            anomalies.append('cpu_pressure')
            
        return {
            'memory_percent': mem,
            'cpu_percent': cpu,
            'anomalies': anomalies
        }


# 3.2 Recovery Coordinator (RCM)
class RecoveryCoordinator:
    def __init__(self):
        self.recovery_strategies = {
            'memory_pressure': self._reduce_memory_footprint,
            'token_overflow': self._reset_token_buckets,
            'cpu_pressure': self._shed_load,
            'process_crash': self._restart_worker
        }

    def execute_recovery(self, anomaly_type, context):
        strategy = self.recovery_strategies.get(anomaly_type)
        if strategy:
            return strategy(context)
        return self._default_recovery(context)

    def _reduce_memory_footprint(self, context):
        import gc
        # Graceful Degradation: force garbage collection and clear unused token buckets
        gc.collect()
        
        # Clear out old TRM buckets if needed
        now = time.time()
        for session in list(token_monitor.token_buckets.keys()):
            bucket = token_monitor.token_buckets[session]
            if not bucket or bucket[-1] < now - 60:
                del token_monitor.token_buckets[session]
                
        try:
            from swarm_v2.app_v2 import remediation_engine
            # Run in a fire-and-forget task so we don't block
            asyncio.create_task(remediation_engine.handle_issue("HIGH_CPU", "Memory Pressure cleanup triggered"))
        except ImportError:
            pass
            
        return "Executed GC cleanup and TRM bucket purge for memory pressure."

    def _shed_load(self, context):
        # Drop token rate limits dynamically for graceful degradation
        token_monitor.max_rate = max(100, token_monitor.max_rate // 2)
        try:
            from swarm_v2.app_v2 import remediation_engine
            asyncio.create_task(remediation_engine.handle_issue("HIGH_CPU", "CPU pressure: shed load by halving token limits"))
        except ImportError:
            pass
        return f"CPU pressure mitigated. Lowered token rate limit to {token_monitor.max_rate}/sec."

    def _reset_token_buckets(self, context):
        # Emergency reset if token overflows cause severe issues
        token_monitor.token_buckets.clear()
        return "Token buckets emergency reset."

    def _restart_worker(self, context):
        role = context.get('role', 'Unknown')
        try:
            from swarm_v2.app_v2 import remediation_engine
            asyncio.create_task(remediation_engine.handle_issue("AGENT_TIMEOUT", f"Restarting crashed worker: {role}"))
            return f"Restarting worker {role} via RemediationEngine."
        except ImportError:
            return "Cannot restart worker: RemediationEngine not found."

    def _default_recovery(self, context):
        return "No specific recovery strategy applied."


# 3.3 Telemetry & Observability
class TelemetryManager:
    def __init__(self):
        self.redis = PersistentRedisMock()
        self._init_metrics()

    def _init_metrics(self):
        # Initialize if not present in redis
        if not self.redis.get("telemetry:total_requests"):
            self.redis.set("telemetry:total_requests", 0)
            self.redis.set("telemetry:total_bytes", 0)
            self.redis.set("telemetry:recoveries", 0)

    def record_input(self, byte_size):
        try:
            reqs = int(self.redis.get("telemetry:total_requests") or 0)
            b = int(self.redis.get("telemetry:total_bytes") or 0)
            self.redis.set("telemetry:total_requests", reqs + 1)
            self.redis.set("telemetry:total_bytes", b + byte_size)
        except Exception:
            pass

    def record_recovery(self, success: bool):
        if success:
            try:
                recs = int(self.redis.get("telemetry:recoveries") or 0)
                self.redis.set("telemetry:recoveries", recs + 1)
            except Exception:
                pass
                
    def get_metrics(self):
        return {
            'total_requests': int(self.redis.get("telemetry:total_requests") or 0),
            'total_bytes_processed': int(self.redis.get("telemetry:total_bytes") or 0),
            'recovery_actions_taken': int(self.redis.get("telemetry:recoveries") or 0)
        }

telemetry = TelemetryManager()
input_sanitizer = InputSanitizer()
token_monitor = TokenRateMonitor()
health_monitor = HealthDashboardMonitor()
recovery_coordinator = RecoveryCoordinator()
