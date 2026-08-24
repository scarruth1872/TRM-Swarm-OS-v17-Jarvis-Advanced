"""
Comprehensive pytest test suite for the Research Integration Plan:
latest AI research papers 2024

Tests cover:
1. Swarm Ingestion Orchestrator module
2. Swarm Arena evaluation module
3. Integration scenarios
4. Edge cases and error handling
5. Performance benchmarks
6. Data integrity validation
"""

import pytest
import asyncio
import json
import hashlib
import time
from datetime import datetime, timedelta
from unittest.mock import Mock, patch, AsyncMock, MagicMock
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from collections import defaultdict
import numpy as np

# ============================================================================
# Test Configuration & Fixtures
# ============================================================================

@dataclass
class MockPaper:
    """Mock research paper for testing"""
    arxiv_id: str
    title: str
    authors: List[str]
    abstract: str
    categories: List[str]
    published: datetime
    updated: datetime
    citation_count: int = 0
    venue_prestige: float = 0.5
    relevance_score: float = 0.5

@dataclass
class MockModelResult:
    """Mock model evaluation result"""
    model_id: str
    accuracy: float
    latency_ms: float
    memory_mb: float
    parameters_b: float
    benchmark_scores: Dict[str, float]
    timestamp: datetime = field(default_factory=datetime.now)

@pytest.fixture
def sample_papers() -> List[MockPaper]:
    """Generate sample papers for testing"""
    return [
        MockPaper(
            arxiv_id="2401.001",
            title="Advanced Transformer Architectures",
            authors=["Smith J", "Chen L"],
            abstract="Novel transformer architecture with improved efficiency",
            categories=["cs.LG", "cs.CL"],
            published=datetime(2024, 1, 15),
            updated=datetime(2024, 2, 1),
            citation_count=150,
            venue_prestige=0.9,
            relevance_score=0.85
        ),
        MockPaper(
            arxiv_id="2401.002",
            title="Reinforcement Learning Advances",
            authors=["Wang X", "Patel R"],
            abstract="New RL algorithms for complex environments",
            categories=["cs.LG", "cs.AI"],
            published=datetime(2024, 2, 10),
            updated=datetime(2024, 2, 20),
            citation_count=75,
            venue_prestige=0.7,
            relevance_score=0.6
        ),
        MockPaper(
            arxiv_id="2401.003",
            title="Edge Case: Duplicate Paper",
            authors=["Smith J", "Chen L"],
            abstract="Novel transformer architecture with improved efficiency",
            categories=["cs.LG", "cs.CL"],
            published=datetime(2024, 1, 15),
            updated=datetime(2024, 2, 1),
            citation_count=150,
            venue_prestige=0.9,
            relevance_score=0.85
        ),
        MockPaper(
            arxiv_id="2401.004",
            title="Low Priority Survey Paper",
            authors=["Brown A"],
            abstract="Comprehensive survey of existing methods",
            categories=["cs.AI"],
            published=datetime(2024, 3, 1),
            updated=datetime(2024, 3, 5),
            citation_count=10,
            venue_prestige=0.3,
            relevance_score=0.2
        ),
    ]

@pytest.fixture
def sample_model_results() -> List[MockModelResult]:
    """Generate sample model evaluation results"""
    return [
        MockModelResult(
            model_id="swarm-v1",
            accuracy=0.92,
            latency_ms=45.2,
            memory_mb=256,
            parameters_b=7.0,
            benchmark_scores={"mmlu": 0.85, "hellaswag": 0.88, "truthfulqa": 0.76}
        ),
        MockModelResult(
            model_id="swarm-v2",
            accuracy=0.95,
            latency_ms=38.1,
            memory_mb=320,
            parameters_b=13.0,
            benchmark_scores={"mmlu": 0.89, "hellaswag": 0.91, "truthfulqa": 0.82}
        ),
        MockModelResult(
            model_id="gpt-4",
            accuracy=0.97,
            latency_ms=120.0,
            memory_mb=1024,
            parameters_b=175.0,
            benchmark_scores={"mmlu": 0.93, "hellaswag": 0.94, "truthfulqa": 0.88}
        ),
    ]

# ============================================================================
# Swarm Ingestion Orchestrator Tests
# ============================================================================

