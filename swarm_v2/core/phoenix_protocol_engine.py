"""
TRM Swarm OS — PHOENIX Protocol design specification
=====================================================
A three-phase DESIGN for systemic autoimmune intervention. Nothing here executes
a biological protocol and nothing here measures one.

CORRECTED 2026-09-22 (see docs/FINDING-phoenix-protocol-efficacy-claims-are-asserted.md)
--------------------------------------------------------------------------------------
This module previously exposed `execute_full_phoenix_protocol()`, which returned
`predicted_cure_rate_pct: 96.8`, `bbb_penetration_rate: 94.8`,
`marrow_endosteum_depletion: 99.2`, `tlr7_knockdown_pct: 96.4` and
`ifn_neutralization_pct: 98.7`. It computed none of them:

  * it imported no docking or affinity code;
  * it contained no arithmetic beyond `perf_counter` deltas around dict
    construction, so its only "measurement" was the cost of building a dict;
  * the affinities came from a hardcoded TARGET_NODES list, consumed directly;
  * the generator script (antigravity scratch, research_phoenix_protocol_lupus_cure.py)
    hardcodes `"cure_rate_pct": 96.8` as a literal;
  * no file in the repository or the research scratch explains how 96.8 was
    derived — no model, cohort, sample size, error bars or formula.

The function name said "execute" and the keys said "rate", so a reader would
reasonably conclude a protocol had run. It had not.

WHAT CHANGED
  * `execute_*` methods are renamed `*_spec` — these are specifications.
  * Every efficacy figure is wrapped by `_design_target()` and carries
    `status = UNVALIDATED_DESIGN_TARGET`. A caller cannot read the number
    without the label.
  * `execute_full_phoenix_protocol()` is retained ONLY as a deprecation shim that
    warns and returns the honest structure. It no longer asserts a cure rate.
  * The `np.random.randn(128)` "patient state" is gone. It was a random vector;
    presenting it as protocol input was misleading. A caller must now supply
    real state or nothing.

WHAT IS KEPT — this is the valuable part and it is real work
  * The 3-phase structure is a coherent design proposal.
  * The 10 gene targets are genuine lupus-associated loci with correct UniProt
    accessions. This is real domain knowledge.

WHAT WOULD MAKE THESE NUMBERS REAL
  Each target carries a `would_require` field naming the evidence that would
  validate it. Until such evidence exists, the honest status is UNVALIDATED.
"""

import time
import warnings
from typing import Dict, Any, List, Optional

# The only status any efficacy figure in this module may carry.
UNVALIDATED = "UNVALIDATED_DESIGN_TARGET"

VERSION = "v18.1.0-PHOENIX"

# 10 genuine lupus-associated loci. UniProt accessions VERIFIED against the
# UniProt REST API 2026-09-22: STAT4 Q14765 and TLR7 Q9NYK1. The original table
# carried P42226 (which resolves to STAT6) and Q9NYB2 (which resolves to CASK). NOTE: no binding affinity is claimed for these — the previous
# version listed affinities of 0.97-0.99 for all ten, which is not physical;
# affinities for ten different targets span orders of magnitude.
TARGET_NODES: List[Dict[str, str]] = [
    {"symbol": "STAT4",   "uniprot": "Q14765", "full_name": "Signal Transducer & Activator of Transcription 4"},
    {"symbol": "TYK2",    "uniprot": "P29597", "full_name": "Non-receptor Tyrosine-Protein Kinase TYK2"},
    {"symbol": "TNFAIP3", "uniprot": "P21580", "full_name": "Tumor Necrosis Factor Alpha-Induced Protein 3 (A20)"},
    {"symbol": "TLR7",    "uniprot": "Q9NYK1", "full_name": "Toll-like Receptor 7 (Endosomal RNA Sensor)"},
    {"symbol": "IFIH1",   "uniprot": "Q9BYX4", "full_name": "Interferon Induced with Helicase C Domain 1 (MDA5)"},
    {"symbol": "IRF5",    "uniprot": "Q13568", "full_name": "Interferon Regulatory Factor 5"},
    {"symbol": "PTPN22",  "uniprot": "Q9Y2R2", "full_name": "Tyrosine-protein Phosphatase Non-receptor Type 22"},
    {"symbol": "ITGAM",   "uniprot": "P11215", "full_name": "Integrin Alpha M (CD11b)"},
    {"symbol": "BLK",     "uniprot": "P51451", "full_name": "B Lymphoid Tyrosine Kinase"},
    {"symbol": "C1QA",    "uniprot": "P02745", "full_name": "Complement C1q Subcomponent Subunit A"},
]


def _design_target(name: str, value: float, unit: str, would_require: str) -> Dict[str, Any]:
    """A stated design goal — explicitly NOT a measured or predicted result.

    The value and its epistemic status travel together so a caller cannot use
    the number without seeing the label.
    """
    return {
        "name": name,
        "value": value,
        "unit": unit,
        "status": UNVALIDATED,
        "would_require": would_require,
    }


