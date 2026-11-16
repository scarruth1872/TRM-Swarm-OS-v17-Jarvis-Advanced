//! swarm-instella — the 64-expert sparse MoE gate in Rust.
//!
//! Port of c_instella_gate_topk from the C++ kernel:
//!   logits[e] = Σ_i(hidden_states[i] * expert_weights[e][i])  (384-dim)
//!   top-k = partial sort of (logit, expert_id) descending
//!   output: selected ids + softmax gate weights (matches c_instella_gate)
//!
//! Verify gate: identical top-k expert selection vs the C kernel on the
//! same hidden state + weights.

pub const TOTAL_EXPERTS: usize = 64;
pub const HIDDEN_DIM: usize = 384;

/// Compute expert logits for a hidden state, then select top-k.
/// Returns (selected_ids, softmax_weights) — same contract as the C API.
pub fn gate_topk(hidden: &[f32; HIDDEN_DIM], weights: &[[f32; HIDDEN_DIM]; TOTAL_EXPERTS],
                 top_k: usize) -> (Vec<u8>, Vec<f32>) {
    assert!(top_k > 0 && top_k <= TOTAL_EXPERTS);
    let mut logits: Vec<f32> = Vec::with_capacity(TOTAL_EXPERTS);
    for e in 0..TOTAL_EXPERTS {
        let mut acc = 0.0f32;
        for i in 0..HIDDEN_DIM {
            acc += hidden[i] * weights[e][i];
        }
        logits.push(acc);
    }

    // partial sort descending (matches std::partial_sort semantics)
    let mut scores: Vec<(f32, u8)> = logits.iter().enumerate()
        .map(|(i, &l)| (l, i as u8)).collect();
    scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scores.truncate(top_k);

    // softmax over the top-k logits
    let max_l = scores[0].0;
    let exps: Vec<f32> = scores.iter().map(|(l, _)| (l - max_l).exp()).collect();
    let sum: f32 = exps.iter().sum();
    let ids: Vec<u8> = scores.iter().map(|(_, i)| *i).collect();
    let weights: Vec<f32> = exps.iter().map(|e| e / sum).collect();
    (ids, weights)
}

/// Single-expert forward: dot product (simplest expert op).
pub fn expert_forward(hidden: &[f32; HIDDEN_DIM], expert_w: &[f32; HIDDEN_DIM]) -> f32 {
    hidden.iter().zip(expert_w.iter()).map(|(h, w)| h * w).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_selects_highest_logit() {
        let mut hidden = [0.0f32; HIDDEN_DIM];
        hidden[0] = 1.0;
        let mut weights = [[0.0f32; HIDDEN_DIM]; TOTAL_EXPERTS];
        // expert 7 has weight 1.0 on dim 0 → highest logit
        weights[7][0] = 1.0;
        weights[3][0] = 0.5;
        let (ids, _) = gate_topk(&hidden, &weights, 1);
        assert_eq!(ids, vec![7]);
    }

    #[test]
    fn topk_ordered_and_softmax_sums_to_1() {
        let mut hidden = [0.0f32; HIDDEN_DIM];
        hidden[0] = 1.0;
        let mut weights = [[0.0f32; HIDDEN_DIM]; TOTAL_EXPERTS];
        for e in 0..TOTAL_EXPERTS {
            weights[e][0] = e as f32 / 64.0;
        }
        let (ids, ws) = gate_topk(&hidden, &weights, 4);
        assert_eq!(ids.len(), 4);
        // descending by logit → ids descending expert index
        assert!(ids[0] > ids[1] && ids[1] > ids[2] && ids[2] > ids[3]);
        let sum: f32 = ws.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }
}
