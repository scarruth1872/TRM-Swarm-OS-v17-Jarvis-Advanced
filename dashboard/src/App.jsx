
import React, { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';

const API_BASE = 'http://127.0.0.1:8021';
import {
  Terminal, Cpu, Activity, RefreshCw, MessageSquare, Users, Send,
  Shield, Zap, ChevronRight, Lock, Boxes, GitBranch, Play, X, Plus,
  Loader2, CheckCircle2, Brain, FileText, XCircle, ArrowRight,
  TestTube, Package, Eye, Check, Ban, RotateCcw, Rocket, ScrollText,
  BookOpen, Sparkles, Trash2, Upload, GraduationCap, Network, Database,
  Wrench, Globe, Radio, Orbit, LayoutDashboard, Layers,
  Beaker, GitMerge, Server, Kanban, ShieldAlert, Mail, Bug, KeyRound,
  Scan, Inbox, AlertTriangle, History, Image, Paperclip, Mic, MicOff, Volume2, VolumeX
} from 'lucide-react';
import axios from 'axios';
import MeshHeatmap from './components/MeshHeatmap';
import SpatialMesh from './components/SpatialMesh';
import MitosisLog from './components/MitosisLog';
import GenerativeArchitectStudio from './components/GenerativeArchitectStudio';

// ─── Component: File Write Action Card ──────────────────────────────────
const FileWriteCard = ({ filename, lang, code }) => {
  const [isOpen, setIsOpen] = useState(false);
  
  return (
    <div className="border border-white/10 rounded-lg my-3 bg-white/5 overflow-hidden font-mono text-xs max-w-full">
      <div className="flex items-center justify-between p-3 bg-white/5 border-b border-white/5 select-none">
        <div className="flex items-center gap-2 text-accent-primary overflow-hidden">
          <FileText size={14} className="flex-shrink-0" />
          <span className="font-semibold truncate">{filename}</span>
          <span className="text-[10px] text-text-secondary flex-shrink-0">({lang})</span>
        </div>
        <button 
          onClick={() => setIsOpen(!isOpen)}
          className="text-accent-primary hover:text-white px-2 py-0.5 rounded border border-white/10 bg-white/5 text-[10px] uppercase font-bold flex-shrink-0"
        >
          {isOpen ? 'Hide Code' : 'View Code'}
        </button>
      </div>
      
      {isOpen && (
        <div className="p-3 bg-background-primary/50 overflow-x-auto max-h-[300px] border-t border-white/5 leading-relaxed text-text-secondary">
          <pre className="whitespace-pre font-mono text-[11px]">{code}</pre>
        </div>
      )}
    </div>
  );
};

const renderMessageText = (text) => {
  if (!text) return null;

  // Pattern to catch WRITE_FILE and markdown blocks
  const writePattern = /WRITE_FILE:\s*(\S+)[\s\S]*?```([\w]*)\n([\s\S]*?)\n?```/g;
  
  let match;
  let lastIndex = 0;
  const elements = [];
  
  while ((match = writePattern.exec(text)) !== null) {
    const textBefore = text.slice(lastIndex, match.index);
    if (textBefore.trim()) {
      elements.push(<span key={`text-${lastIndex}`} className="whitespace-pre-wrap">{textBefore}</span>);
    }
    
    const filename = match[1];
    const lang = match[2];
    const code = match[3];
    const key = `file-${match.index}`;
    
    elements.push(<FileWriteCard key={key} filename={filename} lang={lang} code={code} />);
    
    lastIndex = writePattern.lastIndex;
  }
  
  const textAfter = text.slice(lastIndex);
  if (textAfter.trim()) {
    elements.push(<span key={`text-${lastIndex}`} className="whitespace-pre-wrap">{textAfter}</span>);
  }
  
  return elements.length > 0 ? elements : <span className="whitespace-pre-wrap">{text}</span>;
};

// ─── Component: LCARS Sidebar ───────────────────────────────────────────
const Sidebar = ({ activeTab, onTabChange, stats }) => {
  const menuItems = [
    { id: 'overview', label: '01 SYSTEM STATUS', color: 'orange' },
    { id: 'chat', label: '02 NEURAL BRIDGE', color: 'purple' },
    { id: 'intel', label: '03 SWARM INTEL', color: 'blue' },
    { id: 'pipeline', label: '04 ARTIFACT FLOW', color: 'tan' },
    { id: 'learning', label: '05 SKILL REGISTRY', color: 'gold' },
    { id: 'mesh', label: '06 MESH NET', color: 'orange' },
    { id: 'telemetry', label: '07 TELEMETRY', color: 'cyan' },
    { id: 'federation', label: '08 FEDERATION', color: 'red' },
    { id: 'security', label: '09 SECURITY', color: 'green' },
    { id: 'research', label: '10 RESEARCH', color: 'yellow' },
    { id: 'verification', label: '11 VERIFICATION', color: 'teal' },
    { id: 'infra', label: '12 INFRASTRUCTURE', color: 'pink' },
    { id: 'testing', label: '13 TESTING', color: 'indigo' },
    { id: 'kanban', label: '14 KANBAN BOARD', color: 'purple' },
    { id: 'ddr', label: '15 DDR & VAULT', color: 'red' },
    { id: 'comms', label: '16 AGENT COMMS', color: 'cyan' },
    { id: 'spatial', label: '17 SPATIAL MESH', color: 'emerald' },
    { id: 'evolution', label: '18 EVOLUTION', color: 'blue' },
    { id: 'architect', label: '19 GENERATIVE ARCHITECT', color: 'gold' },
  ];

  return (
    <aside className="sidebar">
      {menuItems.map(item => (
        <button
          key={item.id}
          onClick={() => onTabChange(item.id)}
          className={`sidebar-button ${activeTab === item.id ? 'active' : ''}`}>
          {item.label}
        </button>
      ))}
    </aside>
  );
};

// ─── Main Application ───────────────────────────────────────────────────
export default function App() {
  const [activeTab, setActiveTab] = useState('overview');
  const [experts, setExperts] = useState([]);
  const [selectedRole, setSelectedRole] = useState(null);
  const [messages, setMessages] = useState({});
  const [inputMsg, setInputMsg] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [artifacts, setArtifacts] = useState([]);
  const [artStats, setArtStats] = useState({});
  const [selectedArtifact, setSelectedArtifact] = useState(null);
  const [orchestratorStats, setOrchestratorStats] = useState({ active_tasks: 0, status: 'offline' });
  const [agentMetrics, setAgentMetrics] = useState({});
  const [resources, setResources] = useState(null);
  const [meshTopology, setMeshTopology] = useState({ nodes: [], connections: [], alive: 0 });
  const [selectedNode, setSelectedNode] = useState(null);

  const [learnedSkills, setLearnedSkills] = useState([]);
  const [learnName, setLearnName] = useState('');
  const [learnContent, setLearnContent] = useState('');
  const [isLearning, setIsLearning] = useState(false);
  const [memoryStats, setMemStats] = useState({});
  const [memQuery, setMemQuery] = useState('');
  const [memResults, setMemResults] = useState([]);
  const [isQuerying, setIsQuerying] = useState(false);

  const [meshRouteTask, setMeshRouteTask] = useState('');
  const [isRouting, setIsRouting] = useState(false);
  const [meshRouteResult, setMeshRouteResult] = useState(null);

  const [federationData, setFederationData] = useState({ stats: null, peers: [] });
  const [securityData, setSecurityData] = useState({ stats: null, threats: [] });
  const [researchData, setResearchData] = useState({ stats: null, tasks: [] });
  const [verificationData, setVerificationData] = useState({ stats: null, queue: [] });
  const [infraData, setInfraData] = useState({ status: null, nodes: [], health: null, history: [] });
  const [testingData, setTestingData] = useState({ stats: null, runs: [] });
  const [overview, setOverview] = useState({});
  const [evolutionData, setEvolutionData] = useState({ genome: null, proposals: [] });
  const [selectedTargetFile, setSelectedTargetFile] = useState('');
  const [isMutating, setIsMutating] = useState(false);
  const [mutationStep, setMutationStep] = useState('');
  const [isVerifying, setIsVerifying] = useState(null);
  const [isIntegrating, setIsIntegrating] = useState(null);
  const [expandedProposal, setExpandedProposal] = useState(null);

  // Multimodal Voice, Image & History State
  const [attachedImage, setAttachedImage] = useState(null);
  const [isListening, setIsListening] = useState(false);
  const [speakingMsgIdx, setSpeakingMsgIdx] = useState(null);
  const [showHistoryModal, setShowHistoryModal] = useState(false);
  const [chatHistoryLogs, setChatHistoryLogs] = useState([]);
  const fileInputRef = useRef(null);

  const toggleVoiceRecording = () => {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SpeechRecognition) {
      alert("Speech Recognition API is not supported in this browser. Please use Chrome or Edge.");
      return;
    }
    if (isListening) {
      setIsListening(false);
      if (window._recognitionInstance) window._recognitionInstance.stop();
      return;
    }
    try {
      const recognition = new SpeechRecognition();
      recognition.continuous = false;
      recognition.interimResults = true;
      recognition.lang = 'en-US';
      recognition.onstart = () => setIsListening(true);
      recognition.onresult = (event) => {
        let transcript = '';
        for (let i = event.resultIndex; i < event.results.length; i++) {
          transcript += event.results[i][0].transcript;
        }
        setInputMsg(transcript);
      };
      recognition.onerror = (e) => {
        console.error('Speech recognition error:', e.error);
        setIsListening(false);
      };
      recognition.onend = () => setIsListening(false);
      window._recognitionInstance = recognition;
      recognition.start();
    } catch (e) {
      console.error('Failed to start speech recognition:', e);
      setIsListening(false);
    }
  };

  const handleSpeakMessage = (text, idx) => {
    if (!('speechSynthesis' in window)) {
      alert("Speech Synthesis is not supported in your browser.");
      return;
    }
    if (speakingMsgIdx === idx) {
      window.speechSynthesis.cancel();
      setSpeakingMsgIdx(null);
      return;
    }
    window.speechSynthesis.cancel();
    const cleanText = text.replace(/[#\*\_`\[\]\(\)]/g, ' ');
    const utterance = new SpeechSynthesisUtterance(cleanText);
    utterance.rate = 1.0;
    utterance.pitch = 1.0;
    utterance.onend = () => setSpeakingMsgIdx(null);
    utterance.onerror = () => setSpeakingMsgIdx(null);
    setSpeakingMsgIdx(idx);
    window.speechSynthesis.speak(utterance);
  };

  const handleImageUpload = (e) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (evt) => {
      setAttachedImage({
        name: file.name,
        dataUrl: evt.target.result
      });
    };
    reader.readAsDataURL(file);
  };

  const openHistoryModal = () => {
    try {
      const stored = JSON.parse(localStorage.getItem('swarm_chat_history') || '[]');
      setChatHistoryLogs(stored);
    } catch (e) {
      setChatHistoryLogs([]);
    }
    setShowHistoryModal(true);
  };

  const clearChatHistory = () => {
    localStorage.removeItem('swarm_chat_history');
    setChatHistoryLogs([]);
  };

  // QIAE Module State
  const [kanbanBoard, setKanbanBoard] = useState({});
  const [kanbanStats, setKanbanStats] = useState({});
  const [newCardTitle, setNewCardTitle] = useState('');
  const [newCardAssignee, setNewCardAssignee] = useState('');
  const [newCardPriority, setNewCardPriority] = useState('medium');
  const [ddrAntibodies, setDdrAntibodies] = useState([]);
  const [ddrStats, setDdrStats] = useState({});
  const [ddrScanCode, setDdrScanCode] = useState('');
  const [ddrScanResult, setDdrScanResult] = useState(null);
  const [secretKeys, setSecretKeys] = useState([]);
  const [mailboxAgents, setMailboxAgents] = useState([]);
  const [selectedMailbox, setSelectedMailbox] = useState(null);
  const [mailboxMessages, setMailboxMessages] = useState([]);
  const [sendMsgTo, setSendMsgTo] = useState('');
  const [sendMsgBody, setSendMsgBody] = useState('');
  const [sendMsgFrom, setSendMsgFrom] = useState('operator');
  const [uwMissions, setUwMissions] = useState([]);
  const [portableSkills, setPortableSkills] = useState([]);

  // Secrets & DDR UI Forms States
  const [newSecretKey, setNewSecretKey] = useState('');
  const [newSecretValue, setNewSecretValue] = useState('');
  const [decryptedSecrets, setDecryptedSecrets] = useState({});
  const [newAbErrorType, setNewAbErrorType] = useState('');
  const [newAbFilePattern, setNewAbFilePattern] = useState('');
  const [newAbLinePattern, setNewAbLinePattern] = useState('');
  const [newAbFix, setNewAbFix] = useState('');
  const [newAbSeverity, setNewAbSeverity] = useState('medium');

  // Autonomous Research UI States
  const [newResearchTopic, setNewResearchTopic] = useState('');
  const [researchSynthesis, setResearchSynthesis] = useState('');
  const [isSynthesizing, setIsSynthesizing] = useState(false);
  const [infraHealingData, setInfraHealingData] = useState(null);

  const chatEndRef = useRef(null);

  // Sync Logic
  useEffect(() => {
    const fetchData = async () => {
      try {
        const endpoints = [
          { key: 'experts', url: '/swarm/experts' },
          { key: 'artifacts', url: '/artifacts' },
          { key: 'resources', url: '/system/resources' },
          { key: 'mesh', url: '/mesh/topology' },
          { key: 'skills', url: '/learning/skills' },
          { key: 'memory', url: '/memory/stats' },
          { key: 'telemetry', url: '/swarm/telemetry' },
        ];

        const dataPromises = endpoints.map(e => fetch(`${API_BASE}${e.url}`).then(res => res.json()));
        const [expertsData, artifactsData, resourcesData, meshData, skillsData, memData, telemetryData] = await Promise.all(dataPromises);

        setExperts(expertsData);
        if (expertsData.length > 0 && !selectedRole) setSelectedRole(expertsData[0].role);

        // Fetch artifacts with content preview for display
        const artRes = await fetch(`${API_BASE}/artifacts?include_content=true`);
        const artData = await artRes.json();
        setArtifacts(artData.artifacts || []);
        setArtStats(artifactsData.stats);
        setResources(resourcesData);
        setMeshTopology(meshData);
        setLearnedSkills(skillsData.skills || []);
        setMemStats(memData);
        setOverview(telemetryData);

      } catch (err) { console.error("Sync Error", err); }
    };

    const fetchTabData = async (tab) => {
      try {
        if (tab === 'federation') {
          const [statsRes, peersRes] = await Promise.all([
            fetch(`${API_BASE}/federation/stats`).then(res => res.json()),
            fetch(`${API_BASE}/federation/peers`).then(res => res.json())
          ]);
          setFederationData({ stats: statsRes, peers: peersRes.peers || [] });
        } else if (tab === 'security') {
          const [statsRes, threatsRes] = await Promise.all([
            fetch(`${API_BASE}/security/stats`).then(res => res.json()),
            fetch(`${API_BASE}/security/threats`).then(res => res.json())
          ]);
          setSecurityData({ stats: statsRes, threats: threatsRes.threats || [] });
        } else if (tab === 'research') {
          const [statsRes, tasksRes] = await Promise.all([
            fetch(`${API_BASE}/research/stats`).then(res => res.json()),
            fetch(`${API_BASE}/research/tasks`).then(res => res.json())
          ]);
          setResearchData({ stats: statsRes, tasks: tasksRes.tasks || [] });
        } else if (tab === 'verification') {
          const [statsRes, queueRes] = await Promise.all([
            fetch(`${API_BASE}/verification/stats`).then(res => res.json()),
            fetch(`${API_BASE}/verification/queue`).then(res => res.json())
          ]);
          setVerificationData({ stats: statsRes, queue: queueRes.queue || [] });
        } else if (tab === 'infra') {
          const [statusRes, nodesRes, healthRes, historyRes] = await Promise.all([
            fetch(`${API_BASE}/infrastructure/status`).then(res => res.json()),
            fetch(`${API_BASE}/infrastructure/nodes`).then(res => res.json()),
            fetch(`${API_BASE}/infra/health`).then(res => res.json()),
            fetch(`${API_BASE}/infra/history`).then(res => res.json())
          ]);
          setInfraData({ 
            status: statusRes, 
            nodes: nodesRes.nodes || [], 
            health: healthRes, 
            history: historyRes.restart_history || [] 
          });
        } else if (tab === 'testing') {
          const [statsRes, runsRes] = await Promise.all([
            fetch(`${API_BASE}/testing/stats`).then(res => res.json()),
            fetch(`${API_BASE}/testing/runs`).then(res => res.json())
          ]);
          setTestingData({ stats: statsRes, runs: runsRes.runs || [] });
        } else if (tab === 'kanban') {
          const [boardRes, statsRes] = await Promise.all([
            fetch(`${API_BASE}/kanban/board`).then(r => r.json()),
            fetch(`${API_BASE}/kanban/stats`).then(r => r.json())
          ]);
          setKanbanBoard(boardRes);
          setKanbanStats(statsRes);
        } else if (tab === 'ddr') {
          const [abRes, statsRes, keysRes, infraRes] = await Promise.all([
            fetch(`${API_BASE}/ddr/antibodies`).then(r => r.json()),
            fetch(`${API_BASE}/ddr/stats`).then(r => r.json()),
            fetch(`${API_BASE}/secrets/keys`).then(r => r.json()),
            fetch(`${API_BASE}/infra/health`).then(r => r.json())
          ]);
          setDdrAntibodies(abRes.antibodies || []);
          setDdrStats(statsRes);
          setSecretKeys(keysRes.keys || []);
          setInfraHealingData(infraRes);
        } else if (tab === 'comms') {
          const [agentsRes, missionsRes, skillsRes] = await Promise.all([
            fetch(`${API_BASE}/mailbox/agents`).then(r => r.json()),
            fetch(`${API_BASE}/ultrawork/missions`).then(r => r.json()),
            fetch(`${API_BASE}/skills/portable`).then(r => r.json())
          ]);
          setMailboxAgents(agentsRes.agents || []);
          setUwMissions(missionsRes.missions || []);
          setPortableSkills(skillsRes.skills || []);
        } else if (tab === 'evolution') {
          const [genomeRes, proposalsRes] = await Promise.all([
            fetch(`${API_BASE}/evolution/genome`).then(r => r.json()),
            fetch(`${API_BASE}/evolution/proposals`).then(r => r.json())
          ]);
          setEvolutionData({ genome: genomeRes, proposals: proposalsRes.proposals || [] });
        }
      } catch (error) {
        console.error(`Error fetching data for tab ${tab}:`, error);
      }
    };

    const fetchSystemData = async () => {
      try {
        const [resArr, infraArr, testArr, orchArr] = await Promise.all([
          axios.get(`${API_BASE}/artifacts`),
          axios.get(`${API_BASE}/infrastructure/status`),
          axios.get(`${API_BASE}/testing/stats`),
          axios.get(`${API_BASE}/swarm/orchestrator/stats`)
        ]);
        setArtifacts(resArr.data.artifacts || []);
        setInfraData(prev => ({ ...prev, status: infraArr.data || {} })); // Adjusted to use setInfraData
        setTestingData(prev => ({ ...prev, stats: testArr.data || {} })); // Adjusted to use setTestingData
        setOrchestratorStats(orchArr.data || { active_tasks: 0, status: 'offline' });
      } catch (err) {
        console.error('Core data fetch error:', err);
      }
    };

    fetchData(); // Initial global fetch
    fetchTabData(activeTab); // Initial fetch for the active tab
    fetchSystemData(); // Call the new system data fetch

    // Poll lightweight metrics every 10 seconds to prevent event-loop choking
    const interval = setInterval(() => {
      fetchSystemData(); 
    }, 10000);
    return () => clearInterval(interval);
  }, [selectedRole, activeTab]);

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, selectedRole]);

  const refreshDdrAndVault = async () => {
    try {
      const [abRes, statsRes, keysRes] = await Promise.all([
        axios.get(`${API_BASE}/ddr/antibodies`),
        axios.get(`${API_BASE}/ddr/stats`),
        axios.get(`${API_BASE}/secrets/keys`)
      ]);
      setDdrAntibodies(abRes.data.antibodies || []);
      setDdrStats(statsRes.data);
      setSecretKeys(keysRes.data.keys || []);
    } catch (err) {
      console.error("Failed to refresh DDR/Vault data", err);
    }
  };

  const handleAddSecret = async () => {
    if (!newSecretKey.trim() || !newSecretValue.trim()) return;
    try {
      await axios.post(`${API_BASE}/secrets/set`, { key: newSecretKey, value: newSecretValue });
      setNewSecretKey('');
      setNewSecretValue('');
      await refreshDdrAndVault();
    } catch (err) {
      console.error("Failed to store secret", err);
    }
  };

  const handleDeleteSecret = async (key) => {
    if (!window.confirm(`Are you sure you want to delete secret key: ${key}?`)) return;
    try {
      await axios.delete(`${API_BASE}/secrets/delete/${key}`);
      setDecryptedSecrets(prev => {
        const copy = { ...prev };
        delete copy[key];
        return copy;
      });
      await refreshDdrAndVault();
    } catch (err) {
      console.error("Failed to delete secret", err);
    }
  };

  const handleDecryptSecret = async (key) => {
    try {
      const res = await axios.post(`${API_BASE}/secrets/decrypt`, { key });
      setDecryptedSecrets(prev => ({
        ...prev,
        [key]: res.data.value
      }));
      setTimeout(() => {
        setDecryptedSecrets(prev => {
          const copy = { ...prev };
          delete copy[key];
          return copy;
        });
      }, 10000);
    } catch (err) {
      console.error("Failed to decrypt secret", err);
    }
  };

  const handleRotateVaultKey = async () => {
    if (!window.confirm("Are you sure you want to rotate the Master Key? This will re-encrypt all credentials.")) return;
    try {
      await axios.post(`${API_BASE}/secrets/rotate`);
      alert("Vault master key rotated and all secrets re-encrypted successfully.");
      await refreshDdrAndVault();
    } catch (err) {
      console.error("Failed to rotate vault key", err);
    }
  };
  
  const handleToggleWatchdog = async (currentlyRunning) => {
    try {
      const endpoint = currentlyRunning ? '/infra/stop' : '/infra/start';
      await axios.post(`${API_BASE}${endpoint}`);
      fetchTabData('infra');
    } catch (err) {
      console.error("Failed to toggle watchdog", err);
    }
  };

  const handleRestartService = async (serviceName) => {
    if (!window.confirm(`Are you sure you want to force restart the service: ${serviceName}?`)) return;
    try {
      await axios.post(`${API_BASE}/infra/restart/${serviceName}`);
      fetchTabData('infra');
    } catch (err) {
      console.error(`Failed to restart service ${serviceName}`, err);
    }
  };

  const handleResetServiceCounter = async (serviceName) => {
    try {
      await axios.post(`${API_BASE}/infra/reset/${serviceName}`);
      fetchTabData('infra');
    } catch (err) {
      console.error(`Failed to reset counter for ${serviceName}`, err);
    }
  };

  const handleAddAntibody = async () => {
    if (!newAbErrorType.trim() || !newAbFix.trim() || !newAbLinePattern.trim()) return;
    try {
      await axios.post(`${API_BASE}/ddr/antibodies/add`, {
        error_type: newAbErrorType,
        file_pattern: newAbFilePattern || "*",
        line_pattern: newAbLinePattern,
        fix_description: newAbFix,
        severity: newAbSeverity
      });
      setNewAbErrorType('');
      setNewAbFilePattern('');
      setNewAbLinePattern('');
      setNewAbFix('');
      setNewAbSeverity('medium');
      await refreshDdrAndVault();
    } catch (err) {
      console.error("Failed to add antibody", err);
    }
  };

  const refreshResearch = async () => {
    try {
      const [statsRes, tasksRes] = await Promise.all([
        fetch(`${API_BASE}/research/stats`).then(res => res.json()),
        fetch(`${API_BASE}/research/tasks`).then(res => res.json())
      ]);
      setResearchData({ stats: statsRes, tasks: tasksRes.tasks || [] });
    } catch (err) {
      console.error("Failed to refresh research", err);
    }
  };

  const handleStartResearchDaemon = async () => {
    try {
      await axios.post(`${API_BASE}/research/start?interval_hours=24`);
      await refreshResearch();
    } catch (err) {
      console.error("Failed to start research daemon", err);
    }
  };

  const handleStopResearchDaemon = async () => {
    try {
      await axios.post(`${API_BASE}/research/stop`);
      await refreshResearch();
    } catch (err) {
      console.error("Failed to stop research daemon", err);
    }
  };

  const handleTriggerResearchRun = async () => {
    try {
      await axios.post(`${API_BASE}/research/run`);
      alert("Research cycle triggered in background.");
      await refreshResearch();
    } catch (err) {
      console.error("Failed to run immediate research", err);
    }
  };

  const handleAddResearchTopic = async () => {
    if (!newResearchTopic.trim()) return;
    try {
      await axios.post(`${API_BASE}/research/topics?topic=${encodeURIComponent(newResearchTopic)}`);
      setNewResearchTopic('');
      await refreshResearch();
    } catch (err) {
      console.error("Failed to add research topic", err);
    }
  };

  const handleRemoveResearchTopic = async (topic) => {
    if (!window.confirm(`Are you sure you want to remove topic: "${topic}"?`)) return;
    try {
      await axios.delete(`${API_BASE}/research/topics/${encodeURIComponent(topic)}`);
      await refreshResearch();
    } catch (err) {
      console.error("Failed to remove research topic", err);
    }
  };

  const handleGenerateResearchSynthesis = async () => {
    setIsSynthesizing(true);
    setResearchSynthesis('Synthesizing recent findings using Researcher agent...');
    try {
      const res = await axios.post(`${API_BASE}/research/synthesis`);
      setResearchSynthesis(res.data.synthesis);
    } catch (err) {
      setResearchSynthesis(`Failed to generate synthesis: ${err.message}`);
    } finally {
      setIsSynthesizing(false);
    }
  };

  const handleToggleInfraWatchdog = async () => {
    if (!infraHealingData) return;
    const isRunning = infraHealingData.running;
    try {
      if (isRunning) {
        await axios.post(`${API_BASE}/infra/stop`);
      } else {
        await axios.post(`${API_BASE}/infra/start`);
      }
      const res = await fetch(`${API_BASE}/infra/health`).then(r => r.json());
      setInfraHealingData(res);
    } catch (err) {
      console.error("Failed to toggle infra watchdog", err);
    }
  };

  const handleInfraForceRestart = async (serviceName) => {
    if (!window.confirm(`Are you sure you want to force restart service: "${serviceName}"?`)) return;
    try {
      await axios.post(`${API_BASE}/infra/restart/${encodeURIComponent(serviceName)}`);
      alert(`Force restart initiated for ${serviceName}`);
      const res = await fetch(`${API_BASE}/infra/health`).then(r => r.json());
      setInfraHealingData(res);
    } catch (err) {
      console.error("Failed to force restart service", err);
    }
  };

  const handleInfraResetCounter = async (serviceName) => {
    try {
      await axios.post(`${API_BASE}/infra/reset/${encodeURIComponent(serviceName)}`);
      const res = await fetch(`${API_BASE}/infra/health`).then(r => r.json());
      setInfraHealingData(res);
    } catch (err) {
      console.error("Failed to reset restart counter", err);
    }
  };

  const sendMessage = async () => {
    if ((!inputMsg.trim() && !attachedImage) || !selectedRole || isProcessing) return;
    const role = selectedRole;
    let fullMsg = inputMsg;
    if (attachedImage) {
      fullMsg += `\n\n[ATTACHED_MULTIMODAL_IMAGE: ${attachedImage.name}]`;
    }
    const userMsgObj = { 
      text: fullMsg, 
      sender: 'user', 
      image: attachedImage?.dataUrl,
      time: new Date().toLocaleTimeString() 
    };

    setMessages(prev => {
      const updated = { ...prev, [role]: [...(prev[role] || []), userMsgObj] };
      try {
        const hist = JSON.parse(localStorage.getItem('swarm_chat_history') || '[]');
        hist.push({ role, sender: 'user', text: fullMsg, timestamp: new Date().toISOString() });
        localStorage.setItem('swarm_chat_history', JSON.stringify(hist.slice(-100)));
      } catch (e) {}
      return updated;
    });

    setInputMsg('');
    setAttachedImage(null);
    setIsProcessing(true);
    try {
      console.log('Sending message:', { role, message: fullMsg, sender: 'user' });
      const res = await axios.post(`${API_BASE}/swarm/chat`, { role, message: fullMsg, sender: 'user' });
      console.log('Received response:', res.data);
      const agentMsgObj = {
        text: res.data.response,
        sender: 'agent',
        name: res.data.name,
        reasoning_trace: res.data.reasoning_trace,
        time: new Date().toLocaleTimeString()
      };
      setMessages(prev => {
        const updated = {
          ...prev,
          [role]: [...(prev[role] || []), agentMsgObj]
        };
        try {
          const hist = JSON.parse(localStorage.getItem('swarm_chat_history') || '[]');
          hist.push({ role, sender: 'agent', name: res.data.name, text: res.data.response, timestamp: new Date().toISOString() });
          localStorage.setItem('swarm_chat_history', JSON.stringify(hist.slice(-100)));
        } catch (e) {}
        return updated;
      });
    } catch (err) {
      console.error('Chat Error:', err);
      setMessages(prev => ({ ...prev, [role]: [...(prev[role] || []), { text: '⚠️ [NEURAL_LINK_STALLED]', sender: 'system', time: new Date().toLocaleTimeString() }] }));
    } finally { setIsProcessing(false); }
  };

  const handleArtifactAction = async (filename, action, notes) => {
    try {
      if (action === 'approve' || action === 'reject') await axios.post(`${API_BASE}/artifacts/review`, { filename, action, notes });
      else if (action === 'test') await axios.post(`${API_BASE}/artifacts/test`, { filename });
      else if (action === 'integrate') await axios.post(`${API_BASE}/artifacts/integrate`, { filename });
    } catch (err) { console.error(err); }
  };

  const queryMemory = async () => {
    if (!memQuery.trim()) return;
    setIsQuerying(true);
    try {
      const res = await axios.post(`${API_BASE}/memory/query`, { query: memQuery });
      setMemResults(res.data.results || []);
    } catch { } finally { setIsQuerying(false); }
  };

  const routeMesh = async () => {
    if (!meshRouteTask.trim()) return;
    setIsRouting(true);
    try {
      const res = await axios.post(`${API_BASE}/mesh/route`, { task: meshRouteTask });
      setMeshRouteResult(res.data);
      setMeshRouteTask('');
    } catch (err) { setMeshRouteResult({ error: err.message }); }
    finally { setIsRouting(false); }
  };

  const handleTriggerMutation = async () => {
    let fileToMutate = selectedTargetFile;
    if (!fileToMutate && evolutionData.genome?.targets?.length > 0) {
      fileToMutate = evolutionData.genome.targets[0];
    }
    if (!fileToMutate) {
      alert("No target file available for mutation!");
      return;
    }
    setIsMutating(true);
    setMutationStep("Initiating autonomous evolution thread...");
    
    try {
      setTimeout(() => setMutationStep("Analyzing codebase structure & complexity patterns..."), 2000);
      setTimeout(() => setMutationStep("Swarm Architect designing custom genetic blueprint..."), 5000);
      setTimeout(() => setMutationStep("Lead Developer generating mutated code logic..."), 8000);
      
      const response = await fetch(`${API_BASE}/evolution/mutate?target_file=${encodeURIComponent(fileToMutate)}`, {
        method: 'POST'
      });
      const result = await response.json();
      
      if (result.status === "mutation_proposed") {
        const [genomeRes, proposalsRes] = await Promise.all([
          fetch(`${API_BASE}/evolution/genome`).then(r => r.json()),
          fetch(`${API_BASE}/evolution/proposals`).then(r => r.json())
        ]);
        setEvolutionData({ genome: genomeRes, proposals: proposalsRes.proposals || [] });
        setExpandedProposal(result.proposal.proposal_id);
      } else {
        alert(`Mutation failed: ${result.message || 'Generation failed'}`);
      }
    } catch (err) {
      console.error("Mutation trigger error:", err);
      alert("Failed to trigger codebase mutation");
    } finally {
      setIsMutating(false);
      setMutationStep("");
    }
  };

  const handleVerifyProposal = async (proposalId) => {
    setIsVerifying(proposalId);
    try {
      const response = await fetch(`${API_BASE}/evolution/verify/${proposalId}`, {
        method: 'POST'
      });
      const result = await response.json();
      if (result.status === "verified") {
        const [genomeRes, proposalsRes] = await Promise.all([
          fetch(`${API_BASE}/evolution/genome`).then(r => r.json()),
          fetch(`${API_BASE}/evolution/proposals`).then(r => r.json())
        ]);
        setEvolutionData({ genome: genomeRes, proposals: proposalsRes.proposals || [] });
      } else {
        alert("Verification failed: Syntax checks or validation constraints did not pass!");
      }
    } catch (err) {
      console.error("Verification error:", err);
      alert("Failed to verify genetic proposal in isolated sandbox");
    } finally {
      setIsVerifying(null);
    }
  };

  const handleIntegrateProposal = async (proposalId) => {
    setIsIntegrating(proposalId);
    try {
      const response = await fetch(`${API_BASE}/evolution/integrate/${proposalId}`, {
        method: 'POST'
      });
      const result = await response.json();
      if (result.status === "integrated") {
        alert("Integration Successful! Mutation integrated into production codebase.");
        const [genomeRes, proposalsRes] = await Promise.all([
          fetch(`${API_BASE}/evolution/genome`).then(r => r.json()),
          fetch(`${API_BASE}/evolution/proposals`).then(r => r.json())
        ]);
        setEvolutionData({ genome: genomeRes, proposals: proposalsRes.proposals || [] });
      } else {
        alert(`Integration failed: ${result.detail || 'Internal server error'}`);
      }
    } catch (err) {
      console.error("Integration error:", err);
      alert("Failed to integrate genetic proposal");
    } finally {
      setIsIntegrating(null);
    }
  };

  return (
    <div className="container">
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} stats={{ resources, artifacts, memoryStats }} />
      <main className="main-content">
        <AnimatePresence mode="wait">

          {/* 01 SYSTEM STATUS */}
          {activeTab === 'overview' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Emergence Telemetry</h1>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="stat-card">
                  <div className="stat-card-label">Overall Status</div>
                  <div className={`stat-card-value text-lg ${overview?.status === 'Stable' ? 'text-accent-success' : 'text-accent-warning'}`}>{overview?.status || '...'}</div>
                </div>
                <div className="stat-card">
                  <div className="stat-card-label">Mesh Coherence</div>
                  <div className="stat-card-value text-lg">{(overview?.mesh_coherence * 100)?.toFixed(0) || 0}%</div>
                </div>
                <div className="stat-card">
                  <div className="stat-card-label">Harmony Index</div>
                  <div className="stat-card-value text-lg">{overview?.harmony_index?.toFixed(2) || 0}</div>
                </div>
                <div className="stat-card">
                  <div className="stat-card-label">Active Proposals</div>
                  <div className="stat-card-value text-lg">{overview?.active_proposals || 0}</div>
                </div>
              </div>

              <div className="panel-grid panel-grid-3-col">
                {/* Column 1: System & Resources */}
                <div className="col-span-1 flex flex-col gap-4">
                  <div className="panel">
                    <div className="panel-header">
                      <h2 className="panel-title">System Resources</h2>
                      <Cpu className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body space-y-3">
                      <div>
                        <div className="flex justify-between text-sm mb-1"><span>CPU Usage</span><span>{overview?.system?.cpu_percent?.toFixed(1) || 0}%</span></div>
                        <div className="w-full bg-background-primary rounded-full h-2.5"><div className="bg-accent-primary h-2.5 rounded-full" style={{ width: `${overview?.system?.cpu_percent || 0}%` }}></div></div>
                      </div>
                      <div>
                        <div className="flex justify-between text-sm mb-1"><span>Memory Usage</span><span>{overview?.system?.memory_percent?.toFixed(1) || 0}%</span></div>
                        <div className="w-full bg-background-primary rounded-full h-2.5"><div className="bg-accent-primary h-2.5 rounded-full" style={{ width: `${overview?.system?.memory_percent || 0}%` }}></div></div>
                      </div>
                    </div>
                  </div>
                  <div className="panel">
                    <div className="panel-header">
                      <h2 className="panel-title">Resource Arbiter (VRAM)</h2>
                      <Database className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body space-y-2">
                      <div className="flex justify-between text-sm"><span>Total:</span><span>{overview?.resource_arbiter?.total_gb?.toFixed(2) || 0} GB</span></div>
                      <div className="flex justify-between text-sm"><span>Allocated:</span><span>{overview?.resource_arbiter?.allocated_gb?.toFixed(2) || 0} GB</span></div>
                      <div className="flex justify-between text-sm"><span>Available:</span><span>{overview?.resource_arbiter?.available_gb?.toFixed(2) || 0} GB</span></div>
                    </div>
                  </div>
                </div>

                {/* Column 2: Distributed Stacks */}
                <div className="col-span-1 panel">
                  <div className="panel-header">
                    <h2 className="panel-title">Distributed Stacks</h2>
                    <Layers className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body space-y-2">
                    {overview?.distributed_stacks && Object.entries(overview.distributed_stacks).map(([stack, data]) => (
                      <div key={stack} className="card-secondary">
                        <div className="flex justify-between items-center">
                          <span className="font-bold capitalize">{stack.replace('_', ' ')}</span>
                          <div className={`tag ${data.status === 'Healthy' ? 'tag-success' : 'tag-warning'}`}>{data.status}</div>
                        </div>
                        <div className="text-xs text-text-secondary mt-1">Load: {data.load}% | Agents: {data.agents}</div>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="col-span-1 panel">
                  <div className="panel-header">
                    <h2 className="panel-title">Active Superpositions</h2>
                    <GitMerge className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body space-y-2">
                    {overview?.superpositions?.map((sup) => (
                      <div key={sup.id || sup.protocol + sup.agents.join('')} className="card-secondary">
                        <div className="font-bold text-sm">{sup.protocol}</div>
                        <div className="text-xs text-text-secondary mt-1">Agents: {sup.agents.join(', ')}</div>
                        <div className="text-xs text-text-secondary">State: {sup.state}</div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </motion.div>
          )}

          {/* 02 NEURAL BRIDGE */}
          {activeTab === 'chat' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full chat-container-layout">
              <div className="flex items-center justify-between mb-2">
                <h1 className="section-title">Neural Bridge // Swarm Comm</h1>
                <div className="flex items-center gap-4">
                  <div className="tag tag-success flex items-center gap-1">
                    <div className="w-1.5 h-1.5 rounded-full bg-accent-success animate-pulse" />
                    <span>Synchronized</span>
                  </div>
                  <div className="text-xs text-text-secondary font-mono">Channel: DIRECT_ENCRYPTED</div>
                </div>
              </div>

              <div className="flex items-center gap-2 mb-4 overflow-x-auto pb-2 custom-scrollbar no-scrollbar scroll-smooth">
                {experts.map(e => (
                  <button key={e.role} onClick={() => setSelectedRole(e.role)}
                    className={`expert-chip ${selectedRole === e.role ? 'active' : ''}`}>
                    <div className="w-3 h-3 rounded-full shadow-sm" style={{ backgroundColor: e.avatar_color }} />
                    <div className="flex flex-col items-start leading-none gap-0.5">
                      <span className="text-xs font-bold">{e.name}</span>
                      <span className="text-[9px] opacity-60 uppercase">{e.role}</span>
                    </div>
                  </button>
                ))}
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-12 gap-4 flex-1 min-h-0">
                {/* Main Chat Area */}
                <div className="lg:col-span-8 flex flex-col min-h-0 h-full">
                  <div className="panel flex-1 flex flex-col min-h-0 bg-background-primary/30 backdrop-blur-md border-white/5 shadow-2xl">
                    <div className="panel-header border-b border-white/5 py-3 px-4 flex justify-between items-center">
                      <div className="flex items-center gap-3">
                        {experts.find(e => e.role === selectedRole) && (
                          <div className="w-8 h-8 rounded-lg flex items-center justify-center text-white font-bold text-sm shadow-inner"
                            style={{ backgroundColor: experts.find(e => e.role === selectedRole).avatar_color }}>
                            {experts.find(e => e.role === selectedRole).name.charAt(0)}
                          </div>
                        )}
                        <div>
                          <h2 className="panel-title text-sm">{selectedRole?.toUpperCase()}</h2>
                          <div className="text-[10px] text-accent-success font-mono">REALTIME COGNITION STREAM</div>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <button onClick={openHistoryModal} className="px-2.5 py-1 rounded-md hover:bg-white/10 text-text-secondary hover:text-white transition-colors flex items-center gap-1.5 text-xs font-mono border border-white/10" title="View Chat History">
                          <History size={14} />
                          <span>History</span>
                        </button>
                        <button className="p-1.5 rounded-md hover:bg-white/10 text-text-secondary transition-colors" title="Export Log">
                          <FileText size={16} />
                        </button>
                        <button className="p-1.5 rounded-md hover:bg-white/10 text-text-secondary transition-colors" title="Clear Buffer" onClick={() => setMessages(prev => ({ ...prev, [selectedRole]: [] }))}>
                          <XCircle size={16} />
                        </button>
                      </div>
                    </div>

                    <div className="panel-body flex-1 overflow-y-auto custom-scrollbar p-6 space-y-6 flex flex-col">
                      {(messages[selectedRole] || []).length === 0 && !isProcessing && (
                        <div className="flex-1 flex flex-col items-center justify-center text-center opacity-30 select-none">
                          <MessageSquare size={64} className="mb-4 text-accent-primary" />
                          <p className="text-lg font-mono">SECURE BRIDGE ESTABLISHED</p>
                          <p className="text-xs font-mono max-w-[200px]">Waiting for operator input to begin neural transfer.</p>
                        </div>
                      )}

                      {(messages[selectedRole] || []).map((msg, i) => (
                        <div key={`${selectedRole}-${i}-${msg.time}`} className={`chat-message-row ${msg.sender === 'user' ? 'user' : 'agent'}`}>
                          {msg.sender === 'agent' && (
                            <div className="w-8 h-8 rounded-lg flex-shrink-0 mt-1 flex items-center justify-center text-white text-xs font-bold"
                              style={{ backgroundColor: experts.find(e => e.role === selectedRole)?.avatar_color || '#555' }}>
                              {msg.name?.charAt(0) || 'A'}
                            </div>
                          )}
                          <div className="message-bubble-group">
                            <div className="message-bubble relative group/bubble">
                              {msg.image && (
                                <img src={msg.image} alt="Uploaded attachment" className="max-w-[240px] max-h-[160px] rounded-lg mb-2 border border-white/10" />
                              )}
                              <div className="message-text">{renderMessageText(msg.text)}</div>
                              {msg.sender === 'agent' && (
                                <button 
                                  onClick={() => handleSpeakMessage(msg.text, i)}
                                  className={`mt-2 flex items-center gap-1 text-[10px] font-mono px-2 py-0.5 rounded border transition-colors ${speakingMsgIdx === i ? 'bg-accent-primary text-white border-accent-primary animate-pulse' : 'bg-white/5 border-white/10 text-text-secondary hover:text-white'}`}
                                  title="Voice Output (Text-to-Speech)"
                                >
                                  {speakingMsgIdx === i ? <VolumeX size={12} /> : <Volume2 size={12} />}
                                  <span>{speakingMsgIdx === i ? 'Stop Voice' : 'Read Out Loud'}</span>
                                </button>
                              )}
                              {msg.reasoning_trace && (
                                <div className="reasoning-indicator mt-3 pt-3 border-t border-white/10">
                                  <div className="flex items-center gap-1.5 text-[9px] uppercase tracking-tighter text-accent-primary mb-1">
                                    <Zap size={10} />
                                    <span>TRM Logic Trace</span>
                                  </div>
                                  <div className="font-mono text-[10px] bg-background-primary/30 p-2 rounded border border-white/5 text-accent-primary/80 overflow-x-auto no-scrollbar">
                                    {msg.reasoning_trace}
                                  </div>
                                </div>
                              )}
                            </div>
                            <div className="message-meta flex justify-between items-center">
                              <span>{msg.sender === 'user' ? 'OPERATOR' : (msg.name || 'SWARM_AGENT')} // {msg.time}</span>
                            </div>
                          </div>
                          {msg.sender === 'user' && (
                            <div className="w-8 h-8 rounded-lg flex-shrink-0 mt-1 flex items-center justify-center bg-accent-primary text-white text-xs font-bold">
                              OP
                            </div>
                          )}
                        </div>
                      ))}
                      {isProcessing && (
                        <div className="chat-message-row agent items-center">
                          <div className="w-8 h-8 rounded-lg flex-shrink-0 flex items-center justify-center bg-accent-primary/20 animate-pulse">
                            <Loader2 className="animate-spin text-accent-primary" size={14} />
                          </div>
                          <div className="flex flex-col gap-1 ml-3">
                            <div className="flex gap-1">
                              <span className="w-1.5 h-1.5 rounded-full bg-accent-primary animate-bounce" style={{ animationDelay: '0ms' }} />
                              <span className="w-1.5 h-1.5 rounded-full bg-accent-primary animate-bounce" style={{ animationDelay: '150ms' }} />
                              <span className="w-1.5 h-1.5 rounded-full bg-accent-primary animate-bounce" style={{ animationDelay: '300ms' }} />
                            </div>
                            <div className="text-[10px] font-mono text-accent-primary/60 uppercase tracking-widest">Cognitive Processing</div>
                          </div>
                        </div>
                      )}
                      <div ref={chatEndRef} />
                    </div>

                    <div className="panel-footer border-t border-white/5 p-4 bg-background-primary/50">
                      {attachedImage && (
                        <div className="flex items-center gap-2 mb-2 p-2 bg-white/5 border border-white/10 rounded-lg max-w-fit">
                          <img src={attachedImage.dataUrl} alt="Preview" className="w-8 h-8 object-cover rounded" />
                          <span className="text-xs font-mono text-accent-primary truncate max-w-[150px]">{attachedImage.name}</span>
                          <button onClick={() => setAttachedImage(null)} className="text-text-secondary hover:text-white p-1">
                            <X size={12} />
                          </button>
                        </div>
                      )}

                      <input type="file" ref={fileInputRef} className="hidden" accept="image/*,.pdf,.txt" onChange={handleImageUpload} />

                      <div className="relative group flex items-center gap-2">
                        <div className="relative flex-1">
                          <input
                            value={inputMsg}
                            onChange={e => setInputMsg(e.target.value)}
                            onKeyDown={e => e.key === 'Enter' && sendMessage()}
                            placeholder={isListening ? "Listening to your voice..." : "Transmit neural command or upload media..."}
                            className={`chat-input-field pr-20 ${isListening ? 'border-accent-primary ring-2 ring-accent-primary/30' : ''}`}
                          />
                          <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
                            <button
                              onClick={() => fileInputRef.current?.click()}
                              className="p-1.5 rounded-md hover:bg-white/10 text-text-secondary hover:text-accent-primary transition-colors"
                              title="Upload Image / Media File"
                            >
                              <Paperclip size={16} />
                            </button>
                            <button
                              onClick={toggleVoiceRecording}
                              className={`p-1.5 rounded-md hover:bg-white/10 transition-colors ${isListening ? 'text-accent-primary animate-pulse bg-accent-primary/20' : 'text-text-secondary hover:text-accent-primary'}`}
                              title={isListening ? "Stop Listening" : "Voice Input (Speech-to-Text)"}
                            >
                              {isListening ? <MicOff size={16} /> : <Mic size={16} />}
                            </button>
                          </div>
                        </div>

                        <button onClick={sendMessage} className="chat-send-button" disabled={(!inputMsg.trim() && !attachedImage) || isProcessing}>
                          {isProcessing ? <Loader2 className="animate-spin" size={18} /> : <Send size={18} />}
                        </button>
                      </div>
                      <div className="flex justify-between items-center mt-3 px-1">
                        <div className="flex gap-2">
                          <div className="text-[9px] text-text-secondary uppercase">Status: <span className="text-accent-success">{isListening ? 'Voice Recording Active' : 'Ready'}</span></div>
                          <div className="text-[9px] text-text-secondary uppercase">Buffer: <span className="text-accent-primary">{messages[selectedRole]?.length || 0} msgs</span></div>
                        </div>
                        <div className="text-[9px] text-text-secondary font-mono">MULTIMODAL_VOICE_STT_TTS_ACTIVE</div>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Neural Reasoning Sidebar */}
                <div className="lg:col-span-4 flex flex-col h-full min-h-0">
                  <div className="panel flex-1 flex flex-col min-h-0 border-white/5 bg-background-primary/20 backdrop-blur-sm">
                    <div className="panel-header border-b border-white/5 py-3 px-4">
                      <h2 className="panel-title text-xs flex items-center gap-2">
                        <Brain size={14} className="text-accent-primary" />
                        <span>NEURAL REASONING ENGINE</span>
                      </h2>
                    </div>
                    <div className="panel-body flex-1 overflow-y-auto custom-scrollbar p-4 space-y-4">
                      <div className="reasoning-card active">
                        <div className="reasoning-card-header">
                          <div className="flex items-center gap-2">
                            <Activity size={12} />
                            <span>Executive Thread</span>
                          </div>
                          <div className="tag tag-success text-[8px]">ONLINE</div>
                        </div>
                        <div className="reasoning-card-body">
                          <p className="text-xs text-text-secondary leading-relaxed">
                            Monitoring active cognition for <span className="text-accent-primary font-bold">@{selectedRole}</span>.
                            The Reasoning core (TRM) is analyzed per turn to ensure objective alignment.
                          </p>
                        </div>
                      </div>

                      <div className="system-feed">
                        <div className="system-feed-header">SYSTEM_ACTIVITY_FEED</div>
                        <div className="system-feed-body">
                          {(messages[selectedRole] || []).slice(-5).map((m, idx) => (
                            <div key={idx} className="feed-item">
                              <span className="feed-time">[{m.time?.split(' ')[0] || '--:--:--'}]</span>
                              <span className={`feed-sender ${m.sender === 'user' ? 'text-accent-primary' : 'text-accent-success'}`}>
                                {m.sender.toUpperCase()}
                              </span>
                              <span className="feed-action">
                                {m.sender === 'user' ? 'TRX_SENT' : 'COGNITION_RTX'}
                              </span>
                            </div>
                          ))}
                          {isProcessing && (
                            <div className="feed-item active">
                              <span className="feed-time font-mono">[{new Date().toLocaleTimeString().split(' ')[0]}]</span>
                              <span className="text-accent-warning">PROCESS</span>
                              <span className="animate-pulse">_REASONING_CORE_...</span>
                            </div>
                          )}
                        </div>
                      </div>

                      <div className="trace-legend">
                        <div className="system-feed-header">TRM_NODE_LEGEND</div>
                        <div className="grid grid-cols-2 gap-1 px-2 pt-2">
                          <div className="text-[8px] font-mono text-text-secondary">SYN: Synthesis</div>
                          <div className="text-[8px] font-mono text-text-secondary">ANA: Analysis</div>
                          <div className="text-[8px] font-mono text-text-secondary">VAL: Validation</div>
                          <div className="text-[8px] font-mono text-text-secondary">GEN: Generation</div>
                          <div className="text-[8px] font-mono text-text-secondary">EXT: Extraction</div>
                          <div className="text-[8px] font-mono text-text-secondary">FLW: Flow</div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </motion.div>
          )}

          {/* 03 SWARM INTEL */}
          {activeTab === 'intel' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Shared Intel Reservoir</h1>
              <div className="panel-grid panel-grid-3-col">
                {/* Column 1: Stats & Categories */}
                <div className="col-span-1 flex flex-col gap-4">
                  <div className="panel">
                    <div className="panel-header">
                      <h2 className="panel-title">Collective Stats</h2>
                      <Brain className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body">
                      <div className="flex justify-between items-center py-2 border-b border-border-color">
                        <span className="stat-card-label">Total Memories</span>
                        <span className="stat-card-value text-lg">{memoryStats.total_memories || 0}</span>
                      </div>
                      <div className="flex justify-between items-center py-2">
                        <span className="stat-card-label">Sync Events</span>
                        <span className="stat-card-value text-lg">{memoryStats.sync_events || 0}</span>
                      </div>
                    </div>
                  </div>
                  <div className="panel">
                    <div className="panel-header">
                      <h2 className="panel-title">Type Breakdown</h2>
                      <Boxes className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body">
                      <div className="grid grid-cols-2 gap-2">
                        {Object.entries(memoryStats.by_type || {}).map(([type, count]) => (
                          <div key={type} className="card-secondary text-center">
                            <div className="font-bold text-accent-primary text-xl">{count}</div>
                            <div className="text-xs text-text-secondary uppercase">{type}</div>
                          </div>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>

                {/* Column 2: Query Interface */}
                <div className="col-span-1 flex flex-col gap-4">
                  <div className="panel flex-1">
                    <div className="panel-header">
                      <h2 className="panel-title">Neural Query</h2>
                      <FileText className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body flex flex-col gap-4">
                      <textarea
                        value={memQuery}
                        onChange={e => setMemQuery(e.target.value)}
                        placeholder="Search the persistent knowledge bridge..."
                        className="input-field flex-1"
                      />
                    </div>
                    <div className="panel-footer">
                      <button onClick={queryMemory} disabled={isQuerying} className="button-primary w-full justify-center">
                        {isQuerying ? <Loader2 className="animate-spin" size={16} /> : <Send size={16} />}
                        <span>{isQuerying ? 'Querying...' : 'Execute Query'}</span>
                      </button>
                    </div>
                  </div>
                </div>

                {/* Column 3: Results */}
                <div className="col-span-1 flex flex-col gap-4">
                  <div className="panel flex-1">
                    <div className="panel-header">
                      <h2 className="panel-title">Query Results</h2>
                      <Sparkles className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body space-y-2 overflow-y-auto custom-scrollbar" style={{ maxHeight: '600px' }}>
                      {memResults.length > 0 ? memResults.map((r, i) => (
                        <div key={r.id || `${i}-${r.score}`} className="card-secondary">
                          <div className="flex justify-between items-start">
                            <p className="text-sm text-text-primary font-mono">{r.entry?.content || r.content || r.memory}</p>
                            <div className="tag whitespace-nowrap">{((r.score || r.match_percentage || 0) * 100).toFixed(0)}%</div>
                          </div>
                          <p className="text-xs text-text-secondary mt-2">Source: {r.entry?.author || r.source || 'Unknown'}</p>
                        </div>
                      )) : (
                        <div className="text-center py-12 text-text-secondary opacity-40">
                          <Globe size={48} className="mx-auto mb-4" />
                          <p>Enter a query to bridge collective intelligence</p>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            </motion.div>
          )}

          {/* 04 ARTIFACT FLOW / NEURAL PIPELINE */}
          {activeTab === 'pipeline' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <div className="flex justify-between items-end">
                <div>
                  <h1 className="section-title mb-1">Autonomous Neural Pipeline</h1>
                  <p className="text-[10px] text-text-secondary uppercase tracking-widest font-mono">
                    Monitoring: <span className="text-accent-primary">{orchestratorStats.active_tasks}</span> In-Flight / Registry: <span className="text-accent-primary">{artifacts.length}</span> Entities
                  </p>
                </div>
                <div className="pipeline-status-bar gap-6 bg-background-primary/30 border border-white/5">
                  <div className="flex items-center gap-2">
                    <Activity size={10} className={orchestratorStats.active_tasks > 0 ? "text-accent-primary animate-pulse" : "text-text-secondary"} />
                    <span>ORCH_LOOP: <span className={orchestratorStats.status === 'online' ? "text-accent-success" : "text-accent-warning"}>{orchestratorStats.status?.toUpperCase()}</span></span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Brain size={10} className="text-accent-primary" />
                    <span>SYNAPSE_LOAD: <span className="text-text-primary">{(12.5 * orchestratorStats.active_tasks).toFixed(1)}%</span></span>
                  </div>
                </div>
              </div>

              <div className="neural-pipeline flex-1 min-h-0">
                {/* Left Column: Adaptive Node List */}
                <div className="pipeline-node-container custom-scrollbar">
                  <div className="text-[9px] font-mono text-text-secondary mb-2 uppercase tracking-tighter opacity-50 px-2">_Active_Reasoning_Nodes</div>

                  {artifacts.filter(a => a.status === 'pending').map(a => (
                    <div
                      key={a.filename}
                      onClick={() => setSelectedArtifact(a)}
                      className={`neural-node ${selectedArtifact?.filename === a.filename ? 'active' : ''}`}
                    >
                      {a.status === 'pending' && <div className="node-pulse" />}
                      <div className="flex justify-between items-start mb-2">
                        <div className="flex flex-col">
                          <span className="font-mono text-[11px] text-text-primary truncate max-w-[180px]">{a.filename}</span>
                          <span className="text-[9px] text-text-secondary uppercase mt-0.5">Author: <span className="text-accent-primary font-bold">{a.agent || 'SYSTEM'}</span></span>
                        </div>
                        <div className="tag text-[8px] px-1">{a.type}</div>
                      </div>
                      <div className="flex items-center gap-2 mt-3">
                        <div className="flex-1 h-1 bg-white/5 rounded-full overflow-hidden">
                          <motion.div
                            initial={{ width: 0 }}
                            animate={{ width: a.status === 'pending' ? '65%' : '100%' }}
                            className={`h-full ${a.status === 'pending' ? 'bg-accent-primary animate-pulse' : 'bg-accent-success'}`}
                          />
                        </div>
                        <span className="text-[8px] font-mono opacity-60">{a.status === 'pending' ? 'ANALYZING' : 'VERIFIED'}</span>
                      </div>
                      <div className="reasoning-badge">
                        <Zap size={8} className="inline mr-1" />
                        TRACE: [SYN-&gt;ANA-&gt;VAL-&gt;GEN]
                      </div>
                    </div>
                  ))}

                  {artifacts.filter(a => a.status === 'pending').length === 0 && (
                    <div className="text-center py-12 border border-dashed border-white/5 rounded-xl opacity-30">
                      <div className="font-mono text-[10px]">NO_PENDING_SYNAPSES</div>
                    </div>
                  )}

                  <div className="text-[9px] font-mono text-text-secondary mt-6 mb-2 uppercase tracking-tighter opacity-50 px-2">_Integrated_Knowledge</div>
                  <div className="space-y-1 max-h-[400px] overflow-y-auto custom-scrollbar pr-2">
                    {artifacts.filter(a => a.status !== 'pending').map(a => (
                      <div
                        key={a.filename}
                        onClick={() => setSelectedArtifact(a)}
                        className={`neural-node py-2 ${selectedArtifact?.filename === a.filename ? 'active bg-white/5' : 'opacity-60 hover:opacity-100'}`}
                      >
                        <div className="flex justify-between items-center">
                          <span className="font-mono text-[9px] truncate max-w-[200px]">{a.filename}</span>
                          <div className={`tag text-[6px] px-1 py-0 ${a.status === 'approved' ? 'tag-success' : a.status === 'rejected' ? 'tag-danger' : 'tag-info'}`}>{a.status.toUpperCase()}</div>
                        </div>
                      </div>
                    ))}
                  </div>
                  {/* ─── Global Reasoning Overview ─── */}
                  <div className="mt-8 p-4 bg-accent-primary/5 border border-accent-primary/10 rounded-xl">
                    <div className="flex items-center gap-2 mb-3">
                      <Cpu size={14} className="text-accent-primary" />
                      <h3 className="text-[10px] font-mono font-bold tracking-wider text-accent-primary">ORCHESTRATOR_GLOBAL_REASONING</h3>
                    </div>
                    <div className="space-y-3">
                      <div className="flex justify-between items-center text-[9px] font-mono">
                        <span className="text-text-secondary uppercase">Active Proposals</span>
                        <span className="text-accent-primary">{orchestratorStats.triggered_proposals_count || 0}</span>
                      </div>
                      <div className="flex justify-between items-center text-[9px] font-mono">
                        <span className="text-text-secondary uppercase">Internal State</span>
                        <span className={`px-2 rounded-full ${orchestratorStats.status === 'online' ? 'bg-accent-success/20 text-accent-success' : 'bg-accent-warning/20 text-accent-warning'}`}>
                          {orchestratorStats.status?.toUpperCase() || 'BUSY'}
                        </span>
                      </div>

                      {orchestratorStats.recent_proposals?.length > 0 && (
                        <div className="pt-2 border-t border-white/5">
                          <p className="text-[8px] text-text-secondary uppercase mb-2">Recent Brain Projections:</p>
                          <div className="space-y-1">
                            {orchestratorStats.recent_proposals.map(p => (
                              <div key={p} className="flex items-center gap-2 text-[8px] font-mono text-text-primary/70">
                                <ChevronRight size={8} className="text-accent-primary" />
                                <span className="truncate">{p}</span>
                              </div>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  </div>
                </div>

                {/* Right Column: Deep Inspection */}
                <div className="pipeline-detail-container">
                  {selectedArtifact ? (
                    <div className="panel flex-1 flex flex-col min-h-0 bg-background-primary/30 border-accent-primary/20 backdrop-blur-md">
                      <div className="panel-header border-b border-white/5">
                        <div className="flex flex-col">
                          <h2 className="panel-title font-mono text-accent-primary">{selectedArtifact.filename}</h2>
                          <div className="text-[9px] font-mono text-text-secondary">UUID: {selectedArtifact.id || 'N/A'} // ORIGIN: {selectedArtifact.agent}</div>
                        </div>
                        <div className="flex items-center gap-2">
                          <div className={`tag ${selectedArtifact.status === 'pending' ? 'tag-warning' : 'tag-success'}`}>
                            {selectedArtifact.status.toUpperCase()}
                          </div>
                          <button onClick={() => setSelectedArtifact(null)} className="p-1 hover:bg-white/5 rounded">
                            <X size={16} />
                          </button>
                        </div>
                      </div>

                      {/* Added Metadata Section */}
                      <div className="px-4 py-2 border-b border-white/5 bg-white/5 flex gap-4 text-[10px] font-mono">
                         <div className="flex flex-col"><span className="text-text-secondary uppercase">Last Action</span><span className="text-accent-primary">{selectedArtifact.status?.toUpperCase()}</span></div>
                         <div className="flex flex-col"><span className="text-text-secondary uppercase">Category</span><span className="text-accent-success">{selectedArtifact.type || 'SOURCE'}</span></div>
                         <div className="flex flex-col flex-1"><span className="text-text-secondary uppercase">Notes</span><span className="text-text-primary truncate">{selectedArtifact.review_notes || 'No notes available'}</span></div>
                      </div>

                      <div className="panel-body flex-1 bg-background-primary/30 font-mono text-[11px] overflow-auto custom-scrollbar p-0">
                        <div className="sticky top-0 right-0 p-2 z-10 flex justify-end">
                          <div className="px-2 py-1 bg-accent-primary/10 border border-accent-primary/20 text-[9px] text-accent-primary rounded">
                            LANGUAGE: {selectedArtifact.type?.toUpperCase() || 'UNKNOWN'}
                          </div>
                        </div>
                        <pre className="p-4 leading-relaxed"><code className="text-text-primary/90">{selectedArtifact.content}</code></pre>
                      </div>

                      <div className="panel-footer border-t border-white/5 p-4 flex gap-3">
                        <button
                          onClick={() => handleArtifactAction(selectedArtifact.filename, 'reject')}
                          className="flex-1 py-2 px-4 bg-accent-danger/10 border border-accent-danger/30 text-accent-danger text-[10px] font-bold uppercase rounded-lg hover:bg-accent-danger hover:text-white transition-all flex items-center justify-center gap-2"
                        >
                          <Ban size={14} /><span>Terminate & Reject</span>
                        </button>

                        <div className="w-px h-8 bg-white/5 mx-2" />

                        <button
                          onClick={() => handleArtifactAction(selectedArtifact.filename, 'test')}
                          className="flex-1 py-2 px-4 bg-white/5 border border-white/10 text-white text-[10px] font-bold uppercase rounded-lg hover:bg-white/10 transition-all flex items-center justify-center gap-2"
                        >
                          <Beaker size={14} /><span>Neural Audit</span>
                        </button>

                        <button
                          onClick={() => handleArtifactAction(selectedArtifact.filename, 'approve')}
                          className="flex-1 py-2 px-4 bg-accent-success/10 border border-accent-success/30 text-accent-success text-[10px] font-bold uppercase rounded-lg hover:bg-accent-success hover:text-white transition-all flex items-center justify-center gap-2 shadow-[0_0_15px_rgba(34,197,94,0.1)]"
                        >
                          <Check size={14} /><span>Verify & Commit</span>
                        </button>
                      </div>
                    </div>
                  ) : (
                    <div className="flex-1 rounded-2xl border border-dashed border-white/5 flex items-center justify-center relative overflow-hidden group">
                      <div className="absolute inset-0 bg-gradient-to-br from-accent-primary/5 to-transparent pointer-events-none" />
                      <div className="text-center z-10">
                        <Package size={48} className="mx-auto mb-4 text-accent-primary/30 group-hover:scale-110 transition-transform duration-500" />
                        <h3 className="text-sm font-mono font-bold tracking-widest text-text-primary/60">NEURAL_IDLE_STATE</h3>
                        <p className="text-[10px] font-mono text-text-secondary mt-1">SELECT_ACTIVE_NODE_FOR_INSPECTION</p>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </motion.div>
          )}

          {/* 05 SKILL REGISTRY */}
          {activeTab === 'learning' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Adaptive Skill Matrix</h1>
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 flex-1">
                <div className="panel">
                  <div className="panel-header">
                    <h2 className="panel-title">Learned Skills</h2>
                    <BookOpen className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body">
                    {learnedSkills.map(skill => (
                      <div key={skill.name} className="card">
                        <div className="font-bold text-text-primary">{skill.name}</div>
                        <p className="text-sm text-text-secondary mt-1">{skill.description}</p>
                      </div>
                    ))}
                  </div>
                </div>
                <div className="panel">
                  <div className="panel-header">
                    <h2 className="panel-title">Acquire New Skill</h2>
                    <Sparkles className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body flex flex-col gap-4">
                    <input value={learnName} onChange={e => setLearnName(e.target.value)} placeholder="Skill Name (e.g., 'parse_api_docs')" className="input-field" />
                    <textarea value={learnContent} onChange={e => setLearnContent(e.target.value)} placeholder="Skill Description or Code..." rows="8" className="input-field font-mono" />
                  </div>
                  <div className="panel-footer">
                    <button onClick={() => { }} disabled={isLearning} className="button-primary">
                      {isLearning ? <Loader2 className="animate-spin" size={16} /> : <Plus size={16} />}
                      <span>{isLearning ? 'Assimilating...' : 'Assimilate Skill'}</span>
                    </button>
                  </div>
                </div>
              </div>
            </motion.div>
          )}

          {/* 06 MESH NET */}
          {activeTab === 'mesh' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Distributed Neural Mesh</h1>
              <div className="grid grid-cols-1 lg:grid-cols-12 gap-4 flex-1">
                <div className="lg:col-span-4 xl:col-span-3 flex flex-col gap-4">
                  <div className="panel">
                    <div className="panel-header">
                      <h2 className="panel-title">Mesh Overview</h2>
                      <Orbit className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body">
                      <div className="flex justify-between items-center py-2 border-b border-border-color">
                        <span className="stat-card-label">Total Nodes</span>
                        <span className="stat-card-value text-lg">{meshTopology.nodes.length}</span>
                      </div>
                      <div className="flex justify-between items-center py-2 border-b border-border-color">
                        <span className="stat-card-label">Alive Nodes</span>
                        <span className="stat-card-value text-lg text-accent-success">{meshTopology.alive_count || 0}</span>
                      </div>
                      <div className="flex justify-between items-center py-2">
                        <span className="stat-card-label">Connections</span>
                        <span className="stat-card-value text-lg">{(meshTopology.connections || []).length}</span>
                      </div>
                    </div>
                  </div>
                  <div className="panel flex-1">
                    <div className="panel-header">
                      <h2 className="panel-title">Task Routing</h2>
                      <Radio className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body">
                      <textarea value={meshRouteTask} onChange={e => setMeshRouteTask(e.target.value)} placeholder="Describe task to route..." rows="3" className="input-field font-mono" />
                    </div>
                    <div className="panel-footer">
                      <button onClick={routeMesh} disabled={isRouting} className="button-primary">
                        {isRouting ? <Loader2 className="animate-spin" size={16} /> : <Send size={16} />}
                        <span>{isRouting ? 'Routing...' : 'Route Task'}</span>
                      </button>
                    </div>
                    {meshRouteResult && (
                      <div className="p-4 border-t border-border-color">
                        <h3 className="text-xs font-bold text-text-secondary uppercase tracking-wider">Routing Solution:</h3>
                        <pre className="text-xs font-mono mt-2 overflow-auto custom-scrollbar bg-background-primary p-2 rounded-md">{JSON.stringify(meshRouteResult, null, 2)}</pre>
                      </div>
                    )}
                  </div>
                </div>
                <div className="lg:col-span-8 xl:col-span-9 panel relative">
                  <MeshHeatmap nodes={meshTopology.nodes} connections={meshTopology.connections} onNodeClick={setSelectedNode} />
                  {selectedNode && (
                    <div className="absolute bottom-4 right-4 w-72 bg-background-secondary/90 backdrop-blur-md border border-white/10 rounded-lg p-4 shadow-2xl z-50">
                      <div className="flex justify-between items-start mb-2">
                        <h3 className="font-bold text-accent-primary">{selectedNode.name}</h3>
                        <button onClick={() => setSelectedNode(null)} className="p-1 hover:bg-white/5 rounded"><X size={12}/></button>
                      </div>
                      <div className="text-[10px] space-y-2">
                        <div className="flex justify-between"><span className="text-text-secondary uppercase">Role</span><span>{selectedNode.role}</span></div>
                        <div className="flex justify-between"><span className="text-text-secondary uppercase">Status</span><span className="text-accent-success">{selectedNode.status}</span></div>
                        <div className="flex justify-between"><span className="text-text-secondary uppercase">Tasks</span><span>{selectedNode.task_count}</span></div>
                        
                        {selectedNode.hardware && (
                          <div className="pt-2 border-t border-white/5">
                            <div className="text-[8px] text-text-secondary uppercase mb-1 font-bold">Hardware specs</div>
                            <div className="grid grid-cols-2 gap-x-4 gap-y-1">
                              <div className="flex flex-col"><span className="text-[8px] text-text-secondary">CPU</span><span>{selectedNode.hardware.cores} Cores</span></div>
                              <div className="flex flex-col"><span className="text-[8px] text-text-secondary">RAM</span><span>{selectedNode.hardware.ram_gb} GB</span></div>
                              <div className="flex flex-col"><span className="text-[8px] text-text-secondary">OS</span><span>{selectedNode.hardware.os}</span></div>
                              <div className="flex flex-col"><span className="text-[8px] text-text-secondary">ARCH</span><span>{selectedNode.hardware.arch}</span></div>
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </motion.div>
          )}

          {/* 07 TELEMETRY */}
          {activeTab === 'telemetry' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Real-Time Telemetry</h1>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div className="panel flex flex-col">
                  <div className="panel-header">
                    <h2 className="panel-title">System Coherence</h2>
                    <Activity className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body flex-1 flex flex-col justify-center items-center">
                    <div className="text-5xl font-bold text-accent-primary mb-2">{(overview?.mesh_coherence * 100)?.toFixed(1) || 0}%</div>
                    <div className="text-sm text-text-secondary uppercase tracking-widest text-center">Global Swarm Coherence</div>
                  </div>
                </div>
                <div className="panel flex flex-col">
                  <div className="panel-header">
                    <h2 className="panel-title">Harmony Index</h2>
                    <Zap className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body flex-1 flex flex-col justify-center items-center">
                    <div className="text-5xl font-bold text-accent-success mb-2">{overview?.harmony_index?.toFixed(2) || '0.00'}</div>
                    <div className="text-sm text-text-secondary uppercase tracking-widest text-center">Alignment Balance</div>
                  </div>
                </div>
                <div className="panel flex flex-col">
                  <div className="panel-header">
                    <h2 className="panel-title">Active Superpositions</h2>
                    <Orbit className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body flex-1 flex flex-col justify-center items-center">
                    <div className="text-5xl font-bold text-accent-warning mb-2">{overview?.superpositions?.length || 0}</div>
                    <div className="text-sm text-text-secondary uppercase tracking-widest text-center">Parallel Reasoning Threads</div>
                  </div>
                </div>
              </div>

              <div className="panel flex-1">
                <div className="panel-header">
                  <h2 className="panel-title">Resource Utilization History</h2>
                  <Cpu className="panel-icon" size={18} />
                </div>
                <div className="panel-body">
                  <div className="space-y-6">
                    <div>
                      <div className="flex justify-between text-sm mb-2">
                        <span className="text-text-secondary">Collective CPU Core Utilization</span>
                        <span className="text-accent-primary font-mono">{overview?.system?.cpu_load?.toFixed(1) || 0}%</span>
                      </div>
                      <div className="w-full bg-background-primary rounded-full h-4 overflow-hidden border border-border-color">
                        <motion.div
                          initial={{ width: 0 }}
                          animate={{ width: `${overview?.system?.cpu_load || 0}%` }}
                          className="bg-accent-primary h-full shadow-[0_0_10px_rgba(0,170,255,0.5)]"
                        />
                      </div>
                    </div>
                    <div>
                      <div className="flex justify-between text-sm mb-2">
                        <span className="text-text-secondary">Distributed Memory Pressure</span>
                        <span className="text-accent-success font-mono">{overview?.system?.memory_usage?.toFixed(1) || 0}%</span>
                      </div>
                      <div className="w-full bg-background-primary rounded-full h-4 overflow-hidden border border-border-color">
                        <motion.div
                          initial={{ width: 0 }}
                          animate={{ width: `${overview?.system?.memory_usage || 0}%` }}
                          className="bg-accent-success h-full shadow-[0_0_10px_rgba(0,255,65,0.5)]"
                        />
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </motion.div>
          )}

          {/* 08 FEDERATION */}
          {activeTab === 'federation' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <div className="flex justify-between items-end">
                <h1 className="section-title mb-0">Federation Control</h1>
                <div className="flex items-center gap-2 text-[10px] font-mono text-accent-success bg-accent-success/10 border border-accent-success/30 px-3 py-1 rounded-full shadow-[0_0_10px_rgba(34,197,94,0.2)]">
                  <Shield size={12} />
                  <span>HMAC-SHA256 SECURED</span>
                </div>
              </div>
              {federationData.stats?.error ? (
                <div className="panel flex-1 flex flex-col items-center justify-center p-12 text-center">
                  <XCircle size={64} className="text-accent-danger mb-4 opacity-50" />
                  <h2 className="text-2xl font-bold mb-2 text-text-primary">Federated Mesh Inactive</h2>
                  <p className="text-text-secondary max-w-md">The federation service is currently waiting for secondary nodes or has not been initialized on this instance. Check logs for SEED_NODE connectivity.</p>
                </div>
              ) : (
                <>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                    <div className="stat-card">
                      <div className="flex items-center justify-between">
                        <span className="stat-card-label">Active Peers</span>
                        <Users className="stat-card-icon" size={20} />
                      </div>
                      <div className="stat-card-value">{federationData.stats?.connected_nodes || 0}</div>
                    </div>
                    <div className="stat-card">
                      <div className="flex items-center justify-between">
                        <span className="stat-card-label">Total Shared Nodes</span>
                        <Package className="stat-card-icon" size={20} />
                      </div>
                      <div className="stat-card-value">{federationData.stats?.total_nodes || 0}</div>
                    </div>
                    <div className="stat-card">
                      <div className="flex items-center justify-between">
                        <span className="stat-card-label">Local Node</span>
                        <Server className="stat-card-icon" size={20} />
                      </div>
                      <div className="text-xs font-mono text-accent-primary truncate">{federationData.stats?.local_node?.node_id || "OFFLINE"}</div>
                    </div>
                  </div>
                  <div className="panel flex-1">
                    <div className="panel-header">
                      <h2 className="panel-title">Federated Peers</h2>
                      <Server className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body overflow-x-auto">
                      <table className="data-table">
                        <thead className="data-table-header">
                          <tr>
                            <th>Peer Name</th>
                            <th>Status</th>
                            <th>Last Seen</th>
                            <th>Host/Port</th>
                            <th>Capabilities</th>
                          </tr>
                        </thead>
                        <tbody className="data-table-body">
                          {federationData.peers && federationData.peers.length > 0 ? federationData.peers.map(peer => (
                            <tr key={peer.node_id} className="data-table-row">
                              <td className="font-bold">{peer.name}</td>
                              <td><div className={`tag ${peer.status === 'online' ? 'tag-success' : 'tag-danger'}`}>{peer.status}</div></td>
                              <td className="text-xs font-mono">{peer.last_seen ? new Date(peer.last_seen).toLocaleString() : 'N/A'}</td>
                              <td className="text-xs font-mono">{peer.host}:{peer.port}</td>
                              <td>
                                <div className="flex gap-1">
                                  {peer.capabilities?.slice(0, 2).map(c => <span key={c} className="tag-xs">{c}</span>)}
                                </div>
                              </td>
                            </tr>
                          )) : (
                            <tr><td colSpan="5" className="text-center py-12 text-text-secondary opacity-50">No external peers discovered on the mesh.</td></tr>
                          )}
                        </tbody>
                      </table>
                    </div>
                  </div>
                </>
              )}
            </motion.div>
          )}

          {/* 09 SECURITY */}
          {activeTab === 'security' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Sentinel Neural Wall</h1>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div className="stat-card">
                  <div className="flex items-center justify-between">
                    <span className="stat-card-label">Threats Detected</span>
                    <Shield className="stat-card-icon" size={20} />
                  </div>
                  <div className="stat-card-value">{securityData.stats?.threats_detected_24h || 0}</div>
                </div>
                <div className="stat-card">
                  <div className="flex items-center justify-between">
                    <span className="stat-card-label">Threats Neutralized</span>
                    <CheckCircle2 className="stat-card-icon" size={20} />
                  </div>
                  <div className="stat-card-value">{securityData.stats?.threats_neutralized_24h || 0}</div>
                </div>
              </div>
              <div className="panel flex-1">
                <div className="panel-header">
                  <h2 className="panel-title">Threat Log</h2>
                  <Activity className="panel-icon" size={18} />
                </div>
                <div className="panel-body">
                  <table className="data-table">
                    <thead className="data-table-header">
                      <tr>
                        <th>Timestamp</th>
                        <th>Threat Type</th>
                        <th>Severity</th>
                        <th>Action Taken</th>
                        <th>Source</th>
                      </tr>
                    </thead>
                    <tbody className="data-table-body">
                      {securityData.threats.map(threat => (
                        <tr key={threat.id} className="data-table-row">
                          <td>{new Date(threat.timestamp).toLocaleString()}</td>
                          <td>{threat.threat_type}</td>
                          <td><div className={`tag ${threat.severity === 'critical' ? 'tag-danger' : threat.severity === 'high' ? 'tag-warning' : 'tag-info'}`}>{threat.severity}</div></td>
                          <td>{threat.action_taken}</td>
                          <td>{threat.source_ip}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </motion.div>
          )}

          {/* 10 RESEARCH */}
          {activeTab === 'research' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Autonomous Research</h1>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div className="stat-card">
                  <div className="flex items-center justify-between"><span className="stat-card-label">Active Tasks</span><Activity className="stat-card-icon text-accent-primary" size={20} /></div>
                  <div className="stat-card-value">{researchData.stats?.active_tasks || 0}</div>
                </div>
                <div className="stat-card">
                  <div className="flex items-center justify-between"><span className="stat-card-label">Completed Tasks</span><CheckCircle2 className="stat-card-icon text-accent-success" size={20} /></div>
                  <div className="stat-card-value">{researchData.stats?.run_count || researchData.stats?.completed_tasks_24h || 0}</div>
                </div>
                <div className="stat-card">
                  <div className="flex items-center justify-between"><span className="stat-card-label">Topics Tracked</span><Layers className="stat-card-icon text-accent-warning" size={20} /></div>
                  <div className="stat-card-value">{researchData.stats?.topics_count || 0}</div>
                </div>
                <div className="stat-card">
                  <div className="flex items-center justify-between"><span className="stat-card-label">Daemon Status</span><Server className="stat-card-icon" size={20} /></div>
                  <div className={`stat-card-value text-sm font-bold ${researchData.stats?.is_running ? 'text-accent-success animate-pulse' : 'text-text-secondary'}`}>
                    {researchData.stats?.is_running ? 'RUNNING' : 'STOPPED'}
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 flex-1 min-h-0">
                {/* Left Column (Span 2): Research Tasks & Synthesis */}
                <div className="lg:col-span-2 flex flex-col gap-4">
                  <div className="panel flex-1 flex flex-col min-h-0">
                    <div className="panel-header">
                      <h2 className="panel-title">Research Tasks</h2>
                      <BookOpen className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body flex-1 overflow-y-auto custom-scrollbar" style={{ maxHeight: 350 }}>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {researchData.tasks.map(task => (
                          <div key={task.task_id} className="card-secondary border-l-2 border-accent-primary flex flex-col justify-between">
                            <div>
                              <div className="flex justify-between items-start mb-2">
                                <div className="font-bold text-accent-primary text-xs">{task.objective}</div>
                                <div className="tag tag-success text-[8px] uppercase">{task.status}</div>
                              </div>
                              <div className="text-[11px] text-text-secondary line-clamp-3 mb-2">{task.summary}</div>
                            </div>
                            <div className="flex justify-between items-center text-[10px] pt-2 border-t border-border-color/30 mt-auto">
                              <div className="text-text-secondary italic font-mono">{task.timestamp?.split('T')[0] || 'Recently'}</div>
                              <button className="text-accent-primary hover:underline font-bold" onClick={() => {
                                alert(`RESEARCH: ${task.objective}\n\nSUMMARY:\n${task.summary}\n\nSOURCES:\n${task.sources?.join(', ') || 'N/A'}`);
                              }}>Read Analysis</button>
                            </div>
                          </div>
                        ))}
                        {researchData.tasks.length === 0 && (
                          <div className="col-span-2 text-center py-8 text-text-secondary opacity-40">No completed tasks yet.</div>
                        )}
                      </div>
                    </div>
                  </div>

                  <div className="panel">
                    <div className="panel-header justify-between">
                      <div className="flex items-center gap-2">
                        <h2 className="panel-title">Swarm Intelligence Synthesis</h2>
                        <Brain className="panel-icon" size={18} />
                      </div>
                      <button className="button-primary text-xs py-1" disabled={isSynthesizing || researchData.tasks.length === 0} onClick={handleGenerateResearchSynthesis}>
                        {isSynthesizing ? 'Synthesizing...' : 'Generate Synthesis'}
                      </button>
                    </div>
                    <div className="panel-body">
                      {researchSynthesis ? (
                        <div className="p-3 bg-slate-950 border border-border-color rounded text-xs font-mono whitespace-pre-wrap max-h-40 overflow-y-auto custom-scrollbar">
                          {researchSynthesis}
                        </div>
                      ) : (
                        <div className="text-center py-4 text-text-secondary opacity-40 text-xs">Click "Generate Synthesis" to summarize latest AI research findings via Seeker agent.</div>
                      )}
                    </div>
                  </div>
                </div>

                {/* Right Column: Daemon Controls & Topic Management */}
                <div className="flex flex-col gap-4">
                  <div className="panel">
                    <div className="panel-header">
                      <h2 className="panel-title">Daemon Controls</h2>
                      <Wrench className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body flex flex-col gap-3">
                      <div className="flex gap-2">
                        {researchData.stats?.is_running ? (
                          <button className="button-secondary flex-1 py-2 text-xs" onClick={handleStopResearchDaemon}>
                            <Ban size={14} className="inline mr-1" />Stop Daemon
                          </button>
                        ) : (
                          <button className="button-primary flex-1 py-2 text-xs" onClick={handleStartResearchDaemon}>
                            <Play size={14} className="inline mr-1" />Start Daemon
                          </button>
                        )}
                        <button className="button-primary flex-1 py-2 text-xs" onClick={handleTriggerResearchRun}>
                          <RefreshCw size={14} className="inline mr-1" />Trigger Run
                        </button>
                      </div>
                      {researchData.stats?.last_run && (
                        <div className="text-[10px] font-mono text-text-secondary mt-1">
                          Last cycle executed: <span className="text-text-primary">{new Date(researchData.stats.last_run).toLocaleString()}</span>
                        </div>
                      )}
                    </div>
                  </div>

                  <div className="panel flex-1 flex flex-col min-h-0">
                    <div className="panel-header">
                      <h2 className="panel-title">Research Topics</h2>
                      <Globe className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body flex-1 overflow-y-auto custom-scrollbar max-h-60">
                      {researchData.stats?.topics ? (
                        researchData.stats.topics.map(topic => (
                          <div key={topic} className="flex items-center justify-between py-1 border-b border-border-color/30 text-xs font-mono">
                            <span>• {topic}</span>
                            <button className="text-accent-danger hover:text-red-400 p-1" onClick={() => handleRemoveResearchTopic(topic)}>
                              <Trash2 size={12} />
                            </button>
                          </div>
                        ))
                      ) : (
                        <div className="text-center py-4 text-text-secondary opacity-40 text-xs">No active topics found.</div>
                      )}
                    </div>
                    <div className="panel-footer flex flex-col gap-2 border-t border-border-color pt-3 mt-auto">
                      <div className="text-[10px] font-mono text-text-secondary uppercase">Add New Topic</div>
                      <div className="flex gap-2">
                        <input value={newResearchTopic} onChange={e => setNewResearchTopic(e.target.value)}
                          placeholder="e.g. multi-agent coordination" className="input-field text-xs flex-1" />
                        <button className="button-primary px-3 py-1.5" disabled={!newResearchTopic.trim()} onClick={handleAddResearchTopic}>
                          <Plus size={14} />
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </motion.div>
          )}

          {/* 11 VERIFICATION */}
          {activeTab === 'verification' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Chain of Verification</h1>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div className="stat-card">
                  <span className="stat-card-label">Items in Queue</span>
                  <div className="stat-card-value">{verificationData.stats?.items_in_queue || 0}</div>
                </div>
                <div className="stat-card">
                  <span className="stat-card-label">Verified Last Hour</span>
                  <div className="stat-card-value">{verificationData.stats?.verified_last_hour || 0}</div>
                </div>
              </div>
              <div className="panel flex-1">
                <div className="panel-header">
                  <h2 className="panel-title">Verification Queue</h2>
                </div>
                <div className="panel-body">
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {verificationData.queue.map(item => (
                      <div key={item.item_id} className="card">
                        <div className="font-bold">{item.item_type}</div>
                        <div className="text-sm text-text-secondary">Status: {item.status}</div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </motion.div>
          )}

          {/* 12 INFRASTRUCTURE */}
          {activeTab === 'infra' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Self-Healing Infrastructure</h1>
              
              {/* Watchdog and Stats Cards */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div className="stat-card flex flex-col justify-between">
                  <div>
                    <span className="stat-card-label">Autonomic Watchdog</span>
                    <div className={`stat-card-value font-bold ${infraData.health?.running ? 'text-green-500' : 'text-red-500'}`}>
                      {infraData.health?.running ? 'ACTIVE' : 'INACTIVE'}
                    </div>
                  </div>
                  <button
                    onClick={() => handleToggleWatchdog(infraData.health?.running)}
                    className={`mt-4 px-4 py-2 rounded text-sm font-semibold transition-colors ${
                      infraData.health?.running 
                        ? 'bg-red-600 hover:bg-red-700 text-white' 
                        : 'bg-green-600 hover:bg-green-700 text-white'
                    }`}
                  >
                    {infraData.health?.running ? 'Stop Watchdog' : 'Start Watchdog'}
                  </button>
                </div>
                
                <div className="stat-card">
                  <span className="stat-card-label">System Health Status</span>
                  <div className="stat-card-value">{infraData.status?.overall_status || 'Unknown'}</div>
                </div>

                <div className="stat-card">
                  <span className="stat-card-label">Active Mesh Nodes</span>
                  <div className="stat-card-value">{infraData.status?.active_nodes || 0}</div>
                </div>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 flex-1">
                {/* Monitored Services */}
                <div className="panel flex flex-col">
                  <div className="panel-header">
                    <h2 className="panel-title">Monitored Services</h2>
                  </div>
                  <div className="panel-body overflow-auto flex-1">
                    <table className="data-table">
                      <thead className="data-table-header">
                        <tr>
                          <th>Service</th>
                          <th>Status</th>
                          <th>Latency</th>
                          <th>Restarts</th>
                          <th>Actions</th>
                        </tr>
                      </thead>
                      <tbody className="data-table-body">
                        {infraData.health?.services && Object.entries(infraData.health.services).map(([name, svc]) => (
                          <tr key={name} className="data-table-row">
                            <td className="font-semibold">{name}</td>
                            <td>
                              <span className={`px-2 py-1 rounded text-xs font-bold ${
                                svc.status === 'healthy' ? 'bg-green-900 text-green-200' :
                                svc.status === 'degraded' ? 'bg-yellow-900 text-yellow-200' :
                                svc.status === 'restarting' ? 'bg-blue-900 text-blue-200' :
                                'bg-red-900 text-red-200'
                              }`}>
                                {svc.status.toUpperCase()}
                              </span>
                            </td>
                            <td>{svc.last_response_time_ms} ms</td>
                            <td>{svc.restart_count}</td>
                            <td className="flex gap-2">
                              <button 
                                onClick={() => handleRestartService(name)}
                                className="px-2 py-1 bg-yellow-600 hover:bg-yellow-700 text-white rounded text-xs transition-colors"
                              >
                                Restart
                              </button>
                              {svc.restart_count > 0 && (
                                <button 
                                  onClick={() => handleResetServiceCounter(name)}
                                  className="px-2 py-1 bg-gray-600 hover:bg-gray-700 text-white rounded text-xs transition-colors"
                                >
                                  Reset
                                </button>
                              )}
                            </td>
                          </tr>
                        ))}
                        {(!infraData.health?.services || Object.keys(infraData.health.services).length === 0) && (
                          <tr>
                            <td colSpan="5" className="text-center py-4 text-gray-400">No monitored services found.</td>
                          </tr>
                        )}
                      </tbody>
                    </table>
                  </div>
                </div>

                {/* Mesh Nodes */}
                <div className="panel flex flex-col">
                  <div className="panel-header">
                    <h2 className="panel-title">Mesh Node Status</h2>
                  </div>
                  <div className="panel-body overflow-auto flex-1">
                    <table className="data-table">
                      <thead className="data-table-header">
                        <tr>
                          <th>Node ID</th>
                          <th>Status</th>
                          <th>CPU</th>
                          <th>Memory</th>
                        </tr>
                      </thead>
                      <tbody className="data-table-body">
                        {infraData.nodes.map(node => (
                          <tr key={node.node_id} className="data-table-row">
                            <td>{node.node_id}</td>
                            <td>{node.status}</td>
                            <td>{node.cpu_usage}%</td>
                            <td>{node.memory_usage}%</td>
                          </tr>
                        ))}
                        {infraData.nodes.length === 0 && (
                          <tr>
                            <td colSpan="4" className="text-center py-4 text-gray-400">No active mesh nodes found.</td>
                          </tr>
                        )}
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>

              {/* Restart History */}
              <div className="panel">
                <div className="panel-header">
                  <h2 className="panel-title">Restart Log & History</h2>
                </div>
                <div className="panel-body overflow-auto max-h-60">
                  <table className="data-table">
                    <thead className="data-table-header">
                      <tr>
                        <th>Timestamp</th>
                        <th>Service</th>
                        <th>Action/Reason</th>
                      </tr>
                    </thead>
                    <tbody className="data-table-body">
                      {infraData.history && infraData.history.map((entry, idx) => (
                        <tr key={idx} className="data-table-row">
                          <td>{new Date(entry.timestamp * 1000).toLocaleString()}</td>
                          <td className="font-semibold">{entry.service_name}</td>
                          <td>{entry.reason || 'Autonomic watchdog triggered service recovery restart.'}</td>
                        </tr>
                      ))}
                      {(!infraData.history || infraData.history.length === 0) && (
                        <tr>
                          <td colSpan="3" className="text-center py-4 text-gray-400">No restart events recorded. System is stable.</td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              </div>
            </motion.div>
          )}

          {/* 13 TESTING */}
          {activeTab === 'testing' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Zero-Human Testing</h1>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div className="stat-card">
                  <span className="stat-card-label">Tests Running</span>
                  <div className="stat-card-value">{testingData.stats?.tests_running || 0}</div>
                </div>
                <div className="stat-card">
                  <span className="stat-card-label">Success Rate (24h)</span>
                  <div className="stat-card-value">{testingData.stats?.success_rate_24h || 0}%</div>
                </div>
              </div>
              <div className="panel flex-1">
                <div className="panel-header">
                  <h2 className="panel-title">Test Runs</h2>
                </div>
                <div className="panel-body">
                  <table className="data-table">
                    <thead className="data-table-header">
                      <tr>
                        <th>Run ID</th>
                        <th>Status</th>
                        <th>Timestamp</th>
                        <th>Duration</th>
                      </tr>
                    </thead>
                    <tbody className="data-table-body">
                      {testingData.runs.map(run => (
                        <tr key={run.run_id} className="data-table-row">
                          <td>{run.run_id}</td>
                          <td>{run.status}</td>
                          <td>{new Date(run.timestamp).toLocaleString()}</td>
                          <td>{run.duration_seconds}s</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </motion.div>
          )}

          {/* 14 KANBAN BOARD */}
          {activeTab === 'kanban' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <div className="flex justify-between items-end">
                <h1 className="section-title">Vibe Kanban // Task Orchestration</h1>
                <div className="flex items-center gap-4">
                  <div className="tag tag-success">{kanbanStats.total_cards || 0} Cards</div>
                  <div className="tag">{kanbanStats.in_progress_count || 0} Active</div>
                </div>
              </div>

              {/* Card Creator */}
              <div className="panel">
                <div className="panel-header">
                  <h2 className="panel-title">Create Task Card</h2>
                  <Plus className="panel-icon" size={18} />
                </div>
                <div className="panel-body flex gap-4 items-end">
                  <input value={newCardTitle} onChange={e => setNewCardTitle(e.target.value)} placeholder="Task title..." className="input-field flex-1" />
                  <input value={newCardAssignee} onChange={e => setNewCardAssignee(e.target.value)} placeholder="Assignee (agent)" className="input-field" style={{ maxWidth: 180 }} />
                  <select value={newCardPriority} onChange={e => setNewCardPriority(e.target.value)} className="input-field" style={{ maxWidth: 140 }}>
                    <option value="critical">Critical</option>
                    <option value="high">High</option>
                    <option value="medium">Medium</option>
                    <option value="low">Low</option>
                  </select>
                  <button className="button-primary" disabled={!newCardTitle.trim()} onClick={async () => {
                    await axios.post(`${API_BASE}/kanban/cards`, { title: newCardTitle, assignee: newCardAssignee, priority: newCardPriority });
                    setNewCardTitle(''); setNewCardAssignee('');
                    const [b, s] = await Promise.all([fetch(`${API_BASE}/kanban/board`).then(r => r.json()), fetch(`${API_BASE}/kanban/stats`).then(r => r.json())]);
                    setKanbanBoard(b); setKanbanStats(s);
                  }}>
                    <Plus size={16} /><span>Create</span>
                  </button>
                </div>
              </div>

              {/* Kanban Columns */}
              <div className="grid grid-cols-4 gap-4 flex-1 min-h-0">
                {['TODO', 'IN_PROGRESS', 'REVIEW', 'DONE'].map(col => (
                  <div key={col} className="panel flex flex-col min-h-0">
                    <div className="panel-header">
                      <h2 className="panel-title text-xs">{col.replace('_', ' ')}</h2>
                      <div className="tag text-[8px]">{(kanbanBoard[col] || []).length}</div>
                    </div>
                    <div className="panel-body flex-1 overflow-y-auto custom-scrollbar" style={{ maxHeight: 500 }}>
                      <div className="space-y-2">
                        {(kanbanBoard[col] || []).map(card => (
                          <div key={card.card_id} className="card-secondary">
                             <div className="flex justify-between items-start mb-1">
                               <span className="font-bold text-sm">{card.title}</span>
                               <div className={`tag text-[7px] px-1 ${card.priority === 'critical' ? 'tag-danger' : card.priority === 'high' ? 'tag-warning' : 'tag-info'
                                 }`}>{card.priority}</div>
                             </div>
                             {card.assignee && <div className="text-[9px] text-text-secondary">Agent: <span className="text-accent-primary">{card.assignee}</span></div>}
                             {card.description && <div className="text-[10px] text-text-primary mt-1 line-clamp-2 opacity-80">{card.description}</div>}
                             {card.allocated_port > 0 && <div className="text-[9px] text-text-secondary">Port: {card.allocated_port}</div>}
                             <div className="flex gap-1 mt-2">
                               {col === 'TODO' && <button className="glass-button text-[8px] py-0.5 px-2" onClick={async () => {
                                 await axios.post(`${API_BASE}/kanban/cards/${card.card_id}/move`, { target_status: 'IN_PROGRESS' });
                                 const b = await fetch(`${API_BASE}/kanban/board`).then(r => r.json()); setKanbanBoard(b);
                               }}>▶ Start</button>}
                               {col === 'IN_PROGRESS' && <button className="glass-button text-[8px] py-0.5 px-2" onClick={async () => {
                                 await axios.post(`${API_BASE}/kanban/cards/${card.card_id}/move`, { target_status: 'REVIEW' });
                                 const b = await fetch(`${API_BASE}/kanban/board`).then(r => r.json()); setKanbanBoard(b);
                               }}>⏫ Review</button>}
                               {col === 'REVIEW' && <>
                                 <button className="glass-button text-[8px] py-0.5 px-2" style={{ color: '#22c55e' }} onClick={async () => {
                                   await axios.post(`${API_BASE}/kanban/cards/${card.card_id}/move`, { target_status: 'DONE' });
                                   const b = await fetch(`${API_BASE}/kanban/board`).then(r => r.json()); setKanbanBoard(b);
                                 }}>✓ Done</button>
                                 <button className="glass-button text-[8px] py-0.5 px-2" style={{ color: '#f97316' }} onClick={async () => {
                                   await axios.post(`${API_BASE}/kanban/cards/${card.card_id}/move`, { target_status: 'IN_PROGRESS' });
                                   const b = await fetch(`${API_BASE}/kanban/board`).then(r => r.json()); setKanbanBoard(b);
                                 }}>↩ Rework</button>
                               </>}
                             </div>
                           </div>
                        ))}
                        {(kanbanBoard[col] || []).length === 0 && (
                          <div className="text-center py-8 text-text-secondary opacity-30 text-[10px] font-mono">EMPTY</div>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </motion.div>
          )}

          {/* 15 DDR & VAULT */}
          {activeTab === 'ddr' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Digital DNA Repository & Secrets Vault</h1>

              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div className="stat-card">
                  <div className="flex items-center justify-between"><span className="stat-card-label">Total Antibodies</span><ShieldAlert className="stat-card-icon" size={20} /></div>
                  <div className="stat-card-value">{ddrStats.total_antibodies || 0}</div>
                </div>
                <div className="stat-card">
                  <div className="flex items-center justify-between"><span className="stat-card-label">Errors Prevented</span><Bug className="stat-card-icon" size={20} /></div>
                  <div className="stat-card-value">{ddrStats.total_prevented || 0}</div>
                </div>
                <div className="stat-card">
                  <div className="flex items-center justify-between"><span className="stat-card-label">Vault Keys</span><KeyRound className="stat-card-icon" size={20} /></div>
                  <div className="stat-card-value">{secretKeys.length}</div>
                </div>
                <div className="stat-card">
                  <div className="flex items-center justify-between"><span className="stat-card-label">Last Scan</span><Scan className="stat-card-icon" size={20} /></div>
                  <div className="stat-card-value text-sm">{ddrStats.last_scan || 'Never'}</div>
                </div>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 flex-1 min-h-0">
                {/* Left Column: Antibodies List & Registration */}
                <div className="flex flex-col gap-4">
                  <div className="panel flex flex-col min-h-0">
                    <div className="panel-header">
                      <h2 className="panel-title">Active Antibodies</h2>
                      <ShieldAlert className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body flex-1 overflow-y-auto custom-scrollbar" style={{ maxHeight: 280 }}>
                      {ddrAntibodies.map((ab, i) => (
                        <div key={ab.error_type + '-' + i} className="card-secondary mb-2">
                          <div className="flex justify-between items-start">
                            <span className="font-bold text-sm text-accent-danger">{ab.error_type}</span>
                            <div className={`tag text-[7px] ${ab.severity === 'critical' ? 'tag-danger' : ab.severity === 'high' ? 'tag-warning' : 'tag-info'}`}>
                              {ab.severity || 'medium'}
                            </div>
                          </div>
                          <p className="text-xs text-text-secondary mt-1">{ab.fix}</p>
                          {ab.pattern && <div className="text-[9px] font-mono text-accent-primary/60 mt-1">Pattern: {ab.pattern}</div>}
                        </div>
                      ))}
                      {ddrAntibodies.length === 0 && <div className="text-center py-8 text-text-secondary opacity-40">No antibodies registered</div>}
                    </div>
                  </div>

                  <div className="panel">
                    <div className="panel-header">
                      <h2 className="panel-title">Register Custom Antibody</h2>
                      <Plus className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body grid grid-cols-2 gap-2 text-xs">
                      <div className="flex flex-col col-span-2">
                        <label className="text-[10px] text-text-secondary mb-1">Error Type Name</label>
                        <input value={newAbErrorType} onChange={e => setNewAbErrorType(e.target.value)}
                          placeholder="e.g. sql_injection" className="input-field" />
                      </div>
                      <div className="flex flex-col">
                        <label className="text-[10px] text-text-secondary mb-1">File Glob</label>
                        <input value={newAbFilePattern} onChange={e => setNewAbFilePattern(e.target.value)}
                          placeholder="e.g. *.py" className="input-field" />
                      </div>
                      <div className="flex flex-col">
                        <label className="text-[10px] text-text-secondary mb-1">Severity</label>
                        <select value={newAbSeverity} onChange={e => setNewAbSeverity(e.target.value)} className="input-field">
                          <option value="low">Low</option>
                          <option value="medium">Medium</option>
                          <option value="high">High</option>
                          <option value="critical">Critical</option>
                        </select>
                      </div>
                      <div className="flex flex-col col-span-2">
                        <label className="text-[10px] text-text-secondary mb-1">Problematic Regex Pattern</label>
                        <input value={newAbLinePattern} onChange={e => setNewAbLinePattern(e.target.value)}
                          placeholder="e.g. select\b.*?\bwhere\b.*?{.*?}" className="input-field font-mono" />
                      </div>
                      <div className="flex flex-col col-span-2">
                        <label className="text-[10px] text-text-secondary mb-1">Fix Instructions</label>
                        <input value={newAbFix} onChange={e => setNewAbFix(e.target.value)}
                          placeholder="e.g. Use parameterized query execution..." className="input-field" />
                      </div>
                      <button className="button-primary col-span-2 mt-2" disabled={!newAbErrorType || !newAbFix || !newAbLinePattern} onClick={handleAddAntibody}>
                        <span>Add Antibody</span>
                      </button>
                    </div>
                  </div>
                </div>

                {/* Right Column: Code Scanner & Secrets Vault */}
                <div className="flex flex-col gap-4">
                  <div className="panel">
                    <div className="panel-header">
                      <h2 className="panel-title">Code Scanner</h2>
                      <Scan className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body">
                      <textarea value={ddrScanCode} onChange={e => setDdrScanCode(e.target.value)}
                        placeholder='Paste code to scan for vulnerabilities...\ne.g. query = f"SELECT * FROM users WHERE id={user_id}"'
                        rows="4" className="input-field font-mono w-full" />
                    </div>
                    <div className="panel-footer">
                      <button className="button-primary" disabled={!ddrScanCode.trim()} onClick={async () => {
                        const res = await axios.post(`${API_BASE}/ddr/scan`, { code: ddrScanCode });
                        setDdrScanResult(res.data);
                      }}>
                        <Scan size={16} /><span>Scan Code</span>
                      </button>
                    </div>
                    {ddrScanResult && (
                      <div className="p-4 border-t border-border-color">
                        <div className="text-xs font-bold mb-2">
                          {ddrScanResult.matches?.length > 0
                            ? <span className="text-accent-danger">⚠ {ddrScanResult.matches.length} vulnerabilities found</span>
                            : <span className="text-accent-success">✓ No known vulnerabilities detected</span>}
                        </div>
                        {ddrScanResult.matches?.map((m, i) => (
                          <div key={i} className="text-[10px] font-mono text-accent-danger/80 mb-1">
                            [{m.error_type}] {m.fix}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>

                  <div className="panel">
                    <div className="panel-header flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <h2 className="panel-title">Secrets Vault</h2>
                        <KeyRound className="panel-icon" size={18} />
                      </div>
                      <button className="text-[10px] border border-accent-warning/30 text-accent-warning bg-accent-warning/10 px-2 py-0.5 rounded hover:bg-accent-warning/20 transition-all font-mono" onClick={handleRotateVaultKey}>
                        Rotate Master Key
                      </button>
                    </div>
                    <div className="panel-body overflow-y-auto max-h-40 custom-scrollbar">
                      {secretKeys.length > 0 ? secretKeys.map(k => (
                        <div key={k} className="flex items-center gap-2 py-1.5 border-b border-border-color/40 text-xs">
                          <Lock size={12} className="text-accent-warning" />
                          <span className="font-mono">{k}</span>
                          {decryptedSecrets[k] ? (
                            <span className="font-mono text-accent-success bg-accent-success/15 px-1.5 py-0.5 rounded text-[10px] select-all ml-2">{decryptedSecrets[k]}</span>
                          ) : (
                            <button className="text-[9px] text-text-secondary hover:text-text-primary underline font-mono ml-2 animate-pulse" onClick={() => handleDecryptSecret(k)}>Decrypt</button>
                          )}
                          <button className="text-accent-danger hover:text-red-400 ml-auto p-1" onClick={() => handleDeleteSecret(k)}>
                            <Trash2 size={12} />
                          </button>
                        </div>
                      )) : (
                        <div className="text-center py-6 text-text-secondary opacity-40 text-sm">Vault empty — no secrets stored</div>
                      )}
                    </div>
                    <div className="panel-footer flex flex-col gap-2 border-t border-border-color pt-3">
                      <div className="text-[10px] font-mono text-text-secondary uppercase">Store New Secret</div>
                      <div className="grid grid-cols-2 gap-2 w-full text-xs">
                        <input value={newSecretKey} onChange={e => setNewSecretKey(e.target.value)}
                          placeholder="Secret key name (e.g. GEMINI_API_KEY)" className="input-field col-span-2" />
                        <input type="password" value={newSecretValue} onChange={e => setNewSecretValue(e.target.value)}
                          placeholder="Secret raw value" className="input-field col-span-2" />
                        <button className="button-primary col-span-2 mt-1" disabled={!newSecretKey.trim() || !newSecretValue.trim()} onClick={handleAddSecret}>
                          <span>Encrypt & Store</span>
                        </button>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Column 3: Self-Healing Infrastructure Watchdog */}
                <div className="flex flex-col gap-4">
                  <div className="panel flex-1 flex flex-col min-h-0">
                    <div className="panel-header justify-between">
                      <div className="flex items-center gap-2">
                        <h2 className="panel-title">Infrastructure Watchdog</h2>
                        <Server className="panel-icon" size={18} />
                      </div>
                      <button className={`text-[10px] border px-2 py-0.5 rounded font-mono font-bold transition-all ${
                        infraHealingData?.running 
                          ? 'border-accent-success/30 text-accent-success bg-accent-success/10 hover:bg-accent-success/20' 
                          : 'border-text-secondary/30 text-text-secondary bg-text-secondary/10 hover:bg-text-secondary/20'
                      }`} onClick={handleToggleInfraWatchdog}>
                        {infraHealingData?.running ? 'WATCHDOG: ON' : 'WATCHDOG: OFF'}
                      </button>
                    </div>
                    <div className="panel-body flex-1 overflow-y-auto custom-scrollbar">
                      <div className="flex flex-col gap-3">
                        {infraHealingData?.services && Object.entries(infraHealingData.services).map(([name, svc]) => (
                          <div key={name} className="card-secondary flex flex-col gap-2">
                            <div className="flex justify-between items-center">
                              <span className="font-bold text-xs text-accent-primary">{name}</span>
                              <span className={`tag text-[8px] uppercase ${
                                svc.status === 'healthy' ? 'tag-success animate-pulse' : svc.status === 'restarting' ? 'tag-warning' : 'tag-danger'
                              }`}>{svc.status}</span>
                            </div>
                            <div className="grid grid-cols-2 gap-1 text-[10px] font-mono text-text-secondary">
                              <div>Port: <span className="text-text-primary">:{name === 'swarm_api' ? '8021' : name === 'swarm_dashboard' ? '5173' : '5174'}</span></div>
                              <div>Latency: <span className="text-text-primary">{svc.last_response_time_ms} ms</span></div>
                              <div className="col-span-2">Failures: <span className="text-text-primary">{svc.consecutive_failures} (Restarts: {svc.restart_count})</span></div>
                            </div>
                            <div className="flex gap-2 mt-1">
                              <button className="button-secondary text-[9px] py-1 flex-1 font-mono" onClick={() => handleInfraForceRestart(name)}>
                                Restart
                              </button>
                              <button className="button-secondary text-[9px] py-1 flex-1 font-mono" onClick={() => handleInfraResetCounter(name)}>
                                Reset
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  </div>

                  <div className="panel max-h-60 flex flex-col min-h-0">
                    <div className="panel-header">
                      <h2 className="panel-title">Restart Log</h2>
                      <History className="panel-icon" size={18} />
                    </div>
                    <div className="panel-body flex-1 overflow-y-auto custom-scrollbar text-[10px] font-mono">
                      {infraHealingData?.restart_history && infraHealingData.restart_history.length > 0 ? (
                        infraHealingData.restart_history.map((log, index) => (
                          <div key={index} className="py-1 border-b border-border-color/20 flex flex-col gap-0.5">
                            <div className="flex justify-between text-accent-warning">
                              <span>🔄 {log.service}</span>
                              <span>Attempt #{log.attempt}</span>
                            </div>
                            <div className="text-[8px] text-text-secondary">
                              {new Date(log.timestamp).toLocaleString()} (PID: {log.pid})
                            </div>
                          </div>
                        ))
                      ) : (
                        <div className="text-center py-6 text-text-secondary opacity-40">No restart events logged</div>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            </motion.div>
          )}

          {/* 16 AGENT COMMS */}
          {activeTab === 'comms' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Agent Communications & Missions</h1>

              <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 flex-1 min-h-0">
                {/* Mailbox */}
                <div className="panel flex flex-col min-h-0">
                  <div className="panel-header">
                    <h2 className="panel-title">Agent Mailboxes</h2>
                    <Inbox className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body">
                    <div className="space-y-1 mb-4">
                      {mailboxAgents.map(agent => (
                        <button key={agent} className={`w-full text-left px-3 py-2 rounded-lg text-sm font-mono transition-all ${selectedMailbox === agent ? 'bg-accent-primary/20 text-accent-primary border border-accent-primary/30' : 'hover:bg-white/5 text-text-secondary'
                          }`} onClick={async () => {
                            setSelectedMailbox(agent);
                            const res = await fetch(`${API_BASE}/mailbox/${agent}/inbox`).then(r => r.json());
                            setMailboxMessages(res.messages || []);
                          }}>
                          <Mail size={12} className="inline mr-2" />{agent}
                        </button>
                      ))}
                      {mailboxAgents.length === 0 && <div className="text-center py-4 text-text-secondary opacity-40 text-xs">No mailboxes found</div>}
                    </div>

                    {selectedMailbox && (
                      <div className="border-t border-border-color pt-3">
                        <div className="text-[9px] font-mono text-text-secondary uppercase mb-2">Inbox: {selectedMailbox} ({mailboxMessages.length} msgs)</div>
                        <div className="space-y-2 max-h-40 overflow-y-auto custom-scrollbar">
                          {mailboxMessages.map((msg, i) => (
                            <div key={i} className="card-secondary text-[10px]">
                              <div className="font-bold">{msg.subject || '(no subject)'}</div>
                              <div className="text-text-secondary">{msg.body?.slice(0, 100)}</div>
                              <div className="text-[8px] text-accent-primary mt-1">From: {msg.from}</div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>

                  {/* Send Message */}
                  <div className="panel-footer flex-col gap-2">
                    <div className="text-[9px] font-mono text-text-secondary uppercase">Send Message</div>
                    <input value={sendMsgFrom} onChange={e => setSendMsgFrom(e.target.value)} placeholder="From agent" className="input-field text-xs" />
                    <input value={sendMsgTo} onChange={e => setSendMsgTo(e.target.value)} placeholder="To agent" className="input-field text-xs" />
                    <textarea value={sendMsgBody} onChange={e => setSendMsgBody(e.target.value)} placeholder="Message body..." rows="2" className="input-field text-xs" />
                    <button className="button-primary w-full justify-center" disabled={!sendMsgTo.trim() || !sendMsgBody.trim()} onClick={async () => {
                      await axios.post(`${API_BASE}/mailbox/send`, { from_agent: sendMsgFrom, to_agent: sendMsgTo, body: sendMsgBody, subject: 'Dashboard Message' });
                      setSendMsgBody(''); setSendMsgTo('');
                    }}>
                      <Send size={14} /><span>Send</span>
                    </button>
                  </div>
                </div>

                {/* Ultrawork Missions */}
                <div className="panel flex flex-col min-h-0">
                  <div className="panel-header">
                    <h2 className="panel-title">Ultrawork Missions</h2>
                    <Rocket className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body flex-1 overflow-y-auto custom-scrollbar">
                    {uwMissions.length > 0 ? uwMissions.map(m => (
                      <div key={m.mission_id} className="card-secondary mb-2">
                        <div className="flex justify-between items-start">
                          <span className="font-bold text-sm">{m.objective}</span>
                          <div className={`tag text-[7px] ${m.phase === 'completed' ? 'tag-success' : m.phase === 'acting' ? 'tag-warning' : 'tag-info'
                            }`}>{m.phase}</div>
                        </div>
                        <div className="text-[9px] text-text-secondary mt-1">ID: {m.mission_id}</div>
                        {m.attempt > 1 && <div className="text-[9px] text-accent-warning">Attempt #{m.attempt}</div>}
                      </div>
                    )) : (
                      <div className="text-center py-12 text-text-secondary opacity-40">
                        <Rocket size={32} className="mx-auto mb-2" />
                        <p className="text-xs">No active missions</p>
                      </div>
                    )}
                  </div>
                </div>

                {/* Portable Skills */}
                <div className="panel flex flex-col min-h-0">
                  <div className="panel-header">
                    <h2 className="panel-title">Portable Skills (SKILL.md)</h2>
                    <BookOpen className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body flex-1 overflow-y-auto custom-scrollbar">
                    {portableSkills.length > 0 ? portableSkills.map(s => (
                      <div key={s.name} className="card-secondary mb-2">
                        <div className="flex justify-between items-start">
                          <span className="font-bold text-sm">{s.name}</span>
                          <div className={`tag text-[7px] ${s.source === 'skill_md' ? 'tag-success' : 'tag-info'}`}>
                            {s.source === 'skill_md' ? 'SKILL.md' : 'Python'}
                          </div>
                        </div>
                        <p className="text-xs text-text-secondary mt-1">{s.description}</p>
                      </div>
                    )) : (
                      <div className="text-center py-12 text-text-secondary opacity-40">
                        <BookOpen size={32} className="mx-auto mb-2" />
                        <p className="text-xs">No skills discovered</p>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </motion.div>
          )}
          {/* 17 SPATIAL MESH */}
          {activeTab === 'spatial' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Swarm OS v14 // Spatial Mesh Projection</h1>
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 flex-1 min-h-0">
                <SpatialMesh />
                <MitosisLog />
              </div>
            </motion.div>
          )}

          {/* 18 EVOLUTION */}
          {activeTab === 'evolution' && (
            <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full gap-6">
              <h1 className="section-title">Evolutionary Ecosystem</h1>
              <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 flex-1 min-h-0">
                
                {/* Left Panel: Genome Status & Trigger Mutation */}
                <div className="panel lg:col-span-4 flex flex-col min-h-0">
                  <div className="panel-header">
                    <h2 className="panel-title">Genome Status</h2>
                    <Activity className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body flex-1 overflow-y-auto space-y-6">
                    {evolutionData.genome ? (
                      <div className="space-y-6">
                        <div className="space-y-3">
                          <div className="flex justify-between items-center pb-2 border-b border-white/5">
                            <span className="text-text-secondary uppercase text-xs font-mono">Genome Version</span>
                            <span className="text-accent-primary font-mono font-bold text-lg">{evolutionData.genome.version}</span>
                          </div>
                          <div className="flex justify-between items-center pb-2 border-b border-white/5">
                            <span className="text-text-secondary uppercase text-xs font-mono">Stability Index</span>
                            <span className="text-accent-success font-mono font-bold text-lg">{evolutionData.genome.genome_stability * 100}%</span>
                          </div>
                          <div className="flex justify-between items-center pb-2 border-b border-white/5">
                            <span className="text-text-secondary uppercase text-xs font-mono">Mutations Applied</span>
                            <span className="text-white font-mono font-bold text-lg">{evolutionData.genome.mutation_count}</span>
                          </div>
                        </div>

                        {/* Interactive Target Files Picker */}
                        <div className="space-y-3">
                          <span className="text-text-secondary uppercase text-xs font-mono block">Eligible Codebase Targets</span>
                          <div className="flex flex-col gap-2">
                            {evolutionData.genome.targets?.map(t => (
                              <button
                                key={t}
                                onClick={() => setSelectedTargetFile(t)}
                                className={`w-full text-left font-mono text-xs px-3 py-2.5 rounded border transition-all ${
                                  (selectedTargetFile === t || (!selectedTargetFile && evolutionData.genome.targets[0] === t))
                                    ? 'bg-accent-primary/10 border-accent-primary text-accent-primary font-bold shadow-[0_0_10px_rgba(59,130,246,0.15)]'
                                    : 'bg-white/5 border-white/10 text-slate-300 hover:bg-white/10 hover:border-slate-500'
                                }`}
                              >
                                {t}
                              </button>
                            ))}
                            {(!evolutionData.genome.targets || evolutionData.genome.targets.length === 0) && (
                              <div className="text-xs text-text-secondary italic">No targets found. Scanning codebase...</div>
                            )}
                          </div>
                        </div>

                        {/* Mutator Trigger */}
                        <div className="pt-4 border-t border-white/5">
                          {isMutating ? (
                            <div className="bg-slate-900 border border-accent-primary/30 p-4 rounded text-center space-y-3">
                              <Loader2 className="animate-spin text-accent-primary mx-auto" size={24} />
                              <div className="text-xs font-mono text-accent-primary animate-pulse">{mutationStep}</div>
                            </div>
                          ) : (
                            <button
                              onClick={handleTriggerMutation}
                              className="w-full bg-accent-primary hover:bg-accent-primary-hover text-white py-3 rounded font-mono uppercase tracking-wider text-xs font-bold transition-all shadow-[0_0_15px_rgba(59,130,246,0.25)] flex items-center justify-center gap-2"
                            >
                              <Sparkles size={14} />
                              <span>Trigger Codebase Mutation Cycle</span>
                            </button>
                          )}
                        </div>
                      </div>
                    ) : (
                      <div className="flex items-center justify-center h-full text-text-secondary opacity-50">
                        <Loader2 className="animate-spin mr-2" size={16} /> Loading genome...
                      </div>
                    )}
                  </div>
                </div>

                {/* Right Panel: Proposals Drawer with Diff Details */}
                <div className="panel lg:col-span-8 flex flex-col min-h-0">
                  <div className="panel-header">
                    <h2 className="panel-title">Genetic Proposals & Diff Viewer</h2>
                    <FileText className="panel-icon" size={18} />
                  </div>
                  <div className="panel-body flex-1 overflow-y-auto space-y-4">
                    {evolutionData.proposals.map(p => {
                      const isExpanded = expandedProposal === p.proposal_id;
                      return (
                        <div 
                          key={p.proposal_id} 
                          className={`border rounded overflow-hidden transition-all duration-300 ${
                            isExpanded 
                              ? 'bg-slate-950 border-accent-primary/50 shadow-lg' 
                              : 'bg-white/5 border-white/10 hover:border-slate-600'
                          }`}
                        >
                          {/* Proposal Header */}
                          <div 
                            onClick={() => setExpandedProposal(isExpanded ? null : p.proposal_id)}
                            className="p-4 flex justify-between items-center cursor-pointer hover:bg-white/5 transition-colors"
                          >
                            <div className="flex flex-col gap-1 min-w-0 flex-1 pr-4">
                              <span className="font-bold font-mono text-sm text-accent-primary truncate">{p.proposal_id}</span>
                              <span className="text-[10px] font-mono text-text-secondary truncate">{p.target_file}</span>
                            </div>
                            <div className="flex items-center gap-3">
                              <span className={`tag uppercase text-[10px] tracking-wider px-2 py-0.5 rounded font-bold ${
                                p.status === 'integrated' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' :
                                p.status === 'passed' ? 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/30' :
                                p.status === 'failed' ? 'bg-rose-500/20 text-rose-400 border border-rose-500/30' :
                                p.status === 'testing' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 animate-pulse' :
                                'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30'
                              }`}>{p.status}</span>
                              <ChevronRight 
                                size={16} 
                                className={`text-text-secondary transition-transform duration-300 ${isExpanded ? 'rotate-90 text-white' : ''}`} 
                              />
                            </div>
                          </div>

                          {/* Expanded Details / Diff Viewer */}
                          {isExpanded && (
                            <div className="p-4 border-t border-white/5 bg-slate-950 space-y-4 font-mono text-xs">
                              {/* Details / Metrics */}
                              <div className="grid grid-cols-2 gap-4 bg-slate-900/60 p-3 border border-white/5 rounded">
                                <div>
                                  <span className="text-[10px] text-text-secondary block">MUTATION SCORE</span>
                                  <span className="text-white font-bold font-mono text-sm">{(p.score * 100).toFixed(1)}% Efficacy</span>
                                </div>
                                <div>
                                  <span className="text-[10px] text-text-secondary block">MUTATION TYPE</span>
                                  <span className="text-white font-bold font-mono text-sm uppercase">{p.mutation_type}</span>
                                </div>
                              </div>

                              {/* Architectural Description */}
                              <div className="space-y-1">
                                <span className="text-[10px] text-text-secondary font-bold">ARCHITECTURAL RATIONALE</span>
                                <div className="text-slate-300 p-2.5 rounded bg-slate-900 border border-white/5 leading-relaxed font-sans text-xs">
                                  {p.description}
                                </div>
                              </div>

                              {/* Side-by-Side/Toggle Original vs Mutated Code */}
                              <div className="space-y-2">
                                <span className="text-[10px] text-text-secondary font-bold">MUTATED CODE BLUEPRINT</span>
                                <div className="max-h-64 overflow-y-auto border border-white/10 rounded bg-slate-900 p-3 leading-relaxed select-all">
                                  <pre className="text-emerald-400 font-mono text-[10px] leading-tight">
                                    {p.mutated_code}
                                  </pre>
                                </div>
                              </div>

                              {/* Interactive Verification and Integration Controls */}
                              <div className="flex gap-3 pt-3 border-t border-white/5">
                                {p.status === 'pending' && (
                                  <button
                                    onClick={() => handleVerifyProposal(p.proposal_id)}
                                    disabled={isVerifying === p.proposal_id}
                                    className="flex-1 bg-indigo-600 hover:bg-indigo-700 text-white py-2.5 rounded font-mono uppercase tracking-wider text-[10px] font-bold transition-all flex items-center justify-center gap-2"
                                  >
                                    {isVerifying === p.proposal_id ? (
                                      <>
                                        <Loader2 className="animate-spin" size={12} />
                                        <span>Running Sandbox Verification...</span>
                                      </>
                                    ) : (
                                      <>
                                        <TestTube size={12} />
                                        <span>Verify in Evolutionary Sandbox</span>
                                      </>
                                    )}
                                  </button>
                                )}

                                {p.status === 'passed' && (
                                  <button
                                    onClick={() => handleIntegrateProposal(p.proposal_id)}
                                    disabled={isIntegrating === p.proposal_id}
                                    className="flex-1 bg-emerald-600 hover:bg-emerald-700 text-white py-2.5 rounded font-mono uppercase tracking-wider text-[10px] font-bold transition-all flex items-center justify-center gap-2 shadow-[0_0_10px_rgba(16,185,129,0.2)]"
                                  >
                                    {isIntegrating === p.proposal_id ? (
                                      <>
                                        <Loader2 className="animate-spin" size={12} />
                                        <span>Integrating Codebase...</span>
                                      </>
                                    ) : (
                                      <>
                                        <GitMerge size={12} />
                                        <span>Integrate Into Production</span>
                                      </>
                                    )}
                                  </button>
                                )}

                                {p.status === 'integrated' && (
                                  <div className="flex-1 py-2 text-center text-emerald-400 font-bold bg-emerald-500/10 border border-emerald-500/20 rounded font-mono uppercase text-[10px] flex items-center justify-center gap-2">
                                    <CheckCircle2 size={12} />
                                    <span>Integrated into Production Sandbox</span>
                                  </div>
                                )}

                                {p.status === 'failed' && (
                                  <div className="flex-1 py-2 text-center text-rose-400 font-bold bg-rose-500/10 border border-rose-500/20 rounded font-mono uppercase text-[10px] flex items-center justify-center gap-2">
                                    <XCircle size={12} />
                                    <span>Verification Failed - Rejected</span>
                                  </div>
                                )}
                              </div>
                            </div>
                          )}
                        </div>
                      );
                    })}

                    {evolutionData.proposals.length === 0 && (
                      <div className="text-center py-16 text-text-secondary opacity-40 border border-dashed border-white/10 rounded-lg">
                        <Orbit size={36} className="mx-auto mb-3 animate-spin" style={{ animationDuration: '10s' }} />
                        <p className="text-xs font-mono uppercase tracking-wider">No active genetic proposals</p>
                        <p className="text-[10px] mt-1 normal-case font-sans">Trigger a codebase mutation cycle on a target file to generate a blueprint proposal.</p>
                      </div>
                    )}
                  </div>
                </div>

              </div>
            </motion.div>
          )}

          {activeTab === 'architect' && (
            <motion.div key="architect" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
              <GenerativeArchitectStudio />
            </motion.div>
          )}

        </AnimatePresence>
      </main>

      {/* ── Chat History Modal ── */}
      {showHistoryModal && (
        <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-4">
          <div className="bg-background-secondary border border-white/10 rounded-2xl w-full max-w-2xl max-h-[80vh] flex flex-col shadow-2xl overflow-hidden">
            <div className="p-4 border-b border-white/10 flex justify-between items-center bg-white/5">
              <div className="flex items-center gap-2">
                <History className="text-accent-primary" size={18} />
                <h3 className="font-bold text-sm text-text-primary uppercase tracking-wider font-mono">Chat Conversation History Log</h3>
              </div>
              <div className="flex items-center gap-2">
                <button onClick={clearChatHistory} className="px-2.5 py-1 text-xs font-mono text-accent-warning hover:bg-accent-warning/10 rounded border border-accent-warning/20 transition-colors">
                  Clear History
                </button>
                <button onClick={() => setShowHistoryModal(false)} className="p-1 text-text-secondary hover:text-white rounded hover:bg-white/10">
                  <X size={18} />
                </button>
              </div>
            </div>

            <div className="p-4 flex-1 overflow-y-auto space-y-3 custom-scrollbar">
              {chatHistoryLogs.length > 0 ? chatHistoryLogs.map((log, idx) => (
                <div key={idx} className="p-3 bg-white/5 border border-white/5 rounded-xl font-mono text-xs space-y-1">
                  <div className="flex justify-between items-center text-[10px] text-text-secondary">
                    <span className="text-accent-primary font-bold">[{log.role}] {log.sender === 'user' ? 'OPERATOR' : (log.name || log.role)}</span>
                    <span>{new Date(log.timestamp).toLocaleString()}</span>
                  </div>
                  <p className="text-text-primary text-xs leading-relaxed whitespace-pre-wrap">{log.text}</p>
                </div>
              )) : (
                <div className="text-center py-12 text-text-secondary opacity-40">
                  <History size={48} className="mx-auto mb-3" />
                  <p className="text-sm font-mono">No stored conversation history found.</p>
                </div>
              )}
            </div>

            <div className="p-3 border-t border-white/10 bg-white/5 flex justify-end">
              <button onClick={() => setShowHistoryModal(false)} className="button-primary text-xs px-4 py-1.5">
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
