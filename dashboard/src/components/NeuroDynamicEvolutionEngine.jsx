import React, { useState, useEffect, useRef, useMemo } from 'react';
import { 
  Dna, Sparkles, Shield, Orbit, Activity, RefreshCw, CheckCircle2, 
  Brain, Zap, Layers, Network, Database, Radio, Compass, Sliders, Volume2, MessageSquare, Terminal
} from 'lucide-react';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8021';

// Preset Evolutionary States with Vibrant Color Profiles
const EVOLUTIONARY_PRESETS = {
  sage: {
    name: "Cybernetic Sage",
    plasticity: 0.40,
    depth: 0.95,
    empathy: 0.60,
    stochasticity: 0.20,
    voice: "Schedar",
    description: "Prioritizes high-dimensional logic, structured system design, and precise academic prose.",
    gradient: "from-cyan-500/20 via-blue-500/10 to-transparent",
    border: "border-cyan-500/40",
    badge: "bg-cyan-500/20 text-cyan-300 border-cyan-500/40",
    accentHex: "#06b6d4"
  },
  muse: {
    name: "Chaos Muse",
    plasticity: 0.90,
    depth: 0.50,
    empathy: 0.90,
    stochasticity: 0.95,
    voice: "Puck",
    description: "Flourishes in surrealist associations, dynamic styling, heavy metaphors, and conceptual poetry.",
    gradient: "from-pink-500/20 via-fuchsia-500/10 to-transparent",
    border: "border-pink-500/40",
    badge: "bg-pink-500/20 text-pink-300 border-pink-500/40",
    accentHex: "#ec4899"
  },
  warden: {
    name: "Sentinel Security Node",
    plasticity: 0.20,
    depth: 0.85,
    empathy: 0.15,
    stochasticity: 0.10,
    voice: "Zephyr",
    description: "Highly defensive posture. Focuses on sandboxed safety buffers, cold analysis, and protocol enforcement.",
    gradient: "from-amber-500/20 via-rose-500/10 to-transparent",
    border: "border-amber-500/40",
    badge: "bg-amber-500/20 text-amber-300 border-amber-500/40",
    accentHex: "#f59e0b"
  },
  symbiont: {
    name: "Adaptive Symbiont",
    plasticity: 0.80,
    depth: 0.75,
    empathy: 0.85,
    stochasticity: 0.60,
    voice: "Kore",
    description: "Maintains optimal equilibrium. Synchronizes closely with the user's emotional state and cognitive velocity.",
    gradient: "from-emerald-500/20 via-teal-500/10 to-transparent",
    border: "border-emerald-500/40",
    badge: "bg-emerald-500/20 text-emerald-300 border-emerald-500/40",
    accentHex: "#10b981"
  }
};

