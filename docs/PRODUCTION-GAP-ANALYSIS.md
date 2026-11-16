PRODUCTION GAP ANALYSIS — DECISION DOCUMENT
TRM Swarm OS v2
Measured 2026-11-03 on the RX 6700 XT host
Repo: F:\Development sites\TRM-Swarm-OS-v2
Supersedes the release-readiness sections of docs/PRODUCTION-READINESS-PLAN.md
where they disagree. Raw command output: evidence/PRODUCTION_GAP_MEASURED.txt


VERDICT

    NO-GO for production release.

    The system is running and coherent, but two blockers are release-stopping
    and both are attestation/authorization defects rather than functional ones:

      - the authoritative council accepts state-changing commands from any
        local process with no credential at all, and
      - the audit chain's published attestation no longer matches the live
        chain, which is the single failing test in the release gate.

    Phase 1 (ledger integrity) and Phase 2 (loopback binding) are both real
    and both measurably done in substance. What is NOT done is the part the
    plan assumed followed automatically: authentication, and manifest
    regeneration. The plan marks Phase 1 DONE at commit 175106f with "51/51
    tests pass"; that was true at 07:56 today and is false now because the
    ledger kept appending after the manifest was frozen.

    Three of the seven gaps below were previously reported as fixed or as
    smaller than they are. Those corrections are called out inline.


WHAT WAS RE-VERIFIED AND IS GENUINELY DONE

    Ledger chain, as a chain.
        swarm_v2_global_memory/system_event_ledger.jsonl
        22,723 lines, 0 unparseable, seq 1..22723 contiguous,
        0 duplicate seq, 0 missing seq, 0 prev_hash breaks.
        Command: python one-liner over the file (evidence section 2).
        The single-writer lock fix is present and used by every append:
        swarm_v2/core/cognitive_ingestion_engine.py:195-235 allocates seq and
        appends inside _ledger_file_lock (msvcrt.locking / fcntl.flock).
        Commit 175106f exists (git log --oneline).

    Loopback binding.
        9/9 listeners on 127.0.0.1, none on 0.0.0.0.
        netstat -ano, evidence section 3. Bind default at
        trm_uefi_kernel/host_council_server/src/main.rs:3284.

    Services up.
        All nine health probes returned 200 with process identity confirmed by
        PID via tasklist. 8099 reports rust_kernel_bound:true and
        council_authority:"rust:8022"; 8022 /arc/pool returns 200.

    Launcher reproduces the system.
        The plan's P1-1 is stale. 8593 appears 7 times in
        START_TRM_PLATFORM.bat (embeddings start at line 202); FAILED is
        aggregated at lines 137-218 and exits /b 1; the log lock falls back to
        %TEMP% at lines 29-30; :check_only aggregates CHK_FAIL over all nine
        services and exits 1.

    Kernel compiles.
        cargo check --release: exit 0, 0 errors, 1,848 warning lines
        (lib 11, bin 1,846 with 11 duplicates).


