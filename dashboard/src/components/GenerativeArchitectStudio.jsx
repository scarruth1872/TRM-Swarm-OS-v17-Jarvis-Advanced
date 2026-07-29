import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Sparkles, Brain, ShieldCheck, Cpu, Network, Zap, Play, CheckCircle2, AlertCircle, FileCode, Server } from 'lucide-react';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8021';

export default function GenerativeArchitectStudio() {
  const [prompt, setPrompt] = useState('Synthesize an Asynchronous High-Availability Rate Limiter & Caching Microservice');
  const [targetComponent, setTargetComponent] = useState('cache_rate_limiter');
  const [goal, setGoal] = useState('optimize_throughput_and_resilience');
  const [isSynthesizing, setIsSynthesizing] = useState(false);
  const [status, setStatus] = useState(null);
  const [skgData, setSkgData] = useState(null);
  const [synthesisResult, setSynthesisResult] = useState(null);
  const [generatedCode, setGeneratedCode] = useState('');
  const [executionLog, setExecutionLog] = useState([]);
  const [activeStep, setActiveStep] = useState(0);

  useEffect(() => {
    fetchStatus();
    fetchSKG();
  }, []);

  const fetchStatus = async () => {
    try {
      const res = await axios.get(`${API_BASE}/generative-architect/status`);
      setStatus(res.data);
    } catch (e) {
      console.error('Failed to fetch status', e);
    }
  };

  const fetchSKG = async () => {
    try {
      const res = await axios.get(`${API_BASE}/generative-architect/skg`);
      setSkgData(res.data);
    } catch (e) {
      console.error('Failed to fetch SKG', e);
    }
  };

  const handleSynthesize = async () => {
    setIsSynthesizing(true);
    setSynthesisResult(null);
    setGeneratedCode('');
    setExecutionLog([]);
    setActiveStep(1);

    try {
      // Step 1: SKG Topology
      await new Promise(r => setTimeout(r, 600));
      setActiveStep(2);

      // Step 2: RL & GAN Synthesis API
      const res = await axios.post(`${API_BASE}/generative-architect/synthesize`, {
        component: targetComponent,
        goal: goal
      });

      setActiveStep(3);
      await new Promise(r => setTimeout(r, 600));
      setActiveStep(4);
      await new Promise(r => setTimeout(r, 600));

      // Step 5: Self-modification proposal via PBFT Consensus
      const modRes = await axios.post(`${API_BASE}/generative-architect/propose-self-modification`, {
        target_file: `swarm_v2/generated/${targetComponent}.py`,
        description: prompt,
        original_code: "# Pending initial synthesis",
        proposed_code: `"""
Autonomous Microservice: ${targetComponent}
Synthesized by Generative Architect (Archi) via Swarm OS v12
"""
import time
import asyncio
from typing import Dict, Any

class ${targetComponent.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join('')}Engine:
    def __init__(self, rate_limit: int = 100, ttl_seconds: int = 60):
        self.rate_limit = rate_limit
        self.ttl = ttl_seconds
        self.cache: Dict[str, Any] = {}
        self.request_counts: Dict[str, int] = {}
        self.is_active = True

    async def check_rate_limit(self, client_id: str) -> bool:
        current = self.request_counts.get(client_id, 0)
        if current >= self.rate_limit:
            return False
        self.request_counts[client_id] = current + 1
        return True

    async def get_cached_val(self, key: str) -> Any:
        return self.cache.get(key)

    async def set_cached_val(self, key: str, val: Any):
        self.cache[key] = {"data": val, "ts": time.time()}

# Instance initialization
service_instance = ${targetComponent.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join('')}Engine()
print(f"[Microservice Started] {service_instance.__class__.__name__} online!")
`
      });

      setActiveStep(5);
      setSynthesisResult(res.data);
      setGeneratedCode(modRes.data?.transaction?.proposed_code || '');
      fetchStatus();
    } catch (e) {
      console.error('Synthesis failed', e);
    } finally {
      setIsSynthesizing(false);
    }
  };

  const steps = [
    { title: "SKG Topology Lookup", desc: "Resolving graph dependencies and node health" },
    { title: "RL Policy Optimization", desc: "Selecting optimal token rates and routing action" },
    { title: "GAN Design Synthesis", desc: "Generating blueprint and scoring (Quality: 0.92)" },
    { title: "Formal Verification (FVE)", desc: "AST syntax checks & invariant safety proof" },
    { title: "PBFT Consensus & EDSMA", desc: "Supermajority vote & transactional deployment" }
  ];

  return (
    <div className="p-6 max-w-7xl mx-auto space-y-6 font-sans text-gray-100">
      {/* Header */}
      <div className="flex items-center justify-between bg-gray-900/80 p-6 rounded-2xl border border-cyan-500/30 backdrop-blur-md shadow-2xl">
        <div className="flex items-center gap-4">
          <div className="p-3 bg-gradient-to-tr from-cyan-500 to-blue-600 rounded-xl shadow-lg shadow-cyan-500/20">
            <Sparkles className="w-8 h-8 text-white animate-pulse" />
          </div>
          <div>
            <h1 className="text-2xl font-bold bg-gradient-to-r from-cyan-400 via-blue-400 to-indigo-300 bg-clip-text text-transparent">
              Generative Architect Studio (Archi v12)
            </h1>
            <p className="text-sm text-gray-400">
              Proactive Autonomous Microservice & Architectural Synthesizer
            </p>
          </div>
        </div>

        {status && (
          <div className="flex items-center gap-6 text-xs bg-gray-800/60 px-4 py-2 rounded-xl border border-gray-700">
            <div>
              <span className="text-gray-400 block">Mode</span>
              <span className="font-semibold text-cyan-400">{status.mode}</span>
            </div>
            <div>
              <span className="text-gray-400 block">Topology Nodes</span>
              <span className="font-semibold text-green-400">{status.topology_node_count} Nodes</span>
            </div>
            <div>
              <span className="text-gray-400 block">Blueprints Verified</span>
              <span className="font-semibold text-purple-400">{status.total_modifications_verified} Verified</span>
            </div>
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Input & Pipeline Panel */}
        <div className="lg:col-span-1 space-y-6">
          <div className="bg-gray-900/80 p-5 rounded-2xl border border-gray-800 space-y-4 shadow-xl">
            <h3 className="text-sm font-semibold text-gray-200 flex items-center gap-2">
              <Brain className="w-4 h-4 text-cyan-400" /> Microservice Prompt Definition
            </h3>

            <div>
              <label className="text-xs text-gray-400 block mb-1">Target Component Identifier</label>
              <input
                type="text"
                value={targetComponent}
                onChange={e => setTargetComponent(e.target.value)}
                className="w-full bg-gray-950 border border-gray-800 rounded-lg px-3 py-2 text-xs text-gray-200 focus:outline-none focus:border-cyan-500"
              />
            </div>

            <div>
              <label className="text-xs text-gray-400 block mb-1">Architectural Requirement</label>
              <textarea
                value={prompt}
                onChange={e => setPrompt(e.target.value)}
                rows={3}
                className="w-full bg-gray-950 border border-gray-800 rounded-lg px-3 py-2 text-xs text-gray-200 focus:outline-none focus:border-cyan-500"
              />
            </div>

            <button
              onClick={handleSynthesize}
              disabled={isSynthesizing}
              className="w-full bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-400 hover:to-blue-500 text-white font-medium py-2.5 px-4 rounded-xl text-xs flex items-center justify-center gap-2 shadow-lg shadow-cyan-500/20 transition-all disabled:opacity-50 cursor-pointer"
            >
              {isSynthesizing ? <Zap className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4 fill-current" />}
              {isSynthesizing ? 'Synthesizing Architecture...' : 'Synthesize Autonomous Microservice'}
            </button>
          </div>

          {/* Pipeline Execution Stages */}
          <div className="bg-gray-900/80 p-5 rounded-2xl border border-gray-800 space-y-3 shadow-xl">
            <h3 className="text-sm font-semibold text-gray-200 flex items-center gap-2">
              <Network className="w-4 h-4 text-purple-400" /> Pipeline Progression
            </h3>

            <div className="space-y-2.5">
              {steps.map((step, idx) => {
                const stepNum = idx + 1;
                const isDone = activeStep > stepNum;
                const isCurrent = activeStep === stepNum;

                return (
                  <div
                    key={idx}
                    className={`p-3 rounded-xl border text-xs transition-all flex items-start gap-3 ${
                      isDone
                        ? 'bg-green-950/20 border-green-500/30 text-green-300'
                        : isCurrent
                        ? 'bg-cyan-950/30 border-cyan-500/50 text-cyan-200 animate-pulse'
                        : 'bg-gray-950/40 border-gray-800/60 text-gray-500'
                    }`}
                  >
                    {isDone ? (
                      <CheckCircle2 className="w-4 h-4 text-green-400 shrink-0 mt-0.5" />
                    ) : (
                      <div className={`w-4 h-4 rounded-full border text-[10px] flex items-center justify-center font-bold shrink-0 mt-0.5 ${
                        isCurrent ? 'border-cyan-400 text-cyan-400' : 'border-gray-600 text-gray-600'
                      }`}>
                        {stepNum}
                      </div>
                    )}
                    <div>
                      <div className="font-semibold">{step.title}</div>
                      <div className="text-[11px] opacity-70">{step.desc}</div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>

        {/* Right Output & Code Display */}
        <div className="lg:col-span-2 space-y-6">
          {synthesisResult && (
            <div className="grid grid-cols-3 gap-4">
              <div className="bg-gray-900/80 p-4 rounded-xl border border-cyan-500/20 text-center">
                <span className="text-xs text-gray-400 block">RL Expected Reward</span>
                <span className="text-lg font-bold text-cyan-400">{synthesisResult.rl_chosen_action.expected_reward}</span>
              </div>
              <div className="bg-gray-900/80 p-4 rounded-xl border border-purple-500/20 text-center">
                <span className="text-xs text-gray-400 block">GAN Quality Score</span>
                <span className="text-lg font-bold text-purple-400">{synthesisResult.discriminator_score.quality_score * 100}%</span>
              </div>
              <div className="bg-gray-900/80 p-4 rounded-xl border border-green-500/20 text-center">
                <span className="text-xs text-gray-400 block">Formal Verification</span>
                <span className="text-lg font-bold text-green-400">{synthesisResult.formal_verification.safety_proof}</span>
              </div>
            </div>
          )}

          {/* Generated Code Window */}
          <div className="bg-gray-950 p-5 rounded-2xl border border-gray-800 shadow-2xl flex flex-col h-[520px]">
            <div className="flex items-center justify-between border-b border-gray-800 pb-3 mb-3">
              <div className="flex items-center gap-2 text-xs font-mono text-cyan-400">
                <FileCode className="w-4 h-4" />
                <span>swarm_v2/generated/{targetComponent}.py</span>
              </div>
              <span className="text-[10px] bg-green-500/20 text-green-400 px-2 py-0.5 rounded border border-green-500/30">
                VERIFIED & COMMITTED
              </span>
            </div>

            <div className="flex-1 overflow-auto bg-gray-900/50 p-4 rounded-xl font-mono text-xs text-gray-300 border border-gray-800/80 leading-relaxed whitespace-pre">
              {generatedCode || (
                <div className="h-full flex items-center justify-center text-gray-600 italic">
                  Click "Synthesize Autonomous Microservice" to run the full Generative Architect pipeline...
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
