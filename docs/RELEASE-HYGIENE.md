RELEASE HYGIENE INVENTORY
TRM Swarm OS v2  --  F:\Development sites\TRM-Swarm-OS-v2
Snapshot taken 2026-11-03 10:49 EST, branch sovereign-v19-clean.
Dirty-tree classification at HEAD a80ea37; this document written at HEAD d3eb006.

This is a read-only audit. No file was committed, tagged, staged, ignored or
edited except this document. Nine services were running during the snapshot
and other agents were committing to this branch, so HEAD and the dirty count
move underneath the numbers below. Re-run "git status --porcelain" before you
act on any of it.


1. WHY THIS DOCUMENT EXISTS

The prior measurement said "253 dirty entries, no release tag". Both halves are
literally true and jointly meaningless without a breakdown, because almost all
of the dirt is the platform writing to its own state files, not unfinished
work. The decision the numbers should support is: what gets committed, what
gets ignored, and what the tag points at. That is sections 2 through 6.

At snapshot time the count is 269 entries, not 253. It moved while this audit
ran (280 at 10:44, 269 at 10:49) because concurrent agents committed docs and
the live services kept appending. The composition is stable; the number is
not.


2. BUCKET TABLE

    bucket                     count   share   gitignored as-is
    ------------------------   -----   -----   ----------------
    GENERATED-BUILD-OUTPUT        4     1.5%   no (0 of 4)
    RUNTIME-STATE               244    90.7%   no (1 exception)
    SOURCE-CHANGE                20     7.4%   no
    UNKNOWN                       1      0.4%   no
    ------------------------   -----   -----
    total                       269   100.0%

Reading of the table: 244 of 269 entries (90.7%) are machine-written state or
generated output. 20 entries are work a human or agent would want in the
release. The release decision is therefore about 20 paths, not 269.


