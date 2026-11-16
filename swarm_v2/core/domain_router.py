# Semantic domain router: query text -> one of 6 harness domains.
import json, math, urllib.request

EMB_URL = 'http://127.0.0.1:8593/embeddings'

# Per-domain abstain thresholds, fitted on 84 in-domain + 22 OOD queries
# (tools/calibrate_per_domain.py, 2-fold CV). Margin = top1 - top2 cosine.
#
#   Security   -1.0000  no query was ever misrouted INTO security, so accept all
#   Logic       0.0543  precision 1.000  recall 0.929  cv 1.000
#   Code        0.0594  precision 0.889  recall 0.857  cv 0.900
#   Telemetry   0.0352  precision 0.889  recall 0.786  cv 0.857
#   Synthesis   0.0138  precision 0.917  recall 0.786  cv 0.917
#   Healing     0.0912  precision 1.000  recall 1.000  cv 1.000
#
# A single global tau hid opposite failure modes: Security fails by MISSING
# (recall 0.643), Healing fails by leaking (highest threshold needed).
ABSTAIN_MARGIN = [-1.0000, 0.0543, 0.0594, 0.0352, 0.0138, 0.0912]

# Absolute similarity floor, in addition to the per-domain margin.
#
# The margin alone only asks "is the winner clearly ahead of the runner-up?".
# Off-topic text can still be uniformly, weakly similar to every domain
# keyword string, which produces a healthy margin at a low absolute score —
# that is how "what is the airspeed velocity of an unladen swallow" was
# accepted at 0.4461. The floor asks the other question: "is the winner
# similar to *anything* we know?".
#
# Fitted on the calibration corpora (48 in-domain + 12 OOD in
# tools/calibrate_domain_router.py) plus the one off-topic probe the gap report
# already measured as a failure ("what is the airspeed velocity of an unladen
# swallow", top-1 cosine 0.4465). The highest calibration OOD cosine is 0.420
# and the in-domain median is 0.538, so 0.45 sits in the empty band above every
# known off-topic sample and below the bulk of in-domain ones.
#
# Cost on the calibration set: 29/48 in-domain accepted (28 correct, precision
# 0.966) and 0/12 OOD accepted. Held-out measurement, including the in-domain
# side of the trade-off: tools/measure_domain_router.py.
SIMILARITY_FLOOR = 0.45

DOMAINS = [
    ('Security & Cryptography',
     'security vulnerability audit cryptography encryption exploit CVE hardening shield'),
    ('Logic & Mathematical Proof',
     'logic theorem proof induction mathematics derivation solver formal verification'),
    ('Code Engineering & Refactoring',
     'code bug refactor implement compile test buffer overflow function patch build rust'),
    ('Telemetry & Memory Synchronization',
     'telemetry metrics monitoring memory synchronization logs latency health heartbeat'),
    ('Synthesis & Chain-of-Thought',
     'synthesis reasoning chain of thought design architecture planning explain summary'),
    ('Autonomous Healing & Resilience',
     'healing resilience restart crash recover failover chaos sentinel respawn watchdog'),
]

_DOMAIN_VECS = None

def _embed(texts):
    body = json.dumps({'input': list(texts)}).encode()
    req = urllib.request.Request(EMB_URL, data=body,
        headers={'Content-Type': 'application/json'})
    d = json.loads(urllib.request.urlopen(req, timeout=30).read())
    out = []
    for e in d:
        v = e['embedding'] if isinstance(e, dict) else e
        while isinstance(v, list) and v and isinstance(v[0], list):
            v = v[0]
        out.append([float(x) for x in v])
    return out

def _cos(a, b):
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(x * x for x in b))
    if na == 0.0 or nb == 0.0:
        return 0.0
    return sum(x * y for x, y in zip(a, b)) / (na * nb)

def _vecs():
    global _DOMAIN_VECS
    if _DOMAIN_VECS is None:
        _DOMAIN_VECS = _embed([d[1] for d in DOMAINS])
    return _DOMAIN_VECS

def route(text):
    # Pick the harness domain by meaning; fall back to keyword match when
    # the embedding service is down (method field reports which was used).
    try:
        q = _embed([text])[0]
        vs = _vecs()
        scores = [_cos(q, v) for v in vs]
        best = max(range(len(scores)), key=lambda i: scores[i])
        rest = sorted(scores, reverse=True)[1] if len(scores) > 1 else 0.0
        margin = scores[best] - rest
        below_margin = margin < ABSTAIN_MARGIN[best]
        below_floor = scores[best] < SIMILARITY_FLOOR
        if below_floor and below_margin:
            reason = 'below_floor_and_margin'
        elif below_floor:
            reason = 'below_similarity_floor'
        elif below_margin:
            reason = 'below_domain_margin'
        else:
            reason = None
        return {'domain': DOMAINS[best][0], 'index': best,
                'score': scores[best], 'margin': margin,
                'threshold': ABSTAIN_MARGIN[best],
                'floor': SIMILARITY_FLOOR,
                'abstain': below_margin or below_floor,
                'abstain_reason': reason,
                'scores': scores, 'method': 'embedding'}
    except Exception as exc:
        low = text.lower()
        best, hit = 0, 0.0
        for i, (name, kw) in enumerate(DOMAINS):
            s = sum(1 for w in kw.split() if w in low)
            if s > hit:
                best, hit = i, s
        # Keyword fallback has no calibrated margin, so abstention is
        # undecidable here - report None rather than pretending it is False.
        return {'domain': DOMAINS[best][0], 'index': best,
                'score': float(hit), 'margin': None, 'abstain': None,
                'scores': [], 'method': 'keyword_fallback',
                'error': repr(exc)[:200]}
