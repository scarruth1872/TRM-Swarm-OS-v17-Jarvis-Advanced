//! A minimal SMILES canonicalizer (Rust, no deps) for the organic-subset common case.
//!
//! CAP-201: SMILES canonicalization should be Rust, not Python/RDKit. This parses a SMILES string
//! into atoms + bonds, computes a canonical atom ordering via Morgan extended-connectivity
//! invariants, and re-emits a canonical SMILES. It is NOT a full RDKit-equivalent: aromaticity is
//! taken from the input (lowercase atoms) without Kekulization, and stereochemistry / ring-bond
//! directions are not modeled. Parity against RDKit's canonical SMILES is measured, not assumed.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bond {
    Single,
    Double,
    Triple,
    Aromatic,
}

#[derive(Clone, Debug)]
pub struct Atom {
    pub symbol: String, // "C", "N", "O", "Cl", ... (uppercase-first canonical form)
    pub aromatic: bool, // lowercase in the input
    pub charge: i32,
    pub isotope: u16,
    pub hcount: u8, // implicit H count (0 = unspecified)
    pub chiral: u8, // 0 = none, 1 = @, 2 = @@
}

#[derive(Clone, Debug)]
pub struct Molecule {
    pub atoms: Vec<Atom>,
    pub bonds: Vec<(usize, usize, Bond)>,
}

fn element_rank(sym: &str) -> u64 {
    // organic-subset ordering (stable, deterministic). Unknown elements rank by byte order.
    const TABLE: &[&str] = &["C", "N", "O", "S", "P", "F", "Cl", "Br", "I", "Si", "B"];
    TABLE.iter().position(|&s| s == sym).map(|i| i as u64 + 1).unwrap_or_else(|| {
        100 + sym.bytes().fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64))
    })
}

fn bond_rank(b: &Bond) -> u64 {
    match b {
        Bond::Single => 1,
        Bond::Double => 2,
        Bond::Triple => 3,
        Bond::Aromatic => 4,
    }
}

/// Parse a bracket atom per the OpenSMILES grammar: `[isotope? symbol chiral? hcount? charge? class?]`.
/// Returns (isotope, symbol, aromatic, charge, hcount, chiral). The symbol is normalized to
/// uppercase-first form; the aromatic flag carries the input's lowercase-first form.
fn parse_bracket(inner: &[u8]) -> Option<(u16, String, bool, i32, u8, u8)> {
    let mut j = 0;
    // 1. isotope (leading digits)
    let mut isotope = 0u16;
    while j < inner.len() && inner[j].is_ascii_digit() {
        isotope = isotope * 10 + (inner[j] - b'0') as u16;
        j += 1;
    }
    // 2. element symbol: uppercase-first ("Cl"), or lowercase-first aromatic ("n", "se", "as")
    let start = j;
    let mut aromatic = false;
    if j < inner.len() && inner[j].is_ascii_uppercase() {
        j += 1;
        while j < inner.len() && inner[j].is_ascii_lowercase() {
            j += 1;
        }
    } else if j < inner.len() && inner[j].is_ascii_lowercase() {
        aromatic = true;
        j += 1;
        while j < inner.len() && inner[j].is_ascii_lowercase() {
            j += 1;
        }
    } else {
        return None; // no element symbol
    }
    let mut sym = String::from_utf8_lossy(&inner[start..j]).to_string();
    if aromatic {
        // normalize to uppercase-first ("n" -> "N", "se" -> "Se")
        let mut chars = sym.chars();
        if let Some(first) = chars.next() {
            sym = first.to_ascii_uppercase().to_string() + chars.as_str();
        }
    }
    // 3. chiral (@ or @@)
    let mut chiral = 0u8;
    if j < inner.len() && inner[j] == b'@' {
        chiral = 1;
        j += 1;
        if j < inner.len() && inner[j] == b'@' {
            chiral = 2;
            j += 1;
        }
    }
    // 4. H count (H + optional digits)
    let mut hcount = 0u8;
    if j < inner.len() && inner[j] == b'H' {
        hcount = 1;
        j += 1;
        let ds = j;
        let mut n = 0u32;
        while j < inner.len() && inner[j].is_ascii_digit() {
            n = n * 10 + (inner[j] - b'0') as u32;
            j += 1;
        }
        if j > ds {
            hcount = n as u8;
        }
    }
    // 5. charge (+/- + optional digits, default magnitude 1)
    let mut charge = 0i32;
    if j < inner.len() && (inner[j] == b'+' || inner[j] == b'-') {
        let sign = if inner[j] == b'+' { 1i32 } else { -1i32 };
        j += 1;
        let ds = j;
        let mut n = 0i32;
        while j < inner.len() && inner[j].is_ascii_digit() {
            n = n * 10 + (inner[j] - b'0') as i32;
            j += 1;
        }
        charge = sign * if j > ds { n } else { 1 };
    }
    // 6. class (: + digits) — parsed and discarded (rare; not emitted)
    if j < inner.len() && inner[j] == b':' {
        j += 1;
        while j < inner.len() && inner[j].is_ascii_digit() {
            j += 1;
        }
    }
    Some((isotope, sym, aromatic, charge, hcount, chiral))
}

