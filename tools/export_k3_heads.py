"""Export the 15 trained Bonsai-K3 heads into one portable binary for the Rust kernel.

Why: the retrained K3 heads exist only as PyTorch .pt files. The Rust kernel cannot
read pickle, so the trained trits are unreachable from the microkernel - the same
"trained but unloaded" gap that keeps 12 GB of _trits.bin dead. This writes a
single little-endian f32 blob with a documented layout, plus a manifest carrying
the SHA-256 so the loader can refuse a stale or truncated file.

LAYOUT (little-endian)
  magic       4 bytes   b"K3HD"
  version     u32       1
  n_agents    u32
  per agent:
    name_len  u16, name bytes (UTF-8)
    dim u32, state_dim u32, n_experts u32
    f32 tensors in this exact order:
      A   [state_dim*state_dim]
      B   [state_dim*dim]
      C   [dim*state_dim]
      D   [dim*dim]
      for e in 0..n_experts: w1 [64*dim], b1 [64], w2 [dim*64], b2 [dim]
      gate_w [n_experts*dim], gate_b [n_experts]
      cls_w1 [64*dim], cls_b1 [64], cls_w2 [3*64], cls_b2 [3]
      out_w [dim*dim], out_b [dim]
      norm_w [dim], norm_b [dim]

FEATURISER (must be reproduced byte-for-byte in Rust)
  lower ascii; tokens = runs of [a-z0-9']; grams = unigrams then "a_b" bigrams;
  for each gram h = crc32_iso_hdlc(utf8 gram); v[h % 96] += +1 if (h>>16)&1 else -1;
  then v /= ||v||2 (leave zero vector as zero).

WHY A NOTE ON A: h starts at zero, so dh = h@A.T + x@B.T collapses to x@B.T and A
never influences the output. It is exported anyway to keep the blob a faithful
copy of the checkpoint (and so a future recurrent version can use it).
"""
import hashlib
import json
import struct
import sys
from pathlib import Path

import os

import numpy as np
import torch

ROOT = Path(__file__).resolve().parent.parent
# Env overrides so an experiment can be exported to a scratch blob without
# disturbing the live asset the council binary embeds.
CKPT = Path(os.environ.get("TRM_K3_CKPT_DIR") or (ROOT / "swarm_v2_global_memory" / "trm_7m_agents"))
OUT_BIN = Path(os.environ.get("TRM_K3_BLOB") or (ROOT / "trm_uefi_kernel" / "assets" / "k3_heads.bin"))
OUT_MAN = Path(str(OUT_BIN) + ".json") if os.environ.get("TRM_K3_BLOB") else (ROOT / "trm_uefi_kernel" / "assets" / "k3_heads.json")

AGENTS = ['Shield', 'Logic', 'Devo', 'Seeker', 'Verify', 'Archi', 'Flow', 'Bridge',
          'Orchestra', 'EventLedger', 'LogicMaster', 'Pulse', 'THzGateway', 'Jarvis', 'Hermes']
HIDDEN = 64


def f32(t):
    return t.detach().to(torch.float32).contiguous().numpy().tobytes()


def quantise_state(ms, factor, block_factor=0.0):
    """TWN ternary quantisation of the same tensors QATLinear wraps.

        alpha = factor * mean(|w|);  q = sign(w) if |w| > alpha else 0;  w' = alpha*q

    Only the matrices that QATLinear replaced are quantised: the expert, gate,
    classifier and out_proj weights. The K3 block matrices A/B/C/D are raw
    Parameters, not nn.Linear, so the trained model keeps them in f32 - quantising
    them here produced 4 wrong trits on Archi (296/300 parity) because the export
    no longer matched the model that was trained. They stay f32 until the block
    itself is trained ternary.
    """
    # emb.weight is skipped for the same reason as A/B/C/D: the embedding table is
    # not an nn.Linear, so QATLinear never wrapped it and the trained model keeps
    # it in f32. Quantising it would change what the shipped model computes.
    skip = {'emb.weight'}
    if block_factor <= 0:
        # Skipped ONLY when the block was not trained ternary: quantising weights the
        # trainer kept in f32 ships a different model (296/300 parity when tried).
        # With --qat-block the trainer quantises A/B/C/D in the forward pass, so the
        # export must match it.
        skip |= {'A', 'B', 'C', 'D'}
    out = {}
    nz = tot = 0
    for k, v in ms.items():
        a = v.detach().cpu().numpy()
        if a.ndim == 2 and k not in skip:
            alpha = factor * float(np.abs(a).mean())
            q = np.sign(a)
            q[np.abs(a) <= alpha] = 0.0
            out[k] = torch.from_numpy((alpha * q).astype(np.float32))
            nz += int(np.count_nonzero(q))
            tot += a.size
        else:
            out[k] = v.detach().cpu()
    return out, nz, tot