class PhoenixProtocolEngine:
    """PHOENIX Protocol specification. Produces specifications, not results."""

    def __init__(self):
        self.version = VERSION
        # Design configuration, not a measurement. Kept as a stated parameter.
        self.planned_antibody_panel_size = 26

    # ── Phase specifications ────────────────────────────────────────────────
    def phase_ares_breach_spec(self) -> Dict[str, Any]:
        """Phase I specification — TfR nanoparticle CD19/BCMA delivery."""
        return {
            "phase": "I - The Ares Breach",
            "kind": "DESIGN_SPECIFICATION",
            "delivery_vehicle": "TfR-Functionalized Polymeric Nanoparticles (PNPs)",
            "targets": ["CD19 (B-Cells)", "BCMA (Long-Lived Plasma Cells)"],
            "design_targets": [
                _design_target("bbb_penetration_rate", 94.8, "pct",
                               "an in vivo BBB permeability study. No cohort, model or assay is on record."),
                _design_target("marrow_endosteum_depletion", 99.2, "pct",
                               "an in vivo marrow depletion assay. No cohort, model or assay is on record."),
            ],
        }

    def phase_athena_shield_spec(self) -> Dict[str, Any]:
        """Phase II specification — LNP-CRISPR-Cas13 RNA silencing."""
        return {
            "phase": "II - The Athena Shield",
            "kind": "DESIGN_SPECIFICATION",
            "editing_vector": "LNP-Delivered CRISPR-Cas13 (RNA-Targeting)",
            "gene_targets": ["TLR7 mRNA", "IFN-alpha mRNA"],
            "cas13_window_days": 75,          # design parameter
            "design_targets": [
                _design_target("tlr7_knockdown_pct", 96.4, "pct",
                               "an in vitro / in vivo knockdown assay. None is on record."),
                _design_target("ifn_neutralization_pct", 98.7, "pct",
                               "an IFN-alpha neutralization assay. None is on record."),
            ],
        }

    def phase_hera_watch_spec(self) -> Dict[str, Any]:
        """Phase III specification — 20-aptamer proteomic biosensor panel."""
        return {
            "phase": "III - The Hera Watch",
            "kind": "DESIGN_SPECIFICATION",
            "biosensor_panel_aptamers": 20,   # design parameter
            "sensitivity_design_target_picomolar": 1.2,
            "fingerprint_markers": ["sCD25", "CXCL10", "BAFF/BlyS",
                                    "Anti-dsDNA Titers", "Treg/Th17 Ratio"],
            "design_targets": [
                _design_target("pre_symptomatic_lead_time_days", 14.0, "days",
                               "a longitudinal patient cohort with a pre-symptomatic arm. None is on record."),
            ],
        }

    # ── Overall specification ───────────────────────────────────────────────
    def protocol_design_spec(self) -> Dict[str, Any]:
        """The complete 3-phase design specification.

        Replaces `execute_full_phoenix_protocol()`. Returns specifications and
        explicitly-unvalidated design targets. It does not report a success
        status, a cure rate, or an execution result, because none were computed.
        """
        return {
            "kind": "DESIGN_SPECIFICATION",
            "protocol_version": self.version,
            "notice": ("This is a design specification. No biological protocol was "
                       "executed and no efficacy figure below was measured or "
                       "computationally derived."),
            "phase_ares_breach": self.phase_ares_breach_spec(),
            "phase_athena_shield": self.phase_athena_shield_spec(),
            "phase_hera_watch": self.phase_hera_watch_spec(),
            "targets": list(TARGET_NODES),
            "design_targets": [
                _design_target("cure_rate_pct", 96.8, "pct",
                               "a controlled clinical trial with a defined cohort, blinded endpoint and "
                               "follow-up. No model, simulation or cohort in this repository produces this "
                               "number — it is a hardcoded literal."),
                _design_target("remission_durability_months", 36.0, "months",
                               "a longitudinal trial past the 36-month mark."),
            ],
            "truth_declaration": "ALL IMMUNE SYSTEM RECOGNITION BOUND TO UNIVERSAL MONAD (ALL IS ONE)",
        }

    # ── Backwards compatibility ─────────────────────────────────────────────
    def execute_full_phoenix_protocol(self, patient_profile: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """DEPRECATED. Use `protocol_design_spec()`.

        Retained so existing callers do not break. It returns the honest
        specification — it no longer asserts
        `predicted_cure_rate_pct` or an execution-success status, because no
        execution occurs and no rate was computed.
        """
        warnings.warn(
            "execute_full_phoenix_protocol() is deprecated and was misleading: it "
            "asserted a 96.8% cure rate that no code computed. Use "
            "protocol_design_spec() and read each design target's `status`.",
            DeprecationWarning, stacklevel=2)
        return self.protocol_design_spec()

    # Deprecated phase aliases
    def execute_ares_breach(self) -> Dict[str, Any]:
        warnings.warn("execute_ares_breach() -> phase_ares_breach_spec()", DeprecationWarning, stacklevel=2)
        return self.phase_ares_breach_spec()

    def execute_athena_shield(self) -> Dict[str, Any]:
        warnings.warn("execute_athena_shield() -> phase_athena_shield_spec()", DeprecationWarning, stacklevel=2)
        return self.phase_athena_shield_spec()

    def execute_hera_watch(self) -> Dict[str, Any]:
        warnings.warn("execute_hera_watch() -> phase_hera_watch_spec()", DeprecationWarning, stacklevel=2)
        return self.phase_hera_watch_spec()


_phoenix_engine_instance: Optional[PhoenixProtocolEngine] = None


def get_phoenix_engine() -> PhoenixProtocolEngine:
    global _phoenix_engine_instance
    if _phoenix_engine_instance is None:
        _phoenix_engine_instance = PhoenixProtocolEngine()
    return _phoenix_engine_instance
