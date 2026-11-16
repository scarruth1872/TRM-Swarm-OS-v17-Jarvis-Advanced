const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
fn main() {
    let m = swarm_gguf::parse_file(MODEL).unwrap();
    let names = [
        "blk.0.ffn_gate.weight", "blk.0.ffn_up.weight", "blk.0.ffn_down.weight",
        "blk.1.ffn_gate_inp.weight", "blk.1.ffn_gate_exps.weight", "blk.1.ffn_up_exps.weight", "blk.1.ffn_down_exps.weight",
        "blk.1.ffn_gate_shexp.weight", "blk.1.ffn_up_shexp.weight", "blk.1.ffn_down_shexp.weight",
        "blk.1.attn_q.weight", "blk.1.attn_gate_proj.weight", "blk.1.attn_o_proj.weight",
        "blk.1.attn_kv_a_mqa.weight", "blk.1.attn_k_b.weight", "blk.1.attn_v_b.weight",
        "output.weight", "token_embd.weight",
    ];
    for n in names {
        match m.tensors.iter().find(|t| t.name == n) {
            Some(t) => println!("{n:38} type={} ne0={} ne1={}", t.ggml_type, t.dims[0], t.dims.get(1).copied().unwrap_or(0)),
            None => println!("{n:38} MISSING"),
        }
    }
}
