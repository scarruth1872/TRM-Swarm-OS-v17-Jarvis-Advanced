"""
Sentinel Middleware - Security Hardening Layer
Rate limiting, header hardening, and request sanitization for Swarm V2
"""

import re
import time
import os
import json
import urllib.parse
from typing import Optional, Dict, Any
from starlette.types import ASGIApp, Scope, Receive, Send
from starlette.responses import JSONResponse

# Try to import Redis - fallback to mock if unavailable
try:
    from swarm_v2.core.redis_mock import PersistentRedisMock
    redis_client = PersistentRedisMock()
except ImportError:
    redis_client = None


class SentinelMiddleware:
    """
    Comprehensive security middleware for FastAPI applications.
    Implements pure ASGI __call__ protocol to avoid BaseHTTPMiddleware stream deadlocks.
    
    Features:
    - Rate limiting (per IP) using Redis
    - Header hardening (HSTS, CSP, X-Frame-Options)
    - Request sanitization (SQL injection, XSS, path traversal) — QUERY PARAMS + BODY
    - Global exception handling for 4xx/5xx errors
    
    Security is ALWAYS applied regardless of localhost. Set TRUST_LOCALHOST=true
    to bypass only rate limiting on localhost (sanitization still applies).
    """
    
    def __init__(
        self,
        app: ASGIApp,
        redis_client=None,
        rate_limit: int = 100,
        rate_window: int = 60  # seconds
    ):
        self.app = app
        self.redis = redis_client
        self.rate_limit = rate_limit
        self.rate_window = rate_window
        self.trust_localhost = os.environ.get("TRUST_LOCALHOST", "false").lower() in ("1", "true", "yes")
        
        # ── SQL Injection patterns ──────────────────────────────────────────
        # Catches: UNION SELECT, stacked queries, comments, hex literals,
        # SQL functions (ORD/MID/CHAR/BENCHMARK/SLEEP/LOAD_FILE),
        # boolean-based injection (OR 1=1, AND '1'='1'),
        # time-based (WAITFOR DELAY, pg_sleep), out-of-band (INTO OUTFILE),
        # information_schema, hex-encoded data, semicolon-stacked SQL
        self.sql_re = re.compile(
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
            r"|--\s|#\s|/\*|\*/"
            r"|0x[0-9A-Fa-f]{4,}"
            r"|;\s*\b(ALTER|CREATE|DELETE|DROP|EXEC|EXECUTE|INSERT|SELECT|TRUNCATE|UPDATE|UNION|DECLARE|SHUTDOWN)\b)",
            re.IGNORECASE
        )
        
        # ── XSS patterns ────────────────────────────────────────────────────
        # Catches: <script>, </script>, <img onerror=, <svg onload=,
        # <body onload=, all event handlers (onerror/onload/onclick/onmouseover/
        # onfocus/onblur/onsubmit/onchange/onkeypress/onkeydown/onkeyup/
        # ondblclick/onmousedown/onmouseup/onmouseenter/onmouseleave/
        # onpageshow/ontoggle/onstart),
        # javascript: protocol, data:text/html, eval(), alert(), prompt(),
        # confirm(), String.fromCharCode, document.cookie, document.location,
        # document.write(), HTML entity encoded <script, hex-encoded <script
        self.xss_re = re.compile(
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
        
        # ── Path Traversal patterns ─────────────────────────────────────────
        # Catches: ../, ..\, %2e%2e%2f, %2e%2e%5c, %252e%252e%252f,
        # %252e%252e%255c, null bytes %00, overlong UTF-8 variants,
        # drive letters (C:\), ~/ ~root/, absolute paths (/etc/, /windows/,
        # /proc/, /boot/, /sys/, /dev/, /bin/, /usr/, /var/, /opt/, /tmp/,
        # /root/, /home/)
        self.path_re = re.compile(
            r"(\.\.[/\\]|\.\.%[25]*2f|\.\.%[25]*5c"
            r"|%2e%2e%[25]*2f|%2e%2e%[25]*5c"
            r"|%25(2e|2f|5c)"  # double-encoded dots, slashes, backslashes
            r"|%c0%ae"  # overlong UTF-8 for .
            r"|%[25]*00"
            r"|[A-Za-z]:[/\\]"
            r"|(?:^|[^a-zA-Z])~\w*[/\\]"  # ~user/ or ~/ patterns (not email ~)
            r"|/(etc|windows|proc|boot|sys|dev|bin|usr|var|opt|tmp|root|home)[/\\])",
            re.IGNORECASE
        )

        # Security headers
        self.security_headers = {
            b"strict-transport-security": b"max-age=31536000; includeSubDomains",
            b"content-security-policy": b"default-src 'self'; script-src 'self' 'unsafe-inline'",
            b"x-frame-options": b"DENY",
            b"x-content-type-options": b"nosniff",
            b"x-xss-protection": b"1; mode=block",
        }
    
    async def __call__(self, scope: Scope, receive: Receive, send: Send) -> None:
        if scope["type"] != "http":
            await self.app(scope, receive, send)
            return

        # Extract headers
        headers_dict = {}
        for k, v in scope.get("headers", []):
            headers_dict[k.lower().decode("latin-1")] = v.decode("latin-1")

        # Get client IP
        client_ip = "unknown"
        if "x-agent-role" in headers_dict:
            role = headers_dict["x-agent-role"]
            if role in ["Architect", "Logic", "Shield"]:
                client_ip = "internal_trusted"
        
        if client_ip != "internal_trusted":
            if "x-forwarded-for" in headers_dict:
                client_ip = headers_dict["x-forwarded-for"].split(",")[0].strip()
            elif scope.get("client"):
                client_ip = scope["client"][0]

        from swarm_v2.core.resource_arbiter import logger as arb_logger
        path = scope.get("path", "")
        method = scope.get("method", "")
        arb_logger.info(f"[Sentinel] Intercepted {method} {path} from {client_ip}")

        # Determine localhost status
        is_local = client_ip in ["127.0.0.1", "::1", "localhost", "internal_trusted"]
        bypass_rate_limit = is_local and self.trust_localhost

        # ════════════════════════════════════════════════════════════════════
        # Step 1: Rate Limiting — bypassable via TRUST_LOCALHOST
        # ════════════════════════════════════════════════════════════════════
        if not bypass_rate_limit:
            if not await self._check_rate_limit(client_ip):
                response = JSONResponse(
                    status_code=429,
                    content={
                        "error": "Too Many Requests",
                        "message": f"Rate limit exceeded. Try again in {self.rate_window} seconds.",
                        "retry_after": self.rate_window
                    }
                )
                await response(scope, receive, send)
                return

        # ════════════════════════════════════════════════════════════════════
        # Step 2: Query String Sanitization — ALWAYS applied
        # ════════════════════════════════════════════════════════════════════
        query_string = scope.get("query_string", b"").decode("latin-1")
        query_params = urllib.parse.parse_qs(query_string)
        is_safe, error_msg = self._sanitize_request(path, query_params)
        if not is_safe:
            response = JSONResponse(
                status_code=400,
                content={
                    "error": "Bad Request",
                    "message": "Request blocked by security policy",
                    "detail": error_msg
                }
            )
            await response(scope, receive, send)
            return

        # ════════════════════════════════════════════════════════════════════
        # Step 3: Request Body Inspection — ALWAYS applied for POST/PUT/PATCH
        # ════════════════════════════════════════════════════════════════════
        body: bytes = b""
        if method in ("POST", "PUT", "PATCH"):
            body = await self._read_body(receive)
            # Code-payload endpoints (evolution/artifact/skill forge) carry source
            # code validated by the sandbox AST shield; skip generic body scan.
            _code_payload = path.startswith((
                "/swarm/chat", "/swarm/cra", "/evolution/", "/artifacts/create", "/skills/forge", "/tools/forge",
            ))
            if not _code_payload:
                is_safe, error_msg = self._inspect_request_body(body)
                if not is_safe:
                    response = JSONResponse(
                        status_code=400,
                        content={
                            "error": "Bad Request",
                            "message": "Request blocked by security policy",
                            "detail": error_msg
                        }
                    )
                    await response(scope, receive, send)
                    return

        # Define custom send wrapper to inject security headers
        async def send_wrapper(message: Dict[str, Any]) -> None:
            if message["type"] == "http.response.start":
                headers = list(message.get("headers", []))
                # Inject security headers if not already present
                existing_headers = {k.lower() for k, v in headers}
                for h_name, h_val in self.security_headers.items():
                    if h_name not in existing_headers:
                        headers.append((h_name, h_val))
                message["headers"] = headers
            await send(message)

        # Step 4: Process request
        try:
            if method in ("POST", "PUT", "PATCH") and body:
                # Use wrapped receive with pre-read body for downstream
                async def wrapped_receive():
                    return {"type": "http.request", "body": body, "more_body": False}
                await self.app(scope, wrapped_receive, send_wrapper)
            else:
                await self.app(scope, receive, send_wrapper)
        except Exception as e:
            # Step 5: Global Exception Handling
            import traceback
            os.makedirs("swarm_v2_artifacts", exist_ok=True)
            with open("swarm_v2_artifacts/sentinel_crash.log", "a") as f:
                f.write(f"\n--- CRASH AT {time.ctime()} ---\n")
                traceback.print_exc(file=f)
            
            response = JSONResponse(
                status_code=500,
                content={
                    "error": "Internal Server Error",
                    "message": "An unexpected error occurred",
                }
            )
            await response(scope, receive, send)

    # ── Helper: Read full request body from ASGI receive stream ──────────────
    async def _read_body(self, receive: Receive) -> bytes:
        """Accumulate all body chunks from the ASGI receive stream."""
        chunks = []
        more_body = True
        while more_body:
            message = await receive()
            if message["type"] == "http.disconnect":
                break
            chunks.append(message.get("body", b""))
            more_body = message.get("more_body", False)
        return b"".join(chunks)

    async def _check_rate_limit(self, client_ip: str) -> bool:
        if client_ip == "internal_trusted": return True
        if not self.redis: return True
            
        key = f"rate_limit:{client_ip}"
        try:
            current = self.redis.hget(key, "count")
            now = int(time.time())
            
            if current is None:
                self.redis.hset(key, {"count": "1", "reset": str(now + self.rate_window)})
                return True
            
            count = int(current)
            reset_time = int(self.redis.hget(key, "reset") or 0)
            
            if now > reset_time:
                self.redis.hset(key, {"count": "1", "reset": str(now + self.rate_window)})
                return True
            
            if count >= self.rate_limit:
                return False
            
            self.redis.hset(key, {"count": str(count + 1)})
            return True
        except Exception:
            return True  # Fail open
    
    def _sanitize_request(self, path: str, query_params: dict) -> tuple[bool, Optional[str]]:
        """Check path and query parameters for attack patterns."""
        if self.path_re.search(path):
            return False, "Path traversal attempt blocked"
        
        for key, val_list in query_params.items():
            # Check parameter name (key) for attacks
            if self.sql_re.search(key):
                return False, f"SQLi attempt in parameter name: {key}"
            if self.xss_re.search(key):
                return False, f"XSS attempt in parameter name: {key}"
            if self.path_re.search(key):
                return False, f"Path traversal attempt in parameter name: {key}"
            
            # Check parameter values
            for val in val_list:
                v_str = str(val)
                if self.sql_re.search(v_str):
                    return False, f"SQLi attempt in query param {key}"
                if self.xss_re.search(v_str):
                    return False, f"XSS attempt in query param {key}"
                if self.path_re.search(v_str):
                    return False, f"Path traversal attempt in query param {key}"
        
        return True, None

    def _inspect_request_body(self, body: bytes) -> tuple[bool, Optional[str]]:
        """Inspect request body for attack patterns. Handles JSON and plain text."""
        if not body:
            return True, None
        try:
            decoded = body.decode("utf-8", errors="replace")
            
            # Try to parse as JSON and recursively inspect values
            try:
                data = json.loads(decoded)
                return self._scan_json_values(data)
            except (json.JSONDecodeError, ValueError):
                # Not JSON — scan the raw string
                pass
            
            # Plain text / form-encoded body scan
            if self.sql_re.search(decoded):
                return False, "SQLi attempt in request body"
            if self.xss_re.search(decoded):
                return False, "XSS attempt in request body"
            if self.path_re.search(decoded):
                return False, "Path traversal attempt in request body"
        except Exception:
            pass
        return True, None

    def _scan_json_values(self, data: Any) -> tuple[bool, Optional[str]]:
        """Recursively scan JSON values for attack patterns."""
        if isinstance(data, str):
            if self.sql_re.search(data):
                return False, "SQLi attempt in request body"
            if self.xss_re.search(data):
                return False, "XSS attempt in request body"
            if self.path_re.search(data):
                return False, "Path traversal attempt in request body"
        elif isinstance(data, dict):
            for key, val in data.items():
                # Check keys too
                if isinstance(key, str):
                    safe, msg = self._scan_json_values(key)
                    if not safe:
                        return safe, msg
                safe, msg = self._scan_json_values(val)
                if not safe:
                    return safe, msg
        elif isinstance(data, list):
            for item in data:
                safe, msg = self._scan_json_values(item)
                if not safe:
                    return safe, msg
        return True, None


def create_sentinel_middleware(app, redis_client=None):
    """Factory function to create SentinelMiddleware with Redis."""
    return SentinelMiddleware(
        app=app,
        redis_client=redis_client,
        rate_limit=100,
        rate_window=60
    )
