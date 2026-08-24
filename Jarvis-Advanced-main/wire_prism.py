"""Wire PRISM-FQM panel to live telemetry data."""
import re

filepath = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\src\components\ResonanceResearchHub.tsx"

with open(filepath, 'r', errors='ignore') as f:
    content = f.read()

changes = 0

# 1. Add prismTelemetry state + fetch effect
old_state = "const [syncState, setSyncState] = useState<string | null>(null);"
new_state = """const [syncState, setSyncState] = useState<string | null>(null);
  const [prismTelemetry, setPrismTelemetry] = useState<any>(null);
  const [prismLoading, setPrismLoading] = useState(false);

  // Fetch live PRISM-FQM telemetry from TRM every 30s
  useEffect(() => {
    const fetchPrism = async () => {
      setPrismLoading(true);
      try {
        const res = await fetch("/api/trm/prism-fqm/telemetry");
        if (res.ok) {
          const data = await res.json();
          setPrismTelemetry(data);
        }
      } catch (e) {
        console.warn("PRISM-FQM telemetry unavailable, using defaults");
      } finally {
        setPrismLoading(false);
      }
    };
    fetchPrism();
    const interval = setInterval(fetchPrism, 30000);
    return () => clearInterval(interval);
  }, []);"""

if old_state in content:
    content = content.replace(old_state, new_state, 1)
    changes += 1

# 2. Replace hardcoded values with live data
replacements = [
    ('Depth = 8 (L-System)', '{prismTelemetry ? `Depth = ${prismTelemetry.ftl.recursion_depth} (L-System)` : \'Depth = 8 (L-System)\'}'),
    ('D_H = 1.58 (Sierpinski)', '{prismTelemetry ? `D_H = ${prismTelemetry.ftl.load_dimension} (Sierpinski)` : \'D_H = 1.58 (Sierpinski)\'}'),
    ('w-[78%] animate-pulse', 'animate-pulse style={{width: prismTelemetry ? `${prismTelemetry.ftl.progress}%` : \'78%\'}}'),
    ('128 Qubits', '{prismTelemetry ? `${prismTelemetry.qsi.superposition_states} Qubits` : \'128 Qubits\'}'),
    ('32.0x Parallel', '{prismTelemetry ? `${prismTelemetry.qsi.grover_speedup}x Parallel` : \'32.0x Parallel\'}'),
    ('w-[92%] animate-pulse', 'animate-pulse style={{width: prismTelemetry ? `${prismTelemetry.qsi.progress}%` : \'92%\'}}'),
]

for old_val, new_val in replacements:
    if old_val in content:
        content = content.replace(old_val, new_val, 1)
        changes += 1

# 3. Update tunneling formula
old_tunnel = 'z(n+1) = z(n)^2 + c &nbsp;&nbsp;|&nbsp;&nbsp; Tunneling Prob: exp(-&beta;&bull;&Delta;E)'
new_tunnel = 'z(n+1) = z(n)\u00b2 + c &nbsp;&nbsp;|&nbsp;&nbsp; Tunneling Prob: {prismTelemetry ? prismTelemetry.mycorrhizal.formula : \'exp(-\u03b2\u2022\u0394E)\'} | P={prismTelemetry ? prismTelemetry.mycorrhizal.tunneling_probability : \'0.85\'}'
if old_tunnel in content:
    content = content.replace(old_tunnel, new_tunnel, 1)
    changes += 1

# 4. Update sync button to also call trigger endpoint
old_btn = 'speakText("Anchoring PRISM FQM theoretical paradigm to memory graph.");\n                      handleRegisterAsMemoryNode'
new_btn = 'speakText("Anchoring PRISM FQM theoretical paradigm to memory graph.");\n                      try { await fetch("/api/trm/prism-fqm/trigger", { method: "POST" }); } catch(e) {}\n                      handleRegisterAsMemoryNode'
if old_btn in content:
    content = content.replace(old_btn, new_btn, 1)
    changes += 1

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(content)

print(f"PRISM-FQM panel wired! {changes} replacements made.")
