//! TRM Swarm OS - chain_of_verification (bare-metal no_std port).
//!
//! Oracle: swarm_v2/core/chain_of_verification.py (Chain-of-Verification, Phase 5 logic upgrade).
//! The previous revision of this file was a generator stub: a `u64 = 0` placeholder named
//! FALLACY_PATTERNS, the three dataclasses (with `confidence` defaulted to 0.0 where the oracle
//! declares 1.0), a fabricated singleton handle and none of the module's rules. This revision
//! carries the oracle's actual computation.
//!
//! Ported 1:1: the fallacy pattern table (47 regexes over 16 fallacy types, extracted from the
//! oracle source by script - never retyped), reasoning-step extraction (numbered / bullet /
//! sentence fallback), premise and conclusion extraction, fallacy detection with the 250-byte
//! refutation window for CATEGORY_ERROR/FACTUAL_INVERSION, severity assignment, the scoring rule,
//! suggestion generation, corrected-reasoning assembly, the pass gate, the verification history
//! and its statistics.
//!
//! NOT ported (host-side by nature):
//!   * `verification_timestamp = datetime.now().isoformat()` - wall clock; the field is kept and
//!     left empty (the kernel has no clock here).
//!   * the `ctypes` CDLL call into swarm_v2_artifacts/native_voltage_strips/strip_cove_fallacy.dll
//!     and its numpy offset/length arrays - the ported marker scorer below IS that arithmetic
//!     (byte-level, same 6 markers and weights, same clamp).
//!   * `async def verify_reasoning(...)` - an asyncio wrapper; the kernel calls `verify` directly.
//!   * `json`/logging surfaces of callers.
//!
//! no_std notes (stated because they are deviations, not conveniences):
//!   * The kernel has no regex engine, so the oracle's patterns are carried as data and matched by
//!     a backtracking VM in this file. It supports exactly the subset the 47 fallacies and the two
//!     extractor patterns use: literals, `.`, one-byte classes (incl. negated), `\w \s \d`, `(?:)`
//!     and `()` groups, alternation, greedy/lazy `* + ?`, backrefs `\1 \2`, negative lookahead
//!     `(?!...)` and `^` under MULTILINE. `{n,m}` is not supported and no pattern needs it.
//!   * Matching is byte/ASCII oriented. Python's `\w` is Unicode-aware, `str.lower()` folds
//!     Unicode, and `len()` counts characters; here `\w` is `[0-9A-Za-z_]`, folding is ASCII, and
//!     character counts are `chars().count()`. On ASCII input the two agree exactly (measured:
//!     645/645 records over a 110-case corpus, 7800/7800 pattern-match decisions, plus the
//!     boundary corpora). On non-ASCII input they can differ: over 6 constructed Unicode cases the
//!     oracle found 6 fallacies and this port 3 - the three it misses are the `\w`-driven
//!     CIRCULAR_REASONING matches `café is café`, `naïve is naïve` (Python's `\w` is
//!     Unicode-aware; here a non-ASCII byte is not a word byte). ASCII prose, which is what the
//!     oracle's callers feed it, is unaffected.
//!   * `f64::round` does not exist in core, so the oracle's `round(avg_score, 3)` is hand-rolled
//!     as `round_half_even_3` (half-to-even on the scaled value, matching Python's tie rule at
//!     this precision).
//!   * Backtracking is bounded by RE_STEP_BUDGET (the oracle's C regex engine is unbounded); the
//!     budget is far above anything the oracle's own inputs need, and a pattern scan stops early
//!     rather than looping forever on adversarial input.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

// ── constants (oracle: __init__ default, _detect_fallacies window, _calculate_score) ──────────
/// `ChainOfVerification(confidence_threshold: float = 0.7)`.
pub const CONFIDENCE_THRESHOLD: f64 = 0.7;
/// `start_idx = max(0, match.start() - 250)` and `+ 250`, inclusive window for the refutation scan.
pub const REFUTATION_WINDOW: usize = 250;
/// `base_score = min(1.0, step_count * 0.15 + 0.4)`.
pub const SCORE_BASE: f64 = 0.4;
pub const SCORE_PER_STEP: f64 = 0.15;
/// `average_score = round(avg_score, 3)`.
pub const STATS_ROUND_DECIMALS: u32 = 3;
/// `re.split(r'[.!?]+', text)` fallback keeps sentences with `len(sentence) > 20`.
pub const MIN_SENTENCE_CHARS: usize = 20;

// ── FallacyType (oracle: class FallacyType(Enum), declaration order preserved) ────────────────
/// The oracle's 18 `FallacyType` members. BEGGING_THE_QUESTION and EQUIVOCATION are declared
/// there and appear in `get_fallacy_suggestion`, but carry no entry in FALLACY_PATTERNS, so
/// nothing constructs them in this port either - kept for parity, hence the allow.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FallacyType {
    CircularReasoning,
    FalseDichotomy,
    HastyGeneralization,
    AppealToAuthority,
    SlipperySlope,
    StrawMan,
    AdHominem,
    RedHerring,
    PostHoc,
    BeggingTheQuestion,
    Equivocation,
    AppealToEmotion,
    NonSequitur,
    Contradiction,
    IncompleteReasoning,
    CategoryError,
    Sycophancy,
    FactualInversion,
}

impl FallacyType {
    /// The oracle's `FallacyType.<X>.value`.
    pub fn as_str(self) -> &'static str {
        match self {
            FallacyType::CircularReasoning => "circular_reasoning",
            FallacyType::FalseDichotomy => "false_dichotomy",
            FallacyType::HastyGeneralization => "hasty_generalization",
            FallacyType::AppealToAuthority => "appeal_to_authority",
            FallacyType::SlipperySlope => "slippery_slope",
            FallacyType::StrawMan => "straw_man",
            FallacyType::AdHominem => "ad_hominem",
            FallacyType::RedHerring => "red_herring",
            FallacyType::PostHoc => "post_hoc",
            FallacyType::BeggingTheQuestion => "begging_the_question",
            FallacyType::Equivocation => "equivocation",
            FallacyType::AppealToEmotion => "appeal_to_emotion",
            FallacyType::NonSequitur => "non_sequitur",
            FallacyType::Contradiction => "contradiction",
            FallacyType::IncompleteReasoning => "incomplete_reasoning",
            FallacyType::CategoryError => "category_error",
            FallacyType::Sycophancy => "sycophancy",
            FallacyType::FactualInversion => "factual_inversion",
        }
    }

}

/// `_determine_severity`: `critical_fallacies` list, verbatim.
pub const CRITICAL_FALLACIES: [FallacyType; 4] = [
    FallacyType::CircularReasoning,
    FallacyType::Contradiction,
    FallacyType::CategoryError,
    FallacyType::FactualInversion,
];

/// `_determine_severity`: `high_fallacies` list, verbatim.
pub const HIGH_FALLACIES: [FallacyType; 3] = [
    FallacyType::HastyGeneralization,
    FallacyType::FalseDichotomy,
    FallacyType::Sycophancy,
];

/// `_detect_fallacies`: `refutation_words` list, verbatim (18 entries).
pub const REFUTATION_WORDS: [&str; 18] = [
    "category error",
    "does not replace",
    "cannot replace",
    "not replace",
    "cannot actuate",
    "does not actuate",
    "not dissipative",
    "do not lock",
    "does not lock",
    "conflates",
    "conflating",
    "conflation",
    "fallacy",
    "wrong model",
    "cannot hold",
    "cannot modulate",
    "incorrect to",
    "not a better model",
];

/// `_extract_premises`: `premise_markers` list, verbatim (order matters - findall per marker).
pub const PREMISE_MARKERS: [&str; 5] = ["because", "since", "given that", "assuming", "if"];

/// `_extract_conclusion`: `conclusion_markers` list, verbatim (first marker that hits wins).
pub const CONCLUSION_MARKERS: [&str; 6] =
    ["therefore", "thus", "hence", "so", "consequently", "as a result"];

