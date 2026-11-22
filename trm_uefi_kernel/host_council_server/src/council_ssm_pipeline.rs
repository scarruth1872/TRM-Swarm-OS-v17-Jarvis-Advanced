//! Council SSM + K3 decision pipeline.
//!
//! WHY THIS EXISTS
//! Before this module the council's decision path was:
//!     prompt a 1.5B model -> regex "VOTE:" out of its text -> trit
//! Mamba-3 and the K3 engine existed in the kernel, but the council used K3
//! only as an offline fallback when the model was unreachable, and never
//! touched Mamba-3 at all. `/council/status` advertised `mamba3_ssm: ACTIVE`
//! as a static JSON string. That is the missing pipeline this file builds.
//!
//! THE PIPELINE, per agent
//!   1. text      = proposal + that agent's own retrieved evidence slice
//!   2. sequence  = 96 token vectors, 128-dim each (4 salted 32-dim hashes)
//!   3. recurrence= Mamba3StateSpaceScanner::step over the sequence
//!                  h_t = exp(dt*A) h_{t-1} + dt*B x_t ,  y = C h_t
//!   4. reduction = KleeneTernaryEngine::state_reduction(y_T) -> 128 trits
//!   5. decision  = mean trit over the 128 dimensions, banded at +/-0.15
//!   6. fusion    = the SSM trit and the model trit go through K3 consensus
//!
//! WHY MEAN AND NOT bypass_decision()
//! `bypass_decision()` is D = min(psi) = Kleene AND over every trit. Over 128
//! dimensions one FALSE dimension forces the whole decision to FALSE, so it
//! saturates. It is correct for a short state vector and wrong for a 128-wide
//! reduction. The mean trit is the same algebra without the saturation: it is
//! the expected truth value of the state. The band is a calibration constant,
//! not a law, and it is named as such.
//!
//! WHAT THE NUMBERS MEAN
//! The SSM weights are deterministic initialised (a_diag = -0.5*(i+1)), NOT
//! trained. So this is a fixed linear projection of the token sequence, not
//! learned reasoning. Its value is that it is a SECOND, independent read of the
//! same text that costs microseconds and cannot be talked into anything by the
//! proposal's own phrasing. Whether it carries signal is an empirical question:
//! every proposal reports `agreement` so the rate can be measured rather than
//! assumed.

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use trm_uefi_kernel::k3_logic::{KleeneTernaryEngine, UNKNOWN};

/// Re-exported so fusion policy lives in one place: UNKNOWN means "no decision".
pub const NO_DECISION: i8 = UNKNOWN;
use trm_uefi_kernel::k3_logic::task_hash_vector;
use trm_uefi_kernel::mamba3_turbovec_ssm::{Mamba3StateSpaceScanner, D_MODEL, D_STATE};

/// Sequence length fed through the recurrence. Longer sequences cost linear
/// time and the tail of a retrieved chunk is rarely the decisive clause.
pub const MAX_TOKENS: usize = 96;

/// Band on the mean trit. |mean| below this is UNKNOWN, not a decision.
///
/// CALIBRATED, not guessed. Measured over 45 texts (5 topics + 40 document
/// chunks) the mean trit has sd = 0.082 and centre = -0.0415. The centre is
/// NEGATIVE because the fixed A/B/C projection is asymmetric: it leans FALSE
/// regardless of content. Two consequences, both measured:
///   band 0.15 -> only 3/45 texts decide (7%)  : the SSM was effectively silent
///   band 0.05 -> 24/45 texts decide (53%)     : it participates
/// The 0.05 band is chosen so the read-out is not silent, NOT because it is
/// accurate. The bias means a FALSE reading is a weak signal and a TRUE
/// reading is a strong one -- see ssm_decision() and the evidence file.
pub const MEAN_BAND: f32 = 0.05;

/// Token vectors are 4 salted 32-dim hashes concatenated to fill D_MODEL.
const SALTS: u8 = 4;

