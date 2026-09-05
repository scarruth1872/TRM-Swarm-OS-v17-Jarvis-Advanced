"""Council: fabric reorganization enacted via the platform mesh (bridge council op)."""
import sys, json, socket, time
sys.path.insert(0, r"F:\Development sites\TRM-Swarm-OS-v2")
from swarm_v2.core.comm_protocol import generate_trust_token

def bridge_call(op, extra=None):
    token = generate_trust_token("kernel", "resident", f"bridge-{op}")
    msg = {"op": op, "token": token}
    if extra: msg.update(extra)
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.settimeout(5)
    try:
        s.sendto(json.dumps(msg).encode(), ("127.0.0.1", 8022))
        data, _ = s.recvfrom(65535)
        return json.loads(data.decode())
    except Exception as e:
        return {"error": str(e)[:150]}
    finally:
        s.close()

motion = (
    "RATIFY capability-fabric reorganization: 23 model-roles across 7 "
    "capabilities (language 8, code 3, reasoning 4, vision 2, audio 1, "
    "retrieval 4, native-ternary 1) mapped to kernel routing paths. "
    "Ingested: Qwen3-4B, Instella, gemma-270m, CLIP measured. In flight: "
    "qwen3-1.7b, gpt2, tinyllama. Fabric = model dimension of totality "
    "graph; each shard Monad-sealed, on-device only."
)
r = bridge_call("council", {"motion": motion})
print("COUNCIL (via mesh):", json.dumps(r, indent=1)[:800])
