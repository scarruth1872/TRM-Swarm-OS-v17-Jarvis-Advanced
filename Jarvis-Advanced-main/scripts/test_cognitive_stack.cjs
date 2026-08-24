const fs = require('fs');
const path = require('path');
const dotenv = require('dotenv');

dotenv.config();

async function runDiagnostics() {
  console.log("=============================================================");
  console.log("          J.A.R.V.I.S. COGNITIVE STACK DIAGNOSTIC TOOL       ");
  console.log("=============================================================\n");

  // 1. Cloud Gemini API Test
  console.log("1. Checking Cloud Intelligence (Gemini 3.5)...");
  const apiKey = process.env.GEMINI_API_KEY;
  if (!apiKey) {
    console.log("   [-] Result: GEMINI_API_KEY is missing in .env!\n");
  } else {
    try {
      const url = `https://generativelanguage.googleapis.com/v1beta/models/gemini-3.5-flash:generateContent?key=${apiKey}`;
      const start = Date.now();
      const res = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ contents: [{ parts: [{ text: "Hello" }] }] })
      });
      if (res.ok) {
        console.log(`   [+] Result: ONLINE (Latency: ${Date.now() - start}ms)\n`);
      } else {
        const errData = await res.json();
        console.log(`   [-] Result: ERROR - Status ${res.status}: ${JSON.stringify(errData.error?.message || errData)}\n`);
      }
    } catch (err) {
      console.log(`   [-] Result: FETCH EXCEPTION - ${err.message}\n`);
    }
  }

  // 2. Local Ollama & Gemma4 Test
  console.log("2. Checking Local Model Host (Ollama on port 11434)...");
  try {
    const start = Date.now();
    const res = await fetch("http://127.0.0.1:11434/api/tags");
    if (res.ok) {
      const tags = await res.json();
      const models = (tags.models || []).map(m => m.name);
      console.log(`   [+] Result: ONLINE (Latency: ${Date.now() - start}ms)`);
      console.log(`   [+] Installed Models: ${models.join(", ")}`);
      const hasGemma = models.some(m => m.includes("gemma4:12b"));
      console.log(`   [+] Gemma4 12B Status: ${hasGemma ? "READY" : "NOT FOUND (Install via 'ollama pull gemma4:12b')"} \n`);
    } else {
      console.log(`   [-] Result: FAILED - Status ${res.status}\n`);
    }
  } catch (err) {
    console.log(`   [-] Result: OFFLINE (Ollama daemon is stopped) - ${err.message}\n`);
  }

  // 3. Ternary-Bonsai Stack Test
  console.log("3. Checking Ternary-Bonsai Docker Containers...");
  const tiers = [
    { name: "Triage (1.7B)", port: 8581 },
    { name: "Execution (4B)", port: 8582 },
    { name: "Synthesis (8B)", port: 8583 }
  ];

  for (const tier of tiers) {
    try {
      const start = Date.now();
      const res = await fetch(`http://127.0.0.1:${tier.port}/health`);
      if (res.ok) {
        const health = await res.json();
        console.log(`   [+] ${tier.name} Port ${tier.port}: ${health.status || "OK"} (Latency: ${Date.now() - start}ms)`);
      } else {
        const health = await res.json().catch(() => ({}));
        console.log(`   [-] ${tier.name} Port ${tier.port}: ${health.error?.message || "LOADING/ERROR"} (Status ${res.status})`);
      }
    } catch (err) {
      console.log(`   [-] ${tier.name} Port ${tier.port}: OFFLINE (Container stopped)`);
    }
  }
  console.log("");

  // 4. Benchmark Triage Classifier Performance
  console.log("4. Benchmarking Ternary-Bonsai 1.7B Classifier...");
  try {
    const start = Date.now();
    const res = await fetch("http://127.0.0.1:8581/v1/chat/completions", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        model: "bitnet",
        messages: [
          { role: "system", content: "Choose exactly one label: ROUTE_SIMPLE, ROUTE_TOOL." },
          { role: "user", content: "Hello!" }
        ]
      })
    });
    if (res.ok) {
      const data = await res.json();
      const classification = data.choices?.[0]?.message?.content?.trim();
      console.log(`   [+] Classification Result: "${classification}"`);
      console.log(`   [+] Performance Speed: ${Date.now() - start}ms (Extremely fast 1-bit inference!)\n`);
    } else {
      console.log(`   [-] Classification Failed: Status ${res.status}\n`);
    }
  } catch (err) {
    console.log(`   [-] Benchmark Skipped: Triage container offline.\n`);
  }

  console.log("=============================================================");
}

runDiagnostics();
