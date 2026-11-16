//! TRM Swarm OS - mcp_bus (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/mcp_bus.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;


//  SchemaSpec -- the portable part of python's `input_schema` / `output_schema`
//  dicts (JSON Schema). Every oracle code path reads exactly one key of them:
//  `input_schema.get("required", [])` in `_validate_input`; `output_schema` is
//  carried but never inspected. The generated port had these as
//  `Vec<(String, f64)>` (a name->float map), which cannot express `required`
//  and had no python counterpart -- replaced with the faithful form.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SchemaSpec {
    pub required: Vec<String>,
}

impl SchemaSpec {
    pub fn new() -> Self {
        SchemaSpec { required: Vec::new() }
    }
    /// python: `{"required": [...]}`.
    pub fn from_required(names: &[&str]) -> Self {
        SchemaSpec {
            required: names.iter().map(|n| String::from(*n)).collect(),
        }
    }
    /// python truthiness of the schema dict (`not schema.input_schema`).
    /// An empty dict and a dict without "required" are observationally
    /// identical in the oracle (both yield `required = []` and no error), so
    /// this reduces to "no required names".
    pub fn is_empty(&self) -> bool {
        self.required.is_empty()
    }
}

//  ToolSchema 
#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: SchemaSpec,
    pub output_schema: SchemaSpec,
    pub provider: String,
    pub version: String,
    pub registered_at: String,
}

impl ToolSchema {
    pub fn new() -> Self {
        ToolSchema {
            name: String::new(),
            description: String::new(),
            input_schema: SchemaSpec::new(),
            output_schema: SchemaSpec::new(),
            provider: String::new(),
            version: String::new(),
            registered_at: String::new(),
        }
    }
}

//  ToolCallResult 
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub success: bool,
    pub result: u64,
    pub error: String,
    pub elapsed_ms: f64,
}

impl ToolCallResult {
    pub fn new() -> Self {
        ToolCallResult {
            tool_name: String::new(),
            success: false,
            result: 0,
            error: String::new(),
            elapsed_ms: 0.0,
        }
    }
}

