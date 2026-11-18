# Vendored inference engine

`vendor/llama.cpp` — the llama.cpp tree this platform serves from. It is a **nested git clone**, so
it is git-ignored by the TRM repo; the change TRM depends on is carried as a patch file instead
(`vendor/patches/`). Do not commit `vendor/llama.cpp` itself — a bare gitlink without
`.gitmodules` breaks clones.

## Provenance

| item | value |
|---|---|
| upstream | `https://github.com/ggml-org/llama.cpp` |
| upstream base commit | `571d0d5` (`model: rotate injected K/V cache for DFlash`) |
| TRM-relevant commit | `138346a` — Instella MoE gated attention + FarSkip |
| patch | `vendor/patches/0001-Instella-MoE-gated-attention-FarSkip-two-stream-resi.patch` |
| previous location | `F:/Development sites/ub3/llama.cpp` (borrowed from another project's folder) |

Q1_0/Q2_0 are **upstream** at `571d0d5`, not a local patch — that is why this fork decodes the
ternary Bonsai GGUFs while the 2026-07-18 prebuilt tree could not.

## Rebuild from scratch

```
git clone https://github.com/ggml-org/llama.cpp vendor/llama.cpp
cd vendor/llama.cpp
git checkout 571d0d5
git am ../patches/0001-Instella-MoE-gated-attention-FarSkip-two-stream-resi.patch
cp -r <ui-dist-assets> tools/ui/dist      # see "Offline UI assets" below
cmake -S . -B build-win -G Ninja -DCMAKE_BUILD_TYPE=Release \
      -DGGML_VULKAN=ON -DLLAMA_CURL=OFF -DGGML_NATIVE=OFF -DLLAMA_BUILD_UI=OFF
cmake --build build-win --target llama-server llama-quantize llama-cli -j 12
```

`-DGGML_NATIVE=OFF` is required: an AVX-512 build faults on this 3900X.

### Gotcha: the UI target fails offline

`llama-ui-assets` tries to download `dist.tar.gz`, and `-DLLAMA_BUILD_UI=OFF` alone does **not**
stop it (`LLAMA_USE_PREBUILT_UI=ON` still fetches). The build only proceeds when a local
`tools/ui/dist/` satisfies `llama-ui-embed`, which demands all of:
`index.html`, `manifest.webmanifest`, `sw.js`, `build.json`, `version.json`,
`bundle*.js`, `bundle*.css`, `workbox*.js`. Minimal placeholders are enough — the HTTP API is
unaffected.

### Gotcha: DLL lock

`build-win/bin/ggml-vulkan.dll` is locked by any running `llama-server.exe`. Stop the server on
`:8587`/`:8593`/`:8596` before relinking, or the link fails with
`LNK1104: cannot open file 'bin\ggml-vulkan.dll'`.

## Measured on this host (RX 6700 XT 12 GB, Vulkan, 3900X)

Instella MoE 16B-A3B, Q4_K_M 9,985 MB, fully VRAM-resident, 4k ctx:

| prompt | tok/s | prefill | wall |
|---|---|---|---|
| plain | 52.38 | 624 ms | 1.3 s |
| verdict-shaped | 94.13 | 1,985 ms | 2.3 s |
| reasoning | 94.41 | 163 ms | 0.5 s |

mean **80.31 tok/s** (n=3, `n_predict` 32, temp 0). The q8_0 build of the same weights ran at
**0.48 tok/s** because 16.9 GB spilled to CPU/HDD — VRAM residency is the whole difference.
