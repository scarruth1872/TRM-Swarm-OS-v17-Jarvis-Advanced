// /native/brain -- swarm-brain's connectome store, sparse neural mode and four-architecture switch.
//
// The crate decides; these routes transcribe. Nothing here reimplements arithmetic the crate owns: the
// connectome weights, the firing pattern, the collapse verdict and the CA/Ising/NAND interpretations are all
// the crate's own functions, called directly. Sizes are capped at this boundary and the cap is stated on the
// payload, because a route may not let a caller ask for an unbounded allocation.
use serde_json::{json, Value};
use swarm_brain::neural::{build_neural, firing_rate, neuron_collapse, step};
use swarm_brain::pattern_switch::{compute, switch_loop, Architecture};
use swarm_brain::{active_neurons, build_connectome, sparse_weight};

/// Stated cap: 1e6 neurons at a 1% hub fraction is 80k trits (160 KB packed) -- a route must not be able to
/// ask for an unbounded allocation.
const MAX_NEURONS: u64 = 1_000_000;
/// Stated cap: 1e6 neurons x 64 synapses is 64M ternary weights (64 MB) before packing.
const MAX_SYNAPSES: u64 = 64;

fn int(doc: &Value, key: &str) -> Result<u64, String> {
    doc.get(key)
        .and_then(|v| v.as_u64())
        .ok_or(format!("`{}` (non-negative integer) is required", key))
}

fn field(doc: &Value) -> Result<Vec<f32>, String> {
    let rows = doc
        .get("field")
        .and_then(|v| v.as_array())
        .ok_or("`field` (array of numbers, the input to the architecture) is required")?;
    if rows.is_empty() {
        return Err("`field` is empty: there is nothing to compute".to_string());
    }
    let mut out = Vec::with_capacity(rows.len());
    for (i, r) in rows.iter().enumerate() {
        out.push(r.as_f64().ok_or(format!("field[{}] must be a number", i))? as f32);
    }
    Ok(out)
}

fn arch(name: &str) -> Result<Architecture, String> {
    Ok(match name {
        "k3" => Architecture::K3,
        "ca" => Architecture::CellularAutomaton,
        "ising" => Architecture::Ising,
        "nand" => Architecture::Nand,
        other => {
            return Err(format!(
                "unknown arch `{}`; expected k3, ca, ising or nand",
                other
            ))
        }
    })
}

/// The scale-free connectome store. Reports what the crate built, plus a sample of the seed-regenerated
/// sparse synapses (the 215 GB path is a seed, not a file).
pub fn connectome(doc: &Value) -> Result<Value, String> {
    let n = int(doc, "n_neurons")?;
    if n == 0 || n > MAX_NEURONS {
        return Err(format!("`n_neurons` must be 1..={}", MAX_NEURONS));
    }
    let hub_fraction = doc.get("hub_fraction").and_then(|v| v.as_f64()).unwrap_or(0.01);
    if !(0.0..=1.0).contains(&hub_fraction) {
        return Err("`hub_fraction` must be within 0..=1".to_string());
    }
    let seed = int(doc, "seed").unwrap_or(42);
    let store = build_connectome(n as usize, hub_fraction, seed);
    let sample: Vec<Value> = (0..5)
        .map(|i| json!({ "index": i, "weight": sparse_weight(seed, i) }))
        .collect();
    Ok(json!({
        "n_neurons": n,
        "hub_fraction": hub_fraction,
        "seed": seed,
        "n_hubs": ((n as f64) * hub_fraction) as u64,
        "hub_trits": store.hub_trits.len(),
        "packed_bytes": store.packed.len(),
        "bits_per_trit": "2 (TurboVec)",
        "active_neurons": active_neurons(n as usize),
        "sparse_weight_sample": sample,
        "basis": "swarm_brain::build_connectome + active_neurons + sparse_weight -- the crate's own functions; \
                  the packed length is the crate's TurboVec output, not a computed estimate",
        "cap": format!("n_neurons <= {}", MAX_NEURONS),
    }))
}