//  get_mcp_bus() 
static SINGLETON_MCP_BUS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_mcp_bus() -> u64 {
    // Singleton handle (lazy-init pattern from mcp_bus.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_MCP_BUS.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_MCP_BUS.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}


// ═══════════════════════════════════════════════════════════════════════════
// MCPBus — deterministic core of swarm_v2/core/mcp_bus.py (verified).
//
// Every function below is named after the oracle. Rules reproduced:
//   register_tool   dict semantics: re-registering an existing name REPLACES the
//                   value in place, so the insertion position (and therefore the
//                   list_tools order) is preserved. `if handler:` = a falsy
//                   handler cannot exist in python, so a handler updates the
//                   handler table and an omitted handler LEAVES THE OLD ONE.
//   unregister_tool removes schema AND handler; returns whether the name existed.
//   list_tools      insertion order; the provider filter is python-truthy, so a
//                   filter of "" / None returns every tool. The returned view
//                   omits `registered_at` (the oracle's dict omits it too).
//   validate_input  (_validate_input) no schema, or an EMPTY input_schema, skips
//                   validation; otherwise returns the FIRST missing required
//                   name in declaration order as "Missing required field: <name>".
//   call_tool       total_calls += 1 BEFORE any check; check order is
//                   not-found -> validation -> no-handler; those three early
//                   returns increment failed_calls, report elapsed_ms = 0.0 and
//                   are NEVER appended to the call log. Only a call that reaches
//                   the handler (success or raised) is logged.
//   connect_remote  registers "{server_name}/{tool['name']}" with
//                   provider = server_name, returns the imported count, and
//                   RESETS the server entry (url/connected_at/tool_count) even if
//                   the server was connected before -- previously imported tools
//                   stay in the registry.
//   get_stats       local = provider == "local", remote = provider != "local"
//                   (a server literally named "local" therefore counts as
//                   local_tools), recent_calls = last 10 log entries.
//   call log        when it grows past 200 entries it collapses to the LAST 100.
//
// HOST-SIDE (deliberately NOT ported): the handler callable itself and its
// `handler(**args)` invocation, the `asyncio.iscoroutinefunction` sync/async
// dispatch and the `except Exception` capture -- the caller injects the handler
// outcome. Also: `logging`, `datetime.now().isoformat()` timestamps (passed in
// as empty strings here), the ctypes CDLL strip loader, and `json` (imported by
// the oracle but never used).
// ═══════════════════════════════════════════════════════════════════════════

// ─── Constants (1:1 python literals) ───
/// python `provider: str = "local"`.
pub const DEFAULT_PROVIDER: &str = "local";
/// python ToolSchema `version: str = "1.0.0"`.
pub const DEFAULT_VERSION: &str = "1.0.0";
/// python: `if len(self._call_log) > 200`.
pub const CALL_LOG_MAX: usize = 200;
/// python: `self._call_log = self._call_log[-100:]`.
pub const CALL_LOG_KEEP: usize = 100;
/// python: `self._call_log[-10:]` in get_stats.
pub const RECENT_CALLS: usize = 10;
/// python: `(datetime.now() - start).total_seconds() * 1000`.
pub const MS_PER_SECOND: f64 = 1000.0;

/// python: `(datetime.now() - start).total_seconds() * 1000`.
pub fn elapsed_ms_between(start_secs: f64, end_secs: f64) -> f64 {
    (end_secs - start_secs) * MS_PER_SECOND
}

/// Dekker two-product: returns `(p, e)` with `p = fl(a * b)` and
/// `p + e == a * b` EXACTLY (plain f64 arithmetic, no FMA, no std). Requires no
/// overflow in the split, so the caller must keep `|a|` small (round2 guards it).
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    const SPLITTER: f64 = 134_217_729.0; // 2^27 + 1
    let p = a * b;
    let ca = SPLITTER * a;
    let a_hi = ca - (ca - a);
    let a_lo = a - a_hi;
    let cb = SPLITTER * b;
    let b_hi = cb - (cb - b);
    let b_lo = b - b_hi;
    let e = ((a_hi * b_hi - p) + a_hi * b_lo + a_lo * b_hi) + a_lo * b_lo;
    (p, e)
}

/// no_std floor: core's `f64::floor` is NOT available in a `#![no_std]` kernel
/// (it lowers to a libm intrinsic, like `f64::round_ties_even` / `f64::mul_add`,
/// and rustc rejects those methods outright without std). Hand-rolled from the
/// saturating `as i64` truncation + a correction for negatives; exact for
/// `|x| < 2^63`, which covers every value round2 can hand it (`|x| < 2^52`).
fn floor_f64(x: f64) -> f64 {
    let t = x as i64 as f64; // truncates toward zero
    if x < 0.0 && t != x {
        t - 1.0
    } else {
        t
    }
}

/// no_std half-even rounding to an integer (same reason as `floor_f64`).
/// Only called on the small `two_prod` residual (`|x| <= ulp(p)/2`), where the
/// `as i64` truncation is exact.
fn round_ties_even_f64(x: f64) -> f64 {
    let f = floor_f64(x);
    let d = x - f;
    if d > 0.5 {
        f + 1.0
    } else if d < 0.5 {
        f
    } else if f % 2.0 == 0.0 {
        f
    } else {
        f + 1.0
    }
}