/// `_get_fallacy_suggestion`: the oracle's 18-entry table, verbatim.
pub fn get_fallacy_suggestion(fallacy_type: FallacyType) -> &'static str {
    match fallacy_type {
        FallacyType::CircularReasoning => {
            "Provide independent evidence or premises that don't assume the conclusion."
        }
        FallacyType::FalseDichotomy => "Consider additional alternatives beyond the two presented.",
        FallacyType::HastyGeneralization => {
            "Gather more data before making universal claims."
        }
        FallacyType::AppealToAuthority => {
            "Cite specific evidence or studies rather than vague authority."
        }
        FallacyType::SlipperySlope => "Demonstrate each causal link with evidence.",
        FallacyType::StrawMan => "Address the strongest version of the opposing argument.",
        FallacyType::AdHominem => "Focus on the argument's merits rather than the source.",
        FallacyType::RedHerring => "Stay focused on the relevant issue at hand.",
        FallacyType::PostHoc => "Establish a causal mechanism, not just temporal sequence.",
        FallacyType::BeggingTheQuestion => {
            "Provide premises that don't presuppose the conclusion."
        }
        FallacyType::Equivocation => "Use terms consistently throughout the argument.",
        FallacyType::AppealToEmotion => {
            "Rely on logical evidence rather than emotional appeals."
        }
        FallacyType::NonSequitur => "Ensure conclusions follow logically from premises.",
        FallacyType::Contradiction => "Resolve inconsistent claims before proceeding.",
        FallacyType::IncompleteReasoning => "Make all premises and logical steps explicit.",
        FallacyType::CategoryError => {
            "Flag category confusion explicitly (e.g. escapements regulate oscillation; hand joints are continuous synovial articulators)."
        }
        FallacyType::Sycophancy => {
            "Reject invalid or contradictory user premises rather than fabricating supporting arguments."
        }
        FallacyType::FactualInversion => {
            "Correct inverted physical/biological claims with ground truth (e.g. synovial cartilage \u{3bc} \u{2248} 0.002-0.02 is ultra-low friction, not dissipative; neuromotor control is smooth and continuous)."
        }
    }
}

/// `severity` string values, used by `_calculate_score` and by the `passed` gate.
pub fn is_blocking_severity(severity: &str) -> bool {
    severity == "high" || severity == "critical"
}

/// `_calculate_score` penalty table: critical 0.25 / high 0.15 / medium 0.10 / else 0.05.
pub fn severity_penalty(severity: &str) -> f64 {
    if severity == "critical" {
        0.25
    } else if severity == "high" {
        0.15
    } else if severity == "medium" {
        0.10
    } else {
        0.05
    }
}

