//! TRM Swarm OS — zero_human_test_gen (no_std port).
//! Deterministic core ported 1:1 from swarm_v2/core/zero_human_test_gen.py:
//! test-type/status enums, endpoint → sanitized test-function naming, code →
//! function-name extraction, per-suite test counts and coverage sets, file
//! naming for the four generated suites, the get_stats() roll-ups and the
//! singleton accessors. Actual pytest/Playwright code emission, file writes
//! and subprocess test runs are // MANUAL (no_std has no fs or process).

#![allow(dead_code)]

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

// TestType enum values (python: pytest_unit, pytest_api, playwright_e2e,
// integration).
pub const TYPE_PYTEST_UNIT: u8 = 0;
pub const TYPE_PYTEST_API: u8 = 1;
pub const TYPE_PLAYWRIGHT_E2E: u8 = 2;
pub const TYPE_INTEGRATION: u8 = 3;

// TestStatus enum values (python: generated, passed, failed, pending).
pub const STATUS_GENERATED: u8 = 0;
pub const STATUS_PASSED: u8 = 1;
pub const STATUS_FAILED: u8 = 2;
pub const STATUS_PENDING: u8 = 3;

/// Map a test-type code to its python string value.
pub fn test_type_str(t: u8) -> &'static str {
    match t {
        TYPE_PYTEST_API => "pytest_api",
        TYPE_PLAYWRIGHT_E2E => "playwright_e2e",
        TYPE_INTEGRATION => "integration",
        _ => "pytest_unit",
    }
}

/// Map a test-status code to its python string value.
pub fn test_status_str(s: u8) -> &'static str {
    match s {
        STATUS_PASSED => "passed",
        STATUS_FAILED => "failed",
        STATUS_PENDING => "pending",
        _ => "generated",
    }
}

/// Lowercased tool name — python uses tool_name.lower() in file paths.
pub fn tool_name_lower(tool_name: &str) -> String {
    tool_name.to_lowercase()
}

// ---------------------------------------------------------------------------
// Naming helpers (deterministic core)
// ---------------------------------------------------------------------------

/// Sanitize a path fragment into a python identifier fragment: replace every
/// non-alphanumeric char with '_' (python re.sub(r'[^a-zA-Z0-9]', '_', path)).
pub fn sanitize_path_segment(path: &str) -> String {
    let mut out = String::new();
    for c in path.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    out
}

/// safe_fn = sanitize(path.strip('/')) or "root".
pub fn safe_function_name(path: &str) -> String {
    let stripped = path.trim_start_matches('/').trim_end_matches('/');
    let safe = sanitize_path_segment(stripped);
    if safe.is_empty() {
        "root".to_string()
    } else {
        safe
    }
}

/// API test function name: `test_{method}_{safe_fn}` where method is
/// lowercased in the python API generator.
pub fn api_test_fn_name(method: &str, path: &str) -> String {
    let mut out = String::new();
    out.push_str("test_");
    out.push_str(&method.to_lowercase());
    out.push('_');
    out.push_str(&safe_function_name(path));
    out
}

/// Playwright E2E test name: replaces '/' with '_', strips underscores,
/// defaults to "root".
pub fn e2e_test_fn_name(path: &str) -> String {
    let replaced = path.replace('/', "_");
    let trimmed = replaced.trim_matches('_');
    if trimmed.is_empty() {
        "test_e2e_root".to_string()
    } else {
        let mut out = String::new();
        out.push_str("test_e2e_");
        out.push_str(trimmed);
        out
    }
}

/// Python title-case class name fragment: tool_name.replace('_','').title().
/// Faithful for ASCII identifiers (python .title() capitalizes the first
/// alpha char of each word after non-alpha splits; for tool ids this equals
/// simple capitalization of the underscore-stripped name).
pub fn tool_class_fragment(tool_name: &str) -> String {
    let stripped: String = tool_name.replace('_', "");
    let mut chars = stripped.chars();
    match chars.next() {
        Some(first) => {
            let mut out = String::new();
            out.extend(first.to_uppercase());
            for c in chars {
                out.push_str(&c.to_lowercase().to_string());
            }
            out
        }
        None => String::new(),
    }
}

/// Class names of the generated suites:
/// Test{tool}API / Test{tool}Unit / Test{tool}E2E / Test{tool}Integration.
pub fn suite_class_name(tool_name: &str, suite_kind: u8) -> String {
    let mut out = String::new();
    out.push_str("Test");
    out.push_str(&tool_class_fragment(tool_name));
    match suite_kind {
        TYPE_PYTEST_API => out.push_str("API"),
        TYPE_PYTEST_UNIT => out.push_str("Unit"),
        TYPE_PLAYWRIGHT_E2E => out.push_str("E2E"),
        _ => out.push_str("Integration"),
    }
    out
}

