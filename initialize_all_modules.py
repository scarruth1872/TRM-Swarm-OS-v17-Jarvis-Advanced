#!/usr/bin/env python3
"""
Swarm OS v12 - Comprehensive Module Initialization & Verification Script

This script:
1. Verifies all required directories exist
2. Checks configuration files
3. Validates API connections (Ollama, external APIs)
4. Initializes all module states
5. Reports comprehensive status

Run this after initial setup to ensure all modules are properly configured.
"""

import json
import logging
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)
logger = logging.getLogger("ModuleInitializer")


# ANSI Color codes for terminal output
class Colors:
    HEADER = "\033[95m"
    OKBLUE = "\033[94m"
    OKCYAN = "\033[96m"
    OKGREEN = "\033[92m"
    WARNING = "\033[93m"
    FAIL = "\033[91m"
    ENDC = "\033[0m"
    BOLD = "\033[1m"
    UNDERLINE = "\033[4m"


def print_header(text: str):
    print(f"\n{Colors.HEADER}{Colors.BOLD}{'=' * 60}{Colors.ENDC}")
    print(f"{Colors.HEADER}{Colors.BOLD}{text.center(60)}{Colors.ENDC}")
    print(f"{Colors.HEADER}{Colors.BOLD}{'=' * 60}{Colors.ENDC}\n")


def print_success(text: str):
    print(f"{Colors.OKGREEN}✓{Colors.ENDC} {text}")


def print_warning(text: str):
    print(f"{Colors.WARNING}⚠{Colors.ENDC} {text}")


def print_error(text: str):
    print(f"{Colors.FAIL}✗{Colors.ENDC} {text}")


def print_info(text: str):
    print(f"{Colors.OKCYAN}ℹ{Colors.ENDC} {text}")


