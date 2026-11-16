# Federation Sync Protocol — Deploy Bundle (2026-10-19)

MOTION-045 authorized deploying the standardized `/federation/stats` schema + `/sync` write endpoint
to the two remote Rust peers. This bundle carries the exact change.

## The change (2 files, surgical — do NOT overwrite with v20.0.0 files)

The remote kernels are **v19.001**; the local is **v20.0.0**. Only the federation functions changed,
so apply the *specific* edit below, not a whole-file copy.

### 1. `trm_uefi_kernel/host_council_server/src/state_handlers.rs`

Replace the old `federation_payload` (which returned hardcoded `mesh_agents=20` / `pbft_sync="100%"`
/ `voltage_harmony=0.8986` / `backend "v19.002"`) and the old `federation_sync_payload` (which
returned hardcoded `nodes_synced=4` / `AGREED_14_0_0`) with the versions in
[`state_handlers_federation.rs`](state_handlers_federation.rs) in this bundle.

The replacement is three functions: a `read_registry()` helper, a standardized `federation_payload`
(returns the registry's v4 shape), and `federation_sync_payload(body: &str)` (verifies the Monad
anchor, returns actual counts). Note the **signature change** on `federation_sync_payload` — it now
takes `body: &str`.

### 2. `trm_uefi_kernel/host_council_server/src/main.rs`

Replace the `/federation/sync` arm so it reads the body, and add a `/sync` arm:

```rust
    } else if method == Method::Post && url.starts_with("/federation/sync") {
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        response_json = state_handlers::federation_sync_payload(&body);
    } else if method == Method::Post && url == "/sync" {
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        response_json = state_handlers::federation_sync_payload(&body);
    } else if method == Method::Get && url.starts_with("/security/threats/query") {
```

## Apply on each remote node

**DESKTOP-MEP4VQN** (100.93.191.43, Windows) and **carruth-MacPro5-1** (100.76.102.6, Linux):

1. Copy this bundle to the node (scp / USB / any channel — see the SSH note below).
2. Apply the two edits above to the node's `state_handlers.rs` and `main.rs`.
3. Rebuild: `cargo build --release --bin host_council_server`.
4. Restart the kernel on :8022.
5. Verify: `curl -s http://127.0.0.1:8022/federation/stats` should return the v4 shape, and
   `curl -s -X POST http://127.0.0.1:8022/sync -H 'Content-Type: application/json' -d '{"monad_anchor":"0x4D4F4E4144000001"}'`
   should return `"status":"ok"` (wrong anchor → `"anchor_mismatch"`).

## Why this is on-site, not remote

As of 2026-10-19 the deploy cannot be executed from the coordinating host:

- **DESKTOP-MEP4VQN** (100.93.191.43): SSH answers but rejects key auth —
  `Permission denied (publickey,password,keyboard-interactive)`. No private key is installed on this
  host, so a password (not held here) would be required.
- **carruth-MacPro5-1** (100.76.102.6): SSH port 22 **refused** — no SSH server running.

So the change must be applied at each machine's console/desktop by its operator. The bundle is
ready; only the on-site copy + rebuild + restart remains.

## Verification receipt (local, already passing)

| endpoint | expected |
|---|---|
| `GET /federation/stats` | v4 shape: `{status, monad_anchor, registry_version, tailnet, connected_nodes, total_nodes, nodes}` |
| `POST /sync` correct anchor | `"status":"ok", "synced":true` |
| `POST /sync` wrong anchor | `"status":"anchor_mismatch", "synced":false` |
