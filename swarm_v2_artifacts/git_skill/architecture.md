# GitSkill Implementation Structure

/git_skill
├── core/
│   ├── dispatcher.py       # Routes actions to strategies
│   ├── executor.py         # Low-level shell/libgit2 wrapper
│   └── validator.py        # Input schema and state verification
├── strategies/
│   ├── sync_strategy.py    # Pull/Push/Rebase logic
│   ├── branch_strategy.py  # Branch creation/deletion/switching
│   └── commit_strategy.py  # Staging and committing logic
├── utils/
│   ├── logger.py           # Telemetry and diagnostic logging
│   └── git_helpers.py      # Common git command shortcuts
└── main.py                 # Entry point for Swarm OS integration