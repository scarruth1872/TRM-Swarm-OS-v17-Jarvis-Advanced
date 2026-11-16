# The three remaining skeletons: ported, exported, gated

**Block 145 companion.** `chaos_engine`, `expert_registry` and `persona_matrix` each now have (a) their real
logic in the kernel, (b) a C-ABI strip that links that kernel source by `#[path]` rather than copying it, and
(c) a parity ruler that drives the strip and the oracle through the same cases. **22 checks, 0 failed.**

## The defect they shared

All three shipped strips exported the same generic template — `<module>_execute`, `<module>_monad_commit`,
`<module>_phase_info` — and **no caller anywhere invoked them**. The oracles load the DLL and set
`_NATIVE_LOADED = True` (with an honest `_NATIVE_REASON`), so a module could report "PE CDYLIB loaded" while
nothing had ever crossed the boundary. Reachability was advertised, not exercised. Each strip now exports the
module's **own** api instead, and nothing else referenced the template symbols (grepped across both trees).

| module | what it owed | state now | ruler |
|---|---|---|---|
| `chaos_engine` | an export for a state machine that was already correct | kernel port was already 1:1 (`disable` zeroes, clamp, sample-injected drop); only the shell was missing | **12 checks** |
| `expert_registry` | the resolution RULE, entirely unported | `sanitize_identifier` + `resolve_identifier` (3 stages, first match wins) in the kernel | **4 checks / 18 identifiers** |
| `persona_matrix` | the enforcement decision and the prompt text | `count_violations` + `enforce_decision` (clock injected) + `compiled_prompt` | **6 checks** |

## chaos_engine

The drop decision takes the uniform draw as a **parameter**: the oracle calls `random.random() < dropout_rate`,
so the comparison is the module's logic and the randomness stays the caller's. That is what makes it
reproducible — and therefore checkable. The ruler replays the oracle's own operations and compares after each:
fresh profile, `enable`, the clamp at −0.3/0.0/0.25/1.0/1.5, `set_latency`, `disable` resetting both fields,
and **28 (rate, draw) pairs**; plus the disabled-engine case (rate 1.0, draw 0.0 → no drop on either side).
`strip_chaos_set_dropout_rate` returns the value it stored, so a caller cannot believe an out-of-range rate
stuck.

## expert_registry

The kernel file held `_team: Vec<(String, f64)>` and `get_expert_registry() -> 0 // TODO: complete 1:1 math`;
the rule that actually answers `get_agent` was absent. The oracle's rule, ported in its order:

1. an empty **raw** identifier resolves to nothing (the truthiness test happens *before* cleaning)
2. exact match on a role key
3. case-insensitive match on a role key
4. containment (`p_name.lower() in clean_id.lower()`) — the persona **name/role appears inside the identifier**,
   not the reverse — name first, then role

Row **order is part of the answer** (first match wins), so the table is a vector, never sorted. The registry's
payload is live agent objects and does not cross FFI; what crosses is the rule, which is the comparable
surface. The ruler loads the same table on both sides and compares on 18 identifiers, including
`** Logic (Reasoning Engine)` (sanitize + containment), `` `Memory` ``, `the archivist role`, `ARGUS`,
`unknown_expert`, `""`, `"   "` (python's `" "` is truthy but cleans to `""` → `None`), and two rows sharing a
persona name to prove first-match order rather than a lucky answer.

## persona_matrix

Two surfaces, both checkable:

- **the decision** — `enforce_decision(is_locked, secs_since_reset, window, max_violations, output, forbidden)`
  returns `(state, status, content)`. The oracle reads the clock; here it is **injected**, so the same case is
  replayable. Compared against the oracle's own `PersonaStateMachine` on 8 cases (violation, clean,
  case-insensitive match, near-miss, empty output, two violations at `max_violations=5` → `RESET`, locked
  inside the window, locked-past-window falling through to `IDLE`), including the oracle's sentinel strings.
  A violation count matches the oracle's own `violation_count` attribute, so the count is compared to the
  oracle's number rather than to a re-implementation of it.
- **the prompt** — 577 bytes on both sides, byte-for-byte. The signature is SHA3-256 and **no SHA3 exists in
  this kernel tree**, so it keeps one owner (python): the ruler hashes the *strip's* body with `sha3_256` and
  compares to the signature inside the oracle's own output (`61c106fec4ce438f…`). Equal input, equal digest —
  that is the proof the bytes are identical, not a claim about them.

Two findings recorded rather than smoothed: an **empty forbidden entry counts as a violation** (`"" in text` is
true in python exactly as `contains("")` is in rust — there is deliberately no guard), and the oracle's
`required_lexicon` field is **declared but never read** by `enforce` (verified behaviourally: the decision is
identical with and without it).

## Honest bounds

- **Reachability changed; wiring did not.** The three oracles still never *call* their strip — they import it
  and set the loaded flag. Native execution is now possible and parity-proven; making the oracle prefer the
  native path (with the strip as the single state owner) is the next rung, and it is a real refactor, not a
  one-liner.
- The platform's own registry was re-run and its counts are **unchanged** (PORTED 40 / REAL 226 / EXCLUDED 9 /
  FABRICATED 3 of 278). Removing the two `// TODO` markers did not move a bucket, so this round makes no claim
  that it did.
- `persona_matrix`'s `GenerationContext.identity_kernel` is still a handle (`u64`) in the struct; the typed
  functions take the kernel's contents directly. The struct was left as the mechanical extraction it is.
- Unicode case folding uses rust's `to_lowercase` vs python's `str.lower()`. They agree on ASCII, which is all
  the ruler exercises; a non-ASCII identifier is not covered.
- The GPU strip still has no route, and `load_architecture`/`benchmark_all` (the Vulkan path) remain host-side.
