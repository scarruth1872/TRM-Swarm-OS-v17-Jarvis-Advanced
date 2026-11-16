//! swarm-gguf — native GGUF loader in Rust.
//!
//! Port of swarm_v2/core/gguf_loader.cpp with the CORRECTED format facts
//! (verified against the actual amd.Instella-MoE-16B-A3B-Think Q2_K file):
//!   - Header (24B): magic 'GGUF'(LE), version u32(=3), n_tensors u64(=457),
//!     n_kv u64(=57)  ← tensor count BEFORE kv count (llama.cpp writer order)
//!   - KV value types: llama.cpp gguf_type enum (0=u8 ... 12=f64)
//!   - Tensor types: ggml_type (0=F32 1=F16 2=Q4_0 10=Q2_K 11=Q3_K 14=Q6_K)
//!     (NOT the wrong GGUF_TENSOR_Q2_K==22 in the old header)
//!   - Tensor offsets RELATIVE to data_start = align32(end of tensor index)
//!
//! Verify gate: parse the REAL 6.5GB model, compare every tensor name,
//! shape, type, and offset against the C++ loader's report.

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

pub const GGUF_MAGIC: u32 = 0x4655_4747; // "GGUF" LE
// ggml_type
pub const GGML_F32: u32 = 0;
pub const GGML_F16: u32 = 1;
pub const GGML_Q4_0: u32 = 2;
pub const GGML_Q2_K: u32 = 10;
pub const GGML_Q3_K: u32 = 11;
pub const GGML_Q6_K: u32 = 14;

#[derive(Debug, Clone, PartialEq)]
pub struct GgufHeader {
    pub magic: u32,
    pub version: u32,
    pub n_tensors: u64,
    pub n_kv: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TensorInfo {
    pub name: String,
    pub n_dims: u32,
    pub dims: Vec<u64>,
    pub ggml_type: u32,
    pub offset: u64, // relative to data_start
}

#[derive(Debug)]
pub struct GgufModel {
    pub header: GgufHeader,
    pub kvs: Vec<(String, String)>, // name → debug repr of value
    pub tensors: Vec<TensorInfo>,
    pub data_start: u64,
    pub file_size: u64,
}

pub fn read_string(r: &mut impl Read) -> std::io::Result<String> {
    let len = r.read_u64::<LittleEndian>()?;
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

pub fn parse_file(path: &str) -> std::io::Result<GgufModel> {
    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let mut r = BufReader::new(file);

    // header
    let magic = r.read_u32::<LittleEndian>()?;
    let version = r.read_u32::<LittleEndian>()?;
    let n_tensors = r.read_u64::<LittleEndian>()?;
    let n_kv = r.read_u64::<LittleEndian>()?;
    let header = GgufHeader { magic, version, n_tensors, n_kv };
    assert_eq!(magic, GGUF_MAGIC, "not a GGUF file");

    // KV metadata (llama.cpp gguf_type)
    let mut kvs = Vec::new();
    for _ in 0..n_kv {
        let key = read_string(&mut r)?;
        let vtype = r.read_u32::<LittleEndian>()?;
        let val = match vtype {
            0 => format!("u8:{}", r.read_u8()?),
            1 => format!("i8:{}", r.read_i8()?),
            2 => format!("u16:{}", r.read_u16::<LittleEndian>()?),
            3 => format!("i16:{}", r.read_i16::<LittleEndian>()?),
            4 => format!("u32:{}", r.read_u32::<LittleEndian>()?),
            5 => format!("i32:{}", r.read_i32::<LittleEndian>()?),
            6 => format!("f32:{}", r.read_f32::<LittleEndian>()?),
            7 => format!("bool:{}", r.read_u8()? != 0),
            8 => format!("str:{}", read_string(&mut r)?),
            9 => {
                // array: type u32, count u64, then count values
                let atype = r.read_u32::<LittleEndian>()?;
                let acount = r.read_u64::<LittleEndian>()?;
                let mut items = Vec::new();
                for _ in 0..acount {
                    match atype {
                        0 => items.push(format!("{}", r.read_u8()?)),
                        1 => items.push(format!("{}", r.read_i8()?)),
                        4 => items.push(format!("{}", r.read_u32::<LittleEndian>()?)),
                        5 => items.push(format!("{}", r.read_i32::<LittleEndian>()?)),
                        6 => items.push(format!("{}", r.read_f32::<LittleEndian>()?)),
                        8 => items.push(read_string(&mut r)?),
                        10 => items.push(format!("{}", r.read_u64::<LittleEndian>()?)),
                        11 => items.push(format!("{}", r.read_i64::<LittleEndian>()?)),
                        _ => items.push(format!("?t{atype}")),
                    }
                }
                format!("arr[{}]:{}", acount, items.join(","))
            }
            10 => format!("u64:{}", r.read_u64::<LittleEndian>()?),
            11 => format!("i64:{}", r.read_i64::<LittleEndian>()?),
            12 => format!("f64:{}", r.read_f64::<LittleEndian>()?),
            _ => format!("?t{vtype}"),
        };
        kvs.push((key, val));
    }

    // tensor index
    let mut tensors = Vec::with_capacity(n_tensors as usize);
    for _ in 0..n_tensors {
        let name = read_string(&mut r)?;
        let n_dims = r.read_u32::<LittleEndian>()?;
        let mut dims = Vec::with_capacity(n_dims as usize);
        for _ in 0..n_dims {
            dims.push(r.read_u64::<LittleEndian>()?);
        }
        let ggml_type = r.read_u32::<LittleEndian>()?;
        let offset = r.read_u64::<LittleEndian>()?;
        tensors.push(TensorInfo { name, n_dims, dims, ggml_type, offset });
    }

    // data_start = align32(end of tensor index)
    let idx_end = r.stream_position()?;
    let data_start = (idx_end + 31) & !31;
    let _ = r.seek(SeekFrom::Start(data_start));

    Ok(GgufModel { header, kvs, tensors, data_start, file_size })
}

/// Look up a tensor's absolute byte offset in the file.
pub fn tensor_abs_offset(m: &GgufModel, name: &str) -> Option<u64> {
    m.tensors.iter().find(|t| t.name == name).map(|t| m.data_start + t.offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_layout_matches_cpp_doc() {
        // The C++ loader doc: header 24B, version 3, 457 tensors, 57 KVs
        // — verified against the real file. Parse the real model.
        let m = parse_file(
            &swarm_paths::resolve("/home/shawn/models/instella-gguf/Q2_K/amd.Instella-MoE-16B-A3B-Think.f16.gguf.Q2_K.gguf"),
        )
        .expect("parse real model");
        assert_eq!(m.header.version, 3);
        assert_eq!(m.header.n_tensors, 457);
        assert_eq!(m.header.n_kv, 57);
    }
}
