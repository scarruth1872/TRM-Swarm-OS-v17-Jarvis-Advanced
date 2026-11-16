# Structured index - full ingest, and a corrupt source found (2026-09-23)

## Final state

```
ethics      95,847 rows   all five subsets, every row with text
molecule 2,238,290 rows   every row with a real ChEMBL id
clinical   201,572 rows   40 measurements x 400 patients
prose        1,633 rows   Shakespeare / Law of One / wisdom
structure        9 rows   PDB with resolution + atom counts
                                    408,739,840 B   sha256 d91ec8e1c7038281...
built in 107 s (molecules: 80.3 s)
```

## The source is truncated, and raising the bound is what revealed it

`chembl_37.sdf.gz` is **733,869,465 bytes with no end-of-stream marker**. Independent test:

```
$ gzip -t chembl_37.sdf.gz
gzip: chembl_37.sdf.gz: unexpected end of file
        (trailer declares 2,793,187,232 uncompressed bytes)
```

**Every earlier bounded run passed.** 5k, 250k and 1M all stopped at their limit and never reached EOF,
so the corrupt source was invisible - the bound was hiding it. Only streaming to the end surfaced it,
and then only as an uncaught `EOFError` inside the parser. That is the lesson worth keeping: a bound
that stops before the end of a stream cannot vouch for the stream.

The parser now reads to EOF, keeps and commits what is readable, and records the truncation in
`provenance` so the artifact carries its own caveat:

> 2238290 records read (unrestricted, streamed to EOF). TRUNCATED SOURCE: the gzip ended before its
> stream terminator; the trailer declares 2793187232 uncompressed bytes. Partial ChEMBL download -
> re-fetch to complete it.

## Sizing, now measured instead of extrapolated

My earlier extrapolation said ~145 bytes/molecule and ~320 MB for the full set. Measured on the real
data: 2,238,290 records produce ~408.7 MB, i.e. **~183 bytes/molecule** - the estimate was ~20% low,
which is why the rule is to derive rather than assert. Time was accurate: ~80 s of parsing, 107 s total.

| span | time | index |
|---|---|---|
| 250k | 7-11 s | 112.6 MB |
| 1M | 37 s | 171.6 MB *(measured)* |
| 2,238,290 (all readable) | 80 s | 408.7 MB *(measured)* |

So the earlier recommendation - the bound is not needed, the constraint is disk - holds, and even the
full set is only 0.4 GB against 119 GB free. The remaining action is not a size decision but a
**re-download**: the source itself is incomplete.

## Two process notes

- My patch to the parser replaced a `with gzip.open(...) as fh:` block but left the loop iterating `fh`,
  so the rebuild ran with a closed handle and wrote an index with **0 molecules** - content assertions
  caught it immediately (a count-based check would not have).
- A rebuild then failed with `PermissionError` on the db because **my own sqlite connection from a
  prior call was still open** in the persistent kernel session, and Windows locks it against deletion.
  Closing session handles before rebuilding is the fix.
