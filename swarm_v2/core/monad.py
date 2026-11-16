#!/usr/bin/env python3
"""Monad — Experience Gatherer for TRM Platform.
Gathers all experiences, training history, benchmarks, and model states
into a composable monad structure for chained operations."""

import json, time, numpy as np
from pathlib import Path
from typing import Dict, List, Any, Callable, Optional, Union
from dataclasses import dataclass, field

@dataclass
class Experience:
    """Single platform experience — a training run, benchmark, or model event."""
    timestamp: str
    category: str          # training, benchmark, architecture, deployment
    agent: Optional[str]   # which agent (None for platform-wide)
    metric: str            # what was measured
    value: Any
    metadata: Dict = field(default_factory=dict)

class Monad:
    """Monadic experience gatherer — wraps platform state for chained operations.
    
    Operations:
      map(fn) — transform experiences
      filter(pred) — filter experiences  
      flatMap(fn) — chain to new monad
      reduce(fn, initial) — aggregate experiences
      gather() — collect all platform experiences
      bind(fn) — monadic bind (>>=)
    """
    
    def __init__(self, experiences: List[Experience] = None):
        self._experiences = experiences or []
        self._context = {}
    
    # ─── Monad Core Operations ───
    
    def bind(self, fn: Callable[['Monad'], 'Monad']) -> 'Monad':
        """Monadic bind (>>=) — chain operations over experiences."""
        return fn(self)
    
    def fmap(self, fn: Callable[[Experience], Experience]) -> 'Monad':
        """Functor map — transform each experience."""
        return Monad([fn(e) for e in self._experiences])
    
    def filter(self, pred: Callable[[Experience], bool]) -> 'Monad':
        """Filter experiences by predicate."""
        return Monad([e for e in self._experiences if pred(e)])
    
    def flatMap(self, fn: Callable[[Experience], List[Experience]]) -> 'Monad':
        """FlatMap — each experience yields multiple."""
        result = []
        for e in self._experiences:
            result.extend(fn(e))
        return Monad(result)
    
    def reduce(self, fn: Callable[[Any, Experience], Any], initial: Any) -> Any:
        """Fold/reduce over experiences."""
        acc = initial
        for e in self._experiences:
            acc = fn(acc, e)
        return acc
    
    def pure(self, value: Any) -> 'Monad':
        """Lift a value into the Monad."""
        return Monad([Experience(
            timestamp=time.strftime('%Y-%m-%dT%H:%M:%S'),
            category='lifted', agent=None, metric='pure', value=value
        )])
    
    # ─── Platform Experience Gatherers ───
    
    def gather_training(self) -> 'Monad':
        """Gather all training experiences."""
        training_dir = Path('swarm_v2_global_memory')
        exps = []
        
        # LoRA adapters
        lora_dir = training_dir / 'lora_adapters'
        if lora_dir.exists():
            for f in lora_dir.glob('*.pt'):
                exps.append(Experience(
                    timestamp=time.ctime(f.stat().st_mtime),
                    category='training',
                    agent=f.stem.replace('_v2_v18', ''),
                    metric='lora_adapter_loss',
                    value={'file': str(f), 'size_kb': f.stat().st_size / 1024},
                ))
        
        # TRM agents
        trm_dir = training_dir / 'trm_7m_agents'
        if trm_dir.exists():
            for f in trm_dir.glob('*.pt'):
                exps.append(Experience(
                    timestamp=time.ctime(f.stat().st_mtime),
                    category='training',
                    agent=f.stem.replace('_50k', '').replace('_mamba3_50k', '').replace('_bonsai_instella', ''),
                    metric='agent_model',
                    value={'file': str(f), 'size_kb': f.stat().st_size / 1024},
                ))
        
        # Microkernel experts
        me_dir = training_dir / 'microkernel_experts'
        if me_dir.exists():
            for f in me_dir.glob('*.pt'):
                exps.append(Experience(
                    timestamp=time.ctime(f.stat().st_mtime),
                    category='training',
                    agent=f.stem.split('_')[0],
                    metric='microkernel_expert',
                    value={'file': str(f), 'size_kb': f.stat().st_size / 1024},
                ))
        
        # Curriculum
        curriculum_dir = training_dir / 'curriculum_50k'
        if curriculum_dir.exists():
            for f in curriculum_dir.glob('*.jsonl'):
                lines = sum(1 for _ in open(f))
                exps.append(Experience(
                    timestamp=time.ctime(f.stat().st_mtime),
                    category='training',
                    agent=f.stem.replace('_50k', ''),
                    metric='curriculum_pairs',
                    value=lines,
                ))
        
        # Bonsai weights
        for wf in ['bonsai_ternary_weights.json', 'bonsai_code_weights.json']:
            p = training_dir / wf
            if p.exists():
                data = json.loads(p.read_text())
                exps.append(Experience(
                    timestamp=time.ctime(p.stat().st_mtime),
                    category='training',
                    agent='Bonsai',
                    metric='ternary_loss' if 'ternary' in wf else 'code_loss',
                    value=data.get('loss', 0),
                    metadata={'file': str(p), 'threshold': data.get('threshold', 0)},
                ))
        
        self._experiences.extend(exps)
        return self
    
    def gather_benchmarks(self) -> 'Monad':
        """Gather all benchmark results."""
        artifacts_dir = Path('swarm_v2_artifacts')
        exps = []
        
        for f in artifacts_dir.glob('*.json'):
            try:
                data = json.loads(f.read_text())
                if 'time' in f.name or 'benchmark' in f.name or 'audit' in f.name:
                    exps.append(Experience(
                        timestamp=data.get('timestamp', 'unknown'),
                        category='benchmark',
                        agent='Platform',
                        metric=f.stem,
                        value=data,
                        metadata={'file': str(f)},
                    ))
            except:
                pass
        
        self._experiences.extend(exps)
        return self
    
    def gather_models(self) -> 'Monad':
        """Gather all model details."""
        exps = []
        
        # Instella MoE
        instella_path = Path('D:/huggingface_cache/hub/models--amd--Instella-MoE-16B-A3B-Think')
        if instella_path.exists():
            size_gb = sum(f.stat().st_size for f in instella_path.rglob('*.safetensors') if f.is_file()) / 1e9
            exps.append(Experience(
                timestamp='2026-08-05',
                category='model',
                agent='Instella',
                metric='instella_moe',
                value={'size_gb': size_gb, 'experts': 64, 'params': '16B', 'active': '2.8B'},
            ))
        
        # Bonsai
        exps.append(Experience(
            timestamp='2026-08-05',
            category='model',
            agent='Bonsai',
            metric='bonsai_27b',
            value={'size_gb': 3.8, 'type': 'ternary_k3', 'params': '27B', 'precision': '1.58-bit'},
        ))
        
        self._experiences.extend(exps)
        return self
    
    def gather_commits(self) -> 'Monad':
        """Gather git commit history."""
        import subprocess
        try:
            r = subprocess.run(['git', 'log', '--oneline', '--since=2026-08-01'], 
                             capture_output=True, text=True, cwd='.', timeout=5)
            commits = r.stdout.strip().split('\n')
            for c in commits:
                if c:
                    parts = c.split(' ', 1)
                    exps = [Experience(
                        timestamp='2026-08-05',
                        category='commit',
                        agent='Platform',
                        metric='commit_hash',
                        value=parts[0],
                        metadata={'message': parts[1] if len(parts) > 1 else ''},
                    )]
                    self._experiences.extend(exps)
        except:
            pass
        return self
    
    def gather_all(self) -> 'Monad':
        """Gather all platform experiences."""
        return self.bind(lambda m: m.gather_training()
                        .bind(lambda m2: m2.gather_benchmarks())
                        .bind(lambda m3: m3.gather_models())
                        .bind(lambda m4: m4.gather_commits()))
    
    # ─── Analysis Operations ───
    
    def summarize(self) -> Dict[str, Any]:
        """Produce a summary of all gathered experiences."""
        categories = {}
        for e in self._experiences:
            if e.category not in categories:
                categories[e.category] = []
            categories[e.category].append(e)
        
        summary = {
            'total_experiences': len(self._experiences),
            'categories': {cat: len(exps) for cat, exps in categories.items()},
            'agents': {},
            'timestamp': time.strftime('%Y-%m-%dT%H:%M:%S'),
        }
        
        # Per-agent stats
        for e in self._experiences:
            if e.agent and e.agent not in summary['agents']:
                summary['agents'][e.agent] = {'count': 0, 'metrics': {}}
            if e.agent:
                summary['agents'][e.agent]['count'] += 1
                if e.metric not in summary['agents'][e.agent]['metrics']:
                    summary['agents'][e.agent]['metrics'][e.metric] = []
                summary['agents'][e.agent]['metrics'][e.metric].append(e.value)
        
        return summary
    
    def to_json(self, path: str = None) -> str:
        """Serialize the monad experiences to JSON."""
        data = {
            'experiences': [
                {
                    'timestamp': e.timestamp,
                    'category': e.category,
                    'agent': e.agent,
                    'metric': e.metric,
                    'value': e.value,
                    'metadata': e.metadata,
                }
                for e in self._experiences
            ],
            'summary': self.summarize(),
        }
        
        if path:
            Path(path).write_text(json.dumps(data, indent=2))
        
        return json.dumps(data, indent=2)
    
    def __len__(self):
        return len(self._experiences)
    
    def __repr__(self):
        return f'Monad({len(self._experiences)} experiences)'


