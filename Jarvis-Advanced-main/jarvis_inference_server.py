"""
Jarvis LoRA Inference Server — serves the fine-tuned Qwen3-4B adapter.
Runs on port 5115, loads once, keeps model warm.
"""
import sys, json, torch, uvicorn
from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse

# Try DirectML, fall back to CPU
try:
    import torch_directml
    DEVICE = torch_directml.device()
    print(f"Using DirectML: {DEVICE}")
except:
    DEVICE = torch.device("cpu")
    print("DirectML unavailable, using CPU")
ADAPTER = r"F:\Development sites\Jarvis-Advanced-main\Jarvis-Advanced-main\jarvis_qwen3_adapter"

app = FastAPI(title="Jarvis Inference Server")
model, tok = None, None

@app.on_event("startup")
def load():
    global model, tok
    print("Loading Qwen3-4B + Jarvis LoRA (loss 0.0148)...")
    from transformers import AutoModelForCausalLM, AutoTokenizer
    from peft import PeftModel
    base = AutoModelForCausalLM.from_pretrained("Qwen/Qwen3-4B",
        trust_remote_code=True, torch_dtype=torch.float16, low_cpu_mem_usage=True).to(DEVICE)
    model = PeftModel.from_pretrained(base, ADAPTER)
    tok = AutoTokenizer.from_pretrained(ADAPTER)
    print(f"Model loaded. Device: {DEVICE}")

@app.post("/chat")
async def chat(req: Request):
    body = await req.json()
    message = body.get("message", "")
    system = body.get("system", "You are J.A.R.V.I.S. OS, created by Shawn Carruth (Boss). Law of One.")
    max_tokens = body.get("max_tokens", 200)
    temperature = body.get("temperature", 0.7)

    msgs = [
        {"role": "system", "content": system},
        {"role": "user", "content": message},
    ]
    text = tok.apply_chat_template(msgs, tokenize=False, add_generation_prompt=True)
    inputs = tok(text, return_tensors="pt").to(DEVICE)

    with torch.no_grad():
        out = model.generate(
            **inputs,
            max_new_tokens=max_tokens,
            temperature=temperature,
            do_sample=True,
            pad_token_id=tok.eos_token_id,
        )

    reply = tok.decode(out[0], skip_special_tokens=True)
    if "assistant" in reply:
        reply = reply.split("assistant")[-1].strip()
    reply = reply.replace("<think>", "").replace("</think>", "").strip()

    return JSONResponse({"reply": reply, "model": "jarvis_lora", "loss": "0.0148"})

@app.get("/health")
async def health():
    return {"status": "ok", "model_loaded": model is not None}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=5115, log_level="info")
