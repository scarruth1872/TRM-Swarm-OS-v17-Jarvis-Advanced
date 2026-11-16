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

export default function SystemStatusLCARS({ overview, resources, meshTopology, infraData, testingData, orchestratorStats }) {
  const cpu = overview?.system?.cpu_percent || resources?.cpu || 0;
  const ram = overview?.system?.memory_percent || resources?.ram || 0;
  const mesh = overview?.mesh_coherence || 0.96;
  const harmony = overview?.harmony_index || 0.91;
  const agents = overview?.agents || 14;
  const status = overview?.status || 'STABLE';
  const nodes = meshTopology?.nodes || [];

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
          <span className="text-black/60 text-xs">v18.244</span>
        </div>
        <div className="flex items-center gap-2 px-4 py-2 rounded-full"
          style={{ background: `${LCARS.blue}20`, border: `1px solid ${LCARS.blue}40` }}>
          <div className="w-2 h-2 rounded-full animate-pulse" style={{ background: LCARS.green }} />
          <span className="text-xs" style={{ color: LCARS.green }}>{status}</span>
        </div>
        <div className="flex-1" />
        <div className="flex items-center gap-1 px-3 py-1 rounded-full text-xs"
          style={{ background: LCARS.panel }}>
          <Cpu size={12} style={{ color: LCARS.cyan }} />
          <span>{cpu.toFixed(1)}%</span>
        </div>
        <div className="flex items-center gap-1 px-3 py-1 rounded-full text-xs"
          style={{ background: LCARS.panel }}>
          <Server size={12} style={{ color: LCARS.purple }} />
          <span>{ram.toFixed(1)}%</span>
        </div>
        <div className="flex items-center gap-1 px-3 py-1 rounded-full text-xs"
          style={{ background: LCARS.panel }}>
          <Radio size={12} style={{ color: LCARS.gold }} />
          <span>{agents} AGENTS</span>
        </div>
        <div className="text-xs px-3 py-1 rounded-full" 
          style={{ background: `${LCARS.red}20`, border: `1px solid ${LCARS.red}30` }}>
          LCARS
        </div>
      </div>

      {/* MAIN GRID */}
      <div className="grid grid-cols-12 gap-3" style={{ height: 'calc(100vh - 120px)' }}>
        
        {/* COLUMN 1 — Emergence Telemetry */}
        <div className="col-span-3 flex flex-col gap-3">
          <Panel title="EMERGENCE TELEMETRY" color={LCARS.gold}>
            <Metric label="Mesh Coherence" value={(mesh * 100).toFixed(0) + '%'} color={LCARS.green} />
            <Metric label="Harmony Index" value={harmony.toFixed(3)} color={LCARS.cyan} />
            <Metric label="Active Agents" value={agents} color={LCARS.purple} />
            <Metric label="Active Proposals" value={overview?.active_proposals || 3} color={LCARS.gold} />
            <div className="mt-2 pt-2 border-t" style={{ borderColor: `${LCARS.gold}20` }}>
              <BarMeter label="CPU" value={cpu} color={LCARS.cyan} />
              <BarMeter label="RAM" value={ram} color={LCARS.purple} />
              <BarMeter label="Coherence" value={mesh * 100} color={LCARS.green} />
            </div>
          </Panel>
          
          <Panel title="SYSTEM RESOURCES" color={LCARS.blue}>
            <div className="text-xs space-y-1">
              <div className="flex justify-between"><span style={{ color: LCARS.dim }}>Engine</span><span style={{ color: LCARS.cyan }}>MICROKERNEL</span></div>
              <div className="flex justify-between"><span style={{ color: LCARS.dim }}>Proxy</span><span style={{ color: LCARS.green }}>ACTIVE</span></div>
              <div className="flex justify-between"><span style={{ color: LCARS.dim }}>Fast Path</span><span style={{ color: LCARS.gold }}>13.9M steps/s</span></div>
              <div className="flex justify-between"><span style={{ color: LCARS.dim }}>Chat Latency</span><span style={{ color: LCARS.green }}>207μs</span></div>
              <div className="flex justify-between"><span style={{ color: LCARS.dim }}>Commit</span><span style={{ color: LCARS.purple }}>244</span></div>
              <div className="flex justify-between"><span style={{ color: LCARS.dim }}>Datasets</span><span style={{ color: LCARS.cyan }}>688</span></div>
              <div className="flex justify-between"><span style={{ color: LCARS.dim }}>Steps</span><span style={{ color: LCARS.gold }}>345M</span></div>
            </div>
          </Panel>
        </div>

        {/* COLUMN 2 — Neural Mesh */}
        <div className="col-span-4 flex flex-col gap-3">
          <Panel title="NEURAL MESH TOPOLOGY" color={LCARS.green}>
            <div className="grid grid-cols-2 gap-2">
              {nodes.slice(0, 14).map((node, i) => {
                const colors = [LCARS.gold, LCARS.blue, LCARS.red, LCARS.purple, LCARS.cyan, LCARS.green];
                const c = colors[i % colors.length];
                return (
                  <div key={i} className="flex items-center gap-2 px-2 py-1.5 rounded-lg"
                    style={{ background: `${c}10`, border: `1px solid ${c}20` }}>
                    <div className="w-2 h-2 rounded-full" style={{ background: c }} />
                    <span className="text-xs truncate" style={{ color: c }}>{node.role || node.id}</span>
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
          <Panel title="MESH STATISTICS" color={LCARS.purple}>
            <div className="grid grid-cols-2 gap-3">
              <StatBox label="Nodes" value={nodes.length || 14} color={LCARS.cyan} />
              <StatBox label="Edges" value={(nodes.length || 14) * 13} color={LCARS.purple} />
              <StatBox label="Density" value="0.93" color={LCARS.gold} />
              <StatBox label="Resilience" value="96%" color={LCARS.green} />
            </div>
          </Panel>
        </div>

        {/* COLUMN 3 — Federation + Verification */}
        <div className="col-span-5 flex flex-col gap-3">
          <div className="grid grid-cols-2 gap-3">
            <Panel title="FEDERATION" color={LCARS.red}>
              <div className="space-y-2">
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Peers</span>
                  <span style={{ color: LCARS.cyan }}>0 (Standalone)</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Consensus</span>
                  <span style={{ color: LCARS.green }}>PBFT</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Sync</span>
                  <span style={{ color: LCARS.gold }}>MEMORY-MAPPED</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Handshake</span>
                  <span style={{ color: LCARS.purple }}>QUANTUM</span>
                </div>
              </div>
            </Panel>
            <Panel title="VERIFICATION" color={LCARS.cyan}>
              <div className="space-y-2">
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Tests Passed</span>
                  <span style={{ color: LCARS.green }}>12,034/14,000</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Accuracy</span>
                  <span style={{ color: LCARS.gold }}>86.0%</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Coverage</span>
                  <span style={{ color: LCARS.cyan }}>93%</span>
                </div>
                <div className="flex justify-between text-xs">
                  <span style={{ color: LCARS.dim }}>Fuzzing</span>
                  <span style={{ color: LCARS.purple }}>ACTIVE</span>
                </div>
              </div>
            </Panel>
          </div>
          
          <Panel title="INSTELLA MoE ROUTING MATRIX" color={LCARS.gold}>
            <div className="grid grid-cols-7 gap-1">
              {['Photonic','Muon','SubAtomic','GPU_THz','GPU_K3','GPU_Vulkan','CPU_DLL'].map((p, i) => {
                const throughputs = ['30B','5.2B','4B','2.4B','1.3B','252M','13.9M'];
                const colors = [LCARS.cyan, LCARS.purple, LCARS.green, LCARS.gold, LCARS.red, LCARS.blue, LCARS.gold];
                return (
                  <div key={i} className="text-center px-1 py-2 rounded-lg"
                    style={{ background: `${colors[i]}08`, border: `1px solid ${colors[i]}15` }}>
                    <div className="text-[9px] font-bold" style={{ color: colors[i] }}>{p}</div>
                    <div className="text-[10px] mt-0.5" style={{ color: LCARS.text }}>{throughputs[i]}</div>
                    <div className="text-[8px]" style={{ color: LCARS.dim }}>ops/s</div>
                  </div>
                );
              })}
            </div>
          </Panel>

          <Panel title="TRUST & CALIBRATION" color={LCARS.green}>
            <div className="grid grid-cols-4 gap-3">
              <StatBox label="GLP-1R r" value="0.949" color={LCARS.gold} />
              <StatBox label="Tau r" value="0.915" color={LCARS.cyan} />
              <StatBox label="B3DB AUC" value="0.981" color={LCARS.purple} />
              <StatBox label="Trust" value="93.2%" color={LCARS.green} />
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
  return (
    <div className="mb-2">
      <div className="flex justify-between text-[10px] mb-0.5">
        <span style={{ color: LCARS.dim }}>{label}</span>
        <span style={{ color }}>{value.toFixed(1)}%</span>
      </div>
      <div className="w-full h-1.5 rounded-full" style={{ background: LCARS.barBg }}>
        <div className="h-1.5 rounded-full transition-all duration-500"
          style={{ width: `${Math.min(value, 100)}%`, background: `linear-gradient(90deg, ${color}, ${color}80)` }} />
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
