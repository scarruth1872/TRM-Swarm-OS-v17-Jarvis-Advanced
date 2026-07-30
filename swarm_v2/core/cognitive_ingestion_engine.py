"""
cognitive_ingestion_engine.py — Jarvis Cognitive & Learning Matrix Ingestion into TRM
TRM Swarm OS · Project Continuum V5.3

This module forms the critical bidirectional bridge between J.A.R.V.I.S. (Port 4000)
and the TRM Swarm OS (Port 8021). It continuously:

1. PULLS all Jarvis cognitive matrices:
   - Core Memory (personality, profile, preferences)
   - Episodic Memory (conversation events, past decisions)
   - Semantic Graph (knowledge nodes & relations)
   - Skills Registry (active/inactive skills + capabilities)
   - Biomimetic Presets (DNA behavioral vectors)
   - Learn-Resource Cognitive Maps (absorbed external knowledge)
   - Chat History (recent conversation context)
   - Noospheric Calibration (resonance state)

2. TOKENIZES each matrix into TRM-compatible integer token sequences
   via hash-based normalization for symbolic reasoning input.

3. FEEDS tokens into TRMBrain.reason() cycles to generate evolved
   cognitive embeddings that represent Jarvis's current knowledge state.

4. COMMITS all ingested data to GlobalMemory with tagged provenance
   ("jarvis_cognitive_ingest") so every Swarm expert agent can query
   Jarvis's knowledge base through the shared vector store.

5. MAINTAINS a persistent SystemEventLedger — a rolling JSON log of
   every significant system event (API calls, agent decisions, ingestion
   cycles, healing events, consensus votes) so the system always knows
   what has occurred.

6. RUNS as a background daemon via asyncio on a 60-second interval.
"""

import os
import json
import math
import time
import hashlib
import asyncio
import logging
import datetime
import urllib.request
import urllib.error
from typing import Dict, List, Any, Optional, Tuple

logger = logging.getLogger("CognitiveIngestionEngine")

# ─────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────

JARVIS_BASE_URL = os.getenv("JARVIS_URL", "http://127.0.0.1:4000")
SWARM_BASE_DIR = os.getenv("SWARM_BASE_DIR", ".")
EVENT_LEDGER_PATH = os.path.join(SWARM_BASE_DIR, "swarm_v2_global_memory", "system_event_ledger.jsonl")
COGNITIVE_SNAPSHOT_PATH = os.path.join(SWARM_BASE_DIR, "swarm_v2_global_memory", "jarvis_cognitive_snapshot.json")
INGESTION_INTERVAL_SECONDS = 60  # Full ingestion cycle every 60s
MAX_LEDGER_ENTRIES = 10000       # Rolling window


# ─────────────────────────────────────────────────────────────
# System Event Ledger — Persistent Event Log
# ─────────────────────────────────────────────────────────────

