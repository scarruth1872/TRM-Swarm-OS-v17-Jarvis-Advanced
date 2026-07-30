import React, { useState, useEffect, useRef, useMemo } from 'react';
import { 
  Compass, Orbit, Shield, Zap, Sparkles, Activity, RefreshCw, Layers, 
  Sliders, Cpu, Terminal, Bell, Settings, Lock, CheckCircle2, AlertTriangle, 
  Radio, Database, Network, Eye, Gauge, FileText, ChevronRight, RefreshCcw, Brain
} from 'lucide-react';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8021';

const EVOLUTIONARY_PRESETS = {
  sage: {
    name: "Cybernetic Sage",
    voice: "Schedar",
    plasticity: 0.40,
    depth: 0.95,
    empathy: 0.40,
    stochasticity: 0.20
  },
  muse: {
    name: "Chaos Muse",
    voice: "Puck",
    plasticity: 0.90,
    depth: 0.50,
    empathy: 0.90,
    stochasticity: 0.95
  },
  sentinel: {
    name: "Sentinel Warden",
    voice: "Zephyr",
    plasticity: 0.20,
    depth: 0.85,
    empathy: 0.20,
    stochasticity: 0.10
  },
  continuum: {
    name: "Continuum Core",
    voice: "Kore",
    plasticity: 0.75,
    depth: 0.80,
    empathy: 0.70,
    stochasticity: 0.50
  }
};

