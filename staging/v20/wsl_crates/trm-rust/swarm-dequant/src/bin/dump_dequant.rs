use std::io::{Read, Seek, SeekFrom, Write};
const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
fn main() {
    let m = swarm_gguf::parse_file(MODEL).unwrap();
    let t = m.tensors.iter().find(|t| t.name == "blk.3.ffn_down_exps.weight").unwrap();
    let n_per = 1408usize * 2048;
    let bytes_per = n_per / 32 * 34;
    let mut f = std::fs::File::open(MODEL).unwrap();
    let mut buf = vec![0u8; bytes_per];
    f.seek(SeekFrom::Start(m.data_start + t.offset + 63 * bytes_per as u64)).unwrap();
    f.read_exact(&mut buf).unwrap();
    std::fs::File::create("D:/llama.cpp-instella/build/bin/Release/attn_q_raw.bin").unwrap().write_all(&buf).unwrap();
    println!("wrote {} bytes (down expert 63)", buf.len());
}
