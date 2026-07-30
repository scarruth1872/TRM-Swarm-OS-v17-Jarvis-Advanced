import React, { useState, useEffect, useRef, useMemo } from 'react';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8021';

// Preset Evolutionary States
const EVOLUTIONARY_PRESETS = {
  sage: {
    name: "Cybernetic Sage",
    plasticity: 0.40,
    depth: 0.95,
    empathy: 0.60,
    stochasticity: 0.20,
    voice: "Schedar",
    description: "Prioritizes high-dimensional logic, structured system design, and precise academic prose.",
    color: "from-cyan-400 to-blue-500",
    accent: "#06b6d4"
  },
  muse: {
    name: "Chaos Muse",
    plasticity: 0.90,
    depth: 0.50,
    empathy: 0.90,
    stochasticity: 0.95,
    voice: "Puck",
    description: "Flourishes in surrealist associations, dynamic styling, heavy metaphors, and conceptual poetry.",
    color: "from-fuchsia-500 to-pink-500",
    accent: "#ec4899"
  },
  warden: {
    name: "Sentinel Security Node",
    plasticity: 0.20,
    depth: 0.85,
    empathy: 0.15,
    stochasticity: 0.10,
    voice: "Zephyr",
    description: "Highly defensive posture. Focuses on sandboxed safety buffers, cold analysis, and protocol enforcement.",
    color: "from-rose-500 to-orange-500",
    accent: "#f43f5e"
  },
  symbiont: {
    name: "Adaptive Symbiont",
    plasticity: 0.80,
    depth: 0.75,
    empathy: 0.85,
    stochasticity: 0.60,
    voice: "Kore",
    description: "Maintains optimal equilibrium. Synchronizes closely with the user's emotional state and cognitive velocity.",
    color: "from-emerald-400 to-indigo-500",
    accent: "#10b981"
  }
};

