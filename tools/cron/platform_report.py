#!/usr/bin/env python3
"""platform_report.py — the platform writes its own reports.

Replaces the LLM-driven cron jobs that fetched these same endpoints and narrated them. Every value
below is a field the council server returned in this run; nothing is inferred, summed in prose, or
filled in when absent. If the kernel is unreachable the report says so and stops.

Usage:  platform_report.py <heartbeat|daily|security|evolution|learning|all>

Exit code is 0 even for a reported outage: the report IS the delivery, and a non-zero exit would turn
a real outage notice into an engine failure. Use --strict to exit 3 when the kernel is unreachable.
"""
import json
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone

BASE = "http://127.0.0.1:8022"
TIMEOUT = 6


class Unreachable(Exception):
    pass


def get(path):
    """Return (status, body-or-None). Raises Unreachable only on connection failure."""
    try:
        with urllib.request.urlopen(BASE + path, timeout=TIMEOUT) as r:
            raw = r.read().decode("utf-8", "replace")
            try:
                return r.status, json.loads(raw)
            except json.JSONDecodeError:
                return r.status, {"_raw": raw[:400]}
    except urllib.error.HTTPError as e:
        return e.code, None
    except (urllib.error.URLError, ConnectionError, TimeoutError, OSError) as e:
        raise Unreachable(f"{path}: {e}")


def fmt(v, missing="unmeasured"):
    if v is None:
        return missing
    if isinstance(v, bool):
        return "true" if v else "false"
    return str(v)


def dig(d, *keys, default=None):
    cur = d
    for k in keys:
        if not isinstance(cur, dict) or k not in cur:
            return default
        cur = cur[k]
    return cur


def header(title):
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    return f"# {title}\n_{ts}_\n"


def report_heartbeat():
    st_h, h = get("/health")
    st_m, m = get("/system/health_matrix")
    out = [header("Swarm heartbeat")]
    out.append(f"Kernel: **{fmt(dig(h,'status'))}** — {fmt(dig(h,'kernel'))} "
               f"build `{fmt(dig(h,'build_commit'))}`, uptime {fmt(dig(h,'uptime_seconds'))}s, "
               f"python_active {fmt(dig(h,'python_active'))}")
    out.append(f"Anomalies: **{fmt(dig(m,'anomaly_count'))}** — k3 {fmt(dig(m,'k3_invariant'))}, "
               f"PBFT {fmt(dig(m,'pbft_quorum'))}")
    anoms = m.get("anomalies") or []
    if anoms:
        for a in anoms:
            out.append(f"  - {fmt(a.get('service'))} :{fmt(a.get('port'))} "
                       f"{a.get('severity','')} — {a.get('message')}")
    svcs = m.get("services") or {}
    bad = [f"{k}={v.get('status')}" for k, v in svcs.items() if v.get("status") != "ONLINE"]
    out.append(f"Services: {len(svcs)} probed, {len(bad)} not ONLINE"
               + (f" ({', '.join(bad)})" if bad else ""))
    rails = h.get("voltage_rails")
    out.append(f"Voltage: measured={fmt(dig(h,'voltage_measured'))} phi_v={fmt(dig(h,'voltage_phi_v'))}"
               + (f" rails={rails}" if rails else " rails=not in /health"))
    if st_h != 200 or st_m != 200:
        out.append(f"\n**Non-200 response**: /health={st_h} /system/health_matrix={st_m}")
    return "\n".join(out)


def report_daily():
    st_h, h = get("/health")
    st_m, m = get("/system/health_matrix")
    st_t, t = get("/mesh/topology")
    st_e, e = get("/swarm/experts")
    st_v, v = get("/verification/stats")
    st_s, s = get("/learning/skills")
    experts = e if isinstance(e, list) else []
    out = [header("Swarm daily health & sync")]
    out.append(f"Kernel: {fmt(dig(h,'kernel'))} / **{fmt(dig(h,'status'))}** "
               f"build `{fmt(dig(h,'build_commit'))}` uptime {fmt(dig(h,'uptime_seconds'))}s "
               f"python_active {fmt(dig(h,'python_active'))}")
    out.append(f"Agents: {len(experts)} experts, mesh {fmt(dig(t,'node_count'))} nodes "
               f"({fmt(dig(t,'status'))}), {len(dig(t,'connections',default=[]) or [])} connections")
    out.append(f"Anomalies: **{fmt(dig(m,'anomaly_count'))}** k3 {fmt(dig(m,'k3_invariant'))} "
               f"PBFT {fmt(dig(m,'pbft_quorum'))}")
    out.append(f"Invariance: rate={fmt(dig(v,'invariance_rate'))} "
               f"total_verified={fmt(dig(v,'total_verified'))} "
               f"theorems={fmt(dig(v,'theorem_statements'))} "
               f"(source: {fmt(dig(v,'theorem_status_source'))})")
    if dig(v, "invariance_note"):
        out.append(f"  note: {dig(v,'invariance_note')}")
    out.append(f"Skills: {fmt(dig(s,'count'))}")
    unmeasured = [n for n, val in (("invariance_rate", dig(v, "invariance_rate")),) if val is None]
    if unmeasured:
        out.append(f"Unmeasured: {', '.join(unmeasured)} (endpoint returned null)")
    codes = {"/health": st_h, "/system/health_matrix": st_m, "/mesh/topology": st_t,
             "/swarm/experts": st_e, "/verification/stats": st_v, "/learning/skills": st_s}
    nonok = {k: c for k, c in codes.items() if c != 200}
    out.append(f"Endpoints: {len(codes) - len(nonok)}/{len(codes)} HTTP 200"
               + (f" — failed: {nonok}" if nonok else ""))
    return "\n".join(out)


