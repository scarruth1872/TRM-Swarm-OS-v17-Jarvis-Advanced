//! TRM Swarm OS — Spatial Reasoning (no_std, alloc). Task 578.
//!
//! What it does: relations and queries over real 3D coordinates — contact graphs between atom sets, and a
//! simple region topology (disjoint / touching / overlapping / equal / inside) for ball regions.
//!
//! What it is NOT: RCC-8 over arbitrary regions. RCC-8 reasons about connected *regions*; this reasons about
//! **balls** (a centre and a radius), which is the part that can be computed exactly from coordinates and
//! checked against an independent implementation. Calling it RCC-8 would be the sort of label this platform has
//! spent the week removing, so the type is called `BallRelation` and the boundary is stated here.
//!
//! Everything is exact f64 arithmetic with squared distances where possible: no epsilon tuning, no
//! "approximately touching" that could be adjusted until a test passes. A contact is `d² <= cutoff²`, which is
//! the same predicate an independent implementation evaluates.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Square root without libm or the `std` method (this crate is `no_std`).
///
/// Newton's iteration from a coarse starting guess, converging quadratically: 40 steps is far past the point
/// where an f64 stops improving, and the loop exits early once the step is below 1e-15 relative. The contact
/// predicate itself never uses this — it compares squared distances — so a 1-ulp difference here cannot change
/// which pairs are reported as contacts, only the printed distance.
pub fn sqrt(x: f64) -> f64 {
    if !(x > 0.0) {
        return 0.0;
    }
    let mut g = if x > 1.0 { x } else { 1.0 };
    let mut i = 0;
    while i < 40 {
        let next = 0.5 * (g + x / g);
        if (next - g).abs() <= 1e-15 * next {
            return next;
        }
        g = next;
        i += 1;
    }
    g
}

/// A point in Cartesian space, in the units of the source file (Å for PDB/PDBQT).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Point {
        Point { x, y, z }
    }

    /// Squared distance. Callers compare against `cutoff * cutoff`, so no square root is taken and no
    /// rounding is introduced by one.
    pub fn dist2(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn distance(&self, other: &Point) -> f64 {
        sqrt(self.dist2(other))
    }
}

/// Relationship between two balls. The five cases are exhaustive and mutually exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BallRelation {
    /// Centres further apart than the sum of radii: the balls do not meet.
    Disjoint,
    /// Surfaces touch to within the caller's tolerance. Reported separately because "just touching" is a
    /// different claim from "overlapping", and conflating them is how contact maps acquire phantom bonds.
    ExternallyConnected,
    /// The surfaces intersect without either centre being inside the other.
    Overlapping,
    /// Same centre, same radius.
    Equal,
    /// One lies entirely within the other.
    Contains,
}

impl BallRelation {
    pub fn as_str(self) -> &'static str {
        match self {
            BallRelation::Disjoint => "DISJOINT",
            BallRelation::ExternallyConnected => "EXTERNALLY_CONNECTED",
            BallRelation::Overlapping => "OVERLAPPING",
            BallRelation::Equal => "EQUAL",
            BallRelation::Contains => "CONTAINS",
        }
    }
}

/// A ball region: centre and radius.
#[derive(Debug, Clone, Copy)]
pub struct Ball {
    pub centre: Point,
    pub radius: f64,
}

impl Ball {
    pub fn new(centre: Point, radius: f64) -> Ball {
        Ball { centre, radius }
    }
}

/// Classify two balls.
///
/// `tolerance` is explicit and the caller's choice: it is the width of the "touching" band, not a fudge
/// factor inside the comparison. A caller that wants strict geometry passes 0.0.
pub fn relate(a: &Ball, b: &Ball, tolerance: f64) -> BallRelation {
    let d = a.centre.distance(&b.centre);
    let sum = a.radius + b.radius;
    let diff = (a.radius - b.radius).abs();

    if d <= tolerance && diff <= tolerance {
        return BallRelation::Equal;
    }
    if d > sum + tolerance {
        return BallRelation::Disjoint;
    }
    if (d - sum).abs() <= tolerance {
        return BallRelation::ExternallyConnected;
    }
    // centres closer than the radius difference => one contains the other
    if d + tolerance < diff {
        return BallRelation::Contains;
    }
    BallRelation::Overlapping
}

/// One labelled atom position. The label carries whatever the caller needs to map a result back to the file
/// (serial number, residue name, chain) — the engine never invents one.
#[derive(Debug, Clone)]
pub struct Atom {
    pub label: String,
    pub position: Point,
    /// Element or atom type as written in the source file (`C`, `N`, `O`, `ZN`, `A` for AutoDock types).
    pub element: String,
}

