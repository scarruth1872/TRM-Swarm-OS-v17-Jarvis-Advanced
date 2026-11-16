use std::io::{Read, Seek, SeekFrom};
const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
fn main() {
    let m = swarm_gguf::parse_file(MODEL).unwrap();
    let mut f = std::fs::File::open(MODEL).unwrap();
    for il in [1, 3, 26] {
        let t = m.tensors.iter().find(|t| t.name == format!("blk.{il}.exp_probs_b.bias")).unwrap();
        let mut buf = vec![0u8; 64*4];
        f.seek(SeekFrom::Start(m.data_start + t.offset)).unwrap();
        f.read_exact(&mut buf).unwrap();
        let vals: Vec<f32> = buf.chunks_exact(4).map(|c| f32::from_le_bytes([c[0],c[1],c[2],c[3]])).collect();
        let mn = vals.iter().cloned().fold(f32::INFINITY, f32::min);
        let mx = vals.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        println!("blk.{il}.exp_probs_b: min={mn:.3} max={mx:.3} first4={:?}", &vals[..4]);
    }
}