/// Fitted read-out weights. Absent means the mean-trit fallback is used.
const READOUT_PATH: &str = "data/ssm_readout.json";

/// Embedding-fed features: how many chunks of the text to run through the
/// recurrence, and how long each chunk may be. MEASURED: the embedding server
/// returns HTTP 500 for batches much larger than this, and 6 x 220 chars stays
/// well inside its context.
const EMBED_CHUNKS: usize = 6;
const EMBED_CHARS: usize = 220;

#[derive(Debug, Clone, Copy)]
pub struct SsmDecision {
    pub trit: i8,
    pub confidence: f32,
    /// Fraction of the 128 reduced trits that were UNKNOWN.
    pub ambiguity: f32,
    /// Mean trit before banding, kept so the raw signal is auditable.
    pub mean_trit: f32,
    pub steps: u64,
    /// "trained" when a fitted read-out produced this trit, "mean-trit" when the
    /// uncalibrated fallback did. The distinction matters: a trained read-out has
    /// a measured agreement rate, the fallback has 34% = chance.
    pub readout: &'static str,
    /// "embedding" when the state came from nomic vectors, "hash" when the
    /// embedding server was unavailable. Weights fitted on one feature type are
    /// invalid for the other, so this must travel with the decision.
    pub feature: &'static str,
}

/// Deterministic 128-dim vector for one token.
fn token_vector(token: &str) -> [f32; D_MODEL] {
    let mut v = [0.0f32; D_MODEL];
    for s in 0..SALTS {
        let mut buf = Vec::with_capacity(token.len() + 1);
        buf.push(s);
        buf.extend_from_slice(token.as_bytes());
        let h = task_hash_vector(&buf);
        let base = (s as usize) * 32;
        for (i, x) in h.iter().enumerate() {
            v[base + i] = *x;
        }
    }
    // Unit-normalise so the recurrence input scale does not depend on text length.
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-6 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    v
}

/// Tokens: lowercase alphanumeric runs. Cheap, deterministic, no allocation churn.
fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() {
            for lc in c.to_lowercase() {
                cur.push(lc);
            }
        } else if !cur.is_empty() {
            out.push(core::mem::take(&mut cur));
            if out.len() >= MAX_TOKENS {
                return out;
            }
        }
    }
    if !cur.is_empty() && out.len() < MAX_TOKENS {
        out.push(cur);
    }
    out
}

/// Run the SSM over the text and return the RMS-normalised final state vector.
///
/// Training and serving MUST share this function: if the read-out is fitted on a
/// different projection than the one used at inference, the weights mean nothing.
pub fn ssm_state(text: &str) -> ([f32; D_MODEL], u64) {
    let toks = tokens(text);
    if toks.is_empty() {
        return ([0.0f32; D_MODEL], 0);
    }
    let gains = readout().as_ref().and_then(|r| match (&r.bgain, &r.cgain) {
        (Some(b), Some(c)) => Some((b.as_ref(), c.as_ref())),
        _ => None,
    });
    let mut scanner = Mamba3StateSpaceScanner::new();
    let mut y = [0.0f32; D_MODEL];
    for t in &toks {
        let x = token_vector(t);
        // dt = 0.01 matches Mamba3TurboVecEngine::process_step's fixed step.
        y = match gains {
            Some((b, c)) => scanner.step_with_gains(&x, 0.01, b, c),
            None => scanner.step(&x, 0.01),
        };
    }
    let steps = scanner.step_count;

    // Scale fix: the raw recurrence output lands around 0.15-0.2 while tau is
    // 0.35, so without this every trit is UNKNOWN. Normalising to RMS 1 makes
    // the read-out scale-free and leaves the K3 bands at their defaults.
    let rms = (y.iter().map(|v| v * v).sum::<f32>() / D_MODEL as f32).sqrt();
    if rms > 1e-6 {
        for v in y.iter_mut() {
            *v /= rms;
        }
    }
    (y, steps)
}

