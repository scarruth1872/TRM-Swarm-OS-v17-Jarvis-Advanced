import json, sys, pathlib
sys.path.insert(0, r'F:\Development sites\TRM-Swarm-OS-v2')

DATA = {
0: [
 'check whether the login endpoint is vulnerable to sql injection',
 'review the encryption used for stored passwords',
 'scan the container image for known CVEs',
 'is the tls certificate chain validated properly',
 'find privilege escalation paths in the sudo config',
 'audit the api for broken authentication',
 'test whether session tokens are predictable',
 'check if the upload path allows remote code execution',
],
1: [
 'prove that the sum of two even numbers is even',
 'is this argument valid, check the syllogism',
 'derive the closed form for this recurrence',
 'verify the proof by contradiction is sound',
 'solve this system of linear equations',
 'check whether the theorem holds for all n',
 'formalize this statement in first order logic',
 'what is the contrapositive of this implication',
],
2: [
 'refactor this function to remove duplicated branches',
 'why does this rust borrow checker complain about the lifetime',
 'write a unit test for the parser edge cases',
 'the build fails with an unresolved import, fix it',
 'rename this variable everywhere and update call sites',
 'implement error handling for the missing file case',
 'this loop has an off by one bug, find and patch it',
 'split this large module into smaller crates',
],
3: [
 'memory usage keeps climbing, find the leak',
 'show me the request latency percentiles for today',
 'the log file is full of warnings, summarize them',
 'is the heartbeat from the node still arriving',
 'why is cpu at 100 percent on this host',
 'sync the cached vectors with the durable store',
 'track how many requests fail per minute',
 'alert me when disk space drops below ten percent',
],
4: [
 'design the module boundaries for this subsystem',
 'explain the tradeoffs between these two approaches',
 'write a short summary of what we decided',
 'plan the migration in phases and order the work',
 'connect these ideas into a coherent argument',
 'compare the alternatives and recommend one',
 'walk me through the reasoning step by step',
 'draft the architecture overview for the team',
],
5: [
 'the worker keeps dying, restart it automatically',
 'add a watchdog that respawns the process',
 'recover from the failed shard and rebalance',
 'the node is unhealthy, fail over to the replica',
 'inject a fault to test the resilience path',
 'back off and retry after the crash',
 'detect the deadlock and break it',
 'self heal the corrupted index automatically',
],
}

OOD = [
 'what is the capital of france',
 'recommend a good pizza recipe',
 'write me a poem about the sea',
 'what will the weather be tomorrow',
 'book a flight to tokyo next week',
 'who won the match last night',
 'translate this sentence to spanish',
 'how tall is mount everest',
 'give me a weekly workout plan',
 'tell me a joke about cats',
 'what is the price of bitcoin today',
 'suggest a name for my new puppy',
]

from swarm_v2.core import domain_router as R

def probe(text):
    q = R._embed([text])[0]
    vs = R._vecs()
    s = [R._cos(q, v) for v in vs]
    order = sorted(range(len(s)), key=lambda i: -s[i])
    return s, order[0], s[order[0]] - s[order[1]]

good, bad, ood = [], [], []
for label, qs in DATA.items():
    for q in qs:
        s, top, marg = probe(q)
        rec = (s[top], marg, top == label, q)
        (good if top == label else bad).append(rec)
for q in OOD:
    s, top, marg = probe(q)
    ood.append((s[top], marg, False, q))

n_in = len(good) + len(bad)
print('in-domain top-1 accuracy: %d/%d = %.3f' % (len(good), n_in, len(good)/n_in))

def med(xs, k):
    v = sorted(x[k] for x in xs)
    return v[len(v)//2] if v else 0.0

for name, grp in (('correct', good), ('wrong  ', bad), ('OOD    ', ood)):
    print('%s  n=%2d  median score=%.4f  median margin=%.4f' % (name, len(grp), med(grp,0), med(grp,1)))

cands = sorted({round(x[1], 4) for x in good + bad + ood})
best = None
for t in cands:
    tpr = sum(1 for x in good if x[1] >= t) / max(1, len(good))
    fpr = sum(1 for x in ood if x[1] >= t) / max(1, len(ood))
    j = tpr - fpr
    if best is None or j > best[0]:
        best = (j, t, tpr, fpr)

j, tau, tpr, fpr = best
acc_in = [x for x in good + bad if x[1] >= tau]
acc_correct = sum(1 for x in acc_in if x[2])
acc_ood = [x for x in ood if x[1] >= tau]
print('')
print('sweep over %d margin thresholds' % len(cands))
print('best margin tau = %.4f   (Youden J = %.3f)' % (tau, j))
print('in-domain accepted %d/%d  correct %d  precision %.3f' % (len(acc_in), n_in, acc_correct, acc_correct/max(1,len(acc_in))))
print('in-domain rejected %d (abstain)' % (n_in - len(acc_in)))
print('OOD accepted     %d/%d  -> OOD rejection %.3f' % (len(acc_ood), len(ood), 1 - len(acc_ood)/len(ood)))

print('')
print('per-domain at tau=%.4f' % tau)
print('%-38s %5s %5s %5s %5s %8s' % ('domain','n','acc','abst','conf','precis'))
for label, qs in DATA.items():
    tot = len(qs); acc = abst = conf = corr = 0
    for q in qs:
        s, top, marg = probe(q)
        if top == label: acc += 1
        if marg < tau: abst += 1
        else:
            conf += 1
            if top == label: corr += 1
    name = R.DOMAINS[label][0]
    pr = corr / conf if conf else float('nan')
    print('%-38s %5d %5d %5d %5d %8.3f' % (name[:38], tot, acc, abst, conf, pr))
