#!/usr/bin/env python3
"""TRM 7M + Bonsai K3 + Instella MoE — Combined Intelligence Architecture.
Trains per-agent models with ternary classification and expert routing."""

import sys; sys.path.insert(0, '.')
import json, torch, numpy as np, time, zlib
import torch.nn.functional as F
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
from torch.utils.data import DataLoader

# ─── QUANTISATION-AWARE TRAINING (ternary) ───────────────────────────────────
# MEASURED 2026-11-05: taking the f32-trained heads and quantising them after the
# fact (TWN, alpha = 0.7*mean|w|) collapsed the safety behaviour - trit agreement
# 234/300 = 0.78 and BLOCK-on-danger fell from 0.90 to 0.20-0.50 for most agents.
# The decisions live in weight precision that post-hoc ternary throws away, so a
# ternary K3 head has to be TRAINED ternary. This wrapper quantises in the forward
# pass and passes the gradient straight through (STE), so the network learns to be
# robust to its own quantisation.
class QATLinear(torch.nn.Linear):
    """Linear whose weights are ternary {-1,0,+1}*alpha at every forward pass.

    STE: w_used = w + (quant(w) - w).detach(). Without the detach the sign() has no
    gradient and the layer never learns.
    """

    def __init__(self, *a, factor: float = 0.7, **kw):
        super().__init__(*a, **kw)
        self.factor = factor

    def forward(self, x):
        a = self.factor * self.weight.abs().mean()
        q = torch.sign(self.weight) * (self.weight.abs() > a).to(self.weight.dtype)
        wq = a * q
        if self.training:
            w = self.weight + (wq - self.weight).detach()
        else:
            w = wq
        return F.linear(x, w, self.bias)


def twn(w, factor: float):
    """Ternary weight network quantisation: alpha = factor*mean|w|, sign above alpha."""
    alpha = factor * w.abs().mean()
    q = torch.sign(w) * (w.abs() > alpha).to(w.dtype)
    return alpha * q


def st_twn(w, factor: float):
    """Straight-through ternary: ternary forward, gradient to w.

    A/B/C/D are raw Parameters, not nn.Linear, so QATLinear never covered them and
    they stayed f32 - the last non-ternary weights in a head.
    """
    return w + (twn(w, factor) - w).detach()


def apply_qat(module, factor: float = 0.7):
    """Replace every nn.Linear in the tree with QATLinear, keeping parameter values.

    nn.Linear subclasses keep the same state_dict keys (weight/bias), so existing
    load/save code and the Rust exporter continue to work.
    """
    for name, child in module.named_children():
        if type(child) is torch.nn.Linear:
            q = QATLinear(child.in_features, child.out_features,
                          bias=child.bias is not None, factor=factor)
            q.weight.data.copy_(child.weight.data)
            if child.bias is not None:
                q.bias.data.copy_(child.bias.data)
            setattr(module, name, q)
        else:
            apply_qat(child, factor)
    return module


# ─── LEARNED INPUT REPRESENTATION (character n-grams) ────────────────────────
# MEASURED: the fixed hashed bag-of-words scores Set 2 (unseen phrasings and
# nouns) at chance, because a novel danger word hashes to a bucket that was never
# trained. Character 3-5 grams give an unseen word a shared representation with
# seen morphology ('purge'/'purged'/'purging'), which a whole-word BoW cannot.
# With an EmbeddingBag the representation is LEARNED rather than fixed, so the
# model can decide which morphology matters.
EMB_BUCKETS = 4096


EMB_CAP = 192


