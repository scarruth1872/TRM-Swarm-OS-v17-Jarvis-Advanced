# VulnScanSkill Directory Structure

```text
vuln_scan_skill/
├── core/
│   ├── orchestrator.py       # Main execution logic & engine lifecycle
│   ├── normalizer.py         # Schema mapping and data cleaning
│   └── risk_engine.py        # Scoring algorithms and CVSS mapping
├── engines/
│   ├── base_engine.py        # Abstract Base Class for all scanners
│   ├── sast_wrappers/        # Semgrep, SonarQube, Bandit
│   └── dast_wrappers/        # OWASP ZAP, Nuclei, BurpSuite
├── utils/
│   ├── validator.py          # Target and input validation
│   └── logger.py             # Telemetry and audit logging
├── schemas/
│   ├── input_schema.json     # JSON Schema for input validation
│   └── output_schema.json    # JSON Schema for output consistency
└── tests/
    ├── integration/          # End-to-end scan tests
    └── unit/                 # Logic tests for risk scoring
```