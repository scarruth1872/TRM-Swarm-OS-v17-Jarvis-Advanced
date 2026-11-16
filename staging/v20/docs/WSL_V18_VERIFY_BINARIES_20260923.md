# The other v18 verify claims — status after recovering the reference (2026-09-23)

All three v18 verify binaries load their reference from the **v17** path, consistent with
`verify_council.rs` and `verify_k3.rs` — that workstream was built against v17.

## `swarm-ssm/src/bin/verify_ssm.rs` — RULE VERIFIED, one harness caveat

Claim: "Runs 100 steps with identical inputs through both; compares y outputs exactly and h-state
evolution", against `c_mamba3_create_state` / `c_mamba3_step`.

**The rule checks out against the v17 source**, line for line:

| element | v18 Rust | v17 C (authoritative) |
|---|---|---|
| state dim | `MAMBA_STATE_DIM = 32` ("VERIFIED against swarm_kernel_ops.h") | `constexpr size_t MAMBA_STATE_DIM = 32` ✓ |
| defaults | `dt = 0.001`, `a = -0.5`, `b = 1.0`, `c = 1.0` | `dt = (dt_init) ? *dt_init : 0.001f`, `A = -0.5f`, `B = 1.0f`, `C = 1.0f` ✓ |
| transition | `dA = 1.0 + a*dt` | `_mm256_fmadd_ps(v_A, v_dt, 1.0f)` ✓ |
| state update | `h' = h*dA + b*x*dt` | `fmadd(v_B*v_input, v_dt, v_h*v_dA)` ✓ |
| readout | `y = Σ h'*c` | `y_accum += temp[k]` over `v_h_new * v_C` ✓ |

I had suspected the crate's `32` was wrong because its own verify binary declares the C struct's
arrays as `[f32; 64]`. **The header settles it: the C dim is 32**, so the crate is right and the
*binaries'* mirror is the odd one out.

**Caveat that matters for the claim:** that mirror declares `h/a/b/c` as `[f32; 64]` while the real
`Mamba3State` is 32-wide per field (`alignas(64)`, `magic_header` first, then four 128-byte arrays).
A direct read of `s->a` through the 64-wide mirror lands inside the real `h`. So the **y** comparison
(done through the API) is sound, but any **direct h-state read** in that binary is reading the wrong
offsets. I cannot run their binary to confirm which path it uses — no Rust toolchain exists in WSL.

**And I cannot reproduce an exact y match**: the C inner loop uses `_mm256_fmadd_ps`, whose single
rounding differs from `f32(a*b) + f32(c)`. A stdlib-Python ruler can verify the *structure* (above)
but not an `exactly equal` numeric claim. Saying so rather than running a ruler built the wrong way —
the alternative would manufacture a difference that is my tooling's, not the port's.

## `swarm-instella/src/bin/verify_instella.rs` — CHECKABLE, not yet run

`c_instella_gate_topk(const InstellaGateInput*, uint8_t top_k, InstellaGateOutput*)` exists in the v17
`.so` **and** in the v19 `.dll` — it is the one C op v19 kept. The C computes a 64-expert dot product
over the 384-dim hidden state, `partial_sort` top-k, softmax over the top-k, and writes
`selected_expert_ids` / `expert_gate_weights`. Reproducing the gate needs the two struct layouts from
`swarm_kernel_ops.h`; not attempted in this pass.

## `swarm-brain/examples/verify_gpu_neural.rs` — LIBRARY FOUND, claim not re-measured

Claim: "byte-identical to CPU, measured **177M/s**", comparing `c_k3_collapse_gpu` (from
`libk3_gpu.so`) against `c_kleene_quantize_avx2` on the same 1M-neuron field.

`libk3_gpu.so` **exists** (997,744 B, v17 tree), so the identity comparison is checkable. The
**177M/s is a performance assertion and is not re-measured here** — on this platform docstring
speedups have been wrong in both directions (a claimed 172.6x measured at 69-81x), so an unmeasured
throughput number stays unverified until it is run on this host.

## Net

Of the v18 claims examined so far: `swarm-k3` **verified** (0 diffs, 4030 values × 3 epsilons),
`swarm-council` **verified faithful** (11/11 rows incl. the arms its own gate never reached) though the
rule it implements is degenerate, `swarm-ssm` **rule verified** with the two caveats above,
`swarm-instella` and `swarm-gpu-neural` **checkable but not yet run**. The reference artifact is real,
so none of these is unfalsifiable any more.