# ─── Activation ───

def activate_monad() -> Monad:
    """Activate the Monad experience gatherer — collects all platform experiences."""
    print('MONAD ACTIVATION — Gathering All Platform Experiences')
    print('=' * 60)
    
    monad = Monad()
    
    # Gather through monadic chain
    monad = monad.bind(lambda m: m.gather_training())
    monad = monad.bind(lambda m: m.gather_benchmarks())
    monad = monad.bind(lambda m: m.gather_models())
    monad = monad.bind(lambda m: m.gather_commits())
    
    summary = monad.summarize()
    
    print(f'Experiences gathered: {len(monad)}')
    print(f'Categories: {summary["categories"]}')
    print(f'Agents: {len(summary["agents"])}')
    print()
    
    # Per-agent breakdown
    for agent in sorted(summary['agents']):
        a = summary['agents'][agent]
        print(f'  {agent:15s} {a["count"]:3d} experiences — {list(a["metrics"].keys())[:3]}')
    
    # Save
    path = Path('swarm_v2_artifacts/monad_experiences.json')
    json_str = monad.to_json(str(path))
    print(f'\nSaved: {path}')
    print(f'Size: {len(json_str):,} bytes')
    
    # Chain: compute stats
    stats = {
        'total': len(monad),
        'training_count': len(monad.filter(lambda e: e.category == 'training')),
        'benchmark_count': len(monad.filter(lambda e: e.category == 'benchmark')),
        'model_count': len(monad.filter(lambda e: e.category == 'model')),
        'commit_count': len(monad.filter(lambda e: e.category == 'commit')),
    }
    print(f'\nMonad Stats: {stats}')
    
    return monad

if __name__ == '__main__':
    m = activate_monad()