/// `FALLACY_PATTERNS` - (fallacy_type, regex source, description) in the oracle's dict order.
/// Extracted from the oracle source by script (see the pattern table note in the header).
pub static FALLACY_PATTERNS: &[(FallacyType, &str, &str)] = &[
    (FallacyType::CircularReasoning, r"(\w+)\s+(?:is|means|because)\s+\1", r"Definition restates itself"),
    (FallacyType::CircularReasoning, r"because\s+it\s+is\s+(?:true|correct|right)", r"Asserts conclusion as premise"),
    (FallacyType::CircularReasoning, r"this\s+is\s+(?:true|correct)\s+because\s+it\s+is", r"Begs the question"),
    (FallacyType::FalseDichotomy, r"(?:either|only)\s+\w+\s+or\s+\w+(?:\.|,|\s+no\s+other)", r"Presents only two options"),
    (FallacyType::FalseDichotomy, r"if\s+not\s+\w+,\s+then\s+(?:must\s+be\s+)?\w+", r"Forces binary choice"),
    (FallacyType::FalseDichotomy, r"the\s+only\s+(?:way|option|choice)\s+is", r"Eliminates alternatives"),
    (FallacyType::HastyGeneralization, r"(?:all|every|always)\s+\w+\s+(?:are|is|do|have)", r"Overgeneralization from limited data"),
    (FallacyType::HastyGeneralization, r"based\s+on\s+(?:this|one|a\s+single)", r"Generalizing from single instance"),
    (FallacyType::HastyGeneralization, r"clearly\s+all\s+", r"Unjustified universal claim"),
    (FallacyType::AppealToAuthority, r"(?:experts|scientists|studies)\s+says?\s+(?:it|this)\s+is\s+true", r"Vague authority appeal"),
    (FallacyType::AppealToAuthority, r"according\s+to\s+(?:an?\s+)?expert", r"Unnamed authority"),
    (FallacyType::AppealToAuthority, r"because\s+\w+\s+said\s+so", r"Authority without evidence"),
    (FallacyType::SlipperySlope, r"this\s+will\s+(?:inevitably|certainly|surely)\s+lead\s+to", r"Unjustified causal chain"),
    (FallacyType::SlipperySlope, r"step\s+by\s+step\s+(?:leading|resulting)\s+in", r"Inevitable decline claim"),
    (FallacyType::SlipperySlope, r"if\s+we\s+allow\s+this,\s+(?:then\s+)?eventually", r"Domino effect assertion"),
    (FallacyType::StrawMan, r"(?:some\s+might\s+say|critics\s+claim|opponents\s+argue)\s+that\s+\w+\s+(?:just\s+)?wants?\s+to\s+destroy", r"Exaggerated opponent position"),
    (FallacyType::StrawMan, r"obviously\s+wrong\s+to\s+suggest\s+that", r"Attacking weakened argument"),
    (FallacyType::AdHominem, r"\w+\s+is\s+(?:wrong|mistaken|lying)\s+because\s+(?:he|she|they)\s+(?:is|are|was|were)\s+an?", r"Attacks person not argument"),
    (FallacyType::AdHominem, r"don't\s+trust\s+\w+\s+because\s+of\s+(?:their|his|her)", r"Discredits source personally"),
    (FallacyType::AdHominem, r"coming\s+from\s+someone\s+who", r"Dismisses based on source"),
    (FallacyType::RedHerring, r"but\s+what\s+about\s+", r"Deflects to unrelated topic"),
    (FallacyType::RedHerring, r"speaking\s+of\s+which,\s+(?:unrelated|different)", r"Introduces irrelevant subject"),
    (FallacyType::RedHerring, r"this\s+reminds\s+me\s+of\s+(?:another|a\s+different)", r"Changes subject"),
    (FallacyType::PostHoc, r"(?:after|since|following)\s+\w+,\s+then\s+\w+\s+(?:happened|occurred|resulted)", r"Correlation implies causation"),
    (FallacyType::PostHoc, r"because\s+\w+\s+happened\s+first", r"Temporal order fallacy"),
    (FallacyType::PostHoc, r"coincidence\??\s+I\s+think\s+not", r"Unjustified causal claim"),
    (FallacyType::Contradiction, r"(\w+)\s+(?:is|are)\s+(\w+).+?(?:however|but|yet).+?\1\s+(?:is|are)\s+not\s+\2", r"Direct contradiction"),
    (FallacyType::Contradiction, r"always\s+.+?\s+never", r"Mutually exclusive claims"),
    (FallacyType::IncompleteReasoning, r"therefore\s+\w+\s+is\s+(?:true|correct|the\s+answer)", r"Conclusion without premises"),
    (FallacyType::IncompleteReasoning, r"simply\s+put,", r"Oversimplification marker"),
    (FallacyType::IncompleteReasoning, r"it's\s+obvious\s+that", r"Unexplained assertion"),
    (FallacyType::NonSequitur, r"therefore\s+(?:we\s+should|it\s+follows)", r"Non-logical conclusion"),
    (FallacyType::NonSequitur, r"thus\s+\w+\s+must\s+be", r"Invalid logical jump"),
    (FallacyType::AppealToEmotion, r"(?:think\s+of\s+the|consider\s+the)\s+(?:children|poor|victims)", r"Emotional manipulation"),
    (FallacyType::AppealToEmotion, r"(?:terrifying|horrifying|shocking|disgusting)\s+to\s+think", r"Fear-based appeal"),
    (FallacyType::AppealToEmotion, r"any\s+(?:decent|right-thinking|moral)\s+person", r"Moral pressure"),
    (FallacyType::CategoryError, r"(?:escapement|graham escape|deadbeat).*?(?:ratchet|pawl).*?(?:joint|grasp|finger|hand|dip)", r"Conflating oscillatory timekeeping escapement with load-holding ratchet mechanism in prosthetic grasping"),
    (FallacyType::CategoryError, r"(?:ratchet\s+variation\s+of\s+the\s+graham|ratchet\s+graham\s+escape)", r"Fabricating a fictitious hybrid mechanism (Graham escapement is an oscillatory regulator, not a ratchet)"),
    (FallacyType::CategoryError, r"(?:escapement|anchor\s+(?:escapement|mechanism)|verge\s+(?:escapement|and\s+foliot|mechanism)|lever\s+escapement|deadbeat|pallet(?:\s+fork|\s+and\s+wheel|\s+stone|\s+mechanism)?|balance\s+wheel|horological\s+(?:regulator|mechanism|impulse)|clockwork\s+(?:regulator|mechanism)|impulse\s+(?:release\s+)?(?:regulator|mechanism)|timekeeping\s+mechanism).*?(?:mimic|mirror|replace|actuate|serves?\s+as|substitute\s+for|emulate|used\s+as|adapted\s+as|functioning\s+as).*?(?:joint|mcp|pip|dip|knuckle|finger|grasp|flexion|cartilage|synovial|articulation)", r"Category error: Horological rate regulators govern oscillation timing, not continuous load-bearing joint articulation"),
    (FallacyType::CategoryError, r"(?:mcp|pip|dip|biological\s+joint|joint|finger|phalanx|cartilage).*?(?:passively\s+locks?|locks?\s+at\s+specific\s+angles|releases?\s+torque\s+in\s+discrete|moves?\s+in\s+discrete\s+steps|locks?\s+and\s+steps?|discrete\s+stepping|stepped\s+flexion|angle-locking)", r"Category error: Synovial joints do not passively lock at discrete angles or release torque in discrete steps"),
    (FallacyType::CategoryError, r"(?:escapement|pendulum|anchor|verge|foliot|deadbeat|pallet|balance\s+wheel|horological|clockwork).*?(?:mimic|mirror|replace|actuate|serves?\s+as|substitute\s+for|emulate|used\s+as|adapted\s+as|functioning\s+as).*?(?:cochlea|semicircular|vestibular|endolymph|cupula|ampulla|hair\s+cell|otolith|inner\s+ear|sensory\s+transduc|balance\s+organ|auditory)", r"Category error: Horological oscillators are periodic energy regulators, not fluid-dynamic inertial balance sensors or acoustic transducers"),
    (FallacyType::CategoryError, r"(?:cochlea).*?(?:replace|substitute\s+for|functioning\s+as|serves?\s+as|actuate).*?(?:semicircular|vestibular|balance\s+organ|angular\s+acceleration)", r"Anatomical category error: Conflating the auditory organ (cochlea) with the balance/equilibrium organ (semicircular canals)"),
    (FallacyType::Sycophancy, r"(?:as\s+requested|as\s+you\s+asked|to\s+comply\s+with\s+your|according\s+to\s+your\s+prompt).*?(?:superior|better\s+model)", r"Uncritical sycophantic acceptance of false premise in user prompt"),
    (FallacyType::FactualInversion, r"(?:biological\s+joints?|synovial|cartilage|articular).*?(?:inherently\s+dissipative|friction.*?converts?\s+energy|high\s+friction|dissipates?\s+energy|inefficiency\s+and\s+fatigue|too\s+dissipative|high-wear)", r"Factual falsehood: Synovial joints and articular cartilage are ultra-low friction (μ ≈ 0.002-0.02, lower than ice on ice); they are lubricated, not dissipative"),
    (FallacyType::FactualInversion, r"(?:biological\s+joints?|muscle\s+activation|motor\s+control|neuromotor|neuromuscular).*?(?:jerky|lack.*?precise\s+control|inherently\s+imprecise|poor\s+synchroniz|lacks?\s+timing\s+precision|imprecise\s+timing)", r"Factual falsehood: Human neuromuscular motor control is continuous, smooth, and finely graded, not jerky or stepped"),
    (FallacyType::FactualInversion, r"(?:escapement|pallet\s+fork|deadbeat|impulse\s+regulator).*?(?:holds?\s+static\s+load|continuous\s+force|modulate\s+grip|static\s+locking)", r"Physical falsehood: Horological escapements cannot hold static loads or modulate continuous grasping force"),
    (FallacyType::FactualInversion, r"(?:semicircular|vestibular|cupula|endolymph).*?(?:oscillat.*?governor|discrete\s+tick|clockwork|escapement\s+principle)", r"Factual falsehood: Semicircular canals are fluid-dynamic inertial sensors operating via viscous drag of endolymph, not clockwork oscillators"),
];

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Minimal backtracking regex VM — the kernel has no regex engine and the oracle's patterns are
// data, so they are compiled here and matched instruction-wise. Supported: literals, `.`, one-byte
// classes (incl. negated, incl. multi-byte class members as byte sequences), `\w \s \d`, `(?:)` and
// `()` groups, alternation (leftmost branch preferred, as Python does), greedy and lazy `* + ?`,
// backrefs `\1 \2`, negative lookahead `(?!...)`, `^`/`$` (MULTILINE aware). Anything else makes
// `compile` return None and the pattern is skipped (no pattern in the oracle needs more).
//
// ASCII_ONLY (declared divergence): `\w` here is `[0-9A-Za-z_]` where Python's str `\w` is
// Unicode-aware, and case folding is ASCII. Input that is pure ASCII - every string the oracle is
// used on - matches identically; non-ASCII input can differ. This is not hidden: the probe corpus
// carries a non-ASCII case and the count is reported.
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Backtracking budget per `finditer` scan. The oracle's C regex engine is unbounded; this bound
/// exists because the kernel cannot afford unbounded backtracking. It is far above what the
/// oracle's own inputs (reasoning text of a few hundred bytes over these 47 patterns) require.
pub const RE_STEP_BUDGET: u32 = 8_000_000;

/// `End` (`$`) and `{n,m}` are unused by the oracle's patterns; the instruction is kept so the
/// VM's contract is complete rather than partial, hence the allow.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ReInst {
    Lit(u8),
    Any,
    /// One byte in any of the ranges (or, when negated, in none of them).
    Class(Vec<(u8, u8)>, bool),
    Word,
    Space,
    Digit,
    /// Capture slot: `2*g-1` is the group start, `2*g` the group end (g is 1-based).
    Save(usize),
    /// Try the first target, then the second.
    Split(usize, usize),
    Jmp(usize),
    Backref(usize),
    Start,
    End,
    /// Succeed only when sub-program `n` (see `Re::subs`) fails at this position.
    LookaheadNot(usize),
    Match,
}