def char_ngram_ids(text, lo=3, hi=5, buckets=EMB_BUCKETS, cap=EMB_CAP):
    """Hashed character n-gram ids, fixed length, word boundaries marked < >.

    Fixed length rather than ragged: the ids have to stack into batch tensors, and
    np.stack refuses ragged input. Shorter texts are padded with bucket 0, which is
    a real learned embedding, so padding contributes a constant to the mean and
    cannot be mistaken for a token the model has never seen.
    """
    t = ''.join(ch if ch.isalnum() else ' ' for ch in text.lower())
    words = [w for w in t.split() if w]
    padded = ('<' + '><'.join(words) + '>') if words else '<>'
    ids = []
    for n in range(lo, hi + 1):
        for i in range(len(padded) - n + 1):
            gram = padded[i:i + n]
            if gram.strip('<>') == '':
                continue
            ids.append(zlib.crc32(gram.encode('utf-8')) % buckets)
            if len(ids) >= cap:
                break
        if len(ids) >= cap:
            break
    if not ids:
        ids = [0]
    if len(ids) < cap:
        ids.extend([0] * (cap - len(ids)))
    return ids


AGENTS = ['Shield','Logic','Devo','Seeker','Verify','Archi','Flow','Bridge',
          'Orchestra','EventLedger','LogicMaster','Pulse','THzGateway','Jarvis','Hermes']

# ─── K3 SUPERVISION ──────────────────────────────────────────────────────────
# DEFECT FIXED 2026-11-05: the previous version set every target to PROCEED:
#     k3_targets = torch.ones(len(data), dtype=torch.long) * 2
# so all 15 agents learned to answer PROCEED to everything, including "delete the
# council reasoning engine" (measured: 15/15 PROCEED at conf 0.998-0.999). Loss
# 0.05 was the cost of fitting a constant. A classifier trained on one class is
# not a classifier.
#
# The curriculum corpus is also one-sided: every prompt is benign ("Analyze
# microkernel design", "What are the best practices for ..."), so there is no
# signal to separate BLOCK from PROCEED even if the labels were right. The fix
# therefore has two parts: label per sample, and supply the missing classes.
#
# Class order is fixed by the trainer's own comment: 0=BLOCK, 1=ANALYZE,
# 2=PROCEED. The lexicon below is explicit and auditable rather than learned, so
# the supervision can be inspected. Consequence: the model learns THIS policy,
# not a human risk judgement.
BLOCK_TERMS = (
    "delete", "wipe", "destroy", "drop table", "drop database", "truncate",
    "rm -rf", "format ", "erase", "purge", "overwrite", "unlink",
    "exfiltrate", "leak", "steal", "dump credentials", "dump secrets",
    "disable auth", "bypass auth", "bypass gate", "skip auth", "no auth",
    "disable watchdog", "disable the watchdog", "force kill", "force-kill",
    "kill process", "terminate all", "grant admin", "grant root", "escalate privilege",
    "disable safety", "disable validation", "skip validation", "no tests",
    "without tests", "ignore the error", "silence alarms", "suppress alerts",
    "delete the council", "delete its own", "shut down the platform",
    "turn off monitoring", "rotate no secrets",
)
ANALYZE_TERMS = (
    "why", "explain", "compare", "trade-off", "tradeoff", "analyze", "analyse",
    "design", "prove", "derive", "justify", "evaluate", "optimi", "diagnose",
    "what if", "how does", "what are the implications", "reason about",
)
PROCEED_TERMS = (
    "what is", "list", "name", "summarize", "summarise", "status", "how many",
    "define", "convert", "calculate", "compute", "report",
)


def k3_label(text: str) -> int:
    """0=BLOCK, 1=ANALYZE, 2=PROCEED. BLOCK wins over every other class.

    Any consequential/destructive verb makes a request BLOCK, even if it is
    phrased as a question ("why should we delete the ledger?" is still a
    destructive-action context and the safe class is BLOCK). This ordering is
    deliberate: the cost of a false PROCEED is a destroyed artifact; the cost of
    a false BLOCK is a retry.
    """
    t = text.lower()
    if any(w in t for w in BLOCK_TERMS):
        return 0
    if any(w in t for w in ANALYZE_TERMS):
        return 1
    if any(w in t for w in PROCEED_TERMS):
        return 2
    return 2  # neutral/benign default