class ModuleInitializer:
    """Comprehensive module initialization and verification."""

    def __init__(self, base_dir: str = None):
        self.base_dir = Path(base_dir) if base_dir else Path(__file__).parent
        self.status_report = {
            "timestamp": datetime.now().isoformat(),
            "modules": {},
            "warnings": [],
            "errors": [],
            "recommendations": [],
        }

        # Required directories
        self.required_dirs = [
            "swarm_v2",
            "swarm_v2/core",
            "swarm_v2/experts",
            "swarm_v2/skills",
            "swarm_v2/mcp",
            "swarm_v2/cli",
            "swarm_v2/scripts",
            "swarm_v2_memory",
            "swarm_v2_memory/chroma_db",
            "swarm_v2_artifacts",
            "swarm_v2_artifacts/kanban",
            "swarm_v2_artifacts/mcp_tools",
            "tiny-recursive-weights",
            "dashboard",
            "dashboard/src",
            "dashboard/src/components",
            "models",
            "models/recursive_reasoning",
            "config",
            "config/arch",
            ".swarm",
            "utils",
        ]

        # Required configuration files
        self.required_files = [
            ".env",
            "requirements.txt",
            "swarm_v2/app_v2.py",
            "swarm_v2/core/llm_brain.py",
            "swarm_v2/core/llm_router.py",
            "swarm_v2/core/global_memory.py",
            "swarm_v2/core/agent_mesh.py",
            "swarm_v2/core/federation.py",
            "swarm_v2/core/kanban_board.py",
            "swarm_v2/core/artifact_pipeline.py",
            "swarm_v2/core/secrets_vault.py",
            "swarm_v2/mcp/bridge.py",
            "swarm_v2/experts/registry.py",
            "swarm_v2/skills/relationship_skill.py",
            "models/recursive_reasoning/trm.py",
        ]

        # Required Python packages
        self.required_packages = [
            "torch",
            "fastapi",
            "uvicorn",
            "pydantic",
            "chromadb",
            "ollama",
            "aiohttp",
            "numpy",
            "pyyaml",
            "requests",
        ]

    def check_directories(self) -> Dict:
        """Check if all required directories exist."""
        print_header("Checking Directories")
        results = {"total": len(self.required_dirs), "exists": 0, "missing": []}

        for dir_path in self.required_dirs:
            full_path = self.base_dir / dir_path
            if full_path.exists():
                print_success(f"Directory exists: {dir_path}")
                results["exists"] += 1
            else:
                print_warning(f"Creating missing directory: {dir_path}")
                try:
                    full_path.mkdir(parents=True, exist_ok=True)
                    print_success(f"Created: {dir_path}")
                    results["exists"] += 1
                except Exception as e:
                    print_error(f"Failed to create {dir_path}: {e}")
                    results["missing"].append(dir_path)
                    self.status_report["errors"].append(
                        f"Failed to create directory: {dir_path}"
                    )

        results["status"] = "ok" if not results["missing"] else "partial"
        self.status_report["modules"]["directories"] = results
        return results

    def check_configuration_files(self) -> Dict:
        """Check if all required configuration files exist."""
        print_header("Checking Configuration Files")
        results = {"total": len(self.required_files), "exists": 0, "missing": []}

        for file_path in self.required_files:
            full_path = self.base_dir / file_path
            if full_path.exists():
                print_success(f"File exists: {file_path}")
                results["exists"] += 1
            else:
                print_error(f"Missing file: {file_path}")
                results["missing"].append(file_path)
                self.status_report["warnings"].append(f"Missing file: {file_path}")

        # Check .env file specifically
        env_file = self.base_dir / ".env"
        if env_file.exists():
            with open(env_file, "r") as f:
                content = f.read()
                api_keys = {
                    "GEMINI_API_KEY": "GEMINI_API_KEY=" in content
                    and "your_gemini_api_key_here" not in content,
                    "OPENROUTER_API_KEY": "OPENROUTER_API_KEY=" in content
                    and "your_openrouter_api_key_here" not in content,
                    "DEEPSEEK_API_KEY": "DEEPSEEK_API_KEY=" in content
                    and "your_deepseek_api_key_here" not in content,
                }

                configured_keys = sum(1 for v in api_keys.values() if v)
                if configured_keys == 0:
                    print_warning(
                        ".env file exists but API keys are not configured (using placeholders)"
                    )
                    self.status_report["recommendations"].append(
                        "Configure API keys in .env file for full Multi-LLM functionality"
                    )
                elif configured_keys < 3:
                    print_warning(
                        f"Only {configured_keys}/3 external API keys configured"
                    )
                    self.status_report["recommendations"].append(
                        f"Configure remaining {3 - configured_keys} API keys in .env for full Multi-LLM functionality"
                    )
                else:
                    print_success("All external API keys are configured")

        results["status"] = "ok" if not results["missing"] else "partial"
        self.status_report["modules"]["configuration_files"] = results
        return results

    def check_python_packages(self) -> Dict:
        """Check if all required Python packages are installed."""
        print_header("Checking Python Packages")
        results = {"total": len(self.required_packages), "installed": 0, "missing": []}

        for package in self.required_packages:
            try:
                __import__(package)
                print_success(f"Package installed: {package}")
                results["installed"] += 1
            except ImportError:
                print_error(f"Missing package: {package}")
                results["missing"].append(package)
                self.status_report["errors"].append(
                    f"Missing Python package: {package}"
                )

        results["status"] = "ok" if not results["missing"] else "error"
        self.status_report["modules"]["python_packages"] = results
        return results

    def check_ollama(self) -> Dict:
        """Check Ollama installation and available models."""
        print_header("Checking Ollama")
        results = {"installed": False, "running": False, "models": []}

        # Check if Ollama is installed
        try:
            result = subprocess.run(
                ["ollama", "--version"], capture_output=True, text=True, timeout=10
            )
            if result.returncode == 0:
                print_success(f"Ollama installed: {result.stdout.strip()}")
                results["installed"] = True
            else:
                print_error("Ollama command failed")
                self.status_report["warnings"].append(
                    "Ollama installed but not responding"
                )
        except FileNotFoundError:
            print_error("Ollama not installed")
            self.status_report["errors"].append(
                "Ollama is not installed on this system"
            )
            self.status_report["recommendations"].append(
                "Install Ollama from https://ollama.ai for local model inference"
            )
            results["status"] = "error"
            self.status_report["modules"]["ollama"] = results
            return results
        except Exception as e:
            print_error(f"Ollama check failed: {e}")
            results["status"] = "error"
            self.status_report["modules"]["ollama"] = results
            return results

        # Check if Ollama is running
        try:
            import requests

            response = requests.get("http://localhost:11434/api/tags", timeout=5)
            if response.status_code == 200:
                print_success("Ollama server is running")
                results["running"] = True

                # Get available models
                models_data = response.json()
                models = models_data.get("models", [])
                if models:
                    print_info(f"Available models: {len(models)}")
                    for model in models:
                        model_name = model.get("name", "unknown")
                        print_success(f"  - {model_name}")
                        results["models"].append(model_name)

                    # Check for recommended models
                    recommended = ["gemma3:270m", "deepseek-r1:1.5b", "llama3.2:latest"]
                    missing_recommended = [
                        m for m in recommended if m not in results["models"]
                    ]
                    if missing_recommended:
                        print_warning(
                            f"Recommended models not found: {', '.join(missing_recommended)}"
                        )
                        self.status_report["recommendations"].append(
                            f"Pull recommended models: ollama pull gemma3:270m"
                        )
                else:
                    print_warning("No models available in Ollama")
                    self.status_report["recommendations"].append(
                        "Pull a model: ollama pull gemma3:270m"
                    )
            else:
                print_warning("Ollama server not responding")
                self.status_report["warnings"].append(
                    "Ollama server may not be running"
                )
                self.status_report["recommendations"].append(
                    "Start Ollama server: ollama serve"
                )
        except requests.exceptions.ConnectionError:
            print_warning("Ollama server is not running")
            self.status_report["warnings"].append("Ollama server is not running")
            self.status_report["recommendations"].append(
                "Start Ollama server: ollama serve"
            )
        except Exception as e:
            print_error(f"Ollama status check failed: {e}")

        results["status"] = (
            "ok" if results["installed"] and results["running"] else "partial"
        )
        self.status_report["modules"]["ollama"] = results
        return results

    def check_api_server(self) -> Dict:
        """Check if the Swarm API server is accessible."""
        print_header("Checking API Server")
        results = {"accessible": False, "endpoints": []}

        try:
            import requests

            base_url = "http://localhost:8021"

            # Check main endpoint
            response = requests.get(f"{base_url}/swarm/status", timeout=5)
            if response.status_code == 200:
                print_success("API server is running on port 8021")
                results["accessible"] = True

                # Check available endpoints
                endpoints = [
                    "/swarm/status",
                    "/swarm/soul",
                    "/task/submit",
                    "/research/stats",
                    "/docs",
                ]

                for endpoint in endpoints:
                    try:
                        resp = requests.get(f"{base_url}{endpoint}", timeout=3)
                        status = "ok" if resp.status_code == 200 else "partial"
                        print_success(f"  Endpoint {endpoint}: {status}")
                        results["endpoints"].append(
                            {"path": endpoint, "status": status}
                        )
                    except:
                        print_warning(f"  Endpoint {endpoint}: not responding")
                        results["endpoints"].append(
                            {"path": endpoint, "status": "error"}
                        )
            else:
                print_warning(f"API server returned status {response.status_code}")
        except requests.exceptions.ConnectionError:
            print_warning("API server is not running")
            self.status_report["recommendations"].append(
                "Start the API server: python launcher.py"
            )
        except Exception as e:
            print_error(f"API server check failed: {e}")

        results["status"] = "ok" if results["accessible"] else "not_running"
        self.status_report["modules"]["api_server"] = results
        return results

    def check_dashboard(self) -> Dict:
        """Check if the dashboard is configured."""
        print_header("Checking Dashboard")
        results = {"configured": False, "node_modules": False, "accessible": False}

        # Check package.json
        package_json = self.base_dir / "dashboard" / "package.json"
        if package_json.exists():
            print_success("Dashboard package.json exists")
            results["configured"] = True
        else:
            print_error("Dashboard package.json missing")

        # Check node_modules
        node_modules = self.base_dir / "dashboard" / "node_modules"
        if node_modules.exists():
            print_success("Dashboard dependencies installed (node_modules exists)")
            results["node_modules"] = True
        else:
            print_warning("Dashboard dependencies not installed")
            self.status_report["recommendations"].append(
                "Install dashboard dependencies: cd dashboard && npm install"
            )

        # Check if dashboard is running
        try:
            import requests

            response = requests.get("http://localhost:5183", timeout=3)
            if response.status_code == 200:
                print_success("Dashboard is running on port 5183")
                results["accessible"] = True
            else:
                print_info("Dashboard server not responding (may need to be started)")
        except:
            print_info("Dashboard server is not running")
            self.status_report["recommendations"].append(
                "Start dashboard: cd dashboard && npm run dev"
            )

        results["status"] = (
            "ok"
            if results["accessible"]
            else "configured"
            if results["configured"]
            else "error"
        )
        self.status_report["modules"]["dashboard"] = results
        return results

    def check_trm_weights(self) -> Dict:
        """Check TRM model weights."""
        print_header("Checking TRM Weights")
        results = {
            "directory_exists": False,
            "weights_found": False,
            "checkpoint_found": False,
        }

        trm_dir = self.base_dir / "tiny-recursive-weights"
        if trm_dir.exists():
            print_success("TRM weights directory exists")
            results["directory_exists"] = True

            # Check for checkpoint files
            checkpoint_files = (
                list(trm_dir.glob("*.pt"))
                + list(trm_dir.glob("*.pth"))
                + list(trm_dir.glob("*.bin"))
            )
            if checkpoint_files:
                print_success(f"Found {len(checkpoint_files)} checkpoint file(s)")
                for f in checkpoint_files:
                    print_info(f"  - {f.name}")
                results["weights_found"] = True
            else:
                print_warning("No TRM checkpoint files found")
                self.status_report["warnings"].append(
                    "TRM weights not found - system will use LLM fallback mode"
                )
                self.status_report["recommendations"].append(
                    "Download or train TRM weights and place in tiny-recursive-weights/"
                )

            # Check for config
            config_file = trm_dir / "config.json"
            if config_file.exists():
                print_success("TRM configuration file exists")
                results["config_exists"] = True
        else:
            print_error("TRM weights directory not found")
            results["directory_exists"] = False

        results["status"] = (
            "ok"
            if results["weights_found"]
            else "partial"
            if results["directory_exists"]
            else "missing"
        )
        self.status_report["modules"]["trm_weights"] = results
        return results

    def check_memory_systems(self) -> Dict:
        """Check memory and storage systems."""
        print_header("Checking Memory Systems")
        results = {"chroma_db": False, "agent_mailbox": False, "secrets_vault": False}

        # Check ChromaDB directory
        chroma_dir = self.base_dir / "swarm_v2_memory" / "chroma_db"
        if chroma_dir.exists():
            print_success("ChromaDB persist directory exists")
            results["chroma_db"] = True
        else:
            print_warning(
                "ChromaDB persist directory not found (will be created on first use)"
            )

        # Check agent mailbox
        mailbox_file = self.base_dir / "swarm_v2_memory" / "agent_mailbox.json"
        if mailbox_file.exists():
            print_success("Agent mailbox initialized")
            try:
                with open(mailbox_file, "r") as f:
                    data = json.load(f)
                    if "message_queues" in data:
                        queues = len(data["message_queues"])
                        print_info(f"  {queues} agent queues configured")
            except:
                pass
            results["agent_mailbox"] = True
        else:
            print_warning("Agent mailbox not initialized")

        # Check secrets vault
        vault_dir = self.base_dir / ".swarm"
        if vault_dir.exists():
            print_success("Secrets vault directory exists")
            vault_file = vault_dir / "secrets.vault"
            if vault_file.exists():
                print_success("Secrets vault initialized")
                results["secrets_vault"] = True
            else:
                print_info("Secrets vault will be created on first use")
        else:
            print_warning(
                "Secrets vault directory not found (will be created on first use)"
            )

        results["status"] = "ok" if all(results.values()) else "partial"
        self.status_report["modules"]["memory_systems"] = results
        return results

    def check_mcp_tools(self) -> Dict:
        """Check MCP tools configuration."""
        print_header("Checking MCP Tools")
        results = {"registry_exists": False, "tools_count": 0, "enabled_tools": 0}

        tools_registry = (
            self.base_dir / "swarm_v2_artifacts" / "mcp_tools" / "tools_registry.json"
        )
        if tools_registry.exists():
            print_success("MCP tools registry exists")
            results["registry_exists"] = True

            try:
                with open(tools_registry, "r") as f:
                    data = json.load(f)
                    tools = data.get("tools", [])
                    results["tools_count"] = len(tools)
                    results["enabled_tools"] = sum(
                        1 for t in tools if t.get("enabled", False)
                    )

                    print_info(f"  Total tools: {len(tools)}")
                    print_info(f"  Enabled tools: {results['enabled_tools']}")

                    # List tool categories
                    categories = set(t.get("category", "unknown") for t in tools)
                    print_info(f"  Categories: {', '.join(categories)}")
            except Exception as e:
                print_error(f"Failed to read tools registry: {e}")
        else:
            print_warning("MCP tools registry not found")

        results["status"] = (
            "ok"
            if results["registry_exists"] and results["tools_count"] > 0
            else "partial"
        )
        self.status_report["modules"]["mcp_tools"] = results
        return results

    def check_kanban_board(self) -> Dict:
        """Check Kanban board state."""
        print_header("Checking Kanban Board")
        results = {"initialized": False, "cards_count": 0, "columns": 0}

        kanban_file = (
            self.base_dir / "swarm_v2_artifacts" / "kanban" / "kanban_state.json"
        )
        if kanban_file.exists():
            print_success("Kanban board initialized")
            results["initialized"] = True

            try:
                with open(kanban_file, "r") as f:
                    data = json.load(f)
                    columns = data.get("columns", {})
                    cards = data.get("cards", {})

                    results["columns"] = len(columns)
                    results["cards_count"] = len(cards)

                    print_info(f"  Columns: {len(columns)}")
                    for col_id, col_data in columns.items():
                        card_count = len(col_data.get("cards", []))
                        print_info(
                            f"    - {col_data.get('name', col_id)}: {card_count} cards"
                        )
            except Exception as e:
                print_error(f"Failed to read Kanban state: {e}")
        else:
            print_warning("Kanban board not initialized")

        results["status"] = "ok" if results["initialized"] else "not_initialized"
        self.status_report["modules"]["kanban_board"] = results
        return results

    def generate_report(self) -> str:
        """Generate a comprehensive status report."""
        print_header("System Status Report")

        report_lines = [
            "=" * 60,
            "SWARM OS v12 - MODULE INITIALIZATION REPORT".center(60),
            "=" * 60,
            f"Generated: {self.status_report['timestamp']}",
            "",
            "MODULE STATUS SUMMARY",
            "-" * 60,
        ]

        for module_name, status_data in self.status_report["modules"].items():
            status = status_data.get("status", "unknown")
            status_icon = "✓" if status == "ok" else "⚠" if status == "partial" else "✗"
            status_color = (
                Colors.OKGREEN
                if status == "ok"
                else Colors.WARNING
                if status == "partial"
                else Colors.FAIL
            )

            report_lines.append(
                f"{status_color}{status_icon} {module_name.replace('_', ' ').title()}: {status.upper()}{Colors.ENDC}"
            )

        # Warnings
        if self.status_report["warnings"]:
            report_lines.extend(["", "WARNINGS", "-" * 60])
            for warning in self.status_report["warnings"]:
                report_lines.append(f"  ⚠ {warning}")

        # Errors
        if self.status_report["errors"]:
            report_lines.extend(["", "ERRORS", "-" * 60])
            for error in self.status_report["errors"]:
                report_lines.append(f"  ✗ {error}")

        # Recommendations
        if self.status_report["recommendations"]:
            report_lines.extend(["", "RECOMMENDATIONS", "-" * 60])
            for i, rec in enumerate(self.status_report["recommendations"], 1):
                report_lines.append(f"  {i}. {rec}")

        # Overall Status
        errors_count = len(self.status_report["errors"])
        warnings_count = len(self.status_report["warnings"])

        report_lines.extend(
            [
                "",
                "=" * 60,
                "OVERALL STATUS",
                "=" * 60,
            ]
        )

        if errors_count == 0 and warnings_count == 0:
            report_lines.append(
                f"{Colors.OKGREEN}✓ ALL SYSTEMS OPERATIONAL{Colors.ENDC}"
            )
        elif errors_count == 0:
            report_lines.append(
                f"{Colors.WARNING}⚠ SYSTEM OPERATIONAL WITH {warnings_count} WARNING(S){Colors.ENDC}"
            )
        else:
            report_lines.append(
                f"{Colors.FAIL}✗ SYSTEM HAS {errors_count} ERROR(S) - ATTENTION REQUIRED{Colors.ENDC}"
            )

        report_lines.extend(
            [
                "",
                "=" * 60,
            ]
        )

        report = "\n".join(report_lines)
        print(report)

        # Save report to file
        report_file = self.base_dir / "system_status_report.txt"
        with open(report_file, "w", encoding="utf-8") as f:
            # Strip ANSI codes for file output
            clean_report = report
            for color in [
                Colors.HEADER,
                Colors.OKBLUE,
                Colors.OKCYAN,
                Colors.OKGREEN,
                Colors.WARNING,
                Colors.FAIL,
                Colors.ENDC,
                Colors.BOLD,
                Colors.UNDERLINE,
            ]:
                clean_report = clean_report.replace(color, "")
            f.write(clean_report)

        print_info(f"Report saved to: {report_file}")

        return report

    def run_all_checks(self):
        """Run all initialization and verification checks."""
        print_header("Swarm OS v12 - Module Initialization")
        print_info(f"Base Directory: {self.base_dir}")
        print_info(f"Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

        # Run all checks
        self.check_directories()
        self.check_configuration_files()
        self.check_python_packages()
        self.check_ollama()
        self.check_trm_weights()
        self.check_memory_systems()
        self.check_mcp_tools()
        self.check_kanban_board()
        self.check_api_server()
        self.check_dashboard()

        # Generate final report
        self.generate_report()

        return self.status_report


def main():
    """Main entry point."""
    print("\n" + "=" * 60)
    print("SWARM OS v12 - COMPREHENSIVE MODULE INITIALIZATION".center(60))
    print("=" * 60 + "\n")

    initializer = ModuleInitializer()
    status = initializer.run_all_checks()

    # Exit with appropriate code
    errors = len(status.get("errors", []))
    if errors > 0:
        print(
            f"\n{Colors.FAIL}Initialization completed with {errors} error(s). Please address the issues above.{Colors.ENDC}"
        )
        sys.exit(1)
    else:
        print(f"\n{Colors.OKGREEN}Initialization completed successfully!{Colors.ENDC}")
        sys.exit(0)


if __name__ == "__main__":
    main()