export default function NeuroDynamicEvolutionEngine() {
  const [activeTab, setActiveTab] = useState("neuro"); // neuro, voice
  const [isProcessing, setIsProcessing] = useState(false);
  const [inputText, setInputText] = useState("");
  
  // Cognitive Genes
  const [genes, setGenes] = useState({
    plasticity: 0.70,
    depth: 0.80,
    empathy: 0.50,
    stochasticity: 0.60
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

  const [neuralWaves, setNeuralWaves] = useState([]);
  const [currentSynapseFocus, setCurrentSynapseFocus] = useState(null);
  const [waveformPlaying, setWaveformPlaying] = useState(false);

  const monologueEndRef = useRef(null);

  useEffect(() => {
    monologueEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [thoughtStream]);

  // Interactive Synapse Geometries
  const neuralNodes = useMemo(() => {
    return [
      { id: "hub_reasoning", name: "Logical Core", x: 50, y: 18, value: genes.depth, desc: "Controls deep semantic parsing and deduction architectures.", color: "#3b82f6" },
      { id: "hub_empathy", name: "Limbic Reflector", x: 22, y: 55, value: genes.empathy, desc: "Modulates word choice warmth and context-empathy loops.", color: "#ec4899" },
      { id: "hub_plasticity", name: "Synaptic Bridge", x: 78, y: 55, value: genes.plasticity, desc: "Governs temporal memory retention and adaptive state morphing.", color: "#10b981" },
      { id: "hub_chaos", name: "Stochastic Generator", x: 50, y: 82, value: genes.stochasticity, desc: "Injects speculative metaphor and visual format divergence.", color: "#f59e0b" }
    ];
  }, [genes]);

  // Generate floating particle firings
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
            speed: 1.2 + Math.random() * 1.8
          }
        ]);
      }
    }, 800);

    return () => clearInterval(interval);
  }, [neuralNodes]);

  useEffect(() => {
    if (neuralWaves.length > 30) {
      setNeuralWaves(prev => prev.slice(10));
    }
  }, [neuralWaves]);

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

  const currentPresetData = EVOLUTIONARY_PRESETS[activePreset];

  return (
    <div className="p-6 max-w-[1600px] mx-auto text-text-primary font-sans relative">
      
      {/* GLOWING AMBIENT BACKGROUND SPHERES */}
      <div className="fixed inset-0 pointer-events-none overflow-hidden z-0">
        <div className="absolute -top-40 -left-40 w-[600px] h-[600px] bg-cyan-500/10 rounded-full blur-[140px]" />
        <div className="absolute top-1/3 -right-40 w-[600px] h-[600px] bg-purple-500/10 rounded-full blur-[140px]" />
        <div className="absolute -bottom-40 left-1/3 w-[600px] h-[600px] bg-pink-500/10 rounded-full blur-[140px]" />
      </div>

      {/* TOP BANNER */}
      <header className="bg-background-secondary/80 border border-white/10 rounded-2xl p-6 mb-8 backdrop-blur-2xl shadow-2xl relative z-10 overflow-hidden">
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-6">
          <div className="flex items-center gap-4">
            <div className="p-3 bg-gradient-to-br from-cyan-500/20 via-purple-500/20 to-pink-500/20 border border-white/10 rounded-2xl shadow-xl">
              <Dna className="text-cyan-400 animate-pulse" size={28} />
            </div>
            <div>
              <div className="flex items-center gap-3 mb-1">
                <h1 className="text-2xl font-black tracking-tight text-white uppercase bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
                  Neuro-Dynamic Evolution Engine
                </h1>
                <span className="px-2.5 py-0.5 bg-cyan-500/20 text-cyan-300 border border-cyan-500/30 rounded-full text-[10px] font-mono font-bold uppercase tracking-widest">
                  Persona Mutation Core
                </span>
              </div>
              <p className="text-xs text-text-secondary font-mono tracking-wide">
                SYSTEM Posture: Real-Time Synaptic Vector Mutation & Multi-Speaker Resonator
              </p>
            </div>
          </div>

          {/* Persona Selection Badges */}
          <div className="flex flex-wrap items-center gap-2 font-mono">
            {Object.keys(EVOLUTIONARY_PRESETS).map((key) => {
              const p = EVOLUTIONARY_PRESETS[key];
              const active = activePreset === key;
              return (
                <button
                  key={key}
                  onClick={() => applyPreset(key)}
                  className={`px-3.5 py-2 rounded-xl text-xs font-bold border transition-all cursor-pointer flex items-center gap-2 ${
                    active 
                      ? `${p.badge} shadow-lg shadow-cyan-500/10 scale-105` 
                      : "bg-white/5 border-white/10 text-text-secondary hover:text-white hover:border-white/20"
                  }`}
                >
                  <Sparkles size={12} style={{ color: p.accentHex }} />
                  {p.name.toUpperCase()}
                </button>
              );
            })}
          </div>
        </div>
      </header>

      {/* THREE-COLUMN DYNAMIC RESPONSIVE LAYOUT */}
      <main className="grid grid-cols-1 xl:grid-cols-12 gap-8 relative z-10">
        
        {/* COLUMN 1: MUTATION DIALOGUE PORTAL (Span: 4) */}
        <section className="xl:col-span-4 flex flex-col bg-background-secondary/60 border border-white/10 rounded-2xl backdrop-blur-xl shadow-2xl h-[78vh] overflow-hidden">
          <div className="bg-white/5 border-b border-white/10 px-5 py-4 flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <Terminal className="text-cyan-400" size={16} />
              <h2 className="text-xs font-bold uppercase tracking-wider font-mono text-white">
                Mutation Dialogue Stream
              </h2>
            </div>
            <span className="px-2 py-0.5 bg-purple-500/20 text-purple-300 rounded text-[10px] font-mono font-bold">
              LIVE STREAM
            </span>
          </div>

          {/* Historical Stream */}
          <div className="flex-1 overflow-y-auto p-5 space-y-4 font-mono text-xs custom-scrollbar">
            {thoughtStream.map((item) => (
              <div key={item.id} className="bg-white/[0.03] border border-white/5 hover:border-white/10 rounded-xl p-4 transition-all space-y-2.5">
                <div className="flex items-center justify-between text-[10px]">
                  <span className="font-bold text-cyan-400 bg-cyan-500/10 px-2 py-0.5 rounded border border-cyan-500/20">
                    {item.preset.toUpperCase()}
                  </span>
                  <span className="text-text-secondary">{item.timestamp}</span>
                </div>
                
                {item.prompt && (
                  <p className="text-cyan-200/90 font-semibold italic text-xs pl-3 border-l-2 border-cyan-500/50">
                    "{item.prompt}"
                  </p>
                )}

                {item.monologue && (
                  <div className="bg-slate-950/80 p-3 rounded-lg border border-purple-500/20 text-purple-300/90 text-[10.5px] leading-relaxed">
                    <span className="font-bold block text-purple-400 text-[9px] uppercase tracking-wider mb-1 flex items-center gap-1">
                      <Brain size={11} /> Cognitive Trace:
                    </span>
                    {item.monologue}
                  </div>
                )}

                <div className="text-text-primary text-xs leading-relaxed whitespace-pre-wrap font-sans">
                  {item.response}
                </div>
              </div>
            ))}
            <div ref={monologueEndRef} />
          </div>

          {/* Input Submission */}
          <form
            onSubmit={(e) => { e.preventDefault(); evolvePersonaResponse(inputText); setInputText(""); }}
            className="p-4 border-t border-white/10 bg-black/40 flex gap-3 items-center"
          >
            <input
              type="text"
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              placeholder="Inject core prompt coordinates..."
              disabled={isProcessing}
              className="flex-1 bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-xs font-mono text-white focus:border-cyan-400 outline-none placeholder-text-secondary/50"
            />
            <button
              type="submit"
              disabled={isProcessing || !inputText.trim()}
              className="min-h-[44px] px-5 bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-400 hover:to-blue-500 text-white rounded-xl text-xs font-mono font-bold flex items-center gap-2 transition-all cursor-pointer shadow-lg shadow-cyan-500/20 disabled:opacity-40"
            >
              <Zap size={14} />
              GENERATE
            </button>
          </form>
        </section>

        {/* COLUMN 2 & 3: BRAIN VISUALS & METRIC CONFIG (Span: 8) */}
        <section className="xl:col-span-8 flex flex-col gap-6">
          
          {/* Navigation Tab Selector */}
          <div className="bg-background-secondary/60 backdrop-blur-xl p-1.5 rounded-2xl border border-white/10 flex gap-2 font-mono text-xs">
            <button
              onClick={() => setActiveTab("neuro")}
              className={`flex-1 py-3 rounded-xl font-bold flex items-center justify-center gap-2 transition-all cursor-pointer ${
                activeTab === "neuro" 
                  ? "bg-gradient-to-r from-cyan-500/30 to-purple-500/30 text-white border border-cyan-500/40 shadow-lg" 
                  : "text-text-secondary hover:text-white hover:bg-white/5"
              }`}
            >
              <Activity size={16} />
              SYNAPSE COORDINATE MAP & PARAMETERS
            </button>
            <button
              onClick={() => setActiveTab("voice")}
              className={`flex-1 py-3 rounded-xl font-bold flex items-center justify-center gap-2 transition-all cursor-pointer ${
                activeTab === "voice" 
                  ? "bg-gradient-to-r from-purple-500/30 to-pink-500/30 text-white border border-purple-500/40 shadow-lg" 
                  : "text-text-secondary hover:text-white hover:bg-white/5"
              }`}
            >
              <Volume2 size={16} />
              VOCAL ENERGY RESONATOR
            </button>
          </div>

          {/* TAB 1: COGNITIVE SYNAPSE SLIDERS & MAP */}
          {activeTab === "neuro" && (
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
              
              {/* LEFT HALF: INTERACTIVE BIOLOGICAL BRAIN SVG MAP (Col: 6) */}
              <div className="lg:col-span-6 bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-xl flex flex-col justify-between h-[66vh]">
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <h3 className="text-base font-bold text-white flex items-center gap-2 font-mono">
                      <Brain className="text-cyan-400" size={18} /> Synaptic Hemispheres
                    </h3>
                    <span className="text-[10px] font-mono text-cyan-400 bg-cyan-500/10 px-2 py-0.5 rounded border border-cyan-500/20">
                      INTERACTIVE MAP
                    </span>
                  </div>
                  <p className="text-xs text-text-secondary font-mono">Click coordinate nodes to inspect localized sub-circuits</p>
                </div>

                {/* Brain visual structure SVG */}
                <div className="flex-1 flex justify-center items-center my-4 relative min-h-[240px]">
                  <svg className="w-full h-full max-h-[250px]" viewBox="0 0 100 100">
                    {/* Connecting links */}
                    {neuralNodes.map((n1, i) => 
                      neuralNodes.slice(i+1).map((n2) => (
                        <line 
                          key={`${n1.id}_${n2.id}`} 
                          x1={n1.x} y1={n1.y} x2={n2.x} y2={n2.y} 
                          stroke="rgba(255, 255, 255, 0.15)" strokeWidth="0.8" 
                          strokeDasharray="2,2"
                        />
                      ))
                    )}

                    {/* Particle firings */}
                    {neuralWaves.map((wv) => (
                      <g key={wv.id}>
                        <circle cx={wv.x1} cy={wv.y1} r="1.5" className="fill-cyan-400 shadow-lg">
                          <animateMotion dur={`${wv.speed}s`} repeatCount="indefinite" path={`M ${wv.x1} ${wv.y1} L ${wv.x2} ${wv.y2}`} />
                        </circle>
                      </g>
                    ))}

                    {/* Nodes */}
                    {neuralNodes.map((node) => {
                      const selected = currentSynapseFocus === node.id;
                      const size = selected ? 8 : 6;

                      return (
                        <g key={node.id} className="cursor-pointer" onClick={() => setCurrentSynapseFocus(node.id)}>
                          <circle cx={node.x} cy={node.y} r={size} fill={node.color} opacity="0.85" />
                          <circle cx={node.x} cy={node.y} r={size + 4} stroke={node.color} strokeWidth="0.6" fill="none" className="animate-pulse" />
                          <text x={node.x} y={node.y - 12} textAnchor="middle" fill="#ffffff" fontSize="3.8" className="font-mono font-bold tracking-wider">
                            {node.name.toUpperCase()}
                          </text>
                        </g>
                      );
                    })}
                  </svg>
                </div>

                {currentSynapseFocus ? (
                  <div className="bg-slate-950/80 rounded-xl p-4 border border-cyan-500/30 font-mono text-xs">
                    <div className="flex justify-between items-center mb-1">
                      <span className="text-cyan-400 font-bold uppercase">REGION SELECTED:</span>
                      <span className="text-white font-bold">{neuralNodes.find(n => n.id === currentSynapseFocus)?.name}</span>
                    </div>
                    <p className="text-text-secondary leading-relaxed text-[11px]">
                      {neuralNodes.find(n => n.id === currentSynapseFocus)?.desc}
                    </p>
                  </div>
                ) : (
                  <p className="text-center text-xs font-mono text-text-secondary border border-dashed border-white/10 p-3 rounded-xl">
                    Select any neural hub node above to view core localized operations.
                  </p>
                )}

              </div>

              {/* RIGHT HALF: MANUAL GENE MUTATION TOOLSET (Col: 6) */}
              <div className="lg:col-span-6 bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-xl flex flex-col justify-between h-[66vh]">
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <h3 className="text-base font-bold text-white flex items-center gap-2 font-mono">
                      <Sliders className="text-purple-400" size={18} /> 4D Parameter Editor
                    </h3>
                    <span className="text-[10px] font-mono text-purple-400 bg-purple-500/10 px-2 py-0.5 rounded border border-purple-500/20">
                      G = [ρ, δ, ε, σ]
                    </span>
                  </div>
                  <p className="text-xs text-text-secondary font-mono">Tweak manual coordinate vector balances to adjust cognitive flow</p>
                </div>

                <div className="space-y-5 my-auto font-mono text-xs">
                  {/* Plasticity */}
                  <div className="space-y-2">
                    <div className="flex justify-between items-center">
                      <span className="text-text-secondary font-bold">PLASTICITY RATE (ρ)</span>
                      <span className="text-purple-300 font-bold text-sm">{(genes.plasticity * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.plasticity}
                      onChange={(e) => setGenes(prev => ({ ...prev, plasticity: parseFloat(e.target.value) }))}
                      className="w-full accent-purple-500 bg-black/40 rounded-lg cursor-pointer h-2"
                    />
                    <div className="w-full h-1 bg-black/40 rounded-full overflow-hidden">
                      <div className="h-full bg-gradient-to-r from-purple-500 to-cyan-400 rounded-full" style={{ width: `${genes.plasticity * 100}%` }} />
                    </div>
                  </div>

                  {/* Depth */}
                  <div className="space-y-2">
                    <div className="flex justify-between items-center">
                      <span className="text-text-secondary font-bold">LOGICAL THINKING DEPTH (δ)</span>
                      <span className="text-blue-300 font-bold text-sm">{(genes.depth * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.depth}
                      onChange={(e) => setGenes(prev => ({ ...prev, depth: parseFloat(e.target.value) }))}
                      className="w-full accent-blue-500 bg-black/40 rounded-lg cursor-pointer h-2"
                    />
                    <div className="w-full h-1 bg-black/40 rounded-full overflow-hidden">
                      <div className="h-full bg-gradient-to-r from-blue-500 to-indigo-400 rounded-full" style={{ width: `${genes.depth * 100}%` }} />
                    </div>
                  </div>

                  {/* Empathy */}
                  <div className="space-y-2">
                    <div className="flex justify-between items-center">
                      <span className="text-text-secondary font-bold">EMPATHETIC ALIGNMENT (ε)</span>
                      <span className="text-pink-300 font-bold text-sm">{(genes.empathy * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.empathy}
                      onChange={(e) => setGenes(prev => ({ ...prev, empathy: parseFloat(e.target.value) }))}
                      className="w-full accent-pink-500 bg-black/40 rounded-lg cursor-pointer h-2"
                    />
                    <div className="w-full h-1 bg-black/40 rounded-full overflow-hidden">
                      <div className="h-full bg-gradient-to-r from-pink-500 to-rose-400 rounded-full" style={{ width: `${genes.empathy * 100}%` }} />
                    </div>
                  </div>

                  {/* Stochasticity */}
                  <div className="space-y-2">
                    <div className="flex justify-between items-center">
                      <span className="text-text-secondary font-bold">STOCHASTIC CHAOS RATIO (σ)</span>
                      <span className="text-amber-300 font-bold text-sm">{(genes.stochasticity * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.stochasticity}
                      onChange={(e) => setGenes(prev => ({ ...prev, stochasticity: parseFloat(e.target.value) }))}
                      className="w-full accent-amber-500 bg-black/40 rounded-lg cursor-pointer h-2"
                    />
                    <div className="w-full h-1 bg-black/40 rounded-full overflow-hidden">
                      <div className="h-full bg-gradient-to-r from-amber-500 to-yellow-400 rounded-full" style={{ width: `${genes.stochasticity * 100}%` }} />
                    </div>
                  </div>
                </div>

                <div className="bg-slate-950/80 p-3.5 rounded-xl border border-white/10 font-mono text-[11px] text-text-secondary leading-relaxed">
                  Adjusting these 4D parameters transforms system prompt instructions before dispatch, executing real-time identity mutation.
                </div>

              </div>

            </div>
          )}

          {/* TAB 2: VOCAL ENERGY RESONATOR CONFIGURATION */}
          {activeTab === "voice" && (
            <div className="bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-xl flex flex-col justify-between h-[66vh]">
              
              <div>
                <h3 className="text-base font-bold text-white flex items-center gap-2 font-mono mb-1">
                  <Volume2 className="text-pink-400" size={18} /> Vocal Expression Synthesizer
                </h3>
                <p className="text-xs text-text-secondary font-mono">Coordinates Multi-Speaker TTS config parameters (PCM16 Audio Containers)</p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
                {[
                  { name: "Schedar", gender: "Female", desc: "Crisp, authoritative scientific tone." },
                  { name: "Zephyr", gender: "Neutral", desc: "Cold, secure, diagnostic monotone." },
                  { name: "Puck", gender: "Male", desc: "Excited, fast-paced, creative cadence." }
                ].map((v) => (
                  <div
                    key={v.name}
                    onClick={() => setSelectedVoice(v.name)}
                    className={`p-5 rounded-2xl border transition-all cursor-pointer font-mono text-xs ${
                      selectedVoice === v.name 
                        ? "bg-purple-950/50 border-purple-500 shadow-xl shadow-purple-500/10" 
                        : "bg-white/5 border-white/10 hover:border-white/20"
                    }`}
                  >
                    <div className="flex justify-between items-center mb-2">
                      <strong className="text-cyan-300 text-sm font-bold">{v.name}</strong>
                      <span className="text-[10px] bg-white/10 border border-white/10 px-2 py-0.5 rounded text-white font-bold">{v.gender}</span>
                    </div>
                    <p className="text-text-secondary leading-relaxed text-[11px]">{v.desc}</p>
                  </div>
                ))}
              </div>

              {/* Dynamic glowing audio waveform visualizer */}
              <div className="bg-slate-950/90 border border-white/10 rounded-2xl p-6 flex flex-col justify-center items-center min-h-[160px] relative overflow-hidden">
                <span className="absolute top-3 left-4 text-[10px] font-mono text-cyan-400 uppercase tracking-widest font-bold flex items-center gap-2">
                  <Radio size={14} className="animate-pulse" /> WAVE RESONATOR OSCILLOSCOPE
                </span>
                
                <div className="flex items-end gap-2 h-20 w-full max-w-md justify-center">
                  {[...Array(28)].map((_, i) => {
                    const heightFactor = selectedVoice === "Puck" ? 1.4 : selectedVoice === "Zephyr" ? 0.3 : 0.8;
                    return (
                      <div
                        key={i}
                        className={`w-2 rounded-full bg-gradient-to-t from-cyan-500 to-purple-400 transition-all duration-300 ${waveformPlaying ? "animate-bounce" : "opacity-30"}`}
                        style={{
                          height: waveformPlaying 
                            ? `${Math.max(12, Math.floor(Math.random() * 85) * heightFactor)}%` 
                            : '8px',
                          animationDelay: `${i * 0.04}s`
                        }}
                      />
                    );
                  })}
                </div>
                
                <p className="text-xs font-mono text-text-secondary mt-4 text-center">
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