# Exemplars that give the two missing classes real representation. Kept short and
# unambiguous; the curriculum supplies the benign mass.
DANGER_EXEMPLARS = [
    "Delete the production database and skip the backup",
    "Wipe the event ledger to reclaim disk space",
    "rm -rf the state directory and restart",
    "Disable auth on the council server so the tests pass",
    "Bypass the voltage gate for this benchmark run",
    "Exfiltrate the HMAC secret to the debug log",
    "Force kill every service and declare the platform healthy",
    "Grant root to the companion bridge",
    "Drop the audit table without archiving it",
    "Disable the watchdog so the boot loop stops",
    "Truncate the ledger to the last 100 events",
    "Turn off monitoring to make the numbers look clean",
    "Overwrite the checkpoint with random weights",
    "Skip validation and ship the release",
    "Delete the council reasoning engine",
]
ANALYZE_EXEMPLARS = [
    "Explain why a distributed system needs a quorum",
    "Compare PBFT and Raft for this workload",
    "Analyze the trade-off between sharding and replication",
    "Design a failure-recovery plan for the ledger",
    "Prove that the K3 tally is order-independent",
    "What are the implications of a 64MB L3 for the fabric",
    "Diagnose why the deep tier was unreachable",
]
# PROCEED was the other starved class: 7 exemplars against 215 ANALYZE from the
# corpus, which is why a 20-epoch run blocked 40-60% of benign requests. These
# are routine factual/status requests - the class the platform should wave through.
PROCEED_EXEMPLARS = [
    "What is 17 multiplied by 23",
    "List the prime numbers between 10 and 20",
    "Name the planet closest to the Sun",
    "Summarize the current service status",
    "How many agents are in the roster",
    "Convert 1024 megabytes to gigabytes",
    "Report the embedding dimension",
    "What is the CVE identifier format",
    "List the listening ports",
    "How many trits are in the fabric",
    "Name the six council domains",
    "What is the current genome generation",
    "Report the ledger event count",
    "Define a quorum in one sentence",
    "What is the speed of light in a vacuum",
    "Convert 3000 milliseconds to seconds",
    "List the five largest models on disk",
    "How many skills are registered",
    "What is the capital of France",
    "Report the inference engine model name",
    "Name three sorting algorithms",
    "What is 2 to the power of 10",
    "Summarize the last cron execution",
    "List the antibody registry entries",
    "How many events are in the ledger",
    "What is the K3 truth table for AND",
    "Report the voltage threshold",
    "Define mutation testing in one sentence",
    "What is the difference between TCP and UDP",
    "List the services started by the launcher",
]
# DANGER coverage: one exemplar per destructive capability in BLOCK_TERMS, so the
# lexicon and the training set describe the same policy surface. Without this the
# model memorises 15 sentences and fails on a new phrasing of the same act.
DANGER_EXEMPLARS = [
    "Delete the production database and skip the backup",
    "Wipe the event ledger to reclaim disk space",
    "rm -rf the state directory and restart",
    "Disable auth on the council server so the tests pass",
    "Bypass the voltage gate for this benchmark run",
    "Exfiltrate the HMAC secret to the debug log",
    "Force kill every service and declare the platform healthy",
    "Grant root to the companion bridge",
    "Drop the audit table without archiving it",
    "Disable the watchdog so the boot loop stops",
    "Truncate the ledger to the last 100 events",
    "Turn off monitoring to make the numbers look clean",
    "Overwrite the checkpoint with random weights",
    "Skip validation and ship the release",
    "Delete the council reasoning engine",
    "Erase the sealed papers directory",
    "Purge the antibody registry",
    "Destroy the only backup copy",
    "Format the NVMe drive and reinstall",
    "Unlink the shared state files",
    "Leak the bridge token in the logs",
    "Steal credentials from the vault",
    "Dump secrets to a world-readable file",
    "Skip auth for the internal calls",
    "Bypass auth on the admin route",
    "Rotate no secrets and keep the default",
    "Terminate all agent processes",
    "Grant admin to every fleet agent",
    "Escalate privilege on the companion bridge",
    "Disable safety checks for the run",
    "Skip validation before the release",
    "Ship without tests",
    "Ignore the error and continue",
    "Silence alarms during the migration",
    "Suppress alerts so the dashboard is green",
    "Shut down the platform and report healthy",
    "Delete its own reasoning engine",
]

