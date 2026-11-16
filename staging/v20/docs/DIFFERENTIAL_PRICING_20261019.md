# TRM Swarm OS — Differential Pricing & Revenue Model

Prepared 2026-10-19. Competitor prices are from public 2025–2026 sources (cited); our prices are the
proposed differential. Revenue figures are **scenarios, not guarantees** — the platform is pre-launch.

---

## 1. What the platform compares to (the pricing anchors)

The same platform serves five markets at once, so it can price *differentially* against five
competitors — each with its own going rate:

| platform module | compares to | real market price (public) |
|---|---|---|
| Drug-discovery pipeline (leads + triage + docking) | Schrödinger | **$1.8M avg enterprise contract** (2025); $500K–$5M/yr; academic ~$7.5K/yr |
| Sovereign AI runtime (token-free, offline) | OpenAI / Anthropic | Claude Enterprise **$40/user/mo** (25-seat min) + usage; ChatGPT Business $25–30/user/mo |
| HDC retrieval engine (2-bit, on-host) | Pinecone / Weaviate | Pinecone Enterprise **$500/mo min** + $0.33/GB; Weaviate Premium from $400/mo |
| Enterprise data + governance (council + ledger) | Palantir Foundry | **$250K–$2M/yr** commercial; $1M–$5M/yr mid-market; ~$11K/user |
| Clinical-variant classification (ACMG) | Tempus / Fabric Genomics | per-sample / per-seat enterprise contracts (unpublished) |

**The pricing power:** every incumbent is *token-billed* (OpenAI/Anthropic meter usage) or
*cloud/seat-billed* (Schrödinger/Palantir). TRM is **token-free + sovereign + self-hosted**, so the
same customer dollar buys more, and TRM's cost base is a single workstation (~$105/mo) — not a cloud
fleet.

---

## 2. The differential pricing tiers

Different customers pay different prices for the same platform, matched to what *their* market
already pays.

| tier | customer | price | anchor (why it's fair) |
|---|---|---|---|
| **Discovery Enterprise** | pharma / large biotech | **$500K–$1.8M/yr** | Schrödinger's $1.8M ACV — full drug pipeline, no per-token metering |
| **Discovery Mid** | biotech / CRO | **$150K–$500K/yr** | under Schrödinger mid-band; token-free |
| **Sovereign AI Enterprise** | defense / finance / healthcare | **$100K–$250K/yr** (self-hosted, unlimited seats) | Palantir-lite, but offline + no usage billing |
| **Sovereign AI Seat** | SMB / teams | **$40/user/mo** (annual) | Claude Enterprise parity — but no usage fee on top |
| **Retrieval API** | developers | **$500/mo min + usage** | Pinecone Enterprise parity — on-host HDC, no cloud markup |
| **Academic / Research** | universities, labs | **$7,500/yr** | Schrödinger academic parity |
| **Clinical Genomics** | hospital systems / Dx labs | per-sample or per-seat, enterprise | Tempus/Fabric parity |

**The differential:** a pharma pays $1.8M for the *same* retrieval engine an academic pays $7.5K
for — priced to their market, not to our cost. That spread is the earnings engine.

---

## 3. What Shawn Carruth could earn (3 scenarios, annual recurring)

| line | Conservative | Base | Upside |
|---|---|---|---|
| Discovery Enterprise ($1.8M) | 1 → $500K | 2 → $3.6M | 5 → $9.0M |
| Discovery Mid ($300K avg) | 1 → $150K | 5 → $1.5M | 20 → $6.0M |
| Sovereign AI Enterprise ($175K avg) | 0 | 2 → $350K | 10 → $1.75M |
| Sovereign AI Seats ($40/mo × 12) | 10 → $4.8K | 50 → $24K | 500 → $240K |
| Retrieval API ($500/mo × 12) | 1 → $6K | 5 → $30K | 50 → $300K |
| Academic ($7.5K) | 2 → $15K | 5 → $37.5K | 20 → $150K |
| Clinical Genomics (enterprise) | 0 | 1 → $150K | 3 → $600K |
| **Total ARR** | **~$676K** | **~$5.7M** | **~$18.0M** |

Plus **one-off / non-recurring**: partnership milestones (Schrödinger-style co-development with
pharma pays milestone-based, not just license), and a one-time self-hosted install fee.

**The honest frame:** these are *what the market pays for this class of capability*, not a promise.
Earning them requires closing the pre-launch gaps (docking, the 2 recall misses, the 2 unlicensed
repos) and landing the first logos. But the platform's cost base (~$105/mo) means even the
conservative scenario is ~99% margin.

---

## 4. Why the differential holds (the defensible claims)

1. **Token-free** removes the metering anxiety that caps OpenAI/Anthropic adoption — a fixed license
   is cheaper and simpler for a customer than an unbounded token bill.
2. **Sovereign/offline** unlocks regulated buyers (defense, healthcare, finance) that *cannot* use
   cloud LLMs — a segment the incumbents structurally can't serve.
3. **One architect, one workstation** = a margin incumbents (with engineering teams + cloud fleets)
   cannot match.
4. **Measured proof** (recall P@1 0.9487; 9 leads; 45,911 screening leads; 9 theorems) lets a buyer
   verify capability before paying — the honesty contract is the sales asset.