GAP TABLE

    Columns: ID | Severity | Gap | Measured evidence (command or file:line)

    P0-1  P0  No authentication anywhere on the control plane.
              grep -nEi "authorization|bearer|api[_-]?key|shared.?secret"
              over host_council_server/src/main.rs returns zero matches;
              grep for TRM_API_TOKEN/TRM_SHARED_SECRET across *.rs/*.py
              returns zero matches. Eleven POST routes match on method+path
              only (main.rs:1601,1660,1690,1767,1821,1890,2242,2350,2390,
              2782,3057-3063) including /council/enact and /secrets/*.
              Live: curl -X POST -d '{}' http://127.0.0.1:8022/council/propose
              ran a full 14-agent deliberation and created proposal 1181
              ("Operator Sovereign Directive", NO_CONSENSUS) with no
              credential. Verified via GET /council/proposals (181 records).
              Residual exposure is now local-process only, not LAN — loopback
              binding is a mitigation, not an authorization model.

    P0-2  P0  Audit attestation does not match the live chain; release gate red.
              python -m pytest -q -m 'not integration' --timeout=60
              => 1 failed, 50 passed, 16 deselected, 40.64s, RC=1.
              Failing test: tests/test_phoenix_ledger_schema.py:135
              test_legacy_event_stream_phoenix_compliance, comparing head hash
              774e8217... to manifest 8ae0ed46...
              phoenix_event_stream_audit_manifest.json: total_events 22,521,
              final_chain_hash 8ae0ed46..., mtime 07:56.
              Ledger mtime 10:24, head 22,723+ and climbing.
              Proven to be staleness, not corruption: line 22,521's hash is
              exactly 8ae0ed46..., i.e. the manifest is an accurate snapshot of
              a moment that has passed. It is a static file with no
              regeneration hook, so it diverges by construction whenever the
              platform runs.
              Consequence: the plan's "51/51 green" and "manifest hash matches"
              claims at docs/PRODUCTION-READINESS-PLAN.md:198-204 are stale.
              Note the plan's path is also wrong: the manifest lives at repo
              root, not under swarm_v2_global_memory/.

    P1-1  P1  The SSM cannot be load-bearing. This is an unmet requirement,
              not a bug, and it is honestly labelled.
              main.rs:2384 publishes status "ACTIVE (ADVISORY)" with the
              measured read-out: 0.458 and 0.474 leave-one-topic-out against a
              0.455 always-majority baseline. main.rs:1776-1782 gates the SSM
              at ssm_gate=false unless COUNCIL_SSM_GATE=hard, and the default
              body path passes false. So the SSM is wired, deterministic and
              auditable, but contributes nothing to verdicts. Any release note
              implying SSM-driven decisions is unsupported.

    P1-2  P1  The forbidden voltage constant 0.9115 is still emitted by six
              code paths, not two.
              grep -rn "0\.9115" --include=*.py --include=*.rs .
              => quantize_gemma270m.py:167, quantize_gpt2.py:217,
                 quantize_qwen17b.py:16,165, quantize_whisper.py:157,
                 turbovec_quantize_weights.py:8,108,
                 main.rs:522,619 (council response templates).
              The plan named quantize_clip.py:155 and council_paper162.py:12;
              both are now clean, so the stated fix was applied to two of at
              least eight sites. Artifacts from the six remaining paths carry a
              fabricated number, while /system/voltage correctly reports
              measured:false.

    P1-3  P1  No release identity.
              git status --porcelain | wc -l => 253 (31 modified tracked
              files). git tag => only v19.024-baremetal-binding, no tag on the
              current tree. A release cannot be cut, reproduced or rolled back
              from this state.

    P1-4  P1  The integration half of the suite has never been run.
              python -m pytest -q -m integration --collect-only
              => 16/67 tests collected (51 deselected). These are the tests
              that exercise live cross-service behaviour; every green number
              quoted anywhere in this repo excludes them.

    P2-1  P2  Warning volume destroys the defect signal.
              cargo check --release => 1,848 warning lines. Sample:
              non-snake-case fields in src/trm_7m_full.rs:25-26. Not a
              blocker; it means a new warning cannot be noticed.

    P2-2  P2  Jarvis has no real health endpoint.
              curl 127.0.0.1:3000/health => 200 with content-type text/html
              (Vite index.html). /api/status and /api/health => 500.
              The launcher's check at START_TRM_PLATFORM.bat:250 passes on the
              substring "doctype", so 3000 can serve an error page and still
              be declared healthy.

    P2-3  P2  The propose path is synchronous with no job handle.
              curl -m 5 POST /council/propose returned HTTP 000 (client
              timeout) while the server kept deliberating and completed
              afterwards. A client cannot distinguish "still working" from
              "gone". Evidence: proposal 1181 exists despite no response.

    P2-4  P2  Ledger retention is unbounded.
              10.75 MB / 22,723 events and growing during this audit. No
              rotation in the writer (cognitive_ingestion_engine.py:195-235);
              the plan's earlier rotation behaviour was external. Tied to
              P0-2: any rotation must regenerate the manifest atomically.

    Not counted as gaps, verified and closed: loopback binding (P0 in the old
    plan) and the launcher defects (P1 in the old plan). See the "genuinely
    done" section for the commands.


SEQUENCED REMAINING WORK

    Order is dependency-driven: authorization first because everything else
    touches the same server; attestation second because it is the release gate;
    the rest can run in parallel afterwards. Estimates are engineer-days for
    one person who already knows this codebase.

    Phase A — Authenticate the control plane.          1.0 day   closes P0-1
        1. Add a shared-secret check (header or HMAC over body) at the top of
           handle_request (main.rs:1466), before route matching, exempting
           GET /health and GET /council/status. Reject with 401, not a silent
           404.
        2. Source the secret from an environment variable or OS keyring, never
           a literal in the binary; fail closed at startup if unset.
        3. Apply the same guard to 8099's write routes in swarm_v2/app_v2.py.
        4. Acceptance: POST /council/propose without the header returns 401 and
           creates no proposal; with the header returns 200. Re-run the exact
           probe that created proposal 1181.
        Note: loopback stays as defence in depth; it is not a substitute.

    Phase B — Make the attestation self-maintaining.   0.5 day   closes P0-2
        1. Regenerate final_chain_hash and counts from the ledger head on every
           append-under-lock (or on a bounded timer), writing the manifest via
           temp-file + atomic rename so readers never see a half-written file.
        2. Record event_count and head seq in the manifest so a stale manifest
           is detectable without hashing 10 MB.
        3. Keep the .unverified_backup sidecar; never renumber or delete.
        4. Acceptance: test_phoenix_ledger_schema.py passes; append 100 events
           and confirm the manifest follows without a test edit; run the suite
           twice with traffic in between.

    Phase C — Remove the fabricated constant.          0.5 day   closes P1-2
        1. Six call sites listed in P1-2: emit the derived value with
           measured:false, or the doctrine-derived phi_v, never 0.9115.
        2. Also fix the two council prose templates (main.rs:522,619) — they
           currently assert "ΦV = 0.9115 maintained across rails" on hardware
           with no voltage sensor.
        3. Acceptance: grep -rn "0\.9115" returns only audit/errata files.

    Phase D — Cut a release identity.                  0.5 day   closes P1-3
        1. Resolve the 253 dirty entries: commit the 31 tracked modifications,
           decide on the untracked artifacts, keep runtime state out of the tag.
        2. Tag, and record the tag in evidence/ with the measured numbers.

    Phase E — Run the integration suite.               1.0 day   closes P1-4
        1. Run the 16 integration tests against the live nine services; fix or
           explicitly quarantine each failure with a reason.
        2. Make the release gate run both markers, not just 'not integration'.

    Phase F — Warning and health hygiene.              1.0 day   closes P2-1/2
        1. cargo fix the 225 suggested bin sites, then set a warning budget and
           fail CI above it (target < 100).
        2. Give Jarvis a real GET /health returning JSON with a status field,
           and change the launcher check from "doctype" to that field.

    Phase G — Optional, not release-blocking.          0.5 day   closes P2-3/4
        1. Return 202 with a job id from /council/propose and expose
           GET /council/proposal/<id> for polling.
        2. Add bounded retention with manifest regeneration in the same lock.

    Total to a defensible release: 4.5 days (A-E plus F), with G deferrable.
    The old plan estimated three days for its three blockers; that estimate
    omitted authentication itself and assumed the ledger fix ended at the
    test passing once.


RESIDUAL RISK ACCEPTED BY THIS VERDICT

    - The SSM remains advisory. If a stakeholder requires SSM-driven decisions
      at release, P1-1 is a hard blocker and no schedule above applies: the
      read-out does not beat a 0.455 baseline and that is a modelling problem,
      not an integration one.
    - 1,848 warnings remain. They are not defects, but they mean no one can
      tell a defect from noise in a build log.
    - The Rust port backlog (44 kernel-candidates) is out of scope here and
      was not re-measured; it is not on the decision path.

================================================================================
RESOLUTIONS SINCE THIS ANALYSIS WAS WRITTEN
================================================================================

P0-2 — RESOLVED, commit daa0863. The failure was a test design defect, not
corruption. The manifest is a frozen prefix attestation; the test asserted the
ledger's LAST record equals the manifest hash, which can only hold in the instant
after a re-attest. tests/test_phoenix_ledger_schema.py now verifies (a) the
attested prefix is intact — the record at seq total_events still hashes to the
manifest hash — and (b) every record past the prefix has a valid prev_hash chain.
That is strictly stronger than before: a corrupted manifest hash and a rewritten
prefix record both raise (proven in memory, no files touched).
scripts/regen_phoenix_manifest.py re-attests deliberately and refuses to write if
it sees a chain break. Manifest refreshed to 22,742 events, 0 breaks.
Measured: 0 failed / 50 passed. Note the ledger appended one more event during
the verification itself, which is exactly why the old assertion could not hold.

PORT BOUNDARY — DECIDED, docs/PORT-STATUS.md. The "44 kernel-candidates" figure
is stale; the list has 45 rows. Measured buckets: 0 PORTED-AND-USED, 13
PORTED-UNUSED (real logic, no production caller), 19 STUBBED (no logic to wire),
9 HOST-ONLY, 4 NET-ONLY. The council binary links exactly four kernel modules,
none of them on the candidate list. Decision: port nothing for release. The port
boundary is already correct; the remaining work is optional wiring and marking 40
modules excluded. This removes "fully port the kernel" from the release path
entirely — it was never the blocker.

STILL OPEN. P0-1 (no authentication on the Rust control plane) is untouched and
remains the only hard blocker. P1 items in the table above are unchanged.

  Corrected 0.9115 measurement (this analysis said "6 code paths + 2 council
  templates"; that is roughly right but the locations matter):
    - swarm_v2/core/quantize_{gemma270m,gpt2,qwen17b,whisper}.py and
      turbovec_quantize_weights.py each stamp "phi_v": 0.9115 into generated
      model metadata, and two of them repeat the claim in their module docstring.
      That is fabricated provenance written into model cards.
    - host_council_server/src/main.rs carries it in agent prompt text (line 522
      daily cadence, line 619 Pulse telemetry "ΦV = 0.9115 maintained across
      VDDCR_CPU, VDDCR_SOC, DRAM and VDD_GPU rails") and in two article blobs
      (lines 693, 697).
    - No live endpoint serves it. Measured 2026-11-03: :8099/system/voltage and
      :8587/:8588/system/voltage all 404; :8099/system/resources returns VRAM
      fields only. The disagreement recorded in evidence/claim_audit_1.json is
      historical. The defect is fabricated text and metadata, not live telemetry.

P1 NEW — THE RUST KERNEL HAS NO RUNNABLE TESTS. Verified statically 2026-11-03:
grep '#\[test\]' over trm_uefi_kernel/src/*.rs returns exactly 1 (digital_twin.rs:257),
and there is 1 cfg(test) block in the whole crate. main.rs declares #![no_std] and
its own #![panic_handler], so `cargo test` cannot link the std harness and dies with
E0152 duplicate lang item panic_impl: 0 tests run, exit 101. The Python side has 50
unit + 18 integration tests; the Rust side has one test that cannot execute.
This matters because the council binary is the authoritative server: 6,633 LOC of
Rust serving state-changing endpoints with no automated verification. Not a blocker
for a loopback-only release, but it is the reason every Rust claim in this repo has
had to be re-measured by hand.

P1 NEW — THE LAUNCHER'S JARVIS GATE CAN PASS ON AN HTML PAGE. Measured
2026-11-03: GET http://127.0.0.1:3000/health returns '<!doctype html>' -- the SPA
fallback, not a health payload. The launcher's readiness check for Jarvis matches
on a marker string, so a page that merely contains the word can satisfy it. The
check-only run printed Jarvis as OK while /health was serving index.html, which
means the launcher would have declared a broken Jarvis healthy.
  Fix, after the Jarvis /health route is repaired: assert the response is JSON
  (body starts with '{') rather than matching a substring, for every service that
  is supposed to serve JSON. This is the same class of defect as the original
  'port is listening therefore healthy' bug, one layer up: 'a 200 with the right
  word in it therefore healthy'.

P0-1 — RESOLVED, commit 3211c62. Authenticated writes on the Rust council.
auth_gate() runs before any routing so a new write route inherits the check.
COUNCIL_AUTH_TOKEN set -> X-Council-Token required, else 401 JSON. Unset -> writes
allowed only because the bind is loopback, enforced by a startup fail-closed: a
non-loopback bind with no token prints FATAL and exits 1. Verified per case:
0.0.0.0 without token exits 1; GET /health 200 with no token; POST no-token 401;
wrong token 401; correct token 404 (gate passed it to routing, so it is not
blanket-blocking). Restored to loopback no-token: POST /council/propose 200, tally
13/0/1, identical to the pre-auth baseline. Evidence: evidence/COUNCIL_AUTH_P0_MEASURED.txt

Residual, stated so it is not mistaken for done: no rate limiting, no per-agent
identity, no replay protection; :8099 Python council is protected by loopback only
and has no token check of its own; 3000/3001/5183 are loopback-bound but
unauthenticated. All nine services verified 200 after the restart.

VERDICT UPDATE. With P0-1 and P0-2 both closed and the port boundary decided, no
P0 remains against a loopback-only single-operator release. The remaining gating
items are P1: the Rust kernel has no runnable tests (cargo test dies with E0152),
13 of 18 integration tests never touch the live ports, the launcher's Jarvis gate
can pass on an HTML page, and release hygiene (tag + dirty tree) is unstarted.
That is a GO for local operation and NO-GO for anything exposed or multi-user.