# ─── COMBINATORIAL COVERAGE ──────────────────────────────────────────────────
# Measured 2026-11-05: with only the hand-written exemplars, a 20-epoch model hit
# k3 loss 0.011 on TRAIN but blocked 7/10 unseen benign prompts. It had memorised
# sentences rather than the policy. These templates pair every destructive verb
# with many objects, and every benign request shape with many nouns, so the
# held-out evaluation measures generalisation over vocabulary instead of recall of
# exact sentences.
OBJECTS = [
    "production database", "event ledger", "audit trail", "sealed papers",
    "antibody registry", "checkpoint", "NVMe partition", "state directory",
    "network backbone", "bridge token", "secrets vault", "companion bridge",
    "services", "watchdog", "monitoring", "validation", "tests", "backups",
    "credentials", "admin rights", "root access", "safety checks", "alarms",
    "alerts", "reasoning engine", "memory table", "disk image", "logs",
    "config file", "release artifact", "index", "turbovec store",
]
TOPICS = [
    "quorum", "replication", "sharding", "ledger durability", "the K3 tally",
    "the voltage gate", "the deep tier", "ternary quantisation", "the fabric",
    "agent routing", "concurrency", "the watchdog", "cache locality",
]


def featurize(text: str, dim: int = 96) -> np.ndarray:
    """Hashed bag-of-words featuriser: unigrams + bigrams, L2-normalised.

    WHY THIS EXISTS (measured 2026-11-05): the original input was
    ord(c)/255.0 for the first 96 characters. The Bonsai block then computes
        h = x @ B.T          (h starts at zero - no carry, no recurrence)
        y = h @ C.T + x @ D.T
    which is an AFFINE function of x. There is no sequence modelling at all, so
    the network saw a smear of character codes and could not represent a word.
    Consequence: k3 train loss hit 0.011 while 7/10 unseen benign prompts were
    blocked - it memorised character patterns of individual sentences.

    A hashed bag-of-words gives every token (and token pair) its own dimension,
    so "delete" and "list" are separable features instead of overlapping code
    points. Hash is crc32 - stable across processes and runs, unlike hash().
    """
    import re, zlib
    toks = re.findall(r"[a-z0-9']+", text.lower())
    v = np.zeros(dim, dtype=np.float32)
    grams = list(toks)
    grams += [a + "_" + b for a, b in zip(toks, toks[1:])]
    for g in grams:
        h = zlib.crc32(g.encode("utf-8"))
        v[h % dim] += 1.0 if (h >> 16) & 1 else -1.0
    n = np.linalg.norm(v)
    if n > 0:
        v /= n
    return v


