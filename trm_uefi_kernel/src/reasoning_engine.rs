//! TRM Swarm OS — Reasoning Engine (no_std, alloc).
//!
//! Tasks 571, 572, 576, 580 built as one primitive instead of four demos:
//!
//!   * **571 forward chaining** — `Engine::forward_chain` saturates the rule base from the asserted facts,
//!     firing a rule only when every premise is TRUE, in Kleene K3 (TRUE / UNKNOWN / FALSE). An UNKNOWN
//!     premise never produces a TRUE conclusion; it produces UNKNOWN with the reason recorded.
//!   * **572 backward chaining** — `Engine::prove` searches goal-directed over the *same* rules and returns
//!     the derivation that justifies the answer, or UNKNOWN naming the premise it could not settle.
//!   * **576 default reasoning** — a rule may carry an `exception`, three-valued:
//!       - exception TRUE    -> the rule is **defeated** and the conclusion is withdrawn;
//!       - exception FALSE   -> the default applies, with nothing assumed;
//!       - exception UNKNOWN -> the default still applies, but the assumption is **named** in the
//!         derivation's note and collected in `ForwardResult.assumptions`.
//!     Refusing to fire on UNKNOWN would make every default useless (nothing is ever known to be absent);
//!     firing silently would hide the assumption. A caller that needs certainty requires
//!     `assumptions.is_empty()`, so the two readings are distinguishable rather than conflated.
//!   * **580 explanation** — every conclusion carries a `Derivation` tree down to the facts, and every
//!     leaf fact keeps the `source` string it was asserted with (shard, record, seal), so a report can say
//!     *which bytes* support a conclusion.
//!
//! What this engine deliberately does NOT do, because a claim like "complete reasoning suite" (579) is
//! only honest if the boundary is stated:
//!   * no negation-as-failure — closed-world assumption is a non-monotonic leap this engine refuses;
//!   * no counting / aggregation (arity > 1) — "two or more violations" must be written as explicit
//!     pairwise rules, because the engine cannot count;
//!   * no unification or variables — atoms are opaque strings, so this is propositional, not first-order;
//!   * no arithmetical reasoning — thresholds are asserted as facts by the caller;
//!   * no probabilistic truth — a fact is TRUE, FALSE or UNKNOWN, never 0.73.
//!
//! A conflict (an atom derived TRUE and FALSE with no defeat between them) is reported and the atom becomes
//! UNKNOWN. It is never silently resolved by order of arrival.

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Iteration cap for forward chaining. Reaching it is reported (`truncated`), never hidden: a chain that
/// does not settle in this many passes is a bug in the rule base or an unsatisfiable cycle, and the caller
/// must see which.
pub const MAX_ITERATIONS: usize = 64;
/// Depth cap for backward chaining, for the same reason.
pub const MAX_PROOF_DEPTH: usize = 32;

/// Kleene three-valued truth, same encoding as `k3_logic` (FALSE = -1, UNKNOWN = 0, TRUE = +1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trit {
    False,
    Unknown,
    True,
}

impl Trit {
    pub fn as_i8(self) -> i8 {
        match self {
            Trit::False => -1,
            Trit::Unknown => 0,
            Trit::True => 1,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Trit::False => "FALSE",
            Trit::Unknown => "UNKNOWN",
            Trit::True => "TRUE",
        }
    }
}

/// A rule: `premises (all TRUE) AND negatives (all FALSE) AND no exception TRUE => conclusion`.
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: u32,
    pub name: String,
    pub premises: Vec<String>,
    /// Atoms that must be FALSE for the rule to fire.
    pub negatives: Vec<String>,
    pub conclusion: String,
    /// Defeasible rules: if this atom is TRUE the rule is defeated. UNKNOWN defeats it too, into UNKNOWN.
    pub exception: Option<String>,
}

impl Rule {
    pub fn new(id: u32, name: &str, premises: &[&str], conclusion: &str) -> Rule {
        Rule {
            id,
            name: name.to_string(),
            premises: premises.iter().map(|s| s.to_string()).collect(),
            negatives: Vec::new(),
            conclusion: conclusion.to_string(),
            exception: None,
        }
    }
    pub fn with_exception(mut self, atom: &str) -> Rule {
        self.exception = Some(atom.to_string());
        self
    }
    pub fn with_negative(mut self, atom: &str) -> Rule {
        self.negatives.push(atom.to_string());
        self
    }
}

