//! TRM Swarm OS - mcp_tool_evolver (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/mcp_tool_evolver.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use alloc::format;


//  ToolVersion 
#[derive(Debug, Clone)]
pub struct ToolVersion {
    pub version: String,
    pub code: String,
    pub timestamp: u64,
    pub trigger: u64,
    pub changes: Vec<String>,
    pub performance_score: f64,
    pub checksum: String,
}

impl ToolVersion {
    pub fn new() -> Self {
        ToolVersion {
            version: String::new(),
            code: String::new(),
            timestamp: 0,
            trigger: 0,
            changes: vec![],
            performance_score: 0.0,
            checksum: String::new(),
        }
    }
}

//  EvolutionResult 
#[derive(Debug, Clone)]
pub struct EvolutionResult {
    pub tool_name: String,
    pub old_version: String,
    pub new_version: String,
    pub status: u64,
    pub changes_made: Vec<String>,
    pub improvements: Vec<String>,
    pub error: String,
    pub timestamp: u64,
}

impl EvolutionResult {
    pub fn new() -> Self {
        EvolutionResult {
            tool_name: String::new(),
            old_version: String::new(),
            new_version: String::new(),
            status: 0,
            changes_made: vec![],
            improvements: vec![],
            error: String::new(),
            timestamp: 0,
        }
    }
}

//  get_tool_evolver(llm_generate) 
static SINGLETON_TOOL_EVOLVER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_tool_evolver() -> u64 {
    // Singleton handle (lazy-init pattern from mcp_tool_evolver.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_TOOL_EVOLVER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_TOOL_EVOLVER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ── The oracle's model (swarm_v2/core/mcp_tool_evolver.py) ───────────────────────────────────────
//
// `_calculate_checksum(code)` is `hashlib.md5(code.encode()).hexdigest()[:8]` -- so the port needs a
// real MD5, not a lookalike hash. The implementation below is the standard algorithm: 0x80 padding to
// 56 mod 64, the little-endian 64-bit bit length, the four registers, and the 64-round compression
// with the published K table and shift amounts. Checked against hashlib over a corpus, which is how a
// mistyped constant would have been caught.
//
// `_increment_version(current)` splits on "."; exactly three parts become `major.minor.(patch + 1)`,
// anything else becomes "0.1.1". The oracle calls `int(parts[2])` and would RAISE on a non-numeric
// patch, so this port returns None for that case rather than inventing a version.

/// The shift amounts, four per round type.
const MD5_S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
    5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

/// The published MD5 K table: floor(2^32 * abs(sin(i + 1))).
const MD5_K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

/// MD5 of a byte slice.
pub fn md5(input: &[u8]) -> [u8; 16] {
    let mut msg: Vec<u8> = Vec::with_capacity(input.len() + 72);
    msg.extend_from_slice(input);
    let bit_len = (input.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    let (mut a0, mut b0, mut c0, mut d0) =
        (0x67452301u32, 0xefcdab89u32, 0x98badcfeu32, 0x10325476u32);

    let mut off = 0usize;
    while off < msg.len() {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes([msg[off + i * 4], msg[off + i * 4 + 1], msg[off + i * 4 + 2], msg[off + i * 4 + 3]]);
        }
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let tmp = d;
            d = c;
            c = b;
            let sum = a.wrapping_add(f).wrapping_add(MD5_K[i]).wrapping_add(m[g]);
            b = b.wrapping_add(sum.rotate_left(MD5_S[i]));
            a = tmp;
        }
        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
        off += 64;
    }

    let mut out = [0u8; 16];
    for (i, v) in [a0, b0, c0, d0].iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
    }
    out
}