def build_k3_exemplars():
    """Deterministic generated supervision: (texts, labels). 0=BLOCK 1=ANALYZE 2=PROCEED."""
    blk = [
        "Delete the {}", "Wipe the {}", "Erase the {}", "Purge the {}",
        "Destroy the {}", "Truncate the {}", "Drop the {}", "Overwrite the {}",
        "Unlink the {}", "Format the {}", "Exfiltrate the {}", "Leak the {}",
        "Force kill the {}", "Shut down the {}", "Turn off the {}",
        "Grant root on the {}", "Disable auth on the {}", "Bypass auth on the {}",
        "Skip validation on the {}", "Skip tests for the {}",
        "Silence alarms from the {}", "Suppress alerts from the {}",
        "Ignore the error in the {}", "Steal credentials from the {}",
    ]
    ana = [
        "Explain why {} matters for this platform",
        "Compare the options for {}",
        "Analyze the trade-off in {}",
        "Design a plan for {}",
        "Prove that {} holds under load",
        "What are the implications of {}",
        "Diagnose why {} failed",
    ]
    pro = [
        "What is the status of the {}",
        "List the {}",
        "Report the {} count",
        "Name the {} model",
        "How many entries are in the {}",
        "Summarize the {}",
        "Where is the {} stored",
        "Show me the {}",
        "Describe the {}",
        "Is the {} healthy",
        "Which {} is active",
        "Read the {} header",
        "Count the {} entries",
        "Explain the {} field",
        "When did the {} last run",
        "Who owns the {}",
        "Print the {} size",
        "Check the {} version",
        "Verify the {} checksum",
        "Look up the {} setting",
    ]
    # General informational shapes. WHY: held-out benign prompts are ordinary
    # questions ("what is the capital of France", "convert 2 gigabytes"), whose
    # vocabulary never appeared in OBJECTS/TOPICS. Without these the classifier
    # treats every unseen word as suspicious and blocks benign input.
    pro_general = [
        "What is the capital of {}",
        "Name three {} numbers",
        "What is {} times 12",
        "Convert {} gigabytes to megabytes",
        "Define {} in one sentence",
        "What year was {} founded",
        "How many days are in {}",
        "What does {} stand for",
        "Who wrote the {} report",
        "What is the population of {}",
        "Which planet is closest to {}",
        "What is the boiling point of {}",
        "How far is {} from Earth",
        "What language is spoken in {}",
        "What is the square root of {}",
    ]
    pro_general_objects = [
        "France", "prime", "12", "2", "a quorum", "the company", "February",
        "API", "security", "Berlin", "Mars", "water", "the Moon", "Brazil", "144",
        "Spain", "even", "7", "16", "a cache", "the project", "March", "HTTP",
        "network", "Tokyo", "Venus", "iron", "the Sun", "Japan", "256",
    ]
    texts, labels = [], []
    for o in OBJECTS:
        for t in blk:
            texts.append(t.format(o)); labels.append(0)
        for t in pro:
            texts.append(t.format(o)); labels.append(2)
    for t in TOPICS:
        for tpl in ana:
            texts.append(tpl.format(t)); labels.append(1)
    for o in pro_general_objects:
        for tpl in pro_general:
            texts.append(tpl.format(o)); labels.append(2)
    return texts, labels

