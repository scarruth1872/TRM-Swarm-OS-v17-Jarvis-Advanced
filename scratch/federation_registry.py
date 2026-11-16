"""Make v20's federation registry MEASURED, and stop unprobed counts reading as zero.

v20 serves /federation/stats and /sync from <repo>/swarm_v2_federation/registry.json and counts a
peer only when its status is "online" or "SYNCED". That file does not exist on this host, so both
endpoints reported 0/0 — indistinguishable from "no peers" when the truth is "never looked".

This does two things:
  1. writes the registry from the peer config, with each status decided by a real TCP connect to
     :8022 (never asserted), plus probed_at and a count of what was actually probed;
  2. patches the two payload builders so an ABSENT registry reports null counts with probed:false
     instead of 0.
"""
import json
import socket
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path('.')
CFG = ROOT / 'trm_uefi_kernel' / 'host_council_server' / 'config' / 'federation_peers.json'
REG = ROOT / 'swarm_v2_federation' / 'registry.json'
SH = ROOT / 'staging' / 'v20' / 'trm_uefi_kernel' / 'host_council_server' / 'src' / 'state_handlers.rs'


def probe(ip, port=8022, timeout=1.5):
    t0 = time.time()
    try:
        with socket.create_connection((ip, port), timeout=timeout):
            return True, round((time.time() - t0) * 1000.0, 1)
    except OSError:
        return False, None


def build_registry():
    cfg = json.loads(CFG.read_text(encoding='utf-8'))
    nodes = []
    for p in cfg.get('peers', []):
        ok, latency = probe(p['ip'])
        nodes.append({
            'node_id': p['id'],
            'node_name': p['name'],
            'tier': p.get('tier'),
            'ip': p['ip'],
            'port': 8022,
            'status': 'online' if ok else 'offline',
            'reachable': ok,
            'latency_ms': latency,
            'probed_at': datetime.now(timezone.utc).isoformat(),
        })
    return {
        'registry_version': 'v4',
        'tailnet': 'tailscale',
        'monad_anchor': '0x4D4F4E4144000001',
        'probe_method': 'TCP connect to :8022, 1.5s timeout, per peer',
        'generated_at': datetime.now(timezone.utc).isoformat(),
        'connected_nodes': nodes,
        'total_nodes': len(nodes),
        'online': sum(1 for n in nodes if n['reachable']),
    }


def patch_null_guard():
    s = SH.read_text(encoding='utf-8')
    if 'probed_registry' in s:
        print('state_handlers already guarded')
        return

    old_reg = '''fn read_registry() -> Value {
    let reg_path = repo_root().join("swarm_v2_federation").join("registry.json");'''
    new_reg = '''/// True only when the registry file existed and carried a peer list. An absent file is
/// "never probed", which must not be rendered as a count of zero.
fn probed_registry(reg: &Value) -> bool {
    reg.get("connected_nodes").map(|c| c.is_array()).unwrap_or(false)
}

fn read_registry() -> Value {
    let reg_path = repo_root().join("swarm_v2_federation").join("registry.json");'''
    if s.count(old_reg) != 1:
        raise SystemExit(f'read_registry anchor {s.count(old_reg)}')
    s = s.replace(old_reg, new_reg, 1)

    # federation_payload: null counts when unprobed
    old_a = '''        "connected_nodes": connected_count,'''
    new_a = '''        "connected_nodes": if probed_registry(&reg) { json!(connected_count) } else { Value::Null },
        "probed": probed_registry(&reg),'''
    if s.count(old_a) != 1:
        raise SystemExit(f'federation_payload anchor {s.count(old_a)}')
    s = s.replace(old_a, new_a, 1)

    # federation_sync_payload: same
    old_b = '''        "connected_nodes": connected,
        "total_nodes": nodes.len(),'''
    new_b = '''        "connected_nodes": if probed_registry(&reg) { json!(connected) } else { Value::Null },
        "total_nodes": if probed_registry(&reg) { json!(nodes.len()) } else { Value::Null },
        "probed": probed_registry(&reg),'''
    if s.count(old_b) != 1:
        raise SystemExit(f'sync_payload anchor {s.count(old_b)}')
    s = s.replace(old_b, new_b, 1)

    SH.write_text(s, encoding='utf-8', newline='')
    print('null guard patched into both payload builders')


if __name__ == '__main__':
    reg = build_registry()
    REG.parent.mkdir(parents=True, exist_ok=True)
    REG.write_text(json.dumps(reg, indent=2), encoding='utf-8', newline='')
    print(f'registry written: {reg["online"]}/{reg["total_nodes"]} online -> {REG}')
    for n in reg['connected_nodes']:
        print(f"  {n['node_id']} {n['ip']:16s} {n['status']:8s} {n['latency_ms']}")
    patch_null_guard()
