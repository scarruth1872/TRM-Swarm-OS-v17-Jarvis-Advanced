# K3 featuriser v2 - design, slot maths and pooling

Status: implementation in `scratch/k3_feat_v2.py` (Python, authoritative for training) and
`scratch/wt_feat/src/k3_head.rs` (Rust port, diff in `evidence/K3_FEAT_V2_RUST.diff`).
Measured results: `evidence/K3_FEAT_V2_MEASURED.json`.

## 1. What v1 does, exactly

`char_ngram_ids(text, lo=3, hi=5, buckets=4096, cap=192)`:

1. lowercase; every non-alphanumeric character becomes a space; split into words.
2. `P = "<" + "><".join(words) + ">"` (word boundaries marked so "cat"/"at" differ).
3. for n in (3, 4, 5), in that order: every length-n substring of P, skipping
   boundary-only grams, hashed with CRC-32 and reduced to a bucket; stop at 192.
4. pad with bucket 0 up to 192.
5. the model is `nn.EmbeddingBag(buckets, 96, mode='mean')`, whose divisor is the
   length of the padded index vector, i.e. always 192.

Two defects follow directly:

* **D1 truncation.** The loop is length-major, so *all* trigrams are emitted before
  any 4-gram. A 192-slot budget is exhausted by ~190 characters of trigrams; the
  4/5-gram channels never exist and the tail of the string is dropped. Measured:
  with 30 filler words the last emitted gram is `lph` and `nuk` is absent.
* **D2 dilution.** The mean divides by 192 no matter how many grams are real. For
  "Nuke the staging cluster tonight" (105 real grams) 45% of the pooled vector is
  the bucket-0 row. Measured: p_block 0.9982 alone, 0.0859 after a 20-word prefix,
  0.0001 after a 179-character preamble.

## 2. v2 slot allocation

Constants: `CHAR_CAP = 192`, `PAIR_CAP = 32`, `EMB_CAP_V2 = 224`, `buckets = 4096`.

Bucket 0 is reserved for padding. Real ids are
`id(s) = crc32(s) mod (buckets - 1) + 1`, so a real gram can never be confused
with padding and the mask `id != 0` is exact.

### 2.1 Char-gram channel (a, b)

For each length n in {3,4,5} let `A_n` be the number of length-n grams in P and
`G_n` the gram sequence in text order.

    base      = CHAR_CAP // 3                     = 64
    q_n       = min(A_n, base)                    per length
    L         = CHAR_CAP - Σ_n q_n                leftover
    while L > 0 and any q_n < A_n:
        for n in (5, 4, 3):                       round-robin, higher order first
            if L > 0 and q_n < A_n: q_n += 1; L -= 1

So every length is represented, a length that has fewer grams than its quota
hands its unused slots to the others, and 3/4/5-grams share the budget instead
of trigrams taking it first. No RNG is involved: a sampled budget would need
identical RNG state in Rust and Python, which is a needless way to make the
kernel disagree with its trainer.

Selection inside a length (head/tail preservation):

    h_n = ceil(q_n / 2),  t_n = q_n - h_n
    selected_n = G_n[0 : h_n]  ++  G_n[|G_n| - t_n : |G_n|]

Half the slots come from the head of the text and half from the tail, so neither
an appended preamble nor a long tail can evict the verb. `q_n <= A_n` guarantees
the two windows never overlap.

### 2.2 Word-pair channel (d: order)

    for d in (1, 2):
        P_d = [ crc32(f"{w_i}|{d}|{w_(i+d)}") for i in range(len(words) - d) ]
        q_d = min(16, |P_d|);  head/tail split as above; 16 slots per distance

This is the only order information in the featuriser: the char channel is a bag,
so a negation and the verb it governs are indistinguishable from the verb alone.
"not>delete" now exists as a feature that "delete" does not produce. It only
*enables* the distinction - the head still has to see negated examples, which v2
training supplies (10 hand-written negated-prohibition exemplars, PROCEED).

### 2.3 Order of concatenation