/// One sparse neural timestep. The weights, the firing pattern and the rate are the crate's; the histogram is
/// counted from the weights the crate returned, so the ternary mix is reported rather than assumed.
pub fn neural_step(doc: &Value) -> Result<Value, String> {
    let n = int(doc, "n_neurons")?;
    let k = int(doc, "k_synapses").unwrap_or(32);
    if n == 0 || n > MAX_NEURONS {
        return Err(format!("`n_neurons` must be 1..={}", MAX_NEURONS));
    }
    if k == 0 || k > MAX_SYNAPSES {
        return Err(format!("`k_synapses` must be 1..={}", MAX_SYNAPSES));
    }
    let seed = int(doc, "seed").unwrap_or(42);
    let t = int(doc, "t").unwrap_or(1);
    let f = field(doc)?;
    let mode = build_neural(n as usize, k as usize, seed);
    let firing = step(&mode, &f, t);
    let (fired, total) = firing_rate(&firing);
    let count = |want: i8| mode.weights.iter().filter(|&&w| w == want).count();
    Ok(json!({
        "n_neurons": n,
        "k_synapses": k,
        "seed": seed,
        "t": t,
        "weights": mode.weights.len(),
        "weight_histogram": { "+1": count(1), "0": count(0), "-1": count(-1) },
        "packed_bytes": mode.packed.len(),
        "fired": fired,
        "of": total,
        "firing_rate": if total > 0 { (fired as f64) / (total as f64) } else { 0.0 },
        "firing": firing,
        "basis": "swarm_brain::neural::build_neural + step + firing_rate -- the sparse ~1% target is the \
                  crate's rule, and the observed rate is measured from the returned pattern",
        "cap": format!("n_neurons <= {}, k_synapses <= {}", MAX_NEURONS, MAX_SYNAPSES),
    }))
}

/// The K3 collapse on a neuron's input field: the crate's verdict and the trits it collapsed.
pub fn collapse(doc: &Value) -> Result<Value, String> {
    let f = field(doc)?;
    let tau = doc.get("tau").and_then(|v| v.as_f64()).unwrap_or(0.35) as f32;
    let (verdict, trits) = neuron_collapse(&f, tau);
    let t = trits.iter().filter(|&&v| v == 1).count();
    let neg = trits.iter().filter(|&&v| v == -1).count();
    Ok(json!({
        "tau": tau,
        "verdict": verdict,
        "trits": trits,
        "true": t,
        "false": neg,
        "zero": trits.len() - t - neg,
        "basis": "swarm_brain::neural::neuron_collapse -- the verdict is the crate's majority rule on its own \
                  collapse, not a rule added at this boundary",
    }))
}

/// The pattern-modulated switch: one substrate, four architectures, the same field. With `arch` the crate
/// computes one; without it the crate's `switch_loop` runs all four and returns its own timing.
pub fn switch(doc: &Value) -> Result<Value, String> {
    let f = field(doc)?;
    let seed = int(doc, "seed").unwrap_or(42);
    if let Some(name) = doc.get("arch").and_then(|v| v.as_str()) {
        let a = arch(name)?;
        let (out, note) = compute(a, &f, seed);
        return Ok(json!({
            "arch": a.as_str(),
            "output": out,
            "note": note,
            "basis": "swarm_brain::pattern_switch::compute -- the interpretation of the field is the crate's",
        }));
    }
    let (results, us) = switch_loop(&f, seed);
    let rows: Vec<Value> = results
        .iter()
        .map(|(a, note)| json!({ "arch": a.as_str(), "note": note }))
        .collect();
    Ok(json!({
        "architectures": rows,
        "count": results.len(),
        "elapsed_us": us,
        "clock": "measured on this server's monotonic clock inside the crate's switch_loop, not a GPU claim",
        "basis": "swarm_brain::pattern_switch::switch_loop over the same field in all four architectures",
    }))
}