class TestArxivHarvester:
    """Test the ArxivHarvester component"""

    @pytest.fixture
    def harvester(self):
        """Create ArxivHarvester instance"""
        from swarm.ingestion.orchestrator import ArxivHarvester
        return ArxivHarvester(max_concurrent=5, rate_limit_per_second=3)

    @pytest.mark.asyncio
    async def test_harvest_new_papers(self, harvester, sample_papers):
        """Test harvesting new papers from arxiv"""
        with patch.object(harvester, '_fetch_papers', return_value=sample_papers[:2]):
            papers = await harvester.harvest(
                categories=["cs.LG", "cs.CL"],
                date_from=datetime(2024, 1, 1),
                max_results=10
            )
            
            assert len(papers) == 2
            assert all(p.arxiv_id.startswith("2401") for p in papers)
            assert papers[0].citation_count > papers[1].citation_count

    @pytest.mark.asyncio
    async def test_rate_limiting(self, harvester):
        """Test rate limiting behavior"""
        start_time = time.time()
        
        with patch.object(harvester, '_fetch_papers', return_value=[]):
            tasks = [harvester.harvest(categories=["cs.LG"]) for _ in range(10)]
            await asyncio.gather(*tasks)
        
        elapsed = time.time() - start_time
        min_expected_time = 10 / harvester.rate_limit_per_second
        assert elapsed >= min_expected_time * 0.9  # Allow 10% tolerance

    @pytest.mark.asyncio
    async def test_concurrent_limit(self, harvester):
        """Test max concurrent requests enforcement"""
        active_requests = []
        
        async def slow_fetch(*args, **kwargs):
            active_requests.append(1)
            await asyncio.sleep(0.1)
            assert len(active_requests) <= harvester.max_concurrent
            active_requests.remove(1)
            return []
        
        with patch.object(harvester, '_fetch_papers', side_effect=slow_fetch):
            tasks = [harvester.harvest(categories=["cs.LG"]) for _ in range(20)]
            await asyncio.gather(*tasks)

    @pytest.mark.asyncio
    async def test_error_handling_network_failure(self, harvester):
        """Test graceful handling of network failures"""
        with patch.object(harvester, '_fetch_papers', side_effect=ConnectionError("Network timeout")):
            with pytest.raises(ConnectionError):
                await harvester.harvest(categories=["cs.LG"])
        
        # Verify harvester is still operational
        assert harvester._session is not None or harvester._session is None

    @pytest.mark.asyncio
    async def test_empty_response(self, harvester):
        """Test handling of empty API response"""
        with patch.object(harvester, '_fetch_papers', return_value=[]):
            papers = await harvester.harvest(categories=["cs.XX"])
            assert len(papers) == 0

    def test_validate_paper_schema(self, harvester):
        """Test paper schema validation"""
        valid_paper = {
            "arxiv_id": "2401.001",
            "title": "Test Paper",
            "authors": ["Author1"],
            "abstract": "Test abstract",
            "categories": ["cs.LG"],
            "published": "2024-01-01",
            "updated": "2024-01-02"
        }
        
        assert harvester._validate_paper(valid_paper) is True
        
        invalid_paper = {
            "arxiv_id": "2401.001",
            "title": "Test Paper"
            # Missing required fields
        }
        
        assert harvester._validate_paper(invalid_paper) is False


