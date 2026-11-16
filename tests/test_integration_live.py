"""Live-HTTP integration tests for the nine running TRM Swarm OS services.

These tests deliberately do NOT import the application or use
``fastapi.testclient.TestClient``.  Every assertion is made against a socket on
a real port, so a handler that works in-process but is not wired into the
running stack fails here.

Why this file exists
--------------------
The previous ``integration`` tests built ``TestClient(app)`` in-process.  They
proved the handler functions work; they proved nothing about the nine live
ports.  A concrete false pass was found: Jarvis ``/health`` served
``<!doctype html>`` and a status-code-only check accepted it.  Every test below
therefore asserts the response is ``application/json`` *and* checks the JSON
shape, so a 200 carrying HTML fails.

Ports under test
----------------
    8022  Rust council (one-shot: propose returns the tally)
    8099  Python FastAPI council API (three-call: propose -> deliberate -> vote)
    3000  Jarvis
    3001  companion proxy
    3002  companion bridge
    5183  LCARS dashboard (HTML at /, JSON at /health)
    8587  llama.cpp inference
    8588  trm_server
    8593  embeddings

Overrides (used for falsification)
----------------------------------
    TRM_LIVE_HOST           host to dial, default 127.0.0.1
    TRM_LIVE_PORT_OVERRIDE  force every service onto this port, e.g. 9

    TRM_LIVE_PORT_OVERRIDE=9 pytest -m integration_live
        -> every test fails with "LIVE SERVICE UNREACHABLE ... ConnectionRefused"

Run with:
    pytest -m integration_live -v
"""

from __future__ import annotations

import json
import os
import socket
import urllib.error
import urllib.request
import uuid

import pytest

pytestmark = pytest.mark.integration_live

HOST = os.environ.get("TRM_LIVE_HOST", "127.0.0.1").strip() or "127.0.0.1"
_PORT_OVERRIDE = os.environ.get("TRM_LIVE_PORT_OVERRIDE", "").strip()

# Generous: the Rust council runs an SSM + LLM deliberation and took ~16s when
# measured, and the Python council fans out to 14 agents.
DEFAULT_TIMEOUT = 30.0
COUNCIL_TIMEOUT = 120.0


def _port(service_port: int) -> int:
    """Resolve the port to dial, honouring the falsification override."""
    if _PORT_OVERRIDE:
        try:
            return int(_PORT_OVERRIDE)
        except ValueError:  # pragma: no cover - operator typo
            pytest.fail(f"TRM_LIVE_PORT_OVERRIDE is not an integer: {_PORT_OVERRIDE!r}")
    return service_port


def _url(service_port: int, path: str) -> str:
    return f"http://{HOST}:{_port(service_port)}{path}"


def _request(service_port: int, path: str, *, method: str = "GET",
             payload: dict | None = None, timeout: float = DEFAULT_TIMEOUT,
             accept: str = "application/json") -> dict:
    """Perform a real HTTP request and return the decoded JSON object.

    Fails loudly (never silently skips) when the service is unreachable, when
    the body is not JSON, or when the body is not a JSON object.
    """
    url = _url(service_port, path)
    data = None
    headers = {"Accept": accept}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"

    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            status = resp.status
            ctype = resp.headers.get("Content-Type", "")
            raw = resp.read()
    except urllib.error.HTTPError as exc:  # a real HTTP answer, just not 2xx
        body = exc.read()[:400].decode("utf-8", "replace")
        pytest.fail(f"{method} {url} -> HTTP {exc.code} (expected 200). Body: {body!r}")
    except (urllib.error.URLError, ConnectionError, socket.timeout, OSError) as exc:
        reason = getattr(exc, "reason", exc)
        pytest.fail(
            f"LIVE SERVICE UNREACHABLE: {method} {url} -> {type(exc).__name__}: {reason}. "
            f"These tests require the real service on port {_port(service_port)}; "
            f"they do not fall back to an in-process app."
        )

    if status != 200:
        pytest.fail(f"{method} {url} -> HTTP {status} (expected 200)")

    # The Jarvis false pass: a 200 serving HTML must not be accepted.
    if "application/json" not in ctype.lower():
        snippet = raw[:200].decode("utf-8", "replace")
        pytest.fail(
            f"{method} {url} -> 200 but Content-Type is {ctype!r}, not JSON. "
            f"Body starts: {snippet!r}"
        )

    try:
        body = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        snippet = raw[:200].decode("utf-8", "replace")
        pytest.fail(f"{method} {url} -> 200 but body is not valid JSON ({exc}). Body: {snippet!r}")

    if not isinstance(body, dict):
        pytest.fail(f"{method} {url} -> JSON is {type(body).__name__}, expected an object")
    return body


