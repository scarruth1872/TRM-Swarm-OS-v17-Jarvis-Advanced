import React, { useState, useEffect, useRef } from 'react';
import { Share2, Play } from 'lucide-react';

const BASE_AGENTS = [
  { id: 'Archi', label: 'Archi', cx: 200, cy: 80, role: 'Architect', color: '#3b82f6' },
  { id: 'Devo', label: 'Devo', cx: 320, cy: 120, role: 'Lead Developer', color: '#10b981' },
  { id: 'Seeker', label: 'Seeker', cx: 360, cy: 220, role: 'Researcher', color: '#a855f7' },
  { id: 'Logic', label: 'Logic', cx: 320, cy: 320, role: 'Reasoning Engine', color: '#f59e0b' },
  { id: 'Shield', label: 'Shield', cx: 200, cy: 360, role: 'Security Auditor', color: '#ef4444' },
  { id: 'Flow', label: 'Flow', cx: 80, cy: 320, role: 'DevOps', color: '#ec4899' },
  { id: 'Vision', label: 'Vision', cx: 40, cy: 220, role: 'UI/UX Designer', color: '#14b8a6' },
  { id: 'Verify', label: 'Verify', cx: 80, cy: 120, role: 'QA Engineer', color: '#6366f1' },
  { id: 'Orchestra', label: 'Orchestra', cx: 200, cy: 220, role: 'Swarm Manager', color: '#eab308' },
];

