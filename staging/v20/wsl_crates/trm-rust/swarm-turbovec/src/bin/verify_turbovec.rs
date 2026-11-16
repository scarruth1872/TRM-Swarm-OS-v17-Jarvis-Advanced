//! Verify gate: swarm-turbovec vs the numpy/Python reference.
//! Packs 1M trits in Rust and compares byte-for-byte with the Python
//! pack_bits_fast output (canonical K3 v2), plus round-trip exactness.

use std::process::Command;
use std::io::Write;

fn main() {
    // 1. Rust pack of 1M deterministic trits → write to scratch file
    let mut state: u64 = 42;
    let n = 1_000_000usize;
    let mut trits = Vec::with_capacity(n);
    for _ in 0..n {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        trits.push(((state >> 33) % 3) as i8 - 1);
    }
    let packed = swarm_turbovec::pack(&trits);
    println!("Rust pack: {} trits → {} bytes ({}×)", n, packed.len(), n * 4 / packed.len());

    let packed_path = swarm_paths::resolve("/tmp/turbovec_rust_packed.bin");
    let mut f = std::fs::File::create(&packed_path).unwrap();
    f.write_all(&packed).unwrap();

    // 2. Python reference pack of the SAME data
    let py = r#"
import os, sys, numpy as np
sys.path.insert(0, os.environ["TRM_V17_ROOT"])
from swarm_v2.core.turbovec_ternary_pack import pack_bits_fast
state = 42
vals = []
for _ in range(1_000_000):
    state = (state * 6364136223846793005 + 1442695040888963407) & ((1<<64)-1)
    vals.append((state >> 33) % 3 - 1)
trits = np.array(vals, dtype=np.int64)
packed = pack_bits_fast(trits)
open(os.environ["TRM_PY_OUT"], "wb").write(packed.tobytes())
print("py packed bytes:", len(packed))
"#;
    // The reference tree and the output path are resolved HERE and handed to the snippet, and the
    // interpreter is whichever one this host has (TRM_PYTHON overrides; `py -3` on Windows).
    let v17 = swarm_paths::resolve("/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main");
    let py_out = swarm_paths::resolve("/tmp/turbovec_py_packed.bin");
    let (exe, pre): (String, Vec<String>) = match std::env::var("TRM_PYTHON") {
        Ok(p) if !p.trim().is_empty() => {
            // A configured interpreter may carry arguments ("py -3"). The first word is the program
            // and the rest are leading args; handing the whole string to Command::new makes the SPAWN
            // fail, which then reads as "python is broken" while the interpreter is perfectly fine.
            let mut it = p.split_whitespace();
            let exe = it.next().unwrap_or("python3").to_string();
            (exe, it.map(|s| s.to_string()).collect())
        }
        _ if cfg!(windows) => ("py".to_string(), vec!["-3".to_string()]),
        _ => ("python3".to_string(), Vec::new()),
    };
    let mut cmd = Command::new(&exe);
    for a in pre {
        cmd.arg(a);
    }
    let out = cmd
        .arg("-c")
        .arg(py)
        .env("TRM_V17_ROOT", &v17)
        .env("TRM_PY_OUT", &py_out)
        .output()
        .expect("run python");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    println!("Python: {}", stdout.trim());
    if !stderr.trim().is_empty() {
        println!("Python stderr: {}", stderr.trim().lines().last().unwrap_or(""));
    }

    // 3. Byte-for-byte comparison
    let py_bytes = std::fs::read(&py_out).expect("read py output");
    let identical = py_bytes == packed;
    println!("Rust {} bytes vs Python {} bytes", packed.len(), py_bytes.len());
    println!("BYTE-IDENTICAL: {identical}");

    if !identical {
        for i in 0..packed.len().min(py_bytes.len()) {
            if packed[i] != py_bytes[i] {
                println!("first diff at byte {i}: rust={:#04x} py={:#04x}", packed[i], py_bytes[i]);
                break;
            }
        }
        std::process::exit(1);
    }

    // 4. Round-trip exactness
    let back = swarm_turbovec::unpack(&packed, n);
    let exact = back == trits;
    println!("round-trip exact: {exact}");
    if exact {
        println!("VERIFY GATE: PASSED — byte-identical to numpy reference");
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
