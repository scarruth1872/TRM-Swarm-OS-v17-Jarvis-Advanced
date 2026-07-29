# Technical Specification: Diagram Generation Skill (DGS)

## 1. Overview
The Diagram Generation Skill (DGS) is an autonomous cognitive module designed to synthesize visual architectural representations from textual technical specifications. It bridges the gap between documentation and visualization by implementing a "Specification-to-Diagram" (S2D) pipeline.

## 2. Functional Requirements
- **Pattern Detection**: Automatically detect if the spec describes a process (Flowchart), an interaction (Sequence), a structure (Class/ERD), or a lifecycle (State).
- **Entity Extraction**: Identify Actors, Components, Databases, and APIs.
- **Relationship Mapping**: Extract verbs and connectors (e.g., "calls", "updates", "triggers", "inherits").
- **Syntax Generation**: Produce production-ready Mermaid.js code.

## 3. Technical Architecture
### 3.1 Pipeline
1. **Ingestion**: Accepts Markdown/JSON.
2. **Analysis**: Uses regex and keyword-based heuristic analysis to determine diagram type.
3. **Synthesis**: Maps extracted triplets `(Subject, Action, Object)` to Mermaid syntax.
4. **Verification**: Validates the generated string against Mermaid grammar rules.

## 4. Diagram Mapping Matrix
| Spec Keyword | Detected Pattern | Diagram Type |
| :--- | :--- | :--- |
| "Sequence", "Call flow", "Interaction" | Temporal Interaction | `sequenceDiagram` |
| "Schema", "Database", "Entity" | Data Relationship | `erDiagram` |
| "Workflow", "Step", "Decision" | Logic Flow | `graph TD` |
| "State", "Transition", "Status" | Lifecycle | `stateDiagram-v2` |
| "Hierarchy", "Composition" | Structural | `classDiagram` |

## 5. Constraints
- **Complexity Limit**: Maximum of 20 entities per diagram to maintain readability.
- **Format**: Output must be encapsulated in ```mermaid blocks.