3. BUCKET CONTENTS

   GENERATED-BUILD-OUTPUT  (4)

      Machine output of a build or bench run. Never hand-edited, regenerable,
      should not be in the tree at all.

      untracked  START_TRM_PLATFORM.bat.bak_20261029
      untracked  START_TRM_PLATFORM.bat.bak_prehardening_20261103_103858
      untracked  scratch/ssm_dataset.npz
      untracked  trm_uefi_kernel/host_test/trm7m_fast_path_sram_manifest.json

   RUNTIME-STATE  (244)

      Written by the nine live services while they run. Ledgers, agent memory,
      caches, research archives, chat history. By design these are never clean
      while the platform is up. 15 of the 244 are already tracked by git, which is
      the real defect: gitignore cannot help until they are untracked.

      modified   Jarvis-Advanced-main/jarvis_chat_history.json
      modified   Jarvis-Advanced-main/user_data/ux2I2mb7PdN5imXt5u9wVWSvZld2/chat_history.json
      untracked  LOG
      untracked  data/council_evidence_index.json
      modified   data/council_ledger.json
      modified   swarm_v2_artifacts/DDR_ANTIBODIES.json
      modified   swarm_v2_artifacts/kanban_state.json
      modified   swarm_v2_artifacts/redis_state.json
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793360990.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793362850.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793383410.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793383861.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793409731.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793410253.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793412175.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793498630.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793585113.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793626362.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793628425.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793628785.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793629432.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793638005.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793638379.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793650495.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793710974.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793719150.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793719402.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793719443.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793720164.md
      untracked  swarm_v2_artifacts/research/research_ai_code_generation_research_1793720336.md
      untracked  swarm_v2_artifacts/research/research_ai_safety_and_alignment_research_1793360982.md
      untracked  swarm_v2_artifacts/research/research_ai_safety_and_alignment_research_1793362841.md
      untracked  swarm_v2_artifacts/research/research_ai_safety_and_alignment_research_1793383402.md
      untracked  swarm_v2_artifacts/research/research_ai_safety_and_alignment_research_1793383851.md
      untracked  swarm_v2_artifacts/research/research_ai_safety_and_alignment_research_1793409723.md
      untracked  swarm_v2_artifacts/research/research_ai_safety_and_alignment_research_1793410251.md
      untracked  swarm_v2_artifacts/research/research_ai_safety_and_alignment_research_1793412167.md
      untracked  swarm_v2_artifacts/research/research_ai_safety_and_alignment_research_1793498622.md
      untracked  swarm_v2_artifacts/research/research_ai_safety_and_alignment_research_1793585104.md
      untracked  swarm_v2_artifacts/research/research_ai_safety_and_alignment_research_1793626351.md
      ... 204 more of the same kind (see section 3.5 summary)

   SOURCE-CHANGE  (20)

      Real edits to code, tests, launchers, evidence and docs. This is the work
      the tag is waiting on.

      modified   Jarvis-Advanced-main/server.ts
      modified   dashboard/package.json
      modified   dashboard/vite.config.js
      untracked  evidence/COUNCIL_K3_FALSIFICATION_POSTFIX.txt
      modified   openfang
      modified   swarm_v2/app_v2.py
      modified   swarm_v2/core/cognitive_stack.py
      modified   swarm_v2/core/llm_router.py
      modified   swarm_v2/core/test_trm_standalone.py
      modified   swarm_v2/scripts/test_cognitive_stack_gemma3_270m.py
      modified   swarm_v2_synthesized/mcp_cloudstorage_server.py
      modified   swarm_v2_synthesized/registry.json
      untracked  tools/run_ci.py
      untracked  tools/smoke_test_platform.py
      untracked  tools/verify_council_parity.py
      modified   trm-sovereign-portal
      modified   trm_uefi_kernel/host_council_server/src/main.rs
      modified   trm_uefi_kernel/host_test/src/arc_object_sram.rs
      modified   trm_uefi_kernel/trm_companion_bridge/src/main.rs
      modified   trm_uefi_kernel/trm_companion_proxy.py

   UNKNOWN  (1)

      Zero-byte file created by a malformed Windows redirect (nul with a stray
      backtick-paren in the name). Artifact of a shell typo on 2026-10-29, not
      content. Delete it.

      untracked  nul`)

   RUNTIME-STATE breakdown by directory

      swarm_v2_artifacts/research/          225   untracked research .md, 14 MB
      swarm_v2_memory/                        5   tracked agent memory JSON, 33 MB
      Jarvis-Advanced-main/                   2   chat history (1 tracked, 1 in user_data/)
      data/                                   2   council_ledger.json (5.3 MB, tracked), evidence index
      swarm_v2_artifacts/                     4   DDR_ANTIBODIES, kanban_state, redis_state, semantic_cache
      swarm_v2_learned_skills/                2   manifest + manifest.backup
      swarm_v2_reconnaissance/                1   findings.json
      trm_uefi_kernel/host_council_server/data/ 1 council_ledger.json (untracked duplicate)
      trm_uefi_kernel/host_council_deliberation/ 1 council_strategic_deliberation.json
      LOG                                     1   56-byte stray log file, no extension
      scratch/                                0   (ssm_dataset.npz moved to GENERATED)


3.5  THE SINGLE BIGGEST SOURCE OF DIRT

225 untracked .md files in swarm_v2_artifacts/research/ -- 83.7% of the whole
dirty count, 14 MB. They are timestamped outputs of the research agent loop
(research_<topic>_<epoch>.md), written on a schedule, never read by the
platform at runtime, and never referenced by a commit. Nothing about the
release depends on them. Ignore that directory and the dirty count drops from
269 to 44.


4. GITIGNORE COVERAGE AND PROPOSED LINES

Method: "git check-ignore -v" on every dirty path. Result: 0 of 269 dirty
paths are matched by an existing ignore rule. The only near-miss is
evidence/COUNCIL_K3_FALSIFICATION_POSTFIX.txt, which is explicitly
un-ignored by the deliberate "!evidence/*.txt" exception -- correct, and it
belongs in SOURCE-CHANGE, not here.

4.1  What the current .gitignore does cover

      target/, **/target/*, dist/, build/, node_modules/, __pycache__/,
      *.py[cod], .pytest_cache/, *.log, *.txt (repo-wide), *.bin, *.gguf,
      .env, .env.*, .swarm/, swarm_v2_global_memory/, *.pdb, *.o

      -> That is why no compiled artefact appears in the dirty list. The
         build-output bucket is 4 entries, not hundreds.
      -> Consequences worth knowing: "*.txt" is ignored repo-wide, so the
         evidence .txt files are only visible because of the explicit
         "!evidence/*.txt" exception. Anything you add under evidence/ with
         a .txt extension will NOT show up in git status until you add a
         matching "!" line.

4.2  What it does NOT cover, and the lines to add

      (a) RUNTIME-STATE -- 229 untracked paths

          swarm_v2_artifacts/research/
          swarm_v2_artifacts/DDR_ANTIBODIES.json
          swarm_v2_artifacts/kanban_state.json
          swarm_v2_artifacts/redis_state.json
          swarm_v2_artifacts/semantic_cache.json
          swarm_v2_memory/
          swarm_v2_learned_skills/
          swarm_v2_reconnaissance/
          data/council_ledger.json
          data/council_evidence_index.json
          trm_uefi_kernel/host_council_server/data/
          Jarvis-Advanced-main/jarvis_chat_history.json
          Jarvis-Advanced-main/user_data/
          scratch/
          /LOG

          Note /LOG rather than LOG: a bare LOG would match a directory
          named LOG anywhere in the tree. The leading slash anchors it to
          the repo root, which is where the stray file is.

      (b) GENERATED-BUILD-OUTPUT -- 4 paths

          *.bak
          *.bak_*
          *.npz
          trm_uefi_kernel/host_test/trm7m_fast_path_sram_manifest.json

          *.npz is the one to think about: if any .npz is a checked-in
          fixture rather than a scratch tensor dump, ignore only the
          directory ("scratch/") instead of the extension. In this tree the
          only dirty .npz is scratch/ssm_dataset.npz, so the narrower
          "scratch/" line already covers it, and "*.npz" is optional.

      (c) UNKNOWN -- 1 path, not a gitignore case

          The file literally named  nul`)  cannot be matched safely by a
          pattern (backtick, paren, no extension). Delete it once.