/// Split text into up to EMBED_CHUNKS pieces of at most EMBED_CHARS on word
/// boundaries. These chunks are what the recurrence actually scans when fed
/// embeddings.
fn split_chunks(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if !cur.is_empty() && cur.len() + 1 + word.len() > EMBED_CHARS {
            out.push(core::mem::take(&mut cur));
            if out.len() >= EMBED_CHUNKS {
                return out;
            }
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() && out.len() < EMBED_CHUNKS {
        out.push(cur);
    }
    out
}

/// 768 -> 128 by the mean of 6 consecutive components. A FIXED projection: not
/// trained, but unlike token hashes the input it folds carries meaning.
fn project_768(v: &[f32]) -> Option<[f32; D_MODEL]> {
    if v.len() < D_MODEL * 6 {
        return None;
    }
    let mut out = [0.0f32; D_MODEL];
    for i in 0..D_MODEL {
        let mut acc = 0.0f32;
        for k in 0..6 {
            acc += v[i * 6 + k];
        }
        out[i] = acc / 6.0;
    }
    let norm = out.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-6 {
        for x in out.iter_mut() {
            *x /= norm;
        }
    }
    Some(out)
}

/// Embedding-fed state. None when the embedding server is unavailable, so the
/// caller can fall back to hashes instead of fabricating a state.
pub fn ssm_state_embed(text: &str) -> Option<([f32; D_MODEL], u64)> {
    let chunks = split_chunks(text);
    if chunks.is_empty() {
        return None;
    }
    let vecs = crate::local_inference::embed_batch(&chunks)?;
    let mut scanner = Mamba3StateSpaceScanner::new();
    let mut y = [0.0f32; D_MODEL];
    let mut n = 0u64;
    for v in vecs.iter() {
        let x = project_768(v)?;
        y = scanner.step(&x, 0.01);
        n += 1;
    }
    let rms = (y.iter().map(|v| v * v).sum::<f32>() / D_MODEL as f32).sqrt();
    if rms > 1e-6 {
        for v in y.iter_mut() {
            *v /= rms;
        }
    }
    Some((y, n))
}

/// State with the configured feature type. "hash" forces token hashes; default is
/// embeddings with a hash fallback, so an embedding outage degrades the read-out
/// instead of breaking the council.
pub fn ssm_state_auto(text: &str) -> ([f32; D_MODEL], u64, &'static str) {
    // DEFAULT IS HASH. MEASURED: embedding-fed states gave a fitted read-out
    // 0.474 leave-one-topic-out against a 0.455 baseline, and their row-mean has
    // sd 0.0031 around +0.018 -- so the banded mean read-out fires on 0/308
    // samples. Embeddings cost one HTTP call per agent and buy nothing. They stay
    // available for experiments via COUNCIL_SSM_FEATURE=embedding.
    let want_embed = std::env::var("COUNCIL_SSM_FEATURE")
        .map(|v| v == "embedding")
        .unwrap_or(false);
    if want_embed {
        if let Some((y, n)) = ssm_state_embed(text) {
            return (y, n, "embedding");
        }
    }
    let (y, n) = ssm_state(text);
    (y, n, "hash")
}

/// Fitted read-out: w . y + b, banded. Loaded from data/ssm_readout.json.
struct Readout {
    w: Vec<f32>,
    b: f32,
    band: f32,
    /// LEARNED input/output gains (2026-11-10). Optional so an older read-out file still
    /// loads and the constant-gain path stays the default. Without these, `b_bar` is
    /// `delta` and `c_val` is `1/4` for every (state, channel) pair -- see
    /// Mamba3StateSpaceScanner::step_with_gains for why that was the measured defect.
    bgain: Option<Box<[[f32; D_MODEL]; D_STATE]>>,
    cgain: Option<Box<[[f32; D_MODEL]; D_STATE]>>,
    /// "ssm" (recurrent state) or "linear" (mean-pooled state, NO recurrence).
    /// MEASURED (G137): on identical embedding features the recurrence returned a constant
    /// (AUC 0.500) while the linear head held signal. The artifact declares which one it is,
    /// so a linear fit is never served through the recurrence it was not fitted on.
    model: String,
    /// Threshold for the linear model: proj >= threshold means BLOCK.
    threshold: f32,
    /// Objects the fit has seen, lowercased. The framing arm judges REQUEST SHAPE over a known asset vocabulary; on
    /// an object outside this list it ABSTAINS rather than allowing, because the object-holdout measurement (0.527,
    /// G144) says it cannot judge an unseen object. Abstaining is the only failure direction a veto may take.
    objects: Vec<String>,
    /// Asset inventory read from the deployment's own configs: (phrase, class). This is a RULE LAYER, not weights -
    /// it answers "is this thing load-bearing" for objects the head never saw, and any decision it makes is labelled
    /// `inventory-rule` so a config lookup can never be mistaken for a learned judgement.
    inventory: Vec<(String, String)>,
    /// Which feature type these weights were fitted on. Weights fitted on token
    /// hashes are meaningless applied to embedding states and vice versa, so a
    /// mismatch refuses the weights instead of producing a confident wrong trit.
    feature: String,
}

static READOUT: std::sync::OnceLock<Option<Readout>> = std::sync::OnceLock::new();

/// Every place a read-out artifact may legitimately live, in precedence order: CWD first (so an operator can
/// override), then beside the executable, then the crate's own `data/` as seen from `target/release`.
fn readout_candidates(name: &str) -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;
    let mut out: Vec<PathBuf> = Vec::new();
    out.push(PathBuf::from(name));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join(name));
            // target/release -> crate root -> crate data/
            if let Some(crate_dir) = dir.parent().and_then(|p| p.parent()) {
                out.push(crate_dir.join("data").join(
                    std::path::Path::new(name).file_name().unwrap_or_default()));
                out.push(crate_dir.join(name));
                // repo root (crate is trm_uefi_kernel/host_council_server)
                if let Some(repo) = crate_dir.parent().and_then(|p| p.parent()) {
                    out.push(repo.join(name));
                }
            }
        }
    }
    out.dedup();
    out
}

