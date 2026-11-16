// debug: actual model's blk.0.ffn_gate first column — int8 dot vs dequant+f32 dot
use std::io::{Read, Seek, SeekFrom};
fn main() {
    let model = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
    let m = swarm_gguf::parse_file(model).expect("parse");
    let mut f = std::fs::File::open(model).expect("open");
    let t = m.tensors.iter().find(|t| t.name == "blk.0.ffn_gate.weight").unwrap();
    println!("type={} dims={:?}", t.ggml_type, t.dims);
    // load the first column (2048 values = 8 Q4_K blocks = 1152 bytes)
    let bytes = 2048 / 256 * 144;
    let mut buf = vec![0u8; bytes];
    f.seek(SeekFrom::Start(m.data_start + t.offset)).unwrap();
    f.read_exact(&mut buf).unwrap();
    // input: a 2048-dim vector
    let mut seed = 9u64;
    let x: Vec<f32> = (0..2048).map(|_| { seed = seed.wrapping_add(0x9E3779B97F4A7C15); let mut z = seed; z = (z ^ (z>>30)).wrapping_mul(0xBF58476D1CE4E5B9); z = (z ^ (z>>27)).wrapping_mul(0x94D049BB133111EB); ((z ^ (z>>31)) as f64 / u64::MAX as f64 * 2.0 - 1.0) as f32 }).collect();
    let wref = swarm_dequant::dequant_q4_k(&buf, 2048);
    let xq = swarm_dequant::quantize_q8_k(&x);
    let s = wref.iter().zip(&xq).map(|(a,b)| a*b).sum::<f32>();
    let fv = swarm_dequant::dot_q4_K_q8_K(&buf, &x);
    println!("dequant+f32 = {s:.6}   int8 = {fv:.6}   diff = {:.6}", (s-fv).abs());
}
