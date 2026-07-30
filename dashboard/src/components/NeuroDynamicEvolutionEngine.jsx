import React, { useState, useEffect, useRef, useMemo } from 'react';
import { 
  Dna, Sparkles, Shield, Orbit, Activity, RefreshCw, CheckCircle2, 
  Brain, Zap, Layers, Network, Database, Radio, Compass, Terminal,
  Volume2, Mic, Settings, Sliders, Cpu, Play, Pause, AlertCircle
} from 'lucide-react';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8021';

// Preset Evolutionary States Matrix
const EVOLUTIONARY_PRESETS = {
  sage: {
    key: "sage",
    name: "Cybernetic Sage",
    plasticity: 0.40,
    depth: 0.95,
    empathy: 0.40,
    stochasticity: 0.20,
    voice: "Schedar",
    function: "Prioritizes high-dimensional logic, structured system design, and precise academic prose.",
    color: "from-cyan-400 to-blue-500",
    accent: "#06b6d4"
  },
  muse: {
    key: "muse",
    name: "Chaos Muse",
    plasticity: 0.90,
    depth: 0.50,
    empathy: 0.90,
    stochasticity: 0.95,
    voice: "Puck",
    function: "Flourishes in surrealist associations, dynamic styling, heavy metaphors, and conceptual poetry.",
    color: "from-fuchsia-500 to-pink-500",
    accent: "#ec4899"
  },
  sentinel: {
    key: "sentinel",
    name: "Sentinel Warden",
    plasticity: 0.20,
    depth: 0.85,
    empathy: 0.20,
    stochasticity: 0.10,
    voice: "Zephyr",
    function: "Highly defensive posture. Focuses on sandboxed safety buffers, cold analysis, and protocol enforcement.",
    color: "from-rose-500 to-orange-500",
    accent: "#f43f5e"
  },
  continuum: {
    key: "continuum",
    name: "Continuum Core",
    plasticity: 0.75,
    depth: 0.80,
    empathy: 0.70,
    stochasticity: 0.50,
    voice: "Kore",
    function: "Maintains optimal equilibrium. Synchronizes closely with the user's emotional state and cognitive velocity.",
    color: "from-emerald-400 to-indigo-500",
    accent: "#10b981"
  }
};

