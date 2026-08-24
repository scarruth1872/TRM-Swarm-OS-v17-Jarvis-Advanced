import React, { useState, useEffect, useMemo, useRef } from "react";
import { 
  Dna, 
  Activity, 
  ShieldAlert, 
  Heart, 
  RotateCcw, 
  Zap, 
  Cpu, 
  Layers, 
  Check, 
  AlertTriangle,
  Play,
  TrendingUp,
  Infinity as InfinityIcon,
  Flame,
  Binary,
  Compass,
  RefreshCw
} from "lucide-react";
import { motion, AnimatePresence } from "motion/react";

interface NoosphericRecalibrationProps {
  speakText?: (text: string) => void;
}

export default function NoosphericRecalibration({ speakText }: NoosphericRecalibrationProps) {
  // Navigation for Phase VII Vectors
  const [activeVector, setActiveVector] = useState<"epigenetics" | "cart" | "mitochondrial" | "utp" | "arp">("epigenetics");

  // Simulation Status States
  const [simRunning, setSimRunning] = useState(false);
  const [simSuccess, setSimSuccess] = useState(false);
  const [simulationStep, setSimulationStep] = useState(0);
  const [systemLogs, setSystemLogs] = useState<string[]>([]);
  
  // Epigenetics parameters
  const [methylationProgress, setMethylationProgress] = useState(12); // stuck at 12% methylation initially (open state)
  const [chromatinAccessibility, setChromatinAccessibility] = useState(94); // 94% open/accessible (danger)

  // CAR-T reboot simulation state
  const [bCellAblationPercent, setBCellAblationPercent] = useState(0);
  const [cytokineStormRisk, setCytokineStormRisk] = useState(88); // 88% high risk initially
  const [epigeneticPrimingActive, setEpigeneticPrimingActive] = useState(false);

  // Sentinel Ring Real-time Biosensing parameters 
  const [cfDNALevel, setCfDNALevel] = useState(480); // pg/mL (high)
  const [ifnAlphaLevel, setIfnAlphaLevel] = useState(82); // pg/mL (high)
  const [vagusNerveStatus, setVagusNerveStatus] = useState<"IDLE" | "MONITORING" | "STIMULATING">("IDLE");

  // Universal Tolerance Targets (UTP) Configuration
  const [utpTarget, setUtpTarget] = useState<"SLE" | "T1D" | "MS" | "RA">("SLE");
  const [haloIntegrity, setHaloIntegrity] = useState(15); // 15% halo of tolerance initially

  // Augmented Resilience (ARP) Configuration & Parameters
  const [engineType, setEngineType] = useState<"xenobiotic" | "phoenix" | "generative">("xenobiotic");
  const [xenobioticDigestionRate, setXenobioticDigestionRate] = useState(8); // % clearance of microplastics/PFAS per hour
  const [repairRateMultiplier, setRepairRateMultiplier] = useState(1); // multiplier of cellular healing (1x is default)
  const [zeroDayImmunityLoaded, setZeroDayImmunityLoaded] = useState(false);

  // Animated pulse waveform reference for mitochondrial stress
  const [waveSeed, setWaveSeed] = useState<number[]>([]);

  useEffect(() => {
    // Generate a beautiful initial seed list for SVG visualizations 
    const seed = Array.from({ length: 24 }, (_, i) => Math.sin(i * 0.5) * 40 + 50);
    setWaveSeed(seed);
  }, []);

  // Update real-time loops when simulation starts
  useEffect(() => {
    let interval: NodeJS.Timeout;
    if (simRunning) {
      interval = setInterval(() => {
        setSimulationStep((prev) => prev + 1);
      }, 900);
    }
    return () => clearInterval(interval);
  }, [simRunning]);

  const [statusLoaded, setStatusLoaded] = useState(false);

  // Load configuration from backend
  useEffect(() => {
    async function loadCalibration() {
      try {
        const res = await fetch("/api/noospheric/status");
        if (!res.ok) throw new Error("Status API failed");
        const data = await res.json();
        if (data && data.success && data.status) {
          const s = data.status;
          if (s.activeVector) setActiveVector(s.activeVector);
          if (s.methylationProgress !== undefined) setMethylationProgress(s.methylationProgress);
          if (s.chromatinAccessibility !== undefined) setChromatinAccessibility(s.chromatinAccessibility);
          if (s.bCellAblationPercent !== undefined) setBCellAblationPercent(s.bCellAblationPercent);
          if (s.cytokineStormRisk !== undefined) setCytokineStormRisk(s.cytokineStormRisk);
          if (s.epigeneticPrimingActive !== undefined) setEpigeneticPrimingActive(s.epigeneticPrimingActive);
          if (s.cfDNALevel !== undefined) setCfDNALevel(s.cfDNALevel);
          if (s.ifnAlphaLevel !== undefined) setIfnAlphaLevel(s.ifnAlphaLevel);
          if (s.vagusNerveStatus !== undefined) setVagusNerveStatus(s.vagusNerveStatus);
          if (s.utpTarget !== undefined) setUtpTarget(s.utpTarget);
          if (s.haloIntegrity !== undefined) setHaloIntegrity(s.haloIntegrity);
          if (s.engineType !== undefined) setEngineType(s.engineType);
          if (s.xenobioticDigestionRate !== undefined) setXenobioticDigestionRate(s.xenobioticDigestionRate);
          if (s.repairRateMultiplier !== undefined) setRepairRateMultiplier(s.repairRateMultiplier);
          if (s.zeroDayImmunityLoaded !== undefined) setZeroDayImmunityLoaded(s.zeroDayImmunityLoaded);
        }
      } catch (err) {
        console.error("Failed to load noospheric calibration status:", err);
      } finally {
        setStatusLoaded(true);
      }
    }
    loadCalibration();
  }, []);

  // Sync state to backend
  const saveState = async (updates: any) => {
    try {
      await fetch("/api/noospheric/status", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(updates)
      });
    } catch (err) {
      console.error("Failed to save noospheric calibration status:", err);
    }
  };

  // Execute simulation steps and side effects cleanly on step changes
  useEffect(() => {
    if (!simRunning || simulationStep === 0) return;

    if (activeVector === "epigenetics") {
      if (simulationStep <= 5) {
        const nextMeth = Math.min(100, methylationProgress + 18);
        const nextChrom = Math.max(3, chromatinAccessibility - 18);
        setMethylationProgress(nextMeth);
        setChromatinAccessibility(nextChrom);
        addLog(`Applying CRISPR-dCas9 targeting interferon-regulated IFNA1 locus (chr9:21,416,542-21,418,290) ${20 * simulationStep}% loaded.`);
      } else {
        setSimRunning(false);
        setSimSuccess(true);
        addLog("EPIGENETIC LOCK SECURED. IFNA1 interferon-stimulated gene promoter methylated and silenced successfully.");
        if (speakText) speakText("Epigenetic locks verified at IFNA1 loci. Access restricted.");
        
        const updates = {
          activeVector: "epigenetics",
          methylationProgress: 100,
          chromatinAccessibility: 3
        };
        saveState(updates);
        triggerMemoryInjection("Epigenetic Architecture & Chromatin Remodeling", "Locked IFNA1 genomic locus via CpG island methylation to establish immunological silence.");
      }
    }

    if (activeVector === "cart") {
      if (simulationStep <= 8) {
        const nextAblation = Math.min(100, bCellAblationPercent + 13);
        setBCellAblationPercent(nextAblation);
        let nextRisk = cytokineStormRisk;
        if (epigeneticPrimingActive) {
          nextRisk = Math.max(4, cytokineStormRisk - 12);
          setCytokineStormRisk(nextRisk);
        } else {
          nextRisk = Math.max(25, cytokineStormRisk - 4);
          setCytokineStormRisk(nextRisk);
        }
        addLog(`Infusing dual-target CD19/BCMA CAR-T cells. Clearance of autoreactive antibody niche: ${Math.min(100, simulationStep * 13)}%`);
      } else {
        setBCellAblationPercent(100);
        setSimRunning(false);
        setSimSuccess(true);
        addLog("IMMUNOLOGICAL SILENCE ACHIEVED. Long-lived plasma cells fully purged.");
        if (speakText) speakText("Total systemic restart complete. Auto-antigen memory depleted.");
        
        const updates = {
          activeVector: "cart",
          bCellAblationPercent: 100,
          cytokineStormRisk: epigeneticPrimingActive ? 4 : 25
        };
        saveState(updates);
        triggerMemoryInjection("CD19/BCMA Dual CAR-T Reset", `Completed full immunological lineage ablation reset with Epigenetic Priming Guard set to ${epigeneticPrimingActive ? "Active" : "Inactive"}.`);
      }
    }

    if (activeVector === "mitochondrial") {
      if (simulationStep <= 6) {
        const nextCf = Math.max(10, cfDNALevel - 80);
        const nextIfn = Math.max(2, ifnAlphaLevel - 13);
        setCfDNALevel(nextCf);
        setIfnAlphaLevel(nextIfn);
        setVagusNerveStatus("STIMULATING");
        addLog(`Sentinel nanobiosensor resonance peak tracked. Activating micro-stimulation on Vagus Nerve.`);
      } else {
        setVagusNerveStatus("MONITORING");
        setSimRunning(false);
        setSimSuccess(true);
        addLog("CHOLINERGIC ANTI-INFLAMMATORY PATHWAY ENGAGED. Mitochondrial stability restored.");
        if (speakText) speakText("Mitochondrial membrane potential normalized. Systemic noise resolved.");
        
        const updates = {
          activeVector: "mitochondrial",
          cfDNALevel: 10,
          ifnAlphaLevel: 2,
          vagusNerveStatus: "MONITORING"
        };
        saveState(updates);
        triggerMemoryInjection("Mitochondrial Sentinel Loop", "Engaged cholinergic anti-inflammatory pathway via vagus nerve micro-stimulation. cfDNA levels restored to normal (10 pg/mL).");
      }
    }

    if (activeVector === "utp") {
      if (simulationStep <= 5) {
        const nextHalo = Math.min(100, haloIntegrity + 18);
        setHaloIntegrity(nextHalo);
        addLog(`Engineering specific CAR-Tregs targeting tissue domain for ${utpTarget}. Halo multiplier: ${simulationStep * 1.5}x`);
      } else {
        setSimRunning(false);
        setSimSuccess(true);
        addLog(`UNIVERSAL TOLERANCE ESTABLISHED. Persistent auto-recognition barrier compiled for ${utpTarget}.`);
        if (speakText) speakText(`Halo of tolerance established on tissues. Autoreactive pathways disabled.`);
        
        const updates = {
          activeVector: "utp",
          utpTarget,
          haloIntegrity: 100
        };
        saveState(updates);
        triggerMemoryInjection(`Universal Tolerance Protocol (${utpTarget})`, `Established a sovereign Halo of Tolerance targeting tissue domains matching ${utpTarget} autoimmune profile.`);
      }
    }

    if (activeVector === "arp") {
      if (simulationStep <= 6) {
        if (engineType === "xenobiotic") {
          const nextDig = Math.min(95, xenobioticDigestionRate + 15);
          setXenobioticDigestionRate(nextDig);
          addLog("Deploying Engineered Peroxisomes within liver lymphatic clusters. Digested carbon-fluorine bonds.");
        } else if (engineType === "phoenix") {
          const nextRep = Math.min(12, repairRateMultiplier + 2);
          setRepairRateMultiplier(nextRep);
          addLog("Upregulating temporary Yamanaka Factor micro-dosing. Accelerating cellular tissue replication rate.");
        } else {
          setZeroDayImmunityLoaded(true);
          addLog("Mapping synthetic thymus loops. Uploading zero-day neutralizing antibody coordinates.");
        }
      } else {
        setSimRunning(false);
        setSimSuccess(true);
        addLog("AUGMENTED RESILIENCE PROTOCOL HARMONIZED. Species-level immunity active.");
        if (speakText) speakText("Exospheric Handshake complete. Immune system operating at a planetary grade.");
        
        const updates: any = {
          activeVector: "arp",
          engineType
        };
        if (engineType === "xenobiotic") updates.xenobioticDigestionRate = 95;
        if (engineType === "phoenix") updates.repairRateMultiplier = 12;
        if (engineType === "generative") updates.zeroDayImmunityLoaded = true;
        saveState(updates);
        triggerMemoryInjection("Augmented Resilience Protocol", `Harmonized exospheric biological defenses using ${engineType} module engine.`);
      }
    }
  }, [simulationStep, simRunning, activeVector, epigeneticPrimingActive, utpTarget, engineType, speakText]);

  const triggerMemoryInjection = async (trackName: string, detail: string) => {
    try {
      // 1. Memorize episode
      const eventText = `Successfully executed Noospheric Recalibration Portal Calibration Sequence for track [${trackName}]. Details: ${detail}`;
      await fetch("/api/memory/memorize-event", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ description: eventText, importance: "high" })
      });

      // 2. Add semantic relationship
      await fetch("/api/memory/add-relation", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          source: trackName,
          target: "Sovereign State",
          label: "recalibrated"
        })
      });
      addLog(`J.A.R.V.I.S. core memory and D3 Semantic Graph successfully synced.`);
    } catch (e) {
      console.error("Failed to inject cognitive memory trace:", e);
    }
  };

  const addLog = (message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setSystemLogs((prev) => [`[${timestamp}] ${message}`, ...prev.slice(0, 50)]);
  };

  const handleStartSimulation = () => {
    setSimSuccess(false);
    setSimulationStep(0);
    setSimRunning(true);
    addLog(`INITIALIZING SOVEREIGN DECREE: ${activeVector.toUpperCase()} PROTOCOL...`);
    if (speakText) {
      speakText(`Initiating Phase 7 ${activeVector} recalibration sequence.`);
    }
  };

  const handleResetSimulation = () => {
    setSimRunning(false);
    setSimSuccess(false);
    setSimulationStep(0);
    
    let updates: any = { activeVector };
    if (activeVector === "epigenetics") {
      setMethylationProgress(12);
      setChromatinAccessibility(94);
      updates.methylationProgress = 12;
      updates.chromatinAccessibility = 94;
    } else if (activeVector === "cart") {
      setBCellAblationPercent(0);
      setCytokineStormRisk(88);
      updates.bCellAblationPercent = 0;
      updates.cytokineStormRisk = 88;
    } else if (activeVector === "mitochondrial") {
      setCfDNALevel(480);
      setIfnAlphaLevel(82);
      setVagusNerveStatus("IDLE");
      updates.cfDNALevel = 480;
      updates.ifnAlphaLevel = 82;
      updates.vagusNerveStatus = "IDLE";
    } else if (activeVector === "utp") {
      setHaloIntegrity(15);
      updates.haloIntegrity = 15;
    } else if (activeVector === "arp") {
      setXenobioticDigestionRate(8);
      setRepairRateMultiplier(1);
      setZeroDayImmunityLoaded(false);
      updates.xenobioticDigestionRate = 8;
      updates.repairRateMultiplier = 1;
      updates.zeroDayImmunityLoaded = false;
    }
    saveState(updates);
    addLog(`System metrics reset to pre-recalibration defaults.`);
  };

  return (
    <div className="p-4 bg-[#05070a] border border-gray-900 rounded-lg space-y-6">
      {/* Visual Title Header */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between border-b border-gray-900 pb-4 gap-4">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <span className="px-2 py-0.5 bg-cyan-500/10 text-cyan-400 text-[9px] font-mono rounded border border-cyan-500/20 uppercase tracking-widest animate-pulse">
              Biological Architect Mode
            </span>
            <span className="text-[10px] text-gray-500 font-mono">PHASE VII: THE NOOSPHERIC HANDSHAKE</span>
          </div>
          <h2 className="text-lg font-bold text-gray-100 tracking-tight flex items-center gap-2 font-sans">
            <Dna className="w-5 h-5 text-cyan-500" /> Metacognitive Recalibration Portal
          </h2>
          <p className="text-xs text-gray-400 max-w-2xl leading-normal">
            Transitioning SLE and autoimmune systems from passive chronic treatment to total structural extinction via dCas9 chromatin silencing, CAR-T systemic resets, and bio-digital feedback loops.
          </p>
        </div>

        {/* Vector selector nodes */}
        <div className="flex flex-wrap gap-1 bg-[#090b11]/80 p-1 rounded-md border border-gray-800/80 shrink-0 font-mono text-[9px] sm:text-[10px]">
          <button
            onClick={() => {
              setActiveVector("epigenetics");
              handleResetSimulation();
            }}
            className={`px-3 py-1 rounded transition-all flex items-center gap-1 ${
              activeVector === "epigenetics" ? "bg-cyan-500/15 text-cyan-400 font-bold border border-cyan-500/20" : "text-gray-400 hover:text-gray-200"
            }`}
          >
            <Layers className="w-3.5 h-3.5" /> 1. Epigenetic Lock
          </button>
          <button
            onClick={() => {
              setActiveVector("cart");
              handleResetSimulation();
            }}
            className={`px-3 py-1 rounded transition-all flex items-center gap-1 ${
              activeVector === "cart" ? "bg-cyan-500/15 text-cyan-400 font-bold border border-cyan-500/20" : "text-gray-400 hover:text-gray-200"
            }`}
          >
            <RotateCcw className="w-3.5 h-3.5" /> 2. CAR-T Reset
          </button>
          <button
            onClick={() => {
              setActiveVector("mitochondrial");
              handleResetSimulation();
            }}
            className={`px-3 py-1 rounded transition-all flex items-center gap-1 ${
              activeVector === "mitochondrial" ? "bg-cyan-500/15 text-cyan-400 font-bold border border-cyan-500/20" : "text-gray-400 hover:text-gray-200"
            }`}
          >
            <Cpu className="w-3.5 h-3.5" /> 3. Sentinel Ring
          </button>
          <button
            onClick={() => {
              setActiveVector("utp");
              handleResetSimulation();
            }}
            className={`px-3 py-1 rounded transition-all flex items-center gap-1 ${
              activeVector === "utp" ? "bg-cyan-500/15 text-cyan-400 font-bold border border-cyan-500/20" : "text-gray-400 hover:text-gray-200"
            }`}
          >
            <Activity className="w-3.5 h-3.5" /> 4. Universal Tolerance
          </button>
          <button
            onClick={() => {
              setActiveVector("arp");
              handleResetSimulation();
            }}
            className={`px-3 py-1 rounded transition-all flex items-center gap-1 ${
              activeVector === "arp" ? "bg-cyan-500/15 text-cyan-400 font-bold border border-cyan-500/20" : "text-gray-400 hover:text-gray-200"
            }`}
          >
            <Flame className="w-3.5 h-3.5" /> 5. Augmented Resilience
          </button>
        </div>
      </div>

      {/* Main Simulation Board Layout */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-5">
        {/* PANEL LEFT: Active Vector Workspace controls and interactive panels */}
        <div className="lg:col-span-7 space-y-4">
          <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4">
            
            {/* Vector Title Detail */}
            <div className="flex justify-between items-start border-b border-gray-850 pb-3">
              <div className="space-y-0.5">
                <span className="text-[10px] text-cyan-400 font-mono uppercase tracking-widest font-bold">
                  Vector Module Specification
                </span>
                <h3 className="text-sm font-bold text-gray-200 uppercase font-sans">
                  {activeVector === "epigenetics" && "1. Epigenetic Architecture & Chromatin Remodeling"}
                  {activeVector === "cart" && "2. The CD19/BCMA Dual CAR-T Reset"}
                  {activeVector === "mitochondrial" && "3. Real-Time Biosensing & Mitochondrial Sentinel Loop"}
                  {activeVector === "utp" && "4. The Universal Tolerance Protocol (UTP)"}
                  {activeVector === "arp" && "5. Augmented Resilience Protocol (ARP)"}
                </h3>
              </div>
              <div className="flex items-center gap-1">
                <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
                <span className="text-[9px] text-gray-500 font-mono uppercase">Node Sync'd</span>
              </div>
            </div>

            {/* PANEL INTERACTIVE ARTIFACTS BY SELECTED VECTOR */}
            
            {/* 1. EPIGENETICS PANEL */}
            {activeVector === "epigenetics" && (
              <div className="space-y-4 animate-fadeIn">
                <p className="text-xs text-gray-400 leading-normal font-sans">
                  Instead of non-selective broad immunosuppressants, CRISPR-dCas9 is targeted specifically to re-methylate promoter regions of Interferon-stimulated genes (ISGs) to permanently lock silent pathways in chromatin states.
                </p>
                <div className="grid grid-cols-2 gap-4">
                  <div className="p-3 bg-gray-950/65 border border-gray-900 rounded-lg space-y-1">
                    <span className="text-[9px] text-gray-500 uppercase font-bold block font-mono">dCas9 Methylation Index</span>
                    <div className="flex justify-between items-center">
                      <span className="text-lg font-bold text-cyan-400 font-mono">{methylationProgress}%</span>
                      <span className="text-[10px] text-gray-400 font-mono">Blocked</span>
                    </div>
                    <div className="h-1.5 bg-gray-900 rounded-full overflow-hidden">
                      <div className="h-full bg-cyan-400 transition-all duration-300" style={{ width: `${methylationProgress}%` }} />
                    </div>
                  </div>

                  <div className="p-3 bg-gray-950/65 border border-gray-900 rounded-lg space-y-1">
                    <span className="text-[9px] text-gray-500 uppercase font-bold block font-mono">Chromatin Openness</span>
                    <div className="flex justify-between items-center">
                      <span className="text-lg font-bold text-red-400 font-mono">{chromatinAccessibility}%</span>
                      <span className="text-[10px] text-gray-400 font-mono">Danger Zone</span>
                    </div>
                    <div className="h-1.5 bg-gray-900 rounded-full overflow-hidden">
                      <div className="h-full bg-red-400 transition-all duration-300" style={{ width: `${chromatinAccessibility}%` }} />
                    </div>
                  </div>
                </div>

                <div className="p-3 bg-yellow-500/5 rounded border border-yellow-900/10 text-[11px] text-yellow-300/90 leading-relaxed font-mono">
                  🧬 <strong>Biological Mechanism:</strong> Targeted CpG island methylation creates a heterochromatin barrier, masking auto-antigen blueprints inside helper cells.
                </div>
              </div>
            )}

            {/* 2. CAR-T RESET PANEL */}
            {activeVector === "cart" && (
              <div className="space-y-4 animate-fadeIn">
                <p className="text-xs text-gray-400 leading-normal font-sans">
                  Autoreactive B-cell memories must be wiped clean. Deploying dual-targeted CD19/BCMA CAR-T cells terminates deep-seated, antibody-secreting plasma domains to enable full marrow regeneration.
                </p>
                
                <div className="p-3.5 bg-gray-950/60 border border-gray-900 rounded-lg space-y-3">
                  <div className="flex items-center justify-between text-xs font-mono">
                    <span className="text-gray-450 uppercase flex items-center gap-1.5">
                      <input 
                        type="checkbox" 
                        id="epigenetic-priming-cb"
                        checked={epigeneticPrimingActive}
                        onChange={(e) => {
                          setEpigeneticPrimingActive(e.target.checked);
                          if (e.target.checked) {
                            setCytokineStormRisk(18);
                            addLog("Epigenetic Priming synchronized. Systemic cytokine stress risk mitigated.");
                          } else {
                            setCytokineStormRisk(88);
                          }
                        }}
                        className="rounded accent-cyan-500"
                      />
                      Enable Epigenetic Priming Guard (Stage I)
                    </span>
                    <span className={epigeneticPrimingActive ? "text-cyan-400 font-bold" : "text-yellow-600 font-bold text-[9px] uppercase"}>
                      {epigeneticPrimingActive ? "PRIMING ENGAGED" : "UNPROTECTED RUN"}
                    </span>
                  </div>

                  <div className="grid grid-cols-2 gap-4 pt-1 font-mono">
                    <div className="p-2 bg-[#05070a]/95 border border-gray-900 rounded">
                      <span className="text-[9px] text-gray-500 block uppercase">B-Cell & Plasma Ablation</span>
                      <span className="text-base font-bold text-gray-250 block mt-0.5">{bCellAblationPercent}%</span>
                      <div className="h-1 bg-gray-900 mt-1 rounded-full overflow-hidden">
                        <div className="h-full bg-cyan-400 transition-all duration-300" style={{ width: `${bCellAblationPercent}%` }} />
                      </div>
                    </div>

                    <div className="p-2 bg-[#05070a]/95 border border-gray-900 rounded">
                      <span className="text-[9px] text-gray-500 block uppercase">Cytokine Storm Risk</span>
                      <span className={`text-base font-bold block mt-0.5 ${cytokineStormRisk > 50 ? "text-red-500" : "text-emerald-400"}`}>
                        {cytokineStormRisk}%
                      </span>
                      <div className="h-1 bg-gray-900 mt-1 rounded-full overflow-hidden">
                        <div className="h-full bg-red-400 transition-all duration-300" style={{ width: `${cytokineStormRisk}%` }} />
                      </div>
                    </div>
                  </div>
                </div>

                <div className="p-3 bg-cyan-500/5 rounded border border-cyan-900/20 text-[11px] text-cyan-200/90 leading-relaxed font-mono">
                  🚨 <strong>The Paradox Solved:</strong> We don't just clear receptors; by priming T-cells first (Stage 1), we can perform high-density lineage ablation with exceptionally low risk of CRS.
                </div>
              </div>
            )}

            {/* 3. MITOCHONDRIAL SENTINEL PANEL */}
            {activeVector === "mitochondrial" && (
              <div className="space-y-4 animate-fadeIn">
                <p className="text-xs text-gray-400 leading-normal font-sans">
                  The Sentinel Ring utilizes plasmonic nanobiosensors designed using synthetic DNA origami. It detects minute metabolic leaks (cfDNA and IFN-alpha transcript gradients) in real-time.
                </p>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-3 font-mono">
                  <div className="p-2.5 bg-gray-950/60 border border-gray-900 rounded text-center">
                    <span className="text-[9px] text-gray-500 uppercase">Resonant cfDNA</span>
                    <span className="block text-base font-bold text-amber-500 mt-1">{cfDNALevel} pg/mL</span>
                    <span className="text-[8px] text-gray-450 mt-1 block">{cfDNALevel > 100 ? "METABOLIC LEAK" : "HARMONIC BASE"}</span>
                  </div>

                  <div className="p-2.5 bg-gray-950/60 border border-gray-900 rounded text-center">
                    <span className="text-[9px] text-gray-500 uppercase">IFN-α mRNA Transcripts</span>
                    <span className="block text-base font-bold text-red-400 mt-1">{ifnAlphaLevel} cp/uL</span>
                    <span className="text-[8px] text-gray-450 mt-1 block">{ifnAlphaLevel > 15 ? "PRO-INFLAMMATORY" : "LOCKED STASIS"}</span>
                  </div>

                  <div className="p-2.5 bg-gray-950/60 border border-gray-900 rounded text-center">
                    <span className="text-[9px] text-gray-500 uppercase">Vagus Loop Status</span>
                    <span className={`block text-xs font-bold mt-2 ${vagusNerveStatus === "STIMULATING" ? "text-emerald-400 animate-pulse" : "text-gray-450"}`}>
                      ● {vagusNerveStatus}
                    </span>
                    <span className="text-[8px] text-gray-550 mt-1 block">CHOLINERGIC TRAIN</span>
                  </div>
                </div>

                {/* Simulated Waveform Visualizer */}
                <div className="p-3 bg-gray-950/60 rounded border border-gray-900 space-y-1.5">
                  <span className="text-[9px] text-gray-400 font-mono uppercase block">Real-time Biosensor Interstitial Frequency</span>
                  <div className="h-12 w-full bg-slate-950/80 rounded border border-gray-900/50 flex items-center justify-center relative overflow-hidden">
                    <svg className="w-full h-full" viewBox="0 0 400 100" preserveAspectRatio="none">
                      <path
                        d={`M 0,50 ${waveSeed.map((val, idx) => `L ${(400 / 23) * idx},${val + (simRunning ? Math.sin(idx + simulationStep) * 15 : 0)}`).join(" ")}`}
                        fill="none"
                        stroke="#06b6d4"
                        strokeWidth="2"
                        className="transition-all duration-300"
                      />
                    </svg>
                  </div>
                </div>
              </div>
            )}

            {/* 4. UNIVERSAL TOLERANCE PROTOCOL (UTP) */}
            {activeVector === "utp" && (
              <div className="space-y-4 animate-fadeIn">
                <p className="text-xs text-gray-400 leading-normal font-sans">
                  Expanding the system logic from specific SLE therapy to a categorical, trans-systemic level. Masking identification rings (HLA complexes) and deploying tissue-specific regulatory T-cells (CAR-Tregs) creates local "Halos of Tolerance".
                </p>

                <div className="space-y-3">
                  <label className="text-[10px] text-gray-400 font-mono uppercase block">Target Autoimmune failure domain</label>
                  <div className="grid grid-cols-4 gap-2">
                    {(["SLE", "T1D", "MS", "RA"] as const).map((tgt) => (
                      <button
                        key={tgt}
                        onClick={() => {
                          setUtpTarget(tgt);
                          handleResetSimulation();
                        }}
                        className={`py-1.5 px-2 text-center rounded text-[10px] font-mono border transition-all ${
                          utpTarget === tgt
                            ? "bg-cyan-500/10 text-cyan-300 border-cyan-500/40 font-bold"
                            : "bg-gray-950/40 text-gray-550 border-gray-900 hover:text-gray-300"
                        }`}
                      >
                        {tgt === "SLE" && "SLE (Lupus)"}
                        {tgt === "T1D" && "T1D (Insulin)"}
                        {tgt === "MS" && "MS (Myelin)"}
                        {tgt === "RA" && "RA (Joints)"}
                      </button>
                    ))}
                  </div>
                </div>

                <div className="p-3 bg-gray-950/60 border border-gray-900 rounded-lg flex items-center justify-between font-mono">
                  <span className="text-[10px] text-gray-400 uppercase">Tissue Domain Halo Integrity:</span>
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-bold text-emerald-400">{haloIntegrity}%</span>
                    <div className="w-24 h-2 bg-gray-900 rounded-full overflow-hidden">
                      <div className="h-full bg-emerald-400 transition-all duration-300" style={{ width: `${haloIntegrity}%` }} />
                    </div>
                  </div>
                </div>

                <div className="p-3 bg-[#0a0c10]/90 rounded border border-gray-900 text-[11px] text-gray-400 leading-relaxed font-mono">
                  🛡️ <strong>Halo Blueprint:</strong> {utpTarget === "T1D" && "Engineering CAR-Tregs directed specifically to the Islets of Langerhans to release protective, stabilizing IL-10 cytokine rings."}
                  {utpTarget === "MS" && "Deploying BBB-crossing CAR-Tregs targeting the myelin sheath matrix, stabilizing neural cells."}
                  {utpTarget === "RA" && "Infusing engineered Treg guards directly into the synovial fluid compartment of the joints to resolve arthritis noise."}
                  {utpTarget === "SLE" && "Using inverse polymer glyco-nanoparticles to present antigens to the liver, triggering systemic delete keys for auto-antibodies."}
                </div>
              </div>
            )}

            {/* 5. AUGMENTED RESILIENCE (ARP) */}
            {activeVector === "arp" && (
              <div className="space-y-4 animate-fadeIn text-left">
                <p className="text-xs text-gray-400 leading-normal font-sans">
                  The Exospheric Handshake. Upgrading human biology to neutralize anthropogenic toxicities, microplastics, environmental forever-chemicals (PFAS), and synthesizing zero-day immunity circuits.
                </p>

                <div className="grid grid-cols-3 gap-2">
                  {(["xenobiotic", "phoenix", "generative"] as const).map((eng) => (
                    <button
                      key={eng}
                      onClick={() => {
                        setEngineType(eng);
                        handleResetSimulation();
                      }}
                      className={`py-1.5 px-2 text-center rounded text-[10px] font-mono border transition-all ${
                        engineType === eng
                          ? "bg-cyan-500/10 text-cyan-300 border-cyan-500/40 font-bold"
                          : "bg-gray-950/40 text-gray-550 border-gray-900 hover:text-gray-300"
                      }`}
                    >
                      {eng === "xenobiotic" && "Xenobiotic Digestion"}
                      {eng === "phoenix" && "Phoenix Healing"}
                      {eng === "generative" && "Generative Immune"}
                    </button>
                  ))}
                </div>

                <div className="p-4 bg-gray-950/65 border border-gray-905 rounded-lg font-mono text-xs space-y-2">
                  {engineType === "xenobiotic" && (
                    <div className="flex justify-between items-center">
                      <span>Peroxisome PFAS Digestion Rate:</span>
                      <span className="font-bold text-cyan-400">{xenobioticDigestionRate}% of load/hr</span>
                    </div>
                  )}

                  {engineType === "phoenix" && (
                    <div className="flex justify-between items-center">
                      <span>Tissue Morphogenesis Multiplier (Yamanaka):</span>
                      <span className="font-bold text-amber-500">{repairRateMultiplier}x normal healing</span>
                    </div>
                  )}

                  {engineType === "generative" && (
                    <div className="flex justify-between items-center">
                      <span>Zero-Day Antigen Shield Signature:</span>
                      <span className={`font-bold ${zeroDayImmunityLoaded ? "text-emerald-400" : "text-gray-500"}`}>
                        {zeroDayImmunityLoaded ? "LOADED & MONITORING" : "WAITING SEED SEQUENCE"}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* ACTION TRIGGERS AREA */}
            <div className="flex gap-3 pt-2">
              <button
                disabled={simRunning}
                onClick={handleStartSimulation}
                className="flex-1 py-2 bg-cyan-600 hover:bg-cyan-500 disabled:bg-cyan-800/40 text-slate-950 text-xs font-bold font-mono tracking-wider rounded uppercase transition-all flex items-center justify-center gap-1.5 cursor-pointer shadow-[0_0_15px_rgba(6,182,212,0.2)]"
              >
                <Play className="w-4 h-4 fill-current" /> Initialize Calibration
              </button>
              
              <button
                onClick={handleResetSimulation}
                className="px-4 py-2 bg-gray-900 border border-gray-800 hover:bg-gray-850 hover:border-gray-700 text-slate-300 text-xs font-bold font-mono rounded transition-all cursor-pointer"
              >
                Reset Default
              </button>
            </div>

          </div>
        </div>

        {/* PANEL RIGHT: Real-time Live Log stream and system status */}
        <div className="lg:col-span-5 space-y-4">
          <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4 h-full flex flex-col justify-between">
            <div className="space-y-3">
              <p className="text-[10px] text-cyan-400 font-bold uppercase tracking-widest font-mono">
                Systemic Calibration Feed
              </p>
              
              {/* Dynamic Status Badges */}
              <div className="p-3 bg-gray-950/60 border border-gray-900 rounded-lg flex justify-between items-center text-xs font-mono">
                <span className="text-gray-450 uppercase">Operational State:</span>
                {simRunning ? (
                  <span className="text-amber-400 font-bold animate-pulse uppercase">TRANSMITTING CALIBRATION...</span>
                ) : simSuccess ? (
                  <span className="text-emerald-400 font-bold uppercase">HARMONY ACHIEVED 100%</span>
                ) : (
                  <span className="text-gray-450 uppercase">STANDING BY FOR INPUT...</span>
                )}
              </div>

              {/* Live Terminal Log Stream */}
              <div className="space-y-1.5">
                <span className="text-[9px] text-gray-500 uppercase font-semibold font-mono">Epigenetic Remodeling Log</span>
                <div className="h-44 bg-[#030406]/95 border border-gray-900/80 rounded p-3 font-mono text-[10px] leading-relaxed text-cyan-300/85 overflow-y-auto scrollbar-thin scrollbar-thumb-gray-900/90 flex flex-col space-y-1">
                  {systemLogs.length === 0 ? (
                    <span className="text-gray-650 italic">[Awaiting supervisor instruction sequence...]</span>
                  ) : (
                    systemLogs.map((log, index) => <div key={index}>{log}</div>)
                  )}
                </div>
              </div>
            </div>

            {/* Micro Graphic representation of closed loops */}
            <div className="p-3 bg-[#0d0f14]/40 border border-gray-850 rounded-lg font-mono text-[9px] space-y-2 mt-4 text-left">
              <span className="text-gray-400 uppercase text-[10px] font-bold block">Current System Topology Indicators:</span>
              <div className="grid grid-cols-2 gap-2">
                <div className="flex items-center gap-1.5">
                  <div className="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_4px_rgba(52,211,153,0.8)]" />
                  <span className="text-slate-350">Epigenetics: [Locked]</span>
                </div>
                <div className="flex items-center gap-1.5">
                  <div className={`w-2 h-2 rounded-full transition-all ${simSuccess ? "bg-emerald-400 shadow-[0_0_4px_rgba(52,211,153,0.8)]" : "bg-red-400 animate-pulse"}`} />
                  <span className="text-slate-350">CAR-T state: [Naive / Re-educated]</span>
                </div>
                <div className="flex items-center gap-1.5">
                  <div className="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_4px_rgba(52,211,153,0.8)]" />
                  <span className="text-slate-350">Nanobiosensors: [Active 293K]</span>
                </div>
                <div className="flex items-center gap-1.5">
                  <div className={`w-2 h-2 rounded-full transition-all ${simSuccess ? "bg-emerald-400 shadow-[0_0_4px_rgba(52,211,153,0.8)]" : "bg-gray-700"}`} />
                  <span className="text-slate-350">Augmented Shield: [Harmonized]</span>
                </div>
              </div>
            </div>

          </div>
        </div>
      </div>
    </div>
  );
}
