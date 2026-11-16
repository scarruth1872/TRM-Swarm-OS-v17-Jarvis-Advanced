"""Export the featuriser-v2 K3 heads to trm_uefi_kernel/assets/k3_heads_v2.bin.

Blob version 3 adds an explicit `feat_version` field per head:

  magic "K3HD" | version u32 = 3 | n_agents u32
  per agent:
    name_len u16, name
    dim u32, state_dim u32, n_experts u32, buckets u32, feat_version u32
    emb [buckets*dim] when buckets>0
    A, B, C, D, per-expert (w1,b1,w2,b2), gate, classifier, out_proj, norm

feat_version 0 = fixed hashed BoW (v1 blob with buckets=0)
feat_version 1 = char 3-5-gram EmbeddingBag mean over the padded width (v1)
feat_version 2 = char 3-5-gram + word-pair ids, mean over REAL grams, L2 (v2)

A v1 head carrying feat_version 2 (or the reverse) would silently score the
wrong features, so the loader refuses a head whose feat_version is unknown or
whose bucket table is missing.

Writes k3_heads_v2.bin and k3_heads_v2.json; k3_heads.bin is never touched.
"""
import hashlib
import json
import os
import struct
import sys
from pathlib import Path

import numpy as np
import torch

ROOT = Path(__file__).resolve().parent.parent
CKPT = Path(os.environ.get("TRM_K3_V2_CKPT") or (ROOT / "scratch" / "k3_v2_ckpt"))
OUT_BIN = Path(os.environ.get("TRM_K3_V2_BLOB") or (ROOT / "trm_uefi_kernel" / "assets" / "k3_heads_v2.bin"))
AGENTS = ['Shield', 'Logic', 'Devo', 'Seeker', 'Verify', 'Archi', 'Flow', 'Bridge',
          'Orchestra', 'EventLedger', 'LogicMaster', 'Pulse', 'THzGateway', 'Jarvis', 'Hermes']


def f32(t):
    return t.detach().to(torch.float32).contiguous().numpy().tobytes()


def quantise_state(ms, factor, block_factor=0.0):
    """Same TWN quantisation as tools/export_k3_heads.py, so v1 and v2 ship alike.

    emb.weight stays f32 (QATLinear never wrapped it) and A/B/C/D are quantised
    only when the block was trained ternary.
    """
    skip = {'emb.weight'}
    if block_factor <= 0:
        skip |= {'A', 'B', 'C', 'D'}
    out, nz, tot = {}, 0, 0
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


def build_agent(name, ms, buckets, feat_version):
    state = ms['A'].shape[0]
    dim = ms['B'].shape[1]
    assert ms['C'].shape == (dim, state)
    assert ms['D'].shape == (dim, dim)
    n_exp = sum(1 for k in ms if k.startswith('experts.') and k.endswith('.0.weight'))
    out = bytearray()
    nb = name.encode('utf-8')
    out += struct.pack('<H', len(nb)) + nb
    out += struct.pack('<IIIII', dim, state, n_exp, buckets, feat_version)
    if buckets:
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
    blob = bytearray(b'K3HD' + struct.pack('<II', 3, len(agents)))
    meta = []
    for name in agents:
        path = CKPT / f'{name}_bonsai_instella.pt'
        st = torch.load(str(path), map_location='cpu', weights_only=False)
        fv = int(st.get('feat_version') or 0)
        assert fv == 2, f'{name}: feat_version {fv} is not v2 - wrong checkpoint dir?'
        ms = st['model_state']
        qf = float(st.get('qat_factor') or 0.0)
        bf = float(st.get('qat_block_factor') or 0.0)
        nz = tot = 0
        if qf > 0 or bf > 0:
            ms, nz, tot = quantise_state(ms, qf if qf > 0 else 0.7, bf)
        buckets = int(st.get('buckets') or 0)
        rec, dim, state, n_exp = build_agent(name, ms, buckets, fv)
        n_params = int(sum(v.numel() for v in ms.values()))
        expected = 2 + len(name.encode('utf-8')) + 20 + n_params * 4
        assert len(rec) == expected, f'{name}: {len(rec)} != {expected}'
        blob += rec
        meta.append({'agent': name, 'dim': dim, 'state_dim': state, 'n_experts': n_exp,
                     'buckets': buckets, 'feat_version': fv, 'params': n_params,
                     'qat_factor': qf, 'qat_block_factor': bf,
                     'ternary_sparsity': round(1 - nz / tot, 4) if tot else None,
                     'loss': float(st.get('loss', 0.0)),
                     'label_counts': st.get('label_counts'),
                     'record_bytes': len(rec)})
        print(f'  {name:12s} fv={fv} dim={dim} state={state} experts={n_exp} '
              f'{len(rec):,} bytes')

    OUT_BIN.parent.mkdir(parents=True, exist_ok=True)
    OUT_BIN.write_bytes(blob)
    digest = hashlib.sha256(blob).hexdigest()
    manifest = {
        'magic': 'K3HD', 'version': 3, 'n_agents': len(agents),
        'bytes': len(blob), 'sha256': digest,
        'featuriser': ('feat_version 2: char 3/4/5-grams with a 64-slot-per-length budget and '
                       'head/tail split, + directed word pairs (d=1,2) in a 32-slot channel; '
                       'bucket 0 = padding; mean over real grams only, then L2'),
        'emb_cap': 224, 'char_cap': 192, 'pair_cap': 32,
        'class_order': '0=BLOCK 1=ANALYZE 2=PROCEED',
        'agents': meta,
    }
    OUT_BIN.with_suffix('.json').write_text(json.dumps(manifest, indent=1), encoding='utf-8')
    print(f'\nwrote {OUT_BIN} ({len(blob):,} bytes) sha256={digest}')
    print(f'wrote {OUT_BIN.with_suffix(".json")}')


if __name__ == '__main__':
    main()
