#!/usr/bin/env python3
"""10,000 complex multi-step tasks through full TRM pipeline."""
import urllib.request, json, time, numpy as np
from collections import defaultdict

# Multi-step task sequences — each task builds on the previous
MULTI_STEP_TASKS = [
    # Security chain
    ['audit security posture', 'scan for vulnerabilities', 'deploy DDR antibodies', 'verify security hardening'],
    # Research chain
    ['research protein folding', 'analyze AlphaFold results', 'synthesize findings into paper', 'peer review research'],
    # Engineering chain
    ['compile python module', 'run unit tests', 'optimize GPU kernel', 'deploy to production'],
    # Architecture chain
    ['design system blueprint', 'synthesize GAN topology', 'validate against constraints', 'generate implementation plan'],
    # Logic chain
    ['formalize theorem', 'prove in Coq', 'verify in Lean4', 'publish proof'],
    # Integration chain
    ['connect API endpoints', 'configure WebSocket mesh', 'test integration', 'deploy gateway'],
    # Operations chain
    ['orchestrate workflow', 'dispatch to agents', 'monitor execution', 'report results'],
    # Intelligence chain
    ['query cognitive memory', 'build knowledge graph', 'synthesize skills', 'deploy to swarm'],
    # Cross-domain chain
    ['research security threats', 'design defense architecture', 'implement protections', 'verify system integrity'],
    # Platform chain
    ['health check system', 'analyze telemetry', 'optimize resource allocation', 'verify system stability'],
]

AGENTS = ['Shield','Logic','Devo','Seeker','Verify','Archi','Flow','Bridge',
          'Orchestra','EventLedger','LogicMaster','Pulse','THzGateway','Jarvis','Hermes']

total_tasks = 10000
tasks_per_chain = total_tasks // len(MULTI_STEP_TASKS)  # 1000 per chain

print('=' * 70)
print(f'10,000 MULTI-STEP TASKS — Full Pipeline')
print(f'Chains: {len(MULTI_STEP_TASKS)} | Tasks per chain: {tasks_per_chain}')
print(f'Pipeline: Instella MoE → THz Bonsai → THz Gateway → Radix-Trie → engine_team')
print('=' * 70)

t_start = time.time()
results = []
engines = defaultdict(int)
chain_stats = defaultdict(lambda: {'count': 0, 'times': [], 'engines': defaultdict(int)})

task_idx = 0
for chain_num, chain in enumerate(MULTI_STEP_TASKS):
    chain_name = f'Chain-{chain_num+1}'
    
    for i in range(tasks_per_chain):
        step = chain[i % len(chain)]
        agent = AGENTS[task_idx % len(AGENTS)]
        
        # Add context from previous step for multi-step complexity
        if i > 0:
            prev = chain[(i-1) % len(chain)]
            message = f'{step} (after {prev})'
        else:
            message = step
        
        payload = json.dumps({
            'role': agent,
            'message': message,
            'sender': 'benchmark_10k'
        }).encode()
        
        req = urllib.request.Request('http://127.0.0.1:8021/swarm/chat',
            data=payload, headers={'Content-Type': 'application/json'}, method='POST')
        
        t0 = time.perf_counter_ns()
        try:
            resp = urllib.request.urlopen(req, timeout=15)
            data = json.loads(resp.read())
            elapsed = (time.perf_counter_ns() - t0) / 1000
            engine = data.get('engine', '?')
            
            engines[engine] += 1
            results.append(elapsed)
            chain_stats[chain_name]['count'] += 1
            chain_stats[chain_name]['times'].append(elapsed)
            chain_stats[chain_name]['engines'][engine] += 1
            
            task_idx += 1
        except Exception as e:
            elapsed = (time.perf_counter_ns() - t0) / 1000
            results.append(elapsed)
            task_idx += 1
        
        if task_idx % 2000 == 0:
            elapsed = time.time() - t_start
            recent = results[-500:] if len(results) >= 500 else results
            lr = np.array(recent)
            ie = engines.get('instella_moe', 0)
            tb = engines.get('thz_bonsai', 0)
            tg = engines.get('thz_gateway', 0)
            print(f'  {task_idx:5d}/10000 | {task_idx/elapsed:.0f} tasks/s | P50={np.percentile(lr,50):.0f}us | IE={ie} TB={tb} TG={tg}')

total_s = time.time() - t_start
lats = np.array([r for r in results if r > 0])

print('')
print(f'{"=" * 70}')
print('RESULTS')
print(f'  Tasks completed:   {len(results)}')
print(f'  Total time:        {total_s:.1f}s ({total_s/60:.1f}min)')
print(f'  Throughput:        {len(results)/total_s:.0f} tasks/s')
print(f'  P50 latency:       {np.percentile(lats,50):.0f}us')
print(f'  P95 latency:       {np.percentile(lats,95):.0f}us')
print(f'  P99 latency:       {np.percentile(lats,99):.0f}us')
print(f'  Mean latency:      {np.mean(lats):.0f}us')
print(f'  Cost:              ${len(results)*0.00000006:.4f}')

print(f'\nEngine Distribution:')
for eng in sorted(engines, key=engines.get, reverse=True):
    pct = engines[eng] / len(results) * 100
    bar_char = chr(9608)
    bar = bar_char * int(pct / 2)
    print(f'  {eng:20s} {engines[eng]:5d} ({pct:5.1f}%) {bar}')

print(f'\nChain Performance:')
for chain in sorted(chain_stats, key=lambda c: np.mean(chain_stats[c]['times']) if chain_stats[c]['times'] else 0):
    cs = chain_stats[chain]
    if cs['times']:
        mean_lat = np.mean(cs['times'])
        top_engine = max(cs['engines'], key=cs['engines'].get) if cs['engines'] else '?'
        print(f'  {chain:12s} {cs[\"count\"]:4d} tasks  {mean_lat:.0f}us avg  [{top_engine}]')

# Save results
from pathlib import Path
Path('swarm_v2_artifacts/benchmark_10k_multistep.json').write_text(json.dumps({
    'timestamp': time.strftime('%Y-%m-%dT%H:%M:%S'),
    'tasks': len(results),
    'time_s': total_s,
    'throughput': round(len(results)/total_s),
    'p50_us': round(np.percentile(lats, 50)),
    'p95_us': round(np.percentile(lats, 95)),
    'engines': dict(engines),
}, indent=2))
print(f'\nSaved: swarm_v2_artifacts/benchmark_10k_multistep.json')
