#!/usr/bin/env python3
# Differential test: the no_std port vs the ORIGINAL Python security regexes.
#
# WHY: the mechanical porter turned self_healing.py's SQL/XSS/path regexes into
# literal-substring needles. The regexes used \s+ / \s* between tokens; the
# needles used one literal space. Measured consequence (before the fix):
#   7 SQL-injection bypasses, 1 XSS bypass, 1 path-traversal bypass
# that the original Python caught and the Rust port MISSED.
# A two-space "UNION  SELECT" defeated the sanitizer entirely.
#
# false negative = port misses what Python caught -> SECURITY HOLE
# false positive = port fires where Python was clean -> safe
#
# Exit 0 when there are no false negatives.
import pathlib, re, subprocess, sys, tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
RS = ROOT / "trm_uefi_kernel" / "src" / "self_healing.rs"
RUSTC = r"C:\Users\shawn\.cargo\bin\rustc.exe"

PY = ROOT / "swarm_v2" / "core" / "self_healing.py"

def load_patterns():
    """Pull the three compiled patterns out of the ORIGINAL Python module, so this
    test can never drift from the source it is comparing against."""
    src = PY.read_text(encoding="utf-8")
    out = {}
    for name in ("SQL_RE", "XSS_RE", "PATH_RE"):
        i = src.index(name + " = re.compile(")
        j = src.index("re.IGNORECASE", i)
        body = src[i:j]
        # take the concatenated r"..." fragments
        frags = re.findall(r'r"((?:[^"\\]|\\.)*)"', body)
        out[name] = re.compile("".join(frags), re.IGNORECASE)
    return out["SQL_RE"], out["XSS_RE"], out["PATH_RE"]

CORPUS = [
 "UNION SELECT","UNION ALL SELECT","INSERT INTO","DELETE FROM","DROP TABLE",
 "DROP DATABASE","ALTER TABLE","CREATE TABLE","TRUNCATE TABLE","INTO OUTFILE",
 "INFORMATION_SCHEMA",
 # whitespace variants the regex allows via \s+ / \s* - these were the bypasses
 "UNION  SELECT","UNION\tSELECT","UNION\nSELECT","INSERT  INTO","CAST (","CAST  (",
 "CONVERT  (","SELECT id FROM users","OR 1 = 1","OR 1=1","AND 2 = 3","or  1  =  1",
 "1' OR '1'='1","0xDEADBEEF","'; DROP TABLE users; --","; ALTER TABLE x",
 "WAITFOR DELAY '0:0:5'","pg_sleep(5)","BENCHMARK(1000000,MD5(1))","admin'--",
 "<script>alert(1)</script>","<img src=x onerror=alert(1)>","<body onload=alert(1)>",
 "<div onclick=x>","javascript:alert(1)","javascript: alert(1)","eval(1)",
 "document.cookie","String.fromCharCode(65)","&lt;script&gt;",
 "<img src=x onerror = alert(1)>","<div onclick = x>","<img src=x onload = alert(1)>",
 "../../etc/passwd","..\\..\\windows\\system32","..%2fetc/passwd","%2e%2e%2fetc/passwd",
 "C:\\Windows\\System32","/etc/passwd","%c0%ae%c0%ae/etc/passwd","..%252fetc/passwd",
 "%252e%252e%252fetc/passwd","/proc/self/environ","/windows/system32/config",
 "the union selected a new table","SELECT the best option","I am considered",
 "concatenate the strings","a path to /home/user","hello world",
]

def run_rust(texts, tmp):
    """Compile the REAL self_healing.rs on the host and run the corpus through it."""
    (tmp / "src").mkdir(parents=True, exist_ok=True)
    (tmp / "src" / "self_healing.rs").write_text(RS.read_text(encoding="utf-8"), encoding="utf-8")
    def esc(s):
        return s.replace("\\", "\\\\").replace('"', '\\"').replace("\t", "\\t").replace("\n", "\\n")
    entries = ",\n".join(f'    ("{esc(x)}")' for x in texts)
    (tmp / "src" / "main.rs").write_text(
        'mod self_healing;\nuse self_healing::InputSanitizer;\n'
        'fn main() {\n'
        f'    let corpus: &[&str] = &[\n{entries}\n    ];\n'
        '    let s = InputSanitizer::new();\n'
        '    for t in corpus {\n'
        '        let r = format!("{:?}", s.sanitize(t.as_bytes()));\n'
        '        println!("{}{}{}", if r.contains("SqlInjection") {"1"} else {"0"},\n'
        '                 if r.contains("Xss") {"1"} else {"0"},\n'
        '                 if r.contains("PathTraversal") {"1"} else {"0"});\n'
        '    }\n}\n', encoding="utf-8")
    b = subprocess.run([RUSTC, "--edition", "2021", "-O", "-o", "dt.exe", "src/main.rs"],
                       cwd=tmp, capture_output=True, text=True, timeout=560)
    if b.returncode != 0:
        print("rustc failed:\n", b.stderr[:900]); sys.exit(2)
    out = subprocess.run([str(tmp / "dt.exe")], cwd=tmp, capture_output=True, text=True, timeout=180)
    return [l for l in out.stdout.strip().splitlines()]

def main():
    sql_re, xss_re, path_re = load_patterns()
    tmp = pathlib.Path(tempfile.mkdtemp(prefix="sec_diff_"))
    rows = run_rust(CORPUS, tmp)
    if len(rows) != len(CORPUS):
        print(f"harness returned {len(rows)} rows for {len(CORPUS)} inputs"); sys.exit(2)
    fn = fp = 0
    for txt, row in zip(CORPUS, rows):
        py = bool(sql_re.search(txt)); px = bool(xss_re.search(txt)); pp = bool(path_re.search(txt))
        rs, rx, rp = (c == "1" for c in row)
        if (py or px or pp) and not (rs or rx or rp):
            fn += 1
            which = "+".join(n for n, f in (("sql", py), ("xss", px), ("path", pp)) if f)
            print(f"  FALSE NEGATIVE [{which}]  {txt!r}")
        elif not (py or px or pp) and (rs or rx or rp):
            fp += 1
            print(f"  false positive (safe)      {txt!r}")
    print(f"\n=== {len(CORPUS)} strings | false negatives: {fn} | false positives: {fp} ===")
    if fn:
        print("FAIL - the port is weaker than the Python it replaced")
    return 1 if fn else 0

if __name__ == "__main__":
    sys.exit(main())
