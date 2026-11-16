# Knowledge harvest: hw_kb + rust_kb into the index (2026-09-26)

Reconciliation manifest: `rust_kb` and `hw_kb` are now `harvested-prose`, not `harvest`. This is what that
means, what it measured, and what it deliberately did not do.

## What moved

`hw_kb` (18 files, 4,112,654 B) and `rust_kb` (5 `.crate` archives, 246,192 B) came out of WSL into
`swarm_v2/knowledge/` — **187 files**, of which **92 `.rs`**, **12 `.h`**, 17 `.md`, 10 `.toml`, 6 `.html`, and the
crates themselves under `rust_kb/_crates/`. One correction worth recording: the first copy took only **8 of the
18** hardware references, because a `*.h` glob cannot see into `hw_kb/kernel/` — and the ten it missed were the
kernel headers (`acpi.h`, `efi.h`, `gdt.h`, `irqflags.h`, `msr.h`, `pci.h`, `pgtable_types.h`, `processor.h`),
the most kernel-relevant part of the corpus. The sealer refused on the count and the copy was completed; two of
the 18 are duplicates (`kernel/nvme.h` == `nvme.h`, `kernel/pci_regs.h` == `pci_regs.h`), so 16 are unique. The crates are the bare-metal dependencies the kernel's own line
comes from: **bootloader 0.11.17, postcard 1.1.3, spin 0.12.2, volatile 0.6.1, x86_64 0.15.5**. The hardware
corpus is NVMe (`nvme.h`), PCI (`pci_regs.h`, `osdev_PCI.html`), UEFI/GDT/IDT/Interrupts and `ptx_isa.html`.

Prose does not go in the structured store, so the check is the index itself:
`swarm_v2_artifacts/kernel_knowledge_index_harvest.json` — **336 documents** (the 186 live keys that resolve
plus the 150 harvested text docs), 768 dims, `native-turbovec-hdc`. Build time is quoted as measured and not
from the artifact's own meta: **163.2 s for 336 docs = 2.1 docs/s**. An earlier run of this same script
reported 4.33 s for the same document count and I could not reproduce that rate, so 2.1 docs/s is the number
to use. The corpus now also carries `tox21full/` — the generator that produces the Tox21 matrix (9 files,
82,937 B, `.git`/`__pycache__` excluded) — so the assay identifiers the store holds verbatim sit beside their
producer's own documentation. Verdicts after this: **6 harvested / 3 harvested-prose / 3 harvest**.

## Determinism, measured before trusting a rebuild

The encoder is not assumed stable — three stored vectors were re-embedded and compared: **identical, 0/768
components differing**, at ~0.006–0.010 s per document. That is why the honest move was one rebuild of the
whole set rather than splicing two artifact generations together.

## The proof: the platform's own metric, on queries the harvest exists for

Cosine over the index (the metric `SemanticIndex.query` uses, line 85), same encoder that built it:

| query | live index top-1 | candidate top-1 |
|---|---|---|
| NVMe submission/completion queue doorbell | `SWARM_OS_PHD_RESEARCH_PAPER.md` 0.2159 | papers 0.2159, then **`idt.rs`** 0.1935 |
| UEFI boot services exit, memory map | `wisdom_14209.txt#32` 0.4005 | `RECONCILIATION_STATUS` 0.3332, `v0.9.md` 0.3208 |
| x86_64 GDT descriptor, bare metal | `enwikiquote.xml.bz2#57100` 0.2291 | **`gdt.rs` 0.4761, `segmentation.rs` 0.3937, `tables.rs` 0.3666** |

**All three** queries have no knowledge document anywhere in the live index's top-3; the x86_64 query lands
squarely in the candidate at nearly double the live score. Stated caveat, not glossed: the NVMe and UEFI
queries only rank a knowledge doc in the top-3 rather than first, and both indexes score coarsely overall
(0.2–0.48) — the builder embeds only the **first 6,000 characters** per document, so a header file whose
relevant structs sit later is partly invisible.

**A suspected defect that is not one.** `SemanticIndex` builds its own skill, `SpectralEmbeddingSkill`, not the
native encoder that wrote the index — which would be a silent encoder-space mismatch. Measured:
`SpectralEmbeddingSkill.embed_text` returns dim 768 and **the same vector** as `NativeEmbeddingSkill`. The
spaces match; the concern was wrong and is recorded as wrong.

## The finding: the live index is 91% one CSV, and its key list is its own output

The rebuild resolved **336 of 2,336** keys and reported **2,000 missing — every one of them a chunk key of a
single file**: `relational/ethics_commonsense.csv#1` … `#1994`. So the live index's 2,180 documents are
1,994 chunks of one structured CSV plus 186 prose documents. Two consequences:

The missing count rose from 1,994 to 2,000 when the indexer's fallback was bounded (see below): six keys had
been resolving by *bare filename* against files anywhere in the tree, which is not a path and can bind a key
to an unrelated document. They now report as missing, which is their true state.

1. **Its key list is the previous index's own output** (`--source-index` defaults to `semantic_index.json`,
   which is the mirror of the built index), so it cannot be reproduced from the tree: each rebuild silently
   drops whatever no longer resolves. That is the "declared where it should be measured" defect class, in the
   retrieval layer.
2. **Structured data in a text index**: the ethics rows are already in the store (`ethics` table, 95,847 rows,
   from `relational/ethics_commonsense.csv` 14,546,224 B). A CSV in the text index is a second, coarser owner
   of content the store already owns.

**Nothing was swapped.** The live index is untouched; the harvest is a candidate. Restoring those 1,994
retrieval paths is a decision, not a cleanup: re-chunk them from the store (the store is the owner), copy the
14.5 MB source into the tree, or accept the drop because the store answers those lookups directly.

## A duplicate found inside the tree

The WSL manifest's `duplicate` class came up empty, but the tree has a real one: `swarm_v2_artifacts/research/`
and `swarm_v2_integrated/docs/research/` each hold **12,270 `.md`** with equal names and, sampled 6/6, byte-
identical contents. It is also the obvious corpus for a declared index expansion — but size it at the
measured rate, not a declared one: at **2.1 docs/s** a 12,270-document sweep is **~1.6 hours**, where a
"163 docs/s" figure (an old artifact's self-reported meta) implied ~75 seconds. The resulting index would be
~300 MB, so it is a scoped decision rather than a silent sweep.

Also fixed while rebuilding: `swarm_v2/core/build_kernel_knowledge_index.py` re-walked the **entire repo** on
every build (`os.walk(ROOT)`), which on a WSL `/mnt/c` mount holding multi-GB artifacts never returned — two
attempts hung outright. Its fallback also matched keys by `stem[:32]` prefix anywhere in the tree. The
fallback is now bounded to the harvested corpus: builds finish in 163 s, the six bare-filename matches above
became honest misses, and nothing resolves to a path the key does not name.