4.3  The part no .gitignore line can fix

      15 RUNTIME-STATE paths are already TRACKED. Ignore rules do not apply
      to tracked files; they will stay dirty forever, and every service
      restart produces new diffs. These are:

          Jarvis-Advanced-main/jarvis_chat_history.json
          Jarvis-Advanced-main/user_data/ux2I2mb7PdN5imXt5u9wVWSvZld2/chat_history.json
          data/council_ledger.json
          swarm_v2_artifacts/DDR_ANTIBODIES.json
          swarm_v2_artifacts/kanban_state.json
          swarm_v2_artifacts/redis_state.json
          swarm_v2_artifacts/semantic_cache.json
          swarm_v2_learned_skills/manifest.backup.json
          swarm_v2_learned_skills/manifest.json
          swarm_v2_memory/Archi_memory.json
          swarm_v2_memory/Devo_memory.json
          swarm_v2_memory/Logic_memory.json
          swarm_v2_memory/Scribe_memory.json
          swarm_v2_memory/Seeker_memory.json
          swarm_v2_reconnaissance/findings.json

      To actually stop the churn, each needs "git rm --cached <path>" once,
      after the ignore line exists, plus a decision that the live baseline
      lives outside the repo rather than in the history. That is
      a commit with real blast radius: the memory JSONs are 33 MB and the
      council ledger is 5.3 MB. Recommend doing it as its own commit, not
      folded into the release commit.


5. WHAT THE TAG SHOULD POINT AT

5.1  History

$ git log --oneline -10   (at snapshot)

    d3eb006 docs: P0-1 closed, verdict updated to GO for local / NO-GO for exposed
    3211c62 security: close P0-1 -- authenticate writes on the council
    9b99545 security: bind the control plane to loopback (TRM_BIND_HOST to override)
    fa18b9a docs: errata on the 51/51 figure -- it was never 51 unit tests
    27f11f6 test: two integration tests could not fail; fix them and the defects behind them
    a80ea37 docs: reconcile the two port documents; record the launcher false-PASS
    05d58fd launcher, voltage constants, port boundary: three measured fixes
    bd00fa4 docs: port boundary decided (port nothing) and the Rust test gap measured
    601b977 docs: record P0-2 resolution and the port-boundary decision in the gap analysis
    daa0863 phoenix: test the attested prefix, not the ledger's moving end

Branch sovereign-v19-clean. Tags on the repo: v19.024-baremetal-binding.
That tag is on an older tree; there is no tag anywhere near HEAD.

5.2  Is the committed HEAD coherent?

Partly, and the honest answer matters more than the convenient one.

The most recent measured suite result in evidence/ is
evidence/INTEGRATION_COVERAGE_MEASURED.txt (2026-11-03 10:52), which records:

    unit         50 passed, 18 deselected     PASS   33.4s
    integration  18 passed, 50 deselected     PASS   34.3s
    ledger       1000/1000 records, 8 workers, 0 duplicates, 0 hash breaks
    OVERALL: PASS    exit 0   (tools/run_ci.py, real exit code,
                              failure path verified by forcing rc=3)

and evidence/PRODUCTION_GAP_MEASURED.txt (same day, 10:31) records:

    python -m pytest -q -m "not integration"   1 failed, 50 passed, 16 deselected
    cargo check --release                      rc=0, 1,848 warning lines

These are not in conflict, they are in sequence. The 10:31 run failed on
tests/test_phoenix_ledger_schema.py asserting the ledger head equals a frozen
manifest; commit daa0863 rewrote it to verify the attested prefix plus the tail
chain instead, and the 10:52 run is green. Read together, the last measured
state is PASS on unit + integration + ledger concurrency, with cargo check
clean.

