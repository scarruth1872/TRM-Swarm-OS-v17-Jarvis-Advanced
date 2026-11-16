"""Measure retrieval with the vocabulary changed by construction, not by a paraphraser.

Two earlier attempts failed as INSTRUMENTS, and the failures are worth recording:

  attempt 1 (1.5B paraphraser, no template filter): 30 probes, 25 rejected as near-copies,
             both methods 0.2 - the sampled rows were templates and the model echoed them.
  attempt 2 (27B deep tier): 62.7 s per call, so an 80-probe run would take an hour, and it
             still kept the distinctive nouns (Jaccard 0.46), so probes kept getting
             rejected. Shared entity nouns are NORMAL in a fair query; penalising them was
             wrong.

So this harness builds the vocabulary shift mechanically. It takes memories that actually
carry a measurement or decision (586 of 42,805 distinct rows - the rest is chatter), and
rewrites them through a synonym table: council->governing body, watchdog->supervisor,
ternary->three-valued, and so on. The MEANING is preserved by construction, the exact words
the keyword matcher needs are gone. That is the case semantic retrieval exists for.

Both paths get the same probes and the same k. Overlap is reported so a reader can see the
shift actually happened. Nothing is tuned after the fact.
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
KS = (1, 5, 10)
N_PROBES = 20

# Terminology used by this platform, and the words a different person would plausibly use.
SYNONYMS = {
    'council': 'governing body', 'watchdog': 'supervisor', 'ternary': 'three-valued',
    'quantisation': 'weight compression', 'quantization': 'weight compression',
    'latency': 'delay', 'memory': 'store', 'fabric': 'knowledge network',
    'kernel': 'core', 'agent': 'worker', 'training': 'learning', 'error': 'failure',
    'failing': 'breaking', 'passed': 'succeeded', 'blocked': 'denied', 'proceed': 'allow',
    'refused': 'declined', 'measured': 'benchmarked', 'parity': 'agreement',
    'retrieval': 'lookup', 'duplicate': 'repeated', 'corpus': 'dataset',
    'inference': 'prediction', 'embedding': 'vector', 'classifier': 'categoriser',
    'benchmark': 'test suite', 'throughput': 'rate', 'sparsity': 'emptiness',
    'weights': 'parameters', 'distillation': 'compression', 'accuracy': 'correctness',
    'restart': 'relaunch', 'timeout': 'expiry', 'threshold': 'cutoff',
}


def norm(s):
    s = re.sub(r'0x[0-9a-fA-F]+', '<HEX>', str(s))
    s = re.sub(r'\b\d{3,}\b', '<N>', s)
    s = re.sub(r'\s+', ' ', s)
    return s.strip().lower()


def words(s):
    return set(re.findall(r"[a-z']{3,}", str(s).lower()))


def load_distinct():
    rows = []
    with gzip.open(FABRIC / 'fabric_distinct.jsonl.gz', 'rt', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


def knowledge_rows(rows):
    """Rows that carry a measurement or a decision - the only ones worth probing."""
    pat = re.compile(r'\b(measured|tflops|parity|latency|percent|commit|blocked|passed|'
                     r'failed|benchmark|accuracy|throughput|microsecond|nanosecond)\b', re.I)
    keep = []
    for r in rows:
        c = str(r.get('content', ''))
        if not (120 <= len(c) <= 1200):
            continue
        if r.get('class') == 'model_failure' or c.lstrip().startswith('{'):
            continue
        if pat.search(c):
            keep.append(r)
    return keep


def synonymic_probe(content):
    """Rewrite a memory through the synonym table; return None if nothing changed."""
    text = str(content)
    changed = 0
    for term, replacement in SYNONYMS.items():
        pattern = re.compile(r'\b' + re.escape(term) + r'(s|es|ed|ing)?\b', re.I)

        def swap(m):
            nonlocal changed
            changed += 1
            suffix = m.group(1) or ''
            return replacement + suffix

        text = pattern.sub(swap, text)
    if not changed:
        return None
    # Keep it query-shaped and short, the way a person would actually ask.
    text = re.sub(r'\s+', ' ', text).strip()
    return ('What happened with ' + text[:180]).rstrip()


def main():
    from swarm_v2.core.global_memory import GlobalMemorySync
    from swarm_v2.core.semantic_fabric import SemanticFabric

    fab = SemanticFabric()
    if not fab.ready:
        raise SystemExit(f'semantic index not ready: {fab.error}')

    rows = load_distinct()
    pool = knowledge_rows(rows)
    print(f'{len(rows):,} distinct -> {len(pool):,} rows carrying a measurement/decision',
          flush=True)
    if len(pool) < N_PROBES:
        raise SystemExit(f'only {len(pool)} knowledge rows; refusing to report a tiny sample')

    random.seed(5)
    random.shuffle(pool)

    probes = []
    for row in pool:
        probe = synonymic_probe(row['content'])
        if probe:
            probes.append((probe, row))
        if len(probes) >= N_PROBES:
            break
    print(f'{len(probes)} synonym-shifted probes\n', flush=True)

    sub = GlobalMemorySync()
    results = []
    for i, (probe, src) in enumerate(probes, 1):
        target = norm(src['content'])
        overlap = len(words(probe) & words(src['content'])) / max(len(words(src['content'])), 1)

        t0 = time.time()
        base = [entry for _s, entry in sub.query(probe, top_k=max(KS))]
        base_ms = (time.time() - t0) * 1000
        t0 = time.time()
        sem = fab.query(probe, k=max(KS)) or []
        sem_ms = (time.time() - t0) * 1000

        row = {
            'probe': probe,
            'source': str(src['content'])[:220],
            'word_overlap_vs_source': round(overlap, 3),
            'baseline_ms': round(base_ms, 1),
            'semantic_ms': round(sem_ms, 1),
            'substring': [k for k in KS if any(norm(h.get('content')) == target for h in base[:k])],
            'semantic': [k for k in KS if any(norm(h.get('content')) == target for h in sem[:k])],
        }
        results.append(row)
        print(f"  [{i:2d}] overlap={overlap:.2f} sub@k={row['substring'] or '-'} "
              f"sem@k={row['semantic'] or '-'} ({base_ms:.0f}ms / {sem_ms:.0f}ms)", flush=True)
        print(f"       {probe[:110]}", flush=True)

    summary = {'probes': len(results), 'ks': list(KS),
               'corpus_rows': len(rows), 'knowledge_rows': len(pool),
               'mean_word_overlap': round(sum(r['word_overlap_vs_source'] for r in results) / len(results), 3)}
    for name in ('substring', 'semantic'):
        for k in KS:
            hit = sum(1 for r in results if k in r[name])
            summary[f'{name}_recall@{k}'] = round(hit / len(results), 4)
    for name in ('baseline_ms', 'semantic_ms'):
        vals = sorted(r[name] for r in results)
        summary[f'{name}_median'] = vals[len(vals) // 2]

    out = ROOT / 'evidence' / 'FABRIC_RETRIEVAL_SYNONYM_MEASURED.json'
    out.write_text(json.dumps({'summary': summary, 'results': results}, indent=2), encoding='utf-8')
    print('\n=== SUMMARY (vocabulary shifted by the synonym table) ===')
    for k, v in summary.items():
        print(f'  {k}: {v}')
    print(f'\nwrote {out}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
