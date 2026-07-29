import React, { useState, useEffect, useRef } from 'react';
import './MindMap.css';

export default function MindMap() {
  const [activeTab, setActiveTab] = useState('briefing');
  const [selectedSource, setSelectedSource] = useState(null);
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [isPlaying, setIsPlaying] = useState(false);
  const [playbackTime, setPlaybackTime] = useState(0);
  const [activeDialogueLine, setActiveDialogueLine] = useState(0);
  const [quizAnswer, setQuizAnswer] = useState(null);
  const [expandedFaq, setExpandedFaq] = useState(null);
  const [voices, setVoices] = useState([]);

  const transcriptEndRef = useRef(null);
  const timerRef = useRef(null);
  const utteranceRef = useRef(null);
  const activeLineRef = useRef(0);
  const isPlayingRef = useRef(false);

  // Simulated sources list
  const sources = [
    {
      id: 'app_v2',
      name: 'app_v2.py',
      category: 'core',
      description: 'Swarm OS Gateway & API Gateway',
      details: 'Initializes the active FastAPI server on port 8021. Handles client chat communication with agents, routes requests based on agent persona preferences, and syncs telemetry data.',
      implements: ['FastAPI Lifespan', 'chat_with_agent route', 'Telemetry endpoint', 'Deferred Background Activation'],
      connections: ['base_agent.py', 'agent_mesh.py', 'telemetry.py']
    },
    {
      id: 'base_agent',
      name: 'base_agent.py',
      category: 'core',
      description: 'Cognitive agent lifecycle base definition',
      details: 'Defines the fundamental BaseAgent class. Implements cognitive preferences, local model fallback chains, VRAM resource management validation, and autonomous self-healing remediation sweeps.',
      implements: ['process_task()', '_autonomous_remediate()', 'Nodal Logging', 'Verification Audit Trigger'],
      connections: ['cognitive_stack.py', 'artifact_pipeline.py', 'expert_registry.py']
    },
    {
      id: 'cognitive_stack',
      name: 'cognitive_stack.py',
      category: 'stack',
      description: 'Distributed Language & Symbolic stack',
      details: 'Orchestrates language reasoning using local Ollama model routers. Splits execution into an Executive layer (e.g. Gemma3 4B) and Samsung TRM 7M integration for mathematical and symbolic logic audits.',
      implements: ['Executive Layer', 'Samsung TRM 7M Core', 'Attempt Sampler (Superposition)', 'Model Routing'],
      connections: ['base_agent.py', 'trm_brain.py', 'llm_router.py']
    },
    {
      id: 'agent_mesh',
      name: 'agent_mesh.py',
      category: 'mesh',
      description: 'P2P Federation Network Topology',
      details: 'Coordinates the physical connection, routing, and heartbeats of the 12-agent mesh. Supports Dijkstra pathfinding and wBFT (weighted Byzantine Fault Tolerance) controller demotions for latency isolation.',
      implements: ['P2P Federation', 'Node Heartbeats', 'Dijkstra QER Pathfinding', 'wBFT Demotions'],
      connections: ['routing_table.py', 'neural_router.py', 'consensus_engine.py']
    },
    {
      id: 'monitor_daemon',
      name: 'monitor_daemon.py',
      category: 'infra',
      description: 'Self-Healing & Telemetry Monitor Daemon',
      details: 'Runs a background daemon thread checking system resource constraints and agent node health. Triggers auto-recoveries if an agent goes unresponsive or if telemetry metrics degrade.',
      implements: ['Background lifespan check', 'get_topology_snapshot()', 'Auto-recovery hooks', 'Resource telemetry'],
      connections: ['agent_mesh.py', 'resource_arbiter.py', 'app_v2.py']
    },
    {
      id: 'mutation_engine',
      name: 'mutation_engine.py',
      category: 'evolution',
      description: 'Autonomous Code Mutation Engine',
      details: 'Scans files to detect security flaws and performance bottlenecks. Interacts with the Lead Developer to write and test optimized code modifications within a secure, isolated sandbox.',
      implements: ['Codebase Scanner', 'Sandbox Verification', 'Pydantic v2 audits', 'Genetic Filtering'],
      connections: ['evolutionary_sandbox.py', 'memory_synchronizer.py', 'base_agent.py']
    }
  ];

  // FAQ list
  const faqs = [
    {
      q: 'How does Superposition Reasoning (Attempt Sampling) solve tasks?',
      a: 'For complex or critical tasks, the system spawns parallel reasoning "universes" (multiple prompt attempts run in parallel). The outputs are evaluated dynamically by a QA verification agent which selects or synthesizes the optimal path, collapsing the wavefunction of attempts into a single perfect result.'
    },
    {
      q: 'What is the role of wBFT (weighted Byzantine Fault Tolerance)?',
      a: 'The Swarm OS federation connects multiple agent nodes. To prevent a sluggish or compromised node from stalling the network, the routing table monitors latency. If a node fails heartbeats or is too slow, wBFT dynamically demotes its weight and Dijkstra pathfinding routes requests through faster channels.'
    },
    {
      q: 'How does the VRAM Resource Arbiter manage model slots?',
      a: 'To support multiple heavy LLMs on local hardware, the Resource Arbiter locks VRAM slots before starting inference. If the required model is not loaded, it evicts the least-recently-used idle model. It also locks active slots to prevent models from being evicted mid-generation.'
    },
    {
      q: 'What is the role of the Mutation Engine and Evolutionary Sandbox?',
      a: 'The Mutation Engine scans the system for performance improvements. When a refactor is proposed, it is loaded into the isolated Evolutionary Sandbox, which runs strict syntax verification and unit tests. Only self-healing, validated modifications are merged back into the active codebase.'
    }
  ];

  // Simulated audio overview dialogue
  const podcastLines = [
    { speaker: 'sarah', text: "Hey everyone, welcome back! Today we're diving deep into a fascinating piece of software: Swarm OS v18.0. Imagine an operating system engineered as a collective intelligence of 12 specialized agents." },
    { speaker: 'todd', text: "Yeah, it's wild! It doesn't just run static programs. It literally self-heals, mutates its own code in an evolutionary sandbox, and arbitrates VRAM on local GPUs. It's like a full DevOps team on a single computer." },
    { speaker: 'sarah', text: "Exactly! And one of the most intriguing features is the 'Distributed Cognitive Stack.' How do the local LLMs coordinate reasoning?" },
    { speaker: 'todd', text: "Right, so instead of just calling one huge API, each agent has an Executive layer like Gemma3 for fast language tasks, and a Samsung TRM Reasoning Core. They work together, caching answers with semantic caching to cut down latency by 60 percent." },
    { speaker: 'sarah', text: "Wow, that's incredibly efficient. What happens if a node fails or starts lagging?" },
    { speaker: 'todd', text: "That's where the self-healing and wBFT controllers come in. If an agent goes down, the Monitor Daemon detects it instantly and routes around it using Dijkstra pathfinding. It keeps the system running 100 percent of the time." },
    { speaker: 'sarah', text: "Incredible. Let's look at the Mutation Engine next—it literally refactors its own codebase!" },
    { speaker: 'todd', text: "Yeah, it uses genetic filtering inside a secure sandbox. It ran an autonomous cycle to refactor the agent mesh into strict Pydantic v2. If a test fails, it remediates itself automatically. Truly state of the art." }
  ];

  // Quiz Question
  const quiz = {
    question: "Which component is responsible for pathfinding and routing latency monitoring within the 12-agent mesh?",
    options: [
      { text: "VRAM Resource Arbiter", correct: false, feedback: "Incorrect. The Resource Arbiter manages GPU model loading and evictions." },
      { text: "Dijkstra QER Pathfinding & wBFT Controller", correct: true, feedback: "Correct! The Dijkstra pathfinder routes requests, while the wBFT controller manages latency-based node demotions." },
      { text: "Evolutionary Sandbox", correct: false, feedback: "Incorrect. The sandbox isolates code mutation testing." },
      { text: "Attempt Sampler", correct: false, feedback: "Incorrect. The Attempt Sampler manages parallel superposition universes." }
    ]
  };

  // Fetch and sync SpeechSynthesis voices
  useEffect(() => {
    const updateVoices = () => {
      if (typeof window !== 'undefined' && window.speechSynthesis) {
        setVoices(window.speechSynthesis.getVoices());
      }
    };
    updateVoices();
    if (typeof window !== 'undefined' && window.speechSynthesis) {
      window.speechSynthesis.onvoiceschanged = updateVoices;
    }
    return () => {
      if (typeof window !== 'undefined' && window.speechSynthesis) {
        window.speechSynthesis.onvoiceschanged = null;
        window.speechSynthesis.cancel();
      }
    };
  }, []);

  // Sync activeLineRef and isPlayingRef to prevent stale closures
  useEffect(() => {
    activeLineRef.current = activeDialogueLine;
  }, [activeDialogueLine]);

  useEffect(() => {
    isPlayingRef.current = isPlaying;
    if (!isPlaying) {
      if (typeof window !== 'undefined' && window.speechSynthesis) {
        window.speechSynthesis.cancel();
      }
    } else {
      speakLine(activeDialogueLine);
    }
  }, [isPlaying]);

  // Playback time smooth progress timer (caps at 120 seconds total)
  useEffect(() => {
    let interval = null;
    if (isPlaying) {
      interval = setInterval(() => {
        setPlaybackTime((prev) => {
          const next = prev + 1;
          if (next >= 120) {
            return 119;
          }
          return next;
        });
      }, 1000);
    } else {
      clearInterval(interval);
    }
    return () => clearInterval(interval);
  }, [isPlaying]);

  // Scroll active dialogue into view
  useEffect(() => {
    if (transcriptEndRef.current) {
      transcriptEndRef.current.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
  }, [activeDialogueLine]);

  // Dynamic voice selectors for the host duo (Sarah and Todd)
  const getHostVoices = () => {
    const enVoices = voices.filter(v => v.lang.startsWith('en'));
    const all = enVoices.length > 0 ? enVoices : voices;
    
    // Sarah: female voice matching common OS voice identifiers
    let sarahVoice = all.find(v => 
      v.name.toLowerCase().includes('zira') || 
      v.name.toLowerCase().includes('samantha') || 
      v.name.toLowerCase().includes('hazel') || 
      v.name.toLowerCase().includes('female') || 
      v.name.toLowerCase().includes('google us english')
    );
    
    // Todd: male voice matching common OS voice identifiers
    let toddVoice = all.find(v => 
      v.name.toLowerCase().includes('david') || 
      v.name.toLowerCase().includes('mark') || 
      v.name.toLowerCase().includes('male') || 
      v.name.toLowerCase().includes('google uk english male')
    );

    // Dynamic fallbacks
    if (!sarahVoice && all.length > 0) {
      sarahVoice = all.find(v => v.name.toLowerCase().includes('female')) || all[0];
    }
    if (!toddVoice && all.length > 0) {
      toddVoice = all.find(v => v !== sarahVoice) || all[0];
    }

    return { sarahVoice, toddVoice };
  };

  // Main voice playback control
  const speakLine = (index) => {
    if (typeof window === 'undefined' || !window.speechSynthesis) return;

    window.speechSynthesis.cancel();

    if (index < 0 || index >= podcastLines.length) {
      setIsPlaying(false);
      setActiveDialogueLine(0);
      setPlaybackTime(0);
      return;
    }

    const line = podcastLines[index];
    const utterance = new SpeechSynthesisUtterance(line.text);
    utteranceRef.current = utterance;

    const { sarahVoice, toddVoice } = getHostVoices();
    if (line.speaker === 'sarah') {
      if (sarahVoice) utterance.voice = sarahVoice;
      utterance.pitch = 1.1; // Host A is female with standard pitch
      utterance.rate = 1.0;
    } else {
      if (toddVoice) utterance.voice = toddVoice;
      utterance.pitch = 0.9; // Host B is male with slightly lower pitch & speed
      utterance.rate = 0.95;
    }

    utterance.onstart = () => {
      setActiveDialogueLine(index);
      setPlaybackTime(Math.min(index * 15, 119));
    };

    utterance.onend = () => {
      if (isPlayingRef.current) {
        speakLine(index + 1);
      }
    };

    utterance.onerror = (e) => {
      console.error("SpeechSynthesis error:", e);
      if (e.error !== 'interrupted' && isPlayingRef.current) {
        speakLine(index + 1);
      }
    };

    window.speechSynthesis.speak(utterance);
  };

  const handlePlayPause = () => {
    setIsPlaying(!isPlaying);
  };

  const handleLineClick = (idx) => {
    setActiveDialogueLine(idx);
    setPlaybackTime(idx * 15);
    if (isPlaying) {
      speakLine(idx);
    } else {
      setIsPlaying(true);
    }
  };

  const handleQuizOption = (option) => {
    setQuizAnswer(option);
  };

  const filteredSources = selectedCategory === 'all' 
    ? sources 
    : sources.filter(s => s.category === selectedCategory);

  return (
    <div className="notebook-container">
      {/* LEFT PANEL: Sources list */}
      <div className="notebook-sources">
        <div className="sources-header">
          <svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16" className="text-pink">
            <path d="M12 0H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2zM4 1h8a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1z"/>
            <path d="M9 3H4v1h5V3zm0 3H4v1h5V6zM5 9h6v1H5V9zm0 2h6v1H5v-2z"/>
          </svg>
          Sources Browser
        </div>

        <div className="mb-3">
          <select 
            value={selectedCategory} 
            onChange={(e) => setSelectedCategory(e.target.value)}
            className="w-full bg-background-dark border border-border text-sm p-2 rounded text-text"
          >
            <option value="all">All Modules</option>
            <option value="core">Core Swarm OS</option>
            <option value="stack">Cognitive Stack</option>
            <option value="mesh">P2P Mesh Federation</option>
            <option value="infra">Self-Healing & Telemetry</option>
            <option value="evolution">Evolution & Mutation</option>
          </select>
        </div>

        <div className="sources-list">
          {filteredSources.map((source) => (
            <div 
              key={source.id} 
              onClick={() => setSelectedSource(source)}
              className={`source-item ${selectedSource?.id === source.id ? 'active' : ''}`}
            >
              <div className="source-title">{source.name}</div>
              <div className="text-xs text-text-secondary line-clamp-1">{source.description}</div>
              <div className="source-meta uppercase">Category: {source.category}</div>
            </div>
          ))}
        </div>
      </div>

      {/* CENTER PANEL: Mind Map Visualizer */}
      <div className="notebook-canvas">
        <div className="canvas-header">
          <div>
            <h2 className="canvas-title">Collective Singularity Canvas</h2>
            <div className="canvas-subtitle">Hover over or click nodes to audit architectural dependencies</div>
          </div>
          <button 
            onClick={() => { setSelectedCategory('all'); setSelectedSource(null); }}
            className="btn btn-secondary text-xs"
          >
            Reset Focus
          </button>
        </div>

        <div className="canvas-viewport">
          <svg className="svg-mindmap" viewBox="0 0 800 600">
            <defs>
              <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
                <feGaussianBlur stdDeviation="8" result="blur" />
                <feComposite in="SourceGraphic" in2="blur" operator="over" />
              </filter>
            </defs>

            {/* Connection Links */}
            {/* Center to Cognitive Stack */}
            <path className="mindmap-link" d="M 400 300 Q 290 230 180 160" stroke="rgba(0, 255, 204, 0.4)" strokeWidth="2" />
            {/* Center to Mesh */}
            <path className="mindmap-link" d="M 400 300 Q 510 230 620 160" stroke="rgba(255, 102, 0, 0.4)" strokeWidth="2" />
            {/* Center to Self-Healing */}
            <path className="mindmap-link" d="M 400 300 Q 290 370 180 440" stroke="rgba(255, 215, 0, 0.4)" strokeWidth="2" />
            {/* Center to Evolution */}
            <path className="mindmap-link" d="M 400 300 Q 510 370 620 440" stroke="rgba(0, 153, 255, 0.4)" strokeWidth="2" />

            {/* Cognitive Stack sub-connections */}
            <line x1="180" y1="160" x2="100" y2="100" stroke="rgba(0, 255, 204, 0.2)" strokeWidth="1.5" strokeDasharray="3,3" />
            <line x1="180" y1="160" x2="260" y2="100" stroke="rgba(0, 255, 204, 0.2)" strokeWidth="1.5" strokeDasharray="3,3" />
            
            {/* Mesh sub-connections */}
            <line x1="620" y1="160" x2="540" y2="100" stroke="rgba(255, 102, 0, 0.2)" strokeWidth="1.5" strokeDasharray="3,3" />
            <line x1="620" y1="160" x2="700" y2="100" stroke="rgba(255, 102, 0, 0.2)" strokeWidth="1.5" strokeDasharray="3,3" />

            {/* Self-Healing sub-connections */}
            <line x1="180" y1="440" x2="100" y2="500" stroke="rgba(255, 215, 0, 0.2)" strokeWidth="1.5" strokeDasharray="3,3" />
            <line x1="180" y1="440" x2="260" y2="500" stroke="rgba(255, 215, 0, 0.2)" strokeWidth="1.5" strokeDasharray="3,3" />

            {/* Evolution sub-connections */}
            <line x1="620" y1="440" x2="540" y2="500" stroke="rgba(0, 153, 255, 0.2)" strokeWidth="1.5" strokeDasharray="3,3" />
            <line x1="620" y1="440" x2="700" y2="500" stroke="rgba(0, 153, 255, 0.2)" strokeWidth="1.5" strokeDasharray="3,3" />

            {/* Sub-node details circles */}
            {/* Cognitive Stack sub-nodes */}
            <circle cx="100" cy="100" r="10" fill="#1a202c" stroke="#00ffcc" strokeWidth="1.5" />
            <text x="100" y="85" textAnchor="middle" style={{fontSize: '9px', fill: '#a0aec0'}}>Executive (Gemma3)</text>
            
            <circle cx="260" cy="100" r="10" fill="#1a202c" stroke="#00ffcc" strokeWidth="1.5" />
            <text x="260" y="85" textAnchor="middle" style={{fontSize: '9px', fill: '#a0aec0'}}>TRM 7M Core</text>

            {/* Mesh sub-nodes */}
            <circle cx="540" cy="100" r="10" fill="#1a202c" stroke="#ff6600" strokeWidth="1.5" />
            <text x="540" y="85" textAnchor="middle" style={{fontSize: '9px', fill: '#a0aec0'}}>Dijkstra QER</text>

            <circle cx="700" cy="100" r="10" fill="#1a202c" stroke="#ff6600" strokeWidth="1.5" />
            <text x="700" y="85" textAnchor="middle" style={{fontSize: '9px', fill: '#a0aec0'}}>wBFT Demotions</text>

            {/* Self-Healing sub-nodes */}
            <circle cx="100" cy="500" r="10" fill="#1a202c" stroke="#ffd700" strokeWidth="1.5" />
            <text x="100" y="522" textAnchor="middle" style={{fontSize: '9px', fill: '#a0aec0'}}>Telemetry API</text>

            <circle cx="260" cy="500" r="10" fill="#1a202c" stroke="#ffd700" strokeWidth="1.5" />
            <text x="260" y="522" textAnchor="middle" style={{fontSize: '9px', fill: '#a0aec0'}}>Resource Arbiter</text>

            {/* Evolution sub-nodes */}
            <circle cx="540" cy="500" r="10" fill="#1a202c" stroke="#0099ff" strokeWidth="1.5" />
            <text x="540" y="522" textAnchor="middle" style={{fontSize: '9px', fill: '#a0aec0'}}>Mutation Engine</text>

            <circle cx="700" cy="500" r="10" fill="#1a202c" stroke="#0099ff" strokeWidth="1.5" />
            <text x="700" y="522" textAnchor="middle" style={{fontSize: '9px', fill: '#a0aec0'}}>Genetic Sandboxing</text>

            {/* Core Pillars (Primary branches) */}
            {/* Cognitive Stack Node */}
            <g className="mindmap-node" onClick={() => { setSelectedCategory('stack'); setSelectedSource(sources.find(s=>s.id==='cognitive_stack')); }}>
              <circle cx="180" cy="160" r="34" fill="#00332c" stroke="#00ffcc" strokeWidth="2.5" />
              <text x="180" y="164" textAnchor="middle" fill="#fff">COGNITIVE</text>
            </g>

            {/* Agent Mesh Node */}
            <g className="mindmap-node" onClick={() => { setSelectedCategory('mesh'); setSelectedSource(sources.find(s=>s.id==='agent_mesh')); }}>
              <circle cx="620" cy="160" r="34" fill="#3d1400" stroke="#ff6600" strokeWidth="2.5" />
              <text x="620" y="164" textAnchor="middle" fill="#fff">MESH FED</text>
            </g>

            {/* Self-Healing / Telemetry Node */}
            <g className="mindmap-node" onClick={() => { setSelectedCategory('infra'); setSelectedSource(sources.find(s=>s.id==='monitor_daemon')); }}>
              <circle cx="180" cy="440" r="34" fill="#3a3000" stroke="#ffd700" strokeWidth="2.5" />
              <text x="180" y="444" textAnchor="middle" fill="#fff">STABILITY</text>
            </g>

            {/* Evolution Node */}
            <g className="mindmap-node" onClick={() => { setSelectedCategory('evolution'); setSelectedSource(sources.find(s=>s.id==='mutation_engine')); }}>
              <circle cx="620" cy="440" r="34" fill="#001f3f" stroke="#0099ff" strokeWidth="2.5" />
              <text x="620" y="444" textAnchor="middle" fill="#fff">EVOLUTION</text>
            </g>

            {/* Center Node (Collective Singularity) */}
            <g className="mindmap-node" onClick={() => { setSelectedCategory('all'); setSelectedSource(sources.find(s=>s.id==='app_v2')); }}>
              <circle cx="400" cy="300" r="50" fill="#2d001e" stroke="#ff0080" strokeWidth="3" style={{ filter: 'url(#glow)' }} />
              <text x="400" y="298" textAnchor="middle" style={{ fontWeight: 'bold', fill: '#fff', fontSize: '12px' }}>SINGULARITY</text>
              <text x="400" y="313" textAnchor="middle" style={{ fill: '#ff0080', fontSize: '9px', fontWeight: 'bold' }}>v18.0 CORE</text>
            </g>
          </svg>
        </div>
      </div>

      {/* RIGHT PANEL: NotebookLM Assistant */}
      <div className="notebook-assistant">
        <div className="assistant-tabs">
          <button 
            onClick={() => setActiveTab('briefing')} 
            className={`assistant-tab ${activeTab === 'briefing' ? 'active' : ''}`}
          >
            Briefing
          </button>
          <button 
            onClick={() => setActiveTab('faq')} 
            className={`assistant-tab ${activeTab === 'faq' ? 'active' : ''}`}
          >
            FAQs
          </button>
          <button 
            onClick={() => setActiveTab('guide')} 
            className={`assistant-tab ${activeTab === 'guide' ? 'active' : ''}`}
          >
            Study Guide
          </button>
          <button 
            onClick={() => setActiveTab('audio')} 
            className={`assistant-tab ${activeTab === 'audio' ? 'active' : ''}`}
          >
            Audio Overview
          </button>
        </div>

        <div className="assistant-content">
          {/* TAB 1: Briefing Doc */}
          {activeTab === 'briefing' && (
            <div>
              <h3 className="study-guide-title">Architectural Briefing</h3>
              <div className="text-xs text-text-secondary mb-4">A high-level summary distilled from all source documentation and system models.</div>
              
              <div className="study-card">
                <div className="font-bold text-sm text-text mb-1">Phase 18 Unified Orchestrator</div>
                <div className="text-xs text-text-secondary leading-relaxed">
                  Swarm OS utilizes a 12-agent mesh network structured in key functional layers. A dynamic router matches incoming tasks against specialized competencies (e.g. Lead Dev, Security Shield, Scribe, QA).
                </div>
              </div>

              <div className="study-card">
                <div className="font-bold text-sm text-text mb-1">Self-Healing Refactoring Loops</div>
                <div className="text-xs text-text-secondary leading-relaxed">
                  The system runs background telemetry that feeds back into an automated remediation parser. When issues arise, agents generate self-directed codebase patches verified safely in isolated sandboxes.
                </div>
              </div>

              <div className="study-card">
                <div className="font-bold text-sm text-text mb-1">Samsung TRM & Semantic Cache</div>
                <div className="text-xs text-text-secondary leading-relaxed">
                  Mathematical tasks trigger the Samsung TRM 7M core. High-frequency queries leverage semantic embeddings, delivering cached results in less than 0.1ms with a 60% system-wide hit rate.
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: FAQs */}
          {activeTab === 'faq' && (
            <div className="flex flex-col gap-2">
              <h3 className="study-guide-title">Frequently Asked Questions</h3>
              {faqs.map((faq, idx) => (
                <div key={idx} className="faq-item">
                  <button 
                    onClick={() => setExpandedFaq(expandedFaq === idx ? null : idx)}
                    className="faq-trigger"
                  >
                    <span>{faq.q}</span>
                    <span className="text-pink ml-2">{expandedFaq === idx ? '−' : '+'}</span>
                  </button>
                  {expandedFaq === idx && (
                    <div className="faq-content">
                      {faq.a}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {/* TAB 3: Study Guide */}
          {activeTab === 'guide' && (
            <div>
              <h3 className="study-guide-title">Study Guide & Glossary</h3>
              <div className="study-card">
                <div className="study-term">Neural Stitch</div>
                <div className="study-definition">A technique to seamlessly combine weights or direct reasoning states across disparate model pipelines, preserving routing context.</div>
              </div>
              <div className="study-card">
                <div className="study-term">Wavefunction Collapse (Superposition)</div>
                <div className="study-definition">Spawning parallel prompt pathways to Samplers, which are subsequently resolved into a single approved response path on validation check.</div>
              </div>

              {/* Interactive Quiz */}
              <div className="quiz-container">
                <div className="quiz-question">{quiz.question}</div>
                <div className="quiz-options">
                  {quiz.options.map((opt, index) => (
                    <button
                      key={index}
                      onClick={() => handleQuizOption(opt)}
                      className={`quiz-option ${
                        quizAnswer !== null 
                          ? opt.correct 
                            ? 'correct' 
                            : quizAnswer.text === opt.text 
                              ? 'incorrect' 
                              : ''
                          : ''
                      }`}
                      disabled={quizAnswer !== null}
                    >
                      {opt.text}
                    </button>
                  ))}
                </div>
                {quizAnswer !== null && (
                  <div className="quiz-feedback">
                    <span className={quizAnswer.correct ? "text-green" : "text-red"}>
                      {quizAnswer.correct ? "✓ " : "✗ "}
                    </span>
                    {quizAnswer.feedback}
                    <div className="mt-2">
                      <button onClick={() => setQuizAnswer(null)} className="btn btn-secondary text-xs py-1">Retry Quiz</button>
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB 4: Audio Overview (Podcast Host Simulator) */}
          {activeTab === 'audio' && (
            <div className="flex flex-col h-full">
              <div className="audio-player-container">
                <div className="player-info">
                  <div className="player-avatar-group">
                    <div className="player-avatar avatar-a">S</div>
                    <div className="player-avatar avatar-b">T</div>
                  </div>
                  <div className="player-meta">
                    <span className="player-title">AI Audio Overview</span>
                    <span className="player-hosts">Sarah & Todd • Swarm OS Analysis</span>
                  </div>
                </div>

                {/* Animated CSS Audio Waves */}
                <div className="equalizer-container">
                  <div className={`eq-bar ${isPlaying ? 'active' : ''}`}></div>
                  <div className={`eq-bar ${isPlaying ? 'active' : ''}`}></div>
                  <div className={`eq-bar ${isPlaying ? 'active' : ''}`}></div>
                  <div className={`eq-bar ${isPlaying ? 'active' : ''}`}></div>
                  <div className={`eq-bar ${isPlaying ? 'active' : ''}`}></div>
                  <div className={`eq-bar ${isPlaying ? 'active' : ''}`}></div>
                  <div className={`eq-bar ${isPlaying ? 'active' : ''}`}></div>
                  <div className={`eq-bar ${isPlaying ? 'active' : ''}`}></div>
                </div>

                <div className="player-controls">
                  <button onClick={handlePlayPause} className="play-button">
                    {isPlaying ? (
                      <svg width="18" height="18" fill="currentColor" viewBox="0 0 16 16">
                        <path d="M5.5 3.5A1.5 1.5 0 0 1 7 5v6a1.5 1.5 0 0 1-3 0V5a1.5 1.5 0 0 1 1.5-1.5zm5 0A1.5 1.5 0 0 1 12 5v6a1.5 1.5 0 0 1-3 0V5a1.5 1.5 0 0 1 1.5-1.5z"/>
                      </svg>
                    ) : (
                      <svg width="18" height="18" fill="currentColor" viewBox="0 0 16 16" className="ml-1">
                        <path d="m11.596 8.697-6.363 3.692c-.54.313-1.233-.066-1.233-.697V4.308c0-.63.692-1.01 1.233-.696l6.363 3.692a.802.802 0 0 1 0 1.393z"/>
                      </svg>
                    )}
                  </button>

                  <div className="progress-bar-container">
                    <span>{Math.floor(playbackTime / 60)}:{(playbackTime % 60).toString().padStart(2, '0')}</span>
                    <div className="progress-bar-track">
                      <div className="progress-bar-fill" style={{ width: `${(playbackTime / 120) * 100}%` }}></div>
                    </div>
                    <span>2:00</span>
                  </div>
                </div>
              </div>

              {/* Timed Scrolling Transcript */}
              <div className="transcript-scroller">
                {podcastLines.map((line, idx) => (
                  <div 
                    key={idx} 
                    onClick={() => handleLineClick(idx)}
                    className={`transcript-line ${activeDialogueLine === idx ? 'active' : ''}`}
                    ref={activeDialogueLine === idx ? transcriptEndRef : null}
                    style={{ cursor: 'pointer' }}
                  >
                    <span className={`speaker-label ${line.speaker}`}>
                      {line.speaker === 'sarah' ? 'Sarah (Host A)' : 'Todd (Host B)'}
                    </span>
                    <p className="dialogue-text">{line.text}</p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* SOURCE DETAILS MODAL / OVERLAY */}
      {selectedSource && (
        <div className="source-modal-overlay">
          <div className="source-modal">
            <div className="modal-header">
              <span className="modal-title">{selectedSource.name}</span>
              <button onClick={() => setSelectedSource(null)} className="modal-close">×</button>
            </div>
            <div className="modal-body">
              <div className="modal-section">
                <div className="modal-section-title">Summary</div>
                <div className="modal-text">{selectedSource.description}</div>
              </div>

              <div className="modal-section">
                <div className="modal-section-title">Technical Role & Details</div>
                <div className="modal-text">{selectedSource.details}</div>
              </div>

              <div className="modal-section">
                <div className="modal-section-title">Core Implementation Functions</div>
                <div className="modal-pill-container">
                  {selectedSource.implements.map((impl, idx) => (
                    <span key={idx} className="modal-pill">{impl}</span>
                  ))}
                </div>
              </div>

              <div className="modal-section">
                <div className="modal-section-title">Active Mesh Connections</div>
                <div className="modal-pill-container">
                  {selectedSource.connections.map((conn, idx) => (
                    <span key={idx} className="modal-pill" style={{borderColor: 'rgba(0, 255, 204, 0.3)', color: '#00ffcc'}}>{conn}</span>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