/// python: `round(x, 2)` -- the double nearest to the DECIMAL value of `x`
/// rounded to 2 places, half-even. python rounds the exact value, not a scaled
/// copy of it, so this splits `x * 100.0` into its exact `(p, e)` pair and uses
/// the residual `e` to break the `p == k + 0.5` cases (python's
/// `round(2.675, 2) == 2.67`, `round(0.005, 2) == 0.01`, `round(0.025, 2) ==
/// 0.03`; a plain `(x*100).round_ties_even()/100` gets all three wrong).
/// `|x| >= 2^52` means `x` is already an integer, so rounding to 2 decimals is a
/// no-op and python returns `x` unchanged (this also covers inf/NaN and the
/// magnitudes where `x * 100.0` would overflow). no_std: only primitive f64
/// arithmetic plus the hand-rolled `floor_f64` / `round_ties_even_f64` above --
/// `f64::floor`, `f64::round_ties_even` and `f64::mul_add` all fail to compile
/// in this `#![no_std]` crate, and correctness also needs plain SSE2 doubles
/// (x87 80-bit intermediates would break the Dekker split).
pub fn round2(x: f64) -> f64 {
    if !(x.abs() < 9_007_199_254_740_992.0) {
        return x; // |x| >= 2^52: integer -> no-op (also inf/NaN)
    }
    let (p, e) = two_prod(x, 100.0); // p + e == x * 100 exactly
    let f = floor_f64(p);
    let d = p - f;
    let out;
    if d == 0.0 {
        // p is integral: the exact scaled value is `p + e` with |e| <= ulp(p)/2.
        // Rounding `p` and dividing by 100 re-rounds and loses the low bits for
        // large |x|, so apply the (tiny, exact) correction to `x` instead:
        // n/100 == (p + r)/100 == x + (r - e)/100  with r the nearest integer to e.
        let r = if e.abs() == 0.5 {
            // exact tie at integer level: half-even on p
            if f % 2.0 == 0.0 {
                0.0
            } else {
                e.signum()
            }
        } else {
            round_ties_even_f64(e)
        };
        out = x + (r - e) / 100.0;
    } else {
        let n = if d > 0.5 {
            f + 1.0
        } else if d < 0.5 {
            f
        } else if e > 0.0 {
            f + 1.0 // exact product sits ABOVE the halfway point
        } else if e < 0.0 {
            f // ...just below it
        } else if f % 2.0 == 0.0 {
            f // exact tie: half-even
        } else {
            f + 1.0
        };
        out = n / 100.0;
    }
    if out == 0.0 {
        // python keeps the sign: round(-1e-101, 2) == -0.0
        return f64::from_bits(out.to_bits() | (x.to_bits() & (1u64 << 63)));
    }
    out
}

/// Immutable view returned by `list_tools` -- python builds a NEW dict per tool
/// with exactly these keys (no `registered_at`).
#[derive(Debug, Clone, PartialEq)]
pub struct ToolView {
    pub name: String,
    pub description: String,
    pub input_schema: SchemaSpec,
    pub output_schema: SchemaSpec,
    pub provider: String,
    pub version: String,
}

/// python `self._remote_servers[name] = {"url": ..., "connected_at": ...,
/// "tool_count": ...}` (connected_at supplied by the host clock; "" in no_std).
#[derive(Debug, Clone)]
pub struct RemoteServer {
    pub name: String,
    pub url: String,
    pub connected_at: String,
    pub tool_count: usize,
}

/// One entry of python `self._call_log`.
#[derive(Debug, Clone)]
pub struct CallLogEntry {
    pub tool: String,
    pub success: bool,
    pub elapsed_ms: f64,
    pub timestamp: String,
}

/// python: one element of `connect_remote(tools=[...])`. Building one mirrors
/// the oracle's `tool.get("description", "")` / `tool.get("input_schema", {})`
/// / `tool.get("output_schema", {})` defaults.
#[derive(Debug, Clone)]
pub struct RemoteTool {
    pub name: String,
    pub description: String,
    pub input_schema: SchemaSpec,
    pub output_schema: SchemaSpec,
}

/// Host-side result of running the registered handler; the two arms are the
/// oracle's `except Exception as e` and its successful return.
#[derive(Debug, Clone)]
pub enum HandlerOutcome {
    /// handler returned `value` after `elapsed_ms`.
    Ok { value: u64, elapsed_ms: f64 },
    /// handler raised; `message` is python's `str(e)`.
    Err { message: String, elapsed_ms: f64 },
}

/// python `get_stats()` return value.
#[derive(Debug, Clone)]
pub struct BusStats {
    pub total_tools: usize,
    pub local_tools: usize,
    pub remote_servers: usize,
    pub remote_tools: usize,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub recent_calls: Vec<CallLogEntry>,
}

