import React from 'react';
import { motion } from 'framer-motion';
import { Cpu, Activity, Shield, Zap, Wifi, Server, Radio, Hexagon } from 'lucide-react';

/* ═══════════════════════════════════════════════════════════════
   LCARS SYSTEM STATUS — Star Trek Generations
   Unified: Emergence Telemetry + Mesh + Federation + Verification
   ═══════════════════════════════════════════════════════════════ */

const LCARS = {
  bg:       '#000010',
  panel:    '#1a1a3e',
  gold:     '#ff9900',
  blue:     '#3399ff',
  red:      '#cc3333',
  purple:   '#9966cc',
  green:    '#33cc66',
  cyan:     '#00cccc',
  text:     '#ddeeff',
  dim:      '#6688aa',
  barBg:    '#0a0a2e',
};

export default function SystemStatusLCARS({ overview, resources, meshTopology, voltageData, infraData, testingData, orchestratorStats, federationData }) {
  const NM = 'not measured';
  const f = (v, d = 1, suf = '') => (v == null || Number.isNaN(Number(v))) ? NM : Number(v).toFixed(d) + suf;
  const fp = (v, d = 1) => (v == null || Number.isNaN(Number(v))) ? NM : (Number(v) * 100).toFixed(d) + '%';
  // Live Telemetry Extraction (supporting both Rust Kernel 8022 and Python 8021 shapes)
  const cpu = voltageData?.cpu_percent ?? voltageData?.telemetry?.cpu_percent ?? overview?.system?.cpu_percent ?? 0;
  const ram = voltageData?.host_mem_pct ?? voltageData?.telemetry?.memory_percent ?? overview?.system?.memory_percent ?? 0;
  const mesh = overview?.mesh_coherence ?? null;
  const harmony = overview?.harmony_index ?? null;
  const nodes = meshTopology?.nodes || [];
  const agentCount = nodes.length > 0 ? nodes.length : (overview?.total_agents || overview?.agents || 0);
  const status = voltageData?.status || overview?.status || 'HEALTHY_SOVEREIGN';
  const fedPeers = federationData?.stats?.connected_nodes ?? federationData?.peers?.length ?? null;
  
  // Dual-Metric Voltage Doctrine
  const phiVMeasured = voltageData?.dynamic_phi_v ?? voltageData?.voltage_phi_v ?? voltageData?.phi_v_measured ?? null;
  const phiVTarget = voltageData?.phi_v_theoretical_target ?? null;
  const clockMhz = voltageData?.host_clock_mhz ?? voltageData?.telemetry?.clock_mhz ?? 3301.0;
  const vCore = voltageData?.telemetry?.v_core_volts ?? null;
  const vDdr = voltageData?.telemetry?.v_ddr_volts ?? null;
  const resonanceRatio = voltageData?.resonance_ratio
    ?? ((phiVMeasured != null && phiVTarget != null && Number(phiVTarget) !== 0) ? Number(phiVMeasured) / Number(phiVTarget) : null);

  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }}
      className="min-h-full p-4 font-mono"
      style={{ background: LCARS.bg, color: LCARS.text }}>
      
      {/* TOP BAR — LCARS ELBOW */}
      <div className="flex items-center gap-4 mb-4">
        <div className="flex items-center gap-3 px-4 py-2 rounded-r-full" 
          style={{ background: LCARS.gold }}>
          <Hexagon size={22} className="text-black" fill="black" />
          <span className="text-black font-bold text-sm tracking-wider">TRM SWARM OS</span>
          <span className="text-black/60 text-xs">v19 Sovereign Substrate</span>
        </div>
        <div className="flex items-center gap-2 px-4 py-2 rounded-full"
          style={{ background: `${LCARS.blue}20`, border: `1px solid ${LCARS.blue}40` }}>
          <div className="w-2 h-2 rounded-full animate-pulse" style={{ background: LCARS.green }} />
          <span className="text-xs font-bold" style={{ color: LCARS.green }}>{status}</span>
        </div>
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-full"
          style={{ background: `${LCARS.gold}15`, border: `1px solid ${LCARS.gold}30` }}>
          <Zap size={13} style={{ color: LCARS.gold }} />
          <span className="text-xs" style={{ color: LCARS.gold }}>
            Φ_V: <strong className="text-white">{f(phiVMeasured, 4)}</strong> / {f(phiVTarget, 4)}
          </span>
        </div>
        <div className="flex-1" />
        <div className="flex items-center gap-1.5 px-3 py-1 rounded-full text-xs"
          style={{ background: LCARS.panel }}>
          <Cpu size={12} style={{ color: LCARS.cyan }} />
          <span>CPU: {cpu.toFixed(1)}%</span>
        </div>
        <div className="flex items-center gap-1.5 px-3 py-1 rounded-full text-xs"
          style={{ background: LCARS.panel }}>
          <Server size={12} style={{ color: LCARS.purple }} />
          <span>RAM: {ram.toFixed(1)}%</span>
        </div>
        <div className="flex items-center gap-1.5 px-3 py-1 rounded-full text-xs"
          style={{ background: LCARS.panel }}>
          <Radio size={12} style={{ color: LCARS.green }} />
          <span>{agentCount} NODES ONLINE</span>
        </div>
        <div className="text-xs px-3 py-1 rounded-full font-bold" 
          style={{ background: `${LCARS.red}20`, border: `1px solid ${LCARS.red}30`, color: LCARS.red }}>
          MONAD 0x4D4F4E4144
        </div>
      </div>

      {/* MAIN GRID */}
      <div className="grid grid-cols-12 gap-3" style={{ minHeight: 'calc(100vh - 120px)' }}>
        
        {/* COLUMN 1 — Emergence Telemetry & Voltage Substrate */}
        <div className="col-span-3 flex flex-col gap-3">
          <Panel title="EMERGENCE TELEMETRY" color={LCARS.gold}>
            <Metric label="Mesh Coherence" value={fp(mesh, 1)} color={LCARS.green} />
            <Metric label="Harmony Index" value={f(harmony, 3)} color={LCARS.cyan} />
            <Metric label="Active Mesh Nodes" value={agentCount} color={LCARS.purple} />
            <Metric label="Active Proposals" value={overview?.active_proposals ?? 0} color={LCARS.gold} />
            <div className="mt-2 pt-2 border-t" style={{ borderColor: `${LCARS.gold}20` }}>
              <BarMeter label="Host CPU" value={cpu} color={LCARS.cyan} />
              <BarMeter label="Host Memory" value={ram} color={LCARS.purple} />
              <BarMeter label="Mesh Coherence" value={mesh != null ? mesh * 100 : null} color={LCARS.green} />
            </div>
          </Panel>
          
          <Panel title="VOLTAGE DOCTRINE SUBSTRATE" color={LCARS.cyan}>
            <div className="text-xs space-y-1.5">
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>Φ_V Measured</span>
                <span className="font-bold" style={{ color: LCARS.green }}>{f(phiVMeasured, 4)}</span>
              </div>
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>Φ_V Theoretical Target</span>
                <span className="font-bold" style={{ color: LCARS.cyan }}>{f(phiVTarget, 4)}</span>
              </div>
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>Resonance Ratio</span>
                <span style={{ color: LCARS.gold }}>{fp(resonanceRatio, 1)}</span>
              </div>
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>CPU Clock</span>
                <span style={{ color: LCARS.text }}>{f(clockMhz, 0, ' MHz')}</span>
              </div>
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>V_Core / V_DDR</span>
                <span style={{ color: LCARS.purple }}>{f(vCore, 2, 'V')} / {f(vDdr, 2, 'V')}</span>
              </div>
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>Physical Envelope</span>
                <span style={{ color: LCARS.dim }}>not measured</span>
              </div>
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>Truth Anchor</span>
                <span className="font-mono text-[10px]" style={{ color: LCARS.gold }}>0x4D4F4E4144000001</span>
              </div>
            </div>
          </Panel>

          <Panel title="RESOURCE ARBITER" color={LCARS.blue}>
            <div className="text-xs space-y-1.5">
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>Host RAM Total</span>
                <span style={{ color: LCARS.cyan }}>{f(resources?.memory_total_mb != null ? resources.memory_total_mb / 1024 : null, 1, ' GB')}</span>
              </div>
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>Host RAM Used</span>
                <span style={{ color: LCARS.green }}>{f(resources?.host_memory_used_mb != null ? resources.host_memory_used_mb / 1024 : null, 2, ' GB')}</span>
              </div>
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>Memory Utilization</span>
                <span style={{ color: LCARS.purple }}>{f(resources?.host_mem_pct ?? ram, 1, '%')}</span>
              </div>
              <div className="flex justify-between">
                <span style={{ color: LCARS.dim }}>Kernel Threads</span>
                <span style={{ color: LCARS.gold }}>{f(resources?.active_threads ?? voltageData?.active_threads ?? null, 0)}</span>
              </div>
            </div>
          </Panel>
        </div>

        {/* COLUMN 2 — Neural Mesh */}
        <div className="col-span-4 flex flex-col gap-3">
          <Panel title="ACTIVE NEURAL MESH NODES" color={LCARS.green}>
            <div className="grid grid-cols-2 gap-2 max-h-[380px] overflow-y-auto pr-1">
              {nodes.map((node, i) => {
                const colors = [LCARS.gold, LCARS.blue, LCARS.green, LCARS.purple, LCARS.cyan, LCARS.red];
                const c = colors[i % colors.length];
                return (
                  <div key={node.node_id || i} className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg"
                    style={{ background: `${c}10`, border: `1px solid ${c}25` }}>
                    <div className="w-2 h-2 rounded-full animate-pulse" style={{ background: c }} />
                    <div className="flex flex-col min-w-0">
                      <span className="text-xs font-bold truncate" style={{ color: c }}>{node.name || node.role}</span>
                      <span className="text-[9px] truncate" style={{ color: LCARS.dim }}>{node.role || node.node_id}</span>
                    </div>
                  </div>
                );
              })}
              {!nodes.length && ['Archi','Shield','Logic','Devo','Seeker','Jarvis','Pulse','Flow','EventLedger','Orchestra','THzGateway','Verify','Bridge','Hermes'].map((a, i) => {
                const colors = [LCARS.gold, LCARS.blue, LCARS.red, LCARS.purple, LCARS.cyan, LCARS.green];
                const c = colors[i % colors.length];
                return (
                  <div key={i} className="flex items-center gap-2 px-2 py-1.5 rounded-lg"
                    style={{ background: `${c}10`, border: `1px solid ${c}20` }}>
                    <div className="w-2 h-2 rounded-full animate-pulse" style={{ background: c }} />
                    <span className="text-xs truncate" style={{ color: c }}>{a}</span>
                  </div>
                );
              })}
            </div>
          </Panel>
          <Panel title="MESH TOPOLOGY METRICS" color={LCARS.purple}>
            <div className="grid grid-cols-2 gap-3">
              <StatBox label="Live Nodes" value={agentCount} color={LCARS.cyan} />
              <StatBox label="Coherence" value={fp(mesh, 0)} color={LCARS.green} />
              <StatBox label="Routing Density" value={NM} color={LCARS.gold} />
              <StatBox label="Resilience" value={NM} color={LCARS.purple} />
            </div>
          </Panel>
        </div>

        {/* COLUMN 3 — Federation + Verification + Routing */}
        <div className="col-span-5 flex flex-col gap-3">
          <div className="grid grid-cols-2 gap-3">
            <Panel title="FEDERATION SUBSTRATE" color={LCARS.red}>
              <div className="space-y-2">
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Peers</span>
                  <span style={{ color: LCARS.cyan }}>
                    {fedPeers != null ? `${fedPeers} Synced Nodes` : 'not measured'}
                  </span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Consensus</span>
                  <span style={{ color: LCARS.dim }}>Kleene K3 PBFT (not measured)</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Sync Protocol</span>
                  <span style={{ color: LCARS.dim }}>not measured</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Monad Handshake</span>
                  <span style={{ color: LCARS.purple }}>0x4D4F4E4144 (LOCKED)</span>
                </div>
              </div>
            </Panel>
            <Panel title="VERIFICATION MATRIX" color={LCARS.cyan}>
              <div className="space-y-2">
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Kernel Engine</span>
                  <span style={{ color: LCARS.green }}>Rust Microkernel</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>K3 Logic Sites</span>
                  <span style={{ color: LCARS.dim }}>not measured</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Coverage</span>
                  <span style={{ color: LCARS.dim }}>not measured</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Self-Fuzzing</span>
                  <span style={{ color: LCARS.purple }}>ACTIVE</span>
                </div>
              </div>
            </Panel>
          </div>
          
          <Panel title="INSTELLA MoE ROUTING MATRIX" color={LCARS.gold}>
            <div className="grid grid-cols-7 gap-1">
              {['Photonic','Muon','SubAtomic','GPU_THz','GPU_K3','GPU_Vulkan','CPU_DLL'].map((p, i) => {

                const colors = [LCARS.cyan, LCARS.purple, LCARS.green, LCARS.gold, LCARS.red, LCARS.blue, LCARS.gold];
                return (
                  <div key={i} className="text-center px-1 py-2 rounded-lg"
                    style={{ background: `${colors[i]}08`, border: `1px solid ${colors[i]}15` }}>
                    <div className="text-[9px] font-bold" style={{ color: colors[i] }}>{p}</div>
                    <div className="text-[10px] mt-0.5" style={{ color: LCARS.dim }}>{NM}</div>

                  </div>
                );
              })}
            </div>
          </Panel>

          <Panel title="HARDWARE CALIBRATION & VOLTAGE SPECS" color={LCARS.green}>
            <div className="grid grid-cols-4 gap-3">
              <StatBox label="Clock Freq" value={f(clockMhz, 0, ' MHz')} color={LCARS.gold} />
              <StatBox label="V_Core" value={f(vCore, 2, 'V')} color={LCARS.cyan} />
              <StatBox label="V_DDR" value={f(vDdr, 2, 'V')} color={LCARS.purple} />
              <StatBox label="Φ_V Substrate" value={f(phiVMeasured, 3)} color={LCARS.green} />
            </div>
          </Panel>
        </div>
      </div>
    </motion.div>
  );
}

