"""Insert MOTION-011 (update rust crates / QSAR port) + vote arms into the council harness (CRLF)."""
import io

P = "src/main.rs"
src = io.open(P, encoding="utf-8", newline="").read()

MOTION = '''    Motion {
        id: "MOTION-011",
        title: "Update Rust Crates: Port the QSAR Antibacterial Predictor into swarm-drug",
        category: "Engineering / Execution",
        description: "The QSAR antibacterial-activity model (80-tree RandomForest, 1024-bit Morgan, 163,768 compounds, scaffold AUC 0.885, active under 10uM) is trained and verified in Python (swarm_v2/core/qsar_predictor.py, 74 MB joblib). Doctrine: Python is offline tooling, never in the runtime path. Port the decision into swarm-drug: export the 80 trees to a Rust-loadable binary, walk them natively, and verify the Rust prediction is bit-exact against Python on the control set. The Morgan fingerprint stays offline (RDKit, pre-computed per compound); the runtime decision is Rust.",
    },
'''

anchor_m = 'until a real caller exists.",\r\n    },\r\n];\r\n'
assert src.count(anchor_m) == 1, "motion anchor hits=%d" % src.count(anchor_m)
src = src.replace(anchor_m, 'until a real caller exists.",\r\n    },\r\n' + MOTION + '];\r\n')

ARMS = '''                ("MOTION-011", "Devo") => {
                    (1i8, 0.94, "Tree traversal is a bounded, contained port; export the 80 trees to a flat binary and walk them with no new dependency.")
                },
                ("MOTION-011", "Archi") => {
                    (1i8, 0.92, "One crate (swarm-drug) owns the drug decision; the narrow waist is preserved.")
                },
                ("MOTION-011", "Seeker") => {
                    (1i8, 0.95, "The antibacterial model is exactly the drug-knowledge the fabric should serve at runtime.")
                },
                ("MOTION-011", "Logic") => {
                    (1i8, 0.91, "Deterministic tree traversal over 1024-bit input is testable and falsifiable against the Python oracle.")
                },
                ("MOTION-011", "Pulse") => {
                    (1i8, 0.88, "Rust tree traversal latency is measurable; the Python joblib load is the removed cost.")
                },
                ("MOTION-011", "EventLedger") => {
                    (1i8, 0.93, "Record the model sha256 and the control-set predictions so the port is auditable.")
                },
                ("MOTION-011", "Bridge") => {
                    (1i8, 0.90, "One model artifact, one loader, one route; the drug path resolves to the crate.")
                },
                ("MOTION-011", "Shield") => {
                    (0i8, 0.87, "ABSTAIN unless the model binary is sha256-gated before load and the loader rejects anything that is not the exact registered artifact.")
                },
                ("MOTION-011", "Verify") => {
                    (0i8, 0.90, "ABSTAIN unless the Rust prediction is bit-exact against Python on the control set before the port is called verified.")
                },
                ("MOTION-011", "LogicMaster") => {
                    (0i8, 0.85, "ABSTAIN unless the port is stated as a machine-checkable claim; bit-exact control predictions is the check.")
                },
                ("MOTION-011", _) => {
                    (1i8, 0.86, "Bringing a verified drug model into the Rust runtime path is consistent with doctrine.")
                },
'''

anchor_a = '                _ => (0i8, 0.50, "Neutral."),'
assert src.count(anchor_a) == 1, "arms anchor hits=%d" % src.count(anchor_a)
src = src.replace(anchor_a, ARMS + anchor_a)

io.open(P, "w", encoding="utf-8", newline="").write(src)
print("inserted MOTION-011 + arms; MOTION arms:", src.count('("MOTION-01'), "| CRLF lines:", src.count("\r\n"))