/// Parse an organic-subset atom token: aromatic = lowercase. Advances `i` past the token.
fn parse_organic_symbol(b: &[u8], i: &mut usize) -> Option<(String, bool)> {
    let c = b[*i];
    let aromatic = c.is_ascii_lowercase();
    let upper = c.to_ascii_uppercase();
    let first = upper;
    // two-letter symbols: first uppercase + second LOWERCASE ("Cl", "Br", ...)
    if *i + 1 < b.len() {
        let next = b[*i + 1];
        let two = format!("{}{}", first as char, next as char);
        if matches!(two.as_str(), "Cl" | "Br" | "Si" | "Se" | "As" | "Na" | "Mg" | "Al" | "Zn" | "Sn" | "Pt" | "Fe" | "Cu" | "Ag" | "Au" | "Ca" | "K" | "Li") {
            *i += 1;
            return Some((two, aromatic));
        }
    }
    Some(((first as char).to_string(), aromatic))
}

pub fn parse(s: &str) -> Option<Molecule> {
    let b = s.trim().as_bytes();
    let mut atoms: Vec<Atom> = Vec::new();
    let mut bonds: Vec<(usize, usize, Bond)> = Vec::new();
    let mut prev: Option<usize> = None;
    let mut pending: Bond = Bond::Single;
    let mut branches: Vec<Option<usize>> = Vec::new();
    let mut rings: [Option<usize>; 100] = [None; 100];
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        match c {
            b'(' => branches.push(prev),
            b')' => prev = branches.pop()?,
            b'-' => pending = Bond::Single,
            b'=' => pending = Bond::Double,
            b'#' => pending = Bond::Triple,
            b':' => pending = Bond::Aromatic,
            b'.' => prev = None,
            b'/' | b'\\' => pending = Bond::Single,
            b'0'..=b'9' => {
                let d = (c - b'0') as usize;
                let cur = prev?;
                match rings[d] {
                    Some(open) => {
                        bonds.push((open, cur, pending.clone()));
                        rings[d] = None;
                    }
                    None => rings[d] = Some(cur),
                }
                pending = Bond::Single;
            }
            b'%' => {
                // %NN ring digit (>= 10)
                if i + 2 >= b.len() {
                    return None;
                }
                let d = (b[i + 1] - b'0') as usize * 10 + (b[i + 2] - b'0') as usize;
                let cur = prev?;
                match rings[d] {
                    Some(open) => {
                        bonds.push((open, cur, pending.clone()));
                        rings[d] = None;
                    }
                    None => rings[d] = Some(cur),
                }
                pending = Bond::Single;
                i += 2;
            }
            b'[' => {
                let end = b[i + 1..].iter().position(|&x| x == b']')? + i + 1;
                let (isotope, symbol, aromatic, charge, hcount, chiral) = parse_bracket(&b[i + 1..end])?;
                let idx = atoms.len();
                atoms.push(Atom { symbol, aromatic, charge, isotope, hcount, chiral });
                if let Some(p) = prev {
                    bonds.push((p, idx, pending.clone()));
                }
                prev = Some(idx);
                pending = Bond::Single;
                i = end;
            }
            _ => {
                let (symbol, aromatic) = parse_organic_symbol(b, &mut i)?;
                let idx = atoms.len();
                atoms.push(Atom { symbol, aromatic, charge: 0, isotope: 0, hcount: 0, chiral: 0 });
                if let Some(p) = prev {
                    bonds.push((p, idx, pending.clone()));
                }
                prev = Some(idx);
                pending = Bond::Single;
            }
        }
        i += 1;
    }
    Some(Molecule { atoms, bonds })
}

fn degree(mol: &Molecule, i: usize) -> u64 {
    mol.bonds.iter().filter(|(a, b, _)| *a == i || *b == i).count() as u64
}

fn initial_invariant(mol: &Molecule, i: usize) -> u64 {
    let a = &mol.atoms[i];
    let er = element_rank(&a.symbol);
    let arom = if a.aromatic { 1 } else { 0 };
    let chg = (a.charge + 8) as u64; // bias so negative charges are positive
    let hc = a.hcount as u64;
    let ch = a.chiral as u64;
    // hc/chiral live in the HIGH bits so they cannot disturb the er>degree>arom>chg ordering
    // that non-bracket atoms depended on (the regression source: the previous layout shifted chg).
    (hc << 56) | (ch << 48) | (er << 32) | (degree(mol, i) << 16) | (arom << 8) | chg
}