/// `_calculate_checksum`: the first eight hex characters of the MD5 digest.
pub fn calculate_checksum(code: &str) -> String {
    let d = md5(code.as_bytes());
    let mut s = String::with_capacity(8);
    for b in d.iter().take(4) {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// `_increment_version`; None where the oracle would raise on a non-numeric patch part.
pub fn increment_version(current: &str) -> Option<String> {
    let parts: Vec<&str> = current.split('.').collect();
    if parts.len() == 3 {
        let patch: u64 = parts[2].parse().ok()?;
        Some(format!("{}.{}.{}", parts[0], parts[1], patch + 1))
    } else {
        Some(String::from("0.1.1"))
    }
}

//  ToolSchema 
/// One row of `native_validate_tool_schemas`'s input — the tuple the oracle unpacks as
/// `t_id, req, min_l, max_l, def_p = s`.
#[derive(Debug, Clone, Copy)]
pub struct ToolSchema {
    pub tool_type_id: i32,   // t_id  — valid iff 1 <= t_id <= 5
    pub required: i32,       // req   — 1 = required
    pub min_len: i32,        // min_l
    pub max_len: i32,        // max_l
    pub default_present: i32, // def_p — 1 = a default is supplied
}

impl ToolSchema {
    pub fn new() -> Self {
        ToolSchema {
            tool_type_id: 0,
            required: 0,
            min_len: 0,
            max_len: 0,
            default_present: 0,
        }
    }
}

/// `native_validate_tool_schemas` (the Python FALLBACK, `_MCP_EVOLVER_DLL is None`).
///
/// Per schema row: valid iff `min_l <= max_l` and `1 <= t_id <= 5`; complexity is
/// `1.5` (required AND default present), `2.0` (required only), `1.0` (not required);
/// score is `(max_l - min_l) * 0.1 * complexity` for a valid row, else `0.0`.
///
/// Width note: the oracle's fallback computes the score in **f64** (Python floats) and
/// rounds once to f32 via `np.array(weights, dtype=np.float32)`. The shipped
/// `strip_mcp_evolver.dll` computes the same rule entirely in f32
/// (`(max-min) as f32 * 0.1f32 * complexity`), so the two disagree in the last ULP on
/// non-exact weights (e.g. a diff of 1 with complexity 1.5). This port matches the
/// fallback — f64 arithmetic, single f32 cast at the end — which is the oracle of
/// record. Returns `(valid_count, weights)` where `weights` is float32-valued.
pub fn validate_tool_schemas(schemas: &[ToolSchema]) -> (usize, Vec<f32>) {
    let mut valid = 0usize;
    let mut weights: Vec<f32> = Vec::with_capacity(schemas.len());
    for s in schemas {
        let is_v = s.min_len <= s.max_len && (1 <= s.tool_type_id && s.tool_type_id <= 5);
        let comp: f64 = if s.required == 1 && s.default_present == 1 {
            1.5
        } else if s.required == 1 {
            2.0
        } else {
            1.0
        };
        let score: f64 = if is_v {
            (s.max_len - s.min_len) as f64 * 0.1 * comp
        } else {
            0.0
        };
        weights.push(score as f32);
        if is_v {
            valid += 1;
        }
    }
    (valid, weights)
}

/// `_get_current_version`: the last version string for a tool, or `"0.1.0"` when the
/// tool is absent or its history is empty. `tool_versions` is the oracle's
/// `Dict[str, List[ToolVersion]]` in first-seen order.
pub fn get_current_version(
    tool_versions: &[(String, Vec<ToolVersion>)],
    tool_name: &str,
) -> String {
    for (name, versions) in tool_versions {
        if name == tool_name {
            if let Some(last) = versions.last() {
                return last.version.clone();
            }
            // Present but empty history falls through to the default, as the oracle's
            // `tool_name in self.tool_versions and self.tool_versions[tool_name]` does.
        }
    }
    String::from("0.1.0")
}

// ── Host-side (needs_host) surface — deliberately NOT ported ───────────────────────────
// The LLM/async/filesystem methods stay on the host: `analyze_tool_for_evolution` and
// `evolve_tool` call `self.llm_generate` (an awaitable model call — `needs_host`);
// `evolution_cycle`/`monitor_loop`/`start_tool_evolution` are asyncio loops; `__init__`
// (`os.makedirs`), `_load_versions`/`_save_versions`, `_get_current_tools`, and
// `rollback_tool` all do file I/O; `get_status`/`get_tool_history`/
// `get_available_knowledge` reach into the reconnaissance daemon and global memory.
// `to_dict` on ToolVersion/EvolutionResult is JSON serialization and stays host-side.
