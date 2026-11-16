# A module that loads a compiled shared library is a HOST FFI SHIM. It exists
# only to call out to native code; inside the kernel there is nothing to call
# out TO, because the kernel IS the native code. Its contract usually already
# exists directly (native_ops_bridge's INSTELLA_GATE lives in
# kernel_instructions.rs). Not a porting target.
FFI_SHIM_MARKERS = ("ctypes", "CDLL", "dlopen", "LoadLibrary")

#!/usr/bin/env python3
# Surface-diff sweep: find "shell without body" ports.
#
# WHY: the MANUAL-marker count only tracks hollow FUNCTIONS the porter emitted.
# It is blind to a module whose struct shell was kept but whose BODIES were
# dropped entirely - sentinel.rs, base_agent.rs and chain_of_verification.rs all
# had ZERO markers while missing real capability.
#
# METHOD: for every Python core module that has a Rust counterpart, compare the
# public surface. A module whose Python defines N functions/classes but whose
# Rust defines almost nothing is a candidate for dropped bodies. Rank by the
# gap, then inspect the outliers by hand.
#
# This finds CANDIDATES, not proof. Every hit still needs a human read.
import ast, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORE = ROOT / "swarm_v2" / "core"
RSDIR = ROOT / "trm_uefi_kernel" / "src"

# helpers/tooling that were never porting targets
SKIP = {
    "__init__", "porting_engine", "porting_fixpass", "final_porting_pass",
    "manual_repair_pass", "manual_clearing_engine", "complete_singletons",
    "find_truncations", "physics_scoring_correction", "ultrawork_tranche",
    "audit_wiring", "system_verify",
}
for i in range(1, 13):
    SKIP.add(f"porting_fixpass{i}")
    SKIP.add(f"final_porting_pass{i}")
    SKIP.add(f"manual_repair_pass{i}")


def py_surface(path):
    """Public names the Python module defines: functions, classes, methods."""
    try:
        tree = ast.parse(path.read_text(encoding="utf-8", errors="replace"))
    except SyntaxError:
        return set()
    names = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            if not node.name.startswith("_"):
                names.add(node.name)
    return names


def rs_surface(path):
    """Public names the Rust module exposes: pub fn, pub struct/enum/trait, methods."""
    txt = path.read_text(encoding="utf-8", errors="replace")
    names = set()
    for m in re.finditer(r"pub\s+(?:unsafe\s+)?fn\s+(\w+)", txt):
        names.add(m.group(1))
    for m in re.finditer(r"pub\s+(?:struct|enum|trait)\s+(\w+)", txt):
        names.add(m.group(1))
    # methods inside impl blocks (pub or not - they are the module's behaviour)
    for m in re.finditer(r"\bfn\s+(\w+)", txt):
        names.add(m.group(1))
    for m in re.finditer(r"^(?:pub\s+)?(?:struct|enum)\s+(\w+)", txt, re.M):
        names.add(m.group(1))
    return names


def snake(s):
    return re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s).lower()


# ── Classification ─────────────────────────────────────────────────────────
# A gap is only a DEFECT if the module's function belongs in a no_std kernel.
# Many shells are CORRECT: torch/PEFT training, Redis mocks, Discord/Telegram
# adapters, mmap arenas and external LLM APIs cannot exist in the kernel, and
# the porter was right to leave them as stubs. Classify by what the Python
# actually imports so the report separates real backlog from by-design stubs.
HARD_HOST = {
    # Genuinely impossible in a no_std kernel: host ML frameworks, OS process
    # introspection, memory mapping, network clients. NOTE: numpy is NOT here.
    # numpy is array math and the kernel does that natively with f32/f64 loops -
    # listing it as host-only hid real backlog behind a false impossibility.
    "torch", "transformers", "peft", "psutil", "mmap", "redis",
    "discord", "requests", "httpx", "socket", "subprocess", "openai", "anthropic",
}
NET_PKGS = {"socket","requests","httpx","aiohttp","urllib","websockets"}


