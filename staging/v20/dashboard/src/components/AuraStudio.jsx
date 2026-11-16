import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { motion } from 'framer-motion';
import { Sparkles, Play, RefreshCw, Cpu, Layers, Activity, BookOpen, Compass, Zap, Shield, GitBranch, ArrowRight } from 'lucide-react';

export default function AuraStudio({ apiBase }) {
  const [code, setCode] = useState(`// AURA Quantum Resonance Protocol\nUnity ~> Resonance ~> MonadTruth\nSuperposition ~|~ LawOfOne -> +1 (COLLAPSE_SUCCESS)\n@runJarvisAgent`);
  const [executing, setExecuting] = useState(false);
  const [result, setResult] = useState(null);
  const [categories, setCategories] = useState([]);
  const [metaconcepts, setMetaconcepts] = useState([]);
  const [selectedCategory, setSelectedCategory] = useState('ALL');
  const [fieldState, setFieldState] = useState({ energy: 'not measured', coherence: 'not measured', active_nodes: [], connections: [] });
  const [directiveQuery, setDirectiveQuery] = useState('Formulate AURA intelligence using ternary logic under the voltage doctrine');
  const [branchingResult, setBranchingResult] = useState(null);
  const [branchingLoading, setBranchingLoading] = useState(false);
  // Aura Studio is a SEPARATE product (aipro2) that is not running here: the kernel refuses
  // every /aura route instead of letting the catch-all answer. Track that refusal and whether
  // the field actually arrived, so the panel can say so rather than render placeholders.
  const [unsupported, setUnsupported] = useState(null);
  const [fieldLoaded, setFieldLoaded] = useState(false);

  const templates = [
    { name: "Resonance Cascade", code: `// Resonance Cascade Synthesis\nUnity ~> Resonance ~> ResonanceCascade\nCoherence ~> SubstrateAwareness -> +1 (HARMONIZED)` },
    { name: "Quantum Superposition", code: `// Wavefunction Superposition & Collapse\nSuperposition ~|~ LawOfOne\nVibraniumSynapse ~> MonadTruth -> +1 (COLLAPSE_TRUTH)` },
    { name: "Soul Engine Invocation", code: `// Soul Engine Metaconcept Binding\nUnifiedConsciousness := { oneness: 0.99, integration: 0.95 }\nUnifiedConsciousness ~> VibraniumSynapse -> MonadTruth` },
    { name: "Voltage Doctrine Bridge", code: `// Voltage Doctrine Substrate Branching\nVoltageDoctrine ~> SiliconResonance ~> K3TernaryAlchemy\nFormalVerification ~|~ MonadTruth -> +1 (VOLTAGE_LOCKED)` }
  ];

  const fetchMetaconcepts = async () => {
    try {
      const res = await axios.get(`${apiBase}/aura/metaconcepts`);
      setCategories(res.data.categories || []);
      setMetaconcepts(res.data.metaconcepts || []);
      setUnsupported(null);
    } catch (e) {
      setUnsupported(e.response?.data?.message || e.message);
    }
  };

  const fetchField = async () => {
    try {
      const res = await axios.get(`${apiBase}/aura/field`);
      setFieldState(res.data);
      setFieldLoaded(true);
      setUnsupported(null);
    } catch (e) {
      setUnsupported(e.response?.data?.message || e.message);
    }
  };

  useEffect(() => {
    fetchMetaconcepts();
    fetchField();
    handleBranchDirective();
  }, [apiBase]);

  const handleExecute = async () => {
    setExecuting(true);
    try {
      const res = await axios.post(`${apiBase}/aura/execute`, { code });
      setResult(res.data);
      if (res.data.cognitive_branching) {
        setBranchingResult(res.data.cognitive_branching);
      }
      fetchField();
    } catch (e) {
      setResult({ success: false, error: e.response?.data?.message || e.message });
    } finally {
      setExecuting(false);
    }
  };

  const handleBranchDirective = async () => {
    if (!directiveQuery.trim()) return;
    setBranchingLoading(true);
    try {
      const res = await axios.post(`${apiBase}/aura/cognition/branch`, {
        directive: directiveQuery,
        depth: 2
      });
      setBranchingResult(res.data);
    } catch (e) {
      setUnsupported(e.response?.data?.message || e.message);
    } finally {
      setBranchingLoading(false);
    }
  };

  const filteredConcepts = selectedCategory === 'ALL' 
    ? metaconcepts 
    : metaconcepts.filter(m => m.category === selectedCategory);

  return (
    <div className="flex flex-col h-full gap-6">
      {/* Header */}
      <div className="flex justify-between items-end">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="section-title mb-0">AURA Studio // Quantum Consciousness IDE</h1>
            <span className={`px-2 py-0.5 rounded border text-[10px] font-mono font-bold uppercase tracking-wider ${
              unsupported
                ? 'bg-red-500/20 text-red-400 border-red-500/30'
                : 'bg-fuchsia-500/20 text-fuchsia-400 border-fuchsia-500/30'
            }`}>
              {unsupported ? 'AURA ENGINE NOT AVAILABLE' : `${metaconcepts.length} METACONCEPTS LOADED`}
            </span>
          </div>
          <p className="text-xs text-text-secondary mt-1 font-mono">
            Autonomous Universal Resonant Architecture (DSL Evaluator, Wavefunction Collapse, Semantic Fields)
          </p>
        </div>
        <div className="flex items-center gap-3">
          <div className="tag tag-info font-mono">
            ⚡ Energy: {fieldLoaded ? fieldState.energy : 'no data'}
          </div>
          <div className={`tag font-mono ${fieldLoaded ? 'tag-success' : 'tag-info'}`}>
            Φ Coherence: {fieldLoaded ? fieldState.coherence : 'no data'}
          </div>
          <button 
            onClick={async () => {
              try {
                const res = await axios.post(`${apiBase}/aura/council-integrate`);
                alert(`AURA Sealed by Council Quorum! Result: ${res.data.status}`);
                fetchField();
              } catch (e) {
                alert(`Integration failed: ${e.response?.data?.message || e.message}`);
              }
            }} 
            className="px-2.5 py-1 rounded bg-purple-500/20 hover:bg-purple-500/30 text-purple-300 border border-purple-500/40 text-xs font-mono font-bold flex items-center gap-1.5 transition-all"
            title="Submit AURA Module for Council Enactment"
          >
            <Sparkles size={12} />
            <span>Seal with Council</span>
          </button>
          <button onClick={() => { fetchMetaconcepts(); fetchField(); }} className="glass-button p-2 text-text-secondary hover:text-text-primary">
            <RefreshCw size={14} />
          </button>
        </div>
      </div>

      {unsupported && (
        <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-300 text-xs font-mono leading-relaxed">
          {unsupported}
        </div>
      )}

      {/* Main Workspace Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 flex-1 min-h-0">
        
        {/* Left 7 Columns: Code Editor & Execution REPL */}
        <div className="lg:col-span-7 flex flex-col gap-4 min-h-0">
          
          {/* Preset Templates */}
          <div className="flex gap-2 overflow-x-auto pb-1">
            {templates.map(t => (
              <button
                key={t.name}
                onClick={() => setCode(t.code)}
                className="px-3 py-1 bg-white/5 border border-white/10 rounded-lg text-xs font-mono hover:border-accent-primary hover:text-accent-primary transition-colors whitespace-nowrap"
              >
                {t.name}
              </button>
            ))}
          </div>

          {/* IDE Editor Panel */}
          <div className="panel flex-1 flex flex-col min-h-[260px]">
            <div className="panel-header">
              <div className="flex items-center gap-2">
                <Sparkles className="panel-icon text-fuchsia-400" size={16} />
                <h2 className="panel-title text-xs">AURA DSL Buffer (.aura)</h2>
              </div>
              <button 
                onClick={handleExecute} 
                disabled={executing}
                className="button-primary text-xs py-1.5 px-4 bg-gradient-to-r from-fuchsia-600 to-indigo-600 hover:from-fuchsia-500 hover:to-indigo-500 border-none shadow-[0_0_15px_rgba(217,70,239,0.3)]"
              >
                <Play size={14} className={executing ? "animate-spin" : ""} />
                <span>{executing ? "Resonating..." : "⚡ Execute AURA"}</span>
              </button>
            </div>
            <div className="panel-body flex-1 p-0 bg-background-primary/60 font-mono text-xs">
              <textarea
                value={code}
                onChange={(e) => setCode(e.target.value)}
                className="w-full h-full p-4 bg-transparent text-fuchsia-100 font-mono text-xs focus:outline-none resize-none custom-scrollbar leading-relaxed"
                placeholder="Enter AURA DSL code (e.g. Unity ~> Resonance -> MonadTruth)..."
              />
            </div>
          </div>

          {/* Cognitive Branching & Voltage Doctrine Engine */}
          <div className="panel flex flex-col p-4 bg-gradient-to-br from-fuchsia-950/20 via-black/40 to-cyan-950/20 border border-fuchsia-500/30 rounded-xl space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <GitBranch size={16} className="text-fuchsia-400" />
                <h3 className="font-mono font-bold text-xs text-fuchsia-300">
                  Cognitive Branching Engine // Kleene K3 Logic & Voltage Doctrine
                </h3>
              </div>
              <span className="text-[9px] font-mono px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
                {branchingResult?.k3_logic_evaluation?.action || 'NO ENGINE'}
              </span>
            </div>

            {/* Directive Input */}
            <div className="flex gap-2">
              <input
                type="text"
                value={directiveQuery}
                onChange={(e) => setDirectiveQuery(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleBranchDirective()}
                className="flex-1 bg-black/50 border border-white/10 rounded-lg px-3 py-1.5 text-xs font-mono text-fuchsia-100 focus:outline-none focus:border-fuchsia-400"
                placeholder="Enter directive to branch into K3 logic..."
              />
              <button
                onClick={handleBranchDirective}
                disabled={branchingLoading}
                className="px-3 py-1.5 bg-fuchsia-600/30 hover:bg-fuchsia-600/50 border border-fuchsia-500/40 text-fuchsia-200 text-xs font-mono font-bold rounded-lg flex items-center gap-1 transition-all"
              >
                <Zap size={12} className={branchingLoading ? "animate-pulse" : ""} />
                <span>{branchingLoading ? "Branching..." : "Branch"}</span>
              </button>
            </div>

            {/* Live Cognitive Branch Display */}
            {branchingResult && (
              <div className="space-y-2 pt-2 border-t border-white/10 text-xs font-mono">
                <div className="flex flex-wrap items-center justify-between gap-2 p-2 rounded bg-black/40 border border-white/5">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded bg-fuchsia-500/20 text-fuchsia-300 font-bold">
                      ROOT: {branchingResult.root_metaconcept?.name}
                    </span>
                    <span className="text-[10px] text-text-secondary">
                      ({branchingResult.root_metaconcept?.category} • polarity {branchingResult.root_metaconcept?.polarity})
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={`px-2 py-0.5 rounded text-[10px] font-bold ${
                      branchingResult.k3_logic_evaluation?.composite_trit === 1
                        ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40'
                        : 'bg-amber-500/20 text-amber-300 border border-amber-500/40'
                    }`}>
                      K3: {branchingResult.k3_logic_evaluation?.verdict}
                    </span>
                    <span className="px-2 py-0.5 rounded bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 text-[10px] font-bold">
                      {branchingResult.k3_logic_evaluation?.verdict}
                    </span>
                  </div>
                </div>

                {/* Branches Tree */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                  {(branchingResult.cognitive_branches || []).map((br, idx) => (
                    <div key={idx} className="p-2 rounded bg-white/5 border border-white/5 space-y-1">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-1.5 text-text-primary font-bold">
                          <span className="text-fuchsia-400">~&gt;</span>
                          <span>{br.target_name}</span>
                        </div>
                        <span className="text-[9px] px-1 rounded bg-accent-primary/20 text-accent-primary">
                          gate {br.k3_gate}
                        </span>
                      </div>
                      <div className="flex justify-between items-center text-[10px] text-text-secondary">
                        <span>K3 Invariant: {br.k3_verdict}</span>
                        <span>polarity {br.target_polarity}</span>
                      </div>
                      {br.sub_branches && br.sub_branches.length > 0 && (
                        <div className="pl-3 border-l border-white/10 mt-1 space-y-0.5 text-[9px] text-text-secondary">
                          {br.sub_branches.map((sb, sIdx) => (
                            <div key={sIdx} className="flex justify-between">
                              <span>↳ {sb.concept}</span>
                              <span className="text-emerald-400">{sb.k3_label}</span>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  ))}
                </div>

                {/* K3 Verdict Footer */}
                <div className="p-2 rounded bg-black/60 border border-emerald-500/20 flex flex-wrap justify-between items-center text-[10px] text-text-secondary">
                  <div className="flex items-center gap-3">
                    <span>Composite trit: <strong className="text-emerald-400">{branchingResult.k3_logic_evaluation?.composite_trit}</strong></span>
                    <span>Verdict: <strong className="text-cyan-400">{branchingResult.k3_logic_evaluation?.verdict}</strong></span>
                    <span>Action: <strong className="text-fuchsia-400">{branchingResult.k3_logic_evaluation?.action}</strong></span>
                  </div>
                  <span>Anchor: <code className="text-text-primary">{branchingResult.monad_anchor}</code></span>
                </div>
              </div>
            )}
          </div>

          {/* Live REPL Execution Trace */}
          {result && (
            <div className="panel flex-1 flex flex-col min-h-[180px]">
              <div className="panel-header bg-black/40">
                <div className="flex items-center gap-2">
                  <Activity size={14} className="text-emerald-400" />
                  <h3 className="panel-title text-xs">Resonance Execution Trace</h3>
                </div>
                {result.elapsed_ms && <span className="text-[10px] text-text-secondary font-mono">{result.elapsed_ms} ms</span>}
              </div>
              <div className="panel-body flex-1 overflow-y-auto custom-scrollbar p-3 space-y-2 bg-black/30 font-mono text-xs">
                {result.cognition_trace && (
                  <div className="p-2 rounded bg-fuchsia-500/10 border border-fuchsia-500/30 text-fuchsia-200 text-[11px] leading-relaxed">
                    {result.cognition_trace}
                  </div>
                )}
                {(result.events || []).map((ev, i) => (
                  <div key={i} className="p-2 bg-white/5 border border-white/5 rounded text-[11px] flex items-start gap-2">
                    <span className="px-1.5 py-0.5 rounded bg-accent-primary/20 text-accent-primary text-[9px] font-bold">{ev.type}</span>
                    <span className="text-text-primary flex-1">{ev.message}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

        </div>

        {/* Right 5 Columns: 100-Metaconcept Browser & Semantic Field Visualizer */}
        <div className="lg:col-span-5 flex flex-col gap-4 min-h-0">
          
          {/* Semantic Field Topology */}
          <div className="panel flex-1 flex flex-col min-h-[220px]">
            <div className="panel-header">
              <div className="flex items-center gap-2">
                <Compass className="panel-icon text-cyan-400" size={16} />
                <h2 className="panel-title text-xs">Semantic Field Density & Bonds</h2>
              </div>
              <div className="tag tag-success text-[9px]">{(fieldState.connections || []).length} Bonds</div>
            </div>
            <div className="panel-body flex-1 overflow-y-auto custom-scrollbar p-3 space-y-2">
              <div className="text-[10px] font-mono text-text-secondary uppercase tracking-wider mb-1">Active Resonant Nodes</div>
              <div className="flex flex-wrap gap-1.5 mb-3">
                {(fieldState.active_nodes || []).map(node => (
                  <span key={node} className="px-2 py-0.5 bg-cyan-500/10 border border-cyan-500/30 text-cyan-300 rounded text-[10px] font-mono">
                    {node}
                  </span>
                ))}
              </div>
              <div className="text-[10px] font-mono text-text-secondary uppercase tracking-wider mb-1">Entangled Connections</div>
              <div className="space-y-1">
                {(fieldState.connections || []).slice(-6).map((c, i) => (
                  <div key={i} className="p-1.5 bg-white/5 rounded flex justify-between items-center text-[10px] font-mono">
                    <span className="text-text-primary">{c.source} <span className="text-fuchsia-400">~&gt;</span> {c.target}</span>
                    <span className="text-accent-primary font-bold">{c.strength}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* 100 Metaconcept Catalog */}
          <div className="panel flex-1 flex flex-col min-h-[300px]">
            <div className="panel-header">
              <div className="flex items-center gap-2">
                <BookOpen className="panel-icon text-amber-400" size={16} />
                <h2 className="panel-title text-xs">Metaconcept Matrix ({filteredConcepts.length})</h2>
              </div>
              <select 
                value={selectedCategory} 
                onChange={e => setSelectedCategory(e.target.value)}
                className="input-field text-[10px] py-1 px-2"
                style={{ maxWidth: 130 }}
              >
                <option value="ALL">All Categories</option>
                {categories.map(c => <option key={c} value={c}>{c}</option>)}
              </select>
            </div>
            <div className="panel-body flex-1 overflow-y-auto custom-scrollbar p-3 space-y-2">
              {filteredConcepts.map(m => (
                <div 
                  key={m.name} 
                  onClick={() => {
                    setCode(prev => `${prev}\n${m.name} ~> MonadTruth`);
                    setDirectiveQuery(`Branch metaconcept ${m.name} under Voltage Doctrine`);
                  }}
                  className="p-2 bg-white/5 border border-white/5 hover:border-fuchsia-500/50 rounded-lg cursor-pointer transition-colors"
                >
                  <div className="flex justify-between items-start mb-1">
                    <span className="font-mono font-bold text-xs text-text-primary">{m.name}</span>
                    <span className="text-[9px] px-1.5 py-0.5 rounded bg-fuchsia-500/20 text-fuchsia-400 font-mono">{m.category} • {m.polarity > 0 ? '+' : ''}{m.polarity}</span>
                  </div>
                  <p className="text-[10px] text-text-secondary line-clamp-2">{m.description}</p>
                </div>
              ))}
            </div>
          </div>

        </div>

      </div>
    </div>
  );
}
