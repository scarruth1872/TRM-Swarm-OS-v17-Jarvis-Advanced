import React, { useState, useEffect } from 'react';
import { Cpu, Shield, Zap, Radio, Play, Loader2, CheckCircle2, AlertTriangle } from 'lucide-react';

export default function SovereignBenchPanel() {
  const [agents584, setAgents584] = useState(null);
  const [cosmos, setCosmos] = useState(null);
  const [federation, setFederation] = useState(null);
  const [gpuResult, setGpuResult] = useState(null);
  const [gpuBusy, setGpuBusy] = useState(null);

  const loadLive = async () => {
    try {
      const [a, c, f] = await Promise.all([
        fetch('/agents/584/status').then(r => r.ok ? r.json() : null).catch(() => null),
        fetch('/cosmos/status').then(r => r.ok ? r.json() : null).catch(() => null),
        fetch('/federation/stats').then(r => r.ok ? r.json() : null).catch(() => null),
      ]);
      if (a) setAgents584(a);
      if (c) setCosmos(c);
      if (f) setFederation(f);
    } catch (_) {}
  };

  useEffect(() => {
    loadLive();
    const id = setInterval(loadLive, 8000);
    return () => clearInterval(id);
  }, []);

  const runBench = async (label, endpoint) => {
    setGpuBusy(label);
    try {
      const resp = await fetch(endpoint);
      const data = await resp.json();
      setGpuResult({ label, data });
    } catch (e) {
      setGpuResult({ label, data: { error: String(e) } });
    } finally {
      setGpuBusy(null);
    }
  };

  return (
    <div className="panel mt-2">
      <div className="panel-header">
        <h2 className="panel-title">Sovereign Compute, 584-Agent Firewall & Federation (Live)</h2>
        <Zap className="panel-icon" size={18} />
      </div>
      <div className="panel-body space-y-4">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
          <div className="p-3 rounded bg-background-primary/60 border border-white/10 space-y-1">
            <div className="flex items-center gap-2 font-semibold text-accent-primary">
              <Shield size={15} /> 584-Agent Bitboard
            </div>
            <div className="flex justify-between"><span>Agents / Lanes:</span><span>{agents584 ? `${agents584.agent_count} / ${agents584.bitboard_lanes}` : '—'}</span></div>
            <div className="flex justify-between"><span>Arena Frame:</span><span>{agents584 ? `${agents584.arena_frame_bytes} B` : '—'}</span></div>
            <div className="flex justify-between"><span>Evals / Consensus:</span><span>{agents584 ? `${agents584.eval_count} (${agents584.last_consensus_trit})` : '—'}</span></div>
          </div>

          <div className="p-3 rounded bg-background-primary/60 border border-white/10 space-y-1">
            <div className="flex items-center gap-2 font-semibold text-accent-primary">
              <Cpu size={15} /> Cosmos 3 Edge Bridge
            </div>
            <div className="flex justify-between"><span>Chunks Processed:</span><span>{cosmos ? cosmos.chunks_processed : '—'}</span></div>
            <div className="flex justify-between"><span>Hazards Intercepted:</span><span>{cosmos ? cosmos.hazards_intercepted : '—'}</span></div>
            <div className="flex justify-between"><span>Latest Verdict:</span><span>{cosmos ? cosmos.latest_verdict : '—'}</span></div>
          </div>

          <div className="p-3 rounded bg-background-primary/60 border border-white/10 space-y-1">
            <div className="flex items-center gap-2 font-semibold text-accent-primary">
              <Radio size={15} /> Tailscale Federation
            </div>
            <div className="flex justify-between"><span>Active / Configured:</span><span>{federation ? `${federation.active_nodes} / ${federation.configured_nodes}` : '—'}</span></div>
            <div className="flex justify-between"><span>Verified Peers:</span><span>{federation ? federation.verified_remote_peers : '—'}</span></div>
            <div className="flex justify-between"><span>Offline Nodes:</span><span>{federation ? federation.offline_nodes : '—'}</span></div>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2 pt-1">
          <button
            onClick={() => runBench('Infinity Cache K3', '/gpu/infinity/bench')}
            disabled={!!gpuBusy}
            className="px-3 py-1.5 rounded bg-accent-primary/20 hover:bg-accent-primary/30 border border-accent-primary/40 text-xs font-mono flex items-center gap-1.5"
          >
            {gpuBusy === 'Infinity Cache K3' ? <Loader2 size={13} className="animate-spin" /> : <Play size={13} />}
            Run RX 6700 XT Infinity K3
          </button>
          <button
            onClick={() => runBench('200M Drug Screen', '/gpu/screen/bench')}
            disabled={!!gpuBusy}
            className="px-3 py-1.5 rounded bg-accent-primary/20 hover:bg-accent-primary/30 border border-accent-primary/40 text-xs font-mono flex items-center gap-1.5"
          >
            {gpuBusy === '200M Drug Screen' ? <Loader2 size={13} className="animate-spin" /> : <Play size={13} />}
            Run 200M-Pair 64-Trit Screen
          </button>
          <button
            onClick={() => runBench('4-Arch Switch', '/gpu/switch/bench')}
            disabled={!!gpuBusy}
            className="px-3 py-1.5 rounded bg-accent-primary/20 hover:bg-accent-primary/30 border border-accent-primary/40 text-xs font-mono flex items-center gap-1.5"
          >
            {gpuBusy === '4-Arch Switch' ? <Loader2 size={13} className="animate-spin" /> : <Play size={13} />}
            Run 4-Arch Switch (1,000x)
          </button>
        </div>

        {gpuResult && (
          <div className="p-3 rounded bg-black/40 border border-white/10 font-mono text-xs overflow-x-auto">
            <div className="flex items-center gap-2 mb-1 font-semibold">
              {gpuResult.data?.error ? <AlertTriangle size={14} className="text-red-400" /> : <CheckCircle2 size={14} className="text-green-400" />}
              <span>{gpuResult.label} ({gpuResult.data?.adapter || 'GPU'})</span>
            </div>
            <pre className="whitespace-pre-wrap text-[11px] opacity-90">{JSON.stringify(gpuResult.data, null, 2)}</pre>
          </div>
        )}
      </div>
    </div>
  );
}
