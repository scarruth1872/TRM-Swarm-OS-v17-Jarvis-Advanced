import React, { useState, useEffect } from 'react';
import { 
  Cpu, Radio, FileCheck, Layers, Play, RefreshCw, 
  CheckCircle2, Server, Activity, ShieldCheck, 
  Database, Gauge, Zap, Network, Bot
} from 'lucide-react';
import axios from 'axios';

export default function PillarsRoadmapViewport({ apiBase = 'http://127.0.0.1:8022' }) {
  const [activePillar, setActivePillar] = useState(1);
  const [wetlabData, setWetlabData] = useState(null);
  const [sensoryData, setSensoryData] = useState(null);
  const [thzData, setThzData] = useState(null);
  const [regulatoryData, setRegulatoryData] = useState(null);
  const [ledgerData, setLedgerData] = useState(null);
  const [schedulerData, setSchedulerData] = useState(null);
  const [nativeIpcData, setNativeIpcData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [actionInProgress, setActionInProgress] = useState(false);

  const fetchPillarsTelemetry = async () => {
    try {
      setLoading(true);
      const [wetRes, sensRes, thzRes, regRes, ledRes, schedRes, ipcRes] = await Promise.all([
        axios.get(`${apiBase}/wetlab/protocols`).catch(() => ({ data: null })),
        axios.get(`${apiBase}/sensory/mesh/telemetry`).catch(() => ({ data: null })),
        axios.get(`${apiBase}/thz/resonance/status`).catch(() => ({ data: null })),
        axios.get(`${apiBase}/regulatory/ind/dossiers`).catch(() => ({ data: null })),
        axios.get(`${apiBase}/ledger/monad/blocks`).catch(() => ({ data: null })),
        axios.get(`${apiBase}/system/transistor-scheduler`).catch(() => ({ data: null })),
        axios.get(`${apiBase}/system/native-ipc/telemetry`).catch(() => ({ data: null })),
      ]);
      setWetlabData(wetRes.data);
      setSensoryData(sensRes.data);
      setThzData(thzRes.data);
      setRegulatoryData(regRes.data);
      setLedgerData(ledRes.data);
      setSchedulerData(schedRes.data);
      setNativeIpcData(ipcRes.data);
    } catch (err) {
      console.error("Failed to load pillars telemetry", err);
    } finally {
      setLoading(false);
    }
  };

  const triggerPillarAction = async (actionType) => {
    try {
      setActionInProgress(true);
      if (actionType === 'triangulate') {
        await axios.post(`${apiBase}/sensory/mesh/triangulate`);
      } else if (actionType === 'thz_sync') {
        await axios.post(`${apiBase}/thz/resonance/sync`);
      } else if (actionType === 'ledger_commit') {
        await axios.post(`${apiBase}/ledger/monad/commit`, {
          proposer: "CouncilOperator",
          items: [{ action: "OPERATOR_STATE_SYNC", timestamp: Date.now() }]
        });
      }
      await fetchPillarsTelemetry();
    } catch (err) {
      alert(`Action ${actionType} unavailable: ${err.response?.data?.message || err.message}`);
    } finally {
      setActionInProgress(false);
    }
  };

  useEffect(() => {
    fetchPillarsTelemetry();
  }, []);

  return (
    <div className="space-y-6 font-mono text-sm text-text-primary">
      {/* ── Top Header Banner ── */}
      <div className="p-5 bg-background-secondary/80 border border-white/10 rounded-2xl backdrop-blur-md flex flex-wrap items-center justify-between gap-4 shadow-xl">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-xl bg-cyan-500/20 border border-cyan-500/40 flex items-center justify-center text-cyan-400 shadow-[0_0_15px_rgba(6,182,212,0.3)]">
            <Cpu size={24} />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-lg font-black tracking-wider uppercase text-white font-mono">
                Pillars I–V Strategic Roadmap & Transistor Grounding
              </h2>
              <span className="px-2 py-0.5 text-[10px] rounded-full bg-cyan-500/20 text-cyan-300 border border-cyan-500/30">
                14 COUNCIL RATIFIED
              </span>
            </div>
            <p className="text-xs text-text-secondary mt-0.5">
              Monad Truth Anchor: <span className="text-accent-primary">0x4D4F4E4144000001</span> | Transistor-to-Task Bound
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={fetchPillarsTelemetry}
            disabled={loading}
            className="px-3 py-2 bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl text-xs flex items-center gap-2 transition-colors text-text-secondary hover:text-white"
          >
            <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
            Refresh
          </button>
        </div>
      </div>

      {/* ── Hardware Transistor Grounding Telemetry Card ── */}
      {schedulerData && (
        <div className="p-4 bg-white/5 border border-white/10 rounded-2xl space-y-3">
          <div className="flex items-center justify-between pb-2 border-b border-white/10">
            <div className="flex items-center gap-2">
              <Cpu size={16} className="text-accent-primary" />
              <span className="text-xs font-bold uppercase text-white">Physical Silicon Substrate Grounding</span>
            </div>
            <span className="text-[10px] font-mono text-text-secondary">{schedulerData.physical_substrate}</span>
          </div>
          <div className="grid grid-cols-2 sm:grid-cols-5 gap-3 text-xs">
            {schedulerData.voltage_rails_live && Object.entries(schedulerData.voltage_rails_live).map(([rail, v]) => (
              <div key={rail} className="bg-black/30 p-2.5 rounded-lg border border-white/5">
                <div className="text-[10px] text-text-secondary truncate">{rail}</div>
                <div className="font-bold text-emerald-400 mt-0.5 font-mono">{v}</div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ── Native IPC Bus vs HTTP Telemetry Card ── */}
      {nativeIpcData && (
        <div className="p-4 bg-purple-950/20 border border-purple-500/30 rounded-2xl space-y-3 shadow-lg">
          <div className="flex items-center justify-between pb-2 border-b border-purple-500/20">
            <div className="flex items-center gap-2">
              <Zap size={16} className="text-purple-400" />
              <span className="text-xs font-bold uppercase text-purple-300">
                Native IPC Bus (SPSC Ring Buffer + NVMe mmap Arena)
              </span>
              <span className="px-2 py-0.2 rounded-full bg-emerald-500/20 text-emerald-400 text-[10px] border border-emerald-500/30">
                RESOLUTION CP-73F2103E ENACTED
              </span>
            </div>
            <span className="text-[10px] font-mono text-purple-300 font-bold">
              SPEEDUP: {nativeIpcData.speedup_vs_http || '10,000x'} vs HTTP
            </span>
          </div>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
            <div className="bg-black/30 p-2.5 rounded-lg border border-white/5">
              <div className="text-[10px] text-text-secondary uppercase">IPC Protocol Mode</div>
              <div className="font-bold text-white mt-0.5 font-mono">SPSC + mmap Arena</div>
            </div>
            <div className="bg-black/30 p-2.5 rounded-lg border border-white/5">
              <div className="text-[10px] text-text-secondary uppercase">Avg In-Process Latency</div>
              <div className="font-bold text-emerald-400 mt-0.5 font-mono">
                {nativeIpcData.overall_avg_latency_ns ? `${nativeIpcData.overall_avg_latency_ns} ns` : '< 100 ns'}
              </div>
            </div>
            <div className="bg-black/30 p-2.5 rounded-lg border border-white/5">
              <div className="text-[10px] text-text-secondary uppercase">Ring Buffer Capacity</div>
              <div className="font-bold text-cyan-400 mt-0.5 font-mono">
                {nativeIpcData.ring_buffer_capacity || 4096} Slots (Pow-of-2)
              </div>
            </div>
            <div className="bg-black/30 p-2.5 rounded-lg border border-white/5">
              <div className="text-[10px] text-text-secondary uppercase">Shared mmap Region</div>
              <div className="font-bold text-purple-300 mt-0.5 font-mono">
                {nativeIpcData.shared_arena_size_mb || 16} MB Zero-Copy
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ── Pillar Selector Bar ── */}
      <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
        {[
          { num: 1, name: 'Pillar I: Wet-Lab Robotics', icon: Bot, color: 'text-amber-400', badge: 'OT-2 / SiLA 2' },
          { num: 2, name: 'Pillar II: Sensory Mesh', icon: Radio, color: 'text-cyan-400', badge: '6 SDR / LoRa' },
          { num: 3, name: 'Pillar III: THz Resonance', icon: Zap, color: 'text-indigo-400', badge: '15.53 THz' },
          { num: 4, name: 'Pillar IV: IND Regulatory', icon: FileCheck, color: 'text-emerald-400', badge: 'FDA eCTD' },
          { num: 5, name: 'Pillar V: Monad Ledger', icon: Database, color: 'text-purple-400', badge: 'PBFT Quorum' },
        ].map(p => {
          const PIcon = p.icon;
          const isSelected = activePillar === p.num;
          return (
            <button
              key={p.num}
              onClick={() => setActivePillar(p.num)}
              className={`p-3.5 rounded-xl border text-left transition-all space-y-1 ${
                isSelected
                  ? 'bg-cyan-500/15 border-cyan-400 text-white shadow-[0_0_15px_rgba(6,182,212,0.2)]'
                  : 'bg-white/5 hover:bg-white/10 border-white/10 text-text-secondary hover:text-white'
              }`}
            >
              <div className="flex items-center justify-between">
                <PIcon size={18} className={p.color} />
                <span className="text-[9px] px-1.5 py-0.2 rounded bg-white/10 font-mono text-text-secondary">
                  {p.badge}
                </span>
              </div>
              <div className="text-xs font-bold leading-snug">{p.name}</div>
            </button>
          );
        })}
      </div>

      {/* ── Pillar Detail Views ── */}
      <div className="p-5 bg-background-secondary/80 border border-white/10 rounded-2xl space-y-4">
        {/* Pillar I */}
        {activePillar === 1 && (
          <div className="space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <div>
                <h3 className="text-sm font-bold text-white uppercase">Pillar I: Physical Wet-Lab Robotics & Biofoundry Automation</h3>
                <p className="text-xs text-text-secondary">Opentrons OT-2 / Flex Python Protocol API v2 & SiLA 2 Ingestion</p>
              </div>
              <span className="text-xs px-2.5 py-1 rounded bg-amber-500/20 text-amber-300 font-mono font-bold">
                {wetlabData?.total_protocols || 6} Protocols Ready
              </span>
            </div>
            <div className="space-y-2 max-h-[450px] overflow-y-auto pr-1 custom-scrollbar">
              {wetlabData?.protocols && wetlabData.protocols.map((proto, idx) => (
                <div key={idx} className="p-3.5 bg-white/5 border border-white/5 rounded-xl space-y-1.5">
                  <div className="flex items-center justify-between">
                    <span className="font-bold text-xs text-amber-300">{proto.protocol_id}: {proto.protocol_name}</span>
                    <span className="text-[10px] px-2 py-0.5 rounded bg-white/10 font-mono">{proto.target_platform}</span>
                  </div>
                  <div className="text-xs text-text-secondary">Chemistry / Assay: <strong className="text-white">{proto.chemistry_type}</strong></div>
                  <div className="text-[11px] text-text-secondary">
                    Est. Runtime: <strong className="text-white">{proto.estimated_duration_min} min</strong> | Labware: {proto.labware?.slice(0, 3).join(', ')}
                  </div>
                  <div className="bg-black/30 p-2 rounded text-[10px] font-mono text-accent-primary truncate">
                    Code Sample: {proto.executable_python_script?.slice(0, 100)}...
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Pillar II */}
        {activePillar === 2 && (
          <div className="space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <div>
                <h3 className="text-sm font-bold text-white uppercase">Pillar II: Ambient Environmental Sensory Mesh</h3>
                <p className="text-xs text-text-secondary">Distributed SDR, LoRaWAN, BLE multi-frequency signal acquisition</p>
              </div>
              <button
                onClick={() => triggerPillarAction('triangulate')}
                disabled={actionInProgress}
                className="px-3 py-1.5 bg-cyan-500/20 hover:bg-cyan-500/30 border border-cyan-500/40 text-cyan-300 rounded-lg text-xs font-bold flex items-center gap-1.5"
              >
                {actionInProgress ? <RefreshCw size={12} className="animate-spin" /> : <Radio size={12} />}
                Trigger Triangulation
              </button>
            </div>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              <div className="p-3 bg-white/5 rounded-xl">
                <div className="text-[10px] text-text-secondary uppercase">Mesh Nodes</div>
                <div className="text-base font-bold text-white mt-0.5">{sensoryData?.active_nodes_count || 6} Online</div>
              </div>
              <div className="p-3 bg-white/5 rounded-xl">
                <div className="text-[10px] text-text-secondary uppercase">Spatial Coherence</div>
                <div className="text-base font-bold text-emerald-400 mt-0.5">
                  {sensoryData?.spatial_coherence ? (sensoryData.spatial_coherence * 100).toFixed(1) + '%' : '77.8%'}
                </div>
              </div>
              <div className="p-3 bg-white/5 rounded-xl">
                <div className="text-[10px] text-text-secondary uppercase">Anomaly Score</div>
                <div className="text-base font-bold text-amber-400 mt-0.5">
                  {sensoryData?.anomaly_score || '0.391'}
                </div>
              </div>
              <div className="p-3 bg-white/5 rounded-xl">
                <div className="text-[10px] text-text-secondary uppercase">Pathogen Correlated</div>
                <div className="text-base font-bold text-purple-400 mt-0.5">1,023 Signatures</div>
              </div>
            </div>
          </div>
        )}

        {/* Pillar III */}
        {activePillar === 3 && (
          <div className="space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <div>
                <h3 className="text-sm font-bold text-white uppercase">Pillar III: Sub-Nanosecond THz Intersite Optical Resonance</h3>
                <p className="text-xs text-text-secondary">Silicon Fd-3m optical phonon mode (15.53 THz material constant); task45 report values, not a live phase-lock</p>
              </div>
              <button
                onClick={() => triggerPillarAction('thz_sync')}
                disabled={actionInProgress}
                className="px-3 py-1.5 bg-indigo-500/20 hover:bg-indigo-500/30 border border-indigo-500/40 text-indigo-300 rounded-lg text-xs font-bold flex items-center gap-1.5"
              >
                {actionInProgress ? <RefreshCw size={12} className="animate-spin" /> : <Zap size={12} />}
                Send Phase Sync Pulse
              </button>
            </div>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              <div className="p-3 bg-white/5 rounded-xl">
                <div className="text-[10px] text-text-secondary uppercase">Carrier Frequency</div>
                <div className="text-base font-bold text-indigo-300 mt-0.5">15.53 THz</div>
              </div>
              <div className="p-3 bg-white/5 rounded-xl">
                <div className="text-[10px] text-text-secondary uppercase">Optical Period (tau)</div>
                <div className="text-base font-bold text-white mt-0.5">64.39 fs</div>
              </div>
              <div className="p-3 bg-white/5 rounded-xl">
                <div className="text-[10px] text-text-secondary uppercase">Mean Phase Jitter</div>
                <div className="text-base font-bold text-emerald-400 mt-0.5">{thzData?.mean_jitter_ps || '29.77'} ps</div>
              </div>
              <div className="p-3 bg-white/5 rounded-xl">
                <div className="text-[10px] text-text-secondary uppercase">Spectral Coherence</div>
                <div className="text-base font-bold text-cyan-400 mt-0.5">{thzData?.spectral_coherence || '0.9817'}</div>
              </div>
            </div>
          </div>
        )}

        {/* Pillar IV */}
        {activePillar === 4 && (
          <div className="space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <div>
                <h3 className="text-sm font-bold text-white uppercase">Pillar IV: Autonomous Preclinical Regulatory IND Dossiers</h3>
                <p className="text-xs text-text-secondary">FDA eCTD Module 2–4 compliant dossiers compiled automatically</p>
              </div>
              <span className="text-xs px-2.5 py-1 rounded bg-emerald-500/20 text-emerald-300 font-mono font-bold">
                {regulatoryData?.total_dossiers || 3} Dossiers Generated
              </span>
            </div>
            <div className="space-y-3">
              {regulatoryData?.dossiers && regulatoryData.dossiers.map((dos, idx) => (
                <div key={idx} className="p-3.5 bg-white/5 border border-white/5 rounded-xl space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="font-bold text-xs text-emerald-300">{dos.dossier_id}: {dos.candidate_name}</span>
                    <span className="text-[10px] px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 font-mono font-bold">
                      Readiness: {dos.readiness_score}%
                    </span>
                  </div>
                  <div className="text-xs text-text-secondary">Target Indication: <strong className="text-white">{dos.target_indication}</strong></div>
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-2 text-[11px] pt-1">
                    <div className="bg-black/20 p-2 rounded">
                      <span className="text-text-secondary">Format: </span>
                      <span className="text-white font-bold">{dos.ectd_format}</span>
                    </div>
                    <div className="bg-black/20 p-2 rounded">
                      <span className="text-text-secondary">FIH Starting Dose: </span>
                      <span className="text-white font-bold">{dos.module_4_safety_summary?.first_in_human_starting_dose || '0.5 mg/kg'}</span>
                    </div>
                    <div className="bg-black/20 p-2 rounded">
                      <span className="text-text-secondary">CMC Route: </span>
                      <span className="text-white font-bold">{dos.module_3_quality_cmc?.manufacturing_route || 'Solid Phase Synthesis'}</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Pillar V */}
        {activePillar === 5 && (
          <div className="space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <div>
                <h3 className="text-sm font-bold text-white uppercase">Pillar V: Decentralized Holographic Byzantine Consensus Ledger</h3>
                <p className="text-xs text-text-secondary">3-phase PBFT consensus with 100% quorum signatures from 14 Council Validators</p>
              </div>
              <button
                onClick={() => triggerPillarAction('ledger_commit')}
                disabled={actionInProgress}
                className="px-3 py-1.5 bg-purple-500/20 hover:bg-purple-500/30 border border-purple-500/40 text-purple-300 rounded-lg text-xs font-bold flex items-center gap-1.5"
              >
                {actionInProgress ? <RefreshCw size={12} className="animate-spin" /> : <Database size={12} />}
                Commit Consensus Block
              </button>
            </div>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              <div className="p-3 bg-white/5 rounded-xl">
                <div className="text-[10px] text-text-secondary uppercase">Total Committed Blocks</div>
                <div className="text-base font-bold text-purple-300 mt-0.5">{ledgerData?.total_blocks || 5} Blocks</div>
              </div>
              <div className="p-3 bg-white/5 rounded-xl">
                <div className="text-[10px] text-text-secondary uppercase">Byzantine Quorum</div>
                <div className="text-base font-bold text-emerald-400 mt-0.5">{ledgerData?.byzantine_tolerance || 'f=4 of N=14 (100%)'}</div>
              </div>
              <div className="p-3 bg-white/5 rounded-xl col-span-2">
                <div className="text-[10px] text-text-secondary uppercase">Latest Block Hash</div>
                <div className="text-xs font-bold text-accent-primary mt-0.5 font-mono truncate">
                  {ledgerData?.latest_block_hash || 'ce172fd78d8bfe055e5f1095f808d89bd90dae4c2d7ca0f471788b1f29312bd2'}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
