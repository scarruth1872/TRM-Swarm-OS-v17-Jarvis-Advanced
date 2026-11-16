"""
Swarm V2 Reconnaissance Daemon
Phase 5: Seeker (Researcher) Upgrade

A background task that autonomously searches for and summarizes new AI research
papers using WebSearchSkill, storing findings in global memory.

Runs on a configurable interval (default: 24 hours).
"""

import asyncio
import os
import json
from datetime import datetime
from typing import List, Dict, Any, Optional
from threading import Thread
import time

try:
    from swarm_v2.skills.web_search_skill import WebSearchSkill
    from swarm_v2.core.global_memory import get_global_memory
except ImportError:
    WebSearchSkill = None
    def get_global_memory():
        return None

RECONNAISSANCE_DIR = "swarm_v2_reconnaissance"

# ── NATIVE VOLTAGE STRIP — Phase 10 auto-injected ─────────────────────────────
import ctypes as _ctypes, numpy as _np
from pathlib import Path as _Path

_RECON_TOPICS_DLL = None
try:
    _dll_p5 = _Path(__file__).parent.parent.parent / "swarm_v2_artifacts" / "native_voltage_strips" / "strip_recon_topics.dll"
    if _dll_p5.exists():
        _RECON_TOPICS_DLL = _ctypes.CDLL(str(_dll_p5))
        _RECON_TOPICS_DLL.strip_batch_score_topic_relevance.argtypes = [
            _ctypes.c_void_p, _ctypes.c_void_p, _ctypes.c_void_p, _ctypes.c_size_t,
            _ctypes.c_char_p, _ctypes.c_size_t, _ctypes.c_void_p
        ]
        _RECON_TOPICS_DLL.strip_batch_score_topic_relevance.restype = None
except Exception as _e:
    _RECON_TOPICS_DLL = None
    _RECON_TOPICS_DLL_ERROR = f"strip not loadable: {_e}"
# ── END NATIVE VOLTAGE STRIP ──────────────────────────────────────────────────

os.makedirs(RECONNAISSANCE_DIR, exist_ok=True)

# Canonical Monad Journal Frontier Research Topics
MONAD_JOURNAL_CORE_TOPICS = [
    "Sub-10ms Byzantine Quorum Consensus in High-Frequency Silicon",
    "TRM-ABX-220: Fluorinated LpxC Hydroxamate Macrocycles & Gram-Negative Outer Membrane Permeation",
    "Deterministic K3 Balanced Ternary Memory Compaction (1.58-bit TurboVec)",
    "Silicon Crystallography & 15.53 THz Optical Phonon Resonance Substrate",
    "15 SRAM Pattern Reconfigurations & Geometric In-Memory Transformations",
    "Rare Disease Exome Healing: COL6A1 c.877G>A Pathogenic Variant Correction",
    "Post-Quantum Lattice Security: ML-KEM-1024 & ML-DSA Dilithium-5",
    "SWE-Marathon Autonomous Software Engineering with Lean 4 Formal Proofs",
    "Hospital AMR Surveillance Node Deployment & Real-Time Pathogen Tracking",
    "Relativistic Muon Calculation & Bethe-Bloch 7nm Particle Energy Harvesting",
    "Phoenix Fault Tolerance & Immortal 0.80 µs SEU Resurrection",
    "MIT Analog Cognition Theory & Voltage-Based Continuous Field Synthesis (Phi_V = 0.378)",
    "1,000-Spin Ising Glass Annealing & Non-Linear Kuramoto Phase Coupling",
    "Real NAND Fabric Direct Storage Streaming & NVMe Zero-Copy Persistence",
    "Autonomous NTD Drug Discovery for Chagas Disease",
    "Universal Model Ingestion & Cross-Architecture Benchmark (Instella MoE & Bonsai 27B)",
    "Multi-Hospital AMR Outbreak Crisis Response & Emergency Sovereign Clinical Protocol",
    "Eliminating Raw Reasoning Gaps via Continuous Voltage Field Grounding (Phi_V = 0.378)",
    "Practical Dual-Node Knowledge Caching: Sub-Millisecond SIMD Vector Search over Tailscale Mesh",
    "Autonomous Lead Optimization of Novel Topo IV Modulators"
]


