# FuzzTestSkill Directory Structure

/fuzz_test_skill
├── /core
│   ├── mutation_engine.py      # Bit-flipping and havoc logic
│   ├── orchestrator.py         # Process management and scheduling
│   └── instrumentation.py      # LLVM/KCOV integration hooks
├── /analysis
│   ├── triager.py              # Crash deduplication and stack analysis
│   └── coverage_mapper.py      # Edge and block tracking
├── /sandbox
│   ├── docker_wrapper.py       # Containerized execution environment
│   └── seccomp_profile.json    # System call restrictions
├── /utils
│   ├── logger.py               # Telemetry and diagnostic logging
│   └── corpus_manager.py       # Seed queue and persistence
└── main.py                     # CLI Entry point