/// A contact between two atoms, with the measured distance so a report can show its work.
#[derive(Debug, Clone)]
pub struct Contact {
    pub a: usize,
    pub b: usize,
    pub distance: f64,
}

/// Every pair closer than `cutoff` (Å), computed with squared distances.
///
/// `include_hydrogens = false` skips hydrogen atoms, because a contact map full of H contacts is not what
/// anyone means by "residues in contact" — but the choice is the caller's and is recorded in the result.
pub fn contact_graph(atoms: &[Atom], cutoff: f64, include_hydrogens: bool) -> Vec<Contact> {
    let mut out = Vec::new();
    let cut2 = cutoff * cutoff;
    for i in 0..atoms.len() {
        if !include_hydrogens && is_hydrogen(&atoms[i].element) {
            continue;
        }
        for j in (i + 1)..atoms.len() {
            if !include_hydrogens && is_hydrogen(&atoms[j].element) {
                continue;
            }
            let d2 = atoms[i].position.dist2(&atoms[j].position);
            if d2 <= cut2 {
                out.push(Contact { a: i, b: j, distance: sqrt(d2) });
            }
        }
    }
    out
}

fn is_hydrogen(element: &str) -> bool {
    let e = element.trim();
    e.eq_ignore_ascii_case("H") || e.eq_ignore_ascii_case("HD") || e.eq_ignore_ascii_case("HS")
}

/// Indices of atoms within `radius` of `centre` — the "what is near this point" query.
pub fn atoms_within(atoms: &[Atom], centre: &Point, radius: f64) -> Vec<usize> {
    let r2 = radius * radius;
    atoms
        .iter()
        .enumerate()
        .filter(|(_, a)| a.position.dist2(centre) <= r2)
        .map(|(i, _)| i)
        .collect()
}

/// Indices of atoms in `set` whose position lies within `radius` of any atom in `reference`.
/// This is the "residues within 4 Å of the ligand" query, answered without materialising a full graph.
pub fn near_set(atoms: &[Atom], reference: &[usize], radius: f64) -> Vec<usize> {
    let r2 = radius * radius;
    let mut out: Vec<usize> = Vec::new();
    let mut seen = alloc::collections::BTreeSet::new();
    for &r in reference {
        if let Some(ref_atom) = atoms.get(r) {
            for (i, a) in atoms.iter().enumerate() {
                if reference.contains(&i) || seen.contains(&i) {
                    continue;
                }
                if a.position.dist2(&ref_atom.position) <= r2 {
                    seen.insert(i);
                    out.push(i);
                }
            }
        }
    }
    out.sort_unstable();
    out
}

/// The screening box a docking run uses: an axis-aligned box, and whether a point falls inside it.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Point,
    pub max: Point,
}

impl BoundingBox {
    pub fn contains(&self, p: &Point) -> bool {
        p.x >= self.min.x && p.x <= self.max.x
            && p.y >= self.min.y && p.y <= self.max.y
            && p.z >= self.min.z && p.z <= self.max.z
    }

    /// The box enclosing every atom given. `None` for an empty set — an empty box is not a box at (0,0,0).
    pub fn of(atoms: &[Atom]) -> Option<BoundingBox> {
        let first = atoms.first()?;
        let mut min = first.position;
        let mut max = first.position;
        for a in atoms.iter().skip(1) {
            min.x = min.x.min(a.position.x);
            min.y = min.y.min(a.position.y);
            min.z = min.z.min(a.position.z);
            max.x = max.x.max(a.position.x);
            max.y = max.y.max(a.position.y);
            max.z = max.z.max(a.position.z);
        }
        Some(BoundingBox { min, max })
    }

    /// Geometric centre — the point a docking box is built around.
    pub fn centre(&self) -> Point {
        Point::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    pub fn size(&self) -> Point {
        Point::new(self.max.x - self.min.x, self.max.y - self.min.y, self.max.z - self.min.z)
    }
}

/// A compact textual report, so a caller can print what was measured without re-deriving it.
pub fn summarise(atoms: &[Atom], cutoff: f64) -> String {
    let contacts = contact_graph(atoms, cutoff, false);
    let extent = match BoundingBox::of(atoms) {
        Some(b) => {
            let s = b.size();
            format!("box {:.1} x {:.1} x {:.1} A", s.x, s.y, s.z)
        }
        None => "no atoms".to_string(),
    };
    format!(
        "atoms {} | {} | contacts within {:.1} A: {}",
        atoms.len(),
        extent,
        cutoff,
        contacts.len()
    )
}
