import React, { useState, useEffect, useRef } from 'react';
import { Share2, Play, Send, Zap, Shield, Cpu, Activity } from 'lucide-react';

const BASE_AGENTS = [
  { id: 'Shield', label: 'Shield', cx: 200, cy: 80, role: 'Security Auditor', color: '#ef4444' },
  { id: 'Logic', label: 'Logic', cx: 350, cy: 80, role: 'Reasoning Engine', color: '#f59e0b' },
  { id: 'Devo', label: 'Devo', cx: 500, cy: 120, role: 'Lead Developer', color: '#10b981' },
  { id: 'Seeker', label: 'Seeker', cx: 550, cy: 220, role: 'Researcher', color: '#a855f7' },
  { id: 'Verify', label: 'Verify', cx: 500, cy: 320, role: 'QA Engineer', color: '#6366f1' },
  { id: 'Archi', label: 'Archi', cx: 350, cy: 360, role: 'Architect', color: '#3b82f6' },
  { id: 'Flow', label: 'Flow', cx: 200, cy: 360, role: 'API Integration', color: '#ec4899' },
  { id: 'Bridge', label: 'Bridge', cx: 50, cy: 320, role: 'Data Fetcher', color: '#14b8a6' },
  { id: 'Orchestra', label: 'Orchestra', cx: 50, cy: 120, role: 'Swarm Manager', color: '#eab308' },
  { id: 'EventLedger', label: 'Ledger', cx: 100, cy: 220, role: 'Observability', color: '#06b6d4' },
  { id: 'LogicMaster', label: 'LogicM', cx: 300, cy: 220, role: 'SAT/SMT Solver', color: '#d946ef' },
  { id: 'Pulse', label: 'Pulse', cx: 500, cy: 220, role: 'Health Monitor', color: '#84cc16' },
  { id: 'THzGateway', label: 'THz', cx: 200, cy: 180, role: 'Wave Processor', color: '#f97316' },
  { id: 'Jarvis', label: 'Jarvis', cx: 350, cy: 140, role: 'Cognitive Memory', color: '#22d3ee' },
  { id: 'Hermes', label: 'Hermes', cx: 200, cy: 280, role: 'Orchestrator', color: '#c084fc' },
];