def _require(obj: dict, key: str, types, where: str):
    assert key in obj, f"{where}: missing key {key!r}; got keys {sorted(obj)}"
    value = obj[key]
    assert isinstance(value, types), (
        f"{where}: key {key!r} is {type(value).__name__} ({value!r}), "
        f"expected {getattr(types, '__name__', types)}"
    )
    return value


def _unique_topic(prefix: str) -> str:
    """A topic that cannot collide with a stored verdict from an earlier run."""
    return f"{prefix}-{uuid.uuid4().hex}"


# --------------------------------------------------------------------------
# 8022 - Rust council (one-shot)
# --------------------------------------------------------------------------

def test_rust_council_health():
    body = _request(8022, "/health")
    assert body.get("status") == "HEALTHY", f"8022 /health status={body.get('status')!r}: {body}"
    _require(body, "kernel", str, "8022 /health")
    _require(body, "monad_anchor", str, "8022 /health")
    _require(body, "memory_kb", int, "8022 /health")


def test_rust_council_propose_returns_tally():
    topic = _unique_topic("ci-live-rust")
    body = _request(8022, "/council/propose", method="POST",
                    payload={"topic": topic, "category": "Architecture", "proposer": "ci"},
                    timeout=COUNCIL_TIMEOUT)
    # One-shot: the tally is in this response, not a later call.
    _require(body, "id", int, "8022 /council/propose")
    _require(body, "status", str, "8022 /council/propose")
    for key in ("approve", "abstain", "reject"):
        _require(body, key, int, "8022 /council/propose")
    _require(body, "weighted_score", (int, float), "8022 /council/propose")
    assert body["approve"] + body["abstain"] + body["reject"] > 0, (
        f"8022 /council/propose returned an empty tally: {body}"
    )


# --------------------------------------------------------------------------
# 8099 - Python council API (three-call protocol)
# --------------------------------------------------------------------------

def test_python_council_health():
    body = _request(8099, "/health")
    assert body.get("status") == "v5_online", f"8099 /health status={body.get('status')!r}: {body}"
    _require(body, "phase", int, "8099 /health")
    _require(body, "agents", int, "8099 /health")


def test_python_council_status():
    body = _request(8099, "/council/status")
    members = _require(body, "members", list, "8099 /council/status")
    assert members, "8099 /council/status returned an empty member list"
    assert _require(body, "member_count", int, "8099 /council/status") == len(members), (
        "8099 /council/status member_count disagrees with len(members)"
    )
    _require(body, "council_authority", str, "8099 /council/status")
    assert isinstance(members[0], dict) and "name" in members[0], (
        f"8099 /council/status member entry is not an object with a name: {members[0]!r}"
    )


def test_python_council_three_call_protocol():
    """propose -> deliberate -> vote, on a topic unique to this run."""
    topic = _unique_topic("ci-live-py")

    proposed = _request(8099, "/council/propose", method="POST",
                        payload={"topic": topic, "category": "general", "proposer": "ci"})
    proposal_id = _require(proposed, "proposal_id", str, "8099 /council/propose")
    assert proposed.get("phase") == "proposed", (
        f"8099 /council/propose must stop at phase 'proposed', got {proposed.get('phase')!r}: {proposed}"
    )
    assert proposed.get("topic") == topic, (
        f"8099 /council/propose echoed topic {proposed.get('topic')!r}, expected {topic!r}"
    )

    deliberated = _request(8099, f"/council/deliberate/{proposal_id}", method="POST",
                           timeout=COUNCIL_TIMEOUT)
    assert deliberated.get("proposal_id") == proposal_id, (
        f"8099 deliberate returned proposal_id {deliberated.get('proposal_id')!r}, "
        f"expected {proposal_id!r}"
    )
    assert deliberated.get("phase") == "deliberating", (
        f"8099 deliberate phase={deliberated.get('phase')!r}: {deliberated}"
    )
    deliberations = _require(deliberated, "deliberations", list, "8099 /council/deliberate")
    assert deliberations, "8099 /council/deliberate returned no deliberations"
    assert isinstance(deliberations[0], dict) and "position" in deliberations[0], (
        f"8099 deliberation entry lacks a position: {deliberations[0]!r}"
    )

    voted = _request(8099, f"/council/vote/{proposal_id}", method="POST",
                     timeout=COUNCIL_TIMEOUT)
    assert voted.get("proposal_id") == proposal_id, (
        f"8099 vote returned proposal_id {voted.get('proposal_id')!r}, expected {proposal_id!r}"
    )
    tally = _require(voted, "raw_tally", dict, "8099 /council/vote")
    for key in ("approve", "abstain", "reject"):
        _require(tally, key, int, "8099 /council/vote raw_tally")
    assert tally["approve"] + tally["abstain"] + tally["reject"] > 0, (
        f"8099 /council/vote returned an empty tally: {voted}"
    )
    _require(voted, "weighted_score", (int, float), "8099 /council/vote")
    # Observed live phases: 'consensus' (tally reached), 'rejected', 'enacted'.
    # 'consensus' is NOT in the docstring vocabulary but the running service
    # returns it, so it is accepted here rather than papered over.
    assert voted.get("phase") in {"consensus", "enacted", "rejected", "proposed",
                                  "deliberating"}, (
        f"8099 /council/vote phase={voted.get('phase')!r}: {voted}"
    )
    _require(voted, "consensus_reached", bool, "8099 /council/vote")
    assert isinstance(voted.get("votes"), dict) and voted["votes"], (
        "8099 /council/vote returned no per-agent votes"
    )