Three caveats, all measured, none fatal, all of which the release note should
carry:

  - The 10:52 green run was taken WITH uncommitted edits in the tree:
    swarm_v2/core/test_trm_standalone.py and
    swarm_v2/scripts/test_cognitive_stack_gemma3_270m.py were rewritten from
    vacuous to 6 and 15 assertions (evidence/INTEGRATION_COVERAGE_MEASURED.txt,
    "the real gap was not failures -- it was tests that could not fail").
    Tests passing on a dirty tree do not attest the commit. Re-run once
    everything below is committed, and cite that run, not this one.
  - 13 of the 18 integration tests use an in-process ASGI TestClient and never
    touch the nine listening ports. "18 integration tests pass" means "18 ASGI
    tests pass" for 13 of them. Stated in the evidence file itself.
  - 1,848 compiler warnings with 0 errors. Not a blocker; also not a signal.

One more ordering fact, because it changes how the evidence should be read:
HEAD moved while this audit ran. It was a80ea37 when the tree was classified
and is d3eb006 now, because parallel agents committed the P0-1 authentication fix
(3211c62: auth_gate before routing, X-Council-Token, fail-closed bind on a
non-loopback host) and its docs. So evidence/PRODUCTION_GAP_MEASURED.txt is
already stale on exactly one point -- its finding 4, "unauthenticated write
routes", is answered by 3211c62. Its test numbers are not affected.

So: HEAD is coherent enough to tag, but only after the SOURCE-CHANGE set
below is committed and the suite is re-run on that commit.

