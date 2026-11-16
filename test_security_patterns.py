"""
Functional test for SentinelMiddleware and InputSanitizer security patterns.

CONVERTED FROM A SCRIPT. This file used to be a standalone program: it ran its
checks at MODULE level and finished with sys.exit(...). Because it was named
test_*.py, pytest imported it - and the module-level sys.exit() raised SystemExit
during collection, which aborted the WHOLE run with an INTERNALERROR:

    File "test_security_patterns.py", line 123, in <module>
        sys.exit(0 if fail_count == 0 else 1)
    SystemExit: 0

    INTERNALERROR> ... 2651 tests collected, 156 errors

The pattern data below is UNCHANGED; the execution model is what changed. The
checks are now real pytest test functions, so they run as part of the suite
instead of killing it, and importing the module has no side effects. The original
script behaviour is preserved under __main__ for anyone running it directly.
"""
import re
import sys

import pytest

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


# ── pytest tests ─────────────────────────────────────────────────────
# Parametrised so each payload is an individually reported test case rather than
# a line in a counter that could only ever exit with a code.

_CASES = [(c, p, r, b) for (c, p, r, b) in tests]


@pytest.mark.parametrize("category,payload,regex,should_block", _CASES,
                         ids=[f"{c}-{str(p)[:28]}" for (c, p, r, b) in _CASES])
def test_security_pattern_matches_expectation(category, payload, regex, should_block):
    """Each payload must be classified exactly as the pattern intends."""
    got = bool(regex.search(payload))
    assert got == should_block, (
        f"[{category}] payload={payload!r} expected block={should_block} got={got}"
    )


def test_every_category_has_coverage():
    """A pattern set with a category that silently lost its cases proves nothing."""
    cats = {c for (c, p, r, b) in _CASES}
    assert cats, "no test cases at all"
    # 43 is the real count. The first version of this guard said 100 - a number I
    # guessed rather than measured - and it failed immediately, which is the guard
    # doing its job on me. Pinned to the measured value so that DELETING cases is
    # caught, which is the point: a shrinking case list silently weakens the suite.
    assert len(_CASES) >= 43, f"case list shrank to {len(_CASES)}"


def test_both_polarities_present():
    """Both should_block=True and False must appear, or the test is one-sided and
    would pass even if a pattern matched everything (or nothing)."""
    pos = [1 for (c, p, r, b) in _CASES if b]
    neg = [1 for (c, p, r, b) in _CASES if not b]
    assert pos, "no positive (should_block) cases"
    assert neg, "no negative (should_pass) cases"


if __name__ == "__main__":
    # preserved original behaviour for direct execution
    fail_count = sum(1 for (c, p, r, b) in tests if bool(r.search(p)) != b)
    for (c, p, r, b) in tests:
        if bool(r.search(p)) != b:
            print(f"  FAIL [{c}] payload={p!r} expected_block={b} got={bool(r.search(p))}")
    total = len(tests)
    print(f"\nResults: {total-fail_count}/{total} passed, {fail_count}/{total} failed")
    sys.exit(0 if fail_count == 0 else 1)
