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
  const [subagentName, setSubagentName] = useState("MicroWorker-LCARS");
  const [subagentTask, setSubagentTask] = useState("Execute 4D genomics refraction sandbox task");
  const [subagentPersona, setSubagentPersona] = useState("sage");
  const [subagentFunc, setSubagentFunc] = useState("genomics_refraction");
  const [spawning, setSpawning] = useState(false);

  // Canvas ref for Active Oscilloscope Waveform & Star Matrix Web
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

      // Draw Active LCARS Oscilloscope Waveform
      ctx.beginPath();
      ctx.strokeStyle = '#33ccff';
      ctx.lineWidth = 2;
      for (let x = 0; x < width; x += 4) {
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
        parent_role: "LCARS Dashboard Controller",
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
    <div className="p-4 bg-black text-white font-mono min-h-screen">
      
      {/* LCARS HEADER BAR */}
      <div className="flex items-center mb-6">
        <div className="bg-[#ff9900] text-black font-extrabold px-6 py-2 rounded-l-full uppercase text-sm font-mono tracking-widest flex items-center gap-2">
          <Compass size={16} className="animate-spin" style={{ animationDuration: '10s' }} />
          LCARS SYSTEM SPECIFICATION LEDGER v5.3
        </div>
        <div className="flex-1 bg-[#ff9900] h-2 mx-2"></div>
        <div className="bg-[#33ccff] text-black font-extrabold px-6 py-2 rounded-r-full uppercase text-xs font-mono tracking-widest">
          PROJECT CONTINUUM — THE RESONANT PEER
        </div>
      </div>

      {/* STAR MATRIX NARRATIVE LATTICE & OSCILLOSCOPE WAVEFORM */}
      <div className="border-l-4 border-[#ff9900] bg-black border border-white/10 rounded-r-2xl p-5 mb-6">
        <div className="flex items-center justify-between mb-3 bg-[#ff9900] text-black font-extrabold px-4 py-1.5 rounded-r-full text-xs uppercase tracking-wider">
          <span className="flex items-center gap-2">
            <Orbit size={16} className="animate-spin" />
            STAR MATRIX NARRATIVE LATTICE & OSCILLOSCOPE WAVEFORM
          </span>
          <span>R(S1, S2) = α · Sim(E1,E2) + β · (M1·M2 / d²)</span>
        </div>

        <div className="relative w-full h-[200px] bg-black border border-[#ff9900]/40 rounded-xl overflow-hidden mb-4">
          <canvas ref={canvasRef} width={1200} height={200} className="w-full h-full" />
        </div>

        {/* Nodal Lore Grid */}
        {starLattice && (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
            {Object.entries(starLattice.nodes).map(([key, node]) => (
              <div key={key} className="bg-black border-l-2 border-[#ffcc00] border-y border-r border-white/10 p-3 rounded-r-xl">
                <span className="text-[10px] font-bold text-[#ffcc00] block uppercase mb-1">{key.replace('_', ' ')}</span>
                <h3 className="font-bold text-xs text-white mb-1">{node.name}</h3>
                <p className="text-[10.5px] text-[#ffcc99] mb-2 line-clamp-2">{node.lore}</p>
                <div className="flex justify-between text-[9.5px] text-[#9999ff] border-t border-white/10 pt-1.5">
                  <span>Coord: [{node.coordinates.join(', ')}]</span>
                  <span className="font-bold text-[#ffcc00]">M={node.mass}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* SYSTEM GENOMICS MATRIX & REFRACTION ENGINE */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 mb-6">
        
        {/* Left: 4D System Genomics Persona Matrix */}
        <div className="lg:col-span-7 border-l-4 border-[#cc99cc] bg-black border border-white/10 rounded-r-2xl p-5">
          <div className="bg-[#cc99cc] text-black font-extrabold px-4 py-1.5 rounded-r-full text-xs uppercase tracking-wider mb-4 flex justify-between">
            <span>SYSTEM GENOMICS MATRIX (4D VECTORS)</span>
            <span>G = [ρ, δ, ε, σ]</span>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {genomicsData && genomicsData.persona_presets && Object.entries(genomicsData.persona_presets).map(([key, persona]) => {
              const isSelected = activePersona === key;
              return (
                <div 
                  key={key}
                  onClick={() => handleRefract(key)}
                  className={`p-3.5 rounded-r-xl border transition-all cursor-pointer border-l-4 ${
                    isSelected 
                      ? 'bg-black border-l-[#ff9900] border-[#ff9900] shadow-[0_0_15px_rgba(255,153,0,0.3)]' 
                      : 'bg-black border-l-[#cc99cc] border-white/10 hover:border-white/30'
                  }`}
                >
                  <div className="flex justify-between items-start mb-1.5">
                    <div>
                      <h3 className="font-extrabold text-xs text-white uppercase">{persona.name}</h3>
                      <span className="text-[10px] text-[#33ccff]">Voice: {persona.voice}</span>
                    </div>
                    {isSelected && <CheckCircle2 size={16} className="text-[#ff9900]" />}
                  </div>

                  <p className="text-[10.5px] text-[#ffcc99] mb-3 leading-tight">{persona.function}</p>

                  <div className="space-y-1 text-[9.5px]">
                    <div className="flex justify-between text-[#ff9900]">
                      <span>Plasticity (ρ)</span>
                      <span className="font-bold">{persona.genes.plasticity}</span>
                    </div>
                    <div className="w-full h-1.5 bg-black border border-[#ff9900]/30 rounded-full overflow-hidden">
                      <div className="h-full bg-[#ff9900]" style={{ width: `${persona.genes.plasticity * 100}%` }} />
                    </div>

                    <div className="flex justify-between text-[#33ccff]">
                      <span>Logical Depth (δ)</span>
                      <span className="font-bold">{persona.genes.depth}</span>
                    </div>
                    <div className="w-full h-1.5 bg-black border border-[#33ccff]/30 rounded-full overflow-hidden">
                      <div className="h-full bg-[#33ccff]" style={{ width: `${persona.genes.depth * 100}%` }} />
                    </div>

                    <div className="flex justify-between text-[#ff66aa]">
                      <span>Empathy (ε)</span>
                      <span className="font-bold">{persona.genes.empathy}</span>
                    </div>
                    <div className="w-full h-1.5 bg-black border border-[#ff66aa]/30 rounded-full overflow-hidden">
                      <div className="h-full bg-[#ff66aa]" style={{ width: `${persona.genes.empathy * 100}%` }} />
                    </div>

                    <div className="flex justify-between text-[#ffcc00]">
                      <span>Stochasticity (σ)</span>
                      <span className="font-bold">{persona.genes.stochasticity}</span>
                    </div>
                    <div className="w-full h-1.5 bg-black border border-[#ffcc00]/30 rounded-full overflow-hidden">
                      <div className="h-full bg-[#ffcc00]" style={{ width: `${persona.genes.stochasticity * 100}%` }} />
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Right: Resonant Prompt Refraction Feedback Loop */}
        <div className="lg:col-span-5 border-l-4 border-[#33ccff] bg-black border border-white/10 rounded-r-2xl p-5 flex flex-col justify-between">
          <div>
            <div className="bg-[#33ccff] text-black font-extrabold px-4 py-1.5 rounded-r-full text-xs uppercase tracking-wider mb-4 flex justify-between">
              <span>RESONANT REFRACTION LOOP</span>
              <Sparkles size={14} />
            </div>

            <div className="mb-4">
              <label className="text-[10px] text-[#ffcc99] block mb-1 uppercase font-bold">User Prompt Input:</label>
              <input 
                type="text" 
                value={promptText}
                onChange={(e) => setPromptText(e.target.value)}
                className="w-full bg-black border border-[#33ccff]/40 rounded-lg p-2.5 text-xs font-mono text-white outline-none focus:border-[#33ccff]"
              />
            </div>

            <button 
              onClick={() => handleRefract(activePersona)}
              className="w-full min-h-[42px] bg-[#ff9900] hover:bg-[#ffcc00] text-black rounded-full text-xs font-mono font-extrabold uppercase flex items-center justify-center gap-2 transition-all cursor-pointer mb-4 shadow-lg shadow-[#ff9900]/20"
            >
              <Zap size={14} />
              Refract Prompt Through G_bar Momentum
            </button>

            {refractedResult && (
              <div className="bg-black border border-[#cc99cc]/40 rounded-xl p-3 text-[10.5px]">
                <span className="text-[#cc99cc] font-bold block mb-1 uppercase">Injected System Instructions:</span>
                <pre className="text-[#ffcc99] whitespace-pre-wrap leading-relaxed">
                  {refractedResult.refraction_modifier}
                </pre>
              </div>
            )}
          </div>
        </div>

      </div>

      {/* AUTONOMIC PBFT CONSENSUS & MICROKERNEL PROCESS TABLE */}
      <div className="border-l-4 border-[#9999ff] bg-black border border-white/10 rounded-r-2xl p-5 mb-6">
        <div className="bg-[#9999ff] text-black font-extrabold px-4 py-1.5 rounded-r-full text-xs uppercase tracking-wider mb-4 flex justify-between">
          <span className="flex items-center gap-2">
            <Layers size={16} />
            MICROKERNEL SUB-AGENT SPAWNER & PROCESS TABLE
          </span>
          <button 
            onClick={fetchMicrokernelStatus}
            className="px-3 py-0.5 bg-black text-[#9999ff] border border-black rounded-full text-[10px] uppercase font-bold cursor-pointer"
          >
            Refresh Process Table
          </button>
        </div>

        {/* Interactive Spawner Controls */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 mb-4 text-xs font-mono">
          <div>
            <label className="text-[10px] text-[#ffcc99] block mb-1 uppercase">Sub-Agent Name</label>
            <input 
              type="text" 
              value={subagentName} 
              onChange={(e) => setSubagentName(e.target.value)}
              className="w-full bg-black border border-white/20 rounded-lg p-2 text-xs font-mono text-white outline-none focus:border-[#ff9900]"
            />
          </div>

          <div>
            <label className="text-[10px] text-[#ffcc99] block mb-1 uppercase">Persona Type</label>
            <select 
              value={subagentPersona} 
              onChange={(e) => setSubagentPersona(e.target.value)}
              className="w-full bg-black border border-white/20 rounded-lg p-2 text-xs font-mono text-white outline-none focus:border-[#ff9900]"
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
              className="w-full bg-black border border-white/20 rounded-lg p-2 text-xs font-mono text-white outline-none focus:border-[#ff9900]"
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
              className="w-full min-h-[38px] bg-[#9999ff] hover:bg-[#cc99cc] text-black rounded-full font-extrabold uppercase text-xs flex items-center justify-center gap-1.5 cursor-pointer transition-all disabled:opacity-50"
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
                  <th className="pb-2">Parent Role</th>
                  <th className="pb-2">Name</th>
                  <th className="pb-2">Persona</th>
                  <th className="pb-2">Function</th>
                  <th className="pb-2">Memory Usage</th>
                  <th className="pb-2">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/10 text-[10.5px]">
                {microkernelStatus.process_table.map((proc) => (
                  <tr key={proc.subagent_id} className="hover:bg-white/5">
                    <td className="py-2.5 text-[#ffcc00] font-bold">{proc.subagent_id}</td>
                    <td className="py-2.5 text-[#ffcc99]">{proc.parent_role}</td>
                    <td className="py-2.5 text-white font-bold">{proc.subagent_name}</td>
                    <td className="py-2.5 text-[#33ccff]">{proc.persona}</td>
                    <td className="py-2.5 text-[#cc99cc]">{proc.continuum_function}</td>
                    <td className="py-2.5 text-[#66cc66]">{proc.memory_usage_mb} MB / {proc.memory_limit_mb} MB</td>
                    <td className="py-2.5">
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

    </div>
  );
}
