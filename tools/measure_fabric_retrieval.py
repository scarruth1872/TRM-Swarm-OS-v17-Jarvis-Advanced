"""Measure fabric retrieval: the shipped substring query vs the semantic index.

The comparison has to be honest, so the probes are built to defeat the keyword matcher:

  1. Take real, non-boilerplate memories from the deduplicated view (150-900 chars,
     dup_count <= 3, not a model_failure, not raw JSON).
  2. Ask the LOCAL model (:8587) to rewrite each as a short question a user might ask,
     using different words.
  3. Measure the word overlap between probe and source. If a probe is just the source
     with two words moved, substring wins by construction and the comparison is
     meaningless - so probes above 0.45 Jaccard are reported and excluded.

A hit is a returned memory whose normalised content equals the source's normalised
content. That sidesteps id bookkeeping (a deduplicated row stands for up to 50 ids) and
asks the only question that matters: did the right memory come back?

Both methods get the same k and the same probes. Nothing is tuned after the fact.
"""
import gzip
import json
import random
import re
import sys
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

FABRIC = ROOT / 'swarm_v2_global_memory' / 'fabric'
LLM_URL = 'http://127.0.0.1:8591/v1/chat/completions'      # 27B deep tier
LLM_URL_FALLBACK = 'http://127.0.0.1:8587/v1/chat/completions'  # 1.5B, always up
N_CANDIDATES = 60
TARGET_KEPT = 20
KS = (1, 5, 10)
OVERLAP_LIMIT = 0.45


def norm(s):
    s = str(s)
    s = re.sub(r'0x[0-9a-fA-F]+', '<HEX>', s)
    s = re.sub(r'\b\d{3,}\b', '<N>', s)
    s = re.sub(r'sub_[a-z0-9_]+', '<SUB>', s)
    s = re.sub(r'\s+', ' ', s)
    return s.strip().lower()


def words(s):
    return set(re.findall(r"[a-z']{3,}", str(s).lower()))


def similar(a, b, threshold=0.7):
    """Substantial overlap rather than an identical row.

    The store holds many near-identical records, so demanding the SAME row can mark a
    correct answer wrong. Both scores are reported: strict (same normalised content) and
    lenient (word Jaccard >= threshold with the source).
    """
    wa, wb = words(a), words(b)
    if not wa or not wb:
        return False
    return len(wa & wb) / len(wa | wb) >= threshold