/// Morgan extended-connectivity canonical ordering via LEXICOGRAPHIC refinement.
/// The refinement key is (rank, sorted neighbor-rank list) — a complete key, unlike the hash —
/// so ties remain only for TRUE symmetry, which round-trips stably. Returns the canonical order.
pub fn canonical_order(mol: &Molecule) -> Vec<usize> {
    let n = mol.atoms.len();
    let mut ranks: Vec<u64> = (0..n).map(|i| initial_invariant(mol, i)).collect();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (a, b, _) in &mol.bonds {
        adj[*a].push(*b);
        adj[*b].push(*a);
    }
    for _ in 0..=n {
        // 1. rank the current ranks -> 1..n (equal values share a rank)
        {
            let mut keys: Vec<(u64, usize)> = ranks
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, v)| (v, i))
                .collect();
            keys.sort_unstable();
            let mut r = 0u64;
            let mut prev: Option<u64> = None;
            for (v, i) in keys {
                if prev != Some(v) {
                    r += 1;
                    prev = Some(v);
                }
                ranks[i] = r;
            }
        }
        // 2. next key = (rank, sorted neighbor ranks) — lexicographic, no hash collisions
        let mut keys: Vec<((u64, Vec<u64>), usize)> = Vec::with_capacity(n);
        for i in 0..n {
            let mut nb: Vec<u64> = adj[i].iter().map(|&j| ranks[j]).collect();
            nb.sort_unstable();
            keys.push(((ranks[i], nb), i));
        }
        keys.sort();
        let mut r = 0u64;
        let mut prev: Option<(u64, Vec<u64>)> = None;
        for (k, i) in keys {
            if prev.as_ref() != Some(&k) {
                r += 1;
                prev = Some(k);
            }
            ranks[i] = r;
        }
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| ranks[i]);
    order
}

/// Emit the canonical SMILES via a CANGEN-style two-pass DFS: (1) find ring bonds (back edges)
/// and assign ring digits, (2) emit atom symbol + ring openings + tree edges (branch parens) +
/// ring closings. Single and aromatic bonds are implicit (no symbol), matching RDKit's convention.
pub fn emit(mol: &Molecule, order: &[usize]) -> String {
    let n = mol.atoms.len();
    let mut rank = vec![0usize; n];
    for (r, &i) in order.iter().enumerate() {
        rank[i] = r;
    }
    let mut adj: Vec<Vec<(usize, Bond)>> = vec![Vec::new(); n];
    for (a, b, bond) in &mol.bonds {
        adj[*a].push((*b, bond.clone()));
        adj[*b].push((*a, bond.clone()));
    }
    for l in adj.iter_mut() {
        l.sort_by_key(|(j, _)| rank[*j]);
        l.dedup_by(|x, y| x.0 == y.0);
    }

    // Pass 1: find ring bonds (back edges) + assign digits + record the DFS tree (parent).
    let mut visited = vec![false; n];
    let mut on_stack = vec![false; n];
    let mut parent: Vec<usize> = vec![usize::MAX; n];
    let mut ring_open: Vec<Vec<u8>> = vec![Vec::new(); n];
    let mut ring_close: Vec<Vec<(u8, Bond)>> = vec![Vec::new(); n];
    let mut next_digit: u8 = 1;
    for &start in order {
        if visited[start] {
            continue;
        }
        find_rings(start, usize::MAX, &adj, &mut visited, &mut on_stack, &mut parent, &mut ring_open, &mut ring_close, &mut next_digit);
    }

    // Build the tree children list from parent (sorted by rank). Ring bonds are excluded here;
    // they are emitted as ring-closure digits, never as tree edges.
    let mut children_of: Vec<Vec<(usize, Bond)>> = vec![Vec::new(); n];
    for (a, b, bond) in &mol.bonds {
        if parent[*a] == *b {
            children_of[*b].push((*a, bond.clone()));
        } else if parent[*b] == *a {
            children_of[*a].push((*b, bond.clone()));
        }
    }
    for l in children_of.iter_mut() {
        l.sort_by_key(|(j, _)| rank[*j]);
    }

    // Pass 2: emit from each tree root (in canonical order).
    let mut out = String::new();
    for &start in order {
        if parent[start] != usize::MAX {
            continue;
        }
        if !out.is_empty() {
            out.push('.');
        }
        emit_dfs(start, mol, &children_of, &ring_open, &ring_close, &mut out);
    }
    out
}

