"""
Register LLM & Multimodal AI Transformation Plans into Global Memory & SKG
"""

import sys

def register_transformation_plans():
    print("--- 1. Registering Transformation Plans in Global Memory ---")
    from swarm_v2.core.global_memory import get_global_memory
    gm = get_global_memory()

    # LLM Plan Memory
    id_llm = gm.contribute(
        content=(
            "TRANSFORMATION PLAN RECORD: Swarm OS Large Language Model (LLM) Integration Plan. "
            "Defines new Cognitive Layer (LLM Core Service, Prompt Management, Knowledge Graph, Reasoning Engine), "
            "Enhanced Orchestration (Intelligent Scheduler, NLI), and Enhanced Execution (Autonomous Code Gen, Self-Healing). "
            "File: swarm_v2_artifacts/transformation_plan_llm_integration.md"
        ),
        author="Archi",
        author_role="Generative Architect",
        memory_type="transformation_plan"
    )
    print(f"[SUCCESS] LLM Plan registered in Global Memory.")

    # Multimodal Plan Memory
    id_mm = gm.contribute(
        content=(
            "TRANSFORMATION PLAN RECORD: Swarm OS Multimodal AI Integration Plan. "
            "Defines new Perception & Fusion Layer (Multimodal Ingestion, Feature Extraction, Multimodal Fusion Engine), "
            "Enhanced Cognitive Layer (Multimodal Reasoning, Predictive Analytics), and Environment-Aware Orchestration. "
            "File: swarm_v2_artifacts/transformation_plan_multimodal_ai_integration.md"
        ),
        author="Archi",
        author_role="Generative Architect",
        memory_type="transformation_plan"
    )
    print(f"[SUCCESS] Multimodal Plan registered in Global Memory.")

    print("\n--- 2. Linking Transformation Plans into Semantic Knowledge Graph (SKG) ---")
    from swarm_v2.core.semantic_knowledge_graph import get_semantic_knowledge_graph, GraphNode, GraphEdge
    skg = get_semantic_knowledge_graph()

    llm_node = GraphNode(
        node_id="transformation_llm",
        node_type="transformation_plan",
        label="LLM Integration Plan",
        properties={
            "file": "swarm_v2_artifacts/transformation_plan_llm_integration.md",
            "phases": ["LLM Core Service", "Reasoning & Planning", "Autonomous Code Gen"]
        }
    )
    mm_node = GraphNode(
        node_id="transformation_multimodal",
        node_type="transformation_plan",
        label="Multimodal AI Integration Plan",
        properties={
            "file": "swarm_v2_artifacts/transformation_plan_multimodal_ai_integration.md",
            "phases": ["Multimodal Fusion Engine", "Perception & Fusion Layer", "Environment-Aware Orchestration"]
        }
    )

    skg.add_node(llm_node)
    skg.add_node(mm_node)
    skg.add_edge(GraphEdge(edge_id="e_archi_llm", source_id="archi", target_id="transformation_llm", relation_type="synthesized"))
    skg.add_edge(GraphEdge(edge_id="e_archi_mm", source_id="archi", target_id="transformation_multimodal", relation_type="synthesized"))

    print("[SUCCESS] SKG linked with LLM & Multimodal Transformation Plan nodes.")

if __name__ == "__main__":
    register_transformation_plans()
