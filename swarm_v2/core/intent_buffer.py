"""
Predictive Intent Buffer
Phase 3: Latency of Intent Synchronization Remediation
Pre-loads target agents with historical patterns and semantic entity context
to anticipate user commands and reduce synchronization latency.
"""

import os
import json
import math
import re
import logging
from typing import Dict, List, Optional, Tuple, Any
from datetime import datetime

logger = logging.getLogger("PredictiveIntentBuffer")

GLOBAL_MEMORY_DIR = os.getenv("SWARM_MEMORY_DIR", "swarm_v2_global_memory")
os.makedirs(GLOBAL_MEMORY_DIR, exist_ok=True)
INTENT_DB_PATH = os.path.join(GLOBAL_MEMORY_DIR, "predictive_intents.json")

# Simple English stop words to filter for TF-IDF fallback
STOP_WORDS = {
    "the", "a", "an", "and", "or", "but", "if", "then", "else", "when", "at", "by",
    "for", "with", "about", "against", "between", "into", "through", "during",
    "before", "after", "above", "below", "to", "from", "up", "down", "in", "out",
    "on", "off", "over", "under", "again", "further", "then", "once", "here",
    "there", "all", "any", "both", "each", "few", "more", "most", "other", "some",
    "such", "no", "nor", "not", "only", "own", "same", "so", "than", "too", "very",
    "s", "t", "can", "will", "just", "don", "should", "now", "i", "you", "my", "we"
}