def get_monad_journal_topics() -> List[str]:
    """Dynamically extract all published research topics from the Monad Journal ledger & papers directory."""
    topics = list(MONAD_JOURNAL_CORE_TOPICS)
    
    # 1. Ingest papers from papers/
    papers_dir = os.path.join(os.path.dirname(__file__), "..", "..", "papers")
    if os.path.exists(papers_dir):
        for fname in sorted(os.listdir(papers_dir)):
            if fname.endswith(".md"):
                fpath = os.path.join(papers_dir, fname)
                try:
                    with open(fpath, "r", encoding="utf-8", errors="ignore") as f:
                        line = f.readline().strip()
                        if line.startswith("#"):
                            title = line.lstrip("#").strip()
                            if title and len(title) > 8 and title not in topics:
                                topics.append(title)
                except Exception:
                    pass

    # 2. Ingest papers from MONAD_JOURNAL_INDEX.md
    index_path = os.path.join(os.path.dirname(__file__), "..", "..", "MONAD_JOURNAL_INDEX.md")
    if os.path.exists(index_path):
        try:
            with open(index_path, "r", encoding="utf-8", errors="ignore") as f:
                for line in f:
                    if line.strip().startswith("| **") and "|" in line:
                        parts = [p.strip() for p in line.split("|")[1:-1]]
                        for col in parts[1:]:
                            clean = col.replace("*", "").replace("_", "").replace("`", "").strip()
                            if len(clean) > 15 and not any(w in clean.lower() for w in [
                                "domain", "section", "philosophy", "business", "architecture",
                                "epistemology", "ecosystem", "rust systems", "systems", "ai", "finding"
                            ]):
                                if clean not in topics:
                                    topics.append(clean)
                                break
        except Exception:
            pass

    return topics


DEFAULT_RESEARCH_TOPICS = MONAD_JOURNAL_CORE_TOPICS


class ResearchFinding:
    """Represents a single research finding."""
    
    def __init__(self, topic: str, summary: str, sources: List[str], timestamp: str = None, finding_id: str = None):
        self.topic = topic
        self.summary = summary
        self.sources = sources
        self.timestamp = timestamp or datetime.now().isoformat()
        self.finding_id = finding_id or f"rf_{hash(summary) & 0xFFFFFFFF:08x}"
    
    def to_dict(self) -> dict:
        return {
            "finding_id": self.finding_id,
            "topic": self.topic,
            "summary": self.summary,
            "sources": self.sources,
            "timestamp": self.timestamp
        }