def report_security():
    st_s, sec = get("/security/stats")
    st_t, thr = get("/security/threats")
    st_d, ddr = get("/ddr/stats")
    st_v, v = get("/verification/stats")
    stats = dig(sec, "stats", default={}) or {}
    out = [header("Swarm security & DDR posture")]
    out.append(f"Active threats: **{fmt(stats.get('active_threats'))}** "
               f"antibodies_loaded={fmt(stats.get('antibodies_loaded'))} "
               f"vault={fmt(stats.get('vault_status'))}")
    out.append(f"24h: detected {fmt(stats.get('threats_detected_24h'))}, "
               f"neutralised {fmt(stats.get('threats_neutralized_24h'))}")
    threats = thr.get("threats") if isinstance(thr, dict) else None
    out.append(f"Threat detail rows: {len(threats) if threats is not None else 'unmeasured'}")
    out.append(f"DDR: {fmt(ddr.get('total_antibodies'))} total antibodies, "
               f"{fmt(ddr.get('distinct_rules'))} distinct rules, "
               f"scans {fmt(ddr.get('scans_performed'))}, "
               f"prevented corruptions {fmt(ddr.get('prevented_corruptions'))}, "
               f"rowhammer {fmt(ddr.get('rowhammer_mitigation'))}")
    out.append(f"Invariance: rate={fmt(dig(v,'invariance_rate'))} "
               f"total_verified={fmt(dig(v,'total_verified'))}")
    flags = []
    for label, val in (("active_threats", stats.get("active_threats")),
                       ("threats_detected_24h", stats.get("threats_detected_24h")),
                       ("prevented_corruptions", ddr.get("prevented_corruptions"))):
        if isinstance(val, (int, float)) and val:
            flags.append(f"{label}={val}")
    if ddr.get("rowhammer_mitigation") not in (None, "IMPLEMENTED"):
        flags.append(f"rowhammer_mitigation={ddr.get('rowhammer_mitigation')}")
    if flags:
        out.append(f"**Flagged non-zero / not-implemented**: {', '.join(flags)}")
    codes = {"/security/stats": st_s, "/security/threats": st_t, "/ddr/stats": st_d,
             "/verification/stats": st_v}
    nonok = {k: c for k, c in codes.items() if c != 200}
    out.append(f"Endpoints: {len(codes)-len(nonok)}/{len(codes)} HTTP 200"
               + (f" — failed: {nonok}" if nonok else ""))
    return "\n".join(out)