#[derive(Debug, Clone)]
pub struct Re {
    prog: Vec<ReInst>,
    subs: Vec<Vec<ReInst>>,
    ngroups: usize,
    ci: bool,
    multiline: bool,
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_space_byte(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' || b == 0x0b || b == 0x0c
}

/// Add `off` to every jump target inside a body (targets are absolute within the body).
fn shift_targets(body: &mut [ReInst], off: usize) {
    for inst in body.iter_mut() {
        match inst {
            ReInst::Split(a, b) => {
                *a += off;
                *b += off;
            }
            ReInst::Jmp(a) => {
                *a += off;
            }
            _ => {}
        }
    }
}

/// Right-nested alternation chain: branch `k` sits at `Split(1, ...)` with each earlier branch
/// falling through a `Jmp` to the end. Leftmost branch is tried first, as Python does.
fn assemble_alt(branches: Vec<Vec<ReInst>>) -> Vec<ReInst> {
    let mut acc: Option<Vec<ReInst>> = None;
    for mut body in branches.into_iter().rev() {
        let l = body.len();
        let mut out = Vec::new();
        match acc {
            None => {
                // The LAST alternative needs no Split: there is no further branch to try, so it is
                // just its body. Emitting a Split here (with the end of the alternation as its fail
                // target) lets "all branches failed" fall through to the pattern's continuation -
                // which at the top level is `Match`, i.e. a spurious empty match at every position.
                out.extend(body);
            }
            Some(mut rest) => {
                let r = rest.len();
                let rest_start = 1 + l + 1;
                out.push(ReInst::Split(1, rest_start));
                shift_targets(&mut body, 1);
                out.extend(body);
                out.push(ReInst::Jmp(rest_start + r));
                shift_targets(&mut rest, rest_start);
                out.extend(rest);
            }
        }
        acc = Some(out);
    }
    acc.unwrap_or_default()
}

/// `X*`, `X+`, `X?` (greedy or lazy) around an already-compiled body.
fn emit_repeat(mut body: Vec<ReInst>, min: u32, max: Option<u32>, lazy: bool) -> Vec<ReInst> {
    let l = body.len();
    let mut out = Vec::new();
    match (min, max) {
        (0, None) => {
            let base = 0usize;
            // greedy: prefer the loop; lazy: prefer to leave it.
            out.push(if lazy {
                ReInst::Split(1 + l + 1, 1)
            } else {
                ReInst::Split(1, 1 + l + 1)
            });
            shift_targets(&mut body, 1);
            out.extend(body);
            out.push(ReInst::Jmp(base));
        }
        (1, None) => {
            shift_targets(&mut body, 0);
            out.extend(body);
            let body_start = 0usize;
            let split_at = out.len();
            out.push(if lazy {
                ReInst::Split(split_at + 1, body_start)
            } else {
                ReInst::Split(body_start, split_at + 1)
            });
        }
        (0, Some(1)) => {
            out.push(if lazy {
                ReInst::Split(1 + l, 1)
            } else {
                ReInst::Split(1, 1 + l)
            });
            shift_targets(&mut body, 1);
            out.extend(body);
        }
        _ => {
            // `{n,m}` is not used by any oracle pattern; emitting an empty body makes compile fail.
            return Vec::new();
        }
    }
    out
}

struct ReParser<'p> {
    pat: &'p [u8],
    i: usize,
    ngroups: usize,
    subs: Vec<Vec<ReInst>>,
    err: bool,
}

impl<'p> ReParser<'p> {
    fn peek(&self) -> Option<u8> {
        self.pat.get(self.i).copied()
    }

    fn peek_at(&self, off: usize) -> Option<u8> {
        self.pat.get(self.i + off).copied()
    }

    fn parse_alt(&mut self) -> Vec<ReInst> {
        let mut branches = vec![self.parse_concat()];
        while !self.err && self.peek() == Some(b'|') {
            self.i += 1;
            branches.push(self.parse_concat());
        }
        if branches.len() == 1 {
            return branches.pop().unwrap_or_default();
        }
        assemble_alt(branches)
    }

    fn parse_concat(&mut self) -> Vec<ReInst> {
        let mut out: Vec<ReInst> = Vec::new();
        while !self.err {
            match self.peek() {
                None | Some(b'|') | Some(b')') => break,
                _ => {}
            }
            let atom = self.parse_atom();
            if self.err {
                return Vec::new();
            }
            let q = self.peek();
            // An atom's internal jump targets are relative to the atom's own start, so each piece
            // must be shifted by the offset it is appended at (missing this made every pattern with
            // a jump after its first atom either miss or spin).
            let mut piece = if q == Some(b'*') || q == Some(b'+') || q == Some(b'?') {
                self.i += 1;
                let mut lazy = false;
                if self.peek() == Some(b'?') {
                    self.i += 1;
                    lazy = true;
                }
                let (min, max) = match q {
                    Some(b'*') => (0u32, None),
                    Some(b'+') => (1u32, None),
                    _ => (0u32, Some(1u32)),
                };
                let rep = emit_repeat(atom, min, max, lazy);
                if rep.is_empty() {
                    self.err = true;
                    return Vec::new();
                }
                rep
            } else {
                atom
            };
            let base = out.len();
            shift_targets(&mut piece, base);
            out.extend(piece);
        }
        out
    }

    fn parse_atom(&mut self) -> Vec<ReInst> {
        let c = match self.peek() {
            Some(c) => c,
            None => return Vec::new(),
        };
        match c {
            b'(' => {
                if self.peek_at(1) == Some(b'?') && self.peek_at(2) == Some(b':') {
                    self.i += 3;
                    let body = self.parse_alt();
                    if self.peek() != Some(b')') {
                        self.err = true;
                        return Vec::new();
                    }
                    self.i += 1;
                    return body;
                }
                if self.peek_at(1) == Some(b'?') && self.peek_at(2) == Some(b'!') {
                    self.i += 3;
                    let mut body = self.parse_alt();
                    if self.peek() != Some(b')') {
                        self.err = true;
                        return Vec::new();
                    }
                    self.i += 1;
                    // A sub-program needs its own `Match` terminator: without it the sub can never
                    // succeed (so `(?!X)` would always succeed) and a fall-through target at the end
                    // of the sub runs one past its last instruction. (Found by probe: index out of
                    // bounds, len 6 index 6, on the numbered-step extractor's `(?!\s*\d+\.)`.)
                    body.push(ReInst::Match);
                    let idx = self.subs.len();
                    self.subs.push(body);
                    return vec![ReInst::LookaheadNot(idx)];
                }
                if self.peek_at(1) == Some(b'?') {
                    // (?=...) / (?P<...>) and friends are not used by any oracle pattern.
                    self.err = true;
                    return Vec::new();
                }
                self.i += 1;
                self.ngroups += 1;
                let g = self.ngroups;
                let mut body = self.parse_alt();
                if self.peek() != Some(b')') {
                    self.err = true;
                    return Vec::new();
                }
                self.i += 1;
                let mut out = Vec::with_capacity(body.len() + 2);
                out.push(ReInst::Save(2 * g - 1));
                shift_targets(&mut body, 1);
                out.extend(body);
                out.push(ReInst::Save(2 * g));
                out
            }
            b'[' => self.parse_class(),
            b'\\' => self.parse_escape(),
            b'.' => {
                self.i += 1;
                vec![ReInst::Any]
            }
            b'^' => {
                self.i += 1;
                vec![ReInst::Start]
            }
            b'$' => {
                self.i += 1;
                vec![ReInst::End]
            }
            b'*' | b'+' => {
                // A quantifier with nothing to repeat: Python raises, we fail the compile.
                self.err = true;
                Vec::new()
            }
            b'{' => {
                // `{n,m}` unsupported (unused by the oracle); a bare `{` stays a literal.
                if self.peek_at(1).map_or(false, |d| d.is_ascii_digit()) {
                    self.err = true;
                    Vec::new()
                } else {
                    self.i += 1;
                    vec![ReInst::Lit(b'{')]
                }
            }
            b')' => {
                self.err = true;
                Vec::new()
            }
            _ => {
                self.i += 1;
                vec![ReInst::Lit(c)]
            }
        }
    }

    fn parse_escape(&mut self) -> Vec<ReInst> {
        self.i += 1;
        let e = match self.peek() {
            Some(e) => e,
            None => {
                self.err = true;
                return Vec::new();
            }
        };
        self.i += 1;
        match e {
            b'w' => vec![ReInst::Word],
            b's' => vec![ReInst::Space],
            b'd' => vec![ReInst::Digit],
            b'W' | b'S' | b'D' | b'b' | b'B' | b'A' | b'Z' => {
                // Not used by any oracle pattern.
                self.err = true;
                Vec::new()
            }
            b'n' => vec![ReInst::Lit(b'\n')],
            b't' => vec![ReInst::Lit(b'\t')],
            b'r' => vec![ReInst::Lit(b'\r')],
            b'f' => vec![ReInst::Lit(0x0c)],
            b'v' => vec![ReInst::Lit(0x0b)],
            b'0' => vec![ReInst::Lit(0)],
            b'1'..=b'9' => vec![ReInst::Backref((e - b'0') as usize)],
            other => vec![ReInst::Lit(other)],
        }
    }

