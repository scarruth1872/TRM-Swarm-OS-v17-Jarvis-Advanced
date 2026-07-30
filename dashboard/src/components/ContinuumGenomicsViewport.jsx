import React, { useState, useEffect, useRef } from 'react';
import { 
  Dna, Sparkles, Shield, Orbit, Activity, RefreshCw, CheckCircle2, 
  Brain, Zap, Layers, Network, Database, Radio, Compass
} from 'lucide-react';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8021';

export default function ContinuumGenomicsViewport() {
  const [activePersona, setActivePersona] = useState('continuum');
  const [genomicsData, setGenomicsData] = useState(null);
  const [starLattice, setStarLattice] = useState(null);
  const [pbftAudit, setPbftAudit] = useState(null);
  const [microkernelStatus, setMicrokernelStatus] = useState(null);
  const [loading, setLoading] = useState(true);
  const [promptText, setPromptText] = useState("Synthesize a self-healing PBFT consensus ledger block");
  const [refractedResult, setRefractedResult] = useState(null);

  // Microkernel Sub-Agent Spawner Input States
  const [subagentName, setSubagentName] = useState("MicroWorker-Interactive");
  const [subagentTask, setSubagentTask] = useState("Execute 4D genomics refraction sandbox task");
  const [subagentPersona, setSubagentPersona] = useState("sage");
  const [subagentFunc, setSubagentFunc] = useState("genomics_refraction");
  const [spawning, setSpawning] = useState(false);

  // Canvas ref for Active Responsive Oscilloscope & Vector Web Canvas
  const canvasRef = useRef(null);

  useEffect(() => {
    fetchGenomicsStatus();
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

      // Draw Glowing Vector Web Lines
      ctx.strokeStyle = 'rgba(168, 85, 247, 0.15)';
      ctx.lineWidth = 1;
      const width = canvas.width;
      const height = canvas.height;

      // Star Matrix Node Coordinates on Canvas
      const nodes = [
        { x: width * 0.5, y: height * 0.2, label: 'α (Alpha Singularity)' },
        { x: width * 0.2, y: height * 0.65, label: 'μ (Monad Anchor)' },
        { x: width * 0.8, y: height * 0.65, label: 'η (Nexus Confluence)' },
        { x: width * 0.5, y: height * 0.52, label: 'γ (Unified Core)' },
      ];

      // Draw connecting glowing vector edges
      for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
          ctx.beginPath();
          ctx.moveTo(nodes[i].x, nodes[i].y);
          ctx.lineTo(nodes[j].x, nodes[j].y);
          ctx.stroke();
        }
      }

      // Draw Node Pulsing Orbs
      nodes.forEach((n, idx) => {
        const pulse = Math.sin(phase + idx) * 3 + 6;
        ctx.fillStyle = idx === 3 ? '#ec4899' : '#a855f7';
        ctx.beginPath();
        ctx.arc(n.x, n.y, pulse, 0, Math.PI * 2);
        ctx.fill();

        ctx.fillStyle = '#ffffff';
        ctx.font = '10px monospace';
        ctx.fillText(n.label, n.x - 30, n.y - 12);
      });

      // Draw Active Oscilloscope Waveform at Bottom
      ctx.beginPath();
      ctx.strokeStyle = '#06b6d4';
      ctx.lineWidth = 1.5;
      for (let x = 0; x < width; x += 5) {
        const y = height - 25 + Math.sin(x * 0.02 + phase) * 12 + Math.cos(x * 0.05 - phase) * 4;
        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();

      animationFrameId = requestAnimationFrame(render);
    };

    render();
    return () => cancelAnimationFrame(animationFrameId);
  }, [starLattice]);

  const fetchGenomicsStatus = async () => {
    try {
      const res = await axios.get(`${API_BASE}/api/continuum/genomics/status`);
      setGenomicsData(res.data);
    } catch (e) {
      console.error("Genomics fetch error:", e);
    } finally {
      setLoading(false);
    }
  };

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

  const handleSpawnMicroagent = async () => {
    setSpawning(true);
    try {
      await axios.post(`${API_BASE}/swarm/microkernel/spawn-continuum`, {
        parent_role: "Continuum Dashboard Controller",
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

  const handleRefract = async (personaKey) => {
    setActivePersona(personaKey);
    try {
      const res = await axios.post(`${API_BASE}/api/continuum/genomics/refract`, {
        prompt: promptText,
        persona: personaKey
      });
      setRefractedResult(res.data.refraction);
      fetchGenomicsStatus();
    } catch (e) {
      console.error("Refraction error:", e);
    }
  };

  return (
    <div className="p-6 max-w-[1400px] mx-auto text-text-primary font-sans relative">
      
      {/* Top Glassmorphism Header */}
      <div className="bg-background-secondary/80 border border-white/10 rounded-2xl p-8 mb-8 backdrop-blur-2xl shadow-2xl relative overflow-hidden">
        <div className="absolute -right-20 -top-20 w-96 h-96 bg-purple-600/15 rounded-full blur-3xl pointer-events-none" />
        <div className="absolute -left-20 -bottom-20 w-96 h-96 bg-cyan-600/15 rounded-full blur-3xl pointer-events-none" />

        <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 relative z-10">
          <div>
            <div className="flex items-center gap-3 mb-3">
              <span className="px-3 py-1 bg-purple-500/20 text-purple-300 border border-purple-500/30 rounded-full text-xs font-mono font-bold tracking-widest uppercase inline-flex items-center gap-1.5">
                <Compass size={12} className="animate-spin" style={{ animationDuration: '12s' }} />
                SYSTEM SPECIFICATION LEDGER v5.3
              </span>
              <span className="px-3 py-1 bg-cyan-500/20 text-cyan-300 border border-cyan-500/30 rounded-full text-xs font-mono font-bold tracking-widest uppercase">
                THE RESONANT PEER
              </span>
            </div>
            <h1 className="text-3xl font-black tracking-tight text-white mb-2">
              Project Continuum — Cinematic Glassmorphism Canvas
            </h1>
            <p className="text-text-secondary text-sm max-w-2xl leading-relaxed">
              Ambient, fluid, multi-dimensional intelligence platform engineered across a continuous, 
              co-evolving landscape of thought, memory, System Genomics, and Star Matrix narrative lattice.
            </p>
          </div>

          <div className="flex items-center gap-3">
            <button 
              onClick={fetchPbftAudit}
              className="px-4 py-2.5 bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-400 border border-emerald-500/30 rounded-xl text-xs font-mono font-bold flex items-center gap-2 transition-all cursor-pointer shadow-lg shadow-emerald-500/10"
            >
              <Shield size={14} />
              Run Autonomic PBFT Healing
            </button>
          </div>
        </div>
      </div>

      {/* Star Matrix Narrative Lattice Interactive Vector Canvas */}
      <div className="bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-xl mb-8 relative overflow-hidden">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Orbit className="text-cyan-400 animate-spin" size={20} style={{ animationDuration: '15s' }} />
            <h2 className="text-lg font-bold text-white font-mono uppercase tracking-wider">
              Star Matrix Narrative Lattice & Oscilloscope Waveform
            </h2>
          </div>
          <span className="text-xs font-mono text-cyan-400">
            R(S1, S2) = α · Sim(E1,E2) + β · (M1·M2 / d²)
          </span>
        </div>

        {/* Oscilloscope & Node Canvas */}
        <div className="relative w-full h-[220px] bg-black/50 border border-white/10 rounded-xl overflow-hidden mb-6">
          <canvas ref={canvasRef} width={1200} height={220} className="w-full h-full" />
        </div>

        {/* Nodal Lore Grid */}
        {starLattice && (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            {Object.entries(starLattice.nodes).map(([key, node]) => (
              <div key={key} className="bg-white/5 border border-white/10 rounded-xl p-4 hover:border-purple-500/40 transition-all">
                <span className="text-[10px] font-mono text-cyan-400 block font-bold mb-1 uppercase">
                  {key.replace('_', ' ')}
                </span>
                <h3 className="font-bold text-sm text-white mb-1">{node.name}</h3>
                <p className="text-[11px] text-text-secondary mb-2 line-clamp-2 leading-relaxed">{node.lore}</p>
                <div className="flex justify-between font-mono text-[10px] text-text-secondary border-t border-white/5 pt-2">
                  <span>Coord: [{node.coordinates.join(', ')}]</span>
                  <span className="text-purple-400 font-bold">Mass M={node.mass}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Grid Layout: Personas Matrix & Refraction Engine */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 mb-8">
        
        {/* Left: 4D System Genomics Persona Matrix */}
        <div className="lg:col-span-7 bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-xl">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-2">
              <Dna className="text-purple-400" size={20} />
              <h2 className="text-lg font-bold text-white font-mono uppercase tracking-wider">
                System Genomics Matrix (4D Vectors)
              </h2>
            </div>
            <span className="text-xs font-mono text-purple-400">G = [ρ, δ, ε, σ]</span>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {genomicsData && genomicsData.persona_presets && Object.entries(genomicsData.persona_presets).map(([key, persona]) => {
              const isSelected = activePersona === key;
              return (
                <div 
                  key={key}
                  onClick={() => handleRefract(key)}
                  className={`p-4 rounded-xl border transition-all cursor-pointer relative overflow-hidden ${
                    isSelected 
                      ? 'bg-purple-950/50 border-purple-500/60 shadow-lg shadow-purple-500/15' 
                      : 'bg-white/5 border-white/10 hover:border-white/20'
                  }`}
                >
                  <div className="flex justify-between items-start mb-2">
                    <div>
                      <h3 className="font-bold text-sm text-white">{persona.name}</h3>
                      <span className="text-[10px] font-mono text-purple-400">Voice: {persona.voice}</span>
                    </div>
                    {isSelected && <CheckCircle2 size={16} className="text-purple-400" />}
                  </div>

                  <p className="text-xs text-text-secondary mb-3 leading-relaxed">{persona.function}</p>

                  {/* Vector Bar Sliders */}
                  <div className="space-y-1.5 font-mono text-[10px]">
                    <div className="flex justify-between">
                      <span className="text-text-secondary">Plasticity (ρ)</span>
                      <span className="text-purple-300 font-bold">{persona.genes.plasticity}</span>
                    </div>
                    <div className="w-full h-1 bg-black/40 rounded-full overflow-hidden">
                      <div className="h-full bg-purple-400 rounded-full" style={{ width: `${persona.genes.plasticity * 100}%` }} />
                    </div>

                    <div className="flex justify-between">
                      <span className="text-text-secondary">Logical Depth (δ)</span>
                      <span className="text-blue-300 font-bold">{persona.genes.depth}</span>
                    </div>
                    <div className="w-full h-1 bg-black/40 rounded-full overflow-hidden">
                      <div className="h-full bg-blue-400 rounded-full" style={{ width: `${persona.genes.depth * 100}%` }} />
                    </div>

                    <div className="flex justify-between">
                      <span className="text-text-secondary">Empathy (ε)</span>
                      <span className="text-pink-300 font-bold">{persona.genes.empathy}</span>
                    </div>
                    <div className="w-full h-1 bg-black/40 rounded-full overflow-hidden">
                      <div className="h-full bg-pink-400 rounded-full" style={{ width: `${persona.genes.empathy * 100}%` }} />
                    </div>

                    <div className="flex justify-between">
                      <span className="text-text-secondary">Stochasticity (σ)</span>
                      <span className="text-amber-300 font-bold">{persona.genes.stochasticity}</span>
                    </div>
                    <div className="w-full h-1 bg-black/40 rounded-full overflow-hidden">
                      <div className="h-full bg-amber-400 rounded-full" style={{ width: `${persona.genes.stochasticity * 100}%` }} />
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Right: Resonant Prompt Refraction Feedback Loop */}
        <div className="lg:col-span-5 bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-xl flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-2 mb-4">
              <Sparkles className="text-cyan-400" size={20} />
              <h2 className="text-lg font-bold text-white font-mono uppercase tracking-wider">
                Resonant Refraction Loop
              </h2>
            </div>
            
            <p className="text-xs text-text-secondary mb-4 leading-relaxed">
              Calculates baseline momentum vector G_bar over rolling interaction windows to eliminate cold-start limits.
            </p>

            <div className="mb-4">
              <label className="text-xs font-mono text-text-secondary block mb-1">User Prompt Input:</label>
              <input 
                type="text" 
                value={promptText}
                onChange={(e) => setPromptText(e.target.value)}
                className="w-full bg-black/40 border border-white/10 rounded-lg p-2.5 text-xs font-mono text-white focus:border-cyan-400 outline-none"
              />
            </div>

            <button 
              onClick={() => handleRefract(activePersona)}
              className="w-full min-h-[44px] bg-cyan-500/20 hover:bg-cyan-500/30 text-cyan-300 border border-cyan-500/30 rounded-xl text-xs font-mono font-bold flex items-center justify-center gap-2 transition-all cursor-pointer mb-4"
            >
              <Zap size={14} />
              Refract Prompt Through G_bar Momentum
            </button>

            {refractedResult && (
              <div className="bg-slate-950/80 border border-white/10 rounded-xl p-3.5 font-mono text-[11px]">
                <span className="text-purple-400 font-bold block mb-1">Injected System Instructions:</span>
                <pre className="text-text-secondary whitespace-pre-wrap leading-relaxed text-[10.5px]">
                  {refractedResult.refraction_modifier}
                </pre>
              </div>
            )}
          </div>
        </div>

      </div>

      {/* Autonomic PBFT Consensus Ledger Status */}
      {pbftAudit && (
        <div className="bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-xl mb-8">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <Shield className="text-emerald-400" size={20} />
              <h2 className="text-lg font-bold text-white font-mono uppercase tracking-wider">
                Autonomic PBFT Self-Healing Ledger Status
              </h2>
            </div>
            <span className="px-3 py-1 bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 rounded-full text-xs font-mono font-bold">
              {pbftAudit.mesh_status}
            </span>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-4 gap-4">
            {Object.entries(pbftAudit.node_states).map(([n_id, node]) => (
              <div key={n_id} className="bg-white/5 border border-emerald-500/20 rounded-xl p-3.5 flex items-center justify-between">
                <div>
                  <span className="text-xs font-bold text-white block">{node.name} ({n_id})</span>
                  <span className="text-[10px] font-mono text-text-secondary">Ledger Blocks: {node.ledger_length}</span>
                </div>
                <span className="px-2 py-0.5 bg-emerald-500/20 text-emerald-400 rounded text-[10px] font-mono font-bold">
                  {node.status.toUpperCase()}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Microkernel Sub-Agent Spawner & Process Table Viewport */}
      <div className="bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-xl">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-2">
            <Layers className="text-purple-400" size={20} />
            <h2 className="text-lg font-bold text-white font-mono uppercase tracking-wider">
              Microkernel Sub-Agent Spawner & Process Table
            </h2>
          </div>
          <button 
            onClick={fetchMicrokernelStatus}
            className="px-3 py-1 bg-purple-500/20 text-purple-300 border border-purple-500/30 rounded-lg text-xs font-mono font-bold flex items-center gap-1.5 cursor-pointer hover:bg-purple-500/30 transition-all"
          >
            <RefreshCw size={12} />
            Refresh Process Table
          </button>
        </div>

        {/* Interactive Spawner Controls */}
        <div className="bg-white/5 border border-white/10 rounded-xl p-4 mb-6">
          <h3 className="text-xs font-bold text-white font-mono uppercase mb-3 text-purple-300">
            Interactive Sub-Agent Spawner Controls
          </h3>
          
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 mb-3">
            <div>
              <label className="text-[10px] font-mono text-text-secondary block mb-1">Sub-Agent Name</label>
              <input 
                type="text" 
                value={subagentName} 
                onChange={(e) => setSubagentName(e.target.value)}
                className="w-full bg-black/40 border border-white/10 rounded-lg p-2 text-xs font-mono text-white outline-none focus:border-purple-400"
              />
            </div>

            <div>
              <label className="text-[10px] font-mono text-text-secondary block mb-1">Persona Type</label>
              <select 
                value={subagentPersona} 
                onChange={(e) => setSubagentPersona(e.target.value)}
                className="w-full bg-black/40 border border-white/10 rounded-lg p-2 text-xs font-mono text-white outline-none focus:border-purple-400"
              >
                <option value="sage">Cybernetic Sage (sage)</option>
                <option value="muse">Chaos Muse (muse)</option>
                <option value="sentinel">Sentinel Warden (sentinel)</option>
                <option value="continuum">Continuum Core (continuum)</option>
              </select>
            </div>

            <div>
              <label className="text-[10px] font-mono text-text-secondary block mb-1">Continuum Function</label>
              <select 
                value={subagentFunc} 
                onChange={(e) => setSubagentFunc(e.target.value)}
                className="w-full bg-black/40 border border-white/10 rounded-lg p-2 text-xs font-mono text-white outline-none focus:border-purple-400"
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
                className="w-full min-h-[38px] bg-purple-600/30 hover:bg-purple-600/50 text-purple-200 border border-purple-500/40 rounded-lg text-xs font-mono font-bold flex items-center justify-center gap-2 cursor-pointer transition-all disabled:opacity-50"
              >
                <Zap size={14} />
                {spawning ? "Spawning..." : "Spawn Sub-Agent"}
              </button>
            </div>
          </div>
        </div>

        {/* Live Process Table */}
        {microkernelStatus && microkernelStatus.process_table && (
          <div className="overflow-x-auto">
            <table className="w-full text-left font-mono text-xs">
              <thead>
                <tr className="border-b border-white/10 text-text-secondary text-[11px]">
                  <th className="pb-2">Sub-Agent ID</th>
                  <th className="pb-2">Parent Role</th>
                  <th className="pb-2">Name</th>
                  <th className="pb-2">Persona</th>
                  <th className="pb-2">Function</th>
                  <th className="pb-2">Memory Usage</th>
                  <th className="pb-2">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5 text-[11px]">
                {microkernelStatus.process_table.map((proc) => (
                  <tr key={proc.subagent_id} className="hover:bg-white/5 transition-all">
                    <td className="py-2.5 text-purple-300 font-bold">{proc.subagent_id}</td>
                    <td className="py-2.5 text-text-secondary">{proc.parent_role}</td>
                    <td className="py-2.5 text-white font-bold">{proc.subagent_name}</td>
                    <td className="py-2.5 text-cyan-400">{proc.persona}</td>
                    <td className="py-2.5 text-amber-300">{proc.continuum_function}</td>
                    <td className="py-2.5 text-emerald-400">{proc.memory_usage_mb} MB / {proc.memory_limit_mb} MB</td>
                    <td className="py-2.5">
                      <span className="px-2 py-0.5 bg-emerald-500/20 text-emerald-400 rounded text-[10px] font-bold">
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

    </div>
  );
}