def report_evolution():
    st_g, g = get("/evolution/genome")
    st_p, prop = get("/evolution/proposals")
    st_s, s = get("/learning/skills")
    st_c, c = get("/council/status")
    genome = dig(g, "genome", default={}) or {}
    proposals = (prop.get("proposals") if isinstance(prop, dict) else None) or []
    phases = {}
    for p in proposals:
        ph = p.get("phase", "?")
        phases[ph] = phases.get(ph, 0) + 1
    out = [header("Swarm weekly evolution & learning")]
    out.append(f"Genome: generation **{fmt(genome.get('generation'))}**, "
               f"fitness {fmt(genome.get('fitness_score'))} "
               f"(basis: {fmt(genome.get('fitness_basis'))}), "
               f"mutations accepted {fmt(genome.get('mutations_accepted'))}")
    out.append(f"Proposals: {len(proposals)} total — "
               + (", ".join(f"{k}={v}" for k, v in sorted(phases.items())) if phases else "none"))
    sentinel = (prop.get("sentinel_proposals") if isinstance(prop, dict) else None) or []
    out.append(f"Sentinel proposals: {len(sentinel)}")
    out.append(f"Skills: {fmt(dig(s,'count'))}")
    members = dig(c, "members", default=[]) or []
    online = [m for m in members if str(m.get("status", "")).upper() == "ONLINE"]
    out.append(f"Council: {fmt(dig(c,'server'))} — {len(members)} members ({len(online)} online), "
               f"python_active {fmt(dig(c,'python_active'))}")
    pending = [p for p in proposals if not p.get("consensus_reached")]
    out.append(f"Still unresolved (consensus_reached=false): **{len(pending)}**")
    codes = {"/evolution/genome": st_g, "/evolution/proposals": st_p,
             "/learning/skills": st_s, "/council/status": st_c}
    nonok = {k: v for k, v in codes.items() if v != 200}
    out.append(f"Endpoints: {len(codes)-len(nonok)}/{len(codes)} HTTP 200"
               + (f" — failed: {nonok}" if nonok else ""))
    if genome.get("fitness_score") is None:
        out.append("Unmeasured: genome fitness_score is null (basis "
                   f"{fmt(genome.get('fitness_basis'))}) — not a zero, an unmeasured field.")
    return "\n".join(out)


def report_learning():
    st_l, l = get("/learning")
    st_s, s = get("/learning/skills")
    st_g, g = get("/evolution/genome")
    genome = dig(g, "genome", default={}) or {}
    rs = dig(l, "registry_stats", default={}) or {}
    out = [header("Swarm learning feedback loop")]
    out.append(f"Skills: **{fmt(dig(l,'count'))}** registered "
               f"(registry: {fmt(dig(l,'registry_file'))})")
    out.append(f"Registry stats: total_learned={fmt(rs.get('total_learned'))} "
               f"total_usages={fmt(rs.get('total_usages'))} log_entries={fmt(rs.get('log_entries'))}")
    out.append(f"Genome: generation {fmt(genome.get('generation'))} "
               f"fitness {fmt(genome.get('fitness_score'))}")
    un = (l.get("unmeasured") if isinstance(l, dict) else None) or []
    if un:
        out.append(f"Unmeasured: {', '.join(map(str, un))}")
    codes = {"/learning": st_l, "/learning/skills": st_s, "/evolution/genome": st_g}
    nonok = {k: v for k, v in codes.items() if v != 200}
    out.append(f"Endpoints: {len(codes)-len(nonok)}/{len(codes)} HTTP 200"
               + (f" — failed: {nonok}" if nonok else ""))
    return "\n".join(out)


def report_skills():
    st_s, s = get("/learning/skills")
    rs = dig(s, "registry_stats", default={}) or {}
    skills = (s.get("skills") if isinstance(s, dict) else None) or []
    out = [header("Swarm skill registry")]
    out.append(f"Skills: **{fmt(dig(s,'count'))}** registered "
               f"({fmt(dig(s,'registry_file'))})")
    out.append(f"Registry stats: skills={fmt(rs.get('skills'))} "
               f"total_learned={fmt(rs.get('total_learned'))} "
               f"total_usages={fmt(rs.get('total_usages'))} "
               f"log_entries={fmt(rs.get('log_entries'))}")
    for sk in skills:
        if isinstance(sk, dict):
            out.append(f"  - {sk.get('id','?')} — {sk.get('name', sk.get('description',''))}"[:160])
    if dig(s, "note"):
        out.append(f"note: {dig(s,'note')}")
    out.append("Registry is read from disk at request time; this report adds nothing to it and "
               "acquires nothing from the network — there is no automatic skill-acquisition "
               "endpoint on the native platform.")
    if st_s != 200:
        out.append(f"**Non-200**: /learning/skills={st_s}")
    return "\n".join(out)


REPORTS = {"heartbeat": report_heartbeat, "daily": report_daily, "security": report_security,
           "evolution": report_evolution, "learning": report_learning, "skills": report_skills}


def main():
    strict = "--strict" in sys.argv
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    which = args[0] if args else "heartbeat"
    try:
        if which == "all":
            print("\n\n".join(fn() for fn in REPORTS.values()))
        elif which in REPORTS:
            print(REPORTS[which]())
        else:
            print(f"platform_report.py: unknown report '{which}'; known: {', '.join(REPORTS)}, all")
            return 2
    except Unreachable as e:
        print(f"# Swarm report: COUNCIL UNREACHABLE\n\n"
              f"The council server on {BASE} did not answer ({e}).\n"
              f"No values are reported because none were retrieved. The service was not started by "
              f"this script — start-if-absent belongs to SwarmKernel (tools/swarm_kernel_guard.ps1).")
        return 3 if strict else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