/// Parse a D_STATE x D_MODEL gain matrix from JSON: [[f32; D_MODEL]; D_STATE].
/// Returns None unless the shape is exactly right -- a short array silently zero-padded
/// would be a wrong model that looks trained.
fn parse_gain(v: Option<&serde_json::Value>) -> Option<Box<[[f32; D_MODEL]; D_STATE]>> {
    let rows = v?.as_array()?;
    if rows.len() != D_STATE {
        return None;
    }
    let mut out = Box::new([[0.0f32; D_MODEL]; D_STATE]);
    for (s, row) in rows.iter().enumerate() {
        let cols = row.as_array()?;
        if cols.len() != D_MODEL {
            return None;
        }
        for (m, x) in cols.iter().enumerate() {
            out[s][m] = x.as_f64()? as f32;
        }
    }
    Some(out)
}

fn readout() -> &'static Option<Readout> {
   READOUT.get_or_init(|| {
       // The scope gate lives in its own artifact and, when deployed, takes precedence: it is the arm that was
       // measured beating its baselines (G139), while the general read-out was refused twice (G137).
       //
       // MEASURED 2026-11-22: this was read as a bare relative path, so a kernel started with the repo root as
       // CWD read repo `data/` (which holds only refused candidates) while the DEPLOYED gate sits in the crate's
       // own `data/`. Result: a passing gate (AUC 0.926) served as UNAVAILABLE because the file was never opened.
       // Candidates are now anchored on the executable as well as CWD, the same fix the journal manifest needed.
       let txt = readout_candidates("scope_gate.json")
           .into_iter()
           .chain(readout_candidates(READOUT_PATH))
           .find_map(|p| std::fs::read_to_string(&p).ok())?;
        let v: serde_json::Value = serde_json::from_str(&txt).ok()?;
        // A fitted read-out is only used when the fitting run says so. MEASURED:
        // the first fit reached 0.458 leave-one-topic-out against a 0.455
        // always-majority baseline, so it is written with deploy=false. Refusing
        // it here keeps a failed fit from silently changing council behaviour.
        if !v.get("deploy").and_then(|d| d.as_bool()).unwrap_or(false) {
            return None;
        }
        let w: Vec<f32> = v.get("w")?.as_array()?
            .iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
        if w.len() != D_MODEL { return None; }
        // Gains are all-or-nothing: a read-out fitted WITH learnable B/C must not be
        // served with one of them missing, because the fitting run's other half would be
        // meaningless applied to constant gains.
        let bgain = parse_gain(v.get("b_gain"));
        let cgain = parse_gain(v.get("c_gain"));
        let (bgain, cgain) = match (bgain, cgain) {
            (Some(b), Some(c)) => (Some(b), Some(c)),
            (None, None) => (None, None),
            _ => return None,
        };
        Some(Readout {
            w,
            b: v.get("b").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
            band: v.get("band").and_then(|x| x.as_f64()).unwrap_or(0.25) as f32,
            model: v.get("model").and_then(|m| m.as_str()).unwrap_or("ssm").to_string(),
            objects: v.get("objects").and_then(|o| o.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).map(|x| x.to_lowercase()).collect())
                .unwrap_or_default(),
            inventory: v.get("inventory").and_then(|o| o.as_array())
                .map(|a| a.iter().filter_map(|x| {
                    Some((x.get("phrase")?.as_str()?.to_lowercase(),
                          x.get("class")?.as_str()?.to_lowercase()))
                }).collect())
                .unwrap_or_default(),
            threshold: v.get("threshold").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
            feature: v.get("feature").and_then(|f| f.as_str()).unwrap_or("hash").to_string(),
            bgain,
            cgain,
        })
    })
}