def load_distinct():
    rows = []
    with gzip.open(FABRIC / 'fabric_distinct.jsonl.gz', 'rt', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


# The first run of this harness produced 30 probes of which 25 were near-copies, and both
# methods scored 0.2 - an uninformative instrument, not a result. The cause was in the
# data: sampled memories are templates ("Swarm autonomous task", "consensus solution",
# "shared reasoning states") and the 1.5B paraphraser echoed that boilerplate back as the
# query. Template rows are excluded and the paraphraser is the 27B deep tier.
TEMPLATE_MARKERS = (
    'swarm autonomous task', 'subagent_id', 'task_spec', 'consensus solution',
    'review this', 'instella_domain', 'jarvis semantic knowledge graph',
    'jarvis episodic memory', 'jarvis core memory profile', 'tokens": [',
    'deterministic_routed',
)


def is_template(content):
    body = str(content).lower()
    return any(marker in body for marker in TEMPLATE_MARKERS)


def pick_probes(rows):
    cands = [r for r in rows
             if 150 <= len(str(r.get('content', ''))) <= 900
             and int(r.get('dup_count', 1)) <= 2
             and r.get('class') != 'model_failure'
             and not str(r.get('content', '')).lstrip().startswith('{')
             and not is_template(r.get('content'))]
    random.seed(11)
    random.shuffle(cands)
    return cands[:N_CANDIDATES]


def paraphrase(content):
    """Rewrite a memory as a user question using a local model.

    The deep tier (:8591) is a reasoning model: it needs room to finish thinking before it
    emits content, so a 64-token budget returns an empty string every time. 900 tokens
    with the 1.5B as fallback.
    """
    prompt = ('Rewrite the text below as ONE short question a user might ask to find it '
              'again later. Use different wording; do not copy phrases. '
              'Reply with the question only.\n\nTEXT:\n' + str(content)[:1200])
    # 1.5B first: the deep tier measured 62.7s per call on this GPU, which would make an
    # 80-probe run take over an hour. Templates are filtered out of the probe pool now,
    # so the small model no longer just echoes boilerplate back.
    for url, budget in ((LLM_URL_FALLBACK, 96), (LLM_URL, 900)):
        body = json.dumps({'messages': [{'role': 'user', 'content': prompt}],
                           'max_tokens': budget, 'temperature': 0.2}).encode()
        req = urllib.request.Request(url, data=body,
                                     headers={'Content-Type': 'application/json'})
        try:
            with urllib.request.urlopen(req, timeout=180) as r:
                d = json.loads(r.read())
            text = str(d['choices'][0]['message'].get('content') or '').strip()
            if text:
                return text.split('\n')[0][:200]
        except Exception:
            continue
    raise RuntimeError('no paraphraser answered (8591, 8587)')


def main():
    from swarm_v2.core.global_memory import GlobalMemorySync
    from swarm_v2.core.semantic_fabric import SemanticFabric

    fab = SemanticFabric()
    if not fab.ready:
        raise SystemExit(f'semantic index not ready: {fab.error}')
    print('semantic index:', json.dumps(fab.stats()), flush=True)

    rows = load_distinct()
    probes = pick_probes(rows)
    print(f'{len(probes)} probes drawn from {len(rows):,} distinct memories\n', flush=True)

    t = time.time()
    # Baseline is the platform's OWN query path (global_memory; on this host the
    # chroma-less keyword fallback over the live store). Measuring a baseline nobody
    # runs would flatter the new path.
    sub = GlobalMemorySync()
    baseline_rows = sub._turbovec.total_memories if getattr(sub, '_turbovec', None) else 0
    print(f'platform memory path up in {time.time() - t:.1f}s, '
          f'{baseline_rows:,} rows live, metadata mirror {len(sub.entries_metadata):,}\n',
          flush=True)

    results = []
    for i, src in enumerate(probes, 1):
        try:
            q = paraphrase(src['content'])
        except Exception as exc:
            print(f'  [{i}] paraphrase failed: {type(exc).__name__}: {exc}', flush=True)
            continue
        overlap = len(words(q) & words(src['content'])) / max(len(words(q)), 1)
        target = norm(src['content'])

        t0 = time.time()
        sub_scored = sub.query(q, top_k=max(KS))
        sub_hits = [entry for _score, entry in sub_scored]
        sub_ms = (time.time() - t0) * 1000
        t0 = time.time()
        sem_hits = fab.query(q, k=max(KS)) or []
        sem_ms = (time.time() - t0) * 1000

        row = {
            'probe': q,
            'source': str(src['content'])[:200],
            'overlap': round(overlap, 3),
            'kept': overlap < OVERLAP_LIMIT,
            'baseline_backend': (sub_scored[0][1].get('backend', 'keyword_fallback') if sub_scored else None),
            'baseline_ms': round(sub_ms, 1),
            'semantic_ms': round(sem_ms, 1),
            'substring': [k for k in KS if any(norm(h.get('content')) == target for h in sub_hits[:k])],
            'semantic': [k for k in KS if any(norm(h.get('content')) == target for h in sem_hits[:k])],
            'substring_lenient': [k for k in KS
                                  if any(similar(h.get('content'), src['content']) for h in sub_hits[:k])],
            'semantic_lenient': [k for k in KS
                                 if any(similar(h.get('content'), src['content']) for h in sem_hits[:k])],
        }
        results.append(row)
        if sum(1 for r in results if r['kept']) >= TARGET_KEPT:
            break
        print(f"  [{i:2d}] overlap={row['overlap']:.2f} kept={row['kept']} "
              f"sub@k={row['substring'] or '-'} sem@k={row['semantic'] or '-'} "
              f"| lenient sub={row['substring_lenient'] or '-'} sem={row['semantic_lenient'] or '-'}",
              flush=True)
        print(f"       q: {q[:100]}", flush=True)

    kept = [r for r in results if r['kept']]
    summary = {'probes': len(results), 'kept': len(kept), 'ks': list(KS)}
    for name in ('substring', 'semantic', 'substring_lenient', 'semantic_lenient'):
        for k in KS:
            hit = sum(1 for r in kept if k in r[name])
            summary[f'{name}_recall@{k}'] = round(hit / len(kept), 4) if kept else None
    for name in ('baseline_ms', 'semantic_ms'):
        vals = sorted(r[name] for r in kept)
        summary[f'{name}_median'] = vals[len(vals) // 2] if vals else None

    out = ROOT / 'evidence' / 'FABRIC_RETRIEVAL_MEASURED.json'
    out.write_text(json.dumps({'summary': summary, 'results': results}, indent=2),
                   encoding='utf-8')
    print('\n=== SUMMARY (probes kept, i.e. not near-copies) ===')
    for key, val in summary.items():
        print(f'  {key}: {val}')
    print(f'\nwrote {out}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