def classify(pyf):
    imps = imports_of(pyf)
    # FFI shim: the module exists only to call out to a compiled shared library.
    # Inside the kernel there is nothing to call out TO - the kernel IS the
    # native code - so its contract usually already exists directly (e.g.
    # native_ops_bridge's INSTELLA_GATE lives in kernel_instructions.rs).
    try:
        _src = pyf.read_text(encoding="utf-8", errors="replace")
    except Exception:
        _src = ""
    if any(m in _src for m in FFI_SHIM_MARKERS):
        return "HOST-ONLY", "ffi-shim(ctypes)"
    hard = imps & HARD_HOST
    if hard:
        return "HOST-ONLY", ",".join(sorted(hard))
    if imps & NET_PKGS:
        return "NET-ONLY", ",".join(sorted(imps & NET_PKGS))
    return "KERNEL-CANDIDATE", ",".join(sorted(imps & {"asyncio","threading","os","time"}))


def imports_of(pyf):
    import ast as _ast
    try:
        tree = _ast.parse(pyf.read_text(encoding="utf-8", errors="replace"))
    except SyntaxError:
        return set()
    out = set()
    for n in _ast.walk(tree):
        if isinstance(n, _ast.Import):
            for a in n.names:
                out.add(a.name.split(".")[0])
        elif isinstance(n, _ast.ImportFrom) and n.module:
            out.add(n.module.split(".")[0])
    return out


# ── Shape classification ───────────────────────────────────────────────────
# Import-based classification over-flags asyncio users. `asyncio` alone does not
# mean "unportable": it can be incidental (`async def` that never awaits), or it
# can be a genuine host daemon loop (`while True: await asyncio.sleep(30)`),
# which the kernel CANNOT host - there is no executor. Split by shape so a
# daemon loop is not reported as a dropped kernel capability.
#
# This corrected a false positive in an earlier run: `background_tasks` was
# reported as a kernel-candidate gap, but it is 1 of only 3 true async daemons
# and its Rust stub is CORRECT.
def shape(pyf):
    src = pyf.read_text(encoding="utf-8", errors="replace")
    has_loop = bool(re.search(r"while\s+True", src))
    async_n = len(re.findall(r"\basync\s+def", src)) + len(re.findall(r"\bawait\b", src))
    daemon = any(w in src.lower() for w in
                 ["asyncio task", "api lifespan", "background loop", "forever",
                  "daemon", "heartbeat"])
    if has_loop and (async_n or daemon):
        return "ASYNC-DAEMON"          # host-only, stub is correct
    if async_n:
        return "ASYNC-OTHER"           # needs review: incidental or structural
    if "socket" in src.lower() or "http" in src.lower():
        return "NET"
    return "SYNC-PORTABLE"             # directly portable


def crate_roots():
    """Stems of this crate's ROOT files, which are never modules.

    A crate root is *declared by Cargo*, not by a `mod X;` line, so it must be
    excluded from the orphan scan or it is always reported as never-compiled.
    `main` was hardcoded for that reason; `lib` was not, so src/lib.rs (a real
    `lib` target per `cargo metadata --no-deps`) was reported as a false
    positive. Read the targets instead of hardcoding them, so neither target
    type can regress. Auto-discovered paths (src/lib.rs, src/main.rs) are the
    Cargo default and are assumed when Cargo.toml declares no explicit path.
    """
    roots = {"main", "lib"}        # Cargo defaults: src/main.rs, src/lib.rs
    manifest = RSDIR.parent / "Cargo.toml"
    try:
        import tomllib
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    except Exception:
        return roots
    for key in ("lib", "bin"):
        targets = data.get(key)
        if isinstance(targets, dict):
            targets = [targets]
        for target in targets or []:
            path = target.get("path") if isinstance(target, dict) else None
            if path:
                roots.add(pathlib.Path(path).stem)
            else:
                roots.add("main" if key == "bin" else "lib")
    return roots


def orphaned_modules():
    """Files on disk that NO `mod X;` declaration pulls in.

    A file that is never declared is never compiled - so it cannot be warned
    about, cannot be dead-code-eliminated, and does not exist as far as the
    linker is concerned. `ses.rs` sat in this state until 2026-09-22: the file
    was present, the registry listed it as ported, and nothing type-checked it.
    This detector is what caught it.
    """
    declared = set()
    for f in RSDIR.glob("*.rs"):
        t = f.read_text(encoding="utf-8", errors="replace")
        for m in re.finditer(r"^\s*(?:pub\s+)?mod\s+(\w+)\s*;", t, re.M):
            declared.add(m.group(1))
        for m in re.finditer(r'#\[path\s*=\s*"([^"]+)"\]\s*(?:pub\s+)?mod\s+(\w+)', t):
            declared.add(m.group(2))
    files = {p.stem for p in RSDIR.glob("*.rs")}
    files -= crate_roots()         # crate roots are targets, not modules
    return sorted(files - declared)


