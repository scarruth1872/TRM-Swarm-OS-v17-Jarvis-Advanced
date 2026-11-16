# Third-Party Notices

Generated from `evidence/JARVIS_PROVENANCE.json` by `tools/module_provenance.py`. Every entry is a git repository
vendored inside this tree with a third-party `origin`, discovered by scanning for nested `.git` directories rather than
by trusting a README. Regenerate with:

```
python tools/module_provenance.py
```

## Vendored repositories

| path | files | upstream | commit | license |
|---|---|---|---|---|
| `openfang` | 13,135 | https://github.com/RightNow-AI/openfang.git | `2026-05-12 acf2587` | Apache-2.0 (LICENSE-APACHE retained in tree) |
| `Jarvis-Advanced-main/VibeThinker` | 88 | https://github.com/WeiboAI/VibeThinker.git | `2026-06-17 e6e3bfb` | MIT, Copyright (c) 2025 WeiboAI (LICENSE retained in tree) |

## Generated code

`Jarvis-Advanced-main/` is a Google AI Studio applet export (`metadata.json` declares
`MAJOR_CAPABILITY_SERVER_SIDE_GEMINI_API`, `package.json` is named `react-example`). Eight of its files carry the
studio's `SPDX-License-Identifier: Apache-2.0` header. It is generated scaffolding, not code from another repository,
though it vendors VibeThinker above.

## Concept sources (not code)

The research panel's vocabulary (`prism`, `fqm`, `mycorrhizal`, `tunneling`, `sierpinski`, `l-system`, `grover`) comes
from documents in downloaded folders, not from any subsystem in the kernel. Those folders are listed in
`evidence/JARVIS_PROVENANCE.json` under `concept_sources_in_downloads`. The panel states on screen that the
corresponding readings are unmeasured (G149).
