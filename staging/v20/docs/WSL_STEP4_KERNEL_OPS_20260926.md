# WSL Step 4: what WSL is for — decided, and the missing `.so` produced (2026-09-26)

## The decision, from measurements

| question | measurement | consequence |
|---|---|---|
| can WSL build Rust? | `command -v cargo` → MISSING; `/home/shawn/.profile` references `/home/shawn/.cargo/env`, which does not exist | No. It cannot be a second Rust host. |
| can WSL build C/C++? | `/usr/bin/g++` (GCC 15), `/usr/bin/nm`, `/usr/bin/python3` all present | Yes — this is a real build host for native artifacts. |
| which side holds the space? | WSL `/` **808 GB free**; Windows `C:` 119 GB free | WSL is the archive/dataset side; `C:` is the constraint for any large ingestion. |

**Recorded role for the WSL side: data/archive + the Linux build host for the platform's `.so`
artifacts.** It is not a peer runtime and not a second Rust workspace; it is where Linux-shaped
artifacts are produced and where the knowledge bases live until the fabric's ingestion path fits
them (`docs/WSL_RECONCILIATION_PLAN_20260923.md`, steps 2 and 5).

## The artifact the PHOENIX report named, produced

`swarm_v2/core/libswarm_kernel_ops.so` — **21,632 B, sha256 `7aff4be98f8889c1…`**, built by the tree's
own `swarm_v2/core/build_kernel_ops.sh` with g++ on this host.

- **8 exported entry points**: `c_mamba3_create_state`, `c_mamba3_destroy_state`, `c_mamba3_step`,
  `c_instella_gate_topk`, `c_mamba3_step_persistent`, `c_nvme_mamba_init`, `c_nvme_mamba_bind_slot`,
  `c_nvme_mamba_flush` — a **superset** of the Windows DLL's four.
- **`c_kleene_quantize_avx2` is in neither library.** The PHOENIX report lists it against a different
  artifact (`c_k3_kleene_ops`, 0 files present), so the gates that bind it never had a satisfiable
  target here. Recording the absence is the honest half of "produce the missing artifact".

### Why it did not exist: a real source defect

`swarm_v2/core/swarm_kernel_ops.h` used `size_t` without including `<cstddef>`. Under GCC 15 that is a
hard error at line 12, cascading into every struct member — the build script could not run at all. The
Windows DLL predates that toolchain (MSVC/libstdc++ versions leak `size_t` through `<cstdint>`), which is
why the defect was invisible for as long as the DLL was the only artifact. One include added, with the
reason in the header.

## The proof

1. **Sandbox first.** The source, header and script were copied to `~/so_build` and built *there* —
   nothing in the tree was touched until the build was green.
2. **The `.so` against the Windows DLL**, same C source, identical inputs
   (`scratch/probe_kernel_ops.py` → `scratch/compare_kernel_ops.py`): **29 checks, 0 failed**.
   mamba3 output max deviation **5.200e-08** over 100 steps; expert ids, active count and gate weights
   **exactly equal** (0.000e+00) at every `top_k` in {0, 1, 4, 8, 9}; the four error paths (null input,
   unknown handle, after destroy, out-of-range `top_k`) agree — including that a rejected `top_k` writes
   no ids.
3. **The consumer, not the directory.** `swarm_v2/core/native_ops_bridge.py` resolves
   `SO_PATH = <dir>/libswarm_kernel_ops.so` and raises `FileNotFoundError` without it — so this module was
   present and *unusable* on Linux. It now constructs: `mamba3 step(1.0) = 0.03200000151991844`
   (= 32 lanes × 1.0 × dt 0.001, exactly), `instella gate top_k=4 → ids [13, 22, 17, 45]`,
   `nvme_init → 0`, `bridge: OK`.
4. **Reproducible on this host.** A fresh build from the *tree's* source after the header fix yields the
   same sha256 — so the artifact in the tree is the source in the tree.
5. **The probe's arithmetic checks itself.** Its first mamba step is `0.020964926` for an input of
   `0.655…` — 32 × 0.655 × 0.001 — which is the library's own formula, reproduced by hand.

## Stated limits (on the artifact, not in a footnote)

- **`-march=native`.** The script compiles for this host's CPU (Ryzen 9 5900HX, AVX2+FMA). The `.so` is
  not portable to a different CPU and is not claimed to be.
- **The Windows DLL is untouched** (392,192 B, still the PE the kernel loads on this host): the `.so` is an
  addition, not a replacement, and the kernel keeps using the DLL.
- **The voltage-strip cdylib cannot load on Linux** — `strip_native_ops_bridge.dll: invalid ELF header`.
  The bridge reports that in `_NATIVE_REASON` instead of pretending to be native; that is a declared
  platform limit, not a silent fallback.
- **step 3's retirement bookkeeping is unaffected**: this produces an artifact, it does not retire a tree.

## Adjacent defect found by this work (fixed)

`swarm_v2/routes/system.py::get_native_ops_bridge_status` answered `{"status": "online"}` after importing
only the two **struct classes** — it never touched the library — and reported
`"dispatch_overhead": "0.00 ns Direct Memory Call"`, a number nobody measured. That is the platform's own
"declared where it must be measured" class, so the handler now constructs `NativeKernelOpsBridge` and reports
what happened, plus the structure sizes read from the structures:

| host | `status` | `loaded` | `reason` | input/output bytes |
|---|---|---|---|---|
| Windows | `unavailable` | false | `OSError: [WinError 193] %1 is not a valid Win32 application` (the `.so` is ELF; the kernel uses the DLL) | 99840 / 44 |
| WSL Linux | `online` | true | — | 99840 / 44 |

Both branches were exercised on their own host (`scratch/probe_native_ops_status.py`), and the invented
latency field is gone rather than reformatted. The route lives on `:8021`, which the supervisor declares with
an empty `cmd` (it can be neither launched nor resurrected), so this changes the offline Python surface, not
the live Rust runtime.
