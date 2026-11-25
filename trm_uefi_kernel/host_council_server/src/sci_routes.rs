//! /sci/* — scientific computing (tasks 421-430).
//!
//! Every response carries the reference its number is checked against, so a caller can tell a validated quantity
//! from a modelled one. The three claims this module refuses are stated in the payloads: Hückel is not DFT, the
//! HP lattice is not structure prediction, and the contact score carries no affinity correlation.

use serde_json::{json, Value};
use trm_uefi_kernel::scientific_computing as sc;

fn u(body: &Value, key: &str, default: f64) -> f64 {
    body.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
}

fn i(body: &Value, key: &str, default: i64) -> i64 {
    body.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

/// 421 — molecular dynamics.
pub fn md(body: &Value) -> Result<Value, String> {
    let n_side = i(body, "n_side", 3).clamp(2, 6) as usize;
    let spacing = u(body, "spacing", 1.12).max(0.8);
    let eps = u(body, "eps", 1.0);
    let sigma = u(body, "sigma", 1.0);
    let dt = u(body, "dt", 0.001).clamp(1e-6, 0.01);
    let steps = i(body, "steps", 500).clamp(1, 20_000) as usize;

    let mut sys = sc::MdSystem::lattice(n_side, spacing, eps, sigma);
    sys.cutoff = 3.0 * sigma + spacing;
    let e0 = sys.total_energy();
    for _ in 0..steps {
        sys.step(dt);
    }
    let e1 = sys.total_energy();
    let drift = ((e1 - e0).abs() / e0.abs().max(1e-12)) as f64;
    Ok(json!({
        "model": "lennard-jones + velocity verlet",
        "particles": sys.pos.len(),
        "dt": dt, "steps": steps,
        "energy_initial": e0, "energy_final": e1,
        "relative_drift": drift,
        "temperature": sys.temperature(1.0),
        "reference": "energy conservation - a symplectic integrator must not drift; drift is the metric, not a promise",
        "verdict": if drift < 1e-2 { "CONSERVING" } else { "DRIFTING" },
        "not_claimed": "no force field parameterisation for a real molecule; eps/sigma are the caller's"
    }))
}

/// 422 — Hückel molecular orbitals (explicitly not DFT).
pub fn huckel(body: &Value) -> Result<Value, String> {
    let kind = body.get("kind").and_then(|v| v.as_str()).unwrap_or("ring");
    let n = i(body, "n", 6).clamp(2, 24) as usize;
    let alpha = u(body, "alpha", 0.0);
    let beta = u(body, "beta", -1.0);
    let electrons = i(body, "electrons", (n as i64).max(2)).clamp(2, 2 * n as i64) as usize;

    let mut m = if kind == "chain" { sc::huckel_chain(n, alpha, beta) } else { sc::huckel_ring(n, alpha, beta) };
    let e = sc::jacobi_eigen(&mut m, 200);
    let pi = sc::pi_energy(&e, electrons);
    let gap = sc::homo_lumo_gap(&e, electrons);
    Ok(json!({
        "model": "Hückel pi-electron theory (Jacobi eigensolver)",
        "topology": kind, "atoms": n, "electrons": electrons,
        "eigenvalues_over_beta": e,
        "pi_energy_over_beta": pi,
        "homo_lumo_gap_over_beta": gap,
        "aromatic": if kind == "ring" {
            // Hückel 4n+2 rule on the electron count
            let e_half = electrons / 2;
            let four_n_plus_2 = (2..=e_half).map(|k| 4 * k - 2).any(|v| v == electrons);
            json!(four_n_plus_2)
        } else { json!(false) },
        "reference": "benzene (ring, 6, 6 electrons) must give eigenvalues -2,-1,-1,1,1,2 and E = -8 beta",
        "not_claimed": "NOT DFT, NOT Hartree-Fock: no basis set, no self-consistent field, no correlation functional"
    }))
}

/// 423 — HP lattice folding (a search, tested against exhaustive ground state).
pub fn fold(body: &Value) -> Result<Value, String> {
    let seq = body.get("sequence").and_then(|v| v.as_str()).unwrap_or("HPHPPHPPHH");
    let bytes: Vec<u8> = seq.bytes().filter(|b| *b == b'H' || *b == b'P').collect();
    if bytes.len() < 3 {
        return Err("sequence needs at least 3 residues of H/P".to_string());
    }
    if bytes.len() > 12 {
        return Err(format!(
            "sequence of {} residues refused: exhaustive ground state is only tractable to 12, and reporting an annealer result as an optimum would be a false claim", bytes.len()));
    }
    let steps = i(body, "steps", 60_000).clamp(1000, 400_000) as usize;
    let seed = i(body, "seed", 7) as u64;

    let (opt, opt_coords, complete) = sc::hp_exhaustive(&bytes, 20_000_000);
    let (anneal, anneal_coords) = sc::hp_anneal(&bytes, steps, seed, 2.0, 0.05);
    Ok(json!({
        "model": "2-D HP lattice, self-avoiding",
        "sequence": seq, "residues": bytes.len(),
        "exhaustive_energy": opt,
        "exhaustive_complete": complete,
        "exhaustive_conformation": opt_coords,
        "annealed_energy": anneal,
        "annealed_conformation": anneal_coords,
        "gap": anneal - opt,
        "reference": "the exhaustive DFS ground state - the annealer may never beat it",
        "not_claimed": "not structure prediction: HP is a toy energy model with one hydrophobic contact term"
    }))
}

/// 424 — contact energy for a pose (no affinity correlation claimed).
pub fn affinity(body: &Value) -> Result<Value, String> {
    let parse = |key: &str| -> Result<Vec<sc::Atom>, String> {
        let arr = body.get(key).and_then(|v| v.as_array()).ok_or(format!("'{key}' must be an array of [x,y,z,charge,radius]"))?;
        let mut out = Vec::new();
        for row in arr {
            let a: Vec<f64> = row.as_array().ok_or("each atom must be an array")?
                .iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
            if a.len() < 5 {
                return Err("each atom needs 5 values: x,y,z,charge,radius".to_string());
            }
            out.push(sc::Atom { pos: [a[0], a[1], a[2]], charge: a[3], radius: a[4].max(0.1) });
        }
        Ok(out)
    };
    let lig = parse("ligand")?;
    let pocket = parse("pocket")?;
    let dielectric = u(body, "dielectric", 4.0).max(1.0);
    let e = sc::contact_energy(&lig, &pocket, dielectric);
    let total_charge: f64 = lig.iter().map(|a| a.charge).sum();
    let buried = u(body, "buried_fraction", 0.0).clamp(0.0, 1.0);
    let desolv = sc::desolvation_penalty(buried, total_charge);
    Ok(json!({
        "model": "Lennard-Jones + screened Coulomb + linear desolvation",
        "ligand_atoms": lig.len(), "pocket_atoms": pocket.len(), "dielectric": dielectric,
        "contact_energy_kcal": e,
        "desolvation_kcal": desolv,
        "total_kcal": e + desolv,
        "reference": "a single LJ pair must reach -eps at 2^(1/6)*sigma, and Coulomb must equal 332.06*q1*q2/(eps_r*r)",
        "not_claimed": "NO correlation with measured affinity. 4463 PDBbind complexes with pKd exist in datasets/, but they have not been extracted and scored, and docking already produced one false result here (RMSD 13.82 A frame mismatch)"
    }))
}

/// 425 — pharmacokinetics.
pub fn pk(body: &Value) -> Result<Value, String> {
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("iv_bolus");
    let dose = u(body, "dose", 100.0);
    let volume = u(body, "volume", 10.0).max(1e-6);
    let k = u(body, "k", 0.1).max(1e-6);
    let ka = u(body, "ka", 1.0).max(1e-6);
    let n = i(body, "points", 24).clamp(2, 500) as usize;
    let tmax = u(body, "t_max", 24.0).max(1e-6);
    let mut series = Vec::with_capacity(n);
    for idx in 0..n {
        let t = tmax * idx as f64 / (n - 1) as f64;
        let c = match model {
            "oral" => sc::pk_oral(dose, volume, k, ka, t),
            "two_compartment" => {
                let a = u(body, "A", 8.0);
                let alpha = u(body, "alpha", 1.0);
                let b = u(body, "B", 2.0);
                let beta = u(body, "beta", 0.1);
                sc::pk_two_compartment(a, alpha, b, beta, t)
            }
            _ => sc::pk_iv_bolus(dose, volume, k, t),
        };
        series.push(json!({"t": t, "c": c}));
    }
    Ok(json!({
        "model": model,
        "half_life": sc::pk_half_life(k),
        "t_max": sc::pk_t_max(k, ka),
        "c_initial": series.first().map(|v| v["c"].clone()).unwrap_or(json!(0.0)),
        "series": series,
        "reference": "closed form: C(0) = dose/V for IV bolus, 0 for oral, and C falls by half each half-life",
        "not_claimed": "no physiological covariates, no protein binding, no patient data - parameters are the caller's"
    }))
}

/// 426 — epidemiology.
pub fn sir(body: &Value) -> Result<Value, String> {
    let beta = u(body, "beta", 0.5).max(0.0);
    let gamma = u(body, "gamma", 0.2).max(1e-9);
    let i0 = u(body, "i0", 1e-4).clamp(1e-9, 0.5);
    let days = u(body, "days", 400.0).clamp(1.0, 5000.0);
    let dt = u(body, "dt", 0.01).clamp(1e-4, 0.05);
    let r = sc::sir(beta, gamma, i0, days, dt);
    let analytic = sc::sir_final_size(beta, gamma, 1.0 - i0);
    Ok(json!({
        "model": "SIR, RK4",
        "r0": r.r0,
        "final_size": r.final_size,
        "analytic_final_size": analytic,
        "agreement_error": (r.final_size - analytic).abs(),
        "peak_infected": r.peak_infected,
        "peak_day": r.peak_day,
        "attack_rate": r.attack_rate,
        "herd_immunity_threshold": if r.r0 > 1.0 { 1.0 - 1.0 / r.r0 } else { 0.0 },
        "reference": "the transcendental final-size equation 1 - z = s0*exp(-R0*z), solved independently by bisection",
        "not_claimed": "no age structure, no waning immunity, no vaccination, no contact matrix - homogeneous mixing only"
    }))
}

/// 427 — population genetics.
pub fn drift(body: &Value) -> Result<Value, String> {
    let p0 = u(body, "p0", 0.3).clamp(1e-6, 1.0 - 1e-6);
    let pop = i(body, "pop", 20).clamp(2, 200) as usize;
    let reps = i(body, "reps", 200).clamp(10, 2000) as usize;
    let seed = i(body, "seed", 42) as u64;
    let s = u(body, "s", 0.0);
    let gens = i(body, "generations", 10).clamp(1, 1000) as usize;
    let measured = sc::wright_fisher_fixation(p0, pop, reps, seed);
    let (aa, ab, bb) = sc::hardy_weinberg(p0);
    Ok(json!({
        "model": "Wright-Fisher neutral drift + haploid selection",
        "p0": p0, "population": pop, "replicates": reps, "seed": seed,
        "measured_fixation_probability": measured,
        "expected_fixation_probability": p0,
        "error": (measured - p0).abs(),
        "hardy_weinberg": {"AA": aa, "Aa": ab, "aa": bb},
        "heterozygosity_after_generations": sc::heterozygosity_after(2.0 * p0 * (1.0 - p0), pop, gens),
        "selected_next": sc::selection_next(p0, s),
        "reference": "neutral theory: fixation probability equals the initial frequency - measured over replicates, not asserted",
        "not_claimed": "no linkage, no recombination, no mutation, no demography"
    }))
}

/// 428 — systems biology.
pub fn lotka(body: &Value) -> Result<Value, String> {
    let alpha = u(body, "alpha", 1.0);
    let beta = u(body, "beta", 0.1);
    let gamma = u(body, "gamma", 1.0);
    let delta = u(body, "delta", 0.1);
    let x0 = u(body, "x0", 10.0).max(1e-6);
    let y0 = u(body, "y0", 5.0).max(1e-6);
    let dt = u(body, "dt", 0.01).clamp(1e-5, 0.05);
    let steps = i(body, "steps", 5000).clamp(10, 200_000) as usize;
    let lv = sc::lotka_volterra(alpha, beta, gamma, delta, x0, y0, dt, steps);
    let sample = 20usize;
    let stride = (lv.x.len() / sample).max(1);
    let points: Vec<Value> = lv.x.iter().enumerate().step_by(stride)
        .map(|(idx, x)| json!({"t": idx as f64 * dt, "prey": x, "predator": lv.y[idx]})).collect();
    Ok(json!({
        "model": "Lotka-Volterra predator-prey, RK4",
        "conserved_quantity": lv.h0,
        "relative_drift": lv.h_drift,
        "points": points,
        "reference": "H = delta*x - gamma*ln(x) + beta*y - alpha*ln(y) is conserved exactly; drift measures integrator quality",
        "verdict": if lv.h_drift < 1e-2 { "CONSERVING" } else { "DRIFTING" },
        "not_claimed": "this is a two-species toy model; SMPDB pathway data exists on disk but no pathway is wired in yet"
    }))
}

/// 429 — spike-train analysis.
pub fn spikes(body: &Value) -> Result<Value, String> {
    let rate = u(body, "rate_hz", 20.0).clamp(0.1, 1000.0);
    let duration = u(body, "duration", 200.0).clamp(1.0, 10_000.0);
    let refractory = u(body, "refractory", 0.0).clamp(0.0, 0.5);
    let seed = i(body, "seed", 99) as u64;
    let bin = u(body, "bin", 1.0).max(1e-3);
    let times = sc::poisson_spikes(rate, duration, refractory, seed);
    let stats = sc::spike_stats(&times, duration, bin);
    Ok(json!({
        "model": "Poisson spike train + ISI statistics",
        "spikes": stats.n,
        "rate_hz": stats.rate_hz,
        "mean_isi_ms": stats.mean_isi * 1000.0,
        "cv": stats.cv,
        "fano": stats.fano,
        "reference": "Poisson: CV -> 1 and Fano -> 1; a refractory period must pull CV below the Poisson value",
        "not_claimed": "no spike sorting, no bursting detection, no model fitting - descriptive statistics only"
    }))
}

/// 430 — suite index.
pub fn suite() -> Value {
    let entries: Vec<Value> = sc::suite_entries().iter().map(|e| json!({
        "task": e.task, "name": e.name, "reference": e.reference, "capability": e.capability,
        "cost": match e.task { 421 => "O(N^2) per step", 423 => "exponential in residues (capped at 12)", _ => "linear in steps" }
    })).collect();
    json!({
        "suite": entries,
        "total": entries.len(),
        "refused": [
            "422 is Hückel, not DFT: no basis set, no SCF, no correlation functional",
            "423 is the HP lattice toy model, not structure prediction",
            "424 has no affinity correlation: 4463 PDBbind complexes exist but were not extracted or scored"
        ],
        "sram_computer": "the Ising spin-glass pattern in sram_pattern_computer is exercised by the relaxation test; lattice folding deliberately uses Metropolis rather than being forced through a 1-D nearest-neighbour pattern"
    })
}