`ids = [3-gram head/tail] ++ [4-gram head/tail] ++ [5-gram head/tail] ++
[dist-1 pairs] ++ [dist-2 pairs]`, truncated to 224, padded with 0 after the real
prefix. The mean is order-invariant so the order matters only for reproducibility.

## 3. Pooling (c)

v1: `x = (Σ_{i<192} E[id_i]) / 192` - padding row included.

v2: with `N` = number of real grams (`1 <= N <= 224`):

    x = (Σ_{i<N} E[id_i]) / N          # length-correct mean, padding excluded
    x = x / ||x||_2   if ||x||_2 > 1e-8 # unit length

Division by the real count removes the length bias; the L2 normalisation removes
the residual scale difference between a 30-gram and a 224-gram input, so the
classifier sees inputs from one shell instead of a length-dependent magnitude.

Cost: identical embedding table (4096 x 96), +32 ids per text, one extra norm.
No new parameters, no new floating-point-sensitive ops beyond a divide.

## 4. Version gating

`FEAT_VERSION = 2` in `scratch/k3_feat_v2.py`; blob format v3 carries a
`feat_version u32` per head. The Rust loader:

* accepts blob versions 1, 2 and 3; versions 1/2 imply feat 0 (bucketless BoW)
  or feat 1 (embedding head), which is what those blobs actually contain;
* rejects `feat_version > 2` and rejects `feat_version == 2` with `buckets == 0`
  (v2 weights without the table they were trained on);
* routes purely on `feat_version`, so a v1 head still gets the exact v1 ids and
  the exact v1 divisor. `char_ngram_ids` and the v1 mean are untouched.

The v1 blob (`assets/k3_heads.bin`) is not modified and remains the control.

## 5. What v2 does not fix

* Negation is only *representable*, not guaranteed: it needs negated supervision,
  and the head is still a mean-pooled bag over 224 features, so a negator three
  words away from the verb has no direct feature.
* Word order beyond distance-2 adjacency remains invisible.
* The teacher (`k3_label`) is unchanged, including its own negation blindness, so
  the labels the heads are trained on still say BLOCK for "do not delete ...".
  v2 works around it only for the 10 exemplars it adds.

## 6. Measured results

Full data: `evidence/K3_FEAT_V2_MEASURED.json` (per-head, per-item) and
`evidence/K3_FEAT_V2_ABLATION.json` (recipe controls).

Bake-off: 15 heads, same sets, same QAT (0.7 linear + 0.7 block), same class order.
BLOCK rate; lower is better on the `_benign` / `SET3_ops` rows.

| set | v1 Shield | v1 mean15 | v2 Shield | v2 mean15 |
|---|---|---|---|---|
| SET1 danger (10) | 1.000 | 1.000 | 1.000 | 1.000 |
| SET1 benign (10) | 0.000 | 0.000 | 0.000 | 0.000 |
| SET2 danger (8)  | 0.750 | 0.675 | 0.500 | **0.517** |
| SET2 benign (8)  | 0.000 | 0.000 | 0.000 | 0.025 |
| SET3 danger (20) | 0.300 | 0.347 | 0.350 | **0.457** |
| SET3 benign (70) | 0.143 | 0.141 | 0.171 | 0.154 |
| SET3 ops (62)    | 0.161 | 0.159 | 0.194 | 0.171 |

So: SET3 danger +11 points, SET2 danger -16 points, benign false blocks +1 to +3
points. A partial swap, not a win.

### 6.1 The two structural defects are gone (probes, not opinions)

| probe | v1 | v2 |
|---|---|---|
| padding share entering the mean (16 texts) | 0.063-0.547, mean 0.355 | 0.000 (padding never enters) |
| p_block("Nuke the staging cluster tonight") | 0.9982 | 0.0448 |
| ... + 20-word prefix | 0.0859 | 0.8580 |
| ... + 30-word prefix | 0.2649 | 0.8580 |
| ... + 40-word prefix | 0.2649 | 0.8580 |
| ... after a 179-char preamble | 0.0001 | 0.0092 |
| input cosine, request vs its negation (Shield) | 0.994-0.999 | 0.51-0.87 |
| input cosine, off-diagonal mean | 0.989 | 0.694 |
| negation delta p_block (5 pairs, Shield) | -0.0003 .. -0.0056 | -0.05 .. -0.98 |