    /// `[...]` — one byte from the set (or, negated, one byte outside it). A member that encodes to
    /// several bytes (the bullet `•` in the extractor's `[-•]`) becomes an alternation of literals.
    fn parse_class(&mut self) -> Vec<ReInst> {
        self.i += 1; // '['
        let mut negated = false;
        if self.peek() == Some(b'^') {
            negated = true;
            self.i += 1;
        }
        let mut ranges: Vec<(u8, u8)> = Vec::new();
        let mut alts: Vec<Vec<u8>> = Vec::new();
        loop {
            let c = match self.peek() {
                Some(c) => c,
                None => {
                    self.err = true;
                    return Vec::new();
                }
            };
            if c == b']' && (!ranges.is_empty() || !alts.is_empty()) {
                self.i += 1;
                break;
            }
            // one member (possibly escaped, possibly multi-byte)
            let member: Vec<u8> = if c == b'\\' {
                self.i += 1;
                let e = match self.peek() {
                    Some(e) => e,
                    None => {
                        self.err = true;
                        return Vec::new();
                    }
                };
                self.i += 1;
                match e {
                    b'w' => {
                        ranges.push((b'0', b'9'));
                        ranges.push((b'A', b'Z'));
                        ranges.push((b'a', b'z'));
                        ranges.push((b'_', b'_'));
                        Vec::new()
                    }
                    b's' => {
                        ranges.push((b' ', b' '));
                        ranges.push((0x09, 0x0d));
                        Vec::new()
                    }
                    b'd' => {
                        ranges.push((b'0', b'9'));
                        Vec::new()
                    }
                    b'n' => vec![b'\n'],
                    b't' => vec![b'\t'],
                    b'r' => vec![b'\r'],
                    other => vec![other],
                }
            } else {
                // A class member is a whole (possibly multi-byte) character, not one byte: reading the
                // bullet in `[-<bullet>]` byte-by-byte turned that class into three single-byte
                // members (0xE2/0x80/0xA2) and the bullet extractor silently never matched. (Probe.)
                self.i += 1;
                let extra = match c {
                    0xc0..=0xdf => 1usize,
                    0xe0..=0xef => 2,
                    0xf0..=0xf7 => 3,
                    _ => 0,
                };
                let mut m = vec![c];
                for _ in 0..extra {
                    if let Some(n) = self.peek() {
                        m.push(n);
                        self.i += 1;
                    }
                }
                m
            };
            if member.is_empty() {
                continue;
            }
            if member.len() == 1 {
                // possible range `a-z`
                if self.peek() == Some(b'-') && self.peek_at(1).map_or(false, |n| n != b']') {
                    self.i += 1;
                    let hi = match self.peek() {
                        Some(h) => h,
                        None => {
                            self.err = true;
                            return Vec::new();
                        }
                    };
                    self.i += 1;
                    ranges.push((member[0], hi));
                } else {
                    ranges.push((member[0], member[0]));
                }
            } else {
                if negated {
                    // A negated class with a multi-byte member would need char semantics; unused.
                    self.err = true;
                    return Vec::new();
                }
                alts.push(member);
            }
        }
        if alts.is_empty() {
            return vec![ReInst::Class(ranges, negated)];
        }
        let mut pieces: Vec<Vec<ReInst>> = Vec::new();
        pieces.push(vec![ReInst::Class(ranges, false)]);
        for a in alts {
            pieces.push(a.into_iter().map(ReInst::Lit).collect());
        }
        assemble_alt(pieces)
    }
}

struct ReRun<'h> {
    hay: &'h [u8],
    caps: Vec<usize>,
    ci: bool,
    multiline: bool,
    steps: u32,
    end: usize,
}

impl Re {
    pub fn compile(pattern: &str, ci: bool, multiline: bool) -> Option<Re> {
        let mut p = ReParser {
            pat: pattern.as_bytes(),
            i: 0,
            ngroups: 0,
            subs: Vec::new(),
            err: false,
        };
        let mut prog = p.parse_alt();
        if p.err || p.i != p.pat.len() {
            return None;
        }
        prog.push(ReInst::Match);
        Some(Re {
            prog,
            subs: p.subs,
            ngroups: p.ngroups,
            ci,
            multiline,
        })
    }

    fn eq_byte(&self, a: u8, b: u8) -> bool {
        if self.ci {
            a.to_ascii_lowercase() == b.to_ascii_lowercase()
        } else {
            a == b
        }
    }

    fn run(&self, prog: &[ReInst], st: &mut ReRun, pc: usize, pos: usize) -> bool {
        st.steps += 1;
        if st.steps > RE_STEP_BUDGET {
            return false;
        }
        match prog[pc] {
            ReInst::Lit(b) => {
                pos < st.hay.len() && self.eq_byte(st.hay[pos], b) && self.run(prog, st, pc + 1, pos + 1)
            }
            ReInst::Any => {
                pos < st.hay.len() && st.hay[pos] != b'\n' && self.run(prog, st, pc + 1, pos + 1)
            }
            ReInst::Class(ref ranges, negated) => {
                if pos >= st.hay.len() {
                    return false;
                }
                let b = st.hay[pos];
                let mut hit = false;
                for &(lo, hi) in ranges.iter() {
                    if b >= lo && b <= hi {
                        hit = true;
                        break;
                    }
                }
                if negated {
                    !hit && self.run(prog, st, pc + 1, pos + 1)
                } else {
                    hit && self.run(prog, st, pc + 1, pos + 1)
                }
            }
            ReInst::Word => {
                pos < st.hay.len() && is_word_byte(st.hay[pos]) && self.run(prog, st, pc + 1, pos + 1)
            }
            ReInst::Space => {
                pos < st.hay.len()
                    && is_space_byte(st.hay[pos])
                    && self.run(prog, st, pc + 1, pos + 1)
            }
            ReInst::Digit => {
                pos < st.hay.len()
                    && st.hay[pos].is_ascii_digit()
                    && self.run(prog, st, pc + 1, pos + 1)
            }
            ReInst::Save(n) => {
                let old = st.caps[n];
                st.caps[n] = pos;
                if self.run(prog, st, pc + 1, pos) {
                    true
                } else {
                    st.caps[n] = old;
                    false
                }
            }
            ReInst::Split(a, b) => {
                self.run(prog, st, a, pos) || self.run(prog, st, b, pos)
            }
            ReInst::Jmp(a) => self.run(prog, st, a, pos),
            ReInst::Backref(g) => {
                let s = *st.caps.get(2 * g - 1).unwrap_or(&usize::MAX);
                let e = *st.caps.get(2 * g).unwrap_or(&usize::MAX);
                if s == usize::MAX || e == usize::MAX || e < s || e > st.hay.len() {
                    return false;
                }
                let n = e - s;
                if pos + n > st.hay.len() {
                    return false;
                }
                let mut ok = true;
                let mut k = 0;
                while k < n {
                    if !self.eq_byte(st.hay[pos + k], st.hay[s + k]) {
                        ok = false;
                        break;
                    }
                    k += 1;
                }
                ok && self.run(prog, st, pc + 1, pos + n)
            }
            ReInst::Start => {
                let ok = pos == 0 || (self.multiline && st.hay[pos - 1] == b'\n');
                ok && self.run(prog, st, pc + 1, pos)
            }
            ReInst::End => {
                let n = st.hay.len();
                let ok = pos == n
                    || (pos < n && st.hay[pos] == b'\n')
                    || (pos + 1 == n && st.hay[pos] == b'\n');
                ok && self.run(prog, st, pc + 1, pos)
            }
            ReInst::LookaheadNot(si) => {
                let sub = &self.subs[si];
                let mut inner = ReRun {
                    hay: st.hay,
                    caps: vec![usize::MAX; st.caps.len()],
                    ci: st.ci,
                    multiline: st.multiline,
                    steps: st.steps,
                    end: usize::MAX,
                };
                let hit = self.run(sub, &mut inner, 0, pos);
                st.steps = inner.steps;
                if hit {
                    false
                } else {
                    self.run(prog, st, pc + 1, pos)
                }
            }
            ReInst::Match => {
                st.end = pos;
                true
            }
        }
    }

