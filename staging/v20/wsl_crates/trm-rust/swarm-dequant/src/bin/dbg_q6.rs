use std::io::{Read, Seek, SeekFrom};
use swarm_dequant::f16_to_f32;
const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
fn main() {
    let m = swarm_gguf::parse_file(MODEL).expect("parse");
    let t = m.tensors.iter().find(|t| t.name == "output.weight").expect("tensor");
    let n = t.dims.iter().product::<u64>() as usize;
    let bytes = n / 256 * 210;
    let mut f = std::fs::File::open(MODEL).expect("open");
    let mut buf = vec![0u8; 256];  // first block only
    f.seek(SeekFrom::Start(m.data_start + t.offset)).unwrap();
    f.read_exact(&mut buf).unwrap();
    let d = f16_to_f32(u16::from_le_bytes([buf[0], buf[1]]));
    println!("d = {d:.9}");
    println!("sc[0..8] (raw bytes at 194..202): {:?}", &buf[194..202]);
    println!("sc[0..8] as i8: {:?}", (194..202).map(|i| buf[i] as i8).collect::<Vec<_>>());
    println!("ql[0..4] = {:?}, qh[0..4] = {:?}", &buf[2..6], &buf[130..134]);
    // first q1
    let q1 = (((buf[2] & 0xF) as i32) | (((buf[130] >> 0) & 3) as i32) << 4) as i8 - 32;
    println!("q1 = {q1}, first value = d * sc[0] * q1 = {}", d * (buf[194] as i8 as f32) * q1 as f32);
}
