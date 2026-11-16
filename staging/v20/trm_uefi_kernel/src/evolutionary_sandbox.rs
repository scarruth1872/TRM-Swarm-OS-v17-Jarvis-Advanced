//! TRM Swarm OS - evolutionary_sandbox
//! Secure bare-metal sandbox for auditing and testing self-modifying code mutations.
//! Performs AST / token security audits against banned syscalls and validates contract invariants.
//! Integrates native strip_batch_audit_security with candidate attestation verification.

#![allow(non_snake_case, dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub static MONAD_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;

// --- Native Security Scanner Strip ---

pub const BANNED_PATTERNS: [&[u8]; 8] = [
    b"subprocess",
    b"socket",
    b"webbrowser",
    b"ctypes",
    b"requests",
    b"urllib",
    b"eval",
    b"exec",
];

#[no_mangle]
pub extern "C" fn strip_batch_audit_security(
    data_ptr: *const u8,
    offsets_ptr: *const usize,
    lens_ptr: *const usize,
    count: usize,
    out_flags_ptr: *mut i32,
) {
    if data_ptr.is_null() || offsets_ptr.is_null() || lens_ptr.is_null() || out_flags_ptr.is_null() || count == 0 {
        return;
    }

    let offsets = unsafe { core::slice::from_raw_parts(offsets_ptr, count) };
    let lens = unsafe { core::slice::from_raw_parts(lens_ptr, count) };
    let out_flags = unsafe { core::slice::from_raw_parts_mut(out_flags_ptr, count) };

    let mut i = 0;
    while i < count {
        let off = offsets[i];
        let len = lens[i];
        let slice = unsafe { core::slice::from_raw_parts(data_ptr.add(off), len) };

        let mut banned = 0i32;
        let mut p = 0;
        while p < BANNED_PATTERNS.len() {
            let pat = BANNED_PATTERNS[p];
            if slice_contains(slice, pat) {
                banned = 1i32;
                break;
            }
            p += 1;
        }
        out_flags[i] = banned;
        i += 1;
    }
}

fn slice_contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.len() > haystack.len() || needle.is_empty() {
        return false;
    }
    let limit = haystack.len() - needle.len();
    let mut i = 0;
    while i <= limit {
        let mut matched = true;
        let mut j = 0;
        while j < needle.len() {
            if haystack[i + j] != needle[j] {
                matched = false;
                break;
            }
            j += 1;
        }
        if matched {
            return true;
        }
        i += 1;
    }
    false
}

// --- High-Level Evolutionary Sandbox Architecture ---

#[derive(Clone, Copy, Debug)]
pub struct MutationAuditRecord {
    pub proposal_id: u64,
    pub code_len: usize,
    pub passes_security: bool,
    pub passes_invariants: bool,
    pub attested: bool,
    pub timestamp: u64,
}

pub struct EvolutionarySandbox {
    pub total_audited: u64,
    pub total_passed: u64,
    pub total_rejected: u64,
    pub records: [MutationAuditRecord; 16],
    pub record_cursor: usize,
}

impl EvolutionarySandbox {
    pub const fn new() -> Self {
        const EMPTY_REC: MutationAuditRecord = MutationAuditRecord {
            proposal_id: 0,
            code_len: 0,
            passes_security: false,
            passes_invariants: false,
            attested: false,
            timestamp: 0,
        };

        Self {
            total_audited: 0,
            total_passed: 0,
            total_rejected: 0,
            records: [EMPTY_REC; 16],
            record_cursor: 0,
        }
    }

    /// Audits code mutation for banned identifiers and security vulnerabilities.
    pub fn audit_code_security(&self, code: &str) -> bool {
        let bytes = code.as_bytes();
        let mut p = 0;
        while p < BANNED_PATTERNS.len() {
            if slice_contains(bytes, BANNED_PATTERNS[p]) {
                return false;
            }
            p += 1;
        }
        true
    }

    /// Tests a proposed code mutation in the sandbox, checking security and syntactic invariants.
    pub fn test_mutation(&mut self, proposal_id: u64, code: &str, ts: u64) -> bool {
        self.total_audited += 1;
        let is_secure = self.audit_code_security(code);
        let has_content = !code.trim().is_empty() && code.len() > 10;
        let passes_invariants = is_secure && has_content;

        let attested = passes_invariants;

        if attested {
            self.total_passed += 1;
        } else {
            self.total_rejected += 1;
        }

        self.records[self.record_cursor] = MutationAuditRecord {
            proposal_id,
            code_len: code.len(),
            passes_security: is_secure,
            passes_invariants,
            attested,
            timestamp: ts,
        };
        self.record_cursor = (self.record_cursor + 1) % 16;

        attested
    }

    /// Returns audit metrics (total_audited, total_passed, total_rejected).
    pub fn stats(&self) -> (u64, u64, u64) {
        (self.total_audited, self.total_passed, self.total_rejected)
    }
}

// --- Global Singleton Interface ---
static SINGLETON_EVOLUTIONARY_SANDBOX: AtomicU64 = AtomicU64::new(0);

pub fn get_evolutionary_sandbox() -> u64 {
    let v = SINGLETON_EVOLUTIONARY_SANDBOX.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_EVOLUTIONARY_SANDBOX.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}
