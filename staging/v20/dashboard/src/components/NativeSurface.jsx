import { useCallback, useEffect, useState } from "react";

// The native surface: what the Rust kernel actually serves on /native, measured -- the module census reads each
// module's own source at request time and the operation table is the kernel's own, fetched rather than typed
// here. Nothing on this page is a hardcoded list of routes; if the kernel stops serving one, the page says so.
const API_BASE = "http://127.0.0.1:8022";

const C = { bg: "#0a0e14", panel: "#111823", line: "#1f2b3a", cyan: "#00d4ff", green: "#00ff88",
            amber: "#ffb000", violet: "#7b61ff", dim: "#4a5568", text: "#c8d6e5" };

// A starting payload per operation family, so "run" works before the operator types anything.
const DEFAULTS = [
  ["/native/chaos/drop", { enabled: true, rate: 0.25, sample: 0.1 }],
  ["/native/chaos/clamp", { delay: 2.5, max: 1.0 }],
  ["/native/expert/resolve", { identifier: "Aura", team: [{ key: "a", name: "Aura", role: "UI" }] }],
  ["/native/persona/check", { output: "hello", forbidden: ["irreversible"], locked: false, secs_since_reset: 5 }],
  ["/native/persona/prompt", { persona: "Aura", axioms: ["stay honest"], forbidden: ["guess"] }],
  ["/native/gpu/pattern", { arch: "k3", size_mb: 96 }],
  ["/native/drug/triage", { candidates: ["CHEMBL123", "CHEMBL456"], tau: 0.35 }],
  ["/native/drug/sdf", { sdf: "C\n     RDKit          3D\n\n  1  0  0  0  0  0  0  0  0  0999 V2000" }],
  ["/native/docking/score-pair", { a: { x: 0, y: 0, z: 0, element: "C" }, b: { x: 3, y: 0, z: 0, element: "O" } }],
  ["/native/intel/route", { task: "triage a molecule", agents: [] }],
  ["/native/intel/vote", { signals: [1, 0.9, 0.7], tau: 0.35 }],
  ["/native/intel/decide", { task: "triage", candidates: [] }],
  ["/native/intel/ensemble", {}],
  ["/native/fit/encode", { entity: "booking", fields: ["userId=1", "serviceId=1"] }],
  ["/native/fit/verdict", { signals: [0.9, 0.8, 0.7], tau: 0.35 }],
  ["/native/fit/serve", { entity: "booking", record: "ui_probe", fields: ["userId=1"], signals: [0.9, 0.9] }],
  ["/native/brain/connectome", { n_neurons: 100000, hub_fraction: 0.01, seed: 42 }],
  ["/native/brain/neural", { n_neurons: 500, k_synapses: 8, seed: 42, t: 1,
                            field: [0.9, -0.8, 0.7, -0.6, 0.5, -0.4, 0.3, -0.2] }],
  ["/native/brain/collapse", { field: [0.8, -0.6, 0.2, -0.9, 0.5, 0.1, -0.4, 0.7], tau: 0.35 }],
  ["/native/brain/switch", { field: [0.8, -0.6, 0.2, -0.9, 0.5, 0.1, -0.4, 0.7], seed: 42 }],
];

function defaultPayload(path) {
  const hit = DEFAULTS.find(([p]) => p === path);
  return JSON.stringify(hit ? hit[1] : {}, null, 1);
}