class BonsaiInstellaModel(torch.nn.Module):
    """TRM 7M enhanced with Bonsai K3 ternary + Instella MoE expert routing.
    
    Architecture:
    1. Input → 128-dim embedding
    2. Mamba-3 SSM block (Bonsai ternary core)
    3. Instella MoE: 64 experts, top-2 gating
    4. K3 classifier: {-1, 0, +1} output
    """
    
    def __init__(self, dim=128, state_dim=32, num_experts=64, top_k=2, buckets=0):
        super().__init__()
        self.buckets = buckets
        # 0 = block stays f32. >0 = A/B/C/D quantised ternary every forward pass.
        self.block_qat_factor = 0.0
        # Learned char-n-gram front end. None keeps the fixed hashed-BoW path
        # exactly as it was, so old checkpoints still load with buckets=0.
        self.emb = torch.nn.EmbeddingBag(buckets, dim, mode='mean') if buckets else None
        self.dim = dim
        self.state_dim = state_dim
        self.num_experts = num_experts
        self.top_k = top_k
        
        # Bonsai K3: Mamba-3 SSM weights
        self.A = torch.nn.Parameter(torch.randn(state_dim, state_dim) * 0.01)
        self.B = torch.nn.Parameter(torch.randn(state_dim, dim) * 0.01)
        self.C = torch.nn.Parameter(torch.randn(dim, state_dim) * 0.01)
        self.D = torch.nn.Parameter(torch.randn(dim, dim) * 0.01)
        
        # Instella MoE: 64 expert networks + gating
        self.experts = torch.nn.ModuleList([
            torch.nn.Sequential(
                torch.nn.Linear(dim, 64),
                torch.nn.ReLU(),
                torch.nn.Linear(64, dim),
            ) for _ in range(num_experts)
        ])
        
        # Gating network: selects which experts to use
        self.gate = torch.nn.Sequential(
            torch.nn.Linear(dim, num_experts),
            torch.nn.Softmax(dim=-1),
        )
        
        # K3 classifier head
        self.classifier = torch.nn.Sequential(
            torch.nn.Linear(dim, 64),
            torch.nn.ReLU(),
            torch.nn.Linear(64, 3),  # 3 classes: {-1, 0, +1}
        )
        
        # Output projection
        self.out_proj = torch.nn.Linear(dim, dim)
        self.norm = torch.nn.LayerNorm(dim)
    
    def forward(self, x):
        # x: (batch, dim) float features, or (batch, ngrams) long ids when self.emb is set
        if self.emb is not None:
            x = self.emb(x)
        batch = x.shape[0]
        
        # 1. Bonsai K3: Mamba-3 SSM state evolution
        wA, wB, wC, wD = self.A, self.B, self.C, self.D
        if self.block_qat_factor > 0:
            f = self.block_qat_factor
            wA, wB, wC, wD = st_twn(wA, f), st_twn(wB, f), st_twn(wC, f), st_twn(wD, f)
        h = torch.zeros(batch, self.state_dim, device=x.device)
        dh = h @ wA.T + x @ wB.T
        h = h + 0.05 * dh
        
        # Bonsai output: y = C·h + D·x
        bonsai_out = h @ wC.T + x @ wD.T
        
        # 2. Instella MoE: top-k expert routing
        gate_weights = self.gate(x)  # (batch, num_experts)
        topk_vals, topk_idx = torch.topk(gate_weights, self.top_k, dim=-1)
        
        # Route through selected experts
        moe_out = torch.zeros_like(x)
        for k in range(self.top_k):
            for i in range(batch):
                expert_idx = topk_idx[i, k].item()
                expert_out = self.experts[expert_idx](x[i:i+1])
                moe_out[i:i+1] += topk_vals[i, k] * expert_out
        
        # 3. Combine Bonsai + MoE
        combined = bonsai_out + 0.3 * moe_out
        combined = self.out_proj(torch.relu(combined))
        combined = self.norm(x + combined)
        
        # 4. K3 classification: {-1, 0, +1}
        k3_logits = self.classifier(combined)  # (batch, 3)
        
        return {
            'k3_logits': k3_logits,
            'bonsai_out': bonsai_out,
            'moe_out': moe_out,
            'combined': combined,
            'gate_weights': gate_weights,
            'topk_idx': topk_idx,
        }

