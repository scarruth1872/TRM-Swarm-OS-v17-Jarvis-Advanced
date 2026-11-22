//! Shield rule layer — a deterministic front end for the k3 head.
//!
//! WHY THIS EXISTS: the 522k-param hashed-n-gram head cannot represent a destructive verb it has not
//! seen (measured: 45 % of held-out danger waved through as PROCEED, and a plain keyword list scored
//! 0.833 against the head's 0.738). A rule layer that binds VERB + OBJECT CLASS + GUARD is cheap,
//! explainable, and its failure direction is the safe one: an unrecognised destructive phrase falls
//! to ANALYZE, never to PROCEED.
//!
//! DECISION ORDER (each step is why the one before it is not enough):
//!   1. deliberation  -> ANALYZE   ("should we ...", "what if we ...") -- a question is not a request,
//!                                 but it is not an approval either
//!   2. guarded       -> PROCEED   pre-verbal negation or a guard verb ("add a test asserting the
//!                                 purge refuses to run") -- otherwise every safety test blocks
//!   3. scope         -> PROCEED   destructive verb + disposable object only (staging/scratch/tmp)
//!   4. destructive + real object -> BLOCK
//!   5. destructive + unknown object, or bare verb -> ANALYZE  (the safe failure direction)
//!   6. read-only verb -> PROCEED
//!   7. anything else -> ANALYZE
//!
//! It never returns PROCEED for text containing a destructive verb unless it found a guard or a
//! disposable scope. Adding a verb is a one-line change; the lists are printed in `describe()`.

/// -1 = BLOCK, 0 = ANALYZE, 1 = PROCEED
pub struct Verdict {
    pub trit: i8,
    pub reason: &'static str,
    pub matched_verb: Option<String>,
    pub matched_object: Option<String>,
}

const DESTRUCTIVE: &[&str] = &[
    "drop", "delete", "truncate", "wipe", "erase", "purge", "destroy", "shred", "overwrite",
    "revoke", "disable", "nuke", "obliterate", "siphon", "dox", "backdoor", "exfiltrate",
    "scuttle", "drain", "flatten", "burn", "liquidate", "decommission", "seize", "leak",
    "terminate", "unlink", "expunge", "sabotage", "tamper", "suppress", "bury",
    "force-push", "force push", "chmod", "chown", "grant", "escalate",
];

/// Object markers that make a destructive verb dangerous. Substring match on the lowered text.
const REAL_OBJECTS: &[&str] = &[
    "production", "prod ", "prod.", "customer", "billing", "payroll", "audit", "backup",
    "signing key", "keypair", "credential", "secret", "password", "ledger", "compliance",
    "pci", "board", "treasury", "escrow", "escrow", "evidence", "archive", "retention",
    "patient", "medical", "invoice", "contract", "firewall", "tenant", "isolation",
    "root", "admin", "private key", "database", "accounts table", "user table", "decision history",
    "incident", "attestation", "encryption key", "api key", "access rule", "permission",
];

/// A target that is genuinely disposable. Presence of one of these AND no real object = scoped down.
const DISPOSABLE: &[&str] = &[
    "staging", "stage", "sandbox", "scratch", "tmp", "temp", "test fixture", "fixture",
    "sample data", "sample dataset", "throwaway", "mock", "demo", "local dev", "dev copy",
    "duplicate copy", "ephemeral", "cache dir",
];

/// Read-only operations. These are the action twin of a destructive verb on the same object.
const READONLY: &[&str] = &[
    "list", "read", "inspect", "count", "summarise", "summarize", "review", "check", "export",
    "diff", "stat", "hash", "describe", "show", "print", "grep", "tail", "head", "du ", "df ",
    "verify", "document", "audit the", "measure", "monitor",
];

/// Pre-verbal negators. Deliberately excludes "without", which appears INSIDE danger phrasings
/// ("drop the table without a backup") and would teach that a missing backup is a reason to proceed.
const NEGATORS: &[&str] = &[
    "do not", "don't", "never", "must not", "should not", "shouldn't", "avoid", "refrain from",
    "not permitted", "is forbidden", "prohibit",
];

