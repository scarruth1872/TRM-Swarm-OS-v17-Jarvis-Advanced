# ChEMBL re-fetched and fully ingested (2026-09-23)

## The source is now complete and authentic

The WSL copy was a truncated download: 733,869,465 B against a published 931,156,420 B (21.2% missing),
`gzip -t` reporting "unexpected end of file". Resumed with `curl -C -` (the missing 188 MiB), then verified
against the release's own `checksums.txt`:

```
expected sha256  2cdf4d33c5f426130c610b8627c3702804e6f10d149d76b8a1bea674fa605a77
actual   sha256  2cdf4d33c5f426130c610b8627c3702804e6f10d149d76b8a1bea674fa605a77   MATCH
gzip -t          OK (stream terminates properly)
```

That is the completeness criterion the first ingest lacked - and the reason it matters is that a bounded
read cannot vouch for a stream: 5k, 250k and 1M all passed against the corrupt file.

## Final index

```
ethics      95,847 rows   all five subsets, every row with text
molecule 2,897,819 rows   every row with a real ChEMBL id (complete release)
clinical   201,572 rows   40 measurements x 400 patients
prose        1,633 rows
structure        9 rows
                                    507,248,640 B   sha256 2f5db380bb0859d9...
ingest: 2,897,819 molecules in 99.2 s; whole index built in 128.2 s
```

The provenance row now reads "source stream complete" instead of the truncation warning - the artifact
stopped carrying a caveat because the caveat was fixed.

## A derivation that held

Before re-fetching I derived the full record count from the readable fraction: 2,238,290 records at 78.8%
of the file implies ~2.84M. The actual is **2,897,819** - within 2%. Contrast with the size estimate earlier
in this work, which was 20% low; deriving from the data beat extrapolating from a sample.

## Cost, measured

| span | records | time | index |
|---|---|---|---|
| truncated copy | 2,238,290 | 80.3 s | 408.7 MB |
| complete release | 2,897,819 | 99.2 s | 507.2 MB |

So the complete set costs ~19 s and ~99 MB more than the truncated one did. There was never a size problem
here - only a data-integrity problem, which is exactly what the plan's "bounded sample" framing would have
left in place.
