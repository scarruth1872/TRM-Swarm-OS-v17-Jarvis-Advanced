from pydantic import BaseModel
from typing import List, Dict, Any, Optional


class ChatRequest(BaseModel):
    role: str
    message: str
    sender: str = "user"


class BroadcastRequest(BaseModel):
    message: str
    sender: str = "user"


class SpawnRequest(BaseModel):
    parent_role: str
    sub_role: str
    task: str


class TaskPipelineRequest(BaseModel):
    task: str
    pipeline: List[str]


class ReviewRequest(BaseModel):
    filename: str
    action: str
    notes: str = ""
    reviewer: str = "user"


class BatchReviewRequest(BaseModel):
    category: Optional[str] = None
    status: Optional[str] = "pending"
    reviewer: str = "user"


class IntegrateRequest(BaseModel):
    filename: str
    target_subdir: str = ""


class BatchIntegrateRequest(BaseModel):
    filenames: List[str]
    target_subdir: str = ""


class TestRequest(BaseModel):
    filename: str


class DeployRequest(BaseModel):
    project_name: str = "Nexus Platform"
    deploy_target: str = ""


class LearnTextRequest(BaseModel):
    name: str
    content: str
    source: str = "user_input"


class LearnFileRequest(BaseModel):
    filepath: str


class UseSkillRequest(BaseModel):
    skill_name: str
    task: str
    target_role: str = ""


class GitHubAcquisitionRequest(BaseModel):
    topics: list[str]
    max_per_topic: int = 3


class SynthesizeRequest(BaseModel):
    skill_name: str
    use_llm: bool = True


class ForgeBuildRequest(BaseModel):
    tool_name: str


class MeshRouteRequest(BaseModel):
    task: str
    target_node_id: str = ""
    required_specialty: str = ""


class MeshAnnounceRequest(BaseModel):
    node_id: str
    name: str
    role: str
    status: str = "online"
    hardware: Dict[str, Any] = {}


class IntentPredictRequest(BaseModel):
    query: str


class IntentRecordRequest(BaseModel):
    query: str
    role: str
    specialties: List[str] = []


class GlobalMemoryContributeRequest(BaseModel):
    content: str
    author: str
    author_role: str
    memory_type: str = "knowledge"
    tags: list = []


class GlobalMemoryQueryRequest(BaseModel):
    query: str
    top_k: int = 5
    author_filter: str = ""
    type_filter: str = ""


class PMDelegationRequest(BaseModel):
    task: str
    sender_role: str


class SignedRequest(BaseModel):
    data: Dict[str, Any]
    signature: str


class HandshakeRequest(BaseModel):
    # Backward compatibility: old fields + new signed structure
    node_id: Optional[str] = None
    name: Optional[str] = None
    host: Optional[str] = None
    port: Optional[int] = None
    api_key: Optional[str] = None
    capabilities: List[str] = []
    timestamp: str = ""
    # New signed fields
    data: Optional[Dict[str, Any]] = None
    signature: Optional[str] = None


class MemorySyncRequest(BaseModel):
    source_node_id: Optional[str] = None
    source_name: Optional[str] = None
    local_memory_count: Optional[int] = None
    # New signed fields
    data: Optional[Dict[str, Any]] = None
    signature: Optional[str] = None


class SpatialFeedRequest(BaseModel):
    user_focus_score: float
    presence_detected: bool
    raw_csi_amplitude: float


class ForceReassignRequest(BaseModel):
    from_role: str
    to_role: str


class JarvisMemorySyncRequest(BaseModel):
    relations: List[Dict[str, Any]]
    nodes: List[Dict[str, Any]]


class FederationHandshakeRequest(BaseModel):
    node_id: str
    name: str
    host: str
    port: int
    api_key: str
    capabilities: List[str] = []
    timestamp: str


class FederationMemorySyncRequest(BaseModel):
    source_node_id: str
    source_name: str
    local_memory_count: int


class VerifyReasoningRequest(BaseModel):
    text: str
    auto_correct: bool = False


class ChatWithVerificationRequest(BaseModel):
    role: str
    message: str
    sender: str = "user"


class AnalyzePromptRequest(BaseModel):
    text: str
    context: Optional[Dict[str, Any]] = None


class GenerateTestsRequest(BaseModel):
    tool_name: str
    tool_data: Optional[Dict[str, Any]] = None


class RunTestsRequest(BaseModel):
    tool_name: str
    test_type: Optional[str] = None  # pytest_unit, pytest_api, playwright_e2e, integration


class CreateCardRequest(BaseModel):
    title: str
    description: str = ""
    assignee: str = ""
    priority: str = "medium"
    tags: List[str] = []


class MoveCardRequest(BaseModel):
    target_status: str


class SendMailboxMessage(BaseModel):
    from_agent: str
    to_agent: str
    body: str
    subject: str = ""


class DDRScanRequest(BaseModel):
    code: str
    filename: str = "untitled.py"


class SecretSetRequest(BaseModel):
    key: str
    value: str


class SecretDecryptRequest(BaseModel):
    key: str


class AntibodyAddRequest(BaseModel):
    error_type: str
    file_pattern: str
    line_pattern: str
    fix_description: str
    severity: str = "medium"


class CRERequest(BaseModel):
    problem: str
    roles: List[str] = ["Logic", "Archi", "Devo"]
