COUNCIL API ASYMMETRY - MEASURED 2026-11-03
============================================
This note exists because I got it wrong first. Writing it down so the next
reader does not repeat the mistake.

WHAT I CONCLUDED, AND WHY IT WAS WRONG
--------------------------------------
Probing both councils with a brand-new topic:

    POST :8022/council/propose -> {"approve":14,"abstain":0,"reject":0, ...}
    POST :8099/council/propose -> {"proposal_id":"CP-0708DD5B",
                                   "phase":"proposed", ...}

The Python response carries no tally, so I concluded :8099's propose was a
dead-end queue and that proposals were created and never decided. That was
wrong. I then polled /council/proposal/<id> for 26 seconds, saw phase stay
"proposed", and treated that as confirmation. It was confirmation of nothing:
the Python API is MULTI-STEP and simply waits for the next call.

THE ACTUAL PROTOCOL
-------------------
    :8022 (Rust)    one call, synchronous:
                    POST /council/propose  ->  full tally in the response

    :8099 (Python)  three calls, explicit phases:
                    POST /council/propose          -> phase "proposed"
                    POST /council/deliberate/<id>  -> phase "deliberating",
                                                      deliberation_count 14
                    POST /council/vote/<id>        -> phase "consensus",
                                                      raw_tally {14,0,0}
                    GET  /council/proposal/<id>    -> final state

Measured end to end on topic "parity probe ... batch ledger writes":
    deliberate -> 200, 14 deliberations
    vote       -> 200, consensus_reached true, raw_tally approve 14 / abstain 0
                  / reject 0
    final      -> phase "consensus"

Rust returned 14/0/0 for the same topic. They agree. There is no dead-end and
no dropped proposal.

WHY THIS MATTERS ANYWAY
-----------------------
The asymmetry is real and it has two practical consequences:

1. A client written against :8022 will appear to "get nothing" from :8099 and
   may conclude the council is broken. It is not; it is waiting.
2. The parity checker compares stored verdicts. On its first run it reported
   PARITY 6/7 - "memory usage keeps climbing" showed Rust (9,1,4) against
   Python (10,1,3) - and then 7/7 on two consecutive re-runs. That is not
   flakiness: the topic had been deliberated in an earlier session, Rust
   returned its stored verdict while Python recomputed, and once both had the
   same stored verdict they agreed. A parity check that reuses previously-seen
   topics therefore does not test independent computation. Use a fresh topic
   for a true parity test, and drive the Python side through all three calls.

STATUS
------
Not a defect. No fix required. Recorded because "does the council work" is a
release question and the answer depends on knowing which protocol you are
speaking.
