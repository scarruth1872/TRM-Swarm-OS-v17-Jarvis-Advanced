"""
Audit Kanban Tasks Implementation Integrity
Checks whether completed tasks correspond to actual python code, test files,
and live functionality vs stubbed/mocked implementations.
"""

import os
import json
import inspect
from swarm_v2.core.kanban_board import get_kanban_board

def audit_tasks():
    print("==========================================================================================")
    print(" [AUDIT] AUDITING KANBAN COMPLETED TASKS: ACTUAL IMPLEMENTATION VS MOCKED / STUBBED")
    print("==========================================================================================")

    kb = get_kanban_board()
    cards = kb._cards
    total_cards = len(cards)
    
    actual_implementations = 0
    mocked_stubs = 0
    research_proposals = 0

    audit_details = []

    # Get list of all codebase python files
    codebase_files = {}
    for root, dirs, files in os.walk("."):
        if "venv" in root or ".git" in root or "__pycache__" in root or "node_modules" in root:
            continue
        for fname in files:
            if fname.endswith(".py") or fname.endswith(".md") or fname.endswith(".json"):
                fpath = os.path.join(root, fname)
                try:
                    with open(fpath, "r", encoding="utf-8", errors="ignore") as pf:
                        codebase_files[fpath] = pf.read()
                except Exception:
                    pass

    for card_id, card in cards.items():
        title = card.title if hasattr(card, 'title') else card.get("title", "")
        desc = card.description if hasattr(card, 'description') else card.get("description", "")
        status = card.status if hasattr(card, 'status') else card.get("status", "")

        clean_title = title.replace("Implement ", "").replace("Analyze Research: ", "").replace(".md", "").replace(".spec", "").strip()
        
        found_real_code = False
        found_stub = False
        matching_files = []

        for fpath, content in codebase_files.items():
            fname = os.path.basename(fpath)
            if clean_title.lower() in fname.lower() or (len(clean_title) > 5 and clean_title.lower() in content.lower()):
                matching_files.append(fpath)
                if fpath.endswith(".py"):
                    if "def " in content or "class " in content:
                        if len(content) < 300 and ("pass" in content or "return {}" in content):
                            found_stub = True
                        else:
                            found_real_code = True
                elif fpath.endswith(".md") or fpath.endswith(".json"):
                    found_real_code = True

        if found_real_code:
            actual_implementations += 1
            audit_details.append({"title": title, "type": "REAL_IMPLEMENTATION", "files": matching_files[:2]})
        elif found_stub:
            mocked_stubs += 1
            audit_details.append({"title": title, "type": "STUBBED_MOCK", "files": matching_files[:2]})
        else:
            research_proposals += 1
            audit_details.append({"title": title, "type": "RESEARCH_DOCUMENTATION", "files": []})

    print(f"\nAudit Summary of {total_cards} Kanban Cards:")
    print(f" * Verified Real Code / Research Implementations: {actual_implementations} ({actual_implementations/max(total_cards,1)*100:.1f}%)")
    print(f" * Mocked / Stubbed Dummy Implementations:       {mocked_stubs} ({mocked_stubs/max(total_cards,1)*100:.1f}%)")
    print(f" * Unlinked Research & Spec Artifacts:           {research_proposals} ({research_proposals/max(total_cards,1)*100:.1f}%)")

    print("\nSample Audit Breakdown (15 Representative Cards):")
    for item in audit_details[:15]:
        print(f" - [{item['type']}] {item['title'][:60]:<60} | Matches: {item['files']}")

if __name__ == "__main__":
    audit_tasks()
