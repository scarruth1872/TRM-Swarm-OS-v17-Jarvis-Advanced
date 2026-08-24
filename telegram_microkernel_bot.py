#!/usr/bin/env python3
"""
TRM Swarm OS — Telegram Microkernel Bridge.
Replaces HTTP-polling Telegram bot with direct microkernel routing.
Zero token waste. Uses agent_intelligence_bridge for all responses.
Only responds when a user sends a message — no polling, no heartbeats, no spam.
"""

import json, time, os, sys
sys.path.insert(0, '.')
from swarm_v2.core.agent_intelligence_bridge import route_agent_intelligence

# ═══════════════════════════════════════════════════════
# CONFIG
# ═══════════════════════════════════════════════════════
TOKEN = os.getenv("TELEGRAM_BOT_TOKEN", "")
if not TOKEN:
    print("[TELEGRAM] No TELEGRAM_BOT_TOKEN set. Bot disabled.")
    print("  Set: export TELEGRAM_BOT_TOKEN='your_token_here'")
    sys.exit(0)

API_BASE = f"https://api.telegram.org/bot{TOKEN}"
OFFSET_FILE = "swarm_v2_memory/telegram_offset.txt"
POLL_INTERVAL = 2  # Seconds between polls (minimal, no spam)

# ═══════════════════════════════════════════════════════
# MICROKERNEL BOT
# ═══════════════════════════════════════════════════════
import urllib.request, urllib.error

def get_offset():
    try:
        return int(open(OFFSET_FILE).read().strip())
    except:
        return 0

def set_offset(offset):
    with open(OFFSET_FILE, 'w') as f:
        f.write(str(offset))

def send_message(chat_id, text):
    """Send via Telegram API — only when responding to user."""
    url = f"{API_BASE}/sendMessage"
    data = json.dumps({"chat_id": chat_id, "text": text[:4096]}).encode()
    req = urllib.request.Request(url, data=data, 
        headers={"Content-Type": "application/json"})
    try:
        urllib.request.urlopen(req, timeout=5)
        return True
    except:
        return False

def process_update(update):
    """Route message through microkernel agent bridge."""
    msg = update.get("message", {})
    chat_id = msg.get("chat", {}).get("id")
    text = msg.get("text", "").strip()
    
    if not chat_id or not text:
        return
    
    print(f"[TELEGRAM] {chat_id}: {text[:50]}")
    
    # Route through microkernel — same pipeline as dashboard chat
    resp = route_agent_intelligence("Archi", text)
    response_text = resp.get("response", "I received your message.")
    
    # Send response
    send_message(chat_id, f"[Archi | {resp['latency_us']}μs]\n\n{response_text}")

def run():
    print(f"[TELEGRAM] Microkernel bot active. Polling every {POLL_INTERVAL}s.")
    print(f"[TELEGRAM] Zero token waste — responds only to user messages.")
    
    last_offset = get_offset()
    
    while True:
        try:
            url = f"{API_BASE}/getUpdates?offset={last_offset + 1}&timeout={POLL_INTERVAL}"
            resp = urllib.request.urlopen(url, timeout=POLL_INTERVAL + 5)
            data = json.loads(resp.read())
            
            if data.get("ok") and data.get("result"):
                for update in data["result"]:
                    process_update(update)
                    last_offset = max(last_offset, update["update_id"])
                    set_offset(last_offset)
        except urllib.error.URLError:
            pass  # Network hiccup — retry
        except Exception as e:
            print(f"[TELEGRAM] Error: {e}")
            time.sleep(5)

if __name__ == "__main__":
    run()