/// Generated file path: test_api_<tool>.py, test_unit_<tool>.py,
/// test_playwright_<tool>.py, test_integration_<tool>.py.
pub fn suite_filename(tool_name: &str, suite_kind: u8) -> String {
    let lower = tool_name_lower(tool_name);
    match suite_kind {
        TYPE_PYTEST_API => format!("test_api_{}.py", lower),
        TYPE_PYTEST_UNIT => format!("test_unit_{}.py", lower),
        TYPE_PLAYWRIGHT_E2E => format!("test_playwright_{}.py", lower),
        _ => format!("test_integration_{}.py", lower),
    }
}

// ---------------------------------------------------------------------------
// Endpoint parsing helpers (deterministic core)
// ---------------------------------------------------------------------------

/// Normalized endpoint path: default "/" when missing (ep["path"] default).
pub fn ep_path(path: Option<&str>) -> String {
    match path {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => "/".to_string(),
    }
}

/// Normalized endpoint method, upper-cased for coverage labels (python
/// lowercases for the function name and upper-cases for coverage).
pub fn ep_method(method: Option<&str>) -> String {
    match method {
        Some(m) if !m.is_empty() => m.to_uppercase(),
        _ => "GET".to_string(),
    }
}

/// coverage label "{METHOD} {path}" (used by the API generator).
pub fn coverage_label(method: &str, path: &str) -> String {
    let mut out = String::new();
    out.push_str(&method.to_uppercase());
    out.push(' ');
    out.push_str(path);
    out
}

// ---------------------------------------------------------------------------
// Code → function extraction (deterministic core)
// ---------------------------------------------------------------------------