export default function NeuroDynamicEvolutionEngine() {
  // Core System States
  const [activeTab, setActiveTab] = useState("neuro"); // neuro, voice, microkernel
  const [activePreset, setActivePreset] = useState("continuum");
  const [isProcessing, setIsProcessing] = useState(false);
  const [inputText, setInputText] = useState("");

  // Cognitive Genes Vector G = [ρ, δ, ε, σ]
  const [genes, setGenes] = useState({
    plasticity: 0.75,
    depth: 0.80,
    empathy: 0.70,
    stochasticity: 0.50
  });

  const [selectedVoice, setSelectedVoice] = useState("Kore");

  // Backend Integration States
  const [starLattice, setStarLattice] = useState(null);
  const [pbftAudit, setPbftAudit] = useState(null);
  const [microkernelStatus, setMicrokernelStatus] = useState(null);
  const [refractedResult, setRefractedResult] = useState(null);

  // Microkernel Sub-Agent Spawner Input States
  const [subagentName, setSubagentName] = useState("MicroWorker-Dynamic");
  const [subagentTask, setSubagentTask] = useState("Execute 4D genomics refraction sandbox task");
  const [subagentPersona, setSubagentPersona] = useState("sage");
  const [subagentFunc, setSubagentFunc] = useState("genomics_refraction");
  const [spawning, setSpawning] = useState(false);

  // Historical Thought & Event Stream
  const [thoughtStream, setThoughtStream] = useState([
    {
      id: "t_init",
      preset: "Continuum Core",
      prompt: "System initialization sequence.",
      monologue: "Assessing neural integrity. Connecting P2P nodes. Synthesizing sensory feedback. The user has initiated the Evolving Persona Engine.",
      response: "System initialized. Cognitive genes are responsive and prepared for 4D vector mutation.",
      timestamp: new Date().toLocaleTimeString()
    }
  ]);

  // Telemetry details & Interactive biological brain SVG map states
  const [neuralWaves, setNeuralWaves] = useState([]);
  const [currentSynapseFocus, setCurrentSynapseFocus] = useState(null);
  const [waveformPlaying, setWaveformPlaying] = useState(false);

  const monologueEndRef = useRef(null);
  const canvasRef = useRef(null);

  // Current Preset Data (Safe Reference)
  const currentPresetData = useMemo(() => {
    return EVOLUTIONARY_PRESETS[activePreset] || EVOLUTIONARY_PRESETS.continuum;
  }, [activePreset]);

  useEffect(() => {
    monologueEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [thoughtStream]);

  // Initial Backend Data Fetch
  useEffect(() => {
    fetchStarLattice();
    fetchPbftAudit();
    fetchMicrokernelStatus();
  }, []);

  // Oscilloscope & Vector Web Canvas Animation Loop
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    let animationFrameId;
    let phase = 0;

    const render = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      phase += 0.05;

      const width = canvas.width;
      const height = canvas.height;

      // Draw Glowing Vector Web Lines
      ctx.strokeStyle = 'rgba(168, 85, 247, 0.15)';
      ctx.lineWidth = 1;

      const nodes = [
        { x: width * 0.5, y: height * 0.2, label: 'α (Alpha Singularity)' },
        { x: width * 0.2, y: height * 0.65, label: 'μ (Monad Anchor)' },
        { x: width * 0.8, y: height * 0.65, label: 'η (Nexus Confluence)' },
        { x: width * 0.5, y: height * 0.52, label: 'γ (Unified Core)' },
      ];

      for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
          ctx.beginPath();
          ctx.moveTo(nodes[i].x, nodes[i].y);
          ctx.lineTo(nodes[j].x, nodes[j].y);
          ctx.stroke();
        }
      }

      nodes.forEach((n, idx) => {
        const pulse = Math.sin(phase + idx) * 3 + 6;
        ctx.fillStyle = idx === 3 ? '#ec4899' : '#06b6d4';
        ctx.beginPath();
        ctx.arc(n.x, n.y, pulse, 0, Math.PI * 2);
        ctx.fill();

        ctx.fillStyle = '#ffffff';
        ctx.font = '10px monospace';
        ctx.fillText(n.label, n.x - 35, n.y - 12);
      });

      // Draw Active Oscilloscope Waveform
      ctx.beginPath();
      ctx.strokeStyle = waveformPlaying ? '#ec4899' : '#06b6d4';
      ctx.lineWidth = 1.8;
      for (let x = 0; x < width; x += 4) {
        const amp = waveformPlaying ? 18 : 8;
        const y = height - 30 + Math.sin(x * 0.02 + phase) * amp + Math.cos(x * 0.05 - phase) * 4;
        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();

      animationFrameId = requestAnimationFrame(render);
    };

    render();
    return () => cancelAnimationFrame(animationFrameId);
  }, [starLattice, waveformPlaying]);

  // Interactive Synapse Geometries
  const neuralNodes = useMemo(() => {
    return [
      { id: "hub_reasoning", name: "Logical Core (δ)", x: 50, y: 15, value: genes.depth, desc: "Controls deep semantic parsing and deduction architectures." },
      { id: "hub_empathy", name: "Limbic Reflector (ε)", x: 20, y: 50, value: genes.empathy, desc: "Modulates word choice warmth and context-empathy loops." },
      { id: "hub_plasticity", name: "Synaptic Bridge (ρ)", x: 80, y: 50, value: genes.plasticity, desc: "Governs temporal memory retention and adaptive state morphing." },
      { id: "hub_chaos", name: "Stochastic Generator (σ)", x: 50, y: 85, value: genes.stochasticity, desc: "Injects speculative metaphor and visual format divergence." }
    ];
  }, [genes]);

  // Neural Particle Wave Generator
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
            y2: target.y
          }
        ]);
      }
    }, 900);

    return () => clearInterval(interval);
  }, [neuralNodes]);

  useEffect(() => {
    if (neuralWaves.length > 25) {
      setNeuralWaves(prev => prev.slice(8));
    }
  }, [neuralWaves]);

  // Backend Calls
  const fetchStarLattice = async () => {
    try {
      const res = await axios.get(`${API_BASE}/api/continuum/star-matrix/lattice`);
      setStarLattice(res.data.lattice);
    } catch (e) {
      console.error("Lattice fetch error:", e);
    }
  };

  const fetchPbftAudit = async () => {
    try {
      const res = await axios.post(`${API_BASE}/api/continuum/pbft/audit-repair`);
      setPbftAudit(res.data.audit_result);
    } catch (e) {
      console.error("PBFT Audit error:", e);
    }
  };

  const fetchMicrokernelStatus = async () => {
    try {
      const res = await axios.get(`${API_BASE}/swarm/microkernel/status`);
      setMicrokernelStatus(res.data);
    } catch (e) {
      console.error("Microkernel status fetch error:", e);
    }
  };

  const applyPreset = (key) => {
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
    addSystemNotification(`Evolved model matrix parameters to [${p.name}] configuration.`);
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

  // Co-Evolution Refraction Execution
  const evolvePersonaResponse = async (userPrompt) => {
    if (isProcessing || !userPrompt.trim()) return;
    setIsProcessing(true);
    setWaveformPlaying(true);

    try {
      const res = await axios.post(`${API_BASE}/api/continuum/genomics/refract`, {
        prompt: userPrompt,
        persona: activePreset
      });

      const refData = res.data.refraction || {};
      setRefractedResult(refData);

      setThoughtStream(prev => [
        ...prev,
        {
          id: `thought_${Date.now()}`,
          preset: currentPresetData.name,
          prompt: userPrompt,
          monologue: `Refracted through 4D Vector G = [ρ:${genes.plasticity}, δ:${genes.depth}, ε:${genes.empathy}, σ:${genes.stochasticity}]. Session momentum G_bar calculated.`,
          response: refData.refraction_modifier || `Response synthesized under ${currentPresetData.name} posture.`,
          timestamp: new Date().toLocaleTimeString()
        }
      ]);
    } catch (err) {
      setThoughtStream(prev => [
        ...prev,
        {
          id: `err_${Date.now()}`,
          preset: currentPresetData.name,
          prompt: userPrompt,
          monologue: `Local refraction engine offline. Simulating response based on active settings (Logical Depth: ${genes.depth}).`,
          response: `The engine simulated response under [${currentPresetData.name}]: Vector G active [Plasticity=${genes.plasticity}, Depth=${genes.depth}].`,
          timestamp: new Date().toLocaleTimeString()
        }
      ]);
    } finally {
      setIsProcessing(false);
      setTimeout(() => setWaveformPlaying(false), 2000);
    }
  };

  const handleSpawnMicroagent = async () => {
    setSpawning(true);
    try {
      await axios.post(`${API_BASE}/swarm/microkernel/spawn-continuum`, {
        parent_role: "Neuro-Dynamic Evolution Controller",
        subagent_name: subagentName,
        task_spec: subagentTask,
        persona: subagentPersona,
        continuum_function: subagentFunc,
        memory_limit_mb: 64,
        ttl_seconds: 15
      });
      await fetchMicrokernelStatus();
      addSystemNotification(`Spawned Microkernel Sub-Agent [${subagentName}] (${subagentPersona}).`);
    } catch (e) {
      console.error("Microkernel spawn error:", e);
    } finally {
      setSpawning(false);
    }
  };

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans antialiased p-6 max-w-[1600px] mx-auto">
      
      {/* HEADER BANNER */}
      <header className="border border-white/10 bg-slate-900/80 backdrop-blur-2xl rounded-2xl p-6 mb-6 flex flex-wrap items-center justify-between gap-4 shadow-2xl">
        <div className="flex items-center gap-3">
          <div className="relative flex">
            <span className="w-3.5 h-3.5 rounded-full bg-cyan-400 animate-ping absolute"></span>
            <span className="w-3.5 h-3.5 rounded-full bg-cyan-500 border border-cyan-300 relative"></span>
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-xl font-extrabold tracking-tight bg-gradient-to-r from-cyan-400 via-purple-400 to-pink-400 bg-clip-text text-transparent uppercase font-mono">
                NEURO-DYNAMIC EVOLUTION ENGINE (V5.3)
              </h1>
              <span className="text-[10px] font-mono border border-cyan-800 text-cyan-400 bg-cyan-950/50 px-2 py-0.5 rounded-full uppercase">
                THE RESONANT PEER
              </span>
            </div>
            <p className="text-xs text-text-secondary font-mono mt-1">
              Active Persona: <strong className="text-white">{currentPresetData.name}</strong> | Voice: <strong className="text-cyan-300">{currentPresetData.voice}</strong>
            </p>
          </div>
        </div>

        {/* Preset Selector Buttons */}
        <div className="flex flex-wrap items-center gap-2 text-xs font-mono">
          {Object.keys(EVOLUTIONARY_PRESETS).map((key) => {
            const p = EVOLUTIONARY_PRESETS[key];
            const active = activePreset === key;
            return (
              <button
                key={key}
                onClick={() => applyPreset(key)}
                className={`px-3 py-1.5 rounded-xl text-[10px] font-bold border transition-all cursor-pointer ${
                  active 
                    ? "bg-purple-950/70 border-purple-500 text-purple-200 shadow-lg shadow-purple-500/20" 
                    : "bg-white/5 border-white/10 text-text-secondary hover:text-white"
                }`}
              >
                {p.name.toUpperCase()}
              </button>
            );
          })}
        </div>
      </header>

      {/* STAR MATRIX VECTOR CANVAS & WAVEFORM */}
      <section className="bg-slate-900/60 border border-white/10 rounded-2xl p-6 backdrop-blur-xl mb-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Orbit className="text-cyan-400 animate-spin" size={20} style={{ animationDuration: '15s' }} />
            <h2 className="text-sm font-bold text-white font-mono uppercase tracking-wider">
              Star Matrix Narrative Lattice & Oscilloscope Waveform
            </h2>
          </div>
          <button 
            onClick={fetchPbftAudit}
            className="px-3 py-1 bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-400 border border-emerald-500/30 rounded-lg text-xs font-mono font-bold flex items-center gap-1.5 cursor-pointer"
          >
            <Shield size={12} />
            Run Autonomic PBFT Healing
          </button>
        </div>

        <div className="relative w-full h-[200px] bg-black/50 border border-white/10 rounded-xl overflow-hidden mb-4">
          <canvas ref={canvasRef} width={1400} height={200} className="w-full h-full" />
        </div>
      </section>

      {/* MAIN 3-COLUMN LAYOUT */}
      <main className="grid grid-cols-1 xl:grid-cols-12 gap-6 mb-6">
        
        {/* LEFT COLUMN: MUTATION DIALOGUE STREAM (Span: 5) */}
        <section className="xl:col-span-5 flex flex-col bg-slate-900/50 rounded-2xl border border-white/10 backdrop-blur-xl h-[700px] overflow-hidden">
          <div className="bg-white/5 border-b border-white/10 px-4 py-3 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Terminal className="text-cyan-400" size={16} />
              <h2 className="text-xs font-bold uppercase tracking-wider font-mono text-white">Mutation Dialogue Stream</h2>
            </div>
            <span className="px-2.5 py-0.5 bg-purple-500/20 text-purple-300 rounded-full text-[10px] font-mono font-bold">
              LIVE STREAM
            </span>
          </div>

          {/* Historical Stream */}
          <div className="flex-1 overflow-y-auto p-4 space-y-4 text-xs font-mono">
            {thoughtStream.map((item) => (
              <div key={item.id} className="space-y-2 border-b border-white/10 pb-3">
                <div className="flex items-center justify-between text-[10px] text-text-secondary">
                  <span className="font-bold text-cyan-400 bg-cyan-950/40 px-2 py-0.5 rounded border border-cyan-950">{item.preset.toUpperCase()}</span>
                  <span>{item.timestamp}</span>
                </div>
                
                {item.prompt && (
                  <p className="text-text-secondary italic text-[11px] pl-2 border-l border-purple-500/50">
                    "{item.prompt}"
                  </p>
                )}

                {item.monologue && (
                  <div className="bg-black/40 p-2.5 rounded-lg border border-white/5 text-purple-300 text-[10.5px] leading-relaxed">
                    <span className="font-bold block text-text-secondary text-[9px] uppercase tracking-wider mb-1">🧠 Pre-processing Trace:</span>
                    {item.monologue}
                  </div>
                )}

                <div className="text-slate-200 text-[11px] leading-relaxed whitespace-pre-wrap font-sans">
                  {item.response}
                </div>
              </div>
            ))}
            <div ref={monologueEndRef} />
          </div>

          {/* Input Form */}
          <form
            onSubmit={(e) => { e.preventDefault(); evolvePersonaResponse(inputText); setInputText(""); }}
            className="p-4 border-t border-white/10 bg-slate-950 flex gap-2 items-center"
          >
            <input
              type="text"
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              placeholder="Inject core prompt coordinates..."
              disabled={isProcessing}
              className="flex-1 bg-black/40 border border-white/10 rounded-lg px-3 py-2.5 text-xs font-mono text-white focus:outline-none focus:border-cyan-400"
            />
            <button
              type="submit"
              disabled={isProcessing || !inputText.trim()}
              className="bg-cyan-600 hover:bg-cyan-500 text-white py-2 px-4 rounded-lg text-xs font-mono font-bold transition-all disabled:opacity-40 cursor-pointer"
            >
              {isProcessing ? "REFRACTING..." : "GENERATE"}
            </button>
          </form>
        </section>

        {/* RIGHT COLUMN: SYNAPSE MAP & GENES CONFIG (Span: 7) */}
        <section className="xl:col-span-7 flex flex-col gap-6">
          
          {/* Navigation Tabs */}
          <div className="bg-white/5 p-1.5 rounded-xl border border-white/10 flex gap-2 text-xs font-mono">
            <button
              onClick={() => setActiveTab("neuro")}
              className={`flex-1 py-2 rounded-lg text-center font-bold transition-all cursor-pointer ${activeTab === "neuro" ? "bg-purple-600 text-white" : "text-text-secondary hover:text-white"}`}
            >
              SYNAPSE COORDINATE MAP
            </button>
            <button
              onClick={() => setActiveTab("microkernel")}
              className={`flex-1 py-2 rounded-lg text-center font-bold transition-all cursor-pointer ${activeTab === "microkernel" ? "bg-purple-600 text-white" : "text-text-secondary hover:text-white"}`}
            >
              MICROKERNEL PROCESS TABLE
            </button>
          </div>

          {/* TAB 1: SYNAPSE MAP & GENES SLIDERS */}
          {activeTab === "neuro" && (
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
              
              {/* Biological Brain SVG Synapse Map (Col: 6) */}
              <div className="lg:col-span-6 bg-slate-900/50 rounded-2xl border border-white/10 p-5 flex flex-col justify-between relative h-[560px]">
                <div>
                  <h3 className="text-sm font-bold text-white font-mono uppercase">Synaptic Hemispheres</h3>
                  <p className="text-[10px] text-text-secondary font-mono">Real-time biological sub-circuit wave trace</p>
                </div>

                {/* SVG Biological Brain Map */}
                <div className="relative w-full h-[380px] bg-black/40 border border-white/10 rounded-xl overflow-hidden flex items-center justify-center">
                  <svg className="w-full h-full" viewBox="0 0 100 100">
                    {/* Connecting Vector Network */}
                    {neuralNodes.map((n1, i) => 
                      neuralNodes.slice(i + 1).map(n2 => (
                        <line
                          key={`${n1.id}-${n2.id}`}
                          x1={n1.x} y1={n1.y} x2={n2.x} y2={n2.y}
                          stroke="rgba(168, 85, 247, 0.25)"
                          strokeWidth="0.8"
                        />
                      ))
                    )}

                    {/* Animated Waves */}
                    {neuralWaves.map(w => (
                      <circle key={w.id} r="1.5" fill="#06b6d4" className="animate-ping">
                        <animateAttribute attributeName="cx" from={w.x1} to={w.x2} dur="1.2s" repeatCount="indefinite" />
                        <animateAttribute attributeName="cy" from={w.y1} to={w.y2} dur="1.2s" repeatCount="indefinite" />
                      </circle>
                    ))}

                    {/* Synapse Nodes */}
                    {neuralNodes.map(n => (
                      <g key={n.id} onClick={() => setCurrentSynapseFocus(n)} className="cursor-pointer">
                        <circle cx={n.x} cy={n.y} r={4 + n.value * 3} fill="#a855f7" opacity="0.8" />
                        <circle cx={n.x} cy={n.y} r="2" fill="#ffffff" />
                        <text x={n.x} y={n.y - 6} textAnchor="middle" fill="#06b6d4" fontSize="3" fontFamily="monospace" fontWeight="bold">
                          {n.name}
                        </text>
                      </g>
                    ))}
                  </svg>
                </div>

                {currentSynapseFocus && (
                  <div className="bg-black/80 border border-purple-500/40 p-2.5 rounded-lg text-xs font-mono">
                    <span className="text-purple-300 font-bold block">{currentSynapseFocus.name}</span>
                    <span className="text-text-secondary text-[10.5px]">{currentSynapseFocus.desc}</span>
                  </div>
                )}
              </div>

              {/* 4D Cognitive Genes Sliders (Col: 6) */}
              <div className="lg:col-span-6 bg-slate-900/50 rounded-2xl border border-white/10 p-5 flex flex-col justify-between">
                <div>
                  <h3 className="text-sm font-bold text-white font-mono uppercase mb-1">4D Cognitive Gene Sliders</h3>
                  <p className="text-[10px] text-text-secondary font-mono mb-4">G = [Plasticity, Depth, Empathy, Stochasticity]</p>
                </div>

                <div className="space-y-4 font-mono text-xs">
                  {/* Plasticity */}
                  <div className="bg-white/5 border border-white/10 rounded-xl p-3">
                    <div className="flex justify-between items-center mb-1">
                      <span className="text-text-secondary">PLASTICITY (ρ)</span>
                      <span className="text-purple-300 font-bold">{(genes.plasticity * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.plasticity}
                      onChange={(e) => setGenes(prev => ({ ...prev, plasticity: parseFloat(e.target.value) }))}
                      className="w-full accent-purple-500 bg-black/40 rounded-lg cursor-pointer h-2"
                    />
                  </div>

                  {/* Depth */}
                  <div className="bg-white/5 border border-white/10 rounded-xl p-3">
                    <div className="flex justify-between items-center mb-1">
                      <span className="text-text-secondary">LOGICAL DEPTH (δ)</span>
                      <span className="text-cyan-300 font-bold">{(genes.depth * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.depth}
                      onChange={(e) => setGenes(prev => ({ ...prev, depth: parseFloat(e.target.value) }))}
                      className="w-full accent-cyan-500 bg-black/40 rounded-lg cursor-pointer h-2"
                    />
                  </div>

                  {/* Empathy */}
                  <div className="bg-white/5 border border-white/10 rounded-xl p-3">
                    <div className="flex justify-between items-center mb-1">
                      <span className="text-text-secondary">EMPATHY (ε)</span>
                      <span className="text-pink-300 font-bold">{(genes.empathy * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.empathy}
                      onChange={(e) => setGenes(prev => ({ ...prev, empathy: parseFloat(e.target.value) }))}
                      className="w-full accent-pink-500 bg-black/40 rounded-lg cursor-pointer h-2"
                    />
                  </div>

                  {/* Stochasticity */}
                  <div className="bg-white/5 border border-white/10 rounded-xl p-3">
                    <div className="flex justify-between items-center mb-1">
                      <span className="text-text-secondary">STOCHASTICITY (σ)</span>
                      <span className="text-amber-300 font-bold">{(genes.stochasticity * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.stochasticity}
                      onChange={(e) => setGenes(prev => ({ ...prev, stochasticity: parseFloat(e.target.value) }))}
                      className="w-full accent-amber-500 bg-black/40 rounded-lg cursor-pointer h-2"
                    />
                  </div>
                </div>

                <div className="bg-black/60 p-3 rounded-xl border border-white/10 font-mono text-[11px] text-text-secondary mt-4">
                  <span>Active Configuration: <strong className="text-white">{currentPresetData.name}</strong></span>
                </div>
              </div>

            </div>
          )}

          {/* TAB 2: MICROKERNEL PROCESS TABLE VIEWPORT */}
          {activeTab === "microkernel" && (
            <div className="bg-slate-900/50 rounded-2xl border border-white/10 p-5 backdrop-blur-xl">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Layers className="text-purple-400" size={18} />
                  <h3 className="text-sm font-bold text-white font-mono uppercase">Microkernel Sub-Agent Spawner</h3>
                </div>
                <button 
                  onClick={fetchMicrokernelStatus}
                  className="px-3 py-1 bg-purple-500/20 text-purple-300 border border-purple-500/30 rounded-lg text-xs font-mono font-bold flex items-center gap-1 cursor-pointer"
                >
                  <RefreshCw size={12} />
                  Refresh
                </button>
              </div>

              {/* Spawner Form */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-4 font-mono text-xs">
                <div>
                  <label className="text-[10px] text-text-secondary block mb-1">Sub-Agent Name</label>
                  <input 
                    type="text" 
                    value={subagentName} 
                    onChange={(e) => setSubagentName(e.target.value)}
                    className="w-full bg-black/40 border border-white/10 rounded-lg p-2 text-white outline-none focus:border-purple-400"
                  />
                </div>

                <div>
                  <label className="text-[10px] text-text-secondary block mb-1">Persona Type</label>
                  <select 
                    value={subagentPersona} 
                    onChange={(e) => setSubagentPersona(e.target.value)}
                    className="w-full bg-black/40 border border-white/10 rounded-lg p-2 text-white outline-none focus:border-purple-400"
                  >
                    <option value="sage">Cybernetic Sage (sage)</option>
                    <option value="muse">Chaos Muse (muse)</option>
                    <option value="sentinel">Sentinel Warden (sentinel)</option>
                    <option value="continuum">Continuum Core (continuum)</option>
                  </select>
                </div>

                <div>
                  <label className="text-[10px] text-text-secondary block mb-1">Continuum Function</label>
                  <select 
                    value={subagentFunc} 
                    onChange={(e) => setSubagentFunc(e.target.value)}
                    className="w-full bg-black/40 border border-white/10 rounded-lg p-2 text-white outline-none focus:border-purple-400"
                  >
                    <option value="genomics_refraction">genomics_refraction</option>
                    <option value="star_matrix_lattice">star_matrix_lattice</option>
                    <option value="pbft_consensus_vote">pbft_consensus_vote</option>
                  </select>
                </div>

                <div className="flex items-end">
                  <button 
                    onClick={handleSpawnMicroagent}
                    disabled={spawning}
                    className="w-full min-h-[38px] bg-purple-600/30 hover:bg-purple-600/50 text-purple-200 border border-purple-500/40 rounded-lg font-bold flex items-center justify-center gap-2 cursor-pointer transition-all disabled:opacity-50"
                  >
                    <Zap size={14} />
                    {spawning ? "Spawning..." : "Spawn Microagent"}
                  </button>
                </div>
              </div>

              {/* Process Table */}
              {microkernelStatus && microkernelStatus.process_table && (
                <div className="overflow-x-auto">
                  <table className="w-full text-left font-mono text-xs">
                    <thead>
                      <tr className="border-b border-white/10 text-text-secondary text-[10.5px]">
                        <th className="pb-2">Sub-Agent ID</th>
                        <th className="pb-2">Name</th>
                        <th className="pb-2">Persona</th>
                        <th className="pb-2">Function</th>
                        <th className="pb-2">Memory</th>
                        <th className="pb-2">Status</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-white/5 text-[10.5px]">
                      {microkernelStatus.process_table.map((proc) => (
                        <tr key={proc.subagent_id} className="hover:bg-white/5">
                          <td className="py-2 text-purple-300 font-bold">{proc.subagent_id}</td>
                          <td className="py-2 text-white">{proc.subagent_name}</td>
                          <td className="py-2 text-cyan-400">{proc.persona}</td>
                          <td className="py-2 text-amber-300">{proc.continuum_function}</td>
                          <td className="py-2 text-emerald-400">{proc.memory_usage_mb} MB</td>
                          <td className="py-2">
                            <span className="px-2 py-0.5 bg-emerald-500/20 text-emerald-400 rounded text-[9.5px] font-bold">
                              {proc.status}
                            </span>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          )}

        </section>

      </main>

    </div>
  );
}
