
# Research Integration Plan: latest AI research papers 2024

## Overview
The 2024 AI research landscape is characterized by a massive surge in publication volume (nearly 2x from 2023) and the rise of competitive benchmarking platforms like "Chatbot Arena." This research enables the QIAE Swarm to evolve from a static knowledge base to a **dynamic, self-updating intelligence network**. The core opportunity is twofold:
1.  **Adaptive Ingestion:** The swarm must autonomously filter, prioritize, and ingest the flood of new papers to remain current.
2.  **Competitive Benchmarking:** The swarm needs an internal "Arena" to pit its own generated models/agents against the latest state-of-the-art results from these papers, closing the feedback loop.

## Proposed Changes

### 1. New Module: `swarm_ingestion_orchestrator`
- **File:** `src/swarm/ingestion/orchestrator.py`
- **Purpose:** Manages the adaptive ingestion pipeline. It will use a priority queue based on citation velocity, venue prestige, and relevance to active swarm projects.
- **Key Components:**
    - `ArxivHarvester`: Async scraper for new papers, respecting rate limits.
    - `DeduplicationEngine`: Uses semantic hashing to avoid re-ingesting the same paper.
    - `PriorityRouter`: Routes high-priority papers to the `Seeker` agent for immediate analysis; low-priority papers go to a batch processing queue.

### 2. New Module: `swarm_arena`
- **File:** `src/swarm/evaluation/arena.py`
- **Purpose:** Implements a "Chatbot Arena"-style Elo rating system for swarm-generated models and agents.
- **Key Components:**
    - `ArenaMatchmaker`: Pairs swarm agents against each other or against baseline models from ingested papers.
    - `EloTracker`: Maintains a leaderboard of agent performance.
    - `BenchmarkSuite`: A library of standardized tasks (e.g., MATH, HumanEval, MMLU) extracted from the latest papers.

### 3. Modified File: `src/swarm/core/config.py`
- **Change:** Add configuration parameters for the ingestion orchestrator (e.g., `MAX_PAPERS_PER_DAY`, `ARENA_ELO_K_FACTOR`, `PRIORITY_THRESHOLD`).

### 4. Modified File: `src/swarm/core/scheduler.py`
- **Change:** Register new periodic tasks:
    - `harvest_new_papers`: Runs every 6 hours.
    - `run_arena_battles`: Runs every 24 hours.

## Verification Plan

### Unit Tests
- **Test `ArxivHarvester`:** Mock the arXiv API to ensure it correctly parses paper metadata and handles network errors.
- **Test `DeduplicationEngine`:** Feed it duplicate paper titles and abstracts; verify it returns a single unique entry.
- **Test `EloTracker`:** Simulate a series of matches and verify the Elo scores converge correctly.

### Integration Tests
- **End-to-End Ingestion:** Start the swarm, trigger a manual harvest, and verify that a new paper appears in the knowledge graph.
- **Arena Battle:** Create two mock agents with known performance, run them through the arena, and verify the leaderboard updates.

### Performance Benchmarks
- **Ingestion Throughput:** Measure the time to process 1000 new papers. Target: < 60 seconds.
- **Arena Latency:** Measure the time to complete a single battle between two agents. Target: < 30 seconds.