5.3  What must be committed before tagging

  20 SOURCE-CHANGE paths:

      modified   Jarvis-Advanced-main/server.ts
      modified   dashboard/package.json
      modified   dashboard/vite.config.js
      new file   evidence/COUNCIL_K3_FALSIFICATION_POSTFIX.txt
      modified   openfang
      modified   swarm_v2/app_v2.py
      modified   swarm_v2/core/cognitive_stack.py
      modified   swarm_v2/core/llm_router.py
      modified   swarm_v2/core/test_trm_standalone.py
      modified   swarm_v2/scripts/test_cognitive_stack_gemma3_270m.py
      modified   swarm_v2_synthesized/mcp_cloudstorage_server.py
      modified   swarm_v2_synthesized/registry.json
      new file   tools/run_ci.py
      new file   tools/smoke_test_platform.py
      new file   tools/verify_council_parity.py
      modified   trm-sovereign-portal
      modified   trm_uefi_kernel/host_council_server/src/main.rs
      modified   trm_uefi_kernel/host_test/src/arc_object_sram.rs
      modified   trm_uefi_kernel/trm_companion_bridge/src/main.rs
      modified   trm_uefi_kernel/trm_companion_proxy.py

  Grouped by what they are:

      kernel / council       trm_uefi_kernel/host_council_server/src/main.rs
                             trm_uefi_kernel/trm_companion_bridge/src/main.rs
                             trm_uefi_kernel/trm_companion_proxy.py
                             trm_uefi_kernel/host_test/src/arc_object_sram.rs
      swarm core             swarm_v2/core/cognitive_stack.py
                             swarm_v2/core/llm_router.py
                             swarm_v2/app_v2.py
                             swarm_v2_synthesized/mcp_cloudstorage_server.py
                             swarm_v2_synthesized/registry.json
      tests (make the gate    swarm_v2/core/test_trm_standalone.py
      non-vacuous)           swarm_v2/scripts/test_cognitive_stack_gemma3_270m.py
      dashboard              dashboard/package.json, dashboard/vite.config.js
      jarvis service         Jarvis-Advanced-main/server.ts
      CI + tooling           tools/run_ci.py, tools/smoke_test_platform.py,
                             tools/verify_council_parity.py
      evidence               evidence/COUNCIL_K3_FALSIFICATION_POSTFIX.txt
      submodule pointers     openfang, trm-sovereign-portal

  Two items need a decision rather than a plain commit:

      - openfang and trm-sovereign-portal are gitlinks (mode 160000) whose
        committed SHA differs from the submodule working tree. Decide whether
        the release pin is the new SHA; if the submodule has uncommitted work,
        the parent pointer would reference a commit nobody can fetch.
      - trm_uefi_kernel/host_test/trm7m_fast_path_sram_manifest.json is
        classified GENERATED above, but it is CITED by evidence/ as the source
        of the SRAM numbers. If any document references it as proof, commit it
        as evidence instead of ignoring it. Generated-by-a-run and
        quoted-as-evidence can both be true; evidence wins for a release.

  Also to be resolved before the tag, though not part of the 20:

      - .gitignore lines from 4.2 added and committed
      - the 15 tracked runtime files untracked in their own commit (4.3)
      - the zero-byte  nul`)  file deleted
      - the two .bak launcher copies kept on disk but out of the tree
      - one clean suite run recorded into evidence/ on the final commit
      - tag message naming the measured numbers, not "release"


6. SECRET SCAN

Scope: every dirty path with a .py/.ts/.js/.rs/.json/.bat/.env/.yml extension, 36 files,
node_modules excluded (none are dirty anyway). Pattern set: api_key, token,
password, secret, bearer, credential -- on assignment/colon form only, so
prose mentions do not drown the result. Then a second pass for key material
shapes: PEM private-key blocks, sk-..., gh[pousr]_..., AKIA..., xox[baprs]-.

Result: 47 pattern hits, 1 real finding, 1 borderline, 45 false positives.
No secret value is reproduced in this document.

6.1  REAL -- hardcoded shared secret in source

    Jarvis-Advanced-main/server.ts:2014
    A literal 30-character shared secret is assigned to a const named SECRET
    and used as the HMAC-SHA256 key that mints the bridge auth token for
    kernel calls on UDP 127.0.0.1:8022. The token is a function of that key,
    the task id and a 5-minute time window, so anyone who reads the repository
    can mint valid bridge tokens for any window.
    Severity: medium. Mitigated by the bridge being loopback-only and UDP, not
    by the secret being secret. This file is in SOURCE-CHANGE and the line
    would be committed as-is.
    Fix before tagging: read it from the environment (TRM_BRIDGE_SECRET) and
    rotate the value; if the Rust side compares against the same constant, it
    needs the same change or the bridge breaks silently. This audit did not
    check the Rust side; that check is part of the fix, not done here.

6.2  BORDERLINE -- token-shaped literal, not a credential

    trm_uefi_kernel/trm_companion_bridge/src/main.rs:114
    A field named biometric_token is initialised to a fixed 32-character
    string of the form BIO-KEY-...-0x...  It is returned to clients as an
    attestation label in a status document. It authenticates nothing in the
    code path that builds it, so it is not a usable credential -- but the name
    and shape invite a scanner to flag it, and a reader to believe biometric
    attestation is happening when it is a string literal. Rename it
    attestation_label or make it derived, before someone trusts it.

6.3  FALSE POSITIVES, by category

    - Environment indirection, correct as written: swarm_v2/core/llm_router.py:46
      reads GEMINI_API_KEY from os.environ and errors out if unset. The 50-char
      hit is the f-string URL that interpolates it.
    - Council and memory prose: data/council_ledger.json,
      swarm_v2_memory/Logic_memory.json, and the untracked duplicate
      trm_uefi_kernel/host_council_server/data/council_ledger.json contain the
      words "token", "tokens" and "secret" inside deliberation text about
      context windows and security doctrine. No assignments.
    - swarm_v2_artifacts/DDR_ANTIBODIES.json holds detection regexes such as
      (SECRET_KEY|API_KEY|PASSWORD)\s*=\s*['"] as data. A scanner pattern,
      not an instance.
    - swarm_v2_learned_skills/manifest*.json: 38-char "secret" hits are file
      paths (learners/test_secret_patterns.py style), 8-char hits are the word
      codebase.
    - host_council_server/src/main.rs /secrets/* handlers return const
      placeholders -- "NEW_SECRET_KEY", "MONAD_TRUTH_SEAL_ACTIVE", and
      SHA256:xxxx fingerprints -- never a value. That the routes were
      unauthenticated was a real finding (evidence/PRODUCTION_GAP_MEASURED.txt,
      live probe), but it is an auth defect, not a secret leak, and it was
      closed after that evidence was written, in commit 3211c62.
    - No PEM private key, no sk- key, no GitHub/AWS/Slack token shape matched
      anywhere in the dirty set. No .env or .env.* file is dirty.

Verdict: one credential-shaped literal that should not ship as-is, one
misleading name, zero leaked keys.


7. THE DECISION, IN ONE PARAGRAPH

The dirty count is not 269 pieces of unfinished work; it is 20 pieces of
work wearing 249 pieces of runtime clothing. Commit the 20, ignore the 229
untracked runtime paths with the lines in 4.2 (225 of them are research
markdown), untrack the 15 already-tracked state files in a separate commit,
delete the junk file, fix the one hardcoded bridge secret, then tag the
resulting commit with the 10:52 numbers re-measured on it. HEAD is coherent
today; it is not yet attested.
