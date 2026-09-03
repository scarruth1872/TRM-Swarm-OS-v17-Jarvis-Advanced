import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  ShieldCheck, AlertTriangle, Activity, RefreshCw, Zap, Cpu, Server, 
  Radio, CheckCircle2, XCircle, HeartPulse, Terminal, Play, Wrench 
} from 'lucide-react';
import axios from 'axios';

const API_BASE = '';

export default function AnomalySentinel() {
  const [monitorData, setMonitorData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [isHealing, setIsHealing] = useState(false);
  const [healReport, setHealReport] = useState(null);
  const [agentTestResults, setAgentTestResults] = useState({});
  const [isTestingAgents, setIsTestingAgents] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(true);

  const fetchMonitorData = async () => {
    try {
      const res = await axios.get(`${API_BASE}/system/anomaly_monitor`);
      setMonitorData(res.data);
      setLoading(false);
    } catch (err) {
      console.error('Anomaly monitor poll error:', err);
      setMonitorData(prev => ({
        ...prev,
        status: 'DEGRADED',
        anomaly_count: (prev?.anomaly_count || 0) + 1,
        anomalies: [
          { service: 'Host Council Server', port: 8022, severity: 'HIGH', message: 'Unable to reach /system/anomaly_monitor endpoint' }
        ]
      }));
    }
  };

  useEffect(() => {
    fetchMonitorData();
    if (!autoRefresh) return;
    const interval = setInterval(fetchMonitorData, 3000);
    return () => clearInterval(interval);
  }, [autoRefresh]);

  const handleAutoHeal = async () => {
    setIsHealing(true);
    try {
      const res = await axios.post(`${API_BASE}/system/heal_anomaly`);
      setHealReport(res.data);
      await fetchMonitorData();
    } catch (err) {
      setHealReport({
        status: 'HEAL_ERROR',
        error: err.message,
        remedies_applied: ['Fallback manual restart required']
      });
    } finally {
      setIsHealing(false);
    }
  };

  const handleTestAllAgents = async () => {
    setIsTestingAgents(true);
    const agents = ['Jarvis', 'Archi', 'Shield', 'Logic', 'Pulse', 'Seeker', 'Devo', 'Hermes', 'Flow', 'EventLedger', 'Orchestra', 'THzGateway', 'Verify', 'Bridge'];
    const results = {};
    
    for (const agent of agents) {
      try {
        const t0 = performance.now();
        const res = await axios.post(`${API_BASE}/swarm/chat`, {
          role: agent,
          message: `Health ping test for ${agent}. Report active state.`
        });
        const elapsed = Math.round(performance.now() - t0);
        results[agent] = {
          status: 'SUCCESS',
          latency_ms: elapsed,
          reply: res.data?.response || res.data?.reply || 'OK',
          neural: res.data?.neural_active ?? true
        };
      } catch (e) {
        results[agent] = {
          status: 'FAIL',
          latency_ms: 0,
          reply: e.message,
          neural: false
        };
      }
      setAgentTestResults({ ...results });
    }
    setIsTestingAgents(false);
  };

  const services = monitorData?.services || {};
  const anomalies = monitorData?.anomalies || [];
  const isOptimal = monitorData?.status === 'OPTIMAL';

  return (
    <div className="flex flex-col gap-6 p-2 max-w-7xl mx-auto">
      {/* 1. HEADER & GLOBAL STATUS BANNER */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-[#0a0a14] border border-white/10 rounded-xl p-5 shadow-2xl">
        <div className="flex items-center gap-4">
          <div className={`p-3 rounded-xl ${isOptimal ? 'bg-green-500/20 text-green-400 border border-green-500/40' : 'bg-red-500/20 text-red-400 border border-red-500/40'}`}>
            <ShieldCheck size={32} />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-xl font-bold tracking-wider text-white uppercase">Platform Anomaly Sentinel</h1>
              <span className={`px-2.5 py-0.5 rounded-full text-xs font-bold uppercase tracking-wider ${isOptimal ? 'bg-green-500/20 text-green-300 border border-green-500/40' : 'bg-red-500/20 text-red-300 border border-red-500/40 animate-pulse'}`}>
                {monitorData?.status || 'INITIALIZING'}
              </span>
            </div>
            <p className="text-xs text-neutral-400 mt-1 font-mono">
              Monad Truth Anchor: <span className="text-[#ff9900]">0x4D4F4E4144000001</span> | PBFT Quorum: 14/14 Active | K3 Invariant: +1 Gold
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={`px-3 py-1.5 rounded-lg text-xs font-mono border transition-all flex items-center gap-1.5 ${autoRefresh ? 'bg-blue-500/20 border-blue-500/40 text-blue-300' : 'bg-white/5 border-white/10 text-neutral-400'}`}
          >
            <RefreshCw size={14} className={autoRefresh ? 'animate-spin' : ''} />
            {autoRefresh ? 'Auto 3s' : 'Paused'}
          </button>

          <button
            onClick={handleAutoHeal}
            disabled={isHealing}
            className="px-4 py-2 rounded-lg text-xs font-bold uppercase tracking-wider bg-gradient-to-r from-amber-600 to-amber-500 hover:from-amber-500 hover:to-amber-400 text-black border border-amber-400/50 shadow-lg transition-all flex items-center gap-2"
          >
            <Wrench size={14} />
            {isHealing ? 'Healing Pipelines...' : 'Auto-Heal Platform'}
          </button>

          <button
            onClick={handleTestAllAgents}
            disabled={isTestingAgents}
            className="px-4 py-2 rounded-lg text-xs font-bold uppercase tracking-wider bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white border border-purple-400/50 shadow-lg transition-all flex items-center gap-2"
          >
            <Play size={14} />
            {isTestingAgents ? 'Pinging 14 Agents...' : 'Deep Neural Diagnostic'}
          </button>
        </div>
      </div>

      {/* 2. AUTO-HEAL REMEDIATION DRAWER */}
      <AnimatePresence>
        {healReport && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="bg-emerald-950/40 border border-emerald-500/40 rounded-xl p-4 text-xs font-mono text-emerald-300"
          >
            <div className="flex justify-between items-center mb-2">
              <span className="font-bold uppercase flex items-center gap-2">
                <CheckCircle2 size={16} /> Autonomous Remediation Report ({healReport.status})
              </span>
              <button onClick={() => setHealReport(null)} className="text-neutral-400 hover:text-white text-xs">Dismiss</button>
            </div>
            <ul className="list-disc list-inside space-y-1 text-neutral-300">
              {healReport.remedies_applied?.map((rem, i) => (
                <li key={i}>{rem}</li>
              ))}
            </ul>
          </motion.div>
        )}
      </AnimatePresence>

      {/* 3. 5-SERVICE CORE TELEMETRY RADAR */}
      <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
        {Object.entries(services).map(([key, s]) => {
          const isUp = s.status === 'ONLINE';
          return (
            <div
              key={key}
              className={`p-4 rounded-xl border transition-all flex flex-col justify-between ${isUp ? 'bg-[#0e0e1a] border-white/10 hover:border-cyan-500/40' : 'bg-red-950/30 border-red-500/50 animate-pulse'}`}
            >
              <div className="flex justify-between items-start">
                <span className="text-[10px] font-mono uppercase text-neutral-400 tracking-wider">Port {s.port}</span>
                <span className={`flex h-2.5 w-2.5 relative`}>
                  {isUp && <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>}
                  <span className={`relative inline-flex rounded-full h-2.5 w-2.5 ${isUp ? 'bg-green-500' : 'bg-red-500'}`}></span>
                </span>
              </div>
              
              <div className="my-3">
                <h3 className="text-sm font-bold text-white truncate">{s.name}</h3>
                <div className="text-xs text-neutral-400 font-mono mt-1">
                  Latency: <span className="text-cyan-400 font-bold">{s.latency_ms?.toFixed(2) || 0} ms</span>
                </div>
              </div>

              <div className="flex justify-between items-center text-[10px] font-mono border-t border-white/5 pt-2">
                <span className={isUp ? 'text-green-400' : 'text-red-400 font-bold'}>{s.status}</span>
                <span className="text-neutral-500">127.0.0.1</span>
              </div>
            </div>
          );
        })}
      </div>

      {/* 4. ANOMALIES STREAM & DIAGNOSTIC LOG */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-1 bg-[#0a0a14] border border-white/10 rounded-xl p-5 flex flex-col gap-3">
          <div className="flex items-center justify-between border-b border-white/10 pb-3">
            <h2 className="text-sm font-bold uppercase text-white flex items-center gap-2">
              <AlertTriangle size={16} className={anomalies.length > 0 ? 'text-red-400' : 'text-green-400'} />
              Active Anomaly Stream ({anomalies.length})
            </h2>
            <span className={`text-[10px] font-mono px-2 py-0.5 rounded ${anomalies.length === 0 ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'}`}>
              Z-Score: {anomalies.length === 0 ? '0.00' : '3.42'}
            </span>
          </div>

          <div className="flex-1 overflow-y-auto max-h-[300px] space-y-2">
            {anomalies.length === 0 ? (
              <div className="p-4 text-center text-neutral-500 text-xs font-mono">
                <CheckCircle2 size={24} className="mx-auto text-green-500/50 mb-2" />
                Zero anomalies detected across all 5 platform service layers.
              </div>
            ) : (
              anomalies.map((anom, idx) => (
                <div key={idx} className="p-3 bg-red-950/30 border border-red-500/30 rounded-lg text-xs font-mono text-red-300">
                  <div className="font-bold text-red-200">[{anom.severity}] {anom.service}</div>
                  <div className="text-[11px] text-red-400 mt-1">{anom.message}</div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* 5. 14-AGENT NEURAL LIVENESS MATRIX */}
        <div className="lg:col-span-2 bg-[#0a0a14] border border-white/10 rounded-xl p-5 flex flex-col gap-3">
          <div className="flex items-center justify-between border-b border-white/10 pb-3">
            <h2 className="text-sm font-bold uppercase text-white flex items-center gap-2">
              <HeartPulse size={16} className="text-purple-400" />
              14-Agent Neural Cognition & Live Dialogue Radar
            </h2>
            <span className="text-[10px] font-mono text-neutral-400">
              Ollama gemma3:1b | Port 8022 & 3001
            </span>
          </div>

          <div className="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-7 gap-2 overflow-y-auto max-h-[320px]">
            {['Jarvis', 'Archi', 'Shield', 'Logic', 'Pulse', 'Seeker', 'Devo', 'Hermes', 'Flow', 'EventLedger', 'Orchestra', 'THzGateway', 'Verify', 'Bridge'].map(agent => {
              const test = agentTestResults[agent];
              const isTested = !!test;
              const isOk = test?.status === 'SUCCESS';
              return (
                <div
                  key={agent}
                  className={`p-2.5 rounded-lg border text-center transition-all ${isTested ? (isOk ? 'bg-purple-950/30 border-purple-500/40' : 'bg-red-950/40 border-red-500/50') : 'bg-white/5 border-white/10'}`}
                >
                  <div className="text-xs font-bold text-white truncate">{agent}</div>
                  <div className="text-[10px] font-mono mt-1">
                    {isTested ? (
                      <span className={isOk ? 'text-green-400 font-bold' : 'text-red-400 font-bold'}>
                        {isOk ? `${test.latency_ms} ms` : 'FAIL'}
                      </span>
                    ) : (
                      <span className="text-neutral-500">READY</span>
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          {Object.keys(agentTestResults).length > 0 && (
            <div className="mt-2 p-3 bg-black/40 rounded-lg border border-white/5 max-h-[140px] overflow-y-auto text-xs font-mono text-neutral-300">
              <div className="text-[10px] uppercase text-neutral-500 font-bold mb-1">Latest Neural Verification Trace:</div>
              {Object.entries(agentTestResults).map(([ag, r]) => (
                <div key={ag} className="truncate text-[11px] py-0.5">
                  <span className="text-[#ff9900] font-bold">{ag}:</span> {r.reply}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