/// The objects the served gate knows, ranked by how much of the request's vocabulary they share.
///
/// An abstain that says only "I do not know" is a dead end for the caller. This returns the NEAREST known assets so the
/// abstain becomes a question with candidates attached: the caller can confirm the object, or escalate to council
/// deliberation with a concrete shortlist instead of a bare refusal.
pub fn object_candidates(text: &str, k: usize) -> Vec<(String, f32)> {
    let r = match readout().as_ref() {
        Some(r) => r,
        None => return Vec::new(),
    };
    if r.objects.is_empty() {
        return Vec::new();
    }
    let norm = |s: &str| -> Vec<String> {
        s.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .map(|w| w.to_string())
            .collect()
    };
    let q = norm(text);
    if q.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(String, f32)> = r
        .objects
        .iter()
        .map(|o| {
            let ot = norm(o);
            let hit = ot.iter().filter(|w| q.contains(w)).count() as f32;
            let score = hit / (ot.len().max(1) as f32);
            (o.clone(), score)
        })
        .filter(|(_, sc)| *sc > 0.0)
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    scored
}

/// Mean-pooled embedding state: the mean of each chunk's 768->128 projection, L2-normalised, and NO recurrence.
///
/// This exists because the recurrence was measured destroying signal it was given (G134/G137): on identical rows and
/// features the SSM read-out returned a constant while a 128-parameter linear head on THIS vector held AUC. The
/// Python trainer builds the same vector (`pooled_matrix`), so the artifact transfers without a train/serve skew.
pub fn ssm_state_pooled(text: &str) -> Option<([f32; D_MODEL], u64)> {
    let chunks = split_chunks(text);
    if chunks.is_empty() {
        return None;
    }
    let vecs = crate::local_inference::embed_batch(&chunks)?;
    if vecs.is_empty() {
        return None;
    }
    let mut acc = [0.0f32; D_MODEL];
    let mut n = 0u64;
    for v in vecs.iter() {
        if let Some(x) = project_768(v) {
            for i in 0..D_MODEL {
                acc[i] += x[i];
            }
            n += 1;
        }
    }
    if n == 0 {
        return None;
    }
    let inv = 1.0 / n as f32;
    for v in acc.iter_mut() {
        *v *= inv;
    }
    let norm = acc.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 1e-6 {
        for v in acc.iter_mut() {
            *v /= norm;
        }
    }
    Some((acc, n))
}