export default function SpatialMesh() {
  const [entanglements, setEntanglements] = useState([]);
  const [nodes, setNodes] = useState(BASE_AGENTS);
  const [activeLinks, setActiveLinks] = useState([]);
  const [isSimulating, setIsSimulating] = useState(false);
  const [selectedNode, setSelectedNode] = useState(null);
  const [routeTaskInput, setRouteTaskInput] = useState('');
  const [isRouting, setIsRouting] = useState(false);
  const [routingResult, setRoutingResult] = useState(null);
  const [meshTopology, setMeshTopology] = useState(null);
  const requestRef = useRef();
  const timeRef = useRef(0);

  // Poll live telemetry and topology
  useEffect(() => {
    const fetchTelemetry = async () => {
      try {
        const response = await fetch('/swarm/spatial/entanglement');
        if (response.ok) {
          const data = await response.json();
          setEntanglements(data.entanglements || []);
          
          // Connect nodes mentioned in entanglements
          if (data.entanglements && data.entanglements.length > 0) {
            const newLinks = [];
            data.entanglements.slice(0, 4).forEach((event, i) => {
              if (event.key && event.key.includes('_TO_')) {
                const parts = event.key.split('_TO_');
                const src = BASE_AGENTS.find(n => n.id.toLowerCase() === parts[0].toLowerCase());
                const tgt = BASE_AGENTS.find(n => n.id.toLowerCase() === parts[1].toLowerCase());
                if (src && tgt) {
                  newLinks.push({
                    id: `${src.id}-${tgt.id}-${Date.now()}-${i}`,
                    source: src,
                    target: tgt,
                    color: src.color
                  });
                }
              }
            });
            if (newLinks.length > 0) {
              setActiveLinks(prev => [...newLinks, ...prev].slice(0, 8));
            }
          }
        }
      } catch (err) {
        console.warn("Telemetry fetch error", err);
      }
    };

    const fetchTopology = async () => {
      try {
        const res = await fetch('/mesh/topology');
        if (res.ok) {
          const topo = await res.json();
          setMeshTopology(topo);
        }
      } catch (e) {
        console.warn("Topology fetch error", e);
      }
    };

    fetchTelemetry();
    fetchTopology();
    const interval = setInterval(fetchTelemetry, 2500);
    return () => clearInterval(interval);
  }, []);

  // Float nodes slightly in an organic way
  useEffect(() => {
    const animate = (timestamp) => {
      timeRef.current = timestamp * 0.001; // in seconds
      
      setNodes(prevNodes => 
        prevNodes.map((node, idx) => {
          const freqX = 1 + idx * 0.15;
          const freqY = 1.2 - idx * 0.1;
          const driftX = Math.sin(timeRef.current * freqX) * 8;
          const driftY = Math.cos(timeRef.current * freqY) * 8;
          return {
            ...node,
            x: node.cx + driftX,
            y: node.cy + driftY
          };
        })
      );
      
      requestRef.current = requestAnimationFrame(animate);
    };
    
    requestRef.current = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(requestRef.current);
  }, []);

  const handleSimulate = async () => {
    setIsSimulating(true);
    try {
      const res = await fetch('/swarm/spatial/simulate', { 
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({})
      });
      if (res.ok) {
        const entRes = await fetch('/swarm/spatial/entanglement');
        if (entRes.ok) {
          const entData = await entRes.json();
          setEntanglements(entData.entanglements || []);
        }
      }
    } catch (err) {
      console.error("Failed to trigger simulation", err);
    } finally {
      setTimeout(() => setIsSimulating(false), 800);
    }
  };

  const handleRouteTask = async (e) => {
    e.preventDefault();
    if (!routeTaskInput.trim()) return;
    setIsRouting(true);
    try {
      const res = await fetch('/mesh/route', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ task: routeTaskInput, payload: {} })
      });
      if (res.ok) {
        const data = await res.json();
        setRoutingResult(data);
        const routedNode = BASE_AGENTS.find(n => n.id.toLowerCase() === data.routed_to?.toLowerCase());
        if (routedNode) {
          setSelectedNode(routedNode);
          const orch = BASE_AGENTS.find(n => n.id === 'Orchestra') || BASE_AGENTS[0];
          setActiveLinks(prev => [
            {
              id: `route-${routedNode.id}-${Date.now()}`,
              source: orch,
              target: routedNode,
              color: routedNode.color
            },
            ...prev
          ].slice(0, 8));
        }
      }
    } catch (err) {
      console.error("Failed to route task through spatial mesh", err);
    } finally {
      setIsRouting(false);
    }
  };

  return (
    <div className="spatial-mesh-container bg-slate-950 border border-cyan-500/20 text-cyan-400 p-6 rounded-lg shadow-2xl flex flex-col h-full min-h-[550px]">
      <div className="flex justify-between items-center mb-4 flex-wrap gap-3">
        <div className="flex items-center gap-2">
          <Share2 className="text-cyan-400 animate-pulse" size={24} />
          <div>
            <h2 className="text-xl font-bold tracking-wider uppercase font-mono text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-blue-500">
              Spatial Mesh & Quantum Routing
            </h2>
            <div className="flex items-center gap-3 text-[11px] font-mono text-slate-400">
              <span>Nodes: <strong className="text-cyan-300">{nodes.length}</strong></span>
              <span>•</span>
              <span>Mesh Coherence: <strong className="text-emerald-400">{meshTopology?.mesh_coherence || 0.98}</strong></span>
              <span>•</span>
              <span>Substrate: <strong className="text-purple-400">QER Shared-Memory</strong></span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button 
            onClick={handleSimulate}
            disabled={isSimulating}
            className={`flex items-center gap-2 px-3 py-1.5 rounded font-mono text-xs border uppercase tracking-wider transition-all duration-300 ${
              isSimulating 
                ? 'bg-cyan-500/20 border-cyan-500 text-cyan-300 animate-pulse'
                : 'bg-transparent border-cyan-500/50 hover:bg-cyan-500/10 hover:border-cyan-400 text-cyan-400 shadow-[0_0_15px_rgba(6,182,212,0.1)]'
            }`}
          >
            <Play size={12} className={isSimulating ? "animate-spin" : ""} />
            <span>{isSimulating ? 'Pulsing Mesh...' : 'Trigger QER Pulse'}</span>
          </button>
        </div>
      </div>

      <form onSubmit={handleRouteTask} className="mb-4 flex gap-2">
        <input 
          type="text"
          value={routeTaskInput}
          onChange={(e) => setRouteTaskInput(e.target.value)}
          placeholder="Route task across spatial mesh (e.g. 'audit memory safety', 'prove K3 invariant', 'synthesize API')..."
          className="flex-1 bg-slate-900 border border-slate-700 rounded px-3 py-1.5 text-xs text-slate-200 placeholder:text-slate-500 focus:outline-none focus:border-cyan-400 font-mono"
        />
        <button
          type="submit"
          disabled={isRouting || !routeTaskInput.trim()}
          className="px-4 py-1.5 bg-cyan-600/30 hover:bg-cyan-600/50 border border-cyan-500 text-cyan-300 rounded font-mono text-xs flex items-center gap-1.5 transition-all"
        >
          <Send size={12} className={isRouting ? "animate-pulse" : ""} />
          <span>{isRouting ? 'Routing...' : 'Route Task'}</span>
        </button>
      </form>

      {routingResult && (
        <div className="mb-3 p-2.5 bg-cyan-950/40 border border-cyan-500/40 rounded text-xs font-mono flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Zap size={14} className="text-yellow-400" />
            <span>Task Routed To: <strong className="text-cyan-300 text-sm">{routingResult.routed_to}</strong></span>
            <span className="text-slate-500">| Score: {routingResult.routing_score}</span>
          </div>
          <span className="text-[10px] text-slate-400">Latency: {routingResult.mesh_latency_us} µs</span>
        </div>
      )}

      <div className="relative flex-1 bg-slate-900/60 border border-slate-800 rounded-lg overflow-hidden min-h-[340px]">
        <div className="absolute inset-0 bg-[linear-gradient(to_right,#0f172a_1px,transparent_1px),linear-gradient(to_bottom,#0f172a_1px,transparent_1px)] bg-[size:24px_24px] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_50%,#000_70%,transparent_100%)] opacity-60"></div>
        
        <svg className="w-full h-full absolute inset-0 select-none">
          <defs>
            <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
              <feGaussianBlur stdDeviation="6" result="blur" />
              <feComposite in="SourceGraphic" in2="blur" operator="over" />
            </filter>
            <radialGradient id="radial-glow" cx="50%" cy="50%" r="50%">
              <stop offset="0%" stopColor="#22d3ee" stopOpacity="0.15" />
              <stop offset="100%" stopColor="#22d3ee" stopOpacity="0" />
            </radialGradient>
          </defs>

          <circle cx="300" cy="200" r="220" fill="url(#radial-glow)" />

          {/* ── Metatron Sacred Geometry Lattice (Harmonic Resonance Layer) ── */}
          <g opacity="0.35" className="pointer-events-none">
            {/* Concentric Golden Ratio Harmonic Circles */}
            <circle cx="300" cy="200" r="70" fill="none" stroke="#38bdf8" strokeWidth="1" strokeDasharray="3, 3" />
            <circle cx="300" cy="200" r="140" fill="none" stroke="#818cf8" strokeWidth="1" strokeDasharray="4, 4" />
            <circle cx="300" cy="200" r="210" fill="none" stroke="#c084fc" strokeWidth="1.5" strokeOpacity="0.6" />

            {/* Metatron Cube Hexagonal Nodes & Inter-connecting Axis Rays */}
            {[0, 60, 120, 180, 240, 300].map((angle, i) => {
              const rad = (angle * Math.PI) / 180;
              const x1 = 300 + 140 * Math.cos(rad);
              const y1 = 200 + 140 * Math.sin(rad);
              const oppRad = ((angle + 180) * Math.PI) / 180;
              const x2 = 300 + 140 * Math.cos(oppRad);
              const y2 = 200 + 140 * Math.sin(oppRad);
              return (
                <g key={`metatron-ray-${i}`}>
                  <line x1={x1} y1={y1} x2={x2} y2={y2} stroke="#38bdf8" strokeWidth="0.75" strokeOpacity="0.4" />
                  <circle cx={x1} cy={y1} r="3" fill="#38bdf8" fillOpacity="0.5" />
                </g>
              );
            })}

            {/* Inner Hexagram Invariant Lattice */}
            <polygon points="300,90 395,255 205,255" fill="none" stroke="#22d3ee" strokeWidth="1" strokeOpacity="0.4" />
            <polygon points="300,310 395,145 205,145" fill="none" stroke="#a855f7" strokeWidth="1" strokeOpacity="0.4" />
          </g>

          {activeLinks.map(link => {
            const srcNode = nodes.find(n => n.id === link.source.id);
            const tgtNode = nodes.find(n => n.id === link.target.id);
            if (!srcNode || !tgtNode) return null;
            return (
              <g key={link.id}>
                <line 
                  x1={srcNode.x} 
                  y1={srcNode.y} 
                  x2={tgtNode.x} 
                  y2={tgtNode.y} 
                  stroke={link.color} 
                  strokeWidth="2.5" 
                  strokeOpacity="0.85"
                  filter="url(#glow)"
                />
                <line 
                  x1={srcNode.x} 
                  y1={srcNode.y} 
                  x2={tgtNode.x} 
                  y2={tgtNode.y} 
                  stroke="#ffffff" 
                  strokeWidth="1" 
                  strokeDasharray="4, 4"
                  className="animate-[dash_10s_linear_infinite]"
                />
              </g>
            );
          })}

          {nodes.map((node, i) => {
            const nextNode = nodes[(i + 1) % nodes.length];
            return (
              <line 
                key={`base-${i}`}
                x1={node.x} 
                y1={node.y} 
                x2={nextNode.x} 
                y2={nextNode.y} 
                stroke="#1e293b" 
                strokeWidth="1" 
                strokeDasharray="2, 2"
              />
            );
          })}

          {nodes.map((node) => {
            const isSelected = selectedNode?.id === node.id;
            return (
              <g 
                key={node.id} 
                className="cursor-pointer group"
                onClick={() => setSelectedNode(isSelected ? null : node)}
              >
                <circle 
                  cx={node.x} 
                  cy={node.y} 
                  r={isSelected ? "26" : "18"} 
                  fill="none" 
                  stroke={node.color} 
                  strokeWidth={isSelected ? "2" : "1"} 
                  strokeOpacity={isSelected ? "0.8" : "0.4"}
                  className="animate-ping"
                  style={{ animationDuration: isSelected ? '1.5s' : '3s' }}
                />
                
                <circle 
                  cx={node.x} 
                  cy={node.y} 
                  r={isSelected ? "14" : "10"} 
                  fill="#020617" 
                  stroke={node.color} 
                  strokeWidth={isSelected ? "3" : "2"} 
                  className="group-hover:scale-125 transition-transform duration-300"
                />

                <circle 
                  cx={node.x} 
                  cy={node.y} 
                  r={isSelected ? "7" : "5"} 
                  fill={isSelected ? node.color : "#ffffff"} 
                  filter="url(#glow)"
                />

                <text 
                  x={node.x} 
                  y={node.y - 18} 
                  fill={isSelected ? "#38bdf8" : "#e2e8f0"} 
                  fontSize="11" 
                  fontFamily="monospace"
                  textAnchor="middle" 
                  className="font-bold drop-shadow-[0_2px_2px_rgba(0,0,0,0.8)]"
                >
                  {node.label}
                </text>
              </g>
            );
          })}
        </svg>
      </div>

      {selectedNode && (
        <div className="mt-3 p-3 bg-slate-900 border border-slate-700 rounded-lg flex items-center justify-between font-mono text-xs">
          <div className="flex items-center gap-3">
            <div className="w-3 h-3 rounded-full" style={{ backgroundColor: selectedNode.color }} />
            <div>
              <span className="font-bold text-slate-100 text-sm">{selectedNode.label}</span>
              <span className="text-slate-400 ml-2">({selectedNode.role})</span>
            </div>
            <span className="text-emerald-400 bg-emerald-950/60 px-2 py-0.5 rounded border border-emerald-800 text-[10px]">
              QER ONLINE
            </span>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => {
                setRouteTaskInput(`Task assigned to ${selectedNode.label}`);
              }}
              className="px-2.5 py-1 bg-cyan-950 hover:bg-cyan-900 border border-cyan-600/40 text-cyan-300 rounded text-[11px]"
            >
              Route Target
            </button>
            <button
              onClick={() => setSelectedNode(null)}
              className="text-slate-500 hover:text-slate-300 text-xs px-1"
            >
              ✕
            </button>
          </div>
        </div>
      )}

      <div className="mt-4 overflow-y-auto max-h-[140px] font-mono text-xs border border-slate-800 rounded bg-slate-950/40 p-2.5">
        <div className="text-[10px] text-slate-500 uppercase tracking-wider mb-1.5 flex justify-between px-1">
          <span>Live Quantum Entanglement Stream</span>
          <span>Buffer: {entanglements.length} events</span>
        </div>
        {entanglements.length === 0 ? (
          <p className="text-slate-500 animate-pulse text-center py-4">Awaiting quantum state changes...</p>
        ) : (
          entanglements.map((event, idx) => (
            <div key={idx} className="flex justify-between border-b border-slate-900/60 py-1 hover:bg-white/5 px-2 transition-colors">
              <span className="text-cyan-600">[{new Date(event.timestamp * 1000).toLocaleTimeString()}]</span>
              <span className="text-cyan-300 font-semibold">{event.state_id}</span>
              <span className="text-pink-400 font-bold">{event.key}</span>
              <span className="text-slate-400 truncate max-w-xs">{event.value}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