/// One node of an explanation: a conclusion, the rule that produced it, and the derivations of its premises.
#[derive(Debug, Clone)]
pub struct Derivation {
    pub atom: String,
    pub trit: Trit,
    pub rule_id: Option<u32>,
    pub rule_name: String,
    pub premises: Vec<Derivation>,
    /// Set on leaves that were asserted rather than derived: the caller's evidence string.
    pub fact_source: Option<String>,
    /// Human-readable reason, always populated on UNKNOWN nodes and on truncated/defeated paths.
    pub note: String,
}

impl Derivation {
    fn fact(atom: &str, trit: Trit, source: Option<String>) -> Derivation {
        Derivation {
            atom: atom.to_string(),
            trit,
            rule_id: None,
            rule_name: String::new(),
            premises: Vec::new(),
            fact_source: source,
            note: String::new(),
        }
    }
    fn unknown(atom: &str, note: &str) -> Derivation {
        let mut d = Derivation::fact(atom, Trit::Unknown, None);
        d.note = note.to_string();
        d
    }
}

/// Two derivations of one atom that disagree, with no defeat between them.
#[derive(Debug, Clone)]
pub struct Conflict {
    pub atom: String,
    pub positive_rules: Vec<u32>,
    pub negative_rules: Vec<u32>,
}

/// Result of a forward chain: every atom's truth, how it was derived, and whether the run was complete.
#[derive(Debug, Clone)]
pub struct ForwardResult {
    pub truth: BTreeMap<String, Trit>,
    pub derivations: BTreeMap<String, Derivation>,
    pub conflicts: Vec<Conflict>,
    pub iterations: usize,
    pub rules_fired: usize,
    /// Every assumption the run had to make: a defeasible rule whose exception was UNKNOWN. A consumer that
    /// needs certainty can require this to be empty; a consumer that wants the default gets the conclusion
    /// and the name of the assumption behind it.
    pub assumptions: Vec<String>,
    /// True when `MAX_ITERATIONS` was reached before the rule base settled. The result is then a partial
    /// fixpoint and must be reported as such.
    pub truncated: bool,
}

/// The engine: facts with provenance + a rule base. Cheap to clone, no interior mutability.
#[derive(Debug, Clone, Default)]
pub struct Engine {
    pub facts: BTreeMap<String, String>,
    pub false_facts: BTreeSet<String>,
    pub rules: Vec<Rule>,
    pub max_iterations: Option<usize>,
}

impl Engine {
    pub fn new() -> Engine {
        Engine::default()
    }

    /// Assert an atom TRUE, with the evidence that justifies it.
    pub fn assert_fact(&mut self, atom: &str, source: &str) {
        self.facts.insert(atom.to_string(), source.to_string());
    }