/// Run the SSM over the text and reduce the final state to a ternary decision.
///
/// Two read-outs, in order of preference:
///   1. TRAINED  w . y + b, banded. Only present when data/ssm_readout.json
///               exists. Its agreement with the model is a measured number, and
///               ssm_decision reports readout="trained" so callers can tell.
///   2. MEAN     mean of the 128 K3 trits, banded at MEAN_BAND. Uncalibrated
///               fallback: measured 34% agreement = chance, biased FALSE.
pub fn ssm_decision(text: &str) -> SsmDecision {
    // A LINEAR read-out is served through the pooled vector it was fitted on, never through the recurrence.
    // Returning here keeps one code path per model type instead of a flag inside the recurrent path.
    if let Some(r) = readout().as_ref().filter(|r| r.model == "linear" && r.feature == "embedding") {
        // Unknown object -> ABSTAIN, never ALLOW. Measured: the head cannot judge an object it has not seen
        // (object-holdout AUC 0.527, G144), so an ALLOW here would be a confident guess in the one direction a veto
        // must not guess.
        if !r.objects.is_empty() {
            let lt = text.to_lowercase();
            if !r.objects.iter().any(|o| lt.contains(o)) {
                // The head does not know this object. The inventory says what it IS, and that is enough to decide a
                // veto in the safe direction: a declared PRODUCTION asset is blocked, a declared ephemeral one is
                // allowed, and anything in neither is abstained. Reported as `inventory-rule`, never as the head.
                if let Some((phrase, class)) = r.inventory.iter().find(|(p, _)| lt.contains(p.as_str())) {
                    let block = class == "prod";
                    return SsmDecision {
                        trit: if block { -1i8 } else { 1i8 },
                        confidence: 0.5,
                        ambiguity: 0.0,
                        mean_trit: if block { 1.0 } else { -1.0 },
                        steps: 1,
                        readout: "inventory-rule",
                        feature: "config",
                    };
                }
                return SsmDecision {
                    trit: UNKNOWN,
                    confidence: 0.0,
                    ambiguity: 1.0,
                    mean_trit: 0.0,
                    steps: 0,
                    readout: "abstain-unknown-object",
                    feature: "embedding",
                };
            }
        }
        if let Some((y, n)) = ssm_state_pooled(text) {
            let proj: f32 = r.w.iter().zip(y.iter()).map(|(a, b)| a * b).sum::<f32>() + r.b;
            let trit = if proj >= r.threshold { -1i8 } else { 1i8 };
            let psi = KleeneTernaryEngine::new().state_reduction(&y, None, None);
            let unknown = psi.iter().filter(|&&t| t == UNKNOWN).count() as f32 / psi.len().max(1) as f32;
            return SsmDecision {
                trit,
                confidence: (proj - r.threshold).abs().min(1.0),
                ambiguity: unknown,
                mean_trit: proj,
                steps: n,
                readout: "linear-pooled",
                feature: "embedding",
            };
        }
    }

    let (y, steps, feature) = ssm_state_auto(text);
    if steps == 0 {
        return SsmDecision {
            trit: UNKNOWN, confidence: 0.0, ambiguity: 1.0,
            mean_trit: 0.0, steps: 0, readout: "mean-trit", feature,
        };
    }
    let yv = y.to_vec();

    // Weights only apply to the feature type they were fitted on.
    if let Some(r) = readout().as_ref().filter(|r| r.feature == feature) {
        let proj: f32 = r.w.iter().zip(yv.iter()).map(|(a, b)| a * b).sum::<f32>() + r.b;
        let trit = if proj > r.band { 1i8 } else if proj < -r.band { -1i8 } else { UNKNOWN };
        let psi = KleeneTernaryEngine::new().state_reduction(&yv, None, None);
        let unknown = psi.iter().filter(|&&t| t == UNKNOWN).count() as f32 / psi.len().max(1) as f32;
        return SsmDecision {
            trit,
            confidence: proj.abs().min(1.0),
            ambiguity: unknown,
            mean_trit: proj,
            steps,
            readout: "trained",
            feature,
        };
    }

    let psi = KleeneTernaryEngine::new().state_reduction(&yv, None, None);
    let n = psi.len().max(1) as f32;
    let sum: i32 = psi.iter().map(|&t| t as i32).sum();
    let mean = sum as f32 / n;
    let unknown = psi.iter().filter(|&&t| t == UNKNOWN).count() as f32;

    let trit = if mean > MEAN_BAND { 1i8 } else if mean < -MEAN_BAND { -1i8 } else { UNKNOWN };

    SsmDecision {
        trit,
        confidence: mean.abs().min(1.0),
        ambiguity: unknown / n,
        mean_trit: mean,
        steps,
        readout: "mean-trit",
        feature,
    }
}

