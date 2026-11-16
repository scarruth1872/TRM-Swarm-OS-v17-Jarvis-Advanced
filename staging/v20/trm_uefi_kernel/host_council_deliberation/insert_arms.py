"""Insert the MOTION-008/009/010 vote arms ahead of the catch-all, then normalize line endings."""
import io
import re

P = "src/main.rs"
src = io.open(P, encoding="utf-8", newline="").read()

ARMS = '''                ("MOTION-008", "Seeker") => {
                    (1i8, 0.96, "19,443 labelled protein-ligand affinities is exactly the exact-lookup shape this store exists for; the fabric answered prose, not a kd/ki.")
                },
                ("MOTION-008", "Bridge") => {
                    (1i8, 0.93, "One 10.9 MB CSV plus three target archives: routing it in costs one table and one provenance row.")
                },
                ("MOTION-008", "Shield") => {
                    (0i8, 0.90, "ABSTAIN: provenance must identify the artifact it names -- the recorded sha256 is a 200 MB prefix hash and cannot.")
                },
                ("MOTION-008", "Verify") => {
                    (0i8, 0.92, "ABSTAIN: record the full-file sha256 and a hash span before any row claims to be verified.")
                },
                ("MOTION-008", "Archi") => {
                    (0i8, 0.86, "ABSTAIN: a pdb_id taken from a filename is not an identifier; derive it from HEADER or store it as DERIVED with its parent.")
                },
                ("MOTION-008", "EventLedger") => {
                    (1i8, 0.95, "Re-record the span and re-seal: the ledger already names this gap.")
                },
                ("MOTION-008", "Devo") => {
                    (1i8, 0.91, "Extend build_structured_index.py so the span stays reproducible rather than hand-patched.")
                },
                ("MOTION-008", "Logic") => {
                    (1i8, 0.90, "Declaring tox21full out of span on the measurement -- a git checkout with no data files -- is a decision, not a dismissal.")
                },
                ("MOTION-008", _) => {
                    (1i8, 0.88, "Scope is measured and bounded: ingest what is a table, declare what is code, re-derive what was asserted.")
                },
                ("MOTION-009", "Seeker") => {
                    (1i8, 0.96, "93.1 percent of molecule rows cannot yield a structure; the store answers ids but not chemistry.")
                },
                ("MOTION-009", "Logic") => {
                    (1i8, 0.91, "A lookup that returns NULL for 2,697,822 of 2,897,819 rows is a partial answer presented as an answer.")
                },
                ("MOTION-009", "Devo") => {
                    (0i8, 0.88, "ABSTAIN: measure the pass rate on 5,000 records first -- rdkit is absent from the WSL interpreter the ingest runs under.")
                },
                ("MOTION-009", "Verify") => {
                    (0i8, 0.90, "ABSTAIN: every derived smiles must cite the source sha256 that produced it, or the column is unresolvable.")
                },
                ("MOTION-009", "Pulse") => {
                    (1i8, 0.85, "Bounded by disk and a measured parse rate; no runtime path is touched.")
                },
                ("MOTION-009", "Shield") => {
                    (1i8, 0.83, "Silently NULL structures change docking outcomes without signalling, which is a correctness risk.")
                },
                ("MOTION-009", _) => {
                    (1i8, 0.87, "Close the gap at a measured rate, or declare the 6.9 percent span in the index own doc.")
                },
                ("MOTION-010", "Archi") => {
                    (0i8, 0.91, "ABSTAIN: a SQLite binding inside the kernel is a dependency the narrow waist does not need.")
                },
                ("MOTION-010", "Devo") => {
                    (0i8, 0.86, "ABSTAIN: no live consumer calls for a route today; wire the seam when a task path does.")
                },
                ("MOTION-010", "Bridge") => {
                    (1i8, 0.92, "A resolver seam keeps one owner per store and lets both the Python and the Rust paths call the same lookup.")
                },
                ("MOTION-010", "Verify") => {
                    (1i8, 0.89, "Declaring offline tooling WITH its consumers named is honest; an unreachable store presented as platform knowledge is not.")
                },
                ("MOTION-010", "Logic") => {
                    (1i8, 0.86, "The boundary is already measured: the fabric for prose, an exact store for exact lookups.")
                },
                ("MOTION-010", _) => {
                    (1i8, 0.84, "Document the consumers and keep the store offline until a task path needs it.")
                },
'''

anchor = '                _ => (0i8, 0.50, "Neutral."),'
assert src.count(anchor) == 1, "arms anchor hits=%d" % src.count(anchor)
src = src.replace(anchor, ARMS + anchor)

# One line-ending convention for the whole file (it was CRLF before this edit).
src = re.sub(r"\r?\n", "\r\n", src)
io.open(P, "w", encoding="utf-8", newline="").write(src)
print("inserted arms; MOTION arms:", src.count('("MOTION-00'), "| CRLF lines:", src.count("\r\n"))
