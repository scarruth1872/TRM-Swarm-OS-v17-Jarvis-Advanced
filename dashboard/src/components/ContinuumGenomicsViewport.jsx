import React, { useState, useEffect } from 'react';
import { 
  Dna, Sparkles, Shield, Orbit, Activity, RefreshCw, CheckCircle2, 
  Brain, Zap, Layers, Network, Database
} from 'lucide-react';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8021';

export default function ContinuumGenomicsViewport() {
  const [activePersona, setActivePersona] = useState('continuum');
  const [genomicsData, setGenomicsData] = useState(null);
  const [starLattice, setStarLattice] = useState(null);
  const [pbftAudit, setPbftAudit] = useState(null);
  const [loading, setLoading] = useState(true);
  const [promptText, setPromptText] = useState("Synthesize a self-healing PBFT consensus ledger block");
  const [refractedResult, setRefractedResult] = useState(null);

  useEffect(() => {
    fetchGenomicsStatus();
    fetchStarLattice();
    fetchPbftAudit();
  }, []);

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
    <div className="p-6 max-w-[1400px] mx-auto text-text-primary font-sans">
      
      {/* Top Ledger Header */}
      <div className="bg-background-secondary/80 border border-white/10 rounded-2xl p-8 mb-8 backdrop-blur-xl shadow-2xl relative overflow-hidden">
        <div className="absolute -right-20 -top-20 w-80 h-80 bg-purple-500/10 rounded-full blur-3xl pointer-events-none" />
        
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 relative z-10">
          <div>
            <span className="px-3 py-1 bg-purple-500/20 text-purple-400 border border-purple-500/30 rounded-full text-xs font-mono font-bold tracking-widest uppercase mb-3 inline-block">
              SYSTEM SPECIFICATION LEDGER v5.3
            </span>
            <h1 className="text-3xl font-black tracking-tight text-white mb-2">
              Project Continuum — Unified Emergence
            </h1>
            <p className="text-text-secondary text-sm max-w-2xl leading-relaxed">
              Ambient, fluid, multi-dimensional intelligence platform engineered across a continuous, 
              co-evolving landscape of thought, memory, System Genomics, and Star Matrix narrative lattice.
            </p>
          </div>

          <div className="flex items-center gap-3">
            <button 
              onClick={fetchPbftAudit}
              className="px-4 py-2.5 bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-400 border border-emerald-500/30 rounded-xl text-xs font-mono font-bold flex items-center gap-2 transition-all cursor-pointer"
            >
              <Shield size={14} />
              Run Autonomic PBFT Healing
            </button>
          </div>
        </div>
      </div>

      {/* Grid Layout: Personas Matrix & Refraction Engine */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 mb-8">
        
        {/* Left: 4D System Genomics Persona Matrix */}
        <div className="lg:col-span-7 bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-md">
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
                      ? 'bg-purple-950/40 border-purple-500/50 shadow-lg shadow-purple-500/10' 
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
        <div className="lg:col-span-5 bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-md flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-2 mb-4">
              <Sparkles className="text-accent-cyan" size={20} />
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
                className="w-full bg-black/40 border border-white/10 rounded-lg p-2.5 text-xs font-mono text-white focus:border-accent-cyan outline-none"
              />
            </div>

            <button 
              onClick={() => handleRefract(activePersona)}
              className="w-full min-h-[44px] bg-accent-cyan/20 hover:bg-accent-cyan/30 text-accent-cyan border border-accent-cyan/30 rounded-xl text-xs font-mono font-bold flex items-center justify-center gap-2 transition-all cursor-pointer mb-4"
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

      {/* Star Matrix Narrative Lattice View */}
      {starLattice && (
        <div className="bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-md mb-8">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-2">
              <Orbit className="text-accent-cyan animate-spin" size={20} style={{ animationDuration: '15s' }} />
              <h2 className="text-lg font-bold text-white font-mono uppercase tracking-wider">
                Star Matrix Narrative Lattice (Gravitational Resonance)
              </h2>
            </div>
            <span className="text-xs font-mono text-accent-cyan">
              R(S1,S2) = α·Sim + β·(M1·M2 / d²)
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
            {Object.entries(starLattice.nodes).map(([key, node]) => (
              <div key={key} className="bg-white/5 border border-white/10 rounded-xl p-4">
                <span className="text-[10px] font-mono text-accent-cyan block font-bold mb-1 uppercase">
                  {key.replace('_', ' ')}
                </span>
                <h3 className="font-bold text-sm text-white mb-1">{node.name}</h3>
                <p className="text-[11px] text-text-secondary mb-2 line-clamp-2">{node.lore}</p>
                <div className="flex justify-between font-mono text-[10px] text-text-secondary border-t border-white/5 pt-2">
                  <span>Coord: [{node.coordinates.join(', ')}]</span>
                  <span className="text-purple-400 font-bold">Mass M={node.mass}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Autonomic PBFT Consensus Ledger Status */}
      {pbftAudit && (
        <div className="bg-background-secondary/60 border border-white/10 rounded-2xl p-6 backdrop-blur-md">
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

    </div>
  );
}