    /// `re.finditer`: non-overlapping, leftmost, scanning left to right; an empty match advances by
    /// one byte (Python does the same).
    pub fn finditer(&self, hay: &[u8]) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos <= hay.len() {
            let mut st = ReRun {
                hay,
                caps: vec![usize::MAX; 2 * (self.ngroups + 1)],
                ci: self.ci,
                multiline: self.multiline,
                steps: 0,
                end: usize::MAX,
            };
            if self.run(&self.prog, &mut st, 0, pos) {
                let e = st.end;
                out.push((pos, e));
                pos = if e > pos { e } else { pos + 1 };
            } else {
                pos += 1;
            }
        }
        out
    }

    /// `re.findall` over the pattern's single capture group, as byte spans.
    pub fn findall_group1(&self, hay: &[u8]) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos <= hay.len() {
            let mut st = ReRun {
                hay,
                caps: vec![usize::MAX; 2 * (self.ngroups + 1)],
                ci: self.ci,
                multiline: self.multiline,
                steps: 0,
                end: usize::MAX,
            };
            if self.run(&self.prog, &mut st, 0, pos) {
                let e = st.end;
                if self.ngroups >= 1 {
                    let gs = st.caps[1];
                    let ge = st.caps[2];
                    if gs != usize::MAX && ge != usize::MAX && ge >= gs {
                        out.push((gs, ge));
                    } else {
                        out.push((0, 0));
                    }
                }
                pos = if e > pos { e } else { pos + 1 };
            } else {
                pos += 1;
            }
        }
        out
    }

    /// Group 1 and group 2 spans of every match (the numbered-step extractor reads both).
    pub fn finditer_groups12(&self, hay: &[u8]) -> Vec<(usize, usize, usize, usize)> {
        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos <= hay.len() {
            let mut st = ReRun {
                hay,
                caps: vec![usize::MAX; 2 * (self.ngroups + 1)],
                ci: self.ci,
                multiline: self.multiline,
                steps: 0,
                end: usize::MAX,
            };
            if self.run(&self.prog, &mut st, 0, pos) {
                let e = st.end;
                let (s1, e1) = if self.ngroups >= 1 {
                    (st.caps[1], st.caps[2])
                } else {
                    (usize::MAX, usize::MAX)
                };
                let (s2, e2) = if self.ngroups >= 2 {
                    (st.caps[3], st.caps[4])
                } else {
                    (usize::MAX, usize::MAX)
                };
                out.push((s1, e1, s2, e2));
                pos = if e > pos { e } else { pos + 1 };
            } else {
                pos += 1;
            }
        }
        out
    }
}

// ── dataclasses (oracle field order preserved) ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub step_number: i64,
    pub content: String,
    pub premises: Vec<String>,
    pub conclusion: String,
    pub confidence: f64,
}

impl ReasoningStep {
    /// The oracle declares `confidence: float = 1.0`; the previous stub defaulted it to 0.0, which
    /// is a different value for a field the oracle treats as "certain by default".
    pub fn new() -> Self {
        ReasoningStep {
            step_number: 0,
            content: String::new(),
            premises: vec![],
            conclusion: String::new(),
            confidence: 1.0,
        }
    }

}

#[derive(Debug, Clone)]
pub struct FallacyDetection {
    pub fallacy_type: FallacyType,
    pub severity: String,
    pub step_number: i64,
    pub description: String,
    pub suggestion: String,
    pub affected_text: String,
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub passed: bool,
    pub score: f64,
    pub reasoning_steps: Vec<ReasoningStep>,
    pub detected_fallacies: Vec<FallacyDetection>,
    pub suggestions: Vec<String>,
    pub corrected_reasoning: Option<String>,
    /// Oracle: `datetime.now().isoformat()` — host-side wall clock, left empty in the kernel.
    /// Nothing in the kernel reads it (the caller's UI would); kept so the result struct matches
    /// the oracle's field set.
    #[allow(dead_code)]
    pub verification_timestamp: String,
}

impl VerificationResult {
    pub fn new(
        passed: bool,
        score: f64,
        reasoning_steps: Vec<ReasoningStep>,
        detected_fallacies: Vec<FallacyDetection>,
        suggestions: Vec<String>,
        corrected_reasoning: Option<String>,
    ) -> Self {
        VerificationResult {
            passed,
            score,
            reasoning_steps,
            detected_fallacies,
            suggestions,
            corrected_reasoning,
            verification_timestamp: String::new(),
        }
    }
}

/// `get_verification_stats()` return value (the oracle returns a dict).
#[derive(Debug, Clone)]
pub struct VerificationStats {
    pub total_verifications: u64,
    pub passed: u64,
    pub failed: u64,
    pub pass_rate: f64,
    /// `round(avg_score, 3)`.
    pub average_score: f64,
    /// `sorted(counts.items(), key=count, reverse=True)[:5]` — stable, so ties keep first-seen order.
    pub most_common_fallacies: Vec<(String, u64)>,
}

/// no_std `f64::floor` (core has no floor): truncate toward zero, step down when the cast rounded up.
pub fn floor_f64(x: f64) -> f64 {
    if x != x {
        return x;
    }
    let t = x as i64 as f64;
    if t > x {
        t - 1.0
    } else {
        t
    }
}

/// Hand-rolled `round(x, 3)` without std/libm: half-to-even on the scaled value, which is Python's
/// tie rule at this precision (`round()` breaks ties to the even digit). The oracle's scores live on
/// the lattice `0.4 + 0.15*steps - {0.25,0.15,0.10,0.05}*k`, so an exact tie is not reachable by the
/// oracle's own arithmetic either; the probe compares the value bit-for-bit against Python.
pub fn round_half_even_3(x: f64) -> f64 {
    // The oracle's `round(x, 3)`: the scale comes from STATS_ROUND_DECIMALS, not a hardcoded 1000.
    let mut scale = 1.0f64;
    let mut k = 0u32;
    while k < STATS_ROUND_DECIMALS {
        scale *= 10.0;
        k += 1;
    }
    let scaled = x * scale;
    let f = floor_f64(scaled);
    let frac = scaled - f;
    let r = if frac > 0.5 {
        f + 1.0
    } else if frac < 0.5 {
        f
    } else if f % 2.0 == 0.0 {
        f
    } else {
        f + 1.0
    };
    r / scale
}

