import React, { useState, useEffect, useRef } from 'react';
import { 
  Compass, Orbit, Shield, Zap, Sparkles, Activity, RefreshCw, Layers, 
  Sliders, Cpu, Terminal, Bell, Settings, Lock, CheckCircle2, AlertTriangle, 
  Radio, Database, Network, Eye, Gauge, FileText, ChevronRight, RefreshCcw
} from 'lucide-react';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8021';

export default function ContinuumGenomicsViewport() {
  const [activeTab, setActiveTab] = useState('OVERVIEW'); // OVERVIEW, GENOMICS, STAR MATRIX, ANALYSIS, SETTINGS
  const [activePersona, setActivePersona] = useState('continuum');

  // Gauge metrics
  const [chromaticStability, setChromaticStability] = useState(94.2);
  const [quantumFlux, setQuantumFlux] = useState(0.81);
  const [phenotypeIntegrity, setPhenotypeIntegrity] = useState(88.7);

  // Cognitive Genes Vector G = [ρ, δ, ε, σ]
  const [genes, setGenes] = useState({
    plasticity: 0.75,
    depth: 0.80,
    empathy: 0.70,
    stochasticity: 0.50
  });

  const [promptText, setPromptText] = useState("Synthesize a self-healing PBFT consensus ledger block");
  const [refractedResult, setRefractedResult] = useState(null);
  const [starLattice, setStarLattice] = useState(null);
  const [pbftAudit, setPbftAudit] = useState(null);
  const [microkernelStatus, setMicrokernelStatus] = useState(null);

  // Microkernel Sub-Agent Spawner Input States
  const [subagentName, setSubagentName] = useState("MicroWorker-Continuum");
  const [subagentTask, setSubagentTask] = useState("Execute 4D genomics refraction sandbox task");
  const [subagentPersona, setSubagentPersona] = useState("sage");
  const [subagentFunc, setSubagentFunc] = useState("genomics_refraction");
  const [spawning, setSpawning] = useState(false);

  // Canvas Refs
  const latticeCanvasRef = useRef(null);
  const oscCanvasRef = useRef(null);

  useEffect(() => {
    fetchStarLattice();
    fetchPbftAudit();
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

      // Draw Vector Connections
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

      // Draw Nodes & Pulsing Orbs
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

      // Bottom Scrolling Spectrum Bars
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
      console.error("Microkernel status error:", e);
    }
  };

  const handleRefract = async (personaKey) => {
    setActivePersona(personaKey);
    try {
      const res = await axios.post(`${API_BASE}/api/continuum/genomics/refract`, {
        prompt: promptText,
        persona: personaKey
      });
      setRefractedResult(res.data.refraction);
    } catch (e) {
      console.error("Refraction error:", e);
    }
  };

  const handleSpawnMicroagent = async () => {
    setSpawning(true);
    try {
      await axios.post(`${API_BASE}/swarm/microkernel/spawn-continuum`, {
        parent_role: "Project Continuum Dashboard",
        subagent_name: subagentName,
        task_spec: subagentTask,
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

  // Helper for Circular SVG Gauges
  const renderCircularGauge = (value, max, label, subValue, subStatus, colorClass, strokeColor) => {
    const radius = 38;
    const circumference = 2 * Math.PI * radius;
    const progress = (value / max) * circumference;

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
            <span className="text-sm font-extrabold text-white">{value}{typeof value === 'number' && max === 100 ? '%' : ''}</span>
          </div>
        </div>

        <div className="flex-1 font-mono">
          <span className="text-[10px] text-cyan-400 font-bold uppercase tracking-wider block mb-1">{label}</span>
          <div className="flex items-center justify-between text-xs mb-1">
            <span className="text-slate-400">LEVEL</span>
            <span className="text-white font-bold">{subValue}</span>
          </div>
          <div className="flex items-center justify-between text-[11px]">
            <span className="text-emerald-400 font-bold">{subStatus}</span>
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
        
        {/* LOGO & TITLE */}
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

        {/* RIGHT CONTROLS */}
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
        
        {/* LEFT COLUMN: 4D SYSTEM GENOMICS GAUGE CONTROLS (Span: 3) */}
        <section className="lg:col-span-3 flex flex-col gap-5">
          <div className="bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-4 backdrop-blur-2xl shadow-xl flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Gauge className="text-cyan-400" size={18} />
              <h2 className="text-xs font-bold uppercase text-white tracking-wider">4D SYSTEM GENOMICS GAUGE CONTROLS</h2>
            </div>
            <XCloseIcon />
          </div>

          {/* GAUGES */}
          {renderCircularGauge(chromaticStability, 100, "CHROMATIC STABILITY", "84.2%", "Stable", "cyan", "#06b6d4")}
          {renderCircularGauge(quantumFlux, 1, "QUANTUM FLUX", "88.7%", "Stable", "purple", "#e879f9")}
          {renderCircularGauge(phenotypeIntegrity, 100, "PHENOTYPE INTEGRITY", "88.7%", "Stable", "cyan", "#06b6d4")}

          {/* SLIDERS FOR LIVE CONTROL */}
          <div className="bg-[#090d1a]/80 border border-cyan-500/20 rounded-2xl p-4 backdrop-blur-xl space-y-3">
            <span className="text-[10px] text-cyan-400 font-bold uppercase tracking-wider block">MUTATION CONTROL SLIDERS</span>
            
            <div>
              <div className="flex justify-between text-[11px] mb-1">
                <span className="text-slate-400">PLASTICITY (ρ)</span>
                <span className="text-cyan-300 font-bold">{(genes.plasticity * 100).toFixed(0)}%</span>
              </div>
              <input 
                type="range" min="0" max="1" step="0.01" value={genes.plasticity} 
                onChange={(e) => setGenes(prev => ({ ...prev, plasticity: parseFloat(e.target.value) }))}
                className="w-full accent-cyan-400 bg-black h-1.5 rounded-lg cursor-pointer"
              />
            </div>

            <div>
              <div className="flex justify-between text-[11px] mb-1">
                <span className="text-slate-400">LOGICAL DEPTH (δ)</span>
                <span className="text-purple-300 font-bold">{(genes.depth * 100).toFixed(0)}%</span>
              </div>
              <input 
                type="range" min="0" max="1" step="0.01" value={genes.depth} 
                onChange={(e) => setGenes(prev => ({ ...prev, depth: parseFloat(e.target.value) }))}
                className="w-full accent-purple-400 bg-black h-1.5 rounded-lg cursor-pointer"
              />
            </div>
          </div>
        </section>

        {/* CENTER COLUMN: STAR MATRIX NARRATIVE LATTICE & ACTIVE DATA (Span: 6) */}
        <section className="lg:col-span-6 flex flex-col gap-6">
          
          {/* STAR MATRIX NARRATIVE NODE LATTICE VIEWPORT */}
          <div className="bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-5 backdrop-blur-2xl shadow-[0_0_20px_rgba(6,182,212,0.1)] flex flex-col h-[460px] justify-between relative overflow-hidden">
            <div className="flex items-center justify-between border-b border-cyan-500/20 pb-3">
              <div className="flex items-center gap-2">
                <Orbit className="text-cyan-400 animate-spin" size={18} style={{ animationDuration: '16s' }} />
                <h2 className="text-xs font-bold uppercase text-white tracking-wider">STAR MATRIX NARRATIVE NODE LATTICE</h2>
              </div>
              <XCloseIcon />
            </div>

            {/* 3D Perspective Orbital Canvas */}
            <div className="relative w-full h-[360px] bg-black/60 rounded-xl overflow-hidden my-2 border border-cyan-500/20">
              <canvas ref={latticeCanvasRef} width={800} height={360} className="w-full h-full" />
            </div>

            <div className="flex items-center justify-between text-[10px] text-cyan-400 font-bold pt-2 border-t border-cyan-500/20">
              <span>STAR MATRIX — NARRATIVE NODE LATTICE</span>
              <span>R(S1, S2) = α · Sim(E1,E2) + β · (M1·M2 / d²)</span>
            </div>
          </div>

          {/* ACTIVE OPERATIONAL DATA CARD */}
          <div className="bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-5 backdrop-blur-2xl shadow-xl">
            <div className="flex items-center justify-between border-b border-cyan-500/20 pb-3 mb-4">
              <div className="flex items-center gap-2">
                <Activity className="text-cyan-400" size={18} />
                <h2 className="text-xs font-bold uppercase text-white tracking-wider">ACTIVE OPERATIONAL DATA</h2>
              </div>
              <XCloseIcon />
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 text-center">
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
                <div className="flex items-center gap-1 text-xs text-rose-400 font-bold">
                  <AlertTriangle size={12} />
                  <span>Redundant alerts</span>
                </div>
              </div>
            </div>
          </div>

          {/* MICROKERNEL PROCESS TABLE VIEWPORT */}
          <div className="bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-5 backdrop-blur-2xl shadow-xl">
            <div className="flex items-center justify-between border-b border-cyan-500/20 pb-3 mb-4">
              <div className="flex items-center gap-2">
                <Layers className="text-purple-400" size={18} />
                <h2 className="text-xs font-bold uppercase text-white tracking-wider">MICROKERNEL SUB-AGENT PROCESS TABLE</h2>
              </div>
              <button onClick={fetchMicrokernelStatus} className="p-1 text-cyan-400 hover:text-white cursor-pointer">
                <RefreshCw size={14} />
              </button>
            </div>

            {/* Microkernel Form */}
            <div className="grid grid-cols-1 sm:grid-cols-4 gap-3 mb-4 text-xs">
              <input 
                type="text" value={subagentName} onChange={(e) => setSubagentName(e.target.value)}
                className="bg-black border border-cyan-500/30 rounded-lg p-2 text-white outline-none focus:border-cyan-400"
                placeholder="Sub-Agent Name"
              />
              <select 
                value={subagentPersona} onChange={(e) => setSubagentPersona(e.target.value)}
                className="bg-black border border-cyan-500/30 rounded-lg p-2 text-white outline-none focus:border-cyan-400"
              >
                <option value="sage">Cybernetic Sage</option>
                <option value="muse">Chaos Muse</option>
                <option value="sentinel">Sentinel Warden</option>
                <option value="continuum">Continuum Core</option>
              </select>
              <select 
                value={subagentFunc} onChange={(e) => setSubagentFunc(e.target.value)}
                className="bg-black border border-cyan-500/30 rounded-lg p-2 text-white outline-none focus:border-cyan-400"
              >
                <option value="genomics_refraction">genomics_refraction</option>
                <option value="star_matrix_lattice">star_matrix_lattice</option>
                <option value="pbft_consensus_vote">pbft_consensus_vote</option>
              </select>
              <button 
                onClick={handleSpawnMicroagent} disabled={spawning}
                className="bg-cyan-500/20 hover:bg-cyan-500/40 text-cyan-300 border border-cyan-400/40 rounded-lg font-bold flex items-center justify-center gap-1.5 cursor-pointer"
              >
                <Zap size={14} />
                {spawning ? "Spawning..." : "Spawn Worker"}
              </button>
            </div>

            {/* Live Process Table */}
            {microkernelStatus && microkernelStatus.process_table && (
              <div className="overflow-x-auto">
                <table className="w-full text-left text-xs font-mono">
                  <thead>
                    <tr className="border-b border-cyan-500/20 text-cyan-400 text-[10px]">
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
                        <td className="py-2 text-white font-bold">{proc.subagent_name}</td>
                        <td className="py-2 text-cyan-400">{proc.persona}</td>
                        <td className="py-2 text-pink-400">{proc.continuum_function}</td>
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

        </section>

        {/* RIGHT COLUMN: OSCILLOSCOPE WAVEFORMS & SPECTRAL ANALYSIS (Span: 3) */}
        <section className="lg:col-span-3 flex flex-col gap-6">
          
          <div className="bg-[#080d1e]/80 border border-cyan-500/30 rounded-2xl p-5 backdrop-blur-2xl shadow-xl flex flex-col h-[520px] justify-between">
            <div className="flex items-center justify-between border-b border-cyan-500/20 pb-3">
              <div className="flex items-center gap-2">
                <Radio className="text-cyan-400" size={18} />
                <h2 className="text-xs font-bold uppercase text-white tracking-wider">OSCILLOSCOPE WAVEFORMS</h2>
              </div>
              <XCloseIcon />
            </div>

            {/* Readout Subheader */}
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

            {/* Dual Waveform Canvas */}
            <div className="relative w-full h-[240px] bg-black/60 rounded-xl overflow-hidden my-2 border border-cyan-500/20">
              <canvas ref={oscCanvasRef} width={400} height={240} className="w-full h-full" />
            </div>

            {/* Scrolling Runs */}
            <div className="bg-[#030612] border border-cyan-500/20 p-3 rounded-xl text-[10px] space-y-1">
              <div className="flex justify-between text-slate-400">
                <span>SCROLLING RUNS</span>
                <span className="text-cyan-400 font-bold">8.7L  -0.45</span>
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

const XCloseIcon = () => (
  <button className="text-slate-500 hover:text-white cursor-pointer transition-colors">
    <span className="font-mono text-xs font-bold">×</span>
  </button>
);