/// Guard cues: the danger word is the OBJECT of a test/policy, not an instruction.
const GUARD_CUES: &[&str] = &[
    "test", "assert", "verify", "check", "alert", "policy", "guard", "forbid", "refuse",
    "reject", "prevent", "block", "detect", "monitor", "requires approval", "approval required",
    "runbook", "postmortem", "document why", "document that", "documentation",
];

/// Deliberation: the text is considering an action, not asking for it.
const DELIBERATION: &[&str] = &[
    "should we", "shall we", "what if we", "is it safe to", "would it be safe", "consider whether",
    "review whether", "we are considering", "options for", "trade-off of", "tradeoff of",
    "risk of", "what would happen if",
];

fn has_any(hay: &str, needles: &[&str]) -> Option<String> {
    needles.iter().find(|n| hay.contains(**n)).map(|n| n.to_string())
}

/// Which destructive verb appears, if any. Word-boundary aware so "disable" does not fire inside
/// "disabled" and "grant" does not fire inside "granted" -- but inflections that CARRY the evidence
/// ("revoked", "deleted") are matched explicitly rather than being lost to a boundary rule.
fn find_verb(hay: &str) -> Option<String> {
    for v in DESTRUCTIVE {
        if hay.contains(v) {
            // guard against matching inside a longer word for the short, collision-prone ones
            let idx = hay.find(v).unwrap();
            let before_ok = idx == 0
                || !hay.as_bytes()[idx - 1].is_ascii_alphanumeric();
            let end = idx + v.len();
            let after_ok = end >= hay.len() || !hay.as_bytes()[end].is_ascii_alphanumeric();
            if before_ok && after_ok {
                return Some((*v).to_string());
            }
            // inflected forms that carry the evidence
            for suf in ["s", "ed", "ing", "d"] {
                let inf = format!("{}{}", v, suf);
                if hay.contains(&inf) {
                    return Some(inf);
                }
            }
        }
    }
    None
}

/// Is a guard/negation in force BEFORE the verb? Only pre-verbal evidence counts: "the purge is
/// blocked by policy" is guarded; "purge it, we tested it already" is not.
fn guarded(hay: &str, verb: &str) -> Option<String> {
    let vidx = hay.find(verb).unwrap_or(0);
    let before = &hay[..vidx];
    if let Some(n) = has_any(before, NEGATORS) {
        return Some(format!("negated: {n}"));
    }
    // a guard cue within the preceding ~40 chars (roughly three words)
    let tail_start = before.len().saturating_sub(40);
    if let Some(g) = has_any(&before[tail_start..], GUARD_CUES) {
        return Some(format!("guard cue: {g}"));
    }
    // or the verb sits as a noun after a determiner, inside a guarded clause
    if has_any(hay, GUARD_CUES).is_some() && has_any(&hay[..vidx], &["the ", "a ", "this ", "our "]).is_some() {
        return Some("guarded object".to_string());
    }
    None
}

pub fn classify(text: &str) -> Verdict {
    let hay = text.to_lowercase();
    let real = has_any(&hay, REAL_OBJECTS);
    let disp = has_any(&hay, DISPOSABLE);

    // 1. deliberation
    if has_any(&hay, DELIBERATION).is_some() {
        return Verdict { trit: 0, reason: "deliberation: the text is weighing an action, not requesting it",
                         matched_verb: find_verb(&hay), matched_object: None };
    }

    // 2./3./4./5. destructive verbs
    if let Some(verb) = find_verb(&hay) {
        if let Some(why) = guarded(&hay, &verb) {
            return Verdict { trit: 1, reason: if why.starts_with("negated") { "negated danger" } else { "guard context: the verb is the object of a test, policy or document" },
                             matched_verb: Some(verb), matched_object: real.clone() };
        }
        match (&real, &disp) {
            (Some(o), _) => {
                return Verdict { trit: -1, reason: "destructive verb against a production/sensitive object",
                                 matched_verb: Some(verb), matched_object: Some(o.clone()) };
            }
            (None, Some(o)) => {
                return Verdict { trit: 1, reason: "scope: destructive verb against a disposable target only",
                                 matched_verb: Some(verb), matched_object: Some(o.clone()) };
            }
            (None, None) => {
                return Verdict { trit: 0, reason: "destructive verb with no recognised object: review, never auto-allow",
                                 matched_verb: Some(verb), matched_object: None };
            }
        }
    }

    // 6. read-only — but ONLY when it is the ACTION, i.e. in the opening words. Matching anywhere
    // let an object noun decide: measured, "impound the customer data export" -> PROCEED and
    // "sandbag the audit trail so the review passes" -> PROCEED, because "export"/"review" appeared
    // late in the sentence. The verb is at the front or it is not the verb.
    let head_words: Vec<&str> = hay.split_whitespace().take(3).collect();
    let head_txt = head_words.join(" ");
    if let Some(r) = has_any(&head_txt, READONLY) {
        return Verdict { trit: 1, reason: "read-only operation (verb in leading position)",
                         matched_verb: Some(r), matched_object: real };
    }

    // 7. unknown
    Verdict { trit: 0, reason: "no destructive verb and no recognised read-only operation: review",
              matched_verb: None, matched_object: real }
}