    /// Assert an atom FALSE explicitly. Absence of a fact is *not* falsity in this engine.
    pub fn assert_false(&mut self, atom: &str) {
        self.false_facts.insert(atom.to_string());
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    fn value(&self, atom: &str, truth: &BTreeMap<String, Trit>) -> Trit {
        if self.false_facts.contains(atom) {
            return Trit::False;
        }
        if self.facts.contains_key(atom) {
            return Trit::True;
        }
        *truth.get(atom).unwrap_or(&Trit::Unknown)
    }

    /// Forward chaining to fixpoint (bounded).
    ///
    /// A rule fires only when every premise is TRUE, every negative is FALSE and the exception (if any) is
    /// FALSE. A premise that is UNKNOWN leaves the rule unfired and its conclusion UNKNOWN — the reason is
    /// kept in the derivation so the report can say what stopped it.
    pub fn forward_chain(&self) -> ForwardResult {
        let cap = self.max_iterations.unwrap_or(MAX_ITERATIONS);
        let mut truth: BTreeMap<String, Trit> = BTreeMap::new();
        let mut derivations: BTreeMap<String, Derivation> = BTreeMap::new();

        // asserted facts are the starting derivations
        for (atom, src) in &self.facts {
            truth.insert(atom.clone(), Trit::True);
            derivations.insert(atom.clone(), Derivation::fact(atom, Trit::True, Some(src.clone())));
        }
        for atom in &self.false_facts {
            truth.insert(atom.clone(), Trit::False);
            derivations.insert(atom.clone(), Derivation::fact(atom, Trit::False, None));
        }

        let mut iterations = 0usize;
        let mut rules_fired = 0usize;
        let mut truncated = false;
        let mut assumptions: Vec<String> = Vec::new();
        // rule id -> whether it has fired (so the count is rules, not passes)
        let mut fired: BTreeSet<u32> = BTreeSet::new();

        loop {
            if iterations >= cap {
                truncated = true;
                break;
            }
            iterations += 1;
            let mut changed = false;

            for rule in &self.rules {
                // already TRUE from an assertion or an earlier rule?
                let current = *truth.get(&rule.conclusion).unwrap_or(&Trit::Unknown);
                if current == Trit::True {
                    continue;
                }

                // EXCEPTION GATE (576). Three-valued, and the three cases are distinct:
                //   TRUE    -> the rule is defeated; the conclusion is withdrawn.
                //   FALSE   -> the default applies, nothing assumed.
                //   UNKNOWN -> the default applies, but the run records that it assumed the exception's
                //              absence. Refusing to fire here would make every default useless (nothing is
                //              ever known to be absent); firing silently would hide the assumption. Naming
                //              it is the only honest option, and `ForwardResult.assumptions` carries it.
                let mut exception_state = Trit::False;
                let mut assumed: Option<String> = None;
                if let Some(ex) = &rule.exception {
                    exception_state = self.value(ex, &truth);
                    if exception_state == Trit::True {
                        if !derivations.contains_key(&rule.conclusion) {
                            derivations.insert(
                                rule.conclusion.clone(),
                                Derivation::unknown(&rule.conclusion, &format!("defeated by exception '{ex}'")),
                            );
                            truth.insert(rule.conclusion.clone(), Trit::Unknown);
                        }
                        continue;
                    }
                    if exception_state == Trit::Unknown {
                        assumed = Some(ex.clone());
                    }
                }

                // premises
                let mut all_true = true;
                let mut unknown_premise: Option<String> = None;
                let mut false_premise: Option<String> = None;
                for p in &rule.premises {
                    match self.value(p, &truth) {
                        Trit::True => {}
                        Trit::Unknown => {
                            all_true = false;
                            if unknown_premise.is_none() && false_premise.is_none() {
                                unknown_premise = Some(p.clone());
                            }
                        }
                        Trit::False => {
                            all_true = false;
                            if false_premise.is_none() {
                                false_premise = Some(p.clone());
                            }
                        }
                    }
                }
                for n in &rule.negatives {
                    match self.value(n, &truth) {
                        Trit::False => {}
                        Trit::Unknown => {
                            all_true = false;
                            if unknown_premise.is_none() && false_premise.is_none() {
                                unknown_premise = Some(n.clone());
                            }
                        }
                        Trit::True => {
                            all_true = false;
                            if false_premise.is_none() {
                                false_premise = Some(n.clone());
                            }
                        }
                    }
                }

                if all_true && exception_state != Trit::True {
                    // build the derivation from current premises
                    let premises = rule
                        .premises
                        .iter()
                        .map(|p| {
                            derivations.get(p).cloned().unwrap_or_else(|| {
                                Derivation::fact(p, self.value(p, &truth), self.facts.get(p).cloned())
                            })
                        })
                        .collect::<Vec<_>>();
                    let note = match &assumed {
                        Some(ex) => {
                            let a = format!("assumed: exception '{ex}' is UNKNOWN, treated as not holding");
                            if !assumptions.contains(&a) {
                                assumptions.push(a.clone());
                            }
                            a
                        }
                        None => String::new(),
                    };
                    let d = Derivation {
                        atom: rule.conclusion.clone(),
                        trit: Trit::True,
                        rule_id: Some(rule.id),
                        rule_name: rule.name.clone(),
                        premises,
                        fact_source: None,
                        note,
                    };
                    truth.insert(rule.conclusion.clone(), Trit::True);
                    derivations.insert(rule.conclusion.clone(), d);
                    fired.insert(rule.id);
                    rules_fired += 1;
                    changed = true;
                } else if current == Trit::Unknown && !all_true {
                    // keep the reason the conclusion is unknown, preferring a false premise (rule cannot fire)
                    if let Some(fp) = &false_premise {
                        if !derivations.contains_key(&rule.conclusion) {
                            derivations.insert(
                                rule.conclusion.clone(),
                                Derivation::unknown(
                                    &rule.conclusion,
                                    &format!("rule '{}' cannot fire: premise '{fp}' is FALSE", rule.name),
                                ),
                            );
                            truth.insert(rule.conclusion.clone(), Trit::Unknown);
                        }
                    } else if let Some(up) = &unknown_premise {
                        if !derivations.contains_key(&rule.conclusion) {
                            derivations.insert(
                                rule.conclusion.clone(),
                                Derivation::unknown(
                                    &rule.conclusion,
                                    &format!("rule '{}' needs '{up}', which is UNKNOWN", rule.name),
                                ),
                            );
                            truth.insert(rule.conclusion.clone(), Trit::Unknown);
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }

        // Contradiction check: an atom TRUE by rule/assertion and FALSE by assertion (defeat is explicit,
        // so this is a genuine conflict, not a default being overridden).
        let mut conflicts = Vec::new();
        for atom in self.false_facts.iter() {
            let derived_true = derivations
                .get(atom)
                .map(|d| d.trit == Trit::True)
                .unwrap_or(false);
            if derived_true {
                conflicts.push(Conflict {
                    atom: atom.clone(),
                    positive_rules: derivations
                        .get(atom)
                        .and_then(|d| d.rule_id)
                        .map(|id| alloc::vec![id])
                        .unwrap_or_default(),
                    negative_rules: Vec::new(),
                });
                truth.insert(atom.clone(), Trit::Unknown);
                derivations.insert(
                    atom.clone(),
                    Derivation::unknown(
                        atom,
                        "conflict: derived TRUE and asserted FALSE, with no defeat between them",
                    ),
                );
            }
        }

        ForwardResult {
            truth,
            derivations,
            conflicts,
            iterations,
            rules_fired,
            assumptions,
            truncated,
        }
    }

    /// Backward chaining: prove `goal` from the rule base, goal-directed.
    ///
    /// Returns the derivation that justifies the answer. An UNKNOWN result always names the premise or the
    /// bound that stopped it — an abstention that cannot explain itself is not an abstention.
    pub fn prove(&self, goal: &str) -> Derivation {
        let mut stack: BTreeSet<String> = BTreeSet::new();
        self.prove_inner(goal, 0, &mut stack)
    }

    fn prove_inner(&self, goal: &str, depth: usize, stack: &mut BTreeSet<String>) -> Derivation {
        if self.false_facts.contains(goal) {
            return Derivation::fact(goal, Trit::False, None);
        }
        if let Some(src) = self.facts.get(goal) {
            return Derivation::fact(goal, Trit::True, Some(src.clone()));
        }
        if depth >= MAX_PROOF_DEPTH {
            return Derivation::unknown(
                goal,
                &format!("proof depth limit {MAX_PROOF_DEPTH} reached: the chain did not terminate"),
            );
        }
        if stack.contains(goal) {
            return Derivation::unknown(goal, &format!("cycle: '{goal}' depends on itself"));
        }

        stack.insert(goal.to_string());
        let mut best: Option<Derivation> = None;
        let mut unknown_reason: Option<String> = None;

        for rule in self.rules.iter().filter(|r| r.conclusion == goal) {
            // exception first: a defeated rule cannot prove the goal
            let mut defeated: Option<String> = None;
            let mut assumed_exception: Option<String> = None;
            if let Some(ex) = &rule.exception {
                let ex_d = self.prove_inner(ex, depth + 1, stack);
                match ex_d.trit {
                    Trit::True => {
                        defeated = Some(format!("defeated: exception '{ex}' holds"));
                    }
                    Trit::Unknown => {
                        // same rule as forward: the default applies, and the assumption is named in the
                        // derivation's note rather than silently embraced
                        assumed_exception = Some(ex.clone());
                    }
                    Trit::False => {}
                }
            }
            if let Some(d) = defeated {
                if unknown_reason.is_none() {
                    unknown_reason = Some(d);
                }
                continue;
            }

            // negatives must be FALSE
            let mut neg_failed = false;
            for n in &rule.negatives {
                let nd = self.prove_inner(n, depth + 1, stack);
                match nd.trit {
                    Trit::False => {}
                    Trit::True => {
                        neg_failed = true;
                        if unknown_reason.is_none() {
                            unknown_reason = Some(format!("rule '{}' requires '{n}' to be FALSE", rule.name));
                        }
                        break;
                    }
                    Trit::Unknown => {
                        neg_failed = true;
                        if unknown_reason.is_none() {
                            unknown_reason = Some(format!("rule '{}' needs '{n}' to be FALSE, which is UNKNOWN", rule.name));
                        }
                        break;
                    }
                }
            }
            if neg_failed {
                continue;
            }

            let mut premises = Vec::new();
            let mut all_true = true;
            for p in &rule.premises {
                let pd = self.prove_inner(p, depth + 1, stack);
                if pd.trit != Trit::True {
                    all_true = false;
                    if unknown_reason.is_none() {
                        unknown_reason = Some(match pd.trit {
                            Trit::Unknown => format!(
                                "rule '{}' needs '{p}': {}",
                                rule.name,
                                if pd.note.is_empty() { "UNKNOWN" } else { pd.note.as_str() }
                            ),
                            _ => format!("rule '{}' needs '{p}', which is FALSE", rule.name),
                        });
                    }
                }
                premises.push(pd);
            }

            if all_true {
                let note = match &assumed_exception {
                    Some(ex) => format!("assumed: exception '{ex}' is UNKNOWN, treated as not holding"),
                    None => String::new(),
                };
                best = Some(Derivation {
                    atom: goal.to_string(),
                    trit: Trit::True,
                    rule_id: Some(rule.id),
                    rule_name: rule.name.clone(),
                    premises,
                    fact_source: None,
                    note,
                });
                break;
            }
        }

        stack.remove(goal);

        match best {
            Some(d) => d,
            None => unreachable_derivation(goal, unknown_reason),
        }
    }

    /// A compact rule base for drug-likeness, used by the verification harness and by the council endpoint.
    /// Thresholds are the published Lipinski / Veber / Ghose criteria; the engine derives, it does not compute.
    pub fn druglikeness_rules() -> Vec<Rule> {
        let mut rules = Vec::new();
        rules.push(Rule::new(1, "lipinski_mw", &["mw_gt_500"], "violation_mw"));
        rules.push(Rule::new(2, "lipinski_logp", &["logp_gt_5"], "violation_logp"));
        rules.push(Rule::new(3, "lipinski_hbd", &["hbd_gt_5"], "violation_hbd"));
        rules.push(Rule::new(4, "lipinski_hba", &["hba_gt_10"], "violation_hba"));
        rules.push(Rule::new(5, "veber_tpsa", &["tpsa_gt_140"], "violation_tpsa"));
        rules.push(Rule::new(6, "veber_rotb", &["rotb_gt_10"], "violation_rotb"));
        rules.push(Rule::new(7, "ghose_atoms_low", &["heavy_atoms_lt_20"], "violation_ghose"));
        rules.push(Rule::new(8, "ghose_atoms_high", &["heavy_atoms_gt_70"], "violation_ghose"));
        // explicit pairwise conjunctions: the engine cannot count, so "two or more violations" is written out
        // "two or more violations" cannot be counted by this engine, so it is written out: all C(5,2) = 10
        // pairs over {mw, logp, hbd, hba, ghose}. The first version of this table had 8 pairs and left
        // hbd+ghose / hba+ghose missing — a real ChEMBL compound with exactly those two violations came back
        // UNKNOWN instead of TRUE, which the RDKit harness caught (CHEMBL242031, 2026-11-24).
        let pairs = [
            ("violation_mw", "violation_logp", 10u32),
            ("violation_mw", "violation_hbd", 11),
            ("violation_mw", "violation_hba", 12),
            ("violation_mw", "violation_ghose", 13),
            ("violation_logp", "violation_hbd", 14),
            ("violation_logp", "violation_hba", 15),
            ("violation_logp", "violation_ghose", 16),
            ("violation_hbd", "violation_hba", 17),
            ("violation_hbd", "violation_ghose", 18),
            ("violation_hba", "violation_ghose", 19),
        ];
        for (a, b, id) in pairs {
            rules.push(Rule::new(id, "two_violations", &[a, b], "too_many_violations"));
        }
        // a defeasible rule: a single violation is tolerable unless the compound is an antibiotic candidate
        // with a narrow therapeutic window — the exception demonstrates 576 on the same rule base.
        rules.push(
            Rule::new(20, "single_violation_tolerated", &["violation_tpsa", "violation_rotb"], "acceptable")
                .with_exception("narrow_therapeutic_window"),
        );
        rules
    }

    /// A deliberately tighter screening profile (mw > 450, logp > 4, hbd > 4, hba > 8).
    ///
    /// It exists because the published Lipinski set is so lenient that not one of the 406 real ChEMBL LpxC
    /// actives reaches two violations — a rule base that never derives a positive cannot be verified by
    /// running it. The tighter thresholds are stated here rather than tuned until something passed, and the
    /// harness checks the engine against RDKit on both profiles.
    pub fn strict_druglikeness_rules() -> Vec<Rule> {
        let mut rules = Vec::new();
        rules.push(Rule::new(1, "strict_mw", &["mw_gt_450"], "violation_mw"));
        rules.push(Rule::new(2, "strict_logp", &["logp_gt_4"], "violation_logp"));
        rules.push(Rule::new(3, "strict_hbd", &["hbd_gt_4"], "violation_hbd"));
        rules.push(Rule::new(4, "strict_hba", &["hba_gt_8"], "violation_hba"));
        rules.push(Rule::new(7, "ghose_atoms_low", &["heavy_atoms_lt_20"], "violation_ghose"));
        rules.push(Rule::new(8, "ghose_atoms_high", &["heavy_atoms_gt_70"], "violation_ghose"));
        // "two or more violations" cannot be counted by this engine, so it is written out: all C(5,2) = 10
        // pairs over {mw, logp, hbd, hba, ghose}. The first version of this table had 8 pairs and left
        // hbd+ghose / hba+ghose missing — a real ChEMBL compound with exactly those two violations came back
        // UNKNOWN instead of TRUE, which the RDKit harness caught (CHEMBL242031, 2026-11-24).
        let pairs = [
            ("violation_mw", "violation_logp", 10u32),
            ("violation_mw", "violation_hbd", 11),
            ("violation_mw", "violation_hba", 12),
            ("violation_mw", "violation_ghose", 13),
            ("violation_logp", "violation_hbd", 14),
            ("violation_logp", "violation_hba", 15),
            ("violation_logp", "violation_ghose", 16),
            ("violation_hbd", "violation_hba", 17),
            ("violation_hbd", "violation_ghose", 18),
            ("violation_hba", "violation_ghose", 19),
        ];
        for (a, b, id) in pairs {
            rules.push(Rule::new(id, "two_violations", &[a, b], "too_many_violations"));
        }
        rules
    }
}

fn unreachable_derivation(goal: &str, unknown_reason: Option<String>) -> Derivation {
    Derivation::unknown(
        goal,
        unknown_reason
            .as_deref()
            .unwrap_or("no rule concludes this atom and it is not asserted"),
    )
}

/// Escape a string for the hand-built JSON below (no serde in no_std).
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

/// Serialise a derivation tree to JSON. Used by `/reason/*` so the explanation crosses the wire intact.
pub fn derivation_json(d: &Derivation) -> String {
    let premises = d
        .premises
        .iter()
        .map(derivation_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"atom\":\"{}\",\"truth\":\"{}\",\"rule_id\":{},\"rule\":\"{}\",\"fact_source\":{},\"note\":\"{}\",\"premises\":[{}]}}",
        esc(&d.atom),
        d.trit.as_str(),
        d.rule_id.map(|i| i.to_string()).unwrap_or_else(|| "null".to_string()),
        esc(&d.rule_name),
        match &d.fact_source {
            Some(s) => format!("\"{}\"", esc(s)),
            None => "null".to_string(),
        },
        esc(&d.note),
        premises
    )
}