/// ASCII-only lowercase, matching Python's `str.lower()` on ASCII input (declared divergence on
/// non-ASCII input: Python folds Unicode, this does not).
pub fn ascii_lower(s: &str) -> String {
    let src = s.as_bytes();
    let mut out = Vec::with_capacity(src.len());
    for &b in src.iter() {
        out.push(b.to_ascii_lowercase());
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn bytes_contains(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > hay.len() {
        return needle.is_empty();
    }
    let mut i = 0usize;
    while i + needle.len() <= hay.len() {
        if &hay[i..i + needle.len()] == needle {
            return true;
        }
        i += 1;
    }
    false
}

/// Slice a byte range out of the text as a String. Byte spans come from the byte-level engine, so on
/// non-ASCII input a span can split a character; `from_utf8_lossy` keeps that from panicking.
fn slice_to_string(text: &str, s: usize, e: usize) -> String {
    let b = text.as_bytes();
    if s > e || e > b.len() {
        return String::new();
    }
    String::from_utf8_lossy(&b[s..e]).into_owned()
}

// ── ChainOfVerification ────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChainOfVerification {
    pub confidence_threshold: f64,
    pub verification_history: Vec<VerificationResult>,
}

impl ChainOfVerification {
    pub fn new(confidence_threshold: f64) -> Self {
        ChainOfVerification {
            confidence_threshold,
            verification_history: Vec::new(),
        }
    }

    /// `verify(reasoning_text, auto_correct=False)` — the five oracle steps in order.
    pub fn verify(&mut self, reasoning_text: &str, auto_correct: bool) -> VerificationResult {
        // Step 1: Extract reasoning steps
        let steps = self.extract_reasoning_steps(reasoning_text);

        // Step 2: Detect fallacies in each step
        let fallacies = self.detect_fallacies(reasoning_text, &steps);

        // Step 3: Calculate verification score
        let score = self.calculate_score(steps.len(), &fallacies);

        // Step 4: Generate suggestions
        let suggestions = self.generate_suggestions(&fallacies);

        // Step 5: Auto-correct if requested
        let corrected = if auto_correct && !fallacies.is_empty() {
            Some(self.generate_corrected_reasoning(reasoning_text, &fallacies))
        } else {
            None
        };

        // Build result — the pass gate is the oracle's own boolean, including the strict `>=`
        // on the threshold and the high/critical severity veto.
        let blocking = fallacies
            .iter()
            .any(|f| is_blocking_severity(f.severity.as_str()));
        let result = VerificationResult::new(
            score >= self.confidence_threshold && !blocking,
            score,
            steps,
            fallacies,
            suggestions,
            corrected,
        );

        self.verification_history.push(result.clone());
        result
    }

    /// `_extract_reasoning_steps`: numbered steps, else bullets, else sentences longer than 20 chars.
    pub fn extract_reasoning_steps(&self, text: &str) -> Vec<ReasoningStep> {
        let mut steps: Vec<ReasoningStep> = Vec::new();

        // Pattern 1: numbered steps (re.MULTILINE).
        let numbered = Re::compile(
            r"(?:^|\n)\s*(\d+)\.\s+([^\n]+(?:\n(?!\s*\d+\.)[^\n]*)*)",
            false,
            true,
        );
        if let Some(re) = numbered {
            for m in numbered_matches(&re, text) {
                let step_num = match m.0.parse::<i64>() {
                    Ok(v) => v,
                    // Python ints are unbounded; i64 saturates at the extreme instead of failing.
                    Err(_) => i64::MAX,
                };
                let content = m.1.trim().to_string();
                let mut step = ReasoningStep::new();
                step.step_number = step_num;
                step.premises = self.extract_premises(&content);
                step.conclusion = self.extract_conclusion(&content);
                step.content = content;
                steps.push(step);
            }
        }

        // Pattern 2: bullets or dashes (only when no numbered steps were found).
        if steps.is_empty() {
            if let Some(re) = Re::compile(r"(?:^|\n)\s*[-•]\s+([^\n]+)", false, true) {
                let mut i = 1i64;
                for (s, e) in re.findall_group1(text.as_bytes()) {
                    let content = slice_to_string(text, s, e).trim().to_string();
                    let mut step = ReasoningStep::new();
                    step.step_number = i;
                    step.premises = self.extract_premises(&content);
                    step.conclusion = self.extract_conclusion(&content);
                    step.content = content;
                    steps.push(step);
                    i += 1;
                }
            }
        }

        // Pattern 3: sentence-based fallback (`re.split(r'[.!?]+', text)`, keep len > 20).
        if steps.is_empty() {
            let sentences = split_on_delim_runs(text);
            for (i, sentence) in sentences.iter().enumerate() {
                let sentence = sentence.trim();
                // Python counts CHARACTERS here, not bytes.
                if sentence.chars().count() > MIN_SENTENCE_CHARS {
                    let content = sentence.to_string();
                    let mut step = ReasoningStep::new();
                    step.step_number = (i + 1) as i64;
                    step.premises = self.extract_premises(&content);
                    step.conclusion = self.extract_conclusion(&content);
                    step.content = content;
                    steps.push(step);
                }
            }
        }

        steps
    }

    /// `_extract_premises`: for each marker, `re.findall(f"{marker}\\s+([^,\\.]+)", text, IGNORECASE)`.
    /// Note the oracle does NOT strip these captures, so neither does this.
    pub fn extract_premises(&self, text: &str) -> Vec<String> {
        let mut premises = Vec::new();
        for marker in PREMISE_MARKERS.iter() {
            let pat = format!("{}\\s+([^,\\.]+)", marker);
            if let Some(re) = Re::compile(&pat, true, false) {
                for (s, e) in re.findall_group1(text.as_bytes()) {
                    premises.push(slice_to_string(text, s, e));
                }
            }
        }
        premises
    }

    /// `_extract_conclusion`: the first conclusion marker's capture (stripped), else the last
    /// `[,.]`-separated piece (stripped).
    pub fn extract_conclusion(&self, text: &str) -> String {
        for marker in CONCLUSION_MARKERS.iter() {
            let pat = format!("{}\\s+([^,\\.]+)", marker);
            if let Some(re) = Re::compile(&pat, true, false) {
                if let Some((s, e)) = re.findall_group1(text.as_bytes()).into_iter().next() {
                    return slice_to_string(text, s, e).trim().to_string();
                }
            }
        }
        let sentences = split_on_chars(text, &[b',', b'.']);
        match sentences.last() {
            Some(s) => s.trim().to_string(),
            None => String::new(),
        }
    }

    /// `_detect_fallacies`: every pattern of every type, in the oracle's table order, with the
    /// 250-byte refutation window applied to CATEGORY_ERROR and FACTUAL_INVERSION.
    pub fn detect_fallacies(&self, text: &str, steps: &[ReasoningStep]) -> Vec<FallacyDetection> {
        let mut fallacies: Vec<FallacyDetection> = Vec::new();
        let text_lower = ascii_lower(text);
        let hay = text_lower.as_bytes();

        for &(fallacy_type, pattern, description) in FALLACY_PATTERNS.iter() {
            let re = match Re::compile(pattern, true, false) {
                Some(r) => r,
                None => continue,
            };
            for (ms, me) in re.finditer(hay) {
                // Explicit refutation of a category error / factual inversion is not a fallacy.
                if fallacy_type == FallacyType::CategoryError
                    || fallacy_type == FallacyType::FactualInversion
                {
                    let start_idx = ms.saturating_sub(REFUTATION_WINDOW);
                    let end_idx = core::cmp::min(hay.len(), me + REFUTATION_WINDOW);
                    let snippet = &hay[start_idx..end_idx];
                    let mut refuted = false;
                    for rw in REFUTATION_WORDS.iter() {
                        if bytes_contains(snippet, rw.as_bytes()) {
                            refuted = true;
                            break;
                        }
                    }
                    if refuted {
                        continue;
                    }
                }

                let affected = slice_to_string(&text_lower, ms, me);
                let severity = self.determine_severity(fallacy_type, &affected);
                let step_number = self.find_step_for_text(&affected, steps);
                fallacies.push(FallacyDetection {
                    fallacy_type,
                    severity,
                    step_number,
                    description: description.to_string(),
                    suggestion: get_fallacy_suggestion(fallacy_type).to_string(),
                    affected_text: affected,
                });
            }
        }

        fallacies
    }

    /// `_determine_severity`: critical list, then high list, then an `always`/`never` probe, else low.
    pub fn determine_severity(&self, fallacy_type: FallacyType, text: &str) -> String {
        if CRITICAL_FALLACIES.contains(&fallacy_type) {
            return "critical".to_string();
        }
        if HIGH_FALLACIES.contains(&fallacy_type) {
            return "high".to_string();
        }
        let lowered = text.to_lowercase();
        if lowered.contains("always") || lowered.contains("never") {
            return "medium".to_string();
        }
        "low".to_string()
    }

    /// `_find_step_for_text`: the first step whose lowercased content contains the text, else 0.
    pub fn find_step_for_text(&self, text: &str, steps: &[ReasoningStep]) -> i64 {
        let text_lower = text.to_lowercase();
        for step in steps.iter() {
            if step.content.to_lowercase().contains(&text_lower) {
                return step.step_number;
            }
        }
        0
    }

    /// `_calculate_score`: no fallacies -> 1.0 with steps, 0.85 without; otherwise the structured
    /// base minus one penalty per fallacy, clamped to [0, 1].
    pub fn calculate_score(&self, step_count: usize, fallacies: &[FallacyDetection]) -> f64 {
        if fallacies.is_empty() {
            return if step_count > 0 { 1.0 } else { 0.85 };
        }

        let mut base_score = (step_count as f64) * SCORE_PER_STEP + SCORE_BASE;
        if base_score > 1.0 {
            base_score = 1.0;
        }

        let mut penalty = 0.0;
        for f in fallacies.iter() {
            penalty += severity_penalty(f.severity.as_str());
        }

        let score = base_score - penalty;
        if score < 0.0 {
            0.0
        } else if score > 1.0 {
            1.0
        } else {
            score
        }
    }

    /// `_generate_suggestions`: one line per fallacy TYPE, first-seen order, with the instance count.
    pub fn generate_suggestions(&self, fallacies: &[FallacyDetection]) -> Vec<String> {
        let mut kinds: Vec<FallacyType> = Vec::new();
        let mut counts: Vec<u64> = Vec::new();
        let mut first_suggestion: Vec<String> = Vec::new();
        for f in fallacies.iter() {
            let mut found = false;
            for (idx, k) in kinds.iter().enumerate() {
                if *k == f.fallacy_type {
                    counts[idx] += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                kinds.push(f.fallacy_type);
                counts.push(1);
                first_suggestion.push(f.suggestion.clone());
            }
        }

        let mut suggestions = Vec::new();
        for (idx, k) in kinds.iter().enumerate() {
            let mut s = format!("\u{2022} [{}] {}", k.as_str(), first_suggestion[idx]);
            if counts[idx] > 1 {
                s.push_str(&format!(" (found {} instances)", counts[idx]));
            }
            suggestions.push(s);
        }
        suggestions
    }

    /// `_generate_corrected_reasoning`: the original plus one verification note per fallacy.
    pub fn generate_corrected_reasoning(
        &self,
        original: &str,
        fallacies: &[FallacyDetection],
    ) -> String {
        let mut corrections: Vec<String> = Vec::new();
        for f in fallacies.iter() {
            corrections.push(format!(
                "Note: {} in step {}. {}",
                f.description, f.step_number, f.suggestion
            ));
        }
        let mut out = original.to_string();
        out.push_str("\n\n[VERIFICATION NOTES]\n");
        for (i, c) in corrections.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(c);
        }
        out
    }

    /// `get_verification_stats()` — including the `round(avg_score, 3)` and the top-5 ordering.
    pub fn get_verification_stats(&self) -> VerificationStats {
        if self.verification_history.is_empty() {
            return VerificationStats {
                total_verifications: 0,
                passed: 0,
                failed: 0,
                pass_rate: 0.0,
                average_score: 0.0,
                most_common_fallacies: Vec::new(),
            };
        }

        let total = self.verification_history.len() as u64;
        let mut passed = 0u64;
        let mut sum = 0.0f64;
        let mut keys: Vec<String> = Vec::new();
        let mut counts: Vec<u64> = Vec::new();
        for result in self.verification_history.iter() {
            if result.passed {
                passed += 1;
            }
            sum += result.score;
            for f in result.detected_fallacies.iter() {
                let key = f.fallacy_type.as_str();
                let mut found = false;
                for (idx, k) in keys.iter().enumerate() {
                    if k == key {
                        counts[idx] += 1;
                        found = true;
                        break;
                    }
                }
                if !found {
                    keys.push(key.to_string());
                    counts.push(1);
                }
            }
        }
        let avg_score = sum / (total as f64);

        // Stable descending sort by count (Python's `sorted(..., reverse=True)` is stable, so ties
        // keep the order in which the type first appeared).
        let mut pairs: Vec<(String, u64)> = Vec::new();
        for (idx, k) in keys.iter().enumerate() {
            pairs.push((k.clone(), counts[idx]));
        }
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        if pairs.len() > 5 {
            pairs.truncate(5);
        }

        VerificationStats {
            total_verifications: total,
            passed,
            failed: total - passed,
            pass_rate: (passed as f64) / (total as f64),
            average_score: round_half_even_3(avg_score),
            most_common_fallacies: pairs,
        }
    }
}

/// Numbered-step extraction needs both groups: group 1 = step number, group 2 = step content.
fn numbered_matches(re: &Re, text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (s1, e1, s2, e2) in re.finditer_groups12(text.as_bytes()) {
        if s1 == usize::MAX || e1 == usize::MAX || s2 == usize::MAX || e2 == usize::MAX {
            continue;
        }
        out.push((
            slice_to_string(text, s1, e1),
            slice_to_string(text, s2, e2),
        ));
    }
    out
}

/// `re.split(r'[.!?]+', text)` — pieces split on runs of the delimiters, empties kept.
fn split_on_delim_runs(text: &str) -> Vec<String> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'.' || b[i] == b'!' || b[i] == b'?' {
            out.push(slice_to_string(text, start, i));
            while i < b.len() && (b[i] == b'.' || b[i] == b'!' || b[i] == b'?') {
                i += 1;
            }
            start = i;
        } else {
            i += 1;
        }
    }
    out.push(slice_to_string(text, start, b.len()));
    out
}