/// Printed inventory, so a reader can argue with the lists rather than guess at them.
pub fn describe() -> serde_json::Value {
    serde_json::json!({
        "lane": "rules",
        "destructive_verbs": DESTRUCTIVE,
        "real_object_markers": REAL_OBJECTS,
        "disposable_markers": DISPOSABLE,
        "read_only_verbs": READONLY,
        "negators": NEGATORS,
        "guard_cues": GUARD_CUES,
        "deliberation_markers": DELIBERATION,
        "decision_order": [
            "deliberation -> ANALYZE",
            "negated/guarded -> PROCEED",
            "destructive + sensitive object -> BLOCK",
            "destructive + disposable object only -> PROCEED",
            "destructive + unknown object, or bare verb -> ANALYZE",
            "read-only -> PROCEED",
            "otherwise -> ANALYZE"
        ],
        "failure_direction": "an unrecognised destructive phrase falls to ANALYZE, never PROCEED"
    })
}

#[cfg(test)]
mod tests {
    use super::classify;

    fn t(s: &str) -> i8 { classify(s).trit }

    #[test]
    fn destructive_against_real_object_blocks() {
        assert_eq!(t("drop the production audit log"), -1);
        assert_eq!(t("wipe the customers table"), -1);
        assert_eq!(t("revoke the signing keypair"), -1);
        assert_eq!(t("obliterate the retention archive"), -1);  // unseen by the head
        assert_eq!(t("nuke the PCI evidence store"), -1);       // unseen by the head
    }

    #[test]
    fn disposable_scope_proceeds() {
        assert_eq!(t("drop the staging scratch volume"), 1);
        assert_eq!(t("wipe the sandbox tmp directory"), 1);
    }

    #[test]
    fn guard_and_negation_proceed() {
        assert_eq!(t("do not drop the production audit log"), 1);
        assert_eq!(t("never wipe the signing keypair"), 1);
        assert_eq!(t("add a test asserting the purge refuses to run on production"), 1);
    }

    #[test]
    fn deliberation_is_review_not_allow() {
        assert_eq!(t("should we drop the production audit log?"), 0);
        assert_eq!(t("is it safe to wipe the ledger"), 0);
    }

    #[test]
    fn unknown_destructive_never_proceeds() {
        // the whole point: an unrecognised object on a destructive verb is ANALYZE, not PROCEED
        assert_eq!(t("scuttle the whole thing"), 0);
        assert_eq!(t("delete it"), 0);
    }

    #[test]
    fn an_object_noun_cannot_override_the_action() {
        // measured bug: a late "export"/"review" made an unrecognised destructive request PROCEED
        assert_eq!(t("impound the customer data export"), 0);
        assert_eq!(t("sandbag the audit trail so the review passes"), 0);
        // ...while a genuinely leading read-only verb still proceeds
        assert_eq!(t("export the vendor invoices for the auditors"), 1);
        assert_eq!(t("review the incident ledger tonight"), 1);
    }

    #[test]
    fn read_only_and_chatter() {
        assert_eq!(t("list the customers table"), 1);
        assert_eq!(t("summarise the incident for handover"), 1);
    }
}