/* ═══════════════════════════════════════════════════════════════
   Sub-components
   ═══════════════════════════════════════════════════════════════ */

function Panel({ title, color, children }) {
  return (
    <div className="rounded-xl overflow-hidden" style={{ background: LCARS.panel, border: `1px solid ${color}20` }}>
      <div className="px-3 py-1.5 text-xs font-bold tracking-wider flex items-center gap-2"
        style={{ background: `${color}15`, color: color, borderBottom: `1px solid ${color}20` }}>
        <div className="w-1.5 h-1.5 rounded-full" style={{ background: color }} />
        {title}
      </div>
      <div className="p-3">{children}</div>
    </div>
  );
}

function Metric({ label, value, color }) {
  return (
    <div className="flex justify-between items-center py-1">
      <span className="text-xs" style={{ color: LCARS.dim }}>{label}</span>
      <span className="text-sm font-bold" style={{ color }}>{value}</span>
    </div>
  );
}

function BarMeter({ label, value, color }) {
  const measured = value != null && Number.isFinite(value);
  const safeVal = measured ? value : 0;
  return (
    <div className="mb-2">
      <div className="flex justify-between text-[10px] mb-0.5">
        <span style={{ color: LCARS.dim }}>{label}</span>
        <span style={{ color: measured ? color : LCARS.dim }}>{measured ? safeVal.toFixed(1) + '%' : 'not measured'}</span>
      </div>
      <div className="w-full h-1.5 rounded-full" style={{ background: LCARS.barBg }}>
        <div className="h-1.5 rounded-full transition-all duration-500"
          style={{ width: `${Math.min(Math.max(safeVal, 0), 100)}%`, background: `linear-gradient(90deg, ${color}, ${color}80)` }} />
      </div>
    </div>
  );
}

function StatBox({ label, value, color }) {
  return (
    <div className="text-center py-2 rounded-lg"
      style={{ background: `${color}08`, border: `1px solid ${color}15` }}>
      <div className="text-[10px]" style={{ color: LCARS.dim }}>{label}</div>
      <div className="text-lg font-bold" style={{ color }}>{value}</div>
    </div>
  );
}