/// Extract function names from tool code (python regex
/// `async def (\w+)\(|def (\w+)\(`), excluding `_`-prefixed, health and
/// mcp_manifest. Returns up to `limit` names.
pub fn extract_functions(code: &str, limit: usize) -> Vec<String> {
    let mut functions = vec![];
    let bytes = code.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // look for "def " and capture the following identifier
        if bytes[i..].starts_with(b"def ") {
            let mut j = i + 4;
            let mut name = String::new();
            while j < bytes.len() && bytes[j].is_ascii_alphanumeric() || (j < bytes.len() && bytes[j] == b'_')
            {
                name.push(bytes[j] as char);
                j += 1;
            }
            if !name.is_empty() {
                let skip_private = name.starts_with('_');
                let skip_reserved = name == "health" || name == "mcp_manifest";
                if !skip_private && !skip_reserved {
                    functions.push(name);
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    if functions.len() > limit {
        functions.truncate(limit);
    }
    functions
}

/// Function-availability filter used by the unit generator: skip names
/// starting with '_' and the reserved health/mcp_manifest names.
pub fn unit_test_allowed(fn_name: &str) -> bool {
    !fn_name.starts_with('_') && fn_name != "health" && fn_name != "mcp_manifest"
}

// ---------------------------------------------------------------------------
// Per-suite counts + coverage sets (deterministic core)
// ---------------------------------------------------------------------------

/// pytest API suite: 1 test per endpoint + 4 fixed module functions
/// (health endpoint, manifest, 3 class methods minus the shared 2 -> python
/// test_count == len(test_functions)+4 with test_functions == endpoints+3
/// module funcs; total endpoint-driven + 4).
pub fn api_test_count(endpoint_count: u64) -> u64 {
    endpoint_count + 4
}

/// pytest unit suite: python test_functions = extracted functions + 2 extra
/// module-level functions (mock response + endpoint parsing); test_count =
/// len(test_functions) + 2 (class methods) == function_count + 4.
pub fn unit_test_count(function_count: u64) -> u64 {
    function_count + 4
}

/// Playwright suite: GET endpoints (≤5) + 3 fixed class methods.
pub fn playwright_test_count(get_endpoint_count: u64) -> u64 {
    get_endpoint_count + 3
}

/// Integration suite: fixed 8-test workflow.
pub fn integration_test_count() -> u64 {
    8
}

/// Collect covered endpoints in first-seen order, skipping duplicates.
pub fn collect_covered_endpoints(mut sets: Vec<Vec<String>>) -> Vec<String> {
    let mut seen: Vec<String> = vec![];
    for set in sets.drain(..) {
        for ep in set {
            if !seen.contains(&ep) {
                seen.push(ep);
            }
        }
    }
    seen
}

/// coverage_percentage: 100.0 when any endpoint covered, else 0.0.
pub fn coverage_percentage(covered_count: u64) -> f64 {
    if covered_count > 0 {
        100.0
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Per-tool registry state + get_stats roll-up
// ---------------------------------------------------------------------------

/// One generated suite's deterministic metadata (python GeneratedTestSuite).
#[derive(Clone, Debug)]
pub struct TestSuiteMeta {
    pub tool_name: String,
    pub test_type: u8,
    pub file_path: String,
    pub test_count: u64,
    pub coverage_endpoints: Vec<String>,
    pub status: u8,
}

/// Tool-level test-generation registry (python generated_suites[tool]).
#[derive(Clone, Debug)]
pub struct ToolTestRegistry {
    pub tool_name: String,
    pub suites: Vec<TestSuiteMeta>,
}

impl ToolTestRegistry {
    /// python generate_all_tests: pytest API + unit + playwright + integration.
    pub fn from_endpoints(
        tool_name: &str,
        endpoints: &[(&str, &str)], // (path, method)
        code: &str,
    ) -> Self {
        let ep_count = endpoints.len() as u64;
        let mut api_coverage: Vec<String> = vec![];
        for (path, method) in endpoints {
            api_coverage.push(coverage_label(method, path));
        }

        let functions = extract_functions(code, 10);
        let unit_coverage: Vec<String> = functions.clone();

        let mut get_paths: Vec<&str> = vec![];
        for (path, method) in endpoints {
            if method.eq_ignore_ascii_case("get") {
                get_paths.push(path);
                if get_paths.len() >= 5 {
                    break;
                }
            }
        }
        let mut e2e_coverage: Vec<String> = vec![];
        for p in &get_paths {
            e2e_coverage.push(format!("GET {}", p));
        }

        let lower = tool_name_lower(tool_name);
        let mut suites = vec![];
        suites.push(TestSuiteMeta {
            tool_name: tool_name.to_string(),
            test_type: TYPE_PYTEST_API,
            file_path: format!("test_api_{}.py", lower),
            test_count: api_test_count(ep_count),
            coverage_endpoints: api_coverage,
            status: STATUS_GENERATED,
        });
        suites.push(TestSuiteMeta {
            tool_name: tool_name.to_string(),
            test_type: TYPE_PYTEST_UNIT,
            file_path: format!("test_unit_{}.py", lower),
            test_count: unit_test_count(functions.len() as u64),
            coverage_endpoints: unit_coverage,
            status: STATUS_GENERATED,
        });
        suites.push(TestSuiteMeta {
            tool_name: tool_name.to_string(),
            test_type: TYPE_PLAYWRIGHT_E2E,
            file_path: format!("test_playwright_{}.py", lower),
            test_count: playwright_test_count(get_paths.len() as u64),
            coverage_endpoints: e2e_coverage,
            status: STATUS_GENERATED,
        });
        suites.push(TestSuiteMeta {
            tool_name: tool_name.to_string(),
            test_type: TYPE_INTEGRATION,
            file_path: format!("test_integration_{}.py", lower),
            test_count: integration_test_count(),
            coverage_endpoints: vec![
                "full_workflow".to_string(),
                "health".to_string(),
                "manifest".to_string(),
                "concurrency".to_string(),
            ],
            status: STATUS_GENERATED,
        });
        ToolTestRegistry {
            tool_name: tool_name.to_string(),
            suites,
        }
    }

    /// get_coverage_report roll-up: total suites/tests + unique covered
    /// endpoints in first-seen order + percentage.
    pub fn coverage_report(&self) -> (u64, u64, Vec<String>, f64) {
        let total_suites = self.suites.len() as u64;
        let mut total_tests = 0u64;
        let mut sets: Vec<Vec<String>> = vec![];
        for s in &self.suites {
            total_tests += s.test_count;
            sets.push(s.coverage_endpoints.clone());
        }
        let covered = collect_covered_endpoints(sets);
        let pct = coverage_percentage(covered.len() as u64);
        (total_suites, total_tests, covered, pct)
    }

    /// Aggregate stats across all registered tools: (tools, suites, tests).
    pub fn aggregate_stats(registries: &[ToolTestRegistry]) -> (u64, u64, u64) {
        let mut suites = 0u64;
        let mut tests = 0u64;
        for r in registries {
            suites += r.suites.len() as u64;
            for s in &r.suites {
                tests += s.test_count;
            }
        }
        (registries.len() as u64, suites, tests)
    }
}

// ---------------------------------------------------------------------------
// Singleton + module hooks
// ---------------------------------------------------------------------------

/// get_test_generator singleton — mirror of the python lazy _test_gen.
pub fn get_test_generator() -> u64 {
    0x5A45_524F_5445_5354
}

/// Deterministic tool identity (FNV-1a of the tool name) used where python
/// keys dicts by tool_name.
pub fn tool_id(tool_name: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in tool_name.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

// ---------------------------------------------------------------------------
// // MANUAL — pytest/Playwright code emission, fs writes, subprocess runs
// ---------------------------------------------------------------------------

/// _generate_* test-code text builders (large f-string bodies, fs-write
/// only). // MANUAL -> "".
pub fn generate_test_code_manual() -> String {
    String::new()
}

/// run_tests: subprocess pytest execution. // MANUAL -> (0 passed, 0 failed).
pub fn run_tests_manual() -> (u64, u64) {
    (0, 0)
}

/// on_tool_synthesized: prints + file writes. // MANUAL -> false.
pub fn on_tool_synthesized_manual() -> bool {
    false
}
