"""Fix Jarvis proxy routes to use Node http module instead of global fetch."""
import re

server_path = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\server.ts"

with open(server_path, 'r', encoding='utf-8') as f:
    content = f.read()

# Find the proxy routes section
old_section = """// ─── TRM Proxy Routes (semantic graph, learning, robotics) ───
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
});"""

new_section = """// ─── TRM Proxy Routes (semantic graph, learning, robotics) ───
app.get("/api/trm/memory/semantic-graph", async (req, res) => {
  try {
    const http = require("http");
    const data = await new Promise((resolve, reject) => {
      http.get("http://127.0.0.1:8021/memory/semantic-graph", (resp) => {
        let raw = "";
        resp.on("data", (chunk) => raw += chunk);
        resp.on("end", () => resolve(JSON.parse(raw)));
      }).on("error", (e) => reject(e));
    });
    res.json(data);
  } catch (err: any) {
    res.status(500).json({ error: err.message, nodes: [], edges: [] });
  }
});

app.get("/api/trm/learning/recommendations", async (req, res) => {
  try {
    const http = require("http");
    const data = await new Promise((resolve, reject) => {
      http.get("http://127.0.0.1:8021/learning/recommendations", (resp) => {
        let raw = "";
        resp.on("data", (chunk) => raw += chunk);
        resp.on("end", () => resolve(JSON.parse(raw)));
      }).on("error", (e) => reject(e));
    });
    res.json(data);
  } catch (err: any) {
    res.status(500).json({ error: err.message, recommendations: [] });
  }
});"""

if old_section in content:
    content = content.replace(old_section, new_section)
    with open(server_path, 'w', encoding='utf-8') as f:
        f.write(content)
    print("Proxy routes fixed!")
else:
    print("Old section not found - checking what's in file...")
    if "TRM Proxy Routes" in content:
        start = content.find("TRM Proxy Routes")
        end = content.find("// API 8:", start)
        print(content[start:end])
    else:
        print("TRM Proxy Routes section not found at all")