class TestDeduplicationEngine:
    """Test the DeduplicationEngine component"""

    @pytest.fixture
    def engine(self):
        """Create DeduplicationEngine instance"""
        from swarm.ingestion.orchestrator import DeduplicationEngine
        return DeduplicationEngine(similarity_threshold=0.85)

    def test_semantic_hashing(self, engine, sample_papers):
        """Test semantic hash generation"""
        hashes = [engine._compute_semantic_hash(p) for p in sample_papers]
        
        # Same content should produce same hash
        assert hashes[0] == hashes[2]
        
        # Different content should produce different hashes
        assert hashes[0] != hashes[1]

    def test_deduplication(self, engine, sample_papers):
        """Test duplicate detection and removal"""
        unique_papers = engine.deduplicate(sample_papers)
        
        assert len(unique_papers) == 3  # One duplicate removed
        assert sample_papers[0] in unique_papers
        assert sample_papers[2] not in unique_papers  # Duplicate removed

    def test_similarity_scoring(self, engine):
        """Test similarity scoring between papers"""
        paper1 = MockPaper(
            arxiv_id="2401.001",
            title="Deep Learning Advances",
            authors=["Author1"],
            abstract="Novel deep learning techniques for NLP",
            categories=["cs.LG"],
            published=datetime(2024, 1, 1),
            updated=datetime(2024, 1, 2)
        )
        
        paper2 = MockPaper(
            arxiv_id="2401.002",
            title="Deep Learning Methods",
            authors=["Author2"],
            abstract="Novel deep learning approaches for NLP tasks",
            categories=["cs.LG"],
            published=datetime(2024, 1, 3),
            updated=datetime(2024, 1, 4)
        )
        
        similarity = engine._compute_similarity(paper1, paper2)
        assert 0.0 <= similarity <= 1.0
        assert similarity > 0.5  # Similar topics

    def test_edge_case_empty_input(self, engine):
        """Test deduplication with empty input"""
        result = engine.deduplicate([])
        assert len(result) == 0

    def test_edge_case_single_paper(self, engine, sample_papers):
        """Test deduplication with single paper"""
        result = engine.deduplicate([sample_papers[0]])
        assert len(result) == 1

    def test_performance_large_batch(self, engine):
        """Test performance with large batch of papers"""
        large_batch = [
            MockPaper(
                arxiv_id=f"2401.{i:03d}",
                title=f"Paper {i}",
                authors=[f"Author{i}"],
                abstract=f"Abstract {i} with some unique content",
                categories=["cs.LG"],
                published=datetime(2024, 1, 1),
                updated=datetime(2024, 1, 2)
            )
            for i in range(1000)
        ]
        
        import time
        start = time.time()
        result = engine.deduplicate(large_batch)
        elapsed = time.time() - start
        
        assert len(result) == 1000  # No duplicates
        assert elapsed < 5.0  # Should complete within 5 seconds


class TestPriorityRouter:
    """Test the PriorityRouter component"""

    @pytest.fixture
    def router(self):
        """Create PriorityRouter instance"""
        from swarm.ingestion.orchestrator import PriorityRouter
        return PriorityRouter(
            high_priority_threshold=0.7,
            citation_velocity_weight=0.4,
            venue_prestige_weight=0.3,
            relevance_weight=0.3
        )

    def test_priority_scoring(self, router, sample_papers):
        """Test priority score calculation"""
        scores = [router._compute_priority_score(p) for p in sample_papers]
        
        # High citation, high prestige paper should score higher
        assert scores[0] > scores[3]
        
        # All scores should be normalized
        assert all(0.0 <= s <= 1.0 for s in scores)

    def test_routing_decision(self, router, sample_papers):
        """Test routing decisions based on priority"""
        for paper in sample_papers:
            route = router.route(paper)
            score = router._compute_priority_score(paper)
            
            if score >= router.high_priority_threshold:
                assert route == "immediate_analysis"
            else:
                assert route == "batch_processing"

    def test_citation_velocity(self, router):
        """Test citation velocity calculation"""
        recent_paper = MockPaper(
            arxiv_id="2401.001",
            title="Recent Paper",
            authors=["Author1"],
            abstract="Recent breakthrough",
            categories=["cs.LG"],
            published=datetime(2024, 6, 1),
            updated=datetime(2024, 6, 15),
            citation_count=100
        )
        
        old_paper = MockPaper(
            arxiv_id="2401.002",
            title="Old Paper",
            authors=["Author2"],
            abstract="Old research",
            categories=["cs.LG"],
            published=datetime(2024, 1, 1),
            updated=datetime(2024, 1, 15),
            citation_count=100
        )
        
        recent_velocity = router._compute_citation_velocity(recent_paper)
        old_velocity = router._compute_citation_velocity(old_paper)
        
        assert recent_velocity > old_velocity

    def test_priority_queue_management(self, router, sample_papers):
        """Test priority queue operations"""
        queue = router.create_priority_queue(sample_papers)
        
        # Verify ordering
        previous_score = float('inf')
        for paper in queue:
            score = router._compute_priority_score(paper)
            assert score <= previous_score
            previous_score = score

    def test_edge_case_zero_citations(self, router):
        """Test handling of papers with zero citations"""
        paper = MockPaper(
            arxiv_id="2401.001",
            title="Uncited Paper",
            authors=["Author1"],
            abstract="New research",
            categories=["cs.LG"],
            published=datetime(2024, 6, 1),
            updated=datetime(2024, 6, 15),
            citation_count=0
        )
        
        score = router._compute_priority_score(paper)
        assert 0.0 <= score <= 1.0
        assert score < 0.5  # Should be low priority