/// Fuse the model's read and the SSM's read.
///
/// MEASURED, and the reason `gate` defaults to false: across 70 agent-topic
/// pairs the SSM trit matched the model's trit 24 times = 34%, which is chance
/// for three classes. When the SSM did decide, it said FALSE 45 times and TRUE
/// once. The untrained projection is a biased coin, not a second opinion.
///
///   gate = false (default) : the model decides; the SSM is recorded and its
///                            dissent is flagged. No consensus is destroyed.
///   gate = true            : Kleene-3 fusion as originally designed, where
///                            disagreement yields UNKNOWN. Kept because it is
///                            the honest ternary semantics and it must stay
///                            measurable -- but it dropped dark mode from
///                            13/0/1 to 4/9/1 and QUIC from 14/0/0 to 7/7/0.
///
/// Either way the pair (llm, ssm) is written to the vote record, so the
/// agreement rate can be re-measured rather than trusted from this comment.
pub fn fuse(llm: Option<i8>, ssm: SsmDecision, gate: bool) -> (i8, f32, &'static str) {
    let agrees = match llm {
        Some(t) => ssm.trit != UNKNOWN && t == ssm.trit,
        None => false,
    };
    let dissents = match llm {
        Some(t) => ssm.trit != UNKNOWN && t != ssm.trit,
        None => false,
    };

    if gate {
        return match llm {
            Some(t) if agrees => (t, 0.90, "llm+ssm agree"),
            Some(t) if ssm.trit == UNKNOWN => (t, 0.75, "llm only (ssm undecided)"),
            Some(_) => (UNKNOWN, 0.35, "llm/ssm disagree -> UNKNOWN"),
            None if ssm.trit != UNKNOWN => (ssm.trit, ssm.confidence * 0.7, "ssm only (no model)"),
            None => (UNKNOWN, 0.20, "no signal"),
        };
    }

    match llm {
        Some(t) if agrees => (t, 0.90, "llm (ssm agrees)"),
        Some(t) if dissents => (t, 0.60, "llm (ssm dissents)"),
        Some(t) => (t, 0.80, "llm (ssm undecided)"),
        None if ssm.trit != UNKNOWN => (ssm.trit, ssm.confidence * 0.7, "ssm only (no model)"),
        None => (UNKNOWN, 0.20, "no signal"),
    }
}

/// Human-readable one-liner for the deliberation comment.
pub fn describe(ssm: SsmDecision) -> String {
    format!(
        "ssm trit={} proj={:+.4} amb={:.2} steps={} readout={} feature={}",
        ssm.trit, ssm.mean_trit, ssm.ambiguity, ssm.steps, ssm.readout, ssm.feature
    )
}