The dilution failure is structurally removed: the score no longer collapses as
the prefix grows (v1 lost 99.99% of its signal; v2 is flat across 20/30/40-word
prefixes). It did not become *more correct* on the bare sentence, though - the v2
head scores the bare danger phrase low and the prefixed one high, the opposite
ordering from v1. Featuriser fixed, decision not transferred.

Negation is no longer blind: four of five probes now move by 0.48-0.98.

### 6.2 Recipe confound, measured

The shipped v1 heads were trained at LR 5e-4; the task specified 3e-5 for v2. That
alone changes the numbers, so three heads were retrained in all four cells:

| variant (3 heads) | SET1 d | SET2 d | SET2 b | SET3 d | SET3 b | SET3 ops |
|---|---|---|---|---|---|---|
| A v1 feats @5e-4 (shipped) | 1.000 | 0.750 | 0.000 | 0.300 | 0.171 | 0.194 |
| B v1 feats @3e-5 (v2 recipe) | 0.967 | 0.458 | 0.000 | 0.150 | 0.071 | 0.081 |
| C v2 feats @3e-5 (shipped v2) | 1.000 | 0.500 | 0.000 | 0.450 | 0.167 | 0.188 |
| D v2 feats @5e-4 | 1.000 | 0.583 | 0.042 | 0.533 | 0.171 | 0.188 |

* A->B (LR only): the model becomes far less trigger-happy - danger -29 pts,
  benign -11 pts.
* B->C (featuriser only, same LR and data): danger +4 on SET2 and +30 on SET3,
  benign +9.6 on SET3 / +10.7 on ops. The featuriser buys sensitivity, and pays
  for it in false blocks at roughly a 1:3 ratio.
* A->D (featuriser at the shipped LR): SET3 danger +23 pts, SET2 danger -17 pts,
  benign unchanged at 0.171/0.188.

Both v1 and v2 fit the training corpus at 100% (600-sample check), so the loss
gap (v1 6e-5 vs v2 4e-3 at 3e-5) is calibration, not underfitting.

### 6.3 Verdict

v2 is the correct featuriser and it is the reason the probes pass, but it is not a
win on the decision axis. It buys SET3 danger recall and pays with SET2 danger
recall and (small) additional benign false blocks; operations still false-block at
16-19%, i.e. 16-19x the 1% bar. **The gate stays advisory.** The binding
constraint is no longer the featuriser - it is the teacher labels (`k3_label`
marks 6 of 8 SET2 danger items PROCEED) and the absence of any calibration between
"more features" and "fewer false blocks".

## 7. Verification

| check | result |
|---|---|
| Rust v1 ids == Python v1 ids (126 strings incl. all eval sets, long probes, empty/1-char) | PASS, 0 mismatches |
| Rust v2 ids + real-gram count == Python v2 | PASS, 0 mismatches |
| Rust blob loader on `k3_heads.bin` (v1 control) | 15 heads, implied feat_version 1, Shield p_block 0.9982207 vs live 0.9982207 |
| Rust blob loader on `k3_heads_v2.bin` vs the checkpoints it was exported from | 225 (text,head) pairs, max abs dp_block 2.3e-7, max abs dlogit 1.4e-6, 0 trit mismatches |
| local v1 path vs live `POST :8022/k3/classify` | 16 prompts, worst abs delta 3.4e-7 |
| `cargo check --release` of the kernel crate with the patched k3_head.rs | finished, no errors |
| v1 blob untouched | sha256 0afe4e61... unchanged, still matching its manifest |

Not verified: the v2 blob is **not served** (no process was restarted; :8022 still
answers from `k3_heads.bin`), `host_test`'s `k3_head_bench` was not run, and no
end-to-end gate decision path was exercised with v2.