def main():
    rows = []
    for pyf in sorted(CORE.glob("*.py")):
        mod = pyf.stem
        if mod in SKIP or mod.startswith("_"):
            continue
        rsf = RSDIR / f"{mod}.rs"
        if not rsf.exists():
            continue
        py = py_surface(pyf)
        rs = rs_surface(rsf)
        if not py:
            continue
        rs_norm = {snake(x) for x in rs}
        missing = {n for n in py if snake(n) not in rs_norm and n not in rs and snake(n) not in rs}
        pyl = len(pyf.read_text(encoding="utf-8", errors="replace").splitlines())
        rsl = len(rsf.read_text(encoding="utf-8", errors="replace").splitlines())
        # Name matching alone is noisy (a faithful port often renames things).
        # SIZE COLLAPSE is the reliable signal: a 1400-line module reduced to 63
        # Rust lines has lost its bodies. base_agent.py (1427 -> 63) is a known
        # true positive, which is what this ranking is calibrated against.
        shrink = 1.0 - (rsl / pyl if pyl else 1.0)
        kind, why = classify(pyf)
        rows.append((shrink, rsl, pyl, mod, sorted(missing)[:6], kind, why))
    rows.sort(reverse=True)
    for kind in ("HOST-ONLY", "NET-ONLY", "KERNEL-CANDIDATE"):
        subset = [r for r in rows if r[5] == kind and r[0] >= 0.60 and r[2] >= 60 and r[1] < r[2]]
        print(f"=== {kind}: {len(subset)} ===")
        if kind == "KERNEL-CANDIDATE":
            # split by SHAPE: a `while True: await sleep` daemon is host-only and
            # its stub is correct; only SYNC-PORTABLE is an unambiguous backlog.
            by_shape = {}
            for row in subset:
                by_shape.setdefault(shape(CORE / f"{row[3]}.py"), []).append(row)
            for sh in ("SYNC-PORTABLE", "ASYNC-OTHER", "ASYNC-DAEMON", "NET"):
                rows_sh = by_shape.get(sh, [])
                if not rows_sh:
                    continue
                print(f"  -- {sh}: {len(rows_sh)}")
                if sh == "SYNC-PORTABLE":
                    print(f"  {'shrink':>6} {'pyL':>5} {'rsL':>5}  {'module':<32} missing-api")
                    for shrink, rsl, pyl, mod, sample, _k, _w in rows_sh:
                        print(f"  {shrink:6.0%} {pyl:5} {rsl:5}  {mod:<32} {', '.join(sample[:3])}")
                else:
                    print("     " + ", ".join(r[3] for r in rows_sh))
            print()
        else:
            for shrink, rsl, pyl, mod, sample, _k, why in subset:
                print(f"  {mod:<34} py={pyl:5} rs={rsl:4}  [{why}]")
        print()
    kc = sum(1 for r in rows if r[5] == "KERNEL-CANDIDATE" and r[0] >= 0.60 and r[2] >= 60 and r[1] < r[2])
    print(f"=== {len(rows)} ported modules compared ===")
    print(f"    KERNEL-CANDIDATE (real backlog): {kc}")
    print(f"    HOST-ONLY / NET-ONLY (correctly stubbed): "
          f"{sum(1 for r in rows if r[5] != 'KERNEL-CANDIDATE' and r[0] >= 0.60 and r[2] >= 60 and r[1] < r[2])}")
    print()
    orph = orphaned_modules()
    print(f"=== NEVER-COMPILED FILES (no `mod` declaration): {len(orph)} ===")
    if orph:
        for o in orph:
            n = len((RSDIR / f"{o}.rs").read_text(encoding="utf-8", errors="replace").splitlines())
            print(f"    {o:<34} {n} lines  <- not type-checked by the compiler")
    else:
        print("    none - every .rs file is declared")
    print()
    print("NOTE: this finds CANDIDATES, not proof. Each still needs a read.")
    print("NOTE: asyncio-importing candidates need re-architecting, not transliteration -")
    print("      the kernel has no executor (see the polling model in baremetal_transition.rs).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
