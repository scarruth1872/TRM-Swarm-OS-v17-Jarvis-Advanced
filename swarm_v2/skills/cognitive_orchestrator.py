import asyncio
import logging
import json
from typing import Optional
from swarm_v2.skills.codebase_indexer import CodebaseIndexer
from swarm_v2.skills.learning_engine import get_learning_engine
from swarm_v2.experts.registry import get_expert_team

logger = logging.getLogger(__name__)

class CognitiveOrchestrator:
    """
    Background orchestrator loop that indexes codebase changes, uses
    deepseek to synthesize learned skill cards, and registers them.
    """

    def __init__(self, root_dir: str = ".", interval_seconds: int = 120):
        self.root_dir = root_dir
        self.interval = interval_seconds
        self.indexer = CodebaseIndexer(root_dir)
        self.engine = get_learning_engine()
        self._active = False
        from swarm_v2.skills.fs_watcher import FilesystemWatcher
        self.watcher = FilesystemWatcher(
            root_dir=root_dir,
            callback=self.on_file_changed,
            interval_seconds=10
        )

    async def start(self):
        """Starts the background cognitive learning cycle loop and file watcher."""
        if self._active:
            return
        self._active = True
        logger.info("[CognitiveLearning] Starting background cycle daemon...")
        asyncio.create_task(self._loop())
        await self.watcher.start()

    async def stop(self):
        self._active = False
        await self.watcher.stop()

    def on_file_changed(self, filepath: str):
        """Callback triggered by the filesystem watcher on modifications."""
        logger.info(f"[CognitiveLearning] Filesystem event triggered for: {filepath}")
        asyncio.create_task(self.process_file(filepath))

    async def process_file(self, filepath: str):
        """Processes, synthesizes, and hot-registers a single code file."""
        # Get LLM generation function from Devo (Lead Developer)
        devo = get_expert_team().get("Lead Developer")
        llm_fn = devo._llm_generate if devo else None
        
        if not llm_fn:
            logger.warning("[CognitiveLearning] No active Lead Developer LLM client found to compile file.")
            return

        from swarm_v2.skills.synthesis_pipeline import SkillSynthesisPipeline
        from swarm_v2.skills.validation_gate import ValidationGate

        pipeline = SkillSynthesisPipeline(llm_generate_fn=llm_fn)
        gate = ValidationGate()

        try:
            info = self.indexer.parse_source_file(filepath)
            if info.get("error"):
                return
                
            clean_filepath = info["filepath"].replace("/", "_").replace("\\", "_").replace(".", "_")
            skill_name = f"Learned_{clean_filepath}"
            
            # Skip if skill already exists
            if self.engine.get_skill(skill_name):
                return

            candidate = await pipeline.synthesize(info)
            if not candidate:
                return

            is_valid, reason = gate.validate(candidate)
            if not is_valid:
                logger.warning(f"[CognitiveLearning] Event validation failed for {filepath}: {reason}")
                return

            content_payload = (
                f"Summary: {candidate['summary']}\n"
                f"Preconditions: {', '.join(candidate['preconditions'])}\n"
                f"Postconditions: {', '.join(candidate['postconditions'])}\n"
                f"Instructions:\n{candidate['instructions']}"
            )

            await self.engine.learn_from_text(
                name=candidate["name"],
                content=content_payload,
                source=candidate["source"],
                llm_generate=None
            )
            logger.info(f"[CognitiveLearning] Hot-registered event-triggered codebase skill: {candidate['name']}")
        except Exception as e:
            logger.error(f"[CognitiveLearning] Event processing failed for {filepath}: {e}")

    async def _loop(self):
        # Allow server to complete startup initialization before starting first scan
        await asyncio.sleep(15)
        
        while self._active:
            try:
                logger.info("[CognitiveLearning] Running dynamic codebase learning cycle...")
                await self.run_learning_cycle()
            except Exception as e:
                logger.error(f"[CognitiveLearning] Loop execution error: {e}")
            await asyncio.sleep(self.interval)

    async def run_learning_cycle(self):
        """Crawls, extracts classes/methods, and compiles new skills via LLM."""
        parsed_files = self.indexer.scan_files(limit=5)
        
        # Get LLM generation function from Devo (Lead Developer)
        devo = get_expert_team().get("Lead Developer")
        llm_fn = devo._llm_generate if devo else None
        
        if not llm_fn:
            logger.warning("[CognitiveLearning] No active Lead Developer LLM client found to compile skills.")
            return

        for info in parsed_files:
            if info.get("error"):
                continue
                
            filepath = info["filepath"]
            clean_filepath = filepath.replace("/", "_").replace("\\", "_").replace(".", "_")
            skill_name = f"Learned_{clean_filepath}"
            
            # Skip if skill already exists
            if self.engine.get_skill(skill_name):
                continue

            await self.process_file(filepath)

_orchestrator: Optional[CognitiveOrchestrator] = None

def get_cognitive_orchestrator() -> CognitiveOrchestrator:
    global _orchestrator
    if _orchestrator is None:
        _orchestrator = CognitiveOrchestrator()
    return _orchestrator
