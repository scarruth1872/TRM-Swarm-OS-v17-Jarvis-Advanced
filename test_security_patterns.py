"""
Functional test for SentinelMiddleware and InputSanitizer security patterns.
"""
import re
import sys

# ── SQLi patterns (identical to those in sentinel.py) ─────────────────
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

# ── Test cases ───────────────────────────────────────────────────────
tests = [
    # SQL Injection
    ("sqli", "UNION SELECT * FROM users", SQL_RE, True),
    ("sqli", "UNION ALL SELECT password FROM admins", SQL_RE, True),
    ("sqli", "SELECT * FROM users WHERE id=1", SQL_RE, True),
    ("sqli", "DROP TABLE users", SQL_RE, True),
    ("sqli", "OR 1=1", SQL_RE, True),
    ("sqli", "OR '1'='1'", SQL_RE, True),
    ("sqli", "1; DROP TABLE users--", SQL_RE, True),
    ("sqli", "0xDEADBEEF", SQL_RE, True),
    ("sqli", "SLEEP(5)", SQL_RE, True),
    ("sqli", "WAITFOR DELAY 0:0:5", SQL_RE, True),
    ("sqli", "pg_sleep(5)", SQL_RE, True),
    ("sqli", "INTO OUTFILE '/tmp/evil'", SQL_RE, True),
    ("sqli", "LOAD_FILE('/etc/passwd')", SQL_RE, True),
    ("sqli", "INFORMATION_SCHEMA.TABLES", SQL_RE, True),
    ("sqli", "CHAR(65,66,67)", SQL_RE, True),
    ("sqli", "hello world", SQL_RE, False),
    ("sqli", "my name is SELECTor", SQL_RE, False),
    ("sqli", "normal text with or", SQL_RE, False),

    # XSS
    ("xss", "<script>alert(1)</script>", XSS_RE, True),
    ("xss", "<img src=x onerror=alert(1)>", XSS_RE, True),
    ("xss", "<svg onload=alert(1)>", XSS_RE, True),
    ("xss", "<body onload=alert(1)>", XSS_RE, True),
    ("xss", "javascript:alert(1)", XSS_RE, True),
    ("xss", "data:text/html,<script>alert(1)</script>", XSS_RE, True),
    ("xss", "document.cookie", XSS_RE, True),
    ("xss", "document.location='http://evil'", XSS_RE, True),
    ("xss", "<img onerror=alert(1)>", XSS_RE, True),
    ("xss", "<div onmouseover=alert(1)>hover</div>", XSS_RE, True),
    ("xss", "<input onfocus=alert(1)>", XSS_RE, True),
    ("xss", "eval('malicious')", XSS_RE, True),
    ("xss", "String.fromCharCode(65)", XSS_RE, True),
    ("xss", "hello world", XSS_RE, False),
    ("xss", "this is normal text", XSS_RE, False),

    # Path Traversal
    ("pt", "../../../etc/passwd", PATH_RE, True),
    ("pt", "..\\..\\windows\\system32", PATH_RE, True),
    ("pt", "%2e%2e%2f%2e%2e%2fetc/passwd", PATH_RE, True),
    ("pt", "%252e%252e%252fetc/passwd", PATH_RE, True),
    ("pt", "C:\\Windows\\System32", PATH_RE, True),
    ("pt", "/etc/passwd", PATH_RE, True),
    ("pt", "/proc/self/environ", PATH_RE, True),
    ("pt", "~root/.bashrc", PATH_RE, True),
    ("pt", "hello world", PATH_RE, False),
    ("pt", "C:\\\\Program Files", PATH_RE, True),
]

fail_count = 0
pass_count = 0

for category, payload, regex, should_block in tests:
    result = bool(regex.search(payload))
    if result == should_block:
        pass_count += 1
    else:
        fail_count += 1
        print(f"  FAIL [{category}] payload={payload!r} expected_block={should_block} got={result}")

total = pass_count + fail_count
print(f"\nResults: {pass_count}/{total} passed, {fail_count}/{total} failed")
sys.exit(0 if fail_count == 0 else 1)