def build_agent(name, ms, buckets=0):
    # dim comes from B (state_dim, dim); A is square (state_dim, state_dim) and
    # reported 16 for both when read from A, which silently truncated every
    # tensor in the blob. Caught by asserting against the expected param count.
    state = ms['A'].shape[0]
    dim = ms['B'].shape[1]
    assert ms['C'].shape == (dim, state), ms['C'].shape
    assert ms['D'].shape == (dim, dim), ms['D'].shape
    n_exp = sum(1 for k in ms if k.startswith('experts.') and k.endswith('.0.weight'))
    out = bytearray()
    nb = name.encode('utf-8')
    out += struct.pack('<H', len(nb)) + nb
    out += struct.pack('<IIII', dim, state, n_exp, buckets)
    if buckets:
        # EmbeddingBag table, row-major [buckets, dim]. The Rust loader averages
        # rows over the n-gram ids, which is what mode='mean' does in torch.
        emb = ms['emb.weight']
        assert emb.shape == (buckets, dim), (emb.shape, buckets, dim)
        out += f32(emb)
    for key in ('A', 'B', 'C', 'D'):
        out += f32(ms[key])
    for e in range(n_exp):
        out += f32(ms[f'experts.{e}.0.weight'])
        out += f32(ms[f'experts.{e}.0.bias'])
        out += f32(ms[f'experts.{e}.2.weight'])
        out += f32(ms[f'experts.{e}.2.bias'])
    for key in ('gate.0.weight', 'gate.0.bias',
                'classifier.0.weight', 'classifier.0.bias',
                'classifier.2.weight', 'classifier.2.bias',
                'out_proj.weight', 'out_proj.bias',
                'norm.weight', 'norm.bias'):
        out += f32(ms[key])
    return bytes(out), dim, state, n_exp


def main():
    agents = sys.argv[1:] or AGENTS
    # Version 2 adds the buckets field and the optional embedding table. Version 1
    # blobs remain readable by the kernel (buckets defaults to 0 = fixed BoW).
    blob = bytearray(b'K3HD' + struct.pack('<II', 2, len(agents)))
    meta = []
    for name in agents:
        path = CKPT / f'{name}_bonsai_instella.pt'
        st = torch.load(str(path), map_location='cpu', weights_only=False)
        ms = st['model_state']
        qf = float(st.get('qat_factor') or 0.0)
        bf = float(st.get('qat_block_factor') or 0.0)
        nz = tot = 0
        if qf > 0 or bf > 0:
            ms, nz, tot = quantise_state(ms, qf if qf > 0 else 0.7, bf)
        buckets = int(st.get('buckets') or 0)
        rec, dim, state, n_exp = build_agent(name, ms, buckets)
        n_params = int(sum(v.numel() for v in ms.values()))
        expected = 2 + len(name.encode('utf-8')) + 16 + n_params * 4
        assert len(rec) == expected, f'{name}: record {len(rec)} != expected {expected}'
        blob += rec
        meta.append({
            'agent': name, 'dim': dim, 'state_dim': state, 'n_experts': n_exp,
            'params': n_params,
            'buckets': buckets,
            'qat_factor': qf,
            'qat_block_factor': bf,
            'ternary_nonzero': nz,
            'ternary_weights': tot,
            'ternary_sparsity': round(1 - nz / tot, 4) if tot else None,
            'loss': float(st.get('loss', 0.0)),
            'label_counts': st.get('label_counts'),
            'record_bytes': len(rec),
        })
        print(f'  {name:12s} dim={dim} state={state} experts={n_exp} {len(rec):,} bytes')

    OUT_BIN.parent.mkdir(parents=True, exist_ok=True)
    OUT_BIN.write_bytes(blob)
    digest = hashlib.sha256(blob).hexdigest()
    manifest = {
        # Must match the header written above: a manifest claiming v1 for a v2 blob
        # is the kind of small lie that makes a stale asset look current.
        'magic': 'K3HD', 'version': 2,
        'n_agents': len(agents),
        'bytes': len(blob),
        'sha256': digest,
        'featuriser': 'hashed BoW, crc32_iso_hdlc, dim=96, unigrams+bigrams, sign=bit16, L2',
        'class_order': '0=BLOCK 1=ANALYZE 2=PROCEED',
        'agents': meta,
    }
    OUT_MAN.write_text(json.dumps(manifest, indent=1), encoding='utf-8')
    print(f'\nwrote {OUT_BIN} ({len(blob):,} bytes) sha256={digest[:16]}...')
    print(f'wrote {OUT_MAN}')


if __name__ == '__main__':
    main()
