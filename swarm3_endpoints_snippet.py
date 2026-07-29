@app.get("/swarm/meta-orchestrator")
async def get_meta_orchestrator_status():
    """Exposes unified scheduling telemetry."""
    from swarm_v2.core.meta_orchestrator import get_meta_orchestrator
    return get_meta_orchestrator().get_status()


@app.get("/swarm/chaos")
async def get_chaos_status():
    """Exposes active chaos simulation settings."""
    from swarm_v2.core.chaos_engine import get_chaos_engine
    return get_chaos_engine().get_profile()


@app.post("/swarm/chaos")
async def set_chaos_profile(latency: float = 0.0, dropout_rate: float = 0.0, enabled: bool = True):
    """Dynamically set or toggle network chaos levels."""
    from swarm_v2.core.chaos_engine import get_chaos_engine
    chaos = get_chaos_engine()
    if enabled:
        chaos.enable()
    else:
        chaos.disable()
    chaos.set_latency(latency)
    chaos.set_dropout_rate(dropout_rate)
    return {"status": "success", "profile": chaos.get_profile()}


@app.get("/evolution/genome")
async def get_evolution_genome():
    """Get the current state of the swarm's architecture and evolution history (Phase 16)."""
    from swarm_v2.core.mutation_engine import get_mutation_engine

    engine = get_mutation_engine()
    if not engine.active_targets:
        await engine.analyze_codebase()
    return {
        "version": "16.0.0-evolutionary",
        "genome_stability": 0.95,
        "mutation_count": len(engine.proposals),
        "targets": engine.active_targets,
    }


@app.post("/evolution/mutate")
async def trigger_manual_mutation(target_file: Optional[str] = None):
    """Manually trigger an evolutionary mutation cycle (Phase 16)."""
    from swarm_v2.core.mutation_engine import get_mutation_engine

    engine = get_mutation_engine()

    if not target_file:
        targets = await engine.analyze_codebase()
        if not targets:
            return {"status": "error", "message": "No targets found"}
        target_file = targets[0]

    proposal = await engine.propose_mutation(target_file, engine_team)
    if proposal:
        return {"status": "mutation_proposed", "proposal": proposal.model_dump()}
    return {"status": "error", "message": "Mutation generation failed"}


@app.get("/evolution/proposals")
async def list_genetic_proposals(
    data: Optional[str] = None, signature: Optional[str] = None
):
    """List all pending and verified genetic proposals (Phase 16)."""
    if data and signature:
        # Federation request verification
        from swarm_v2.core.federation import get_federation

        f = get_federation()
        if f:
            try:
                parsed_data = json.loads(data)
                if not f._verify_hmac(parsed_data, signature):
                    raise HTTPException(
                        status_code=401, detail="Invalid federation signature"
                    )
            except Exception:
                raise HTTPException(
                    status_code=401, detail="Failed to verify signature"
                )
    elif os.getenv("SWARM_FEDERATION_ENFORCE_AUTH", "false").lower() == "true":
        raise HTTPException(
            status_code=401, detail="Federation authentication required"
        )

    from swarm_v2.core.mutation_engine import get_mutation_engine

    engine = get_mutation_engine()
    return {"proposals": engine.get_proposals()}


@app.post("/evolution/proposals/mock")
async def add_mock_proposal(proposal: Dict[str, Any]):
    """Add a mock genetic proposal directly to the mutation engine database for federated testing (Phase 17)."""
    from swarm_v2.core.mutation_engine import GeneticProposal, get_mutation_engine

    engine = get_mutation_engine()
    gp = GeneticProposal(
        proposal_id=proposal["proposal_id"],
        target_file=proposal["target_file"],
        mutation_type=proposal["mutation_type"],
        description=proposal["description"],
        original_code=proposal["original_code"],
        mutated_code=proposal["mutated_code"],
        status=proposal.get("status", "passed"),
        score=proposal.get("score", 0.99),
    )
    engine.proposals[gp.proposal_id] = gp
    return {"status": "success", "proposal_id": gp.proposal_id}


@app.get("/evolution/proposals/synced")
async def list_synced_proposals():
    """Retrieve all synchronized remote proposals from the persistent global memory (Phase 17)."""
    import json

    from swarm_v2.core.global_memory import get_global_memory

    memory = get_global_memory()
    synced = []

    # Extract integrated proposals from ChromaDB/metadata
    for eid, meta in memory.entries_metadata.items():
        if (
            meta.get("author") == "federation"
            and meta.get("memory_type") == "remote_proposal"
        ):
            sig = meta.get("holo_signature")
            if sig in memory.context_vault:
                try:
                    vault_data = json.loads(memory.context_vault[sig])
                    content_dict = json.loads(vault_data["content"])
                    synced.append(content_dict)
                except Exception:
                    pass
    return {"synced": synced}


@app.post("/evolution/verify/{proposal_id}")
async def verify_genetic_proposal(proposal_id: str):
    """Verify a specific genetic proposal in the sandbox (Phase 16)."""
    from swarm_v2.core.evolutionary_sandbox import get_evolutionary_sandbox
    from swarm_v2.core.mutation_engine import get_mutation_engine

    engine = get_mutation_engine()
    sandbox = get_evolutionary_sandbox()

    proposal = engine.proposals.get(proposal_id)
    if not proposal:
        raise HTTPException(status_code=404, detail="Proposal not found")

    passed = await sandbox.verify_mutation(proposal)
    return {
        "status": "verified" if passed else "failed",
        "proposal": proposal.model_dump(),
    }


@app.post("/evolution/integrate/{proposal_id}")
async def integrate_genetic_proposal(proposal_id: str):
    """Integrate a successfully verified genetic proposal into the codebase (Phase 16)."""
    from swarm_v2.core.mutation_engine import get_mutation_engine

    engine = get_mutation_engine()

    proposal = engine.proposals.get(proposal_id)
    if not proposal:
        raise HTTPException(status_code=404, detail="Proposal not found")

    if proposal.status != "passed":
        raise HTTPException(
            status_code=400,
            detail="Only successfully verified proposals can be integrated",
        )

    try:
        with open(proposal.target_file, "w", encoding="utf-8") as f:
            f.write(proposal.mutated_code)

        proposal.status = "integrated"
        return {"status": "integrated", "proposal": proposal.model_dump()}
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Integration failed: {str(e)}")