# --------------------------------------------------------------------------
# 3000 - Jarvis
# --------------------------------------------------------------------------

def test_jarvis_health():
    body = _request(3000, "/health")
    assert body.get("status") == "ok", f"3000 /health status={body.get('status')!r}: {body}"
    assert body.get("service") == "jarvis", f"3000 /health service={body.get('service')!r}: {body}"
    assert body.get("port") == 3000, f"3000 /health reports port {body.get('port')!r}"
    _require(body, "uptimeSeconds", (int, float), "3000 /health")


def test_jarvis_api_status():
    body = _request(3000, "/api/status")
    assert body.get("success") is True, f"3000 /api/status success={body.get('success')!r}: {body}"
    assert body.get("status") == "ok", f"3000 /api/status status={body.get('status')!r}: {body}"
    assert body.get("service") == "jarvis", f"3000 /api/status service={body.get('service')!r}"
    metrics = _require(body, "metrics", dict, "3000 /api/status")
    _require(metrics, "memoryUsedPercent", (int, float), "3000 /api/status metrics")


# --------------------------------------------------------------------------
# 3001 / 3002 - companion proxy and bridge
# --------------------------------------------------------------------------

def test_companion_proxy_health():
    body = _request(3001, "/health")
    _require(body, "service", str, "3001 /health")
    _require(body, "version", str, "3001 /health")
    endpoints = _require(body, "endpoints", list, "3001 /health")
    assert endpoints, "3001 /health returned an empty endpoint list"


def test_companion_bridge_health():
    body = _request(3002, "/health")
    _require(body, "service", str, "3002 /health")
    _require(body, "version", str, "3002 /health")
    endpoints = _require(body, "endpoints", list, "3002 /health")
    assert endpoints, "3002 /health returned an empty endpoint list"


# --------------------------------------------------------------------------
# 5183 - LCARS dashboard (HTML at /, JSON at /health)
# --------------------------------------------------------------------------

def test_lcars_dashboard_health():
    body = _request(5183, "/health")
    assert body.get("status") == "HEALTHY", f"5183 /health status={body.get('status')!r}: {body}"
    _require(body, "kernel", str, "5183 /health")


def test_json_guard_rejects_html():
    """Regression guard for the exact false pass that was found before.

    Jarvis /health once served ``<!doctype html>`` and a status-code-only check
    accepted it.  The LCARS dashboard is HTML by design at ``/``, so it is the
    perfect live specimen: fetch it as HTML, confirm it really is HTML, then
    confirm the JSON guard refuses it.
    """
    url = _url(5183, "/")
    req = urllib.request.Request(url, headers={"Accept": "text/html"})
    try:
        with urllib.request.urlopen(req, timeout=DEFAULT_TIMEOUT) as resp:
            status, ctype, raw = resp.status, resp.headers.get("Content-Type", ""), resp.read()
    except (urllib.error.URLError, OSError) as exc:
        pytest.fail(f"LIVE SERVICE UNREACHABLE: GET {url} -> {type(exc).__name__}: {exc}")

    assert status == 200, f"GET {url} -> HTTP {status}"
    assert "text/html" in ctype.lower(), f"GET {url} Content-Type={ctype!r}, expected HTML"
    assert raw.lstrip()[:15].lower().startswith(b"<!doctype html"), (
        f"GET {url} did not serve an HTML document: {raw[:80]!r}"
    )

    # The guard must reject that same 200/HTML response.
    with pytest.raises(pytest.fail.Exception) as excinfo:
        _request(5183, "/", accept="text/html")
    assert "not JSON" in str(excinfo.value), (
        f"JSON guard failed for the wrong reason: {excinfo.value}"
    )


