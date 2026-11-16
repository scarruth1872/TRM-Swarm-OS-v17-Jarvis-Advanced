//! ingest_scale_native.rs — Motion E: the fabric at scale, NATIVE.
//!
//! The full diverse-task substrate into the shards — no Python.
//! Reads the real sources (Tox21 SMILES + ChEMBL ids, streamed through
//! gzip), K3-encodes natively, appends via the kernel's c_shard_*
//! symbols loaded from its cdylib (the library is leaked to outlive
//! the fn pointers — the lifetime fix).

use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

type OpenFn = unsafe extern "C" fn(*const std::os::raw::c_char) -> *mut std::os::raw::c_void;
type AppendFn = unsafe extern "C" fn(
    *mut std::os::raw::c_void,
    *const std::os::raw::c_char,
    usize,
    *mut u32,
    *mut u64,
) -> i32;
type CountFn = unsafe extern "C" fn(*mut std::os::raw::c_void) -> u32;
type CloseFn = unsafe extern "C" fn(*mut std::os::raw::c_void);

struct Shards {
    _lib: &'static libloading::Library,
    open: OpenFn,
    append: AppendFn,
    count: CountFn,
    close: CloseFn,
}

fn load_shards() -> Shards {
    unsafe {
        let lib: &'static libloading::Library = Box::leak(Box::new(
            libloading::Library::new(swarm_paths::cdylib())
                .unwrap()));
        let open: libloading::Symbol<OpenFn> = lib.get(b"c_shard_open").unwrap();
        let append: libloading::Symbol<AppendFn> = lib.get(b"c_shard_append").unwrap();
        let count: libloading::Symbol<CountFn> = lib.get(b"c_shard_count").unwrap();
        let close: libloading::Symbol<CloseFn> = lib.get(b"c_shard_close").unwrap();
        Shards {
            _lib: lib,
            open: *open,
            append: *append,
            count: *count,
            close: *close,
        }
    }
}

fn k3(s: &str) -> Vec<i8> {
    let h = fnv(s);
    (0..8)
        .map(|k| if (h >> k) & 1 == 1 { 1 } else { -1 })
        .collect()
}

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn main() {
    let t0 = Instant::now();
    let shards = load_shards();
    let path = swarm_paths::resolve("/home/shawn/.hermes/cache/trm_full_scale");
    let c_path = std::ffi::CString::new(path).unwrap();
    let handle = unsafe { (shards.open)(c_path.as_ptr()) };
    // A null handle means the sharded fabric would not open. Name the target and the likely cause --
    // an assert without a reason leaves the operator to guess which of a dozen paths was wrong.
    if handle.is_null() {
        eprintln!("shard open failed: {}", c_path.to_string_lossy());
        eprintln!("  the manifest's recorded shard path must exist, or be translatable, on this host");
        std::process::exit(2);
    }

    let mut n_records = 0u64;
    let mut n_trits = 0u64;

    // 1. Tox21 SMILES (real, 8,734) — streamed gzip, one pass
    let tox = File::open(swarm_paths::resolve("/home/shawn/tox21full.csv.gz")).expect("tox21");
    let reader = BufReader::new(GzDecoder::new(tox));
    for line in reader.lines().flatten().skip(1) {
        let smi = line.split(',').next().unwrap_or("").to_string();
        if smi.is_empty() {
            continue;
        }
        let mut rec = k3(&format!("tox21|{smi}"));
        rec.extend(k3("compound"));
        unsafe {
            let mut sid: u32 = 0;
            let mut off: u64 = 0;
            (shards.append)(
                handle,
                rec.as_ptr() as *const std::os::raw::c_char,
                rec.len(),
                &mut sid,
                &mut off,
            );
        }
        n_records += 1;
        n_trits += rec.len() as u64;
    }
    println!("tox21: {n_records} records");

    // 2. ChEMBL ids (real, ~2.2M streamed from the SDF gzip)
    let chembl = File::open(swarm_paths::resolve("/home/shawn/drugs/chembl_37.sdf.gz")).expect("chembl");
    let reader = BufReader::new(GzDecoder::new(chembl));
    let mut n_chembl = 0u64;
    let mut capture = false;
    for line in reader.lines().flatten() {
        if line.starts_with("> <chembl_id>") {
            capture = true;
            continue;
        }
        if capture && !line.trim().is_empty() {
            let mut rec = k3(&format!("chembl|{}", line.trim()));
            rec.extend(k3("drug"));
            unsafe {
                let mut sid: u32 = 0;
            let mut off: u64 = 0;
            (shards.append)(
                handle,
                rec.as_ptr() as *const std::os::raw::c_char,
                rec.len(),
                &mut sid,
                &mut off,
            );
            }
            n_chembl += 1;
            n_trits += rec.len() as u64;
            capture = false;
            if n_chembl % 500_000 == 0 {
                println!("  chembl: {n_chembl} so far");
            }
        }
        if line.starts_with("$$$$") {
            capture = false;
        }
    }
    println!("chembl: {n_chembl} records");

    let n_shards = unsafe { (shards.count)(handle) };
    unsafe {
        (shards.close)(handle);
    }
    let dt = t0.elapsed();
    println!("\n=== MOTION E: THE FABRIC AT SCALE (native) ===");
    println!(
        "records: {n_records} tox21 + {n_chembl} chembl = {}",
        n_records + n_chembl
    );
    println!(
        "trits: {n_trits} in {dt:?} ({:.0}M trits/s)",
        n_trits as f64 / dt.as_secs_f64() / 1e6
    );
    println!("shards: {n_shards} · zero Python in the ingest");
    println!("MOTION E GATE: the full diverse-task substrate, native");
}