export default function NeuroDynamicEvolutionEngine() {
  const [activeTab, setActiveTab] = useState('OVERVIEW');
  const [activePreset, setActivePreset] = useState('continuum');

  const [genes, setGenes] = useState({
    plasticity: 0.75,
    depth: 0.80,
    empathy: 0.70,
    stochasticity: 0.50
  });

  const [promptText, setPromptText] = useState("Inject core prompt coordinates...");
  const [isProcessing, setIsProcessing] = useState(false);
  const [refractedResult, setRefractedResult] = useState(null);

  const [microkernelStatus, setMicrokernelStatus] = useState(null);
  const [subagentName, setSubagentName] = useState("MicroWorker-Neuro");
  const [subagentPersona, setSubagentPersona] = useState("sage");
  const [subagentFunc, setSubagentFunc] = useState("genomics_refraction");
  const [spawning, setSpawning] = useState(false);

  const [thoughtStream, setThoughtStream] = useState([
    {
      id: "t_init",
      preset: "Continuum Core",
      prompt: "System initialization sequence.",
      monologue: "Assessing neural integrity. Connecting P2P nodes. Synthesizing sensory feedback. Project Continuum v5.3 Dashboard initialized.",
      response: "System initialized. 4D Cognitive Genes prepared for live mutation.",
      timestamp: new Date().toLocaleTimeString()
    }
  ]);

  const latticeCanvasRef = useRef(null);
  const oscCanvasRef = useRef(null);
  const streamEndRef = useRef(null);

  const currentPresetData = useMemo(() => {
    return EVOLUTIONARY_PRESETS[activePreset] || EVOLUTIONARY_PRESETS.continuum;
  }, [activePreset]);

  useEffect(() => {
    streamEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [thoughtStream]);

  useEffect(() => {
    fetchMicrokernelStatus();
  }, []);

  // 3D Perspective Orbital Grid Canvas for Star Matrix Lattice
  useEffect(() => {
    const canvas = latticeCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    let animationFrameId;
    let phase = 0;

    const render = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      phase += 0.015;

      const w = canvas.width;
      const h = canvas.height;
      const cx = w / 2;
      const cy = h / 2 + 10;

      // Draw Perspective Grid Lines
      ctx.strokeStyle = 'rgba(6, 182, 212, 0.12)';
      ctx.lineWidth = 1;
      for (let r = 30; r <= 220; r += 35) {
        ctx.beginPath();
        ctx.ellipse(cx, cy, r, r * 0.45, 0, 0, Math.PI * 2);
        ctx.stroke();
      }

      // Radial spokes
      for (let a = 0; a < Math.PI * 2; a += Math.PI / 6) {
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.lineTo(cx + Math.cos(a + phase * 0.1) * 240, cy + Math.sin(a + phase * 0.1) * 110);
        ctx.stroke();
      }

      // Defined Celestial Nodes
      const nodeDefs = [
        { name: "SAGE CORE", angle: 0, r: 0, color: "#38bdf8", isCore: true },
        { name: "ECHO PROTOCOL", angle: phase + 0.5, r: 120, color: "#38bdf8" },
        { name: "VOG ANOMALY", angle: phase + 1.8, r: 160, color: "#e879f9" },
        { name: "NEBULA ALPHA-1", angle: phase + 3.2, r: 190, color: "#e879f9" },
        { name: "POSIDON", angle: phase + 4.1, r: 90, color: "#38bdf8" },
        { name: "MONAD ANCHOR", angle: phase + 5.0, r: 140, color: "#e879f9" },
        { name: "NEXUS CONFLUENCE", angle: phase + 2.5, r: 110, color: "#38bdf8" },
        { name: "CORE PROTOCOL", angle: phase + 0.9, r: 170, color: "#e879f9" },
        { name: "NEBULA ANOMALY", angle: phase + 3.8, r: 130, color: "#38bdf8" }
      ];

      const projectedNodes = nodeDefs.map(n => {
        const x = cx + Math.cos(n.angle) * n.r;
        const y = cy + Math.sin(n.angle) * (n.r * 0.45);
        return { ...n, x, y };
      });

      ctx.lineWidth = 1;
      for (let i = 0; i < projectedNodes.length; i++) {
        for (let j = i + 1; j < projectedNodes.length; j++) {
          const d = Math.hypot(projectedNodes[i].x - projectedNodes[j].x, projectedNodes[i].y - projectedNodes[j].y);
          if (d < 140) {
            ctx.strokeStyle = i % 2 === 0 ? 'rgba(56, 189, 248, 0.25)' : 'rgba(232, 121, 249, 0.25)';
            ctx.beginPath();
            ctx.moveTo(projectedNodes[i].x, projectedNodes[i].y);
            ctx.lineTo(projectedNodes[j].x, projectedNodes[j].y);
            ctx.stroke();
          }
        }
      }

      projectedNodes.forEach(n => {
        ctx.fillStyle = n.color;
        ctx.shadowColor = n.color;
        ctx.shadowBlur = n.isCore ? 15 : 8;

        ctx.beginPath();
        ctx.arc(n.x, n.y, n.isCore ? 7 : 4.5, 0, Math.PI * 2);
        ctx.fill();

        ctx.shadowBlur = 0;
        ctx.fillStyle = '#f8fafc';
        ctx.font = 'bold 9px "Fira Code", monospace';
        ctx.fillText(n.name, n.x - 25, n.y - 8);
      });

      animationFrameId = requestAnimationFrame(render);
    };

    render();
    return () => cancelAnimationFrame(animationFrameId);
  }, []);

  // Dual Oscilloscope Waveform Canvas
  useEffect(() => {
    const canvas = oscCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    let animationFrameId;
    let phase = 0;

    const render = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      phase += 0.06;

      const w = canvas.width;
      const h = canvas.height;

      // Cyan Top Waveform
      ctx.beginPath();
      ctx.strokeStyle = '#38bdf8';
      ctx.lineWidth = 2;
      ctx.shadowColor = '#38bdf8';
      ctx.shadowBlur = 6;
      for (let x = 0; x < w; x += 4) {
        const y = 45 + Math.sin(x * 0.03 + phase) * 16 + Math.cos(x * 0.08 - phase) * 6;
        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();

      // Magenta Middle Waveform
      ctx.beginPath();
      ctx.strokeStyle = '#e879f9';
      ctx.lineWidth = 2;
      ctx.shadowColor = '#e879f9';
      ctx.shadowBlur = 6;
      for (let x = 0; x < w; x += 4) {
        const y = 95 + Math.sin(x * 0.04 - phase * 1.2) * 14 + Math.cos(x * 0.02 + phase) * 8;
        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();

      ctx.shadowBlur = 0;

      // Bottom Spectrum Bars
      const barCount = 32;
      const barW = (w - 20) / barCount;
      for (let i = 0; i < barCount; i++) {
        const bh = Math.abs(Math.sin(phase + i * 0.3) * 35) + 5;
        ctx.fillStyle = i % 2 === 0 ? '#38bdf8' : '#e879f9';
        ctx.fillRect(10 + i * barW, h - 25 - bh, barW - 2, bh);
      }

      animationFrameId = requestAnimationFrame(render);
    };

    render();
    return () => cancelAnimationFrame(animationFrameId);
  }, []);

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
  };

  const evolvePersonaResponse = async (userPrompt) => {
    if (isProcessing || !userPrompt.trim()) return;
    setIsProcessing(true);

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
          monologue: `Refraction engine simulated under ${currentPresetData.name}. Logical Depth: ${genes.depth}.`,
          response: `Simulated response under [${currentPresetData.name}]: Vector G active.`,
          timestamp: new Date().toLocaleTimeString()
        }
      ]);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleSpawnMicroagent = async () => {
    setSpawning(true);
    try {
      await axios.post(`${API_BASE}/swarm/microkernel/spawn-continuum`, {
        parent_role: "Neuro Evolution Engine",
        subagent_name: subagentName,
        task_spec: "Execute 4D genomics refraction sandbox task",
        persona: subagentPersona,
        continuum_function: subagentFunc,
        memory_limit_mb: 64,
        ttl_seconds: 15
      });
      await fetchMicrokernelStatus();
    } catch (e) {
      console.error("Microkernel spawn error:", e);
    } finally {
      setSpawning(false);
    }
  };

  const renderCircularGauge = (value, label, strokeColor) => {
    const radius = 38;
    const circumference = 2 * Math.PI * radius;
    const progress = (value / 100) * circumference;

    return (
      <div className="flex items-center gap-4 bg-[#090d1a]/80 border border-cyan-500/20 rounded-2xl p-4 backdrop-blur-xl">
        <div className="relative w-24 h-24 flex items-center justify-center">
          <svg className="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
            <circle cx="50" cy="50" r={radius} stroke="rgba(255, 255, 255, 0.08)" strokeWidth="8" fill="transparent" />
            <circle 
              cx="50" cy="50" r={radius} 
              stroke={strokeColor} 
              strokeWidth="8" 
              strokeDasharray={circumference}
              strokeDashoffset={circumference - progress}
              strokeLinecap="round"
              fill="transparent" 
            />
          </svg>
          <div className="absolute inset-0 flex flex-col items-center justify-center font-mono">
            <span className="text-sm font-extrabold text-white">{value}%</span>
          </div>
        </div>

        <div className="flex-1 font-mono">
          <span className="text-[10px] text-cyan-400 font-bold uppercase tracking-wider block mb-1">{label}</span>
          <div className="flex items-center justify-between text-xs mb-1">
            <span className="text-slate-400">STATUS</span>
            <span className="text-emerald-400 font-bold">Stable</span>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="min-h-screen bg-[#030612] text-slate-100 font-mono p-6 relative overflow-hidden select-none">
      
      {/* Background Cosmic Atmosphere */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-indigo-950/40 via-[#030612] to-[#030612] pointer-events-none" />
      <div className="absolute bottom-0 left-0 right-0 h-64 bg-[linear-gradient(to_right,rgba(6,182,212,0.05)_1px,transparent_1px),linear-gradient(to_bottom,rgba(6,182,212,0.05)_1px,transparent_1px)] bg-[size:40px_30px] [transform:perspective(500px)_rotateX(60deg)] pointer-events-none" />

      {/* TOP HEADER NAV BAR */}
      <header className="relative z-10 flex flex-col md:flex-row items-center justify-between gap-4 mb-6 bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-4 backdrop-blur-2xl shadow-[0_0_25px_rgba(6,182,212,0.15)]">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-full bg-gradient-to-tr from-cyan-500 via-purple-500 to-pink-500 flex items-center justify-center shadow-[0_0_15px_rgba(168,85,247,0.5)]">
            <Orbit className="text-white animate-spin" size={22} style={{ animationDuration: '12s' }} />
          </div>
          <div>
            <h1 className="text-lg font-black tracking-tight text-white flex items-center gap-2">
              <span className="bg-gradient-to-r from-cyan-400 via-purple-300 to-pink-400 bg-clip-text text-transparent">PROJECT CONTINUUM</span>
            </h1>
            <p className="text-[10px] text-cyan-400 tracking-widest font-bold uppercase">v5.3 SYSTEM DASHBOARD</p>
          </div>
        </div>

        {/* NAVIGATION TABS */}
        <div className="flex items-center gap-2 bg-[#030612] p-1.5 rounded-xl border border-cyan-500/20 text-xs">
          {['OVERVIEW', 'GENOMICS', 'STAR MATRIX', 'ANALYSIS', 'SETTINGS'].map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-4 py-1.5 rounded-lg font-bold tracking-wider transition-all cursor-pointer ${
                activeTab === tab 
                  ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-400/50 shadow-[0_0_12px_rgba(6,182,212,0.3)]' 
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              {tab}
            </button>
          ))}
        </div>

        <div className="flex items-center gap-2">
          <button className="p-2 bg-white/5 hover:bg-white/10 rounded-xl border border-white/10 text-cyan-400 cursor-pointer">
            <Bell size={16} />
          </button>
          <button className="p-2 bg-white/5 hover:bg-white/10 rounded-xl border border-white/10 text-purple-400 cursor-pointer">
            <Settings size={16} />
          </button>
        </div>
      </header>

      {/* MAIN DASHBOARD GRID */}
      <main className="relative z-10 grid grid-cols-1 lg:grid-cols-12 gap-6">
        
        {/* LEFT COLUMN: GAUGES (Span: 3) */}
        <section className="lg:col-span-3 flex flex-col gap-5">
          <div className="bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-4 backdrop-blur-2xl shadow-xl flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Gauge className="text-cyan-400" size={18} />
              <h2 className="text-xs font-bold uppercase text-white tracking-wider">4D SYSTEM GENOMICS GAUGE CONTROLS</h2>
            </div>
          </div>

          {renderCircularGauge(94.2, "CHROMATIC STABILITY", "#06b6d4")}
          {renderCircularGauge(81.0, "QUANTUM FLUX", "#e879f9")}
          {renderCircularGauge(88.7, "PHENOTYPE INTEGRITY", "#06b6d4")}

          <div className="bg-[#090d1a]/80 border border-cyan-500/20 rounded-2xl p-4 backdrop-blur-xl space-y-3">
            <span className="text-[10px] text-cyan-400 font-bold uppercase tracking-wider block">PRESET POSTURE SELECTOR</span>
            <div className="grid grid-cols-2 gap-2 text-xs font-mono">
              {Object.keys(EVOLUTIONARY_PRESETS).map(key => (
                <button 
                  key={key} 
                  onClick={() => applyPreset(key)}
                  className={`p-2 rounded-lg border text-[10px] font-bold uppercase transition-all cursor-pointer ${
                    activePreset === key ? 'bg-cyan-500/20 border-cyan-400 text-cyan-300' : 'bg-black/40 border-white/10 text-slate-400'
                  }`}
                >
                  {EVOLUTIONARY_PRESETS[key].name.split(' ')[0]}
                </button>
              ))}
            </div>
          </div>
        </section>

        {/* CENTER COLUMN: STAR MATRIX & DIALOGUE (Span: 6) */}
        <section className="lg:col-span-6 flex flex-col gap-6">
          
          <div className="bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-5 backdrop-blur-2xl shadow-xl flex flex-col h-[420px] justify-between relative overflow-hidden">
            <div className="flex items-center justify-between border-b border-cyan-500/20 pb-3">
              <div className="flex items-center gap-2">
                <Orbit className="text-cyan-400 animate-spin" size={18} style={{ animationDuration: '16s' }} />
                <h2 className="text-xs font-bold uppercase text-white tracking-wider">STAR MATRIX NARRATIVE NODE LATTICE</h2>
              </div>
            </div>

            <div className="relative w-full h-[320px] bg-black/60 rounded-xl overflow-hidden my-2 border border-cyan-500/20">
              <canvas ref={latticeCanvasRef} width={800} height={320} className="w-full h-full" />
            </div>

            <div className="flex items-center justify-between text-[10px] text-cyan-400 font-bold pt-2 border-t border-cyan-500/20">
              <span>STAR MATRIX — NARRATIVE NODE LATTICE</span>
              <span>R(S1, S2) = α · Sim(E1,E2) + β · (M1·M2 / d²)</span>
            </div>
          </div>

          {/* ACTIVE OPERATIONAL DATA CARD */}
          <div className="bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-5 backdrop-blur-2xl shadow-xl">
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 text-center font-mono">
              <div className="bg-[#030612] border border-cyan-500/20 p-3 rounded-xl">
                <span className="text-[10px] text-slate-400 block mb-1">TOTAL NODES</span>
                <span className="text-lg font-black text-white">14,812</span>
              </div>

              <div className="bg-[#030612] border border-cyan-500/20 p-3 rounded-xl">
                <span className="text-[10px] text-slate-400 block mb-1">SYSTEM UPTIME</span>
                <span className="text-lg font-black text-cyan-300">312:14:05</span>
              </div>

              <div className="bg-[#030612] border border-emerald-500/30 p-3 rounded-xl">
                <span className="text-[10px] text-slate-400 block mb-1">FLUX STATUS</span>
                <span className="text-lg font-black text-emerald-400">NOMINAL</span>
              </div>

              <div className="bg-[#030612] border border-rose-500/30 p-3 rounded-xl flex flex-col justify-center items-center">
                <span className="text-[10px] text-slate-400 block mb-1">ALERTS</span>
                <span className="text-xs text-rose-400 font-bold flex items-center gap-1">
                  <AlertTriangle size={12} /> Redundant
                </span>
              </div>
            </div>
          </div>

          {/* MUTATION DIALOGUE STREAM */}
          <div className="bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-4 backdrop-blur-2xl flex flex-col h-[300px]">
            <div className="flex items-center gap-2 border-b border-cyan-500/20 pb-2 mb-3">
              <Terminal className="text-cyan-400" size={16} />
              <h2 className="text-xs font-bold text-white uppercase tracking-wider">MUTATION DIALOGUE STREAM</h2>
            </div>

            <div className="flex-1 overflow-y-auto space-y-2 text-xs font-mono">
              {thoughtStream.map(t => (
                <div key={t.id} className="p-2 bg-black/40 border border-white/5 rounded-lg">
                  <span className="text-[10px] text-cyan-400 font-bold block">[{t.preset}] {t.timestamp}</span>
                  <p className="text-slate-200 mt-1">{t.response}</p>
                </div>
              ))}
              <div ref={streamEndRef} />
            </div>

            <form onSubmit={(e) => { e.preventDefault(); evolvePersonaResponse(promptText); setPromptText(""); }} className="flex gap-2 mt-3">
              <input 
                type="text" value={promptText} onChange={(e) => setPromptText(e.target.value)}
                className="flex-1 bg-black border border-cyan-500/30 rounded-lg p-2 text-xs text-white outline-none focus:border-cyan-400"
                placeholder="Inject prompt coordinates..."
              />
              <button className="bg-cyan-500/20 hover:bg-cyan-500/40 text-cyan-300 border border-cyan-400/40 rounded-lg px-4 py-2 text-xs font-bold cursor-pointer">
                GENERATE
              </button>
            </form>
          </div>

        </section>

        {/* RIGHT COLUMN: OSCILLOSCOPE WAVEFORMS (Span: 3) */}
        <section className="lg:col-span-3 flex flex-col gap-6">
          <div className="bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-5 backdrop-blur-2xl shadow-xl flex flex-col h-[520px] justify-between">
            <div className="flex items-center justify-between border-b border-cyan-500/20 pb-3">
              <div className="flex items-center gap-2">
                <Radio className="text-cyan-400" size={18} />
                <h2 className="text-xs font-bold uppercase text-white tracking-wider">OSCILLOSCOPE WAVEFORMS</h2>
              </div>
            </div>

            <div className="bg-[#030612] border border-cyan-500/20 p-3 rounded-xl font-mono text-[10px]">
              <div className="flex justify-between text-cyan-400 font-bold mb-1">
                <span>*SPECTRAL ANALYSIS*</span>
                <div className="flex gap-1">
                  <span className="w-2 h-2 rounded-full bg-cyan-400 animate-ping"></span>
                  <span className="w-2 h-2 rounded-full bg-purple-400"></span>
                </div>
              </div>
              <div className="flex justify-between text-slate-300">
                <span>SYNC 89%</span>
                <span>SIGNAL -64dBm</span>
                <span>FREQ 14.5 GHz</span>
              </div>
            </div>

            <div className="relative w-full h-[240px] bg-black/60 rounded-xl overflow-hidden my-2 border border-cyan-500/20">
              <canvas ref={oscCanvasRef} width={400} height={240} className="w-full h-full" />
            </div>

            <div className="bg-[#030612] border border-cyan-500/20 p-3 rounded-xl text-[10px] space-y-1">
              <div className="flex justify-between text-slate-400">
                <span>SCROLLING RUNS</span>
                <span className="text-cyan-400 font-bold">8.7L -0.45</span>
              </div>
              <div className="flex justify-between text-emerald-400 font-bold">
                <span>STATUS NOMINAL</span>
                <div className="flex gap-1.5">
                  <CheckCircle2 size={14} className="text-emerald-400" />
                  <CheckCircle2 size={14} className="text-purple-400" />
                </div>
              </div>
            </div>
          </div>
        </section>

      </main>

    </div>
  );
}