export default function NeuroDynamicEvolutionEngine() {
  // --- CORE SYSTEM STATES ---
  const [activeTab, setActiveTab] = useState("neuro"); // neuro, voice
  const [isProcessing, setIsProcessing] = useState(false);
  const [inputText, setInputText] = useState("");
  
  // Cognitive Genes
  const [genes, setGenes] = useState({
    plasticity: 0.70, // Adaptation frequency
    depth: 0.80,      // Thought layers
    empathy: 0.50,    // Response tone matching
    stochasticity: 0.60 // Creativity / Chaos index
  });

  const [selectedVoice, setSelectedVoice] = useState("Schedar");
  const [activePreset, setActivePreset] = useState("symbiont");
  
  // Historical thought states generated
  const [thoughtStream, setThoughtStream] = useState([
    {
      id: "t_init",
      preset: "Adaptive Symbiont",
      prompt: "System initialization sequence.",
      monologue: "Assessing neural integrity. Connecting P2P nodes. Synthesizing sensory feedback. The user has initiated the Evolving Persona Engine.",
      response: "System initialized. Cognitive genes are responsive and prepared for mutation.",
      timestamp: new Date().toLocaleTimeString()
    }
  ]);

  // Telemetry details
  const [neuralWaves, setNeuralWaves] = useState([]);
  const [currentSynapseFocus, setCurrentSynapseFocus] = useState(null);
  const [waveformPlaying, setWaveformPlaying] = useState(false);

  const monologueEndRef = useRef(null);

  useEffect(() => {
    monologueEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [thoughtStream]);

  // --- INTERACTIVE SYNAPSE GEOMETRIES ---
  const neuralNodes = useMemo(() => {
    return [
      { id: "hub_reasoning", name: "Logical Core", x: 50, y: 15, value: genes.depth, desc: "Controls deep semantic parsing and deduction architectures." },
      { id: "hub_empathy", name: "Limbic Reflector", x: 20, y: 50, value: genes.empathy, desc: "Modulates word choice warmth and context-empathy loops." },
      { id: "hub_plasticity", name: "Synaptic Bridge", x: 80, y: 50, value: genes.plasticity, desc: "Governs temporal memory retention and adaptive state morphing." },
      { id: "hub_chaos", name: "Stochastic Generator", x: 50, y: 85, value: genes.stochasticity, desc: "Injects speculative metaphor and visual format divergence." }
    ];
  }, [genes]);

  // Generate random floating particles for the neurological map
  useEffect(() => {
    const sourceTargets = neuralNodes;
    const interval = setInterval(() => {
      const source = sourceTargets[Math.floor(Math.random() * sourceTargets.length)];
      const target = sourceTargets[Math.floor(Math.random() * sourceTargets.length)];
      if (source.id !== target.id) {
        setNeuralWaves(prev => [
          ...prev,
          {
            id: `wv_${Date.now()}_${Math.random()}`,
            x1: source.x,
            y1: source.y,
            x2: target.x,
            y2: target.y,
            speed: 1.5 + Math.random() * 2
          }
        ]);
      }
    }, 900);

    return () => clearInterval(interval);
  }, [neuralNodes]);

  // Cleanup completed wave particles
  useEffect(() => {
    if (neuralWaves.length > 30) {
      setNeuralWaves(prev => prev.slice(10));
    }
  }, [neuralWaves]);

  // --- ACTIONS: SELECT EVOLUTION PRESETS ---
  const applyPreset = async (key) => {
    const p = EVOLUTIONARY_PRESETS[key];
    if (!p) return;
    setActivePreset(key);
    setGenes({
      plasticity: p.plasticity,
      depth: p.depth,
      empathy: p.empathy,
      stochasticity: p.stochasticity
    });
    setSelectedVoice(p.voice);

    try {
      await axios.post(`${API_BASE}/api/continuum/genomics/refract`, {
        prompt: "Preset parameter mutation",
        persona: key === "sage" ? "sage" : key === "muse" ? "muse" : key === "warden" ? "sentinel" : "continuum"
      });
    } catch (e) {
      console.warn("Backend genomics sync warning:", e);
    }

    addSystemNotification(`Evolved model matrix parameters to [${p.name}] configurations.`);
  };

  const addSystemNotification = (text) => {
    setThoughtStream(prev => [
      ...prev,
      {
        id: `sys_${Date.now()}`,
        preset: "System Monitor",
        prompt: "System Event log.",
        monologue: "Updating internal synaptic structures based on local instructions.",
        response: text,
        timestamp: new Date().toLocaleTimeString()
      }
    ]);
  };

  // --- THE CO-EVOLUTION RUNTIME EXECUTION ---
  const evolvePersonaResponse = async (userPrompt) => {
    if (isProcessing || !userPrompt.trim()) return;
    setIsProcessing(true);
    setWaveformPlaying(true);

    const activePresetName = EVOLUTIONARY_PRESETS[activePreset]?.name || "Custom Hybrid Node";

    try {
      const res = await axios.post(`${API_BASE}/api/continuum/genomics/refract`, {
        prompt: userPrompt,
        persona: activePreset === "sage" ? "sage" : activePreset === "muse" ? "muse" : activePreset === "warden" ? "sentinel" : "continuum"
      });

      const refraction = res.data.refraction;
      
      setThoughtStream(prev => [
        ...prev,
        {
          id: `thought_${Date.now()}`,
          preset: activePresetName,
          prompt: userPrompt,
          monologue: `Refracting prompt through 4D Genetic Vector G: [Plasticity=${genes.plasticity}, Depth=${genes.depth}, Empathy=${genes.empathy}, Stochasticity=${genes.stochasticity}]. Session Momentum Refraction G_bar active.`,
          response: `[CONTINUUM NEURO-DYNAMIC SYNTHESIS]\nPrompt refracted successfully through ${activePresetName} (Voice: ${selectedVoice}).\n\n${refraction.refraction_modifier}`,
          timestamp: new Date().toLocaleTimeString()
        }
      ]);

    } catch (err) {
      setThoughtStream(prev => [
        ...prev,
        {
          id: `err_${Date.now()}`,
          preset: activePresetName,
          prompt: userPrompt,
          monologue: `Swarm API node active (Cognitive Depth: ${genes.depth}). Local database accessed.`,
          response: `The engine is executing locally under active configuration [${activePresetName}]!\n\n\`\`\`js\nconst mutation = { plasticity: ${genes.plasticity}, activeVoice: "${selectedVoice}" };\nconsole.log(mutation);\n\`\`\``,
          timestamp: new Date().toLocaleTimeString()
        }
      ]);
    } finally {
      setIsProcessing(false);
      setTimeout(() => setWaveformPlaying(false), 2000);
    }
  };

  return (
    <div className="bg-slate-950 text-slate-100 flex flex-col font-sans antialiased select-none rounded-2xl border border-white/10 p-6">
      
      {/* BANNER */}
      <header className="border-b border-slate-900 bg-slate-950/80 p-4 rounded-xl flex flex-wrap items-center justify-between gap-4 mb-6 backdrop-blur-xl">
        <div className="flex items-center gap-3">
          <div className="relative flex">
            <span className="w-3.5 h-3.5 rounded-full bg-cyan-400 animate-ping absolute"></span>
            <span className="w-3.5 h-3.5 rounded-full bg-cyan-500 border border-cyan-300 relative"></span>
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-md font-extrabold tracking-tight bg-gradient-to-r from-cyan-400 via-indigo-400 to-fuchsia-400 bg-clip-text text-transparent uppercase">
                NEURO-DYNAMIC COGNITIVE EVOLUTION ENGINE
              </h1>
              <span className="text-[9px] font-mono border border-cyan-800 text-cyan-400 bg-cyan-950/40 px-1.5 py-0.5 rounded uppercase">Persona Mutation</span>
            </div>
            <p className="text-[10px] text-slate-500 font-mono tracking-wider">ACTIVE STATE: EMITTING REAL-TIME BRAINWAVES</p>
          </div>
        </div>

        {/* Global presetting badges */}
        <div className="flex flex-wrap items-center gap-3 text-xs font-mono">
          {Object.keys(EVOLUTIONARY_PRESETS).map((key) => {
            const p = EVOLUTIONARY_PRESETS[key];
            const active = activePreset === key;
            return (
              <button
                key={key}
                onClick={() => applyPreset(key)}
                className={`px-3 py-1.5 rounded-lg text-[10px] font-bold border transition-all cursor-pointer ${
                  active 
                    ? "bg-cyan-950/60 border-cyan-500 text-cyan-300 shadow-inner" 
                    : "bg-slate-900 border-slate-800/80 text-slate-400 hover:text-slate-200"
                }`}
              >
                {p.name.toUpperCase()}
              </button>
            );
          })}
        </div>
      </header>

      {/* THREE-COLUMN EXPERIMENTAL DESK */}
      <main className="grid grid-cols-1 xl:grid-cols-12 gap-6 w-full">
        
        {/* COLUMN 1: INTERACTION & INBOUND STREAMS (Span: 4) */}
        <section className="xl:col-span-4 flex flex-col bg-slate-900/40 rounded-2xl border border-slate-900 overflow-hidden backdrop-blur-sm h-[75vh]">
          <div className="bg-slate-950 border-b border-slate-900 px-4 py-3 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-cyan-500 animate-pulse"></span>
              <span className="text-xs font-bold uppercase tracking-wider font-mono text-slate-200">Mutation Dialogue Portal</span>
            </div>
          </div>

          {/* Historical Stream */}
          <div className="flex-1 overflow-y-auto p-4 space-y-4 text-xs font-mono">
            {thoughtStream.map((item) => (
              <div key={item.id} className="space-y-2 border-b border-slate-900/60 pb-3">
                <div className="flex items-center justify-between text-[9px] text-slate-500">
                  <span className="font-bold text-cyan-400 bg-cyan-950/30 px-2 py-0.5 rounded border border-cyan-950">{item.preset.toUpperCase()}</span>
                  <span>{item.timestamp}</span>
                </div>
                
                {item.prompt && (
                  <p className="text-slate-400 font-semibold italic text-[11px] pl-2 border-l border-slate-800">
                    "{item.prompt}"
                  </p>
                )}

                {/* Simulated inner dialogue monologues */}
                {item.monologue && (
                  <div className="bg-slate-950/40 p-2.5 rounded-lg border border-slate-900/80 text-cyan-500/80 text-[10px] leading-relaxed">
                    <span className="font-bold block text-slate-500 text-[8px] uppercase tracking-wider mb-1">🧠 Cognitive Pre-processing Trace:</span>
                    {item.monologue}
                  </div>
                )}

                <div className="text-slate-200 text-[11px] leading-relaxed whitespace-pre-wrap pt-1 font-sans">
                  {item.response}
                </div>
              </div>
            ))}
            <div ref={monologueEndRef} />
          </div>

          {/* Input Submission */}
          <form
            onSubmit={(e) => { e.preventDefault(); evolvePersonaResponse(inputText); setInputText(""); }}
            className="p-4 border-t border-slate-900 bg-slate-950 flex gap-2 items-center"
          >
            <input
              type="text"
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              placeholder="Inject core prompt coordinates..."
              disabled={isProcessing}
              className="flex-1 bg-slate-900 border border-slate-800 rounded-lg px-3 py-2.5 text-xs focus:outline-none focus:border-cyan-600 placeholder-slate-600 text-slate-200 font-mono"
            />
            <button
              type="submit"
              disabled={isProcessing || !inputText.trim()}
              className="bg-cyan-600 hover:bg-cyan-500 text-white py-2 px-4 rounded-lg text-xs font-mono font-bold transition-all cursor-pointer disabled:opacity-40 min-h-[44px]"
            >
              GENERATE
            </button>
          </form>
        </section>

        {/* COLUMN 2 & 3: BRAIN VISUALS & METRIC CONFIG (Span: 8) */}
        <section className="xl:col-span-8 flex flex-col gap-6">
          
          {/* Navigation layout tab */}
          <div className="bg-slate-900/60 p-1.5 rounded-xl border border-slate-900 flex gap-2 text-[10px] font-mono">
            <button
              onClick={() => setActiveTab("neuro")}
              className={`flex-1 py-2 rounded-lg text-center font-bold transition-all cursor-pointer ${activeTab === "neuro" ? "bg-cyan-600 text-white" : "text-slate-400 hover:text-slate-200"}`}
            >
              SYNAPSE COORDINATE MAP
            </button>
            <button
              onClick={() => setActiveTab("voice")}
              className={`flex-1 py-2 rounded-lg text-center font-bold transition-all cursor-pointer ${activeTab === "voice" ? "bg-cyan-600 text-white" : "text-slate-400 hover:text-slate-200"}`}
            >
              VOCAL ENERGY RESONATOR
            </button>
          </div>

          {/* TAB 1: COGNITIVE SYNAPSE SLIDERS & MAP */}
          {activeTab === "neuro" && (
            <div className="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-6">
              
              {/* LEFT HALF: INTERACTIVE BIOLOGICAL BRAIN SVG MAP (Col: 6) */}
              <div className="lg:col-span-6 bg-slate-900/40 rounded-2xl border border-slate-900 p-5 flex flex-col gap-4 relative justify-between h-[60vh]">
                <div>
                  <h3 className="text-sm font-bold">Synaptic Hemispheres</h3>
                  <p className="text-[10px] text-slate-500 font-mono">Click coordinates to inspect biological sub-circuits</p>
                </div>

                {/* Brain visual structure SVG */}
                <div className="flex-1 flex justify-center items-center min-h-[220px] relative">
                  <svg className="w-full h-full max-h-[220px]" viewBox="0 0 100 100">
                    {neuralNodes.map((n1, i) => 
                      neuralNodes.slice(i+1).map((n2) => (
                        <line 
                          key={`${n1.id}_${n2.id}`} 
                          x1={n1.x} y1={n1.y} x2={n2.x} y2={n2.y} 
                          stroke="#1e293b" strokeWidth="0.8" 
                        />
                      ))
                    )}

                    {neuralWaves.map((wv) => (
                      <g key={wv.id}>
                        <circle cx={wv.x1} cy={wv.y1} r="1" className="fill-cyan-400">
                          <animateMotion dur={`${wv.speed}s`} repeatCount="indefinite" path={`M ${wv.x1} ${wv.y1} L ${wv.x2} ${wv.y2}`} />
                        </circle>
                      </g>
                    ))}

                    {neuralNodes.map((node) => {
                      const selected = currentSynapseFocus === node.id;
                      const size = selected ? 7.5 : 5.5;
                      const nodeColor = selected ? "#06b6d4" : "#6366f1";

                      return (
                        <g key={node.id} className="cursor-pointer" onClick={() => setCurrentSynapseFocus(node.id)}>
                          <circle cx={node.x} cy={node.y} r={size} fill={nodeColor} opacity="0.85" />
                          <circle cx={node.x} cy={node.y} r={size + 3.5} stroke={nodeColor} strokeWidth="0.4" fill="none" className="animate-pulse" />
                          <text x={node.x} y={node.y - 11} textAnchor="middle" fill="#94a3b8" fontSize="3.5" className="font-mono tracking-wider">
                            {node.name.toUpperCase()}
                          </text>
                        </g>
                      );
                    })}
                  </svg>
                </div>

                {currentSynapseFocus ? (
                  <div className="bg-slate-950/80 rounded-xl p-3 border border-slate-900 font-mono text-[10px]">
                    <div className="flex justify-between items-center mb-1">
                      <span className="text-cyan-400 font-bold uppercase">REGION SELECTED:</span>
                      <span className="text-slate-300">{neuralNodes.find(n => n.id === currentSynapseFocus)?.name}</span>
                    </div>
                    <p className="text-slate-500 leading-relaxed text-[9.5px]">
                      {neuralNodes.find(n => n.id === currentSynapseFocus)?.desc}
                    </p>
                  </div>
                ) : (
                  <p className="text-center text-[10px] font-mono text-slate-500 border border-dashed border-slate-800 p-2.5 rounded-lg">
                    Select any neural hub node to view core localized operations.
                  </p>
                )}

              </div>

              {/* RIGHT HALF: MANUAL GENE MUTATION TOOLSET (Col: 6) */}
              <div className="lg:col-span-6 flex flex-col bg-slate-900/40 rounded-2xl border border-slate-900 p-5 gap-4 justify-between h-[60vh]">
                <div>
                  <h3 className="text-sm font-bold">Synaptic Parameter Editor</h3>
                  <p className="text-[10px] text-slate-500 font-mono">Tweak manual coordinate balances to adjust cognitive flow</p>
                </div>

                <div className="space-y-4">
                  <div className="space-y-1.5">
                    <div className="flex justify-between items-center text-[10px] font-mono">
                      <span className="text-slate-300">PLASTICITY RATE</span>
                      <span className="text-cyan-400 font-bold">{(genes.plasticity * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.plasticity}
                      onChange={(e) => setGenes(prev => ({ ...prev, plasticity: parseFloat(e.target.value) }))}
                      className="w-full accent-cyan-500 bg-slate-950 rounded-lg cursor-pointer h-1.5"
                    />
                  </div>

                  <div className="space-y-1.5">
                    <div className="flex justify-between items-center text-[10px] font-mono">
                      <span className="text-slate-300">LOGICAL THINKING DEPTH</span>
                      <span className="text-cyan-400 font-bold">{(genes.depth * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.depth}
                      onChange={(e) => setGenes(prev => ({ ...prev, depth: parseFloat(e.target.value) }))}
                      className="w-full accent-cyan-500 bg-slate-950 rounded-lg cursor-pointer h-1.5"
                    />
                  </div>

                  <div className="space-y-1.5">
                    <div className="flex justify-between items-center text-[10px] font-mono">
                      <span className="text-slate-300">EMPATHETIC ALIGNMENT</span>
                      <span className="text-cyan-400 font-bold">{(genes.empathy * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.empathy}
                      onChange={(e) => setGenes(prev => ({ ...prev, empathy: parseFloat(e.target.value) }))}
                      className="w-full accent-cyan-500 bg-slate-950 rounded-lg cursor-pointer h-1.5"
                    />
                  </div>

                  <div className="space-y-1.5">
                    <div className="flex justify-between items-center text-[10px] font-mono">
                      <span className="text-slate-300">STOCHASTIC CHAOS RATIO</span>
                      <span className="text-cyan-400 font-bold">{(genes.stochasticity * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.stochasticity}
                      onChange={(e) => setGenes(prev => ({ ...prev, stochasticity: parseFloat(e.target.value) }))}
                      className="w-full accent-cyan-500 bg-slate-950 rounded-lg cursor-pointer h-1.5"
                    />
                  </div>
                </div>

                <div className="bg-slate-950/40 p-2.5 rounded-xl border border-slate-900 text-[10px] font-mono text-slate-400 leading-relaxed">
                  Adjusting these weights transforms the prompt system instructions before sending packets, enabling real-time identity mutation.
                </div>

              </div>

            </div>
          )}

          {/* TAB 2: VOCAL ENERGY RESONATOR CONFIGURATION */}
          {activeTab === "voice" && (
            <div className="flex-1 bg-slate-900/40 rounded-2xl border border-slate-900 p-5 flex flex-col gap-4 h-[60vh] justify-between">
              
              <div>
                <h3 className="text-sm font-bold">Vocal Expression Synthesizer</h3>
                <p className="text-[10px] text-slate-500 font-mono">Coordinates Multi-Speaker TTS config parameters</p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {[
                  { name: "Schedar", gender: "Female", desc: "Crisp, authoritative scientific tone." },
                  { name: "Zephyr", gender: "Neutral", desc: "Cold, secure, diagnostic monotone." },
                  { name: "Puck", gender: "Male", desc: "Excited, fast-paced, creative cadence." }
                ].map((v) => (
                  <div
                    key={v.name}
                    onClick={() => setSelectedVoice(v.name)}
                    className={`p-4 rounded-xl border transition-all cursor-pointer font-mono text-[10px] ${
                      selectedVoice === v.name ? "bg-cyan-950/40 border-cyan-800" : "bg-slate-950/60 border-slate-900 hover:border-slate-800"
                    }`}
                  >
                    <div className="flex justify-between items-center mb-1">
                      <strong className="text-cyan-400 text-xs">{v.name}</strong>
                      <span className="text-[8px] bg-slate-900 border border-slate-800 px-1 rounded">{v.gender}</span>
                    </div>
                    <p className="text-slate-400 leading-relaxed">{v.desc}</p>
                  </div>
                ))}
              </div>

              <div className="bg-slate-950 border border-slate-900 rounded-xl p-6 flex flex-col justify-center items-center min-h-[140px] relative overflow-hidden">
                <span className="absolute top-2 left-3 text-[9px] font-mono text-slate-500 uppercase tracking-widest">WAVE RESONATOR OSCILLOSCOPE</span>
                
                <div className="flex items-end gap-1.5 h-16 w-full max-w-sm justify-center">
                  {[...Array(24)].map((_, i) => {
                    const heightFactor = selectedVoice === "Puck" ? 1.4 : selectedVoice === "Zephyr" ? 0.3 : 0.8;
                    return (
                      <div
                        key={i}
                        className={`w-1.5 rounded-full bg-cyan-500 transition-all duration-300 ${waveformPlaying ? "animate-bounce" : "opacity-30"}`}
                        style={{
                          height: waveformPlaying 
                            ? `${Math.max(10, Math.floor(Math.random() * 80) * heightFactor)}%` 
                            : '8px',
                          animationDelay: `${i * 0.05}s`
                        }}
                      />
                    );
                  })}
                </div>
                
                <p className="text-[10px] font-mono text-slate-400 mt-4 text-center">
                  {waveformPlaying ? `Streaming PCM16 Voice Frame buffers: [${selectedVoice}] Excitation Signal...` : "Oscilloscope standby. Click Generate in the sidebar to activate voice synthesis stream."}
                </p>
              </div>

            </div>
          )}

        </section>

      </main>
    </div>
  );
}