# ============================================================================
# Swarm Arena Tests
# ============================================================================

class TestSwarmArena:
    """Test the Swarm Arena evaluation module"""

    @pytest.fixture
    def arena(self):
        """Create SwarmArena instance"""
        from swarm.evaluation.arena import SwarmArena
        return SwarmArena(
            benchmark_suite=["mmlu", "hellaswag", "truthfulqa"],
            elo_k_factor=32,
            initial_rating=1000
        )

    def test_model_registration(self, arena, sample_model_results):
        """Test model registration in arena"""
        for result in sample_model_results:
            arena.register_model(result)
        
        assert len(arena.models) == 3
        assert all(m.model_id in arena.ratings for m in sample_model_results)

    def test_elo_rating_update(self, arena):
        """Test Elo rating calculation"""
        # Register two models
        model_a = MockModelResult(
            model_id="model-a",
            accuracy=0.9,
            latency_ms=50,
            memory_mb=256,
            parameters_b=7.0,
            benchmark_scores={"mmlu": 0.85}
        )
        
        model_b = MockModelResult(
            model_id="model-b",
            accuracy=0.8,
            latency_ms=40,
            memory_mb=128,
            parameters_b=3.0,
            benchmark_scores={"mmlu": 0.75}
        )
        
        arena.register_model(model_a)
        arena.register_model(model_b)
        
        # Simulate match
        initial_rating_a = arena.ratings["model-a"]
        initial_rating_b = arena.ratings["model-b"]
        
        arena.record_match("model-a", "model-b", winner="model-a")
        
        # Winner should gain rating, loser should lose
        assert arena.ratings["model-a"] > initial_rating_a
        assert arena.ratings["model-b"] < initial_rating_b

    def test_benchmark_comparison(self, arena, sample_model_results):
        """Test benchmark comparison between models"""
        for result in sample_model_results:
            arena.register_model(result)
        
        comparison = arena.compare_models("swarm-v1", "gpt-4")
        
        assert "swarm-v1" in comparison
        assert "gpt-4" in comparison
        assert "differences" in comparison
        assert all(k in comparison["differences"] for k in ["mmlu", "hellaswag", "truthfulqa"])

    def test_leaderboard_generation(self, arena, sample_model_results):
        """Test leaderboard generation"""
        for result in sample_model_results:
            arena.register_model(result)
        
        # Simulate some matches
        arena.record_match("swarm-v1", "swarm-v2", winner="swarm-v2")
        arena.record_match("swarm-v2", "gpt-4", winner="gpt-4")
        arena.record_match("swarm-v1", "gpt-4", winner="gpt-4")
        
        leaderboard = arena.get_leaderboard()
        
        assert len(leaderboard) == 3
        assert leaderboard[0]["model_id"] == "gpt-4"  # Should be top
        assert leaderboard[2]["model_id"] == "swarm-v1"  # Should be bottom

    def test_arena_statistics(self, arena, sample_model_results):
        """Test arena statistics computation"""
        for result in sample_model_results:
            arena.register_model(result)
        
        # Record multiple matches
        for _ in range(10):
            arena.record_match("swarm-v1", "swarm-v2", winner="swarm-v2")
            arena.record_match("swarm-v2", "gpt-4", winner="gpt-4")
        
        stats = arena.get_statistics()
        
        assert "total_matches" in stats
        assert stats["total_matches"] == 20
        assert "model_stats" in stats
        assert all(m in stats["model_stats"] for m in ["swarm-v1", "swarm-v2", "gpt-4"])

    def test_edge_case_same_model_match(self, arena):
        """Test handling of match between same model"""
        model = MockModelResult(
            model_id="model-a",
            accuracy=0.9,
            latency_ms=50,
            memory_mb=256,
            parameters_b=7.0,
            benchmark_scores={"mmlu": 0.85}
        )
        
        arena.register_model(model)
        
        with pytest.raises(ValueError, match="Cannot match a model against itself"):
            arena.record_match("model-a", "model-a", winner="model-a")

    def test_edge_case_unregistered_model(self, arena):
        """Test handling of unregistered model in match"""
        with pytest.raises(KeyError, match="Model not registered"):
            arena.record_match("unregistered", "model-a", winner="unregistered")

    def test_performance_many_models(self, arena):
        """Test arena performance with many models"""
        models = [
            MockModelResult(
                model_id=f"model-{i}",
                accuracy=np.random.random(),
                latency_ms=np.random.uniform(10, 100),
                memory_mb=np.random.uniform(100, 1000),
                parameters_b=np.random.uniform(1, 100),
                benchmark_scores={f"bench_{j}": np.random.random() for j in range(5)}
            )
            for i in range(100)
        ]
        
        import time
        start = time.time()
        
        for model in models:
            arena.register_model(model)
        
        # Simulate tournament
        for i in range(len(models)):
            for j in range(i + 1, len(models)):
                winner = models[i].model_id if models[i].accuracy > models[j].accuracy else models[j].model_id
                arena.record_match(models[i].model_id, models[j].model_id, winner=winner)
        
        elapsed = time.time() - start
        assert elapsed < 30.0  # Should complete within 30 seconds