const panel = {
  background: C.panel, border: `1px solid ${C.line}`, borderRadius: 10, padding: "12px 14px",
  marginBottom: 12,
};
const mono = { fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace", fontSize: 12 };
const th = { textAlign: "left", color: C.cyan, fontWeight: 600, padding: "6px 8px",
             borderBottom: `1px solid ${C.line}`, fontSize: 11, letterSpacing: 0.5, textTransform: "uppercase" };
const td = { padding: "6px 8px", borderBottom: `1px solid ${C.line}`, verticalAlign: "top",
             color: C.text, ...mono };

export default function NativeSurface() {
  const [modules, setModules] = useState(null);
  const [routes, setRoutes] = useState(null);
  const [crates, setCrates] = useState(null);
  const [selected, setSelected] = useState("/native/brain/switch");
  const [payload, setPayload] = useState(defaultPayload("/native/brain/switch"));
  const [result, setResult] = useState(null);
  const [error, setError] = useState(null);
  const [busy, setBusy] = useState(false);
  const [at, setAt] = useState(null);

  const load = useCallback(async () => {
    try {
      const [m, r, c] = await Promise.all([
        fetch(`${API_BASE}/native/modules`).then((x) => x.json()),
        fetch(`${API_BASE}/native/routes`).then((x) => x.json()),
        fetch(`${API_BASE}/crate/ops`).then((x) => x.json()),
      ]);
      setModules(m);
      setRoutes(r);
      setCrates(c);
      setError(null);
      setAt(new Date().toLocaleTimeString());
    } catch (e) {
      setError(`kernel unreachable on ${API_BASE}: ${e.message}`);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const op = (routes?.operations || []).find((o) => o.path === selected);
  const isGet = op?.method === "GET";

  const run = useCallback(async () => {
    setBusy(true);
    setResult(null);
    setError(null);
    try {
      let body;
      const init = { method: op?.method || "POST" };
      if (!isGet) {
        JSON.parse(payload); // refuse to send malformed JSON rather than let the kernel guess
        init.headers = { "Content-Type": "application/json" };
        init.body = payload;
      }
      const res = await fetch(`${API_BASE}${selected}`, init);
      body = await res.json();
      setResult({ status: res.status, body });
    } catch (e) {
      setError(e.message);
    } finally {
      setBusy(false);
    }
  }, [selected, payload, op, isGet]);

  return (
    <div style={{ color: C.text }}>
      <div style={{ ...panel, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <div style={{ color: C.cyan, letterSpacing: 1.5, fontSize: 12 }}>NATIVE SURFACE // RUST MICROKERNEL</div>
          <div style={{ fontSize: 11, color: C.dim, ...mono }}>
            {modules ? `${modules.count} modules` : "…"} · {routes ? `${routes.count} operations` : "…"}
            {crates ? ` · ${crates.wired ?? crates.ops?.length ?? 0} crates` : ""}
            {at ? ` · read ${at}` : ""}
          </div>
        </div>
        <button onClick={load} style={{ background: "transparent", color: C.green, border: `1px solid ${C.green}`,
          borderRadius: 6, padding: "6px 12px", cursor: "pointer", ...mono }}>
          RESCAN
        </button>
      </div>

      {error && (
        <div style={{ ...panel, borderColor: C.amber, color: C.amber, ...mono }}>{error}</div>
      )}

      {modules && modules.modules?.length > 0 && !("routes" in modules.modules[0]) && (
        <div style={{ ...panel, borderColor: C.amber, color: C.amber, ...mono }}>
          the census payload does not carry `routes` on its entries — this panel is reading keys the kernel no
          longer sends, so the columns below would be blank. Shown rather than hidden.
        </div>
      )}

      <div style={{ display: "grid", gridTemplateColumns: "1.15fr 1fr", gap: 12 }}>
        <div style={panel}>
          <div style={{ ...th, borderBottom: "none" }}>Modules measured at request time</div>
          <table style={{ width: "100%", borderCollapse: "collapse" }}>
            <thead><tr><th style={th}>module</th><th style={th}>route</th><th style={th}>source measured</th>
              <th style={th}>parity</th></tr></thead>
            <tbody>
              {(modules?.modules || []).map((m) => (
                <tr key={m.module}>
                  <td style={{ ...td, color: m.module.startsWith("swarm_") ? C.green : C.text }}>{m.module}</td>
                  <td style={{ ...td, color: C.cyan }}>{m.routes}</td>
                  <td style={td}>
                    {Number(m.source_bytes).toLocaleString()} B
                    <div style={{ color: C.dim }}>{m.source}</div>
                  </td>
                  <td style={td}>
                    {m.parity_checks_passed != null || m.parity_checks_failed != null
                      ? `${m.parity_checks_passed != null ? m.parity_checks_passed : "?"} pass / ` +
                        `${m.parity_checks_failed != null ? m.parity_checks_failed : "?"} fail`
                      : <span style={{ color: C.dim }}>—</span>}
                    <div style={{ color: C.dim }}>{(m.parity_artifact || "").split("/").pop()}</div>
                  </td>
                </tr>
              ))}
              {!modules && <tr><td style={td} colSpan={4}>loading…</td></tr>}
            </tbody>
          </table>
        </div>

        <div style={panel}>
          <div style={{ ...th, borderBottom: "none" }}>Operations (the kernel's own table)</div>
          <select value={selected} onChange={(e) => { setSelected(e.target.value);
              setPayload(defaultPayload(e.target.value)); setResult(null); }}
            style={{ width: "100%", background: C.bg, color: C.text, border: `1px solid ${C.line}`,
                     borderRadius: 6, padding: 8, ...mono }}>
            {(routes?.operations || []).map((o) => (
              <option key={`${o.method} ${o.path}`} value={o.path}>{`${o.method} ${o.path}`}</option>
            ))}
          </select>
          <div style={{ fontSize: 11, color: C.dim, margin: "8px 0", ...mono }}>
            {op ? op.basis : ""}
          </div>
          {!isGet && (
            <textarea value={payload} onChange={(e) => setPayload(e.target.value)} rows={7} spellCheck={false}
              style={{ width: "100%", background: C.bg, color: C.text, border: `1px solid ${C.line}`,
                       borderRadius: 6, padding: 8, ...mono }} />
          )}
          <button onClick={run} disabled={busy} style={{ marginTop: 8, background: busy ? C.dim : C.violet,
            color: "#08101a", border: "none", borderRadius: 6, padding: "8px 14px",
            cursor: busy ? "default" : "pointer", fontWeight: 700, ...mono }}>
            {busy ? "RUNNING…" : `RUN ${op?.method || ""} ${selected}`}
          </button>
          {result && (
            <div style={{ marginTop: 10 }}>
              <div style={{ color: result.status === 200 ? C.green : C.amber, ...mono }}>
                HTTP {result.status}
              </div>
              <pre style={{ background: C.bg, border: `1px solid ${C.line}`, borderRadius: 6, padding: 10,
                            maxHeight: 260, overflow: "auto", color: C.text, ...mono }}>
                {JSON.stringify(result.body, null, 1)}
              </pre>
            </div>
          )}
        </div>
      </div>

      <div style={panel}>
        <div style={{ ...th, borderBottom: "none" }}>Verified crates wired as routes (the /crate/ops census)</div>
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead><tr><th style={th}>route</th><th style={th}>engine</th><th style={th}>method</th><th style={th}>gate</th></tr></thead>
          <tbody>
            {(crates?.ops || []).map((o) => (
              <tr key={o.op}>
                <td style={{ ...td, color: C.cyan }}>{o.route}</td>
                <td style={{ ...td, color: o.engine === "swarm-drug" ? C.green : C.text }}>{o.engine}</td>
                <td style={td}>{o.method}</td>
                <td style={{ ...td, color: C.dim }}>{o.gate}</td>
              </tr>
            ))}
            {!crates && <tr><td style={td} colSpan={4}>loading…</td></tr>}
          </tbody>
        </table>
      </div>

      <div style={{ color: C.dim, fontSize: 11, ...mono }}>
        The module list and the operation table both come from the kernel at request time — this page owns no
        route list of its own, so a page that renders means the routes behind it answered.
      </div>
    </div>
  );
}
