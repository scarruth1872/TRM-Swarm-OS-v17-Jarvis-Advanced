# Bridge Secret Rotation

The HMAC secret that authenticates UDP 8022 bridge traffic is **not compiled
into any source file**. It has one source of truth, resolved identically by
every service:

1. `TRM_BRIDGE_SECRET` environment variable, if set and non-empty.
2. `<repo>/data/bridge_secret` — created on first run with 32 random bytes
   (hex-encoded, 64 characters) and mode `0600`.

`data/bridge_secret` is git-ignored. Never commit it, never paste its value
into a document, log, ticket, or chat.

## Where the secret is read

| Service | Port | File |
|---|---|---|
| Jarvis | 3000 | `Jarvis-Advanced-main/server.ts` (`loadBridgeSecret`) |
| Companion bridge | 3002 | `trm_uefi_kernel/trm_companion_bridge` (no secret use; rebuilt for parity) |
| Council server | 8022 (TCP) | `trm_uefi_kernel/host_council_server` |
| Resident UDP bridge | 8022 (UDP) | `swarm_v2/core/comm_protocol.py` (`_load_bridge_secret`) |
| Kernel library | — | `trm_uefi_kernel/src/comm_protocol.rs` (bare-metal: carries no secret) |
| Host tranche harness | — | `trm_uefi_kernel/host_tranche/src/comm_protocol.rs` (`secret_key()`) |
| Snapshot copy | — | `SwarmOS_v6_Snapshot_extracted/swarm_v2/core/comm_protocol.py` |

The bare-metal kernel image deliberately carries an empty constant: it has no
filesystem, so the boot environment injects the value at runtime. A leaked
image therefore cannot authenticate against the bridge.

## Rotate

1. **Generate a new secret** (32 random bytes, hex-encoded):

   ```bash
   python -c "import secrets; print(secrets.token_hex(32))" > data/bridge_secret
   chmod 600 data/bridge_secret
   ```

   Or set `TRM_BRIDGE_SECRET` in the service environment instead of using the
   file. If you use the env var, set it for **all** services; the file is only
   the fallback.

   On Windows, `chmod 600` is a no-op on NTFS. Restrict the ACL to the owner:

   ```bash
   icacls "data\\bridge_secret" /inheritance:r /grant:r "$USERNAME:F"
   ```

2. **Restart every service that reads it**, in this order:

   ```bash
   # 3002 companion bridge
   # 3000 Jarvis (tsx server.ts)
   # 8022 council server (TCP) + resident UDP bridge
   ```

   `taskkill` is unreliable on this host. Terminate with psutil:

   ```python
   import psutil
   psutil.Process(pid).terminate()
   ```

   Start each detached (no console), then poll `/health` until it returns JSON.

3. **Verify**:

   ```bash
   # (a) new secret authenticates
   python - <<'PY'
   import socket, json, hmac, hashlib, time, pathlib
   s = pathlib.Path("data/bridge_secret").read_text().strip()
   w = int(time.time() / 300)
   t = hmac.new(s.encode(), f"kernel->resident:bridge-ping:{w}".encode(),
                hashlib.sha256).hexdigest()
   sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM); sock.settimeout(5)
   sock.sendto(json.dumps({"op": "ping", "token": t}).encode(), ("127.0.0.1", 8022))
   print(sock.recvfrom(65535)[0].decode())
   PY
   # expect {"ok": true, "pong": ...}

   # (b) the retired literal is rejected
   # send the same frame signed with the old value; expect
   # {"ok": false, "error": "unauthorized"}

   # (c) no literal remains in the tree — search for the retired value
   #     (kept out of this document on purpose)
   rg -n "<retired-literal>" -g '!node_modules' -g '!.git' -g '!target' .
   # expect no matches
   ```

4. **Confirm all three services are healthy**:

   ```bash
   curl -s http://127.0.0.1:3002/health
   curl -s http://127.0.0.1:3000/health
   curl -s http://127.0.0.1:8022/health
   ```

## Failure modes

- **Only one side restarted** → every bridge call returns `unauthorized`.
  Restart all readers; there is no partial-rotation mode.
- **`data/bridge_secret` missing** → the first service to start creates it.
  If two services start simultaneously on a fresh checkout, they may race;
  create the file once before starting anything.
- **Env var set for one service only** → that service uses the env value while
  the others use the file. Set it everywhere or nowhere.
- **UDP 8022 already bound** → the resident bridge uses a plain `bind()` with
  no `SO_REUSEADDR`. Kill the old process and confirm the port is free before
  relaunching (`python bridge_status.py`).
