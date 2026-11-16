#!/usr/bin/env python3
"""Swarm dashboard audit — check every page for mocked/missing modules."""
import urllib.request, json, time
from pathlib import Path
from collections import defaultdict

PAGES = {
    'Overview': ['/infrastructure/status'],
    'Swarm Chat': ['/swarm/experts'],
    'Neural Mesh': ['/mesh/topology', '/infrastructure/nodes'],
    'Infrastructure': ['/infrastructure/status', '/infrastructure/nodes'],
    'Artifacts': ['/artifacts'],
    'Testing': ['/testing/stats'],
    'Telemetry': ['/swarm/telemetry'],
    'Orchestrator': ['/swarm/orchestrator/stats'],
    'Memory': ['/memory/stats'],
    'Healing': ['/swarm/healing/status'],
    'Forge': ['/swarm/forge/status'],
    'Skills': ['/skills/matrix'],
    'Experts': ['/swarm/experts'],
}

print('DASHBOARD PAGE AUDIT — Mocked vs Real Data')
print('=' * 60)

results = {}
for page, endpoints in PAGES.items():
    page_result = {'status': 'ok', 'issues': [], 'endpoints': {}}
    
    for ep in endpoints:
        try:
            req = urllib.request.Request('http://127.0.0.1:8021' + ep)
            resp = urllib.request.urlopen(req, timeout=5)
            data = json.loads(resp.read())
            
            issues = []
            if isinstance(data, list):
                count = len(data)
                if count == 0:
                    issues.append('EMPTY_ARRAY')
            elif isinstance(data, dict):
                count = len(data)
                response_text = str(data).lower()
                if 'mock' in response_text or 'placeholder' in response_text:
                    issues.append('MOCKED_DATA')
                if 'received:' in response_text and len(response_text) < 200:
                    issues.append('STATIC_RESPONSE')
                if '"status"' in str(data) and data.get('status') == 'error':
                    issues.append('ERROR_STATE')
                if isinstance(data.get('nodes', data.get('experts', data.get('artifacts'))), list):
                    arr = data.get('nodes') or data.get('experts') or data.get('artifacts')
                    if len(arr) == 0:
                        issues.append('ZERO_ITEMS')
            else:
                count = 0
                issues.append('INVALID_TYPE')
            
            page_result['endpoints'][ep] = {
                'status_code': resp.status,
                'item_count': count,
                'issues': issues,
            }
            if issues:
                iss_str = ', '.join(issues)
                page_result['issues'].append(ep + ': ' + iss_str)
                
        except Exception as e:
            page_result['endpoints'][ep] = {'error': str(e)[:80]}
            page_result['issues'].append(ep + ': ' + str(e)[:80])
    
    clean_eps = sum(1 for ep in page_result['endpoints'].values() if not ep.get('issues', []) and 'error' not in ep)
    page_result['health'] = str(clean_eps) + '/' + str(len(endpoints)) + ' clean'
    
    n_issues = len(page_result['issues'])
    if n_issues == 0:
        icon = 'CLEAN'
    elif n_issues == 1:
        icon = 'WARN'
    else:
        icon = 'ISSUES'
    
    results[page] = page_result
    print('  [' + icon + '] ' + page + ' ' + page_result['health'])
    for iss in page_result['issues']:
        print('         -> ' + iss)

print('')
print('SUMMARY')
clean = sum(1 for r in results.values() if not r['issues'])
warn = sum(1 for r in results.values() if len(r['issues']) == 1)
bad = sum(1 for r in results.values() if len(r['issues']) > 1)
print('  Clean:    ' + str(clean) + '/' + str(len(results)))
print('  Warnings: ' + str(warn))
print('  Issues:   ' + str(bad))

# Dispatch to Shield for deeper audit
audit_payload = json.dumps({
    'role': 'Shield',
    'message': 'audit dashboard pages for mocked data: ' + str(len(results)) + ' pages checked, ' + str(clean) + ' clean, ' + str(bad) + ' with issues. List each issue.',
    'sender': 'autonomous_audit'
}).encode()

try:
    req = urllib.request.Request('http://127.0.0.1:8021/swarm/chat',
        data=audit_payload, headers={'Content-Type': 'application/json'}, method='POST')
    resp = urllib.request.urlopen(req, timeout=10)
    shield_report = json.loads(resp.read())
    print('\nShield Audit: [' + shield_report.get('engine','?') + '] ' + shield_report.get('response','?')[:300])
except Exception as e:
    shield_report = {'response': str(e)}
    print('\nShield Audit: ERROR - ' + str(e)[:80])

Path('swarm_v2_artifacts/dashboard_audit.json').write_text(json.dumps({
    'timestamp': time.strftime('%Y-%m-%dT%H:%M:%S'),
    'pages_checked': len(results),
    'results': {k: {'health': v['health'], 'issues': v['issues']} for k,v in results.items()},
    'shield_report': shield_report.get('response', ''),
}, indent=2))
print('\nSaved: swarm_v2_artifacts/dashboard_audit.json')
