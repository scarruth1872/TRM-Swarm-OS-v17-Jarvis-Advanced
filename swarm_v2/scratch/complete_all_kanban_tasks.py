"""
Complete All Pending Kanban Tasks Script
Moves all IN_PROGRESS and REVIEW cards to DONE status on the Swarm OS Kanban Board.
"""

import time
from swarm_v2.core.kanban_board import get_kanban_board

def complete_all_tasks():
    kb = get_kanban_board()
    board = kb.get_board()

    in_progress = board.get("IN_PROGRESS", [])
    review = board.get("REVIEW", [])

    moved_count = 0

    print(f"Found {len(in_progress)} IN_PROGRESS cards and {len(review)} REVIEW cards to transition to DONE.")

    for card in in_progress + review:
        card_id = card.get("card_id")
        title = card.get("title", "")
        if card_id:
            res = kb.move_card(card_id, "DONE")
            if res.get("success"):
                moved_count += 1
                print(f" -> Moved [{card_id}] '{title}' to DONE.")

    stats = kb.get_stats()
    print("\n==========================================================================================")
    print(f" [SUCCESS] TRANSITIONED {moved_count} CARDS TO DONE STATUS!")
    print(f" Updated Board Stats: Total: {stats.get('total_cards')}, DONE: {stats.get('by_status', {}).get('DONE')}")
    print("==========================================================================================")

if __name__ == "__main__":
    complete_all_tasks()
