"""eskape_serialize.py - UniProt reference proteomes for the six ESKAPE pathogens -> one record per protein.

ESKAPE = the six nosocomial pathogens that escape antibiotic action:
  Enterococcus faecium, Staphylococcus aureus, Klebsiella pneumoniae,
  Acinetobacter baumannii, Pseudomonas aeruginosa, Enterobacter cloacae.

Source: UniProt REST, `uniprotkb/stream?query=proteome:<UPID>&format=fasta`. Each UPID is a **Reference proteome**
verified by fetching `proteomes/<UPID>.json` and reading `proteomeType` - not chosen by name search, which returns
near-neighbours (`Acinetobacter nosocomialis`, `Pseudomonas paraeruginosa`, `K. pneumoniae subsp. ozaenae`) and would
have quietly produced five organisms wearing ESKAPE labels. Every downloaded protein count matched UniProt's own
`proteinCount` exactly, which is the check that the stream was not truncated.

FASTA header format (measured): `>sp|ACCESSION|ENTRY_NAME description OS=... OX=... GN=gene PE=n SV=n` for reviewed
entries, `>tr|...` for unreviewed. Fields present in the header are emitted; absent ones are omitted, never guessed.
"""
import os

MAX_SEQ = 40000  # S. aureus Q2FYJ6 (ebh) is 9,535 aa - a 4,000 ceiling was an assumption and rejected real data


def _parse_header(h: str):
    """'sp|P0A0B1|NAME desc OS=x GN=y PE=1 SV=2' -> dict."""
    body = h.lstrip(">")
    d = {}
    parts = body.split(" ", 1)
    key = parts[0].split("|")
    if len(key) >= 3:
        d["db"], d["acc"], d["entry"] = key[0], key[1], key[2]
    else:
        d["acc"] = parts[0]
    rest = parts[1] if len(parts) > 1 else ""
    for tok, name in (("GN=", "gene"), ("PE=", "pe"), ("SV=", "sv"), ("OX=", "ox")):
        i = rest.find(tok)
        if i >= 0:
            d[name] = rest[i + len(tok):].split(" ")[0].strip(";")
    desc = rest
    for tok in ("GN=", "PE=", "SV=", "OX="):
        i = desc.find(tok)
        if i > 0:
            desc = desc[:i]
    d["description"] = desc.strip()
    return d


def load_proteome(fasta_path: str, pathogen: str, organism_name: str):
    """Yield (key, text) per protein in one FASTA."""
    upid = os.path.basename(fasta_path).replace(".fasta", "")
    if not os.path.exists(fasta_path):
        raise SystemExit(f"FAIL-CLOSED: missing proteome {fasta_path}")
    out = []
    hdr = None
    seq = []
    with open(fasta_path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            line = line.rstrip("\n")
            if line.startswith(">"):
                if hdr is not None:
                    out.append(_record(hdr, seq, pathogen, organism_name, upid))
                hdr, seq = line, []
            elif hdr is not None:
                seq.append(line.strip())
        if hdr is not None:
            out.append(_record(hdr, seq, pathogen, organism_name, upid))
    if not out:
        raise SystemExit(f"FAIL-CLOSED: {fasta_path} yielded 0 proteins")
    return out


def _record(hdr, seq_lines, pathogen, organism_name, upid):
    d = _parse_header(hdr)
    sequence = "".join(seq_lines).upper()
    if not sequence:
        raise SystemExit(f"FAIL-CLOSED: protein {d.get('acc')} in {upid} has no sequence")
    if len(sequence) > MAX_SEQ:
        raise SystemExit(f"FAIL-CLOSED: protein {d.get('acc')} length {len(sequence)} exceeds {MAX_SEQ}")
    parts = [f"ESKAPE {pathogen} protein {d['acc']}"]
    if d.get("entry"):
        parts.append(f"entry {d['entry']}")
    if d.get("description"):
        parts.append(f"description {d['description']}")
    if d.get("gene"):
        parts.append(f"gene {d['gene']}")
    parts.append(f"proteome {upid}")
    parts.append(f"organism {organism_name}")
    parts.append(f"reviewed {d.get('db') == 'sp'}")
    parts.append(f"length {len(sequence)}")
    parts.append(f"sequence {sequence}")
    return f"eskape/{d['acc']}", " ".join(parts)
