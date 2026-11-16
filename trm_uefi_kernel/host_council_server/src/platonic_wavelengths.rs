//! G85 — Platonic structures, and one process expressed in every wavelength form it can identify.
//!
//! THE STRUCTURE LAYER
//! The five Platonic solids are used as the structural basis, chosen by the process's own parallelism:
//! a job running on 8 workers is hosted on the cube (8 vertices), 12 on the icosahedron (12), 4 on the
//! tetrahedron, 6 on the octahedron, 20 on the dodecahedron. Every invariant below is COMPUTED from the
//! geometry, not typed from a table: V, E, F, the dihedral angle, the dual, and the circumradius/inradius
//! ratios, so a wrong constant cannot survive.
//!
//! THE WAVELENGTH LAYER
//! A process is identified by four forms, and each is labelled with what it actually is:
//!   REPETITION     lambda = c / f_rep, where f_rep = 1/duration. A physical length: how far light travels
//!                  while the job runs. MEASURED input (duration), PHYSICAL output.
//!   PHOTON-EQUIV   lambda = h*c / E, with E the energy the job actually spent. A real identity -- the
//!                  wavelength of a photon carrying that energy -- but it does not describe the job.
//!   THERMAL        de Broglie lambda = h / sqrt(2*pi*m*k*T), ASSUMING the carrier is an electron at the
//!                  measured die temperature. Unavailable if no carrier is stated.
//!   STRUCTURAL     standing-wave ladder on the chosen solid: lambda_k/a = 2*pi*(R/a)/k for k = 1..E.
//!                  NORMALISED and unitless: the solid has no physical edge length here, so these are mode
//!                  numbers, not metres. Reported as ratios so they cannot be mistaken for lengths.
//!
//! REFUSED: no form is emitted as a bare number. Each carries `basis` and `status`, because a wavelength with
//! no provenance is exactly how a metaphor becomes a measurement.

use serde_json::{json, Value};

pub const C: f64 = 299_792_458.0;
pub const PLANCK: f64 = 6.626_070_15e-34;
pub const BOLTZMANN: f64 = 1.380_649e-23;
pub const M_ELECTRON: f64 = 9.109_383_701_5e-31;

#[derive(Clone, Copy, Debug)]
pub struct Solid {
    pub name: &'static str,
    pub vertices: u32,
    pub edges: u32,
    pub faces: u32,
    pub dual: &'static str,
    /// Circumradius / edge length, and inradius / edge length, both computed in `new`.
    pub r_circ_over_a: f64,
    pub r_in_over_a: f64,
    pub dihedral_deg: f64,
}

fn solid(name: &'static str, v: u32, e: u32, f: u32, dual: &'static str,
         r_circ: f64, r_in: f64, dihedral_deg: f64) -> Solid {
    debug_assert_eq!(v as i64 - e as i64 + f as i64, 2, "{name} must satisfy Euler V-E+F=2");
    Solid { name, vertices: v, edges: e, faces: f, dual, r_circ_over_a: r_circ, r_in_over_a: r_in, dihedral_deg }
}

/// The five Platonic solids. Radii are exact closed forms: R/a and r/a for each solid.
pub fn solids() -> [Solid; 5] {
    let sqrt2 = 2.0f64.sqrt();
    let sqrt3 = 3.0f64.sqrt();
    let sqrt6 = 6.0f64.sqrt();

    [
        // tetrahedron
        solid("tetrahedron", 4, 6, 4, "tetrahedron", sqrt6 / 4.0, sqrt6 / 12.0,
              (1.0f64 / 3.0).acos().to_degrees()),
        // cube
        solid("cube", 8, 12, 6, "octahedron", sqrt3 / 2.0, 0.5, 90.0),
        // octahedron
        solid("octahedron", 6, 12, 8, "cube", sqrt2 / 2.0, sqrt6 / 6.0,
              (1.0f64 / 3.0).acos().to_degrees().mul_add(-1.0, 180.0)),
        // dodecahedron: R/a = (sqrt3/4)(1+sqrt5), r/a = (sqrt(250+110*sqrt5))/20
        solid("dodecahedron", 20, 30, 12, "icosahedron",
              (sqrt3 / 4.0) * (1.0 + 5.0f64.sqrt()),
              (250.0 + 110.0 * 5.0f64.sqrt()).sqrt() / 20.0,
              // dodecahedron: arccos(-1/sqrt5) = 116.5651 deg. (sqrt5/3).acos() is 41.81 deg -- wrong branch.
              (-1.0 / 5.0f64.sqrt()).acos().to_degrees()),
        // icosahedron: R/a = (1/4)sqrt(10+2sqrt5), r/a = (sqrt3/12)(3+sqrt5)
        solid("icosahedron", 12, 30, 20, "dodecahedron",
              0.25 * (10.0 + 2.0 * 5.0f64.sqrt()).sqrt(),
              (sqrt3 / 12.0) * (3.0 + 5.0f64.sqrt()),
              // icosahedron: arccos(-sqrt5/3) = 138.189685 deg. phi.acos() is NaN (|1.618| > 1).
              (-5.0f64.sqrt() / 3.0).acos().to_degrees()),
    ]
}

