#!/usr/bin/env python3
"""Scan dispatch-relevant artifacts for 64-hex tokens and flag any that are not
a known-legitimate hash. Catches arbitrarily-corrupted lock copies."""
import re,glob,os,json
os.chdir(r"F:\Development sites\TRM-Swarm-OS-v2")

KNOWN = {
 "b558c3470c2b56eed0804578fcda19356d6f00a843a8069ee017e89e2709a5da":"TRM-ABX-219 canonical lock (sha256 SMILES)",
 "4a038fad7847b9ffcec93ba8d39ea2d452977bbdb1e5f6f6f9ce7745387cd501":"TRM-ABX-082 canonical lock (sha256 SMILES)",
 "0fd85de141eb12f13197099dff101b133f90dee503a50d99c11817eec5d0003a":"trm_abx_220_recalibration.json self-hash",
 "2e8d4e9ddefd82b2":"preclinical_safety_screening_leads.json self-hash (prefix)",
 "8da811fd690129be":"eskape_spectrum_full.json self-hash (prefix)",
 "68f81849141ac225bafdaa92d1de87cc4d4650f64f1497a702c961f173e8adf6":"pdbbind_refined.k3 shard",
 "04a9e45cec4b417c2215a157ea19894d9afc544c556d6bda74ce492677570e17":"bindingdb_affinities.k3 shard",
}
# hashes whose first 8 hex chars are enough to identify
PREFIX_OK = [k[:8] for k in KNOWN]

patterns = ["*.md","*.json","dashboard/public/*.html",
            "wetlab_outreach_packages/*.json","wetlab_dispatch/**/*.md",
            "wetlab_dispatch/*.json","trm-sovereign-portal/*.md","trm-sovereign-portal/*.html"]

HEX = re.compile(r'\b[0-9a-f]{64}\b')
flags=[]
for pat in patterns:
    for f in glob.glob(pat, recursive=True):
        if f.startswith('.git'): continue
        try: s=open(f,encoding='utf-8',errors='replace').read()
        except Exception: continue
        for i,line in enumerate(s.split("\n"),1):
            for h in HEX.findall(line):
                if h[:8] not in PREFIX_OK:
                    flags.append((f,i,h[:16]+"..."+h[-8:],line.strip()[:90]))

print(f"files scanned; suspicious 64-hex tokens: {len(flags)}\n")
seen=set()
for f,i,h,ctx in flags:
    k=(f,i)
    if k in seen: continue
    seen.add(k)
    print(f"  {f}:{i}\n     token={h}\n     ctx: {ctx}\n")
