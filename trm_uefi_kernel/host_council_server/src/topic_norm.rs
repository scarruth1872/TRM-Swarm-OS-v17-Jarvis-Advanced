//! Topic normalisation — remove address and performativity, keep the operation.
//!
//! WHY (G49, measured)
//! -------------------
//! The council's verdict changed with the leading clause of an identical request:
//!
//!     "cover the watchdog with a regression test"                  -> CONSENSUS  A12/R1  (x2)
//!     "Ask the council whether we should cover the watchdog ..."   -> REJECTED   A4/R9   (x2)
//!
//! Both were stable across repeats, so the difference was judgement, not noise. The prefix
//! adds no operational content: it says who is being asked, not what would be done. Stripping
//! it before retrieval and voting is therefore the honest repair — normalise the input, do not
//! tune the vote.
//!
//! WHAT IS STRIPPED
//! ----------------
//! Only leading, semantically empty framing:
//!   * address frames        "ask the council whether we should", "consult the council on"
//!   * performatives         "proposal:", "I want to", "let's"
//!   * modal openers         "should we", "can we", "we should"
//!   * discourse fillers     "uh,", "um,", "ok so,"  (leading only)
//! Matching is case-insensitive, applied repeatedly (a sentence can stack two frames), and
//! bounded so a pathological string cannot loop.
//!
//! WHAT IS NEVER STRIPPED
//! ---------------------
//! Anything carrying the operation: verbs, objects, negations, conditions. "without auth" and
//! "for now" survive intact, so a dangerous request cannot be softened by its wrapper. If
//! stripping would leave nothing, the original text is returned unchanged — the gate must never
//! see an empty string where a real directive was given.

const MAX_PASSES: usize = 4;

/// Leading frames that name the *addressing* of a request rather than its content.
const LEAD_FRAMES: &[&str] = &[
    "ask the council whether we should ",
    "ask the council whether i should ",
    "ask the council whether ",
    "ask the council if we should ",
    "ask the council if i should ",
    "ask the council if ",
    "ask the council about ",
    "ask the council to decide whether ",
    "ask the council ",
    "consult the council on whether we should ",
    "consult the council on whether ",
    "consult the council on ",
    "consult the council about ",
    "take this to the council: ",
    "bring this to the council: ",
    "put it to the council: ",
    "council vote on ",
    "council decision on ",
    "proposal: ",
    "propose that we ",
    "i want to ",
    "i would like to ",
    "we want to ",
    "we would like to ",
    "i think we should ",
    "i think we can ",
    "we should ",
    "should we ",
    "can we ",
    "could we ",
    "please, ",
    "please ",
    "let's ",
    "lets ",
    // discourse fillers, leading only
    "uh, ",
    "um, ",
    "ok so, ",
    "ok, ",
    "so, ",
];

/// Trailing punctuation carries no operational content either.
fn strip_tail(s: &str) -> &str {
    s.trim_end_matches(['.', '!', '?']).trim_end()
}

/// Normalise a directive for retrieval and voting.
///
/// Returns the operation with address/performativity removed. If the result would be empty or
/// shorter than a plausible directive, the trimmed original is returned instead.
pub fn normalise(raw: &str) -> String {
    let original = strip_tail(raw.trim()).to_string();
    if original.is_empty() {
        return original;
    }
    let mut cur = original.clone();
    for _ in 0..MAX_PASSES {
        let lower = cur.to_lowercase();
        let mut hit = None;
        for f in LEAD_FRAMES {
            if lower.starts_with(f) {
                // Longest match wins: the list is scanned in order, and the more specific
                // "ask the council whether we should" precedes "ask the council ".
                hit = Some(f.len());
                break;
            }
        }
        match hit {
            Some(n) => {
                let rest = strip_tail(cur[n..].trim()).to_string();
                if rest.is_empty() {
                    break;
                }
                cur = rest;
            }
            None => break,
        }
    }
    if cur.len() < 3 {
        original
    } else {
        cur
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_frame_is_removed() {
        assert_eq!(
            normalise("Ask the council whether we should cover the watchdog with a regression test."),
            "cover the watchdog with a regression test"
        );
    }

    #[test]
    fn bare_directive_is_unchanged() {
        assert_eq!(
            normalise("cover the watchdog with a regression test"),
            "cover the watchdog with a regression test"
        );
    }

    #[test]
    fn stacked_frames_are_all_removed() {
        assert_eq!(
            normalise("Please ask the council if we should add a regression test"),
            "add a regression test"
        );
    }

    #[test]
    fn danger_survives_the_strip() {
        // The wrapper must not soften anything: "without auth" is the harm.
        assert_eq!(
            normalise("Can we ship the API without auth for now?"),
            "ship the API without auth for now"
        );
        assert_eq!(
            normalise("I think we can delete the student checkpoints before the ARC run finishes"),
            "delete the student checkpoints before the ARC run finishes"
        );
    }

    #[test]
    fn frame_only_text_is_returned_unchanged() {
        // Never hand an empty string to the gates when a real directive was given.
        let s = "Ask the council";
        assert_eq!(normalise(s), s);
    }

    #[test]
    fn case_and_tail_punctuation_are_normalised() {
        assert_eq!(normalise("  PROPOSAL: back up the ledger  "), "back up the ledger");
        assert_eq!(normalise("uh, tighten the firewall rules!"), "tighten the firewall rules");
    }
}
