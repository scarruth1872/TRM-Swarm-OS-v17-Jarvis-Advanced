import React, { useState, useEffect, useRef, useMemo } from 'react';
import { 
  Dna, Sparkles, Shield, Orbit, Activity, RefreshCw, CheckCircle2, 
  Brain, Zap, Layers, Network, Database, Radio, Compass, Terminal,
  Volume2, Mic, Settings, Sliders, Cpu, Play, Pause, AlertCircle
} from 'lucide-react';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8021';

// Preset Evolutionary States Matrix (LCARS Palette)
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
    color: "bg-[#33ccff] text-black",
    accent: "#33ccff"
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
    color: "bg-[#cc99cc] text-black",
    accent: "#cc99cc"
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
    color: "bg-[#cc0000] text-white",
    accent: "#cc0000"
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
    color: "bg-[#ff9900] text-black",
    accent: "#ff9900"
  }
};

export default function NeuroDynamicEvolutionEngine() {
  // Core System States
  const [activeTab, setActiveTab] = useState("neuro"); // neuro, microkernel
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
  const [subagentName, setSubagentName] = useState("MicroWorker-LCARS");
  const [subagentTask, setSubagentTask] = useState("Execute 4D genomics refraction sandbox task");
  const [subagentPersona, setSubagentPersona] = useState("sage");
  const [subagentFunc, setSubagentFunc] = useState("genomics_refraction");
  const [spawning, setSpawning] = useState(false);

  // Historical Thought Stream
  const [thoughtStream, setThoughtStream] = useState([
    {
      id: "t_init",
      preset: "Continuum Core",
      prompt: "LCARS Initialization Sequence.",
      monologue: "Assessing neural integrity. Connecting P2P nodes. Synthesizing sensory feedback. The operator has initiated the LCARS Persona Engine.",
      response: "System initialized. Cognitive genes are responsive and prepared for 4D LCARS vector mutation.",
      timestamp: new Date().toLocaleTimeString()
    }
  ]);

  // Telemetry details & Interactive biological brain SVG map states
  const [neuralWaves, setNeuralWaves] = useState([]);
  const [currentSynapseFocus, setCurrentSynapseFocus] = useState(null);
  const [waveformPlaying, setWaveformPlaying] = useState(false);

  const monologueEndRef = useRef(null);
  const canvasRef = useRef(null);

  // Current Preset Data (Safe Memoized Reference)
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

  // LCARS Oscilloscope Waveform Animation
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

      // Draw Glowing LCARS Vector Web Lines
      ctx.strokeStyle = 'rgba(255, 153, 0, 0.25)';
      ctx.lineWidth = 1.5;

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
        ctx.fillStyle = idx === 3 ? '#ffcc00' : '#ff9900';
        ctx.beginPath();
        ctx.arc(n.x, n.y, pulse, 0, Math.PI * 2);
        ctx.fill();

        ctx.fillStyle = '#ffcc99';
        ctx.font = 'bold 10px "Fira Code", monospace';
        ctx.fillText(n.label, n.x - 40, n.y - 12);
      });

      // Draw Active Oscilloscope Waveform
      ctx.beginPath();
      ctx.strokeStyle = waveformPlaying ? '#ffcc00' : '#33ccff';
      ctx.lineWidth = 2;
      for (let x = 0; x < width; x += 4) {
        const amp = waveformPlaying ? 18 : 8;
        const y = height - 25 + Math.sin(x * 0.02 + phase) * amp + Math.cos(x * 0.05 - phase) * 4;
        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();

      animationFrameId = requestAnimationFrame(render);
    };

    render();
    return () => cancelAnimationFrame(animationFrameId);
  }, [starLattice, waveformPlaying]);

  // Synapse Coordinates
  const neuralNodes = useMemo(() => {
    return [
      { id: "hub_reasoning", name: "Logical Core (δ)", x: 50, y: 15, value: genes.depth, desc: "Controls deep semantic parsing and deduction architectures." },
      { id: "hub_empathy", name: "Limbic Reflector (ε)", x: 20, y: 50, value: genes.empathy, desc: "Modulates word choice warmth and context-empathy loops." },
      { id: "hub_plasticity", name: "Synaptic Bridge (ρ)", x: 80, y: 50, value: genes.plasticity, desc: "Governs temporal memory retention and adaptive state morphing." },
      { id: "hub_chaos", name: "Stochastic Generator (σ)", x: 50, y: 85, value: genes.stochasticity, desc: "Injects speculative metaphor and visual format divergence." }
    ];
  }, [genes]);

  // Particle Waves
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
    addSystemNotification(`Evolved LCARS model parameters to [${p.name}] configuration.`);
  };

  const addSystemNotification = (text) => {
    setThoughtStream(prev => [
      ...prev,
      {
        id: `sys_${Date.now()}`,
        preset: "System Monitor",
        prompt: "LCARS Event Log.",
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
          monologue: `Refracted through 4D Vector G = [ρ:${genes.plasticity}, δ:${genes.depth}, ε:${genes.empathy}, σ:${genes.stochasticity}]. Session momentum G_bar active.`,
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
          monologue: `LCARS Refraction link offline. Simulating response based on active settings (Logical Depth: ${genes.depth}).`,
          response: `The LCARS engine simulated response under [${currentPresetData.name}]: Vector G active [Plasticity=${genes.plasticity}, Depth=${genes.depth}].`,
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
        parent_role: "LCARS Evolution Controller",
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
    <div className="min-h-screen bg-black text-white font-mono antialiased p-4 max-w-[1600px] mx-auto">
      
      {/* LCARS BANNER HEADER */}
      <div className="flex items-center mb-6">
        <div className="bg-[#ff9900] text-black font-extrabold px-6 py-2 rounded-l-full uppercase text-sm font-mono tracking-widest flex items-center gap-2">
          <Brain size={18} />
          LCARS NEURO-DYNAMIC EVOLUTION ENGINE v5.3
        </div>
        <div className="flex-1 bg-[#ff9900] h-2 mx-2"></div>
        <div className="bg-[#33ccff] text-black font-extrabold px-6 py-2 rounded-r-full uppercase text-xs font-mono tracking-widest">
          PERSONA MUTATION MATRIX
        </div>
      </div>

      {/* LCARS PRESET PILL BUTTONS */}
      <div className="flex flex-wrap items-center gap-3 mb-6 bg-black border border-white/10 p-3 rounded-xl border-l-4 border-[#ff9900]">
        <span className="text-xs font-bold text-[#ffcc99] uppercase mr-2 font-mono">SELECT PRESET POSTURE:</span>
        {Object.keys(EVOLUTIONARY_PRESETS).map((key) => {
          const p = EVOLUTIONARY_PRESETS[key];
          const active = activePreset === key;
          return (
            <button
              key={key}
              onClick={() => applyPreset(key)}
              className={`px-4 py-1.5 rounded-full text-xs font-mono font-extrabold uppercase transition-all cursor-pointer ${
                active 
                  ? "bg-[#ff9900] text-black shadow-lg shadow-[#ff9900]/30" 
                  : "bg-white/10 text-[#ffcc99] hover:bg-white/20"
              }`}
            >
              {p.name} ({p.voice})
            </button>
          );
        })}
      </div>

      {/* STAR MATRIX VECTOR CANVAS */}
      <div className="border-l-4 border-[#ff9900] bg-black border border-white/10 rounded-r-2xl p-5 mb-6">
        <div className="bg-[#ff9900] text-black font-extrabold px-4 py-1.5 rounded-r-full text-xs uppercase tracking-wider mb-3 flex justify-between">
          <span className="flex items-center gap-2">
            <Orbit size={16} className="animate-spin" />
            STAR MATRIX NARRATIVE LATTICE & OSCILLOSCOPE WAVEFORM
          </span>
          <button 
            onClick={fetchPbftAudit}
            className="px-3 py-0.5 bg-black text-[#ff9900] border border-black rounded-full text-[10px] uppercase font-bold cursor-pointer"
          >
            Run PBFT Healing
          </button>
        </div>

        <div className="relative w-full h-[180px] bg-black border border-[#ff9900]/40 rounded-xl overflow-hidden mb-4">
          <canvas ref={canvasRef} width={1400} height={180} className="w-full h-full" />
        </div>
      </div>

      {/* MAIN 3-COLUMN LAYOUT */}
      <main className="grid grid-cols-1 xl:grid-cols-12 gap-6 mb-6">
        
        {/* LEFT COLUMN: MUTATION DIALOGUE STREAM (Span: 5) */}
        <section className="xl:col-span-5 border-l-4 border-[#cc99cc] bg-black border border-white/10 rounded-r-2xl flex flex-col h-[680px] overflow-hidden">
          <div className="bg-[#cc99cc] text-black font-extrabold px-4 py-2 rounded-r-full text-xs uppercase tracking-wider flex justify-between items-center">
            <span className="flex items-center gap-2">
              <Terminal size={16} />
              MUTATION DIALOGUE STREAM
            </span>
            <span className="px-2.5 py-0.5 bg-black text-[#cc99cc] rounded-full text-[10px] font-bold">
              LIVE STREAM
            </span>
          </div>

          {/* Historical Stream */}
          <div className="flex-1 overflow-y-auto p-4 space-y-4 text-xs font-mono">
            {thoughtStream.map((item) => (
              <div key={item.id} className="space-y-2 border-b border-white/10 pb-3">
                <div className="flex items-center justify-between text-[10px] text-[#ffcc99]">
                  <span className="font-bold text-[#ff9900] bg-black px-2 py-0.5 rounded border border-[#ff9900]/40">{item.preset.toUpperCase()}</span>
                  <span>{item.timestamp}</span>
                </div>
                
                {item.prompt && (
                  <p className="text-[#ffcc99] italic text-[11px] pl-2 border-l border-[#ff9900]">
                    "{item.prompt}"
                  </p>
                )}

                {item.monologue && (
                  <div className="bg-black p-2.5 rounded-lg border border-[#cc99cc]/30 text-[#cc99cc] text-[10.5px] leading-relaxed">
                    <span className="font-bold block text-[#ffcc99] text-[9px] uppercase mb-1">🧠 Pre-processing Trace:</span>
                    {item.monologue}
                  </div>
                )}

                <div className="text-white text-[11px] leading-relaxed whitespace-pre-wrap font-sans">
                  {item.response}
                </div>
              </div>
            ))}
            <div ref={monologueEndRef} />
          </div>

          {/* Input Form */}
          <form
            onSubmit={(e) => { e.preventDefault(); evolvePersonaResponse(inputText); setInputText(""); }}
            className="p-3 border-t border-white/10 bg-black flex gap-2 items-center"
          >
            <input
              type="text"
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              placeholder="Inject core prompt coordinates..."
              disabled={isProcessing}
              className="flex-1 bg-black border border-[#cc99cc]/40 rounded-full px-4 py-2 text-xs font-mono text-white outline-none focus:border-[#cc99cc]"
            />
            <button
              type="submit"
              disabled={isProcessing || !inputText.trim()}
              className="bg-[#cc99cc] hover:bg-[#ff9900] text-black py-2 px-5 rounded-full text-xs font-mono font-extrabold uppercase transition-all disabled:opacity-40 cursor-pointer"
            >
              {isProcessing ? "MUTATING..." : "GENERATE"}
            </button>
          </form>
        </section>

        {/* RIGHT COLUMN: SYNAPSE MAP & GENES CONFIG (Span: 7) */}
        <section className="xl:col-span-7 flex flex-col gap-6">
          
          {/* Navigation Tabs */}
          <div className="flex gap-2 text-xs font-mono">
            <button
              onClick={() => setActiveTab("neuro")}
              className={`flex-1 py-2 rounded-full text-center font-extrabold uppercase transition-all cursor-pointer ${activeTab === "neuro" ? "bg-[#ff9900] text-black" : "bg-white/10 text-[#ffcc99] hover:bg-white/20"}`}
            >
              SYNAPSE COORDINATE MAP
            </button>
            <button
              onClick={() => setActiveTab("microkernel")}
              className={`flex-1 py-2 rounded-full text-center font-extrabold uppercase transition-all cursor-pointer ${activeTab === "microkernel" ? "bg-[#33ccff] text-black" : "bg-white/10 text-[#ffcc99] hover:bg-white/20"}`}
            >
              MICROKERNEL PROCESS TABLE
            </button>
          </div>

          {/* TAB 1: SYNAPSE MAP & 4D GENES */}
          {activeTab === "neuro" && (
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
              
              {/* Biological Brain SVG Map (Col: 6) */}
              <div className="lg:col-span-6 border-l-4 border-[#33ccff] bg-black border border-white/10 rounded-r-2xl p-4 flex flex-col justify-between h-[540px]">
                <div>
                  <h3 className="text-xs font-extrabold text-[#33ccff] font-mono uppercase mb-1">SYNAPTIC HEMISPHERES</h3>
                  <p className="text-[10px] text-[#ffcc99] font-mono">Biological sub-circuit wave trace</p>
                </div>

                <div className="relative w-full h-[360px] bg-black border border-[#33ccff]/30 rounded-xl overflow-hidden flex items-center justify-center">
                  <svg className="w-full h-full" viewBox="0 0 100 100">
                    {neuralNodes.map((n1, i) => 
                      neuralNodes.slice(i + 1).map(n2 => (
                        <line
                          key={`${n1.id}-${n2.id}`}
                          x1={n1.x} y1={n1.y} x2={n2.x} y2={n2.y}
                          stroke="rgba(51, 204, 255, 0.3)"
                          strokeWidth="0.8"
                        />
                      ))
                    )}

                    {neuralWaves.map(w => (
                      <circle key={w.id} r="1.8" fill="#ff9900" className="animate-ping">
                        <animateAttribute attributeName="cx" from={w.x1} to={w.x2} dur="1.2s" repeatCount="indefinite" />
                        <animateAttribute attributeName="cy" from={w.y1} to={w.y2} dur="1.2s" repeatCount="indefinite" />
                      </circle>
                    ))}

                    {neuralNodes.map(n => (
                      <g key={n.id} onClick={() => setCurrentSynapseFocus(n)} className="cursor-pointer">
                        <circle cx={n.x} cy={n.y} r={4 + n.value * 3} fill="#ff9900" opacity="0.8" />
                        <circle cx={n.x} cy={n.y} r="2" fill="#ffffff" />
                        <text x={n.x} y={n.y - 6} textAnchor="middle" fill="#33ccff" fontSize="3" fontFamily="monospace" fontWeight="bold">
                          {n.name}
                        </text>
                      </g>
                    ))}
                  </svg>
                </div>

                {currentSynapseFocus && (
                  <div className="bg-black border border-[#33ccff] p-2 rounded-lg text-xs font-mono">
                    <span className="text-[#33ccff] font-bold block">{currentSynapseFocus.name}</span>
                    <span className="text-[#ffcc99] text-[10px]">{currentSynapseFocus.desc}</span>
                  </div>
                )}
              </div>

              {/* 4D Cognitive Gene Sliders (Col: 6) */}
              <div className="lg:col-span-6 border-l-4 border-[#ff9900] bg-black border border-white/10 rounded-r-2xl p-4 flex flex-col justify-between">
                <div>
                  <h3 className="text-xs font-extrabold text-[#ff9900] font-mono uppercase mb-1">4D COGNITIVE GENE SLIDERS</h3>
                  <p className="text-[10px] text-[#ffcc99] font-mono mb-3">G = [ρ, δ, ε, σ]</p>
                </div>

                <div className="space-y-3 font-mono text-xs">
                  {/* Plasticity */}
                  <div className="bg-black border border-white/10 p-2.5 rounded-lg">
                    <div className="flex justify-between items-center text-[#ff9900] font-bold mb-1">
                      <span>PLASTICITY (ρ)</span>
                      <span>{(genes.plasticity * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.plasticity}
                      onChange={(e) => setGenes(prev => ({ ...prev, plasticity: parseFloat(e.target.value) }))}
                      className="w-full accent-[#ff9900] bg-black cursor-pointer h-2"
                    />
                  </div>

                  {/* Depth */}
                  <div className="bg-black border border-white/10 p-2.5 rounded-lg">
                    <div className="flex justify-between items-center text-[#33ccff] font-bold mb-1">
                      <span>LOGICAL DEPTH (δ)</span>
                      <span>{(genes.depth * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.depth}
                      onChange={(e) => setGenes(prev => ({ ...prev, depth: parseFloat(e.target.value) }))}
                      className="w-full accent-[#33ccff] bg-black cursor-pointer h-2"
                    />
                  </div>

                  {/* Empathy */}
                  <div className="bg-black border border-white/10 p-2.5 rounded-lg">
                    <div className="flex justify-between items-center text-[#ff66aa] font-bold mb-1">
                      <span>EMPATHY (ε)</span>
                      <span>{(genes.empathy * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.empathy}
                      onChange={(e) => setGenes(prev => ({ ...prev, empathy: parseFloat(e.target.value) }))}
                      className="w-full accent-[#ff66aa] bg-black cursor-pointer h-2"
                    />
                  </div>

                  {/* Stochasticity */}
                  <div className="bg-black border border-white/10 p-2.5 rounded-lg">
                    <div className="flex justify-between items-center text-[#ffcc00] font-bold mb-1">
                      <span>STOCHASTICITY (σ)</span>
                      <span>{(genes.stochasticity * 100).toFixed(0)}%</span>
                    </div>
                    <input
                      type="range" min="0" max="1" step="0.01" value={genes.stochasticity}
                      onChange={(e) => setGenes(prev => ({ ...prev, stochasticity: parseFloat(e.target.value) }))}
                      className="w-full accent-[#ffcc00] bg-black cursor-pointer h-2"
                    />
                  </div>
                </div>

                <div className="bg-[#ff9900] text-black font-extrabold p-2.5 rounded-full font-mono text-[11px] uppercase text-center mt-3">
                  ACTIVE POSTURE: {currentPresetData.name} ({currentPresetData.voice})
                </div>
              </div>

            </div>
          )}

          {/* TAB 2: MICROKERNEL PROCESS TABLE */}
          {activeTab === "microkernel" && (
            <div className="border-l-4 border-[#33ccff] bg-black border border-white/10 rounded-r-2xl p-4">
              <div className="bg-[#33ccff] text-black font-extrabold px-4 py-1.5 rounded-r-full text-xs uppercase tracking-wider mb-4 flex justify-between">
                <span>MICROKERNEL SUB-AGENT SPAWNER</span>
                <button 
                  onClick={fetchMicrokernelStatus}
                  className="px-3 py-0.5 bg-black text-[#33ccff] rounded-full text-[10px] uppercase font-bold cursor-pointer"
                >
                  Refresh
                </button>
              </div>

              {/* Spawner Form */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-4 font-mono text-xs">
                <div>
                  <label className="text-[10px] text-[#ffcc99] block mb-1 uppercase">Sub-Agent Name</label>
                  <input 
                    type="text" 
                    value={subagentName} 
                    onChange={(e) => setSubagentName(e.target.value)}
                    className="w-full bg-black border border-white/20 rounded-lg p-2 text-white outline-none focus:border-[#ff9900]"
                  />
                </div>

                <div>
                  <label className="text-[10px] text-[#ffcc99] block mb-1 uppercase">Persona Type</label>
                  <select 
                    value={subagentPersona} 
                    onChange={(e) => setSubagentPersona(e.target.value)}
                    className="w-full bg-black border border-white/20 rounded-lg p-2 text-white outline-none focus:border-[#ff9900]"
                  >
                    <option value="sage">Cybernetic Sage (sage)</option>
                    <option value="muse">Chaos Muse (muse)</option>
                    <option value="sentinel">Sentinel Warden (sentinel)</option>
                    <option value="continuum">Continuum Core (continuum)</option>
                  </select>
                </div>

                <div>
                  <label className="text-[10px] text-[#ffcc99] block mb-1 uppercase">Continuum Function</label>
                  <select 
                    value={subagentFunc} 
                    onChange={(e) => setSubagentFunc(e.target.value)}
                    className="w-full bg-black border border-white/20 rounded-lg p-2 text-white outline-none focus:border-[#ff9900]"
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
                    className="w-full min-h-[38px] bg-[#ff9900] hover:bg-[#ffcc00] text-black rounded-full font-extrabold uppercase text-xs flex items-center justify-center gap-1.5 cursor-pointer transition-all disabled:opacity-50"
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
                      <tr className="border-b border-white/20 text-[#ffcc99] text-[10.5px] uppercase">
                        <th className="pb-2">Sub-Agent ID</th>
                        <th className="pb-2">Name</th>
                        <th className="pb-2">Persona</th>
                        <th className="pb-2">Function</th>
                        <th className="pb-2">Memory</th>
                        <th className="pb-2">Status</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-white/10 text-[10.5px]">
                      {microkernelStatus.process_table.map((proc) => (
                        <tr key={proc.subagent_id} className="hover:bg-white/5">
                          <td className="py-2 text-[#ffcc00] font-bold">{proc.subagent_id}</td>
                          <td className="py-2 text-white font-bold">{proc.subagent_name}</td>
                          <td className="py-2 text-[#33ccff]">{proc.persona}</td>
                          <td className="py-2 text-[#cc99cc]">{proc.continuum_function}</td>
                          <td className="py-2 text-[#66cc66]">{proc.memory_usage_mb} MB</td>
                          <td className="py-2">
                            <span className="px-2 py-0.5 bg-[#66cc66] text-black font-extrabold rounded-full text-[9.5px] uppercase">
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
