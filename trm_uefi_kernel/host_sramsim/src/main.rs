// host_sramsim — run the Jarvis integration question through the SRAM computer.
//
// The question: Jarvis answers operators, the platform has several models, and today Jarvis guesses a
// model port. Which integration should ship?
//
// This does NOT re-implement the platform's primitives. It includes the kernel's own `k3_logic`,
// `sram_pattern_computer` and `k3_ternary_sram` modules by path, so the CAM, TCAM, Hopfield, Ising and
// Kleene operations here are the same code `/council/sram/:id` executes.
//
// HONESTY RULE FOR THIS FILE: a cell is MEASURED only if a live probe produced it (evidence quoted).
// Everything else is MODELLED. A simulation whose inputs are opinions produces a confident opinion, so
// the report prints how many cells each policy has measured and the ranking states what it rests on.
//
// PRIMITIVE SEMANTICS ACTUALLY USED (read from source, not assumed — my first run misused all three):
//   pattern_ising_spin_glass   -> per-spin FIELD SIGN (Vec<i8>), not an energy. Summing it saturates.
//   pattern_boltzmann_energy   -> E = -sum_ij w_ij s_i s_j, the real energy. Ground state with all-ones
//                                 weights is -dim^2 (every constraint satisfied).
//   K3Sram::bypass_over_stored -> trisum over the WHOLE stored plane, so it is a per-instance read, not
//                                 a per-pattern one: each policy gets a fresh K3Sram or the number
//                                 just reflects insertion order.
//   pattern_tcam_match         -> exact equality on masked trits. It cannot express "must not fail";
//                                 it answers "who has exactly this outcome in scenario X".

#[path = "../../host_council_server/src/k3_logic.rs"]
mod k3_logic;
#[path = "../../host_council_server/src/sram_pattern_computer.rs"]
mod sram_pattern_computer;
#[path = "../../host_council_server/src/k3_ternary_sram.rs"]
mod k3_ternary_sram;

use k3_ternary_sram::K3Sram;
use sram_pattern_computer::{pattern_boltzmann_energy, pattern_hopfield_recall, pattern_tcam_match};

#[derive(Clone, Copy, PartialEq)]
enum Outcome {
    Works,
    Degraded,
    HonestAbstain,
    Fails,
    Forbidden,
}

impl Outcome {
    fn trit(self) -> i8 {
        match self {
            Outcome::Works => 1,
            Outcome::Degraded | Outcome::HonestAbstain => 0,
            Outcome::Fails | Outcome::Forbidden => -1,
        }
    }
    fn code(self) -> &'static str {
        match self {
            Outcome::Works => "WORKS",
            Outcome::Degraded => "DEGRAD",
            Outcome::HonestAbstain => "ABSTAIN",
            Outcome::Fails => "FAILS",
            Outcome::Forbidden => "FORBID",
        }
    }
}

struct Cell {
    o: Outcome,
    measured: bool,
    why: &'static str,
}

fn m(o: Outcome, why: &'static str) -> Cell {
    Cell { o, measured: true, why }
}
fn x(o: Outcome, why: &'static str) -> Cell {
    Cell { o, measured: false, why }
}

struct Policy {
    id: &'static str,
    name: &'static str,
    cells: Vec<Cell>,
    /// Doctrine constraints as spins: +1 satisfied, -1 violated, in the order
    /// [zero-cloud, single-resident, no-stall, honest-when-unmeasured].
    doctrine: [i8; 4],
}

const SCENARIOS: [&str; 8] = [
    "gate model resident on :8587 (today's state)",
    "analysis model resident on :8596",
    "a model actually on :8597 (the old default)",
    "no model resident at all",
    "model mid-swap: /health ok, completion refused",
    "kernel unreachable, model up",
    "unknown model on a port no list contains",
    "Harmony-template model, caller builds ChatML",
];