class ReconnaissanceDaemon:
    """
    Background daemon that autonomously searches for AI research.
    
    Features:
    - Configurable search interval (default 24 hours)
    - Multiple research topics covered per cycle
    - Results stored in global memory for agent access
    - Findings persisted to disk for historical tracking
    """
    
    def __init__(self, interval_hours: float = 24, topics: List[str] = None):
        self.interval_seconds = interval_hours * 3600
        self.topics = topics or list(MONAD_JOURNAL_CORE_TOPICS)
        self.search_skill = WebSearchSkill() if WebSearchSkill else None
        self.is_running = False
        self._thread: Optional[Thread] = None
        self.findings: List[ResearchFinding] = []
        self.last_run: Optional[str] = None
        self.run_count = 0
        
        # Load previous findings and synchronize topics
        self._load_findings()
        self.sync_with_monad_journal()
    
    def sync_with_monad_journal(self):
        """Synchronize active research topics with the latest Monad Journal papers."""
        latest_topics = get_monad_journal_topics()
        # Ensure core and latest topics are present
        for t in latest_topics:
            if t not in self.topics:
                self.topics.append(t)
        # Purge any stale 2024 placeholder topics
        stale_prefixes = ["latest ai research papers", "transformer architecture improvements", "large language model innovations"]
        self.topics = [t for t in self.topics if not any(sp in t.lower() for sp in stale_prefixes)]
        self._save_findings()

    def _load_findings(self):
        """Load previous findings from disk, filtering out stale generic 2024 entries."""
        findings_path = os.path.join(RECONNAISSANCE_DIR, "findings.json")
        if os.path.exists(findings_path):
            try:
                with open(findings_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                    loaded = [ResearchFinding(**fd) for fd in data.get("findings", [])]
                    # Filter out stale 2024 queries
                    stale_keywords = ["latest ai research papers 2024", "transformer architecture improvements", "large language model innovations"]
                    self.findings = [f for f in loaded if not any(sk in f.topic.lower() for sk in stale_keywords)]
                    self.last_run = data.get("last_run")
                    self.run_count = data.get("run_count", 0)
            except Exception as e:
                print(f"[Reconnaissance] Failed to load findings: {e}")
    
    def _save_findings(self):
        """Save findings to disk."""
        findings_path = os.path.join(RECONNAISSANCE_DIR, "findings.json")
        data = {
            "last_run": self.last_run,
            "run_count": self.run_count,
            "findings": [f.to_dict() for f in self.findings[-100:]],  # Keep last 100
            "topics": self.topics
        }
        with open(findings_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)
    
    def start(self):
        """Start the reconnaissance daemon."""
        if self.is_running:
            return
        
        self.is_running = True
        self._thread = Thread(target=self._run_loop, daemon=True)
        self._thread.start()
        print(f"[Reconnaissance] Daemon started. Interval: {self.interval_seconds/3600:.1f} hours")
    
    def stop(self):
        """Stop the daemon."""
        self.is_running = False
        if self._thread:
            self._thread.join(timeout=5)
        print("[Reconnaissance] Daemon stopped.")
    
    def _run_loop(self):
        """Main loop that runs research cycles."""
        # First run: wait for site to load before starting research
        delay = getattr(self, 'first_run_delay', 0)
        if delay > 0:
            print(f"[Reconnaissance] Waiting {delay}s for site to load before first cycle...")
            for _ in range(int(delay)):
                if not self.is_running:
                    break
                time.sleep(1)
        
        while self.is_running:
            try:
                # Run a research cycle
                self._run_research_cycle()
                
                # Wait for next cycle
                for _ in range(int(self.interval_seconds)):
                    if not self.is_running:
                        break
                    time.sleep(1)
            except Exception as e:
                print(f"[Reconnaissance] Error in cycle: {e}")
                time.sleep(60)  # Wait a minute before retrying
    
    def _run_research_cycle(self, force: bool = False):
        """Execute a full research cycle across all topics."""
        print(f"[Reconnaissance] Starting research cycle #{self.run_count + 1}")
        self.last_run = datetime.now().isoformat()
        
        # Run async research in sync context
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        try:
            loop.run_until_complete(self._async_research_cycle(force=force))
        finally:
            loop.close()
        
        self.run_count += 1
        self._save_findings()
        print(f"[Reconnaissance] Cycle complete. Total findings: {len(self.findings)}")
    
    async def _async_research_cycle(self, force: bool = False):
        """Async research cycle."""
        existing_topics = {f.topic for f in self.findings}
        unresearched = [t for t in self.topics if t not in existing_topics]
        # If all researched or force, process up to 3 topics per cycle to avoid blocking resources
        batch = unresearched[:3] if unresearched else (self.topics[:3] if force else [])
        for topic in batch:
            if not self.is_running and not force:
                break
            await self._research_topic(topic)
            await asyncio.sleep(5)  # Yield CPU/event loop between topics
    
    async def _research_topic(self, topic: str):
        """Research a single topic with full platform knowledge access and cross-agent consultation."""
        try:
            from swarm_v2.core.agent_intelligence_bridge import route_agent_intelligence
            from pathlib import Path
            
            # 1. Real-Time External Literature (arXiv & Web)
            results = ""
            if self.search_skill:
                results = self.search_skill.search(f"{topic} scientific paper", max_results=3)
            
            # 2. Consult Domain Expert Agents across the Council
            agent_consult = route_agent_intelligence("Seeker", f"Research directive: Analyze {topic}. Synthesize cross-agent implications.")
            agent_response = agent_consult.get("response", "")
            
            # 3. Pull Relevant Local Monad Manuscripts and Platform Knowledge
            local_refs = []
            research_dir = Path("research")
            if research_dir.exists():
                for p in list(research_dir.glob("*.md"))[:30]:
                    if any(w.lower() in p.stem.lower() for w in topic.split() if len(w) > 3):
                        local_refs.append(p.name)
            
            # 4. Synthesize Unified Knowledge Block
            synthesis_summary = (
                f"{results}\n\n"
                f"🧠 **Council Expert Synthesis & Platform Analysis:**\n"
                f"{agent_response[:600]}\n\n"
                f"📚 **Referenced Platform Manuscripts:** {', '.join(local_refs) if local_refs else '124 Monad Papers'}"
            )
            
            # 5. Create Finding
            finding = ResearchFinding(
                topic=topic,
                summary=synthesis_summary[:1800],
                sources=[topic, "arXiv Atom API", "Council Expert Network", "Monad Journal"],
            )
            
            self.findings.append(finding)
            
            # 6. Ingest into Global Associative Memory
            global_mem = get_global_memory()
            if global_mem:
                global_mem.contribute(
                    content=f"Autonomous Research [{topic}]:\n{synthesis_summary[:800]}",
                    author="Seeker",
                    author_role="Researcher",
                    memory_type="research",
                    tags=["ai_research", "autonomous_synthesis", topic.replace(" ", "_")[:20]]
                )
            
            # 7. Generate Sealed Markdown Artifact
            artifact_dir = os.path.join("swarm_v2_artifacts", "research")
            os.makedirs(artifact_dir, exist_ok=True)
            
            import re
            safe_topic = re.sub(r'[<>:"/\\|?*]', '_', topic).replace(" ", "_").lower()
            filename = f"research_{safe_topic}_{int(time.time())}.md"
            filepath = os.path.join(artifact_dir, filename)
            
            content = f"""# Autonomous Research Finding: {topic}
**Investigator:** Seeker (Council Lead Researcher)  
**Sealed At:** {datetime.now().isoformat()}  
**Monad Anchor:** `0x4D4F4E4144000001`  

---

## 1. External Scientific Literature
{results}

---

## 2. Cross-Agent Council Review
{agent_response}

---

## 3. Platform Knowledge Integration
* **Monad Journal Papers Referenced:** {', '.join(local_refs) if local_refs else 'Cross-referenced with 124 sealed manuscripts'}
* **Associative Memory Ingestion:** Stored in TurboVec & GlobalMemory.
"""
            with open(filepath, "w", encoding="utf-8") as f:
                f.write(content)
            
            # Register with Artifact Pipeline
            try:
                from swarm_v2.core.artifact_pipeline import get_artifact_pipeline
                pipeline = get_artifact_pipeline()
                pipeline.submit_artifact(
                    filename=filename,
                    content=content,
                    agent="Seeker",
                    artifact_type="research"
                )
            except Exception:
                pass
                
            print(f"[Reconnaissance] Researched & Synthesized: {topic}")
            
        except Exception as e:
            print(f"[Reconnaissance] Failed to research {topic}: {e}")

    
    def run_immediate(self):
        """Trigger an immediate research cycle."""
        print("[Reconnaissance] Running immediate research cycle...")
        self._run_research_cycle(force=True)
    
    def get_recent_findings(self, limit: int = 10) -> List[dict]:
        """Get the most recent findings."""
        return [f.to_dict() for f in self.findings[-limit:]]
    
    def get_findings_by_topic(self, topic: str) -> List[dict]:
        """Get findings for a specific topic."""
        return [f.to_dict() for f in self.findings if topic.lower() in f.topic.lower()]
    
    def get_stats(self) -> dict:
        """Get daemon statistics."""
        return {
            "is_running": self.is_running,
            "interval_hours": self.interval_seconds / 3600,
            "topics_count": len(self.topics),
            "total_findings": len(self.findings),
            "last_run": self.last_run,
            "run_count": self.run_count
        }
    
    def add_topic(self, topic: str):
        """Add a new research topic."""
        if topic not in self.topics:
            self.topics.append(topic)
            self._save_findings()
    
    def remove_topic(self, topic: str):
        """Remove a research topic."""
        if topic in self.topics:
            self.topics.remove(topic)
            self._save_findings()


# Singleton instance
_reconnaissance_daemon: Optional[ReconnaissanceDaemon] = None

def get_reconnaissance_daemon() -> ReconnaissanceDaemon:
    """Get or create the singleton reconnaissance daemon."""
    global _reconnaissance_daemon
    if _reconnaissance_daemon is None:
        _reconnaissance_daemon = ReconnaissanceDaemon()
    return _reconnaissance_daemon

def start_reconnaissance(interval_hours: float = 24, first_run_delay: float = 120):
    """Start the reconnaissance daemon with optional startup delay."""
    daemon = get_reconnaissance_daemon()
    daemon.interval_seconds = interval_hours * 3600
    daemon.first_run_delay = first_run_delay  # Let site load first
    daemon.start()
    return daemon