class SystemEventLedger:
    """
    A persistent, append-only JSON Lines ledger tracking every significant
    system event. Gives the system total situational awareness at all times.
    """

    CATEGORIES = {
        "INGESTION",       # Jarvis cognitive matrix ingestion cycles
        "API_CALL",        # Inbound REST API calls to Swarm OS
        "AGENT_DECISION",  # Expert agent reasoning outputs
        "CONSENSUS",       # PBFT / heuristic consensus events
        "HEALING",         # Self-healing remediation events
        "SYMBIOSIS",       # Jarvis↔Swarm skill synthesis events
        "TRM_REASONING",   # TRM brain recursive reasoning outputs
        "GENOMICS",        # DNA gene vector mutations / preset saves
        "STAR_MATRIX",     # Celestial node registrations / edge connections
        "MEMORY",          # GlobalMemory contributions / queries
        "SYSTEM",          # Startup, shutdown, error, health events
    }

    def __init__(self, path: str = EVENT_LEDGER_PATH):
        self.path = path
        os.makedirs(os.path.dirname(path), exist_ok=True)
        self._lock = asyncio.Lock() if asyncio.get_event_loop().is_running() else None

    def record(
        self,
        category: str,
        event: str,
        data: Optional[Dict] = None,
        source: str = "system",
        severity: str = "INFO"
    ) -> Dict:
        """Append an event to the ledger (synchronous, thread-safe via file append)."""
        entry = {
            "ts": datetime.datetime.utcnow().isoformat() + "Z",
            "category": category,
            "event": event,
            "source": source,
            "severity": severity,
            "data": data or {}
        }
        try:
            with open(self.path, "a", encoding="utf-8") as f:
                f.write(json.dumps(entry) + "\n")
        except Exception as e:
            logger.warning(f"[Ledger] Failed to write event: {e}")
        return entry

    def tail(self, n: int = 100, category: Optional[str] = None) -> List[Dict]:
        """Read the last N entries from the ledger, optionally filtered by category."""
        entries = []
        try:
            with open(self.path, "r", encoding="utf-8") as f:
                lines = f.readlines()
            for line in reversed(lines):
                line = line.strip()
                if not line:
                    continue
                try:
                    entry = json.loads(line)
                    if category and entry.get("category") != category:
                        continue
                    entries.append(entry)
                    if len(entries) >= n:
                        break
                except json.JSONDecodeError:
                    continue
        except FileNotFoundError:
            pass
        return list(reversed(entries))

    def get_stats(self) -> Dict:
        """Return summary statistics across all ledger entries."""
        stats: Dict[str, int] = {}
        total = 0
        try:
            with open(self.path, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        entry = json.loads(line)
                        cat = entry.get("category", "UNKNOWN")
                        stats[cat] = stats.get(cat, 0) + 1
                        total += 1
                    except json.JSONDecodeError:
                        continue
        except FileNotFoundError:
            pass
        return {"total": total, "by_category": stats}

    def prune(self, keep_last: int = MAX_LEDGER_ENTRIES):
        """Prune ledger to the most recent N entries."""
        try:
            with open(self.path, "r", encoding="utf-8") as f:
                lines = [l for l in f.readlines() if l.strip()]
            if len(lines) > keep_last:
                with open(self.path, "w", encoding="utf-8") as f:
                    f.writelines(lines[-keep_last:])
                logger.info(f"[Ledger] Pruned to {keep_last} entries.")
        except FileNotFoundError:
            pass


# ─────────────────────────────────────────────────────────────
# Jarvis Cognitive Matrix Fetcher
# ─────────────────────────────────────────────────────────────

class JarvisCognitiveFetcher:
    """
    Pulls all cognitive matrices from the live Jarvis instance at Port 4000.
    Uses urllib (stdlib only) for zero-dependency HTTP fetches.
    """

    def __init__(self, base_url: str = JARVIS_BASE_URL):
        self.base_url = base_url

    def _get(self, path: str, timeout: int = 6) -> Optional[Any]:
        """Perform a GET request to Jarvis and return parsed JSON or None."""
        try:
            req = urllib.request.Request(
                f"{self.base_url}{path}",
                headers={"User-Agent": "TRM-SwarmOS-CognitiveIngestion/1.0"}
            )
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                raw = resp.read().decode("utf-8")
                return json.loads(raw)
        except urllib.error.URLError as e:
            logger.warning(f"[Fetcher] GET {path} — connection error: {e.reason}")
            return None
        except Exception as e:
            logger.warning(f"[Fetcher] GET {path} — error: {e}")
            return None

    def fetch_memory(self) -> Optional[Dict]:
        """Fetch core memory, episodic memory, semantic graph."""
        return self._get("/api/memory")

    def fetch_skills(self) -> Optional[List]:
        """Fetch active skills registry."""
        data = self._get("/api/skills")
        if data and isinstance(data, dict):
            return data.get("skills", [])
        return data if isinstance(data, list) else None

    def fetch_skills_matrix(self) -> Optional[Dict]:
        """Fetch the full skills matrix topology."""
        return self._get("/api/skills/matrix")

    def fetch_biomimetic_presets(self) -> Optional[Dict]:
        """Fetch biomimetic behavioral DNA presets."""
        data = self._get("/api/biomimetic/presets")
        if data and isinstance(data, dict):
            return data.get("presets")
        return None

    def fetch_chat_history(self) -> Optional[List]:
        """Fetch recent chat conversation history."""
        data = self._get("/api/chat-history")
        if data and isinstance(data, dict):
            return data.get("history", data.get("messages", []))
        return data if isinstance(data, list) else None

    def fetch_noospheric_status(self) -> Optional[Dict]:
        """Fetch noospheric calibration/resonance state."""
        data = self._get("/api/noospheric/status")
        if data and isinstance(data, dict):
            return data.get("status", data)
        return None

    def fetch_all(self) -> Dict[str, Any]:
        """
        Pull ALL Jarvis cognitive matrices in a single consolidated snapshot.
        Returns a dict keyed by matrix name. Missing matrices are None.
        """
        snapshot = {
            "fetched_at": datetime.datetime.utcnow().isoformat() + "Z",
            "jarvis_url": self.base_url,
            "memory": self.fetch_memory(),
            "skills": self.fetch_skills(),
            "skills_matrix": self.fetch_skills_matrix(),
            "biomimetic_presets": self.fetch_biomimetic_presets(),
            "chat_history": self.fetch_chat_history(),
            "noospheric_status": self.fetch_noospheric_status(),
        }

        # Count non-null matrices
        matrices_loaded = sum(1 for v in snapshot.values() if v is not None and not isinstance(v, str))
        snapshot["matrices_loaded"] = matrices_loaded

        return snapshot


# ─────────────────────────────────────────────────────────────
# Cognitive Matrix → TRM Token Encoder
# ─────────────────────────────────────────────────────────────

class CognitiveTokenEncoder:
    """
    Converts Jarvis cognitive matrices into integer token sequences
    compatible with TRMBrain.reason(). Uses a deterministic hash-based
    encoding so the same knowledge always produces the same tokens.

    Token space: vocab_size=12 (matching TRM config)
    Sequence length: up to 512 tokens per matrix
    """

    VOCAB_SIZE = 12
    MAX_SEQ_LEN = 512

    def encode_string(self, text: str) -> List[int]:
        """Encode arbitrary text as a reproducible token sequence."""
        tokens = []
        words = str(text).split()
        for word in words[:self.MAX_SEQ_LEN]:
            # SHA-256 hash mod vocab_size for stable token assignment
            h = int(hashlib.sha256(word.lower().encode()).hexdigest(), 16)
            tokens.append(h % self.VOCAB_SIZE)
        # Pad to at least 8 tokens
        while len(tokens) < 8:
            tokens.append(0)
        return tokens[:self.MAX_SEQ_LEN]

    def encode_dict(self, d: Dict, depth: int = 0) -> List[int]:
        """Recursively encode a dict into tokens."""
        tokens = []
        for k, v in list(d.items())[:20]:  # limit breadth
            tokens += self.encode_string(str(k))
            if isinstance(v, dict) and depth < 2:
                tokens += self.encode_dict(v, depth + 1)
            elif isinstance(v, list):
                for item in v[:5]:
                    tokens += self.encode_string(str(item))
            else:
                tokens += self.encode_string(str(v))
            if len(tokens) >= self.MAX_SEQ_LEN:
                break
        return tokens[:self.MAX_SEQ_LEN]

    def encode_list(self, lst: List, max_items: int = 20) -> List[int]:
        """Encode a list of items into tokens."""
        tokens = []
        for item in lst[:max_items]:
            if isinstance(item, dict):
                tokens += self.encode_dict(item)
            else:
                tokens += self.encode_string(str(item))
            if len(tokens) >= self.MAX_SEQ_LEN:
                break
        return tokens[:self.MAX_SEQ_LEN]

    def encode_cognitive_gene(self, gene_vec: List[float]) -> List[int]:
        """
        Encode a 4D cognitive gene vector [ρ, δ, ε, σ] into tokens.
        Quantizes each float into vocab_size bins.
        """
        tokens = []
        for val in gene_vec:
            # Quantize float [0,1] into vocab_size bins
            token = min(int(float(val) * self.VOCAB_SIZE), self.VOCAB_SIZE - 1)
            tokens.append(token)
        return tokens

    def snapshot_to_token_sequences(self, snapshot: Dict) -> Dict[str, List[int]]:
        """
        Convert all cognitive matrices in a snapshot to named token sequences.
        Returns dict mapping matrix_name → token_list.
        """
        sequences = {}

        # Memory matrix
        memory = snapshot.get("memory")
        if memory:
            core = memory.get("core", {})
            episodic = memory.get("episodic", [])
            nodes = memory.get("nodes", [])
            relations = memory.get("relations", [])

            sequences["memory_core"] = self.encode_dict(core)
            sequences["memory_episodic"] = self.encode_list(episodic)
            sequences["memory_nodes"] = self.encode_list(nodes)
            sequences["memory_relations"] = self.encode_list(relations)

        # Skills matrix
        skills = snapshot.get("skills")
        if skills:
            sequences["skills_registry"] = self.encode_list(skills)

        skills_matrix = snapshot.get("skills_matrix")
        if skills_matrix and isinstance(skills_matrix, dict):
            sequences["skills_topology"] = self.encode_dict(skills_matrix)

        # Biomimetic presets (DNA behavioral vectors)
        presets = snapshot.get("biomimetic_presets")
        if presets:
            if isinstance(presets, dict):
                # Encode each preset's gene vector
                for preset_name, preset_data in list(presets.items())[:10]:
                    if isinstance(preset_data, dict):
                        gene = [
                            float(preset_data.get("plasticity", 0.5)),
                            float(preset_data.get("logical_depth", 0.5)),
                            float(preset_data.get("empathy", 0.5)),
                            float(preset_data.get("stochasticity", 0.5)),
                        ]
                        sequences[f"preset_{preset_name}"] = self.encode_cognitive_gene(gene)
            elif isinstance(presets, list):
                sequences["biomimetic_presets"] = self.encode_list(presets)

        # Chat history (recent context)
        chat = snapshot.get("chat_history")
        if chat:
            sequences["chat_history"] = self.encode_list(chat[-20:] if len(chat) > 20 else chat)

        # Noospheric status
        noospheric = snapshot.get("noospheric_status")
        if noospheric:
            sequences["noospheric_state"] = self.encode_dict(noospheric) if isinstance(noospheric, dict) else self.encode_string(str(noospheric))

        return sequences


# ─────────────────────────────────────────────────────────────
# TRM Cognitive Ingestion — Main Engine
# ─────────────────────────────────────────────────────────────

class CognitiveIngestionEngine:
    """
    The primary bridge engine. Runs as an asyncio daemon that:
    1. Fetches Jarvis cognitive matrices on every cycle
    2. Tokenizes them for TRM symbolic reasoning
    3. Runs TRMBrain.reason() on each sequence
    4. Commits all knowledge to GlobalMemory with full provenance
    5. Persists the snapshot as jarvis_cognitive_snapshot.json
    6. Records every step in the SystemEventLedger
    """

    def __init__(self):
        self.fetcher = JarvisCognitiveFetcher()
        self.encoder = CognitiveTokenEncoder()
        self.ledger = get_event_ledger()
        self._running = False
        self._cycle_count = 0
        self._last_snapshot: Optional[Dict] = None
        self._last_ingestion_ts: Optional[str] = None

        # Try to load TRM brain (non-blocking — it's optional for symbolic reasoning)
        self._trm_brain = None
        try:
            from swarm_v2.core.trm_brain import get_trm_brain
            self._trm_brain = get_trm_brain()
            logger.info("[CognitiveIngestion] TRM Brain loaded for symbolic token reasoning.")
        except Exception as e:
            logger.warning(f"[CognitiveIngestion] TRM Brain unavailable (will skip symbolic reasoning): {e}")

        self.ledger.record("SYSTEM", "CognitiveIngestionEngine initialized", source="cognitive_ingestion")

    def _run_trm_reasoning(self, tokens: List[int], puzzle_id: int = 42) -> Optional[List[int]]:
        """Run TRMBrain.reason() on a token sequence. Returns output tokens or None."""
        if self._trm_brain is None or not tokens:
            return None
        try:
            return self._trm_brain.reason(tokens, puzzle_id=puzzle_id)
        except Exception as e:
            logger.debug(f"[CognitiveIngestion] TRM reasoning skipped: {e}")
            return None

    def _commit_to_global_memory(self, matrix_name: str, content: str, tags: List[str]):
        """Commit ingested knowledge to GlobalMemory."""
        try:
            from swarm_v2.core.global_memory import get_global_memory
            mem = get_global_memory()
            mem.contribute(
                content=content,
                author="JarvisCognitiveIngestion",
                author_role="cognitive_bridge",
                memory_type="experience",
                tags=["jarvis_cognitive_ingest", "trm_bridge"] + tags
            )
        except Exception as e:
            logger.warning(f"[CognitiveIngestion] GlobalMemory commit failed: {e}")

    def _save_snapshot(self, snapshot: Dict):
        """Save the latest Jarvis cognitive snapshot to disk."""
        try:
            os.makedirs(os.path.dirname(COGNITIVE_SNAPSHOT_PATH), exist_ok=True)
            with open(COGNITIVE_SNAPSHOT_PATH, "w", encoding="utf-8") as f:
                json.dump(snapshot, f, indent=2, default=str)
        except Exception as e:
            logger.warning(f"[CognitiveIngestion] Snapshot save failed: {e}")

    def run_ingestion_cycle(self) -> Dict[str, Any]:
        """
        Execute one full cognitive ingestion cycle synchronously.
        Returns a status report dict.
        """
        cycle_start = time.time()
        self._cycle_count += 1
        ts = datetime.datetime.utcnow().isoformat() + "Z"

        self.ledger.record(
            "INGESTION",
            f"Jarvis cognitive ingestion cycle #{self._cycle_count} started",
            {"cycle": self._cycle_count, "ts": ts},
            source="cognitive_ingestion"
        )

        # ── Step 1: Fetch all Jarvis cognitive matrices ──────────────────
        logger.info(f"[CognitiveIngestion] Cycle #{self._cycle_count}: Fetching Jarvis cognitive matrices...")
        snapshot = self.fetcher.fetch_all()
        matrices_loaded = snapshot.get("matrices_loaded", 0)

        if matrices_loaded == 0:
            self.ledger.record(
                "INGESTION",
                "Jarvis offline — no cognitive matrices fetched",
                {"cycle": self._cycle_count},
                source="cognitive_ingestion",
                severity="WARNING"
            )
            return {"status": "JARVIS_OFFLINE", "cycle": self._cycle_count, "matrices_loaded": 0}

        logger.info(f"[CognitiveIngestion] Fetched {matrices_loaded} cognitive matrices from Jarvis.")

        # ── Step 2: Tokenize all matrices ────────────────────────────────
        token_sequences = self.encoder.snapshot_to_token_sequences(snapshot)
        logger.info(f"[CognitiveIngestion] Encoded {len(token_sequences)} token sequences for TRM.")

        # ── Step 3: TRM symbolic reasoning on each sequence ─────────────
        trm_outputs = {}
        for seq_name, tokens in token_sequences.items():
            if tokens:
                puzzle_id = abs(hash(seq_name)) % 876406
                output = self._run_trm_reasoning(tokens, puzzle_id=puzzle_id)
                if output:
                    trm_outputs[seq_name] = output[:16]  # Store first 16 output tokens

        # ── Step 4: Commit to GlobalMemory ──────────────────────────────
        committed = 0

        # Memory matrices
        memory = snapshot.get("memory")
        if memory:
            core = memory.get("core", {})
            if core:
                profile_summary = f"Jarvis Core Memory Profile — User: {core.get('userName', 'Boss')}, Tone: {core.get('preferredTone', 'professional')}, Goals: {str(core.get('activeGoals', []))[:200]}"
                self._commit_to_global_memory(
                    "memory_core",
                    profile_summary,
                    ["jarvis_memory", "core_profile"]
                )
                committed += 1

            episodic = memory.get("episodic", [])
            if episodic:
                recent = episodic[-5:] if len(episodic) > 5 else episodic
                episodic_summary = f"Jarvis Episodic Memory — {len(episodic)} events. Recent: {json.dumps(recent, default=str)[:400]}"
                self._commit_to_global_memory(
                    "memory_episodic",
                    episodic_summary,
                    ["jarvis_memory", "episodic"]
                )
                committed += 1

            nodes = memory.get("nodes", [])
            if nodes:
                node_names = [n.get("label", n.get("id", "?")) for n in nodes[:20]]
                knowledge_summary = f"Jarvis Semantic Knowledge Graph — {len(nodes)} nodes. Key concepts: {', '.join(node_names)}"
                self._commit_to_global_memory(
                    "memory_semantic_graph",
                    knowledge_summary,
                    ["jarvis_memory", "semantic_graph", "knowledge_nodes"]
                )
                committed += 1

        # Skills matrices
        skills = snapshot.get("skills")
        if skills:
            active_skills = [s.get("name", s.get("id", "?")) for s in skills if s.get("enabled", True)]
            skills_summary = f"Jarvis Active Skills Registry — {len(active_skills)} enabled: {', '.join(active_skills[:15])}"
            self._commit_to_global_memory(
                "skills_registry",
                skills_summary,
                ["jarvis_skills", "skill_registry"]
            )
            committed += 1

        # Biomimetic presets (DNA cognitive gene vectors)
        presets = snapshot.get("biomimetic_presets")
        if presets:
            presets_summary = f"Jarvis Biomimetic DNA Presets — {len(presets) if hasattr(presets, '__len__') else 1} preset configurations loaded. TRM token embeddings: {list(trm_outputs.keys())[:5]}"
            self._commit_to_global_memory(
                "biomimetic_presets",
                presets_summary,
                ["jarvis_dna", "cognitive_genes", "biomimetic"]
            )
            committed += 1

        # TRM reasoning outputs
        if trm_outputs:
            trm_summary = (
                f"TRM Symbolic Reasoning Outputs for {len(trm_outputs)} Jarvis cognitive sequences. "
                f"Sequences processed: {list(trm_outputs.keys())}. "
                f"Sample output tokens (memory_core): {trm_outputs.get('memory_core', [])[:8]}"
            )
            self._commit_to_global_memory(
                "trm_reasoning_outputs",
                trm_summary,
                ["trm_reasoning", "symbolic_output", "jarvis_trm_bridge"]
            )
            committed += 1

        # Noospheric state
        noospheric = snapshot.get("noospheric_status")
        if noospheric:
            noospheric_summary = f"Jarvis Noospheric Calibration State: {json.dumps(noospheric, default=str)[:300]}"
            self._commit_to_global_memory(
                "noospheric_state",
                noospheric_summary,
                ["jarvis_noospheric", "resonance_state"]
            )
            committed += 1

        # ── Step 5: Persist snapshot to disk ────────────────────────────
        snapshot["trm_reasoning_outputs"] = trm_outputs
        snapshot["token_sequence_count"] = len(token_sequences)
        snapshot["global_memory_commits"] = committed
        self._save_snapshot(snapshot)
        self._last_snapshot = snapshot
        self._last_ingestion_ts = ts

        # ── Step 6: Record completion to ledger ──────────────────────────
        elapsed = round(time.time() - cycle_start, 3)
        report = {
            "status": "COMPLETE",
            "cycle": self._cycle_count,
            "ts": ts,
            "elapsed_s": elapsed,
            "matrices_loaded": matrices_loaded,
            "token_sequences": len(token_sequences),
            "trm_outputs": len(trm_outputs),
            "global_memory_commits": committed
        }

        self.ledger.record(
            "INGESTION",
            f"Jarvis cognitive ingestion cycle #{self._cycle_count} complete",
            report,
            source="cognitive_ingestion"
        )

        # Prune ledger periodically
        if self._cycle_count % 50 == 0:
            self.ledger.prune()

        logger.info(
            f"[CognitiveIngestion] Cycle #{self._cycle_count} done in {elapsed}s | "
            f"{matrices_loaded} matrices | {len(token_sequences)} sequences | "
            f"{len(trm_outputs)} TRM outputs | {committed} mem commits"
        )

        return report

    async def run_daemon(self, interval: int = INGESTION_INTERVAL_SECONDS):
        """
        Asyncio background daemon. Runs a full ingestion cycle every `interval` seconds.
        Designed to be launched via asyncio.create_task() during app lifespan startup.
        """
        self._running = True
        logger.info(f"[CognitiveIngestion] Daemon started. Ingestion interval: {interval}s")
        self.ledger.record(
            "SYSTEM",
            "CognitiveIngestionEngine daemon started",
            {"interval_s": interval},
            source="cognitive_ingestion"
        )

        # Run first cycle immediately on startup
        try:
            loop = asyncio.get_event_loop()
            await loop.run_in_executor(None, self.run_ingestion_cycle)
        except Exception as e:
            logger.error(f"[CognitiveIngestion] Initial cycle failed: {e}")
            self.ledger.record("SYSTEM", f"Initial ingestion cycle error: {e}", severity="ERROR", source="cognitive_ingestion")

        while self._running:
            await asyncio.sleep(interval)
            if not self._running:
                break
            try:
                loop = asyncio.get_event_loop()
                await loop.run_in_executor(None, self.run_ingestion_cycle)
            except Exception as e:
                logger.error(f"[CognitiveIngestion] Cycle error: {e}")
                self.ledger.record(
                    "SYSTEM",
                    f"Ingestion cycle #{self._cycle_count} error: {e}",
                    severity="ERROR",
                    source="cognitive_ingestion"
                )

    def stop(self):
        """Stop the daemon."""
        self._running = False
        self.ledger.record("SYSTEM", "CognitiveIngestionEngine daemon stopped", source="cognitive_ingestion")
        logger.info("[CognitiveIngestion] Daemon stopped.")

    def get_status(self) -> Dict[str, Any]:
        """Return current engine status."""
        return {
            "running": self._running,
            "cycle_count": self._cycle_count,
            "last_ingestion_ts": self._last_ingestion_ts,
            "jarvis_url": JARVIS_BASE_URL,
            "trm_brain_loaded": self._trm_brain is not None,
            "snapshot_path": COGNITIVE_SNAPSHOT_PATH,
            "ledger_path": EVENT_LEDGER_PATH,
            "ledger_stats": self.ledger.get_stats()
        }

    def get_latest_snapshot(self) -> Optional[Dict]:
        """Return the most recent cognitive snapshot."""
        if self._last_snapshot:
            return self._last_snapshot
        # Try to load from disk
        try:
            with open(COGNITIVE_SNAPSHOT_PATH, "r", encoding="utf-8") as f:
                return json.load(f)
        except FileNotFoundError:
            return None


# ─────────────────────────────────────────────────────────────
# Singleton Accessors
# ─────────────────────────────────────────────────────────────

_event_ledger: Optional[SystemEventLedger] = None
_ingestion_engine: Optional[CognitiveIngestionEngine] = None


def get_event_ledger() -> SystemEventLedger:
    global _event_ledger
    if _event_ledger is None:
        _event_ledger = SystemEventLedger()
    return _event_ledger


def get_ingestion_engine() -> CognitiveIngestionEngine:
    global _ingestion_engine
    if _ingestion_engine is None:
        _ingestion_engine = CognitiveIngestionEngine()
    return _ingestion_engine