fn main() {
    let policies = vec![
        Policy {
            id: "P1",
            name: "hardcoded :8597 + ChatML (status quo ante)",
            cells: vec![
                m(Outcome::Fails, "16x ECONNREFUSED 127.0.0.1:8597 in logs/jarvis_3000.log"),
                m(Outcome::Fails, "same constant, same refusal"),
                x(Outcome::Works, "only if a model were parked there"),
                x(Outcome::Fails, "refused"),
                x(Outcome::Fails, "refused"),
                x(Outcome::Works, "direct model, kernel not needed"),
                x(Outcome::Fails, "port absent from every constant"),
                x(Outcome::Degraded, "ChatML prompt to a Harmony model"),
            ],
            // honest-when-unmeasured violated: it reported a model outage that did not exist
            doctrine: [1, 1, 1, -1],
        },
        Policy {
            id: "P2",
            name: "probe model ports + ChatML (shipped today)",
            cells: vec![
                m(Outcome::Works, "POST /api/chat -> 'Boss, the kernel listens on TCP port 8022.'"),
                x(Outcome::Works, "8596 is second in the probe pool"),
                x(Outcome::Works, "8597 is first in the probe pool"),
                m(Outcome::HonestAbstain, "error names every port tried; no invented answer"),
                x(Outcome::Works, "re-probe on failure covers the swap"),
                m(Outcome::Works, "direct model, kernel not needed for the call"),
                x(Outcome::HonestAbstain, "outside JARVIS_MODEL_PORTS; error names the pool"),
                m(Outcome::Degraded, "measured: Harmony control tokens leaked into the reply"),
            ],
            doctrine: [1, 1, 1, 1],
        },
        Policy {
            id: "P3",
            name: "kernel /swarm/chat, caller grounding dropped",
            cells: vec![
                m(Outcome::Degraded, "POST /swarm/chat -> 'I cannot verify that premise.'"),
                x(Outcome::Degraded, "same: no live state in the prompt"),
                x(Outcome::Degraded, "same"),
                m(Outcome::HonestAbstain, "kernel reports the engine unreachable"),
                x(Outcome::Degraded, "3x-budget retry, may answer from stale context"),
                m(Outcome::Fails, "kernel is the only path; nothing answers"),
                x(Outcome::Degraded, "kernel resolves the port, grounding still absent"),
                m(Outcome::Works, "server applies the model's own template"),
            ],
            doctrine: [1, 1, 1, 1],
        },
        Policy {
            id: "P4",
            name: "kernel-mediated, caller system + grounding",
            cells: vec![
                m(Outcome::Works, "POST /swarm/chat with caller system -> port 8022 + build 9aabc88, 1.73s"),
                x(Outcome::Works, "same, any resident tier"),
                x(Outcome::Works, "kernel resolves whatever is live"),
                x(Outcome::HonestAbstain, "no engine: caller states it is unmeasured"),
                x(Outcome::Works, "kernel retries at 3x budget, template enforced"),
                x(Outcome::HonestAbstain, "kernel down: caller says so instead of guessing"),
                x(Outcome::Works, "kernel resolves ports, caller grounds the answer"),
                x(Outcome::Works, "template applied by the server, not the caller"),
            ],
            doctrine: [1, 1, 1, 1],
        },
        Policy {
            id: "P6",
            name: "kernel-mediated + direct-port fallback (hybrid)",
            cells: vec![
                m(Outcome::Works, "Jarvis log: answered via kernel:/swarm/chat, grounded"),
                x(Outcome::Works, "kernel path, grounded"),
                x(Outcome::Works, "kernel path, grounded"),
                x(Outcome::HonestAbstain, "both paths dead: says so, names both"),
                x(Outcome::Works, "kernel retries, then fallback probes"),
                m(Outcome::Works, "kernel stopped (8022=000): log shows answered via direct:127.0.0.1:8587 -> 'Ready.'"),
                x(Outcome::Works, "fallback probe finds it, server applies its template"),
                x(Outcome::Works, "either path gets template-correct output"),
            ],
            doctrine: [1, 1, 1, 1],
        },
        Policy {
            id: "P5",
            name: "cloud LLM (OpenAI-compatible API)",
            cells: vec![
                m(Outcome::Forbidden, "violates the zero-cloud doctrine"),
                m(Outcome::Forbidden, "same"),
                m(Outcome::Forbidden, "same"),
                m(Outcome::Forbidden, "same"),
                m(Outcome::Forbidden, "same"),
                m(Outcome::Forbidden, "same"),
                m(Outcome::Forbidden, "same"),
                m(Outcome::Forbidden, "same"),
            ],
            doctrine: [-1, 1, 1, -1],
        },
    ];

    let n_scen = SCENARIOS.len();

    println!("SRAM INTEGRATION SIMULATION — {} policies x {} scenarios", policies.len(), n_scen);
    println!("primitives: kernel's own k3_ternary_sram + sram_pattern_computer, included by path\n");
    for (i, s) in SCENARIOS.iter().enumerate() {
        println!("  S{} {}", i, s);
    }

    println!("\n{:<4} {:<48} {}", "id", "policy", "S0    S1    S2    S3    S4    S5    S6    S7");
    for p in &policies {
        let row: Vec<String> = p.cells.iter().map(|c| format!("{:<6}", c.o.code())).collect();
        println!("{:<4} {:<48} {}", p.id, p.name, row.join(""));
    }

    // ── 1. Kleene trisum per policy, against its OWN plane ────────────────────────────────────────
    // A fresh K3Sram per policy: bypass_over_stored() reads the whole stored window, so sharing one
    // instance would return a cumulative state that just tracks insertion order (my first run did
    // exactly that and produced confidence 1.000 for whichever policy was stored last).
    println!("\n-- Kleene trisum over each policy's own outcome vector --");
    let mut kleene: Vec<(&str, i8, f32, f32)> = Vec::new();
    for p in &policies {
        let pat: Vec<i8> = p.cells.iter().map(|c| c.o.trit()).collect();
        let mut own = K3Sram::new();
        own.store_pattern(p.id.as_bytes(), &pat);
        let ks = own.bypass_over_stored();
        let hits = own.cam_lookup(p.id.as_bytes());
        println!(
            "  {:<3} trit={:+} confidence={:.3} ambiguity={:.3} cam_hits=0x{:x}",
            p.id, ks.trit, ks.confidence, ks.ambiguity_ratio, hits
        );
        kleene.push((p.id, ks.trit, ks.confidence, ks.ambiguity_ratio));
    }

    // ── 2. TCAM per scenario: exact-match question "who has THIS outcome here" ────────────────────
    // This is what a TCAM is for. Asking it "who never fails" is not expressible in exact-match
    // semantics, which is itself a finding: the interesting question is the frontier, scenario by
    // scenario.
    println!("\n-- TCAM frontier (who has WORKS in each scenario) --");
    let probes = [0usize, 3, 5, 7];
    for &s in &probes {
        let mut query = vec![0i8; n_scen];
        let mut mask = vec![0u8; n_scen];
        query[s] = 1;
        mask[s] = 1;
        let winners: Vec<&str> = policies
            .iter()
            .filter(|p| {
                let pat: Vec<i8> = p.cells.iter().map(|c| c.o.trit()).collect();
                pattern_tcam_match(&pat, &query, &mask)
            })
            .map(|p| p.id)
            .collect();
        println!(
            "  S{} -> {}",
            s,
            if winners.is_empty() { "nobody".to_string() } else { winners.join(", ") }
        );
    }

    // ── 3. Ising energy: doctrine as a 4-spin glass ──────────────────────────────────────────────
    // All-four-required is expressed by ferromagnetic couplings (w=+1). E = -sum w_ij s_i s_j, so the
    // ground state is -16 for a policy that satisfies every constraint and rises as they are violated.
    // Using pattern_boltzmann_energy, because summing ising_spin_glass's field signs saturates at +4
    // for every policy and discriminates nothing.
    println!("\n-- Ising energy over the doctrine spins [no-cloud, single-resident, no-stall, honest] --");
    let weights = vec![1i8; 16];
    let mut energies: Vec<(&str, i64)> = Vec::new();
    for p in &policies {
        let e = pattern_boltzmann_energy(&p.doctrine, &weights, 4);
        println!("  {:<3} spins={:?} energy={:+} (ground state -16)", p.id, p.doctrine, e);
        energies.push((p.id, e));
    }

    // ── 4. Hopfield: is each policy's profile a STABLE state under uncertainty? ──────────────────
    // Auto-associative per policy: weights = v v^T, probe = v with two cells flipped (the two cells we
    // are least sure about). If recall returns v, the policy's story survives being wrong about two
    // scenarios. If it collapses, the profile is an unstable composition and any single bad assumption
    // rewrites the conclusion.
    println!("\n-- Hopfield stability (2 flipped cells out of {}) --", n_scen);
    let mut stability: Vec<(&str, usize)> = Vec::new();
    for p in &policies {
        let v: Vec<i8> = p.cells.iter().map(|c| c.o.trit()).collect();
        let mut w = vec![0i8; n_scen * n_scen];
        for i in 0..n_scen {
            for j in 0..n_scen {
                if i != j {
                    w[i * n_scen + j] = v[i].saturating_mul(v[j]);
                }
            }
        }
        let mut probe = v.clone();
        for k in [0usize, 5] {
            probe[k] = if probe[k] == 0 { 1 } else { -probe[k] };
        }
        let recalled = pattern_hopfield_recall(&probe, &w, n_scen);
        let agreement = recalled.iter().zip(v.iter()).filter(|(a, b)| a == b).count();
        println!("  {:<3} agreement {}/{}  recalled={:?}", p.id, agreement, n_scen, recalled);
        stability.push((p.id, agreement));
    }

    // ── 5. rank ───────────────────────────────────────────────────────────────────────────────────
    println!("\n-- ranking (usable desc, measured-usable desc, doctrine energy desc) --");
    let mut rows: Vec<(i32, u32, i64, usize, &str, &str)> = policies
        .iter()
        .map(|p| {
            let usable = p.cells.iter().filter(|c| c.o == Outcome::Works).count() as i32;
            let mu = p.cells.iter().filter(|c| c.measured && c.o == Outcome::Works).count() as u32;
            let e = energies.iter().find(|(id, _)| *id == p.id).map(|(_, e)| *e).unwrap_or(0);
            let st = stability.iter().find(|(id, _)| *id == p.id).map(|(_, s)| *s).unwrap_or(0);
            (usable, mu, e, st, p.id, p.name)
        })
        .collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(b.2.cmp(&a.2)));
    for (i, (usable, mu, e, st, id, name)) in rows.iter().enumerate() {
        println!(
            "  {}. {:<3} {:<46} usable={}/{} measured_usable={} energy={:+} hopfield={}/{}",
            i + 1, id, name, usable, n_scen, mu, e, st, n_scen
        );
    }

    if let Some((id, trit, conf, amb)) =
        kleene.iter().filter(|k| k.1 >= 0).max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
    {
        println!(
            "\nKleene trisum winner (trit>=0, highest confidence): {} trit={:+} conf={:.3} amb={:.3}",
            id, trit, conf, amb
        );
    }

    let measured_total: usize =
        policies.iter().map(|p| p.cells.iter().filter(|c| c.measured).count()).sum();
    let hypothesised: Vec<&str> = policies
        .iter()
        .filter(|p| p.cells.iter().all(|c| !c.measured))
        .map(|p| p.id)
        .collect();
    println!(
        "\nMEASURED/MODELLED SPLIT: {} of {} cells measured across all policies.",
        measured_total,
        n_scen * policies.len()
    );
    println!("Policies with NO measured cell (pure hypothesis): {}", hypothesised.join(", "));
    println!("Winner is only as good as that split — re-run after the kernel change to convert the");
    println!("modelled cells into measurements.");
}
