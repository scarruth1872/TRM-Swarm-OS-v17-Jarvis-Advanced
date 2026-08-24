import React, { useState } from "react";
import { 
  Brain, 
  RefreshCw, 
  TrendingDown, 
  Sparkles, 
  CheckCircle2, 
  FileText, 
  Play, 
  Terminal, 
  AlertTriangle, 
  ArrowRight,
  ShieldCheck,
  Zap,
  Info
} from "lucide-react";

interface Iteration {
  iterationIndex: number;
  inferencePrompt: string;
  lossVal: number;
  coherenceVal: number;
  errorCorrectionApplied: string;
  refinedOutput: string;
}

interface MathematicalState {
  trpEquation: string;
  globalSpotlightStrength: string;
  conflictRatio: string;
}

interface AscilineResult {
  overview: string;
  mathematicalState: MathematicalState;
  iterations: Iteration[];
  tacticalPlaybook: string[];
  witnessTraceId: string;
}

interface AscilineEngineProps {
  speakText?: (text: string) => void;
}

export const AscilineEngine: React.FC<AscilineEngineProps> = ({ speakText }) => {
  const [hypothesis, setHypothesis] = useState(
    "Optimize core cognitive state synchronization loops with automatic microservice failover nodes"
  );
  const [isRunning, setIsRunning] = useState(false);
  const [currentStage, setCurrentStage] = useState<number | null>(null);
  const [report, setReport] = useState<AscilineResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [logMessages, setLogMessages] = useState<string[]>([]);

  const addLog = (msg: string) => {
    setLogMessages((prev) => [...prev, `[${new Date().toLocaleTimeString()}] ${msg}`]);
  };

  const handleProcess = async () => {
    if (!hypothesis.trim() || isRunning) return;

    setIsRunning(true);
    setReport(null);
    setError(null);
    setLogMessages([]);
    setCurrentStage(1);

    if (speakText) {
      speakText(`Initializing ASCILINE self-correcting inference chain for target hypothesis. Monitoring feedback convergence loops in parallel. Standby, Boss.`);
    }

    addLog("ASCILINE: Initializing Autonomous Self-Correcting Iterative Learning core.");
    addLog("UQI INTEGRATION: Binding Recurrent Processing mathematics and Coherence Interpreter sensors.");

    // Stage 1: Inference Generation
    setTimeout(() => {
      setCurrentStage(2);
      addLog("STAGE 1 ACTIVE: Drawing base hypothesis maps and checking hardware bounding contexts...");
    }, 1200);

    // Stage 2: Critical Coherence check
    setTimeout(() => {
      setCurrentStage(3);
      addLog("STAGE 2 ACTIVE: Criticizing initial drafts for latent memory limits or race conditions. Computing error gradient...");
    }, 2500);

    // Stage 3: Error Refinement Loop
    setTimeout(() => {
      setCurrentStage(4);
      addLog("STAGE 3 ACTIVE: Running iterative self-corrections. Pruning redundant allocation branches...");
    }, 3800);

    // Stage 4: Spotlight selection & server query
    setTimeout(async () => {
      addLog("STAGE 4: Spotlight attention selected. Dispatching conscious candidates to Workspace Blackboard.");
      try {
        const localVoltage = parseFloat(localStorage.getItem("trm_voltage") || "1.05");
        const localMode = localStorage.getItem("trm_quantizer_mode") || "bitnet";
        const localNoise = parseFloat(localStorage.getItem("trm_noise_level") || "5");

        const response = await fetch("/api/asciline/process", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ 
            hypothesis,
            voltage: localVoltage,
            quantizerMode: localMode,
            noiseLevel: localNoise
          }),
        });

        if (!response.ok) {
          throw new Error("Cognitive link failed to synthesize inputs.");
        }

        const data: AscilineResult = await response.json();
        setReport(data);
        setCurrentStage(5);
        addLog(`STAGE 5 ACTIVE: Witness sequence audit committed successfully. Trace ID: ${data.witnessTraceId}`);
        addLog("ASCILINE: Inference chain stabilized successfully. Convergence accomplished.");
        
        if (speakText) {
          speakText(`Self-correction loops complete, Boss. Cognitive convergence reached with a final coherence score of ${data.iterations[2].coherenceVal} percent.`);
        }
      } catch (err: any) {
        setError(err.message || "Synthesizer lost communication pathways.");
        addLog("ERROR: Cognitive integration link reported an unexpected fault.");
        if (speakText) {
          speakText("Warning Boss. Self-correcting sequence execution faulted on final synthesis.");
        }
      } finally {
        setIsRunning(false);
      }
    }, 4800);
  };

  return (
    <div className="space-y-4 animate-fadeIn text-gray-300 font-sans">
      {/* Tab Header Banner */}
      <div className="border-b border-gray-900 pb-2.5 flex justify-between items-center">
        <p className="text-[10px] text-blue-400 font-bold uppercase tracking-widest flex items-center gap-1.5">
          <Brain className="w-3.5 h-3.5 text-blue-400 animate-pulse" />
          <span>ASCILINE Cognitive Engine (UQI Plug-in)</span>
        </p>
        <span className="text-[8px] font-mono text-gray-500 uppercase bg-gray-950 border border-gray-900 px-1.5 py-0.5 rounded">
          SELF_CORRECT_ENG_V1.4
        </span>
      </div>

      {/* Integration Context Card */}
      <div className="bg-black/40 border border-gray-900 rounded p-3 text-[10px] leading-relaxed text-gray-400 font-mono flex items-start gap-2.5 shadow-inner">
        <Info className="w-4 h-4 text-blue-400 shrink-0 mt-0.5" />
        <div className="space-y-1">
          <div className="text-gray-200 uppercase font-semibold text-[11px] tracking-wider flex items-center gap-1">
            <span>UQI / ASCILINE Harmonized Architecture</span>
          </div>
          <p className="leading-normal text-gray-405">
            This module represents the selective extraction of YusufB5's **ASCILINE** framework integrated into J.A.R.V.I.S.
            It deploys a **Transcendent Recurrent Processing (TRP)** loop to compute mathematical errors inside complex reasoning,
            publishes attention-spotlight findings into a shared **Global Workspace Blackboard**, and logs order transactions with a **Witness Correlator**.
          </p>
        </div>
      </div>

      {/* Main Grid: Control & Mathematical Indicators */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        {/* Hypotheses Control Box */}
        <div className="md:col-span-2 bg-[#05070a] border border-gray-900/80 rounded-lg p-3 flex flex-col justify-between">
          <div className="space-y-2">
            <label className="text-[9px] text-blue-400 font-bold uppercase tracking-wider block font-mono">
              Target Analytical Hypothesis
            </label>
            <textarea
              value={hypothesis}
              onChange={(e) => setHypothesis(e.target.value)}
              disabled={isRunning}
              rows={3}
              placeholder="Enter technical hypothesis or architectural dilemma to iterate..."
              className="w-full bg-black/60 border border-gray-800 focus:border-blue-500 rounded p-2 text-xs font-mono text-gray-100 placeholder-gray-600 focus:outline-none resize-none transition-colors"
            />
          </div>

          <div className="flex items-center justify-between mt-3 pt-2.5 border-t border-gray-950">
            <span className="text-[8px] text-gray-550 font-mono uppercase">
              RECURRENT FEEDBACK LOOPS: 3 ACTIVE
            </span>
            <button
              onClick={handleProcess}
              disabled={isRunning || !hypothesis.trim()}
              className={`px-3 py-1.5 text-[9px] uppercase font-bold font-mono tracking-wider rounded transition-all flex items-center gap-1.5 cursor-pointer ${
                isRunning
                  ? "bg-blue-950/40 text-blue-400 border border-blue-900/60 animate-pulse pointer-events-none"
                  : "bg-blue-600 hover:bg-blue-500 text-white shadow-[0_0_8px_rgba(37,99,235,0.4)] hover:shadow-[0_0_12px_rgba(37,99,235,0.6)]"
              }`}
            >
              <Play className="w-3 h-3 shrink-0" />
              {isRunning ? "PROCESSING ASCILINE SYNAPSE..." : "RUN ITERATIVE INFERENCE"}
            </button>
          </div>
        </div>

        {/* Live Mathematical Indicators */}
        <div className="bg-[#05070a] border border-gray-900/80 rounded-lg p-3 space-y-3 font-mono">
          <label className="text-[9px] text-blue-400 font-bold uppercase tracking-wider block border-b border-gray-950 pb-1 flex items-center gap-1">
            <Zap className="w-3 h-3 text-amber-400" />
            <span>Mathematical State Sensors</span>
          </label>
          
          <div className="space-y-2 text-[10px]">
            <div className="space-y-0.5">
              <span className="text-[8px] text-gray-500 uppercase block">TRP Eq. Convergence</span>
              <div className="bg-black/50 border border-gray-900 rounded px-2.5 py-1 text-sky-400 font-semibold truncate">
                {report ? report.mathematicalState.trpEquation : "C_recurrent = ∫ f(t) e^(-λt) dt"}
              </div>
            </div>

            <div className="grid grid-cols-2 gap-2">
              <div className="space-y-0.5">
                <span className="text-[8px] text-gray-500 uppercase block">Attention Focus</span>
                <div className="bg-black/50 border border-gray-900 rounded px-2.5 py-1 text-amber-400 font-bold">
                  {report ? report.mathematicalState.globalSpotlightStrength : "0.0% standby"}
                </div>
              </div>
              <div className="space-y-0.5">
                <span className="text-[8px] text-gray-500 uppercase block">Conflict Ratio</span>
                <div className="bg-black/50 border border-gray-900 rounded px-2.5 py-1 text-emerald-400 font-bold">
                  {report ? report.mathematicalState.conflictRatio : "0.000 (stable)"}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Stage Tracker Indicators */}
      {isRunning && (
        <div className="bg-blue-950/20 border border-blue-900/30 rounded-lg p-3 font-mono text-[10px] space-y-2">
          <div className="flex justify-between items-center text-[9px] text-blue-400 font-bold uppercase">
            <span>COGNITIVE CORE STATE: ASCILINE INFERENCE</span>
            <span className="animate-pulse">Active Step {currentStage}/5</span>
          </div>
          
          <div className="grid grid-cols-5 gap-1.5">
            {[
              { idx: 1, label: "INIT" },
              { idx: 2, label: "INFERENCE GEN" },
              { idx: 3, label: "COHERENCE CRITIQUE" },
              { idx: 4, label: "SPOTLIGHT EVAL" },
              { idx: 5, label: "WITNESS AUDIT" }
            ].map((stg) => {
              const active = currentStage !== null && currentStage >= stg.idx;
              const isCurrent = currentStage !== null && currentStage === stg.idx;
              return (
                <div 
                  key={stg.idx} 
                  className={`text-center py-1 rounded border transition-all text-[8px] font-bold ${
                    isCurrent 
                      ? "bg-blue-500/20 text-blue-300 border-blue-500 animate-pulse" 
                      : active 
                      ? "bg-blue-950/30 text-blue-400 border-blue-900/50" 
                      : "bg-[#020305] text-gray-650 border-gray-950"
                  }`}
                >
                  <div className="text-[10px]">{stg.idx}</div>
                  <div className="hidden sm:block text-[6px] truncate">{stg.label}</div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Error Alert */}
      {error && (
        <div className="p-3 bg-red-950/40 border border-red-900/50 text-red-400 rounded-lg text-xs font-mono flex items-center gap-2">
          <AlertTriangle className="w-4 h-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Log Messages Output */}
      {logMessages.length > 0 && (
        <div className="space-y-1">
          <span className="text-[8px] text-gray-500 uppercase font-bold font-mono block">Real-time Pipeline Telemetry logs</span>
          <div className="bg-black/60 border border-gray-900 rounded p-2.5 font-mono text-[9px] text-emerald-400 leading-relaxed max-h-[100px] overflow-y-auto space-y-0.5 select-all">
            {logMessages.map((log, id) => (
              <div key={id}>{log}</div>
            ))}
          </div>
        </div>
      )}

      {/* Compiled Results Section */}
      {report && (
        <div className="space-y-4 animate-fadeIn">
          {/* Executive Overview */}
          <div className="bg-[#05070a] border border-gray-900 rounded-lg p-3 space-y-2">
            <span className="text-[9px] text-blue-400 font-bold uppercase tracking-wider block font-mono flex items-center gap-1.5">
              <ShieldCheck className="w-3.5 h-3.5 text-blue-400" />
              <span>Executive Synthesis Briefing</span>
            </span>
            <p className="text-xs text-gray-350 leading-relaxed font-sans font-medium">
              {report.overview}
            </p>
          </div>

          {/* Tabular Iterations */}
          <div className="space-y-2.5">
            <span className="text-[9px] text-blue-400 font-bold uppercase tracking-wider block font-mono flex items-center gap-1">
              <RefreshCw className="w-3.5 h-3.5 text-blue-400" />
              <span>ASCILINE Self-Correction Mathematical Iteration Steps</span>
            </span>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              {report.iterations.map((iter) => (
                <div 
                  key={iter.iterationIndex} 
                  className={`bg-[#030508] border rounded-lg p-3.5 space-y-3 relative ${
                    iter.iterationIndex === 3 
                      ? "border-emerald-900/60 shadow-[0_0_12px_rgba(16,185,129,0.04)]" 
                      : "border-gray-900"
                  }`}
                >
                  {/* Step Badge */}
                  <div className="flex justify-between items-center border-b border-gray-950 pb-1.5">
                    <span className="text-[10px] font-bold text-gray-200 font-mono">
                      ITERATION #{iter.iterationIndex}
                    </span>
                    <span className={`text-[8px] px-1.5 py-0.5 rounded font-mono font-bold ${
                      iter.iterationIndex === 1 
                        ? "bg-red-950/60 text-red-400 border border-red-900/30" 
                        : iter.iterationIndex === 2 
                        ? "bg-amber-950/60 text-amber-500 border border-amber-900/30" 
                        : "bg-emerald-950 text-emerald-400 border border-emerald-900/30"
                    }`}>
                      {iter.iterationIndex === 1 ? "BASE DRAFT" : iter.iterationIndex === 2 ? "CRITICIZED" : "CONVERGED"}
                    </span>
                  </div>

                  {/* Convergence Math Indicator */}
                  <div className="grid grid-cols-2 gap-1.5 text-[10px] font-mono">
                    <div className="bg-black/40 border border-gray-950 rounded p-1 text-center">
                      <span className="text-[7px] text-gray-550 block uppercase flex items-center justify-center gap-0.5">
                        <TrendingDown className="w-2.5 h-2.5 text-red-400 shrink-0" /> Loss
                      </span>
                      <span className="text-red-400 font-bold text-[11px]">{iter.lossVal}</span>
                    </div>
                    <div className="bg-black/40 border border-gray-950 rounded p-1 text-center">
                      <span className="text-[7px] text-gray-550 block uppercase flex items-center justify-center gap-0.5">
                        <CheckCircle2 className="w-2.5 h-2.5 text-emerald-400 shrink-0" /> Coherence
                      </span>
                      <span className="text-emerald-400 font-bold text-[11px]">{iter.coherenceVal}%</span>
                    </div>
                  </div>

                  {/* Prompts, Critiques & Corrections */}
                  <div className="space-y-1 text-[10px] font-mono leading-relaxed border-t border-gray-950 pt-2 text-gray-405">
                    <div className="space-y-0.5">
                      <span className="text-[7px] text-gray-500 uppercase block">Critique Action Prompt</span>
                      <p className="text-gray-300 text-[10px] italic">"{iter.inferencePrompt}"</p>
                    </div>
                    
                    <div className="space-y-0.5 border-t border-gray-950/50 pt-1 mt-1">
                      <span className="text-[7px] text-blue-500 uppercase block">Error Correction Applied</span>
                      <p className="text-blue-400 text-[10.5px] leading-normal">{iter.errorCorrectionApplied}</p>
                    </div>
                  </div>

                  {/* Refined Blueprint Output */}
                  <div className="space-y-1 font-mono text-[10px] pt-2 border-t border-gray-950">
                    <span className="text-[7px] text-emerald-500 uppercase block">Refined Output Solution</span>
                    <div className="bg-black/70 border border-gray-950 rounded p-2 text-gray-250 leading-relaxed overflow-x-auto max-h-[140px] whitespace-pre-wrap font-sans">
                      {iter.refinedOutput}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 pb-2">
            {/* Tactical Playbook Card */}
            <div className="bg-[#05070a] border border-gray-900 rounded-lg p-3.5 space-y-2">
              <span className="text-[9px] text-blue-400 font-bold uppercase tracking-wider block font-mono flex items-center gap-1.5">
                <Terminal className="w-3.5 h-3.5 text-sky-450" />
                <span>Coherence Interpreter Tactical Playlist</span>
              </span>
              <ul className="space-y-2 text-xs font-mono text-gray-400 leading-normal pl-1.5">
                {report.tacticalPlaybook.map((step, sIdx) => (
                  <li key={sIdx} className="flex items-start gap-1.5">
                    <ArrowRight className="w-3.5 h-3.5 text-blue-500 shrink-0 mt-0.5" />
                    <span>{step}</span>
                  </li>
                ))}
              </ul>
            </div>

            {/* Blackboard & Witness Audit Card */}
            <div className="bg-[#05070a] border border-gray-900 rounded-lg p-3.5 space-y-3 font-mono">
              <span className="text-[9px] text-blue-400 font-bold uppercase tracking-wider block border-b border-gray-950 pb-1.5 flex items-center gap-1.5">
                <FileText className="w-3.5 h-3.5 text-blue-400" />
                <span>Audit & Blackboard Sequence</span>
              </span>
              
              <div className="space-y-2.5 text-[10px]">
                <div className="space-y-1">
                  <span className="text-[8px] text-gray-500 uppercase block">Witness Transaction Trace Identifier</span>
                  <div className="bg-black/55 border border-gray-950 rounded px-2.5 py-1 text-sky-400 font-bold select-all flex justify-between items-center text-[10.5px]">
                    <span>{report.witnessTraceId}</span>
                    <span className="text-[7px] text-gray-650 bg-gray-950 px-1 border border-gray-900 uppercase font-normal font-mono select-none">
                      SECURED BY HSM
                    </span>
                  </div>
                </div>

                <div className="space-y-1">
                  <span className="text-[8px] text-gray-500 uppercase block">Global Blackboard High-Priority Focus Candidate</span>
                  <div className="bg-black/55 border border-gray-950 rounded px-2.5 py-2 text-amber-500 leading-normal">
                    <div className="font-semibold uppercase text-[8px] text-amber-600 mb-0.5">SPOTLIGHT_CONSCIOUS_POSTURE:</div>
                    <span className="italic leading-relaxed">"CONSTRUCT DECOUPLED ARCHITECTURE VERIFIED BY WITNESS TRACE SEQUENCING LOGS"</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