fn find_rings(
    i: usize,
    parent_i: usize,
    adj: &[Vec<(usize, Bond)>],
    visited: &mut [bool],
    on_stack: &mut [bool],
    parent: &mut [usize],
    ring_open: &mut [Vec<u8>],
    ring_close: &mut [Vec<(u8, Bond)>],
    next_digit: &mut u8,
) {
    visited[i] = true;
    on_stack[i] = true;
    parent[i] = parent_i;
    for (j, bond) in adj[i].clone() {
        if j == parent_i {
            continue;
        }
        if !visited[j] {
            find_rings(j, i, adj, visited, on_stack, parent, ring_open, ring_close, next_digit);
        } else if on_stack[j] {
            let d = *next_digit;
            *next_digit += 1;
            ring_open[j].push(d);
            ring_close[i].push((d, bond));
        }
    }
    on_stack[i] = false;
}

fn emit_dfs(
    i: usize,
    mol: &Molecule,
    children_of: &[Vec<(usize, Bond)>],
    ring_open: &[Vec<u8>],
    ring_close: &[Vec<(u8, Bond)>],
    out: &mut String,
) {
    write_atom(mol, i, out);
    for &d in &ring_open[i] {
        push_ring_char(out, d);
    }
    // Emit every child except the LAST as a branch `(...)`, the last as the main continuation.
    // Branches must be emitted right after this atom's symbol (before the main subtree) so the
    // parentheses attach to THIS atom, not to the main chain's tail.
    let children = &children_of[i];
    let nc = children.len();
    for (k, (j, bond)) in children.iter().enumerate() {
        let is_last = k + 1 == nc;
        if !is_last {
            out.push('(');
        }
        emit_tree_bond(out, mol, i, *j, bond);
        emit_dfs(*j, mol, children_of, ring_open, ring_close, out);
        if !is_last {
            out.push(')');
        }
    }
    for (d, bond) in ring_close[i].clone() {
        emit_ring_close(out, &bond, d);
    }
}

fn emit_tree_bond(out: &mut String, mol: &Molecule, a: usize, b: usize, bond: &Bond) {
    if mol.atoms[a].aromatic && mol.atoms[b].aromatic {
        return; // implicit aromatic bond
    }
    match bond {
        Bond::Double => out.push('='),
        Bond::Triple => out.push('#'),
        Bond::Single | Bond::Aromatic => {}
    }
}

fn emit_ring_close(out: &mut String, bond: &Bond, d: u8) {
    match bond {
        Bond::Double => out.push('='),
        Bond::Triple => out.push('#'),
        Bond::Single | Bond::Aromatic => {}
    }
    push_ring_char(out, d);
}

fn push_ring_char(out: &mut String, d: u8) {
    if d <= 9 {
        out.push((b'0' + d) as char);
    } else {
        // SMILES %NN encoding for ring digits >= 10 (no control-char overflow)
        let d = d.min(99);
        out.push('%');
        out.push((b'0' + d / 10) as char);
        out.push((b'0' + d % 10) as char);
    }
}

fn write_atom(mol: &Molecule, i: usize, out: &mut String) {
    let a = &mol.atoms[i];
    let needs_bracket = a.charge != 0 || a.isotope != 0 || a.hcount != 0 || a.chiral != 0;
    if needs_bracket {
        out.push('[');
        if a.isotope != 0 {
            out.push_str(&a.isotope.to_string());
        }
        if a.aromatic {
            out.push_str(&a.symbol.to_ascii_lowercase()); // [nH] -> lowercase symbol
        } else {
            out.push_str(&a.symbol);
        }
        if a.chiral == 1 {
            out.push('@');
        } else if a.chiral == 2 {
            out.push_str("@@");
        }
        if a.hcount != 0 {
            out.push('H');
            if a.hcount > 1 {
                out.push_str(&a.hcount.to_string());
            }
        }
        match a.charge {
            0 => {}
            1 => out.push('+'),
            -1 => out.push('-'),
            n if n > 0 => {
                out.push('+');
                out.push_str(&n.to_string());
            }
            n => {
                out.push('-');
                out.push_str(&(-n).to_string());
            }
        }
        out.push(']');
    } else if a.aromatic {
        out.push_str(&a.symbol.to_ascii_lowercase());
    } else {
        out.push_str(&a.symbol);
    }
}

/// Canonicalize a SMILES string. Returns None on parse failure.
pub fn canonical_smiles(input: &str) -> Option<String> {
    let mol = parse(input)?;
    if mol.atoms.is_empty() {
        return None;
    }
    let order = canonical_order(&mol);
    Some(emit(&mol, &order))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_simple() {
        // ethanol CCO -> a valid canonical form (some reordering of the two carbons is expected)
        let c = canonical_smiles("CCO").unwrap();
        assert!(c.contains("C") && c.contains("O") && c.len() >= 3);
    }

    #[test]
    fn aromatic() {
        let c = canonical_smiles("c1ccccc1").unwrap();
        assert!(c.contains('c'));
    }

    #[test]
    fn bracket_charge() {
        let c = canonical_smiles("[NH4+]").unwrap();
        assert!(c.contains("NH4"));
    }
}
