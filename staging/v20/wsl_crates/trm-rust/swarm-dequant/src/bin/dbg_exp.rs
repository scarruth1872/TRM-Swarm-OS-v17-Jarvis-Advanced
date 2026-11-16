use std::io::{Read, Seek, SeekFrom};
use swarm_dequant::f16_to_f32;
const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
fn main() {
    let m = swarm_gguf::parse_file(MODEL).unwrap();
    let mut f = std::fs::File::open(MODEL).unwrap();
    let t = m.tensors.iter().find(|t| t.name == "blk.3.ffn_down_exps.weight").unwrap();
    let n_per = 1408usize * 2048;
    let bytes_per = n_per / 32 * 34;
    let mut buf = vec![0u8; bytes_per];
    f.seek(SeekFrom::Start(m.data_start + t.offset + 63 * bytes_per as u64)).unwrap();
    f.read_exact(&mut buf).unwrap();
    let mut shown = 0;
    for b in 0..(bytes_per / 34) {
        let raw = u16::from_le_bytes([buf[b*34], buf[b*34+1]]);
        let d = f16_to_f32(raw);
        if d.is_nan() && shown < 3 {
            println!("block {b}: raw=0x{raw:04x} d={d} bytes=[{:02x} {:02x} {:02x} {:02x} {:02x} {:02x}]", buf[b*34], buf[b*34+1], buf[b*34+2], buf[b*34+3], buf[b*34+4], buf[b*34+5]);
            shown += 1;
        }
    }
    // also print the block right before the first NaN (to see the transition)
    let mut first = 0;
    for b in 0..(bytes_per/34) { if f16_to_f32(u16::from_le_bytes([buf[b*34], buf[b*34+1]])).is_nan() { first = b; break; } }
    if first > 0 {
        for bb in first-1..=first {
            let raw = u16::from_le_bytes([buf[bb*34], buf[bb*34+1]]);
            println!("block {bb}: raw=0x{raw:04x} d={}", f16_to_f32(raw));
        }
    }
    println!("expert63 down: total_blocks={} first_nan={first}", bytes_per/34);
}
