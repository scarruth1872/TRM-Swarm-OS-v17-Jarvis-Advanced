"""Add Jarvis proxy routes for TRM endpoints."""
import re

server_path = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\server.ts"

with open(server_path, 'r', encoding='utf-8') as f:
    content = f.read()

# Insert new proxy routes after line 5319 (after force-reassign handler)
new_routes = """
// ─── TRM Proxy Routes (semantic graph, learning, robotics) ───
app.get("/api/trm/memory/semantic-graph", async (req, res) => {
  try {
    const response = await fetch("http://127.0.0.1:8021/memory/semantic-graph");
    if (!response.ok) throw new Error("TRM semantic graph error: " + response.status);
    const data = await response.json();
    res.json(data);
  } catch (err: any) {
    res.status(500).json({ error: err.message, nodes: [], edges: [] });
  }
});

app.get("/api/trm/learning/recommendations", async (req, res) => {
  try {
    const response = await fetch("http://127.0.0.1:8021/learning/recommendations");
    if (!response.ok) throw new Error("TRM learning error: " + response.status);
    const data = await response.json();
    res.json(data);
  } catch (err: any) {
    res.status(500).json({ error: err.message, recommendations: [] });
  }
});
"""

# Insert after line 5319 (before the skills registry section)
insert_pos = content.find("// API 8: Fetch custom skills registry")
if insert_pos > 0:
    content = content[:insert_pos] + new_routes + content[insert_pos:]
    with open(server_path, 'w', encoding='utf-8') as f:
        f.write(content)
    print(f"Routes added successfully at position {insert_pos}")
else:
    print("Could not find insertion point")