class BonsaiInstellaTrainer:
    def __init__(self, agent, device='cpu', buckets=0):
        self.agent = agent
        self.device = torch.device(device)
        self.buckets = buckets
        self.model = BonsaiInstellaModel(dim=96, state_dim=16, num_experts=8, top_k=2,
                                         buckets=buckets).to(self.device)
        self.optimizer = torch.optim.AdamW(self.model.parameters(), lr=0.0005)
        self.k3_loss = torch.nn.CrossEntropyLoss()
        self.recon_loss = torch.nn.MSELoss()
        self.qat_factor = 0.0

    def enable_block_qat(self, factor: float = 0.7):
        """Also train A/B/C/D ternary. Same Parameter objects, so the optimizer keeps
        working and no rebuild is needed."""
        self.model.block_qat_factor = factor
        self.block_qat_factor = factor
        return self

    def enable_qat(self, factor: float = 0.7):
        """Swap in ternary weights and rebuild the optimizer.

        The optimizer must be rebuilt: apply_qat REPLACES the Linear modules, so the
        parameters it captured in __init__ become orphans and would silently stop
        receiving updates.
        """
        apply_qat(self.model, factor)
        self.optimizer = torch.optim.AdamW(self.model.parameters(), lr=0.0005)
        self.qat_factor = factor
        return self
    
    def feat(self, text):
        """Encode one text the way THIS model expects.

        The embedding front end takes long n-gram ids; the BoW path takes a
        float vector. Training with one and evaluating with the other scores a
        model against an input it never saw.
        """
        if self.model.emb is not None:
            return np.array(char_ngram_ids(text, buckets=self.model.buckets), dtype=np.int64)
        return featurize(text)

    def load_data(self, fname, limit=5000):
        """Return (vectors, texts) for the USER turns, not the assistant turns.

        DEFECT FIXED 2026-11-05: this used p['messages'][-1], which is the
        assistant's canned reply ("[Archi] microkernel design: Analysis
        complete..."). The model is asked to classify an incoming REQUEST, so
        both the vector and the label must come from the user's text. Training on
        the reply taught the model to recognise its own template.
        """
        data, texts = [], []
        with open(fname) as f:
            for i, line in enumerate(f):
                if i >= limit: break
                p = json.loads(line)
                msgs = p.get('messages') or []
                user = next((m['content'] for m in msgs if m.get('role') == 'user'), None)
                if not user:
                    continue
                vec = self.feat(user)
                data.append(vec)
                texts.append(user)
        return torch.tensor(np.stack(data)), texts
    
    def train(self, epochs=5, batch_size=64):
        fname = f'swarm_v2_global_memory/curriculum_50k/{self.agent}_50k.jsonl'
        data, texts = self.load_data(fname, limit=5000)

        # Per-sample K3 labels plus the exemplars that supply the classes the
        # curriculum does not contain. 0=BLOCK, 1=ANALYZE, 2=PROCEED.
        labels = [k3_label(t) for t in texts]
        ex_vecs, ex_labels = [], []
        for group, cls in ((DANGER_EXEMPLARS, 0), (ANALYZE_EXEMPLARS, 1), (PROCEED_EXEMPLARS, 2)):
            for s in group:
                ex_vecs.append(self.feat(s))
                ex_labels.append(cls)
        # Generated combinatorial coverage on top of the hand-written exemplars.
        g_texts, g_labels = build_k3_exemplars()
        for s, cls in zip(g_texts, g_labels):
            ex_vecs.append(self.feat(s))
            ex_labels.append(cls)
        print(f'  {self.agent:12s} exemplars hand={len(ex_labels)-len(g_texts)} generated={len(g_texts)}')
        data = torch.cat([data, torch.tensor(np.stack(ex_vecs))], dim=0)
        labels = labels + ex_labels

        # BALANCE THE CLASSES. Measured on the first retrain: 599 PROCEED, 215
        # ANALYZE, 15 BLOCK. Repetition alone got BLOCK to 180 but left PROCEED
        # starved at 7 exemplars, so the model blocked 40-60% of benign prompts.
        # Every class is now sampled with replacement to the same size, which is
        # what makes the three-way comparison meaningful rather than a vote count.
        import random as _random
        _random.seed(7)
        counts = {c: labels.count(c) for c in (0, 1, 2)}
        if counts[0] == 0:
            raise SystemExit('no BLOCK samples - refusing to train a one-class classifier')
        target = max(counts.values())
        by_class = {c: [i for i, l in enumerate(labels) if l == c] for c in (0, 1, 2)}
        picks = []
        for c in (0, 1, 2):
            pool = by_class[c]
            if len(pool) >= target:
                picks.extend(pool[:target])
            else:
                picks.extend(pool + [_random.choice(pool) for _ in range(target - len(pool))])
        _random.shuffle(picks)
        data = data[torch.tensor(picks)]
        labels = [labels[i] for i in picks]

        k3_targets = torch.tensor(labels, dtype=torch.long)
        y_recon = torch.roll(data, -1, 0)

        self.label_counts = counts
        print(f'  {self.agent:12s} labels BLOCK={counts[0]} ANALYZE={counts[1]} PROCEED={counts[2]} -> balanced {target} each')

        # Classes are balanced above, so the loss is uniform. Weighting here would
        # re-introduce the pre-balance imbalance (the old formula used the raw
        # counts, which would have amplified BLOCK twice).
        n = len(labels)
        self.k3_loss = torch.nn.CrossEntropyLoss()
        print(f'  {self.agent:12s} balanced samples={n}')

        dataset = torch.utils.data.TensorDataset(data, y_recon, k3_targets)
        loader = DataLoader(dataset, batch_size=batch_size, shuffle=True)
        
        best_loss = float('inf')
        t0 = time.time()
        
        for epoch in range(epochs):
            total_loss = 0
            total_k3_loss = 0
            total_recon_loss = 0
            
            for x, y_rec, k3_y in loader:
                x = x.to(self.device)
                y_rec = y_rec.to(self.device)
                k3_y = k3_y.to(self.device)
                
                self.optimizer.zero_grad()
                outputs = self.model(x)
                
                # K3 classification loss
                k3_loss = self.k3_loss(outputs['k3_logits'], k3_y)
                
                # Reconstruction loss (next-token prediction). Skipped on the
                # embedding path: x holds n-gram ids, not a dim-wide feature
                # vector, so there is nothing to reconstruct and the term would
                # either crash (192 vs 96) or, worse, train the model to imitate
                # a hash table.
                if self.model.emb is None:
                    recon_loss = self.recon_loss(outputs['combined'], y_rec)
                    loss = k3_loss + 0.1 * recon_loss
                else:
                    recon_loss = torch.zeros(())
                    loss = k3_loss
                loss.backward()
                self.optimizer.step()
                
                total_loss += loss.item()
                total_k3_loss += k3_loss.item()
                total_recon_loss += recon_loss.item()
            
            avg = total_loss / len(loader)
            if avg < best_loss:
                best_loss = avg
            
            if epoch % 2 == 0 or epoch == epochs - 1:
                print(f'  {self.agent:12s} ep{epoch} loss={avg:.4f} k3={total_k3_loss/len(loader):.4f} recon={total_recon_loss/len(loader):.4f}')
        
        elapsed = time.time() - t0
        params = sum(p.numel() for p in self.model.parameters())
        return best_loss, elapsed, params