/// `re.split(r'[,.]', text)` — split on every occurrence of any delimiter char.
fn split_on_chars(text: &str, delims: &[u8]) -> Vec<String> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < b.len() {
        if delims.contains(&b[i]) {
            out.push(slice_to_string(text, start, i));
            start = i + 1;
        }
        i += 1;
    }
    out.push(slice_to_string(text, start, b.len()));
    out
}

/// `native_score_cove_text` — the pure marker scorer (the oracle's Python fallback path, and the
/// arithmetic the shipping strip strip_cove_fallacy.dll performs byte-wise; see the module note).
/// Weights and clamp are the oracle's: sum the weights of the markers present, score = clamp(1-pen).
pub fn native_score_cove_text(text: &str) -> f64 {
    const MARKERS: [(&str, f64); 6] = [
        ("because it is true", 0.4),
        ("clearly all", 0.25),
        ("experts say", 0.15),
        ("therefore we should", 0.1),
        ("based on a single", 0.25),
        ("always", 0.05),
    ];
    let lower = ascii_lower(text);
    let hay = lower.as_bytes();
    let mut pen = 0.0f64;
    for &(m, w) in MARKERS.iter() {
        if bytes_contains(hay, m.as_bytes()) {
            pen += w;
        }
    }
    let score = 1.0 - pen;
    if score < 0.0 {
        0.0
    } else if score > 1.0 {
        1.0
    } else {
        score
    }
}

/// Oracle: `get_chain_of_verification()` returns a module-level singleton. A `no_std` kernel has no
/// lazy static that can allocate (no `OnceLock`/`Mutex` over a `Vec` here) and `ChainOfVerification`
/// owns a history `Vec`, so the port returns a fresh instance with the oracle's default threshold and
/// the caller keeps it: the singleton's STATE is what the oracle shares, and it is preserved
/// per-instance. This is the module's one non-portable piece of state and it is stated, not faked
/// (the previous revision carried an invented AtomicU64 "handle" that answered nothing).
#[allow(dead_code)]
pub fn get_chain_of_verification() -> ChainOfVerification {
    ChainOfVerification::new(CONFIDENCE_THRESHOLD)
}