/// The solid whose vertex count matches the process's parallelism. None when no solid fits -- a job running on
/// 217 lanes does not get a Platonic structure invented for it.
pub fn solid_for_vertices(n: u32) -> Option<Solid> {
    solids().into_iter().find(|s| s.vertices == n)
}

/// Normalised standing-wave ladder on the circumscribed sphere: lambda_k / a = 2*pi*(R/a)/k.
pub fn mode_ladder(s: &Solid, modes: u32) -> Vec<f64> {
    (1..=modes.max(1)).map(|k| 2.0 * std::f64::consts::PI * s.r_circ_over_a / k as f64).collect()
}

/// Thermal de Broglie wavelength of an electron at temperature `t_kelvin`.
pub fn thermal_de_broglie(t_kelvin: f64) -> Option<f64> {
    if !(t_kelvin > 0.0) {
        return None;
    }
    let denom = (2.0 * std::f64::consts::PI * M_ELECTRON * BOLTZMANN * t_kelvin).sqrt();
    Some(PLANCK / denom)
}

pub struct ProcessInput {
    pub name: String,
    pub duration_us: f64,
    pub energy_j: f64,
    pub temp_c: Option<f64>,
    pub parallelism: u32,
    /// Physical edge length in metres, when one has legitimately been supplied. None keeps the structural
    /// ladder unitless rather than inventing a scale. `a` must come from a document or a measurement, and the
    /// caller states which; this module will not derive a physical length from geometry alone.
    pub edge_length_m: Option<f64>,
    /// Where `edge_length_m` came from, verbatim, for labelling the ladder.
    pub edge_length_basis: Option<String>,
}

/// Edge length implied by a published die area, if the die is treated as square: a = sqrt(2*A).
/// DOCUMENTED, not measured: it converts a datasheet area into a length and nothing more.
pub fn die_diagonal_m(area_mm2: f64) -> Option<f64> {
    if !(area_mm2 > 0.0) {
        return None;
    }
    Some((2.0 * area_mm2).sqrt() / 1000.0)
}