export default function SpatialMesh() {
  const [entanglements, setEntanglements] = useState([]);
  const [nodes, setNodes] = useState(BASE_AGENTS);
  const [activeLinks, setActiveLinks] = useState([]);
  const [isSimulating, setIsSimulating] = useState(false);
  const requestRef = useRef();
  const timeRef = useRef(0);

  // Poll telemetry
  useEffect(() => {
    const fetchTelemetry = async () => {
      try {
        const response = await fetch('http://localhost:8021/swarm/spatial/entanglement');
                const data = await response.json();
                setEntanglements(data.entanglements || []);
        
        // Highlight active connections when new events appear
        if (data.entanglements && data.entanglements.length > 0) {
          const newLinks = [];
          // Firing links between random nodes to represent quantum sync
          for (let i = 0; i < Math.min(3, data.entanglements.length); i++) {
            const sourceIdx = Math.floor(Math.random() * BASE_AGENTS.length);
            let targetIdx = Math.floor(Math.random() * BASE_AGENTS.length);
            while (targetIdx === sourceIdx) {
              targetIdx = Math.floor(Math.random() * BASE_AGENTS.length);
            }
            newLinks.push({
              id: `${BASE_AGENTS[sourceIdx].id}-${BASE_AGENTS[targetIdx].id}-${Date.now()}-${i}`,
              source: BASE_AGENTS[sourceIdx],
              target: BASE_AGENTS[targetIdx],
              color: BASE_AGENTS[sourceIdx].color
            });
          }
          setActiveLinks(prev => [...newLinks, ...prev].slice(0, 8));
        }
      } catch (err) {
        console.error("Failed to fetch entanglement telemetry", err);
      }
    };

    const interval = setInterval(fetchTelemetry, 2000);
    return () => clearInterval(interval);
  }, []);

  // Float nodes slightly in an organic way
  useEffect(() => {
    const animate = (timestamp) => {
      timeRef.current = timestamp * 0.001; // in seconds
      
      setNodes(prevNodes => 
        prevNodes.map((node, idx) => {
          // Unique orbit frequencies and radii per node
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
      await fetch('http://localhost:8021/swarm/spatial/simulate', { method: 'POST' });
    } catch (err) {
      console.error("Failed to trigger simulation", err);
    } finally {
      setTimeout(() => setIsSimulating(false), 1000);
    }
  };

  return (
    <div className="spatial-mesh-container bg-slate-950 border border-cyan-500/20 text-cyan-400 p-6 rounded-lg shadow-2xl flex flex-col h-full min-h-[500px]">
      <div className="flex justify-between items-center mb-6">
        <div className="flex items-center gap-2">
          <Share2 className="text-cyan-400 animate-pulse" size={24} />
          <h2 className="text-2xl font-bold tracking-wider uppercase font-mono text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-blue-500">
            Spatial Mesh projection
          </h2>
        </div>
        <button 
          onClick={handleSimulate}
          disabled={isSimulating}
          className={`flex items-center gap-2 px-4 py-2 rounded font-mono text-xs border uppercase tracking-wider transition-all duration-300 ${
            isSimulating 
              ? 'bg-cyan-500/20 border-cyan-500 text-cyan-300 animate-pulse'
              : 'bg-transparent border-cyan-500/50 hover:bg-cyan-500/10 hover:border-cyan-400 text-cyan-400 shadow-[0_0_15px_rgba(6,182,212,0.1)]'
          }`}
        >
          <Play size={12} className={isSimulating ? "animate-spin" : ""} />
          <span>{isSimulating ? 'Simulating...' : 'Trigger Simulation'}</span>
        </button>
      </div>

      {/* SVG Canvas */}
      <div className="relative flex-1 bg-slate-900/60 border border-slate-800 rounded-lg overflow-hidden min-h-[350px]">
        {/* Grid Background overlay */}
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

          {/* Ambient center glow */}
          <circle cx="200" cy="220" r="180" fill="url(#radial-glow)" />

          {/* Active laser links */}
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
                  strokeWidth="2" 
                  strokeOpacity="0.8"
                  filter="url(#glow)"
                />
                <line 
                  x1={srcNode.x} 
                  y1={srcNode.y} 
                  x2={tgtNode.x} 
                  y2={tgtNode.y} 
                  stroke="#ffffff" 
                  strokeWidth="0.8" 
                  strokeDasharray="4, 4"
                  className="animate-[dash_10s_linear_infinite]"
                />
              </g>
            );
          })}

          {/* Static weak base grid links */}
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
            if (node.id === 'Orchestra') return null;
            const orch = nodes.find(n => n.id === 'Orchestra');
            return (
              <line 
                key={`orch-${node.id}`}
                x1={node.x} 
                y1={node.y} 
                x2={orch.x} 
                y2={orch.y} 
                stroke="#0f172a" 
                strokeWidth="0.8" 
              />
            );
          })}

          {/* Floating Nodes */}
          {nodes.map((node) => (
            <g key={node.id} className="cursor-pointer group">
              {/* Ripple circles for dynamic visual feedback */}
              <circle 
                cx={node.x} 
                cy={node.y} 
                r="18" 
                fill="none" 
                stroke={node.color} 
                strokeWidth="1" 
                strokeOpacity="0.4"
                className="animate-ping"
                style={{ animationDuration: '3s' }}
              />
              
              {/* Outer ring */}
              <circle 
                cx={node.x} 
                cy={node.y} 
                r="10" 
                fill="#020617" 
                stroke={node.color} 
                strokeWidth="2" 
                className="group-hover:scale-125 transition-transform duration-300"
              />

              {/* Glowing inner core */}
              <circle 
                cx={node.x} 
                cy={node.y} 
                r="5" 
                fill="#ffffff" 
                filter="url(#glow)"
              />

              {/* Agent ID Label */}
              <text 
                x={node.x} 
                y={node.y - 16} 
                fill="#e2e8f0" 
                fontSize="10" 
                fontFamily="monospace"
                textAnchor="middle" 
                className="font-bold drop-shadow-[0_2px_2px_rgba(0,0,0,0.8)]"
              >
                {node.label}
              </text>
            </g>
          ))}
        </svg>
      </div>

      {/* Real-time Entanglement log feed below the projection */}
      <div className="mt-4 flex-1 overflow-y-auto max-h-[150px] font-mono text-xs border border-slate-800 rounded bg-slate-950/40 p-3">
        {entanglements.length === 0 ? (
          <p className="text-slate-500 animate-pulse text-center py-6">Awaiting quantum state changes...</p>
        ) : (
          entanglements.map((event, idx) => (
            <div key={idx} className="flex justify-between border-b border-slate-900/60 py-1.5 hover:bg-white/5 px-2 transition-colors">
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