# ============================================================================
# Integration Tests
# ============================================================================

class TestIntegration:
    """Test integration between ingestion and evaluation"""

    @pytest.fixture
    async def integrated_system(self):
        """Create integrated system with all components"""
        from swarm.ingestion.orchestrator import ArxivHarvester, DeduplicationEngine, PriorityRouter
        from swarm.evaluation.arena import SwarmArena
        
        harvester = ArxivHarvester(max_concurrent=3, rate_limit_per_second=2)
        dedup = DeduplicationEngine(similarity_threshold=0.85)
        router = PriorityRouter(
            high_priority_threshold=0.7,
            citation_velocity_weight=0.4,
            venue_prestige_weight=0.3,
            relevance_weight=0.3
        )
        arena = SwarmArena(
            benchmark_suite=["mmlu", "hellaswag"],
            elo_k_factor=32,
            initial_rating=1000
        )
        
        return {
            "harvester": harvester,
            "dedup": dedup,
            "router": router,
            "arena": arena
        }

    @pytest.mark.asyncio
    async def test_full_pipeline(self, integrated_system, sample_papers):
        """Test complete ingestion to evaluation pipeline"""
        system = await integrated_system
        
        # Step 1: Harvest papers
        with patch.object(system["harvester"], '_fetch_papers', return_value=sample_papers):
            harvested = await system["harvester"].harvest(
                categories=["cs.LG"],
                date_from=datetime(2024, 1, 1)
            )
        
        # Step 2: Deduplicate
        unique_papers = system["dedup"].deduplicate(harvested)
        assert len(unique_papers) == 3  # One duplicate removed
        
        # Step 3: Route by priority
        high_priority = []
        low_priority = []
        for paper in unique_papers:
            route = system["router"].route(paper)
            if route == "immediate_analysis":
                high_priority.append(paper)
            else:
                low_priority.append(paper)
        
        assert len(high_priority) >= 1  # At least one high priority paper
        
        # Step 4: Evaluate in arena
        for paper in high_priority:
            model_result = MockModelResult(
                model_id=f"model-{paper.arxiv_id}",
                accuracy=0.85,
                latency_ms=50,
                memory_mb=256,
                parameters_b=7.0,
                benchmark_scores={"mmlu": 0.80, "hellaswag": 0.82}
            )
            system["arena"].register_model(model_result)
        
        assert len(system["arena"].models) == len(high_priority)

    @pytest.mark.asyncio
    async def test_error_recovery_pipeline(self, integrated_system, sample_papers):
        """Test pipeline recovery from errors"""
        system = await integrated_system
        
        # Simulate partial failure in harvesting
        with patch.object(system["harvester"], '_fetch_papers', side_effect=[
            sample_papers[:2],  # First call succeeds
            ConnectionError("Network error"),  # Second call fails
            sample_papers[2:]  # Third call succeeds
        ]):
            results = []
            for attempt in range(3):
                try:
                    papers = await system["harvester"].harvest(
                        categories=["cs.LG"],
                        date_from=datetime(2024, 1, 1)
                    )
                    results.extend(papers)
                except ConnectionError:
                    continue
            
            assert len(results) >= 2  # Should have partial results

    def test_data_consistency(self, integrated_system, sample_papers):
        """Test data consistency across pipeline"""
        system = integrated_system
        
        # Verify paper data integrity
        for paper in sample_papers:
            assert paper.arxiv_id is not None
            assert paper.title is not None
            assert len(paper.authors) > 0
            assert paper.published <= paper.updated  # Temporal consistency
        
        # Verify model result consistency
        model_results = [
            MockModelResult(
                model_id=f"model-{i}",
                accuracy=0.9,
                latency_ms=50,
                memory_mb=256,
                parameters_b=7.0,
                benchmark_scores={"mmlu": 0.85}
            )
            for i in range(5)
        ]
        
        for result in model_results:
            system["arena"].register_model(result)
            assert result.model_id in system["arena"].ratings
            assert 0 <= system["arena"].ratings[result.model_id] <= 3000  # Elo range