/// python class `MCPBus`. The oracle's dicts are ordered Vecs here so the
/// insertion-order rules (list_tools order, remote-server order) survive.
#[derive(Debug, Clone)]
pub struct McpBus {
    /// python `self._tools: Dict[str, ToolSchema]`.
    pub tools: Vec<ToolSchema>,
    /// python `self._handlers: Dict[str, Callable]` -- opaque host token per name.
    pub handlers: Vec<(String, u64)>,
    /// python `self._remote_servers: Dict[str, Dict[str, Any]]`.
    pub remote_servers: Vec<RemoteServer>,
    /// python `self._call_log: List[Dict[str, Any]]`.
    pub call_log: Vec<CallLogEntry>,
    /// python `self._stats` flattened.
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
}

impl McpBus {
    /// python `MCPBus.__init__`.
    pub fn init() -> Self {
        McpBus {
            tools: Vec::new(),
            handlers: Vec::new(),
            remote_servers: Vec::new(),
            call_log: Vec::new(),
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
        }
    }

    fn tool_index(&self, name: &str) -> Option<usize> {
        self.tools.iter().position(|t| t.name == name)
    }
    fn handler_index(&self, name: &str) -> Option<usize> {
        self.handlers.iter().position(|(n, _)| n == name)
    }
    /// python: `name in self._tools`.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tool_index(name).is_some()
    }
    /// python: `name in self._handlers`.
    pub fn has_handler(&self, name: &str) -> bool {
        self.handler_index(name).is_some()
    }
    /// python: `len(self._tools)`.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// python `register_tool(...)`. `provider = None` is python's omitted
    /// argument (default "local"); `handler = None` means no handler was passed
    /// -- note the oracle then keeps any previously registered handler.
    pub fn register_tool(
        &mut self,
        name: &str,
        description: &str,
        input_schema: SchemaSpec,
        output_schema: SchemaSpec,
        handler: Option<u64>,
        provider: Option<&str>,
    ) -> ToolSchema {
        let schema = ToolSchema {
            name: String::from(name),
            description: String::from(description),
            input_schema,
            output_schema,
            provider: String::from(provider.unwrap_or(DEFAULT_PROVIDER)),
            version: String::from(DEFAULT_VERSION),
            // python: datetime.now().isoformat() -- host-side, caller's job.
            registered_at: String::new(),
        };
        // python dict assignment: replace in place, else append.
        match self.tool_index(name) {
            Some(i) => self.tools[i] = schema.clone(),
            None => self.tools.push(schema.clone()),
        }
        if let Some(h) = handler {
            match self.handler_index(name) {
                Some(i) => self.handlers[i].1 = h,
                None => self.handlers.push((String::from(name), h)),
            }
        }
        schema
    }

    /// python `unregister_tool`.
    pub fn unregister_tool(&mut self, name: &str) -> bool {
        match self.tool_index(name) {
            Some(i) => {
                self.tools.remove(i);
                if let Some(j) = self.handler_index(name) {
                    self.handlers.remove(j);
                }
                true
            }
            None => false,
        }
    }

    /// python `list_tools(provider=None)`.
    pub fn list_tools(&self, provider: Option<&str>) -> Vec<ToolView> {
        // python: `if provider and schema.provider != provider: continue`
        let filter: Option<&str> = match provider {
            Some(p) if !p.is_empty() => Some(p),
            _ => None,
        };
        let mut out = Vec::new();
        for s in self.tools.iter() {
            if let Some(p) = filter {
                if s.provider != p {
                    continue;
                }
            }
            out.push(ToolView {
                name: s.name.clone(),
                description: s.description.clone(),
                input_schema: s.input_schema.clone(),
                output_schema: s.output_schema.clone(),
                provider: s.provider.clone(),
                version: s.version.clone(),
            });
        }
        out
    }

    /// python `_validate_input` -- `None` = valid. `args` is the set of argument
    /// KEYS (python tests `field_name not in args`, i.e. key presence, so a
    /// required key mapped to null still counts as present).
    pub fn validate_input(&self, tool_name: &str, args: &[String]) -> Option<String> {
        let schema = match self.tool_index(tool_name) {
            Some(i) => &self.tools[i],
            None => return None, // python: `if not schema ... : return None`
        };
        if schema.input_schema.is_empty() {
            return None; // python: `or not schema.input_schema`
        }
        for field_name in schema.input_schema.required.iter() {
            if !args.iter().any(|a| a == field_name) {
                return Some(format!("Missing required field: {}", field_name));
            }
        }
        None
    }

    /// python `call_tool(name, args=None)` with the handler outcome injected by
    /// the host. `args` is the argument-key list (`args or {}` -> empty slice).
    pub fn call_tool(
        &mut self,
        name: &str,
        args: &[String],
        outcome: HandlerOutcome,
    ) -> ToolCallResult {
        self.total_calls += 1;

        if !self.has_tool(name) {
            self.failed_calls += 1;
            return ToolCallResult {
                tool_name: String::from(name),
                success: false,
                result: 0,
                error: format!("Tool '{}' not found", name),
                elapsed_ms: 0.0,
            };
        }

        if let Some(err) = self.validate_input(name, args) {
            self.failed_calls += 1;
            return ToolCallResult {
                tool_name: String::from(name),
                success: false,
                result: 0,
                error: err,
                elapsed_ms: 0.0,
            };
        }

        if !self.has_handler(name) {
            self.failed_calls += 1;
            return ToolCallResult {
                tool_name: String::from(name),
                success: false,
                result: 0,
                error: format!("No handler registered for '{}'", name),
                elapsed_ms: 0.0,
            };
        }

        let r = match outcome {
            HandlerOutcome::Ok { value, elapsed_ms } => {
                self.successful_calls += 1;
                ToolCallResult {
                    tool_name: String::from(name),
                    success: true,
                    result: value,
                    error: String::new(),
                    elapsed_ms,
                }
            }
            HandlerOutcome::Err { message, elapsed_ms } => {
                self.failed_calls += 1;
                ToolCallResult {
                    tool_name: String::from(name),
                    success: false,
                    result: 0,
                    error: message,
                    elapsed_ms,
                }
            }
        };

        // python: only executed calls reach the log; the three early returns above
        // return before the logging block.
        self.call_log.push(CallLogEntry {
            tool: String::from(name),
            success: r.success,
            elapsed_ms: round2(r.elapsed_ms),
            timestamp: String::new(), // python: datetime.now().isoformat()
        });
        if self.call_log.len() > CALL_LOG_MAX {
            let start = self.call_log.len() - CALL_LOG_KEEP;
            self.call_log.drain(0..start); // python: self._call_log[-100:]
        }

        r
    }

    /// python `connect_remote(server_name, server_url, tools=None)`. An empty
    /// `tools` slice is python's `tools = []` / `None` (the `if tools:` guard).
    pub fn connect_remote(
        &mut self,
        server_name: &str,
        server_url: &str,
        tools: &[RemoteTool],
    ) -> usize {
        let entry = RemoteServer {
            name: String::from(server_name),
            url: String::from(server_url),
            connected_at: String::new(), // python: datetime.now().isoformat()
            tool_count: 0,
        };
        match self.remote_servers.iter().position(|s| s.name == server_name) {
            Some(i) => self.remote_servers[i] = entry,
            None => self.remote_servers.push(entry),
        }

        let mut imported: usize = 0;
        for t in tools.iter() {
            let full = format!("{}/{}", server_name, t.name);
            self.register_tool(
                &full,
                &t.description,
                t.input_schema.clone(),
                t.output_schema.clone(),
                None,
                Some(server_name),
            );
            imported += 1;
        }

        if let Some(i) = self.remote_servers.iter().position(|s| s.name == server_name) {
            self.remote_servers[i].tool_count = imported;
        }
        imported
    }

    /// python `get_stats()`.
    pub fn get_stats(&self) -> BusStats {
        let start = self.call_log.len().saturating_sub(RECENT_CALLS);
        BusStats {
            total_tools: self.tools.len(),
            local_tools: self
                .tools
                .iter()
                .filter(|t| t.provider == DEFAULT_PROVIDER)
                .count(),
            remote_servers: self.remote_servers.len(),
            remote_tools: self
                .tools
                .iter()
                .filter(|t| t.provider != DEFAULT_PROVIDER)
                .count(),
            total_calls: self.total_calls,
            successful_calls: self.successful_calls,
            failed_calls: self.failed_calls,
            recent_calls: self.call_log[start..].to_vec(),
        }
    }
}
