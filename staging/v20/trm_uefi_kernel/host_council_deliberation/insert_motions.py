"""Insert MOTION-008/009/010 into the council harness, anchored and asserted (the file is CRLF)."""
import io

P = "src/main.rs"
src = io.open(P, encoding="utf-8", newline="").read()

MOTIONS = '''    Motion {
        id: "MOTION-008",
        title: "Structured Index Span: Ingest LP-PDBBind, Re-derive Provenance, Declare the Rest",
        category: "Data / Integrity",
        description: "Measured 2026-09-26 against swarm_v2_artifacts/structured_index.db (1,099,448,320 B): molecule 2,897,819 / clinical 201,572 / ethics 95,847 / prose 1,633 / structure 9 / provenance 422 rows. Gaps the measurement names: datasets/LP-PDBBind/dataset/LP_PDBBind.csv (10,897,207 B; 19,443 labelled protein-ligand rows carrying smiles, seq, kd/ki and value) has ZERO rows; 11 .pdbqt ligands and receptor.pdbqt are unindexed; provenance.sha256 for drugs/chembl_37.sdf.gz is the builder cap=200MB PREFIX hash (348d9f31...) and does NOT match the release checksum for that file (2cdf4d33c5f42613..., re-measured, gzip -t OK); structure.pdb_id is taken from the filename, so receptor_clean.pdb became RECE and 1M17_prot.pdb collides with 1M17; tox21full is a 352 KB git checkout of a Python package with no data files and belongs out of span; the docs still state 507 MB. Proposal: extend build_structured_index.py with the LP-PDBBind table and the .pdbqt ligands, record FULL-file hashes with an explicit hash span, derive or mark derived pdb_ids, declare tox21full and the model-weight trees out of span, and correct the docs.",
    },
    Motion {
        id: "MOTION-009",
        title: "Molecule Structures: Close the 93 Percent SMILES Gap or Declare the Span",
        category: "Data / Chemistry",
        description: "Measured 2026-09-26: molecule.smiles is NULL for 2,697,822 of 2,897,819 rows (93.1%) and the 199,997 that carry one came from an RDKit pass; molecule_features holds derived descriptors for 2,897,818 (all but CHEMBL6188723); the SDF source is verified complete (931,156,420 B, gzip -t OK, sha256 2cdf4d33c5f42613... matching the release checksum). rdkit is NOT importable in the WSL interpreter the ingest runs under, though the Windows side has it. Proposal: either run the full structural pass at a measured rate, each row citing the source sha256 that produced it, or declare the 6.9 percent span in the index doc -- and close the missing CHEMBL6188723 features row either way.",
    },
    Motion {
        id: "MOTION-010",
        title: "Reachability: Keep the Structured Index Offline Tooling, With Its Consumers Named",
        category: "Architecture",
        description: "Measured 2026-09-26: swarm_v2/core/build_structured_index.py declares offline tooling with no kernel surface and no live module reads structured_index.db -- its readers are build and scratch tooling plus docs, while the platform chemistry paths resolve their inputs from explicit paths. Options: (1) keep it offline and NAME its consumers, adding a resolver seam only when a task path calls it; (2) add a kernel route now, which means a SQLite binding in the kernel dependency set. Proposal: option 1 -- one owner per store, the fabric for prose, an exact store for exact lookups, and no new kernel dependency until a real caller exists.",
    },
'''

anchor = 'rather than promises.",\r\n    },\r\n];\r\n'
assert src.count(anchor) == 1, "motions anchor hits=%d" % src.count(anchor)
src = src.replace(anchor, 'rather than promises.",\r\n    },\r\n' + MOTIONS + '];\r\n')
io.open(P, "w", encoding="utf-8", newline="").write(src)
print("inserted 3 motions; file lines:", src.count("\n"))