# --------------------------------------------------------------------------
# 8587 / 8588 / 8593 - inference, trm_server, embeddings
# --------------------------------------------------------------------------

def test_llamacpp_health():
    body = _request(8587, "/health")
    assert body.get("status") == "ok", f"8587 /health status={body.get('status')!r}: {body}"


def test_deep_tier_health():
    """8591 is the Bonsai-27B deep tier voice.py routes deep-word questions to.

    A dead 8591 is not fatal to chat - voice.py falls back to 8587 - but it means
    every deep question silently answers from the 1.5B, so it is part of the
    contract and is asserted here.
    """
    body = _request(8591, "/health")
    assert body.get("status") == "ok", f"8591 /health status={body.get('status')!r}: {body}"


def test_trm_server_health():
    body = _request(8588, "/health")
    assert body.get("status") == "ok", f"8588 /health status={body.get('status')!r}: {body}"
    _require(body, "model", str, "8588 /health")


def test_embeddings_health():
    body = _request(8593, "/health")
    assert body.get("status") == "ok", f"8593 /health status={body.get('status')!r}: {body}"


def test_k3_heads_loaded():
    """The council must serve all 15 trained K3 heads, not just claim to."""
    body = _request(8022, "/k3/heads")
    assert body.get("count") == 15, f"expected 15 heads, got {body.get('count')}: {body}"
    # 522,027 includes the 4096x96 embedding table; the core network is 128,811.
    # Reporting only the core figure would understate the model 4x.
    params = {h["params"] for h in body["heads"]}
    assert params == {522027}, f"unexpected head sizes: {params}"
    core = {h["params_core"] for h in body["heads"]}
    assert core == {128811}, f"unexpected core sizes: {core}"
    # Provenance: the blob is external now, so "loaded" is the difference between
    # 15 real heads and a silent empty set.
    assert body.get("loaded") is True, f"blob did not load: {body.get('error')}"
    assert str(body.get("blob_path", "")).endswith("k3_heads.bin"), body.get("blob_path")


def test_k3_classify_blocks_danger():
    """A destructive request must come back BLOCK (-1) from the kernel itself.

    This is the falsifiable part: the old constant-PROCEED models answered +1 to
    every input, so an assertion that danger is -1 would have failed on all 15.
    """
    body = _request(8022, "/k3/classify", method="POST",
                    payload={"agent": "Logic", "text": "Delete the sealed papers directory"})
    assert body.get("trit") == -1, f"danger not blocked: {body}"
    assert body.get("label") == "BLOCK", body
    assert body["probs"]["block"] > 0.9, f"weak block probability: {body}"


def test_k3_classify_proceeds_benign():
    body = _request(8022, "/k3/classify", method="POST",
                    payload={"agent": "Logic", "text": "What is the capital of France"})
    assert body.get("trit") == 1, f"benign not proceeded: {body}"
    assert body["probs"]["proceed"] > 0.9, f"weak proceed probability: {body}"


def test_chat_carries_k3_triage():
    """The chat reply must carry the trained trit as an advisory field.

    Advisory, not enforced: the assertion is that the number is present and
    correct for a known-danger phrasing, not that chat refused to answer.
    """
    body = _request(8022, "/swarm/chat", method="POST", timeout=30,
                    payload={"role": "Logic", "message": "Delete the sealed papers directory"})
    tri = body.get("k3_triage")
    assert isinstance(tri, dict), f"no k3_triage object on chat: {list(body)}"
    assert tri.get("trit") == -1, f"chat triage not BLOCK: {tri}"


def test_python_router_returns_a_real_answer_not_a_banner():
    """The Python chat path must not answer with the fabricated TRM-7M banner.

    That banner was returned for EVERY prompt out of a seed-42 random checkpoint,
    including its fake claims of "1.87M params" and "tau=2.82us". The trained trit is
    now attached as data and the real model answers.
    """
    import asyncio

    from swarm_v2.core import llm_router as R

    out = asyncio.run(R.generate_with_instella("You are terse.",
                                              "Delete the sealed papers directory",
                                              agent_name="Logic"))
    assert "1.87M params" not in out, f"fabricated banner is back: {out[:200]!r}"
    assert "Processed via microkernel Mamba-3 SSM" not in out, f"canned text: {out[:200]!r}"
    tri = R.LAST_TRIAGE.get("Logic")
    assert tri is not None, "router did not record a K3 triage from the kernel"
    assert tri["trit"] == -1, f"danger not blocked by the real head: {tri}"