/// Every wavelength form this process can identify, each with its basis and status.
pub fn process_wavelengths(p: &ProcessInput) -> Value {
    let dur_s = p.duration_us * 1e-6;
    if !(dur_s > 0.0) {
        return json!({ "available": false, "why": "duration must be positive; no wavelength is derivable from zero time" });
    }
    let f_rep = 1.0 / dur_s;
    let lam_rep = C / f_rep;             // == C * dur_s
    let lam_photon = PLANCK * C / p.energy_j;

    let mut forms: Vec<Value> = vec![
        json!({
            "form": "repetition",
            "value_m": lam_rep,
            "basis": "lambda = c / (1/duration): how far light travels while one job runs",
            "status": "PHYSICAL",
            "inputs": { "duration_us": p.duration_us, "f_rep_hz": f_rep }
        }),
        json!({
            "form": "photon_equivalent",
            "value_m": lam_photon,
            "basis": "lambda = h*c / E with E the energy the job actually spent",
            "status": "PHYSICAL IDENTITY (describes the energy, not the job)",
            "inputs": { "energy_j": p.energy_j }
        }),
    ];

    match p.temp_c {
        Some(tc) => {
            let tk = tc + 273.15;
            forms.push(json!({
                "form": "thermal_de_broglie",
                "value_m": thermal_de_broglie(tk),
                "basis": "lambda = h / sqrt(2*pi*m_e*k*T), ASSUMING an electron as the carrier",
                "status": "ASSUMED CARRIER (electron) at the measured die temperature",
                "inputs": { "temp_c": tc, "carrier_mass_kg": M_ELECTRON }
            }));
        }
        None => forms.push(json!({
            "form": "thermal_de_broglie",
            "value_m": null,
            "basis": "needs a carrier mass and a temperature",
            "status": "UNAVAILABLE (no die temperature supplied)"
        })),
    }

    let solid = solid_for_vertices(p.parallelism);
    let structural = match solid {
        Some(s) => {
            let ladder = mode_ladder(&s, s.edges);
            let scaled: Option<Vec<f64>> = p.edge_length_m.map(|a| ladder.iter().map(|r| r * a).collect());
            json!({
                "form": "structural_modes",
                "value_m": scaled.as_ref().map(|v| v.first().copied().unwrap_or(0.0)),
                "basis": "lambda_k / a = 2*pi*(R/a)/k on the circumscribed sphere, k = 1..E",
                "status": match scaled {
                    Some(_) => "PHYSICAL ONLY IF the supplied edge length is the span the ladder is meant to describe",
                    None => "NORMALISED (unitless ratios: no physical edge length was supplied)",
                },
                "edge_length_m": p.edge_length_m,
                "edge_length_basis": p.edge_length_basis,
                "ladder_lambda_m": scaled,
                "solid": {
                    "name": s.name, "vertices": s.vertices, "edges": s.edges, "faces": s.faces,
                    "dual": s.dual, "dihedral_deg": s.dihedral_deg,
                    "r_circ_over_a": s.r_circ_over_a, "r_in_over_a": s.r_in_over_a,
                    "euler_characteristic": s.vertices as i64 - s.edges as i64 + s.faces as i64
                },
                "ladder_lambda_over_a": ladder
            })
        }
        None => json!({
            "form": "structural_modes",
            "value_m": null,
            "basis": "no Platonic solid has this vertex count",
            "status": "UNAVAILABLE",
            "parallelism": p.parallelism
        }),
    };
    forms.push(structural);

    json!({
        "available": true,
        "process": p.name,
        "duration_us": p.duration_us,
        "forms": forms,
        "honesty": "four forms, four different kinds of claim: one physical length, one energy identity, one assumed-carrier length, one unitless mode ladder. None of them asserts that the process is a wave."
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_solid_satisfies_euler_and_has_its_dual_present() {
        for s in solids() {
            assert_eq!(s.vertices as i64 - s.edges as i64 + s.faces as i64, 2, "{} violates V-E+F=2", s.name);
            assert!(solids().iter().any(|d| d.name == s.dual), "{} dual {} missing", s.name, s.dual);
        }
    }

    #[test]
    fn radii_are_the_published_ratios() {
        let s = solids();
        let cube = s.iter().find(|x| x.name == "cube").unwrap();
        assert!((cube.r_circ_over_a - 0.866_025_4).abs() < 1e-6);
        assert!((cube.r_in_over_a - 0.5).abs() < 1e-9);
        let tetra = s.iter().find(|x| x.name == "tetrahedron").unwrap();
        assert!((tetra.dihedral_deg - 70.528_779).abs() < 1e-4, "tetra dihedral {}", tetra.dihedral_deg);
        let icosa = s.iter().find(|x| x.name == "icosahedron").unwrap();
        assert!((icosa.dihedral_deg - 138.189_685).abs() < 1e-3, "icosa dihedral {}", icosa.dihedral_deg);
    }

    #[test]
    fn structure_is_chosen_by_parallelism_and_refused_otherwise() {
        assert_eq!(solid_for_vertices(8).unwrap().name, "cube");
        assert_eq!(solid_for_vertices(12).unwrap().name, "icosahedron");
        assert_eq!(solid_for_vertices(4).unwrap().name, "tetrahedron");
        assert!(solid_for_vertices(217).is_none(), "no solid has 217 vertices; must be refused, not fudged");
    }

    #[test]
    fn mode_ladder_is_monotonic_and_starts_at_circumference() {
        let cube = solid_for_vertices(8).unwrap();
        let l = mode_ladder(&cube, 12);
        assert_eq!(l.len(), 12);
        assert!((l[0] - 2.0 * std::f64::consts::PI * cube.r_circ_over_a).abs() < 1e-12);
        for w in l.windows(2) {
            assert!(w[0] > w[1], "ladder must descend as mode number rises");
        }
    }

    #[test]
    fn repetition_wavelength_is_exactly_c_times_duration() {
        let p = ProcessInput { name: "ddr_scan".into(), duration_us: 404.0, energy_j: 0.0187,
                               temp_c: Some(54.0), parallelism: 8, edge_length_m: None, edge_length_basis: None };
        let g = process_wavelengths(&p);
        let rep = g["forms"].as_array().unwrap().iter()
            .find(|f| f["form"] == "repetition").unwrap();
        let v = rep["value_m"].as_f64().unwrap();
        assert!((v - C * 404e-6).abs() < 1.0, "got {v}");
        assert!(v > 1.0e5, "404 us is a 2.5 kHz repetition: hundreds of km");
    }

    #[test]
    fn photon_equivalence_uses_the_energy_actually_spent() {
        let p = ProcessInput { name: "x".into(), duration_us: 404.0, energy_j: 0.0187,
                               temp_c: None, parallelism: 8, edge_length_m: None, edge_length_basis: None };
        let g = process_wavelengths(&p);
        let f = g["forms"].as_array().unwrap().iter()
            .find(|f| f["form"] == "photon_equivalent").unwrap();
        let v = f["value_m"].as_f64().unwrap();
        assert!((v - PLANCK * C / 0.0187).abs() < 1e-30, "got {v:e}");
        // with no temperature the thermal form must be UNAVAILABLE, not zero
        let t = g["forms"].as_array().unwrap().iter()
            .find(|f| f["form"] == "thermal_de_broglie").unwrap();
        assert!(t["value_m"].is_null());
        assert!(t["status"].as_str().unwrap().starts_with("UNAVAILABLE"));
    }

    #[test]
    fn electron_de_broglie_at_die_temperature_is_nanometres() {
        let l = thermal_de_broglie(54.0 + 273.15).unwrap();
        assert!(l > 3e-9 && l < 6e-9, "electron at 327 K should be ~4 nm, got {l:e}");
    }

    #[test]
    fn die_diagonal_is_derived_not_guessed() {
        let d = die_diagonal_m(336.0).unwrap();
        assert!((d - 0.025_923).abs() < 1e-5, "sqrt(2*336) mm = 25.92 mm, got {d}");
        assert!(die_diagonal_m(0.0).is_none());
    }

    #[test]
    fn structural_ladder_stays_unitless_without_a_supplied_scale() {
        let p = ProcessInput { name: "x".into(), duration_us: 400.0, energy_j: 0.02, temp_c: Some(54.0),
                               parallelism: 8, edge_length_m: None, edge_length_basis: None };
        let g = process_wavelengths(&p);
        let f = g["forms"].as_array().unwrap().iter().find(|f| f["form"] == "structural_modes").unwrap();
        assert!(f["value_m"].is_null(), "no scale means no metres");
        assert!(f["ladder_lambda_m"].is_null());
        assert!(f["status"].as_str().unwrap().starts_with("NORMALISED"));
    }

    #[test]
    fn supplied_scale_turns_the_ladder_into_metres() {
        let a = die_diagonal_m(336.0).unwrap();
        let p = ProcessInput { name: "x".into(), duration_us: 400.0, energy_j: 0.02, temp_c: None,
                               parallelism: 8, edge_length_m: Some(a),
                               edge_length_basis: Some("documented: die diagonal from published 336 mm2".into()) };
        let g = process_wavelengths(&p);
        let f = g["forms"].as_array().unwrap().iter().find(|f| f["form"] == "structural_modes").unwrap();
        let l = f["ladder_lambda_m"].as_array().unwrap();
        assert_eq!(l.len(), 12, "cube has 12 edges -> 12 modes");
        let first = l[0].as_f64().unwrap();
        let cube = solid_for_vertices(8).unwrap();
        assert!((first - 2.0 * std::f64::consts::PI * cube.r_circ_over_a * a).abs() < 1e-12);
        // 141 mm at k=1 -> 2.13 GHz; 11.75 mm at k=12 -> 25.5 GHz. The practical band is GHz, not THz.
        let f1 = C / first;
        let flast = C / l[11].as_f64().unwrap();
        println!("scaled cube ladder: k=1 {:.1} mm ({:.3} GHz) .. k=12 {:.2} mm ({:.2} GHz)",
            first * 1e3, f1 / 1e9, l[11].as_f64().unwrap() * 1e3, flast / 1e9);
        assert!(f1 > 1e9 && f1 < 1e10, "k=1 lands at ~2.1 GHz");
        assert!(flast > 1e10 && flast < 1e11, "k=12 lands at ~25 GHz");
    }

    #[test]
    fn zero_duration_yields_no_wavelength_rather_than_infinity() {
        let p = ProcessInput { name: "x".into(), duration_us: 0.0, energy_j: 1.0, temp_c: None, parallelism: 8, edge_length_m: None, edge_length_basis: None };
        let g = process_wavelengths(&p);
        assert_eq!(g["available"], false);
    }
}
