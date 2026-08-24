"""Swap fallback order in Jarvis server.ts — local models first, Gemini as fallback."""
import re

path = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\server.ts"
with open(path, 'r', errors='ignore') as f:
    content = f.read()

# Find the block from Gemini try to the end of Ollama fallback
# We need to swap: Gemini is tried first, then LoRA, then Bonsai, then Ollama
# New order: LoRA -> Bonsai -> Ollama -> Gemini

# The Gemini call starts with:
# "// Primary: Gemini with routeHint injected into system prompt"
# Then the Gemini try block wraps generateContentWithFallback
# Then catch falls through to LoRA, Bonsai, Ollama
# After Ollama fallback, the response is used

# Let me restructure: wrap the gemini block into a try-catch that runs LAST
# Use a simpler approach: comment out the Gemini call and re-add it after Ollama

old_block = '''    // Primary: Gemini with routeHint injected into system prompt
  let response: any;
  try {
    response = await generateContentWithFallback(ai, {
      contents,
      config: { 
        systemInstruction: systemInstruction + (dnaOverrides.personalityPromptModifier ? "\\n" + dnaOverrides.personalityPromptModifier : ""), 
        tools: filteredTools, 
        temperature: dnaOverrides.temperature 
      },
    });
  } catch (geminiErr: any) {
    // Gemini failure — fall back to Jarvis LoRA inference server first (trained Qwen3-4B, 100% pass rate)
    console.warn(
      `[Jarvis Fallback] Gemini call failed (${geminiErr?.message || geminiErr}) — routing to Jarvis LoRA inference server...`,
    );
    // Try Jarvis LoRA (fine-tuned on system knowledge)
    try {
      const loraRes = await fetch("http://127.0.0.1:5115/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          message,
          system: `You are J.A.R.V.I.S. OS, a sovereign intelligence system created by Shawn Carruth (Boss). Law of One. Query type: ${routeHint}. Conversation history:\\n${history?.slice(-6).map((h: any) => `${h.role === "assistant" ? "J.A.R.V.I.S." : "Boss"}: ${h.content}`).join("\\n") || ""}`,
          max_tokens: 200,
          temperature: 0.7,
        }),
      });
      if (loraRes.ok) {
        const loraData: any = await loraRes.json();'''

# Check how many times this pattern occurs
count = content.count(old_block.strip()[:80])
print(f"Pattern occurrences: {count}")

if count >= 1:
    # Find the Gemini block
    start = content.find(old_block) 
    if start >= 0:
        # Find the end - look for where the Ollama fallback ends
        rest = content[start:]
        # The block ends after the Bonsai and Ollama fallbacks
        # Search for "return {"
        end_idx = rest.find("return {")
        if end_idx >= 0:
            end = start + end_idx
            # Find the start of return line
            return_start = rest.rfind("}", 0, end_idx) + 1
            final_end = start + return_start
            print(f"Block: {start} to {final_end}")
            print(f"Block length: {final_end - start}")
            
            # Extract the local model fallback part (LoRA + Bonsai + Ollama)
            block = content[start:final_end]
            # Print first and last 200 chars
            print(f"First: {block[:200]}")
            print(f"Last: {block[-200:]}")
else:
    print("Could not find the exact block - searching for alternatives")
    # Try a simpler search
    idx = content.find("let response: any;")
    if idx >= 0:
        print(f"Found 'let response: any;' at {idx}")
        print(content[idx:idx+50])