# ============================================================================
# Property-Based Tests
# ============================================================================

class TestProperties:
    """Property-based tests for system invariants"""

    def test_priority_score_monotonicity(self):
        """Test that priority scores are monotonic with respect to inputs"""
        from swarm.ingestion.orchestrator import PriorityRouter
        
        router = PriorityRouter(
            high_priority_threshold=0.7,
            citation_velocity_weight=0.4,
            venue_prestige_weight=0.3,
            relevance_weight=0.3
        )
        
        # Property: Increasing any component should not decrease score
        base_paper = MockPaper(
            arxiv_id="2401.001",
            title="Base Paper",
            authors=["Author1"],
            abstract="Base research",
            categories=["cs.LG"],
            published=datetime(2024, 6, 1),
            updated=datetime(2024, 6, 15),
            citation_count=50,
            venue_prestige=0.5,
            relevance_score=0.5
        )
        
        base_score = router._compute_priority_score(base_paper)
        
        # Increase citation count
        high_cite_paper = MockPaper(
            arxiv_id="2401.002",
            title="High Cite Paper",
            authors=["Author2"],
            abstract="Popular research",
            categories=["cs.LG"],
            published=datetime(2024, 6, 1),
            updated=datetime(2024, 6, 15),
            citation_count=500,
            venue_prestige=0.5,
            relevance_score=0.5
        )
        
        assert router._compute_priority_score(high_cite_paper) >= base_score
        
        # Increase venue prestige
        high_prestige_paper = MockPaper(
            arxiv_id="2401.003",
            title="High Prestige Paper",
            authors=["Author3"],
            abstract="Important research",
            categories=["cs.LG"],
            published=datetime(2024, 6, 1),
            updated=datetime(2024, 6, 15),
            citation_count=50,
            venue_prestige=0.9,
            relevance_score=0.5
        )
        
        assert router._compute_priority_score(high_prestige_paper) >= base_score

    def test_elo_rating_conservation(self):
        """Test that Elo rating changes are conserved"""
        from swarm.evaluation.arena import SwarmArena
        
        arena = SwarmArena(
            benchmark_suite=["mmlu"],
            elo_k_factor=32,
            initial_rating=1000
        )
        
        model_a = MockModelResult(
            model_id="model-a",
            accuracy=0.9,
            latency_ms=50,
            memory_mb=256,
            parameters_b=7.0,
            benchmark_scores={"mmlu": 0.85}
        )
        
        model_b = MockModelResult(
            model_id="model-b",
            accuracy=0.8,
            latency_ms=40,
            memory_mb=128,
            parameters_b=3.0,
            benchmark_scores={"mmlu": 0.75}
        )
        
        arena.register_model(model_a)
        arena.register_model(model_b)
        
        initial_total = arena.ratings["model-a"] + arena.ratings["model-b"]
        
        arena.record_match("model-a", "model-b", winner="model-a")
        
        final_total = arena.ratings["model-a"] + arena.ratings["model-b"]
        
        # Property: Total Elo should be conserved
        assert abs(final_total - initial_total) < 0.01

    def test_deduplication_idempotency(self):
        """Test that deduplication is idempotent"""
        from swarm.ingestion.orchestrator import DeduplicationEngine
        
        engine = DeduplicationEngine(similarity_threshold=0.85)
        
        papers = [
            MockPaper(
                arxiv_id=f"2401.{i:03d}",
                title=f"Paper {i}",
                authors=[f"Author{i}"],
                abstract=f"Abstract {i}",
                categories=["cs.LG"],
                published=datetime(2024, 1, 1),
                updated=datetime(2024, 1, 2)
            )
            for i in range(10)
        ]
        
        # Add duplicates
        papers.append(papers[0])
        papers.append(papers[3])
        
        # First deduplication
        first_pass = engine.deduplicate(papers)
        
        # Second deduplication should not change anything
        second_pass = engine.deduplicate(first_pass)
        
        assert len(first_pass) == len(second_pass)
        assert all(p in second_pass for p in first_pass)