class PredictiveIntentBuffer:
    def __init__(self):
        self.intents: List[Dict[str, Any]] = []
        self._load_intents()
        self._initialize_chroma()

    def _initialize_chroma(self):
        self.chroma_available = False
        try:
            import chromadb
            self.client = chromadb.PersistentClient(path=os.path.join(GLOBAL_MEMORY_DIR, "chroma"))
            self.collection = self.client.get_or_create_collection(
                name="predictive_intents",
                metadata={"hnsw:space": "cosine"}
            )
            self.chroma_available = True
            logger.info("ChromaDB Vector Backend connected for Predictive Intent Buffer")
            
            # Sync local intents to Chroma if collection is empty
            if self.collection.count() == 0 and self.intents:
                self._sync_to_chroma()
        except Exception as e:
            logger.warning(f"ChromaDB not available for Intent Buffer, using legacy TF-IDF: {e}")

    def _load_intents(self):
        """Load intents from local file, seeding it with standard patterns if empty."""
        if os.path.exists(INTENT_DB_PATH):
            try:
                with open(INTENT_DB_PATH, "r", encoding="utf-8") as f:
                    self.intents = json.load(f)
                logger.info(f"Loaded {len(self.intents)} intents from intent buffer database.")
            except Exception as e:
                logger.error(f"Failed to read intent database: {e}")
                self.intents = []
        
        if not self.intents:
            self._seed_default_intents()

    def _save_intents(self):
        """Save intents to persistent storage."""
        try:
            with open(INTENT_DB_PATH, "w", encoding="utf-8") as f:
                json.dump(self.intents, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to write intent database: {e}")

    def _seed_default_intents(self):
        """Pre-populate default intents representing typical user directives."""
        defaults = [
            {
                "query": "optimize core cognitive state synchronization loops with automatic failover",
                "role": "Orchestrator",
                "specialties": ["coordination", "state-sync", "failover"],
                "frequency": 1
            },
            {
                "query": "verify the code logic structure and run unit tests to check regressions",
                "role": "Verify",
                "specialties": ["testing", "verification", "regressions"],
                "frequency": 1
            },
            {
                "query": "security scan codebase for vulnerabilities and audit dependency gates",
                "role": "Shield",
                "specialties": ["security", "vulnerabilities", "audit"],
                "frequency": 1
            },
            {
                "query": "analyze data performance profiles and generate metrics graphs",
                "role": "Pulse",
                "specialties": ["data-analysis", "metrics", "performance"],
                "frequency": 1
            },
            {
                "query": "design a premium responsive web app interface dashboard mockup",
                "role": "Vision",
                "specialties": ["design", "ui", "ux", "dashboard"],
                "frequency": 1
            },
            {
                "query": "deploy the current build changes to the production server cluster",
                "role": "Flow",
                "specialties": ["deployment", "infrastructure", "devops"],
                "frequency": 1
            },
            {
                "query": "compile latest code changes and build packaging system",
                "role": "Devo",
                "specialties": ["coding", "compilation", "build"],
                "frequency": 1
            },
            {
                "query": "conduct deep scientific research on neural network architectures",
                "role": "Seeker",
                "specialties": ["research", "neural", "exploration"],
                "frequency": 1
            },
            {
                "query": "solve complex probability calculation or mathematical equation",
                "role": "Logic",
                "specialties": ["math", "logic", "probability"],
                "frequency": 1
            }
        ]
        self.intents = defaults
        self._save_intents()
        logger.info(f"Seeded {len(defaults)} default user intents into buffer.")

    def _sync_to_chroma(self):
        """Pushes all currently loaded intents to ChromaDB."""
        if not self.chroma_available:
            return
        
        ids = []
        documents = []
        metadatas = []
        for i, item in enumerate(self.intents):
            ids.append(f"intent_{i}")
            documents.append(item["query"])
            metadatas.append({
                "role": item["role"],
                "specialties": json.dumps(item["specialties"])
            })
            
        try:
            self.collection.add(ids=ids, documents=documents, metadatas=metadatas)
            logger.info("Synchronized local intents to ChromaDB collection.")
        except Exception as e:
            logger.error(f"Failed to sync ChromaDB collection: {e}")

    def record_intent(self, query: str, role: str, specialties: List[str]):
        """Record a completed user query to learn historical pattern."""
        query_cleaned = query.strip().lower()
        if not query_cleaned:
            return

        # Check if intent exists
        matched = None
        for item in self.intents:
            if item["query"] == query_cleaned:
                matched = item
                break

        if matched:
            matched["frequency"] += 1
            matched.setdefault("last_used", datetime.now().isoformat())
        else:
            new_intent = {
                "query": query_cleaned,
                "role": role,
                "specialties": specialties,
                "frequency": 1,
                "created_at": datetime.now().isoformat()
            }
            self.intents.append(new_intent)
            
            # Sync new intent to ChromaDB
            if self.chroma_available:
                try:
                    idx = len(self.intents) - 1
                    self.collection.add(
                        ids=[f"intent_{idx}"],
                        documents=[query_cleaned],
                        metadatas=[{
                            "role": role,
                            "specialties": json.dumps(specialties)
                        }]
                    )
                except Exception as e:
                    logger.error(f"Failed to sync record to Chroma: {e}")
                    
        self._save_intents()
        logger.info(f"Intent recorded: '{query_cleaned[:40]}' -> {role}")

    def predict_intent(self, query: str) -> Dict[str, Any]:
        """Predict target agent role and specialties based on partial/full query."""
        query_cleaned = query.strip().lower()
        if not query_cleaned:
            return {"role": "", "specialties": [], "confidence": 0.0, "source": "empty"}

        # Attempt Vector matching first
        if self.chroma_available:
            try:
                results = self.collection.query(
                    query_texts=[query_cleaned],
                    n_results=1
                )
                if results and results["documents"] and results["documents"][0]:
                    doc = results["documents"][0][0]
                    meta = results["metadatas"][0][0]
                    dist = results["distances"][0][0]
                    
                    # Convert cosine distance to confidence score
                    confidence = max(0.0, min(1.0, 1.0 - dist))
                    
                    # If confidence is decent, return matches
                    if confidence > 0.35:
                        return {
                            "role": meta["role"],
                            "specialties": json.loads(meta["specialties"]),
                            "confidence": round(confidence, 3),
                            "matched_query": doc,
                            "source": "chromadb"
                        }
            except Exception as e:
                logger.error(f"ChromaDB intent query failed: {e}")

        # Fallback: Cosine Similarity match via TF-IDF token overlap
        best_match = None
        best_score = 0.0
        
        query_tokens = self._tokenize(query_cleaned)
        if not query_tokens:
            return {"role": "", "specialties": [], "confidence": 0.0, "source": "fallback-empty"}

        for item in self.intents:
            item_tokens = self._tokenize(item["query"])
            score = self._cosine_similarity(query_tokens, item_tokens)
            if score > best_score:
                best_score = score
                best_match = item

        if best_match and best_score > 0.15:
            # Map frequency multiplier into confidence weight
            freq_weight = min(0.15, (best_match.get("frequency", 1) - 1) * 0.03)
            confidence = min(0.99, best_score + freq_weight)
            
            return {
                "role": best_match["role"],
                "specialties": best_match["specialties"],
                "confidence": round(confidence, 3),
                "matched_query": best_match["query"],
                "source": "tfidf_fallback"
            }

        return {"role": "", "specialties": [], "confidence": 0.0, "source": "no_match"}

    def prewarm_context(self, predicted_role: str, query: str) -> List[Any]:
        """Pre-fetch semantic graph triples and inject into target agent's working memory."""
        try:
            from swarm_v2.core.ses import get_ses
            from swarm_v2.experts.registry import get_expert_team
            
            ses = get_ses()
            team = get_expert_team()
            agent = team.get(predicted_role)
            if not agent:
                return []
                
            # Extract keywords from the query to find semantic entities
            words = [w.strip(",.?!\"'") for w in query.lower().split() if len(w) > 3]
            prewarmed_triples = []
            
            for word in words:
                context = ses.get_context(word)
                if context and "triples" in context:
                    for t in context["triples"]:
                        prewarmed_triples.append(t)
                        # Inject directly into agent's long/short term memory database
                        if hasattr(agent, "memory"):
                            fact_str = f"Pre-warmed Context: {t[0]} -> {t[1]} -> {t[2]}"
                            agent.memory.add_fact(fact_str, importance=0.8)
            
            if prewarmed_triples:
                logger.info(f"Pre-warmed {predicted_role} with {len(prewarmed_triples)} semantic triples for query: '{query}'")
            return prewarmed_triples
        except Exception as e:
            logger.error(f"Error during context pre-warming: {e}")
            return []

    def _tokenize(self, text: str) -> List[str]:
        words = re.findall(r'\b\w+\b', text.lower())
        return [w for w in words if w not in STOP_WORDS]

    def _cosine_similarity(self, vec1: List[str], vec2: List[str]) -> float:
        """Compute token overlap cosine similarity."""
        set1, set2 = set(vec1), set(vec2)
        intersection = set1.intersection(set2)
        if not intersection:
            return 0.0
        return len(intersection) / math.sqrt(len(set1) * len(set2))
