# Monad Journal — Procedure for Publishing a Breakthrough

The Monad Journal of Autonomous Discovery (ISSN 2998-277X) is the platform's
record of its own findings. This procedure exists to make journaling
**reliable**, because a defect was found that silently discarded entries.

---

## The defect that was swallowing entries (fixed 2026-09-22)

`get_sovereign_journal_articles()` read the manifest with a **bare relative
path**:

```rust
std::fs::read_to_string("phoenix_journal_audit_manifest.json")
```

That only resolves when the server's working directory happens to be the
repository root. Started anywhere else, the read failed silently and the
function **fell back to the hardcoded article list compiled into the binary**.

Observed consequence: the manifest held **94** articles while
`GET /journal/articles` served **88**. Five articles written this session were
invisible. The failure was silent — no error, no log line, just an older list.

**Fixed:** the reader now searches the repository root and its ancestors before
falling back. **The fix takes effect on the next server restart** — a running
binary keeps its old behaviour until restarted.

---

## Procedure

### 1. Append to the manifest (the live source)

```
F:\Development sites\TRM-Swarm-OS-v2\phoenix_journal_audit_manifest.json
```

Article schema:

```json
{
  "id": 191,
  "section": "Computational Epistemology",
  "title": "Article 191 — <claim in one line>",
  "phoenix_status": "CONFIRMED",
  "evidence_logs": ["path/to/raw/artifact", "..."],
  "phoenix_audit_note": "<how it was verified, in one sentence>",
  "finding": "<full text: what was tested, what was measured, what the result was>"
}
```

Ids are sequential from the current maximum (190 at the time of writing).

### 2. Status must be earned

| Status | Requires |
|---|---|
| `CONFIRMED` | A raw artefact exists **and** was reproduced. Name it in `evidence_logs`. |
| `SPEC` | The design exists but has never been executed. |
| `ASPIRATIONAL` | No artefact. A stated intention. |

Do not write `CONFIRMED` for something that was only designed. The honest
status is the point of the journal.

### 3. Update the counts

`total_articles_audited` and the `summary` block (empirical_confirmed /
spec_models / aspirational_unexecuted) must match the article list. Never leave
a count that contradicts the contents.

### 4. Restart the server from the repository root

```
cd "F:/Development sites/TRM-Swarm-OS-v2"
./trm_uefi_kernel/host_council_server/target/release/host_council_server.exe
```

### 5. Verify it is actually served

```python
import json, urllib.request
d = json.loads(urllib.request.urlopen("http://127.0.0.1:8022/journal/articles", timeout=30).read())
ids = sorted(a["id"] for a in d["papers"])
print(len(ids), ids[-1])
```

**A written manifest is not a published article.** Confirm the served count
increased. This step is what would have caught the silent-fallback defect.

---

## What belongs in the journal

Journal a finding when it **changes what the platform believes about itself**:

- A claim that did not reproduce under measurement
- A defect that was invisible to existing checks
- A capability newly verified on real hardware
- A security weakness found or closed

Do **not** journal routine feature work, porting progress, or version bumps.
The journal is for discoveries, not a changelog — the CHANGELOG is for that.

---

## Cross-reference

The current self-assessment, including which claims are ceremonial rather than
validated, lives in `docs/SYSTEM_SELF_KNOWLEDGE.md`. Cite it when journaling,
and update it when a finding changes the picture.