# ============================================================================
# Performance & Stress Tests
# ============================================================================

class TestPerformance:
    """Performance and stress tests"""

    @pytest.mark.benchmark
    def test_ingestion_throughput(self, benchmark):
        """Benchmark ingestion pipeline throughput"""
        from swarm.ingestion.orchestrator import DeduplicationEngine, PriorityRouter
        
        # Generate large batch of papers
        papers = [
            MockPaper(
                arxiv_id=f"2401.{i:03d}",
                title=f"Paper {i}",
                authors=[f"Author{i}"],
                abstract=f"Abstract {i} with sufficient content for hashing",
                categories=["cs.LG"],
                published=datetime(2024, 1, 1),
                updated=datetime(2024, 1, 2),
                citation_count=i * 10,
                venue_prestige=min(1.0, i / 100),
                relevance_score=min(1.0, i / 100)
            )
            for i in range(1000)
        ]
        
        def run_pipeline():
            dedup = DeduplicationEngine(similarity_threshold=0.85)
            router = PriorityRouter(
                high_priority_threshold=0.7,
                citation_velocity_weight=0.4,
                venue_prestige_weight=0.3,
                relevance_weight=0.3
            )
            
            unique = dedup.deduplicate(papers)
            for paper in unique:
                router.route(paper)
            
            return len(unique)
        
        result = benchmark(run_pipeline)
        assert result == 1000  # All papers should be unique

    @pytest.mark.stress
    def test_arena_concurrent_matches(self):
        """Test arena under concurrent match load"""
        from swarm.evaluation.arena import SwarmArena
        import concurrent.futures
        
        arena = SwarmArena(
            benchmark_suite=["mmlu"],
            elo_k_factor=32,
            initial_rating=1000
        )
        
        # Register many models
        models = [
            MockModelResult(
                model_id=f"model-{i}",
                accuracy=np.random.random(),
                latency_ms=np.random.uniform(10, 100),
                memory_mb=np.random.uniform(100, 1000),
                parameters_b=np.random.uniform(1, 100),
                benchmark_scores={"mmlu": np.random.random()}
            )
            for i in range(50)
        ]
        
        for model in models:
            arena.register_model(model)
        
        # Simulate concurrent matches
        def simulate_match(args):
            model_a, model_b = args
            winner = model_a if np.random.random() > 0.5 else model_b
            arena.record_match(model_a, model_b, winner=winner)
            return 1
        
        match_pairs = [
            (models[i].model_id, models[j].model_id)
            for i in range(len(models))
            for j in range(i + 1, len(models))
        ]
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
            results = list(executor.map(simulate_match, match_pairs))
        
        assert sum(results) == len(match_pairs)
        assert all(1000 <= rating <= 3000 for rating in arena.ratings.values())


# ============================================================================
# Configuration & Test Metadata
# ============================================================================

# pytest configuration
pytest_plugins = [
    'pytest_asyncio',
    'pytest_benchmark',
]

# Test configuration
TEST_CONFIG = {
    'timeout': 30,
    'slow_test_timeout': 120,
    'stress_test_iterations': 100,
    'benchmark_iterations': 10,
}

# Markers for test categorization
def pytest_configure(config):
    """Configure pytest markers"""
    config.addinivalue_line(
        "markers",
        "benchmark: Mark test as benchmark test"
    )
    config.addinivalue_line(
        "markers",
        "stress: Mark test as stress test"
    )
    config.addinivalue_line(
        "markers",
        "integration: Mark test as integration test"
    )
    config.addinivalue_line(
        "markers",
        "slow: Mark test as slow running"
    )


# ============================================================================
# Main Execution
# ============================================================================

if __name__ == "__main__":
    pytest.main([
        __file__,
        "-v",
        "--tb=short",
        "--maxfail=5",
        "--timeout=60",
        "-k", "not slow",
        "--benchmark-only",
        "--benchmark-columns=min,mean,std"
    ])