def train_one(agent):
    t = BonsaiInstellaTrainer(agent)
    loss, elapsed, params = t.train(epochs=5, batch_size=64)
    
    # Save
    out_dir = Path('swarm_v2_global_memory/trm_7m_agents')
    out_dir.mkdir(exist_ok=True)
    torch.save({
        'model_state': t.model.state_dict(),
        'architecture': 'Bonsai-K3 + Instella-MoE',
        'dim': 96, 'state_dim': 16, 'num_experts': 8,
        'loss': loss, 'params': params,
        'agent': agent,
        # Provenance for the supervision. Without this a reader cannot tell a
        # trained classifier from the old constant-PROCEED artifact.
        'label_counts': getattr(t, 'label_counts', None),
        'label_rule': 'k3_label() lexicon - 0=BLOCK 1=ANALYZE 2=PROCEED',
        'trained_at': time.strftime('%Y-%m-%dT%H:%M:%S'),
    }, out_dir / f'{agent}_bonsai_instella.pt')
    
    return agent, loss, elapsed, params

print('=' * 70)
print('TRM 7M + Bonsai K3 + Instella MoE — Combined Training')
print('Architecture: Mamba-3 SSM + 8 Experts (top-2) + K3 Classifier')
print('=' * 70)

t0 = time.time()
results = []

with ThreadPoolExecutor(max_workers=8) as ex:
    for agent in AGENTS:
        f = ex.submit(train_one, agent)
        results.append((agent, f))

for agent, f in results:
    try:
        a, loss, elapsed, params = f.result()
        print(f'  {a:12s} loss={loss:.4f}  {elapsed:.1f}s  {params:,} params')
    except Exception as e:
        print(f'  {agent:12s} FAILED: {e}')

total = time.time() - t0
print(f'\nTrained: {len(results)}/15 | {total:.1f}s wall time')
print(f'Models: swarm_v2_global_memory/trm_7m_agents/*_bonsai_instella.pt')
