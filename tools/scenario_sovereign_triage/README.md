# SCENARIO: SOVEREIGN TRIAGE (function over fashion)

ONE end-to-end pipeline. Every stage must prove it WORKS by also proving it
FAILS CORRECTLY when it should. A stage that only produces data is fashion.

STAGES (all against the live platform, port 8022, and real ChEMBL ground truth):
  0 VOLTAGE   gate must REFUSE on an unmeasured/cold substrate
  1 RETRIEVE  nearest known compound, fp32 K40 kept, 1.58-bit as index only
  2 DOCK      real scoring, and it must DISCRIMINATE (blind ground truth)
  3 DECIDE    K3 ternary; UNKNOWN must BLOCK, never approve
  4 AUDIT     DDR must CATCH a real vuln AND clear a benign lookalike
  5 REPORT    must state its own failures

PASS requires every stage to behave correctly IN BOTH DIRECTIONS.
