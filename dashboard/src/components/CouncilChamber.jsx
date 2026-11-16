import React, { useState, useEffect, useRef, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Users, Shield, Brain, Cpu, Radio, Eye, Search, Layers, Activity, Zap, GitBranch, Mail, CheckCircle2, XCircle, AlertTriangle, Send, ChevronRight, ChevronDown, Check, X, ShieldAlert, KeyRound, Bug, LayoutDashboard, Globe } from 'lucide-react';
import axios from 'axios';

const API_BASE = '';

const AGENT_ROLES = {
  'Hermes': { icon: Mail, role: 'comms' },
  'Jarvis': { icon: Brain, role: 'assistant' },
  'Bridge': { icon: Globe, role: 'APIs/data' },
  'Shield': { icon: Shield, role: 'security' },
  'Logic': { icon: Cpu, role: 'reasoning' },
  'Devo': { icon: GitBranch, role: 'code' },
  'Pulse': { icon: Activity, role: 'monitoring' },
  'Flow': { icon: Zap, role: 'workflow' },
  'EventLedger': { icon: Layers, role: 'audit' },
  'Orchestra': { icon: LayoutDashboard, role: 'orchestration' },
  'THzGateway': { icon: Radio, role: 'spectrum' },
  'Verify': { icon: CheckCircle2, role: 'truth' },
  'Seeker': { icon: Search, role: 'research' },
  'Archi': { icon: Eye, role: 'architecture' }
};

const CATEGORIES = ['Security', 'Architecture', 'Code', 'Data', 'Infrastructure', 'Research', 'Protocol'];

const CouncilChamber = () => {
  const [status, setStatus] = useState({ members: [], active_proposals: [], stats: {} });
  const [proposals, setProposals] = useState([]);
  const [activeProposalId, setActiveProposalId] = useState(null);
  const [activeDeliberation, setActiveDeliberation] = useState(null);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [formData, setFormData] = useState({ topic: '', category: 'Architecture', description: '', proposer: 'Jarvis' });
  const feedEndRef = useRef(null);

  const fetchStatus = useCallback(async () => {
    try {
      const res = await axios.get(`${API_BASE}/council/status`);
      setStatus(res.data);
      if (res.data.active_proposals?.length > 0 && !activeProposalId) {
        setActiveProposalId(res.data.active_proposals[0].id);
      }
    } catch (err) {
      console.error('Failed to fetch council status:', err);
    }
  }, [activeProposalId]);

  const fetchProposals = useCallback(async () => {
    try {
      const res = await axios.get(`${API_BASE}/council/proposals`);
      setProposals(res.data.proposals || []);
    } catch (err) {
      console.error('Failed to fetch proposals:', err);
    }
  }, []);

  const fetchActiveDeliberation = useCallback(async () => {
    if (!activeProposalId) return;
    try {
      const res = await axios.get(`${API_BASE}/council/proposal/${activeProposalId}`);
      setActiveDeliberation(res.data);
    } catch (err) {
      console.error('Failed to fetch deliberation:', err);
    }
  }, [activeProposalId]);

  useEffect(() => {
    fetchStatus();
    fetchProposals();
    const interval = setInterval(() => {
      fetchStatus();
      if (activeProposalId) fetchActiveDeliberation();
    }, 3000);
    return () => clearInterval(interval);
  }, [fetchStatus, fetchProposals, activeProposalId, fetchActiveDeliberation]);

  useEffect(() => {
    if (feedEndRef.current) {
      feedEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [activeDeliberation?.feed]);

  const handlePropose = async (e) => {
    e.preventDefault();
    try {
      await axios.post(`${API_BASE}/council/propose`, formData);
      setFormData({ topic: '', category: 'Architecture', description: '', proposer: 'Jarvis' });
      setIsFormOpen(false);
      fetchStatus();
      fetchProposals();
    } catch (err) {
      console.error('Propose failed:', err);
    }
  };

  const handleAction = async (action, id) => {
    try {
      await axios.post(`${API_BASE}/council/${action}/${id}`);
      fetchActiveDeliberation();
    } catch (err) {
      console.error(`Action ${action} failed:`, err);
    }
  };

  const getVoteColor = (vote) => {
    if (vote === 1) return 'text-[#33cc66] border-[#33cc66] bg-[#33cc66]/10';
    if (vote === -1) return 'text-[#cc3333] border-[#cc3333] bg-[#cc3333]/10';
    if (vote === 0) return 'text-[#ffaa00] border-[#ffaa00] bg-[#ffaa00]/10';
    return 'text-gray-400 border-gray-400 bg-gray-400/10';
  };

  const phases = ['PROPOSED', 'DELIBERATING', 'VOTING', 'CONSENSUS', 'ENACTED'];
  const currentPhaseIndex = phases.indexOf(activeDeliberation?.phase || 'PROPOSED');

  const getConsensusColor = (score) => {
    if (score > 0.5) return '#33cc66';
    if (score < -0.5) return '#cc3333';
    return '#ffaa00';
  };

  return (
    <div className="h-full bg-[#0a0a12] text-white p-4 font-mono overflow-y-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-6 border-b border-white/10 pb-4">
        <div className="flex items-center gap-3">
          <Users className="text-[#ff9900]" size={28} />
          <h1 className="text-2xl font-bold tracking-widest text-[#ff9900]">SWARM COUNCIL CHAMBER</h1>
        </div>
        <div className="flex gap-4">
          <div className="px-3 py-1 bg-white/5 border border-white/10 rounded flex items-center gap-2">
            <div className={`w-2 h-2 rounded-full ${status.members.length > 0 ? 'bg-[#33cc66] animate-pulse' : 'bg-[#cc3333]'}`} />
            <span className="text-xs text-accent-primary">LINK ACTIVE</span>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Column: Agents & Consensus */}
        <div className="lg:col-span-1 space-y-6">
          
          <div className="lcars-card bg-white/5 border border-white/10 rounded-xl p-4">
            <h2 className="text-[#66ccff] text-sm tracking-wider mb-4 border-b border-white/10 pb-2">COUNCIL MEMBERS<span className="ml-2 text-[9px] font-normal text-[#8899aa]">confidence only while a deliberation is open</span></h2>
            <div className="grid grid-cols-2 gap-3">
              {Object.keys(AGENT_ROLES).map((agentName) => {
                const agentDef = AGENT_ROLES[agentName];
                const Icon = agentDef.icon;
                const activeVote = activeDeliberation?.votes?.[agentName];
                const voteVal = activeVote?.vote;
                const confVal = activeVote?.confidence ?? null;
                
                return (
                  <motion.div 
                    key={agentName}
                    layout
                    className={`p-2 rounded border flex flex-col gap-2 ${getVoteColor(voteVal)} transition-colors`}
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <Icon size={14} />
                        <span className="text-xs font-bold">{agentName}</span>
                      </div>
                      {voteVal === 1 && <Check size={14} />}
                      {voteVal === -1 && <X size={14} />}
                      {voteVal === 0 && <AlertTriangle size={14} />}
                    </div>
                    <div className="flex justify-between items-end opacity-80">
                      <span className="text-[10px] truncate w-16">{agentDef.role}</span>
                      <span className="text-[10px]">{confVal != null ? Math.round(confVal * 100) + '%' : 'no vote'}</span>
                    </div>
                  </motion.div>
                );
              })}
            </div>
          </div>

          {/* Consensus Meter */}
          <div className="lcars-card bg-white/5 border border-white/10 rounded-xl p-4">
            <h2 className="text-[#ff9900] text-sm tracking-wider mb-4 border-b border-white/10 pb-2">CONSENSUS METER</h2>
            {activeDeliberation ? (
              <div className="space-y-4">
                <div className="flex justify-between text-xs mb-1">
                  <span className="text-[#cc3333]">REJECT</span>
                  <span className="text-[#ffaa00]">ABSTAIN</span>
                  <span className="text-[#33cc66]">APPROVE</span>
                </div>
                <div className="h-4 bg-black/50 rounded-full overflow-hidden flex relative">
                  <div className="absolute left-1/2 top-0 bottom-0 w-px bg-white/30 z-10" />
                  
                  {/* Visualizing Consensus from -1 to 1 */}
                  <motion.div 
                    className="absolute h-full"
                    animate={{ 
                      width: `${Math.abs(activeDeliberation.consensus_score || 0) * 50}%`,
                      left: (activeDeliberation.consensus_score || 0) < 0 
                        ? `${50 - Math.abs(activeDeliberation.consensus_score || 0) * 50}%` 
                        : '50%',
                      backgroundColor: getConsensusColor(activeDeliberation.consensus_score || 0)
                    }}
                    transition={{ duration: 0.5 }}
                  />
                </div>
                <div className="text-center font-bold text-xl mt-2" style={{ color: getConsensusColor(activeDeliberation.consensus_score || 0) }}>
                  {((activeDeliberation.consensus_score || 0) * 100).toFixed(1)}%
                </div>
                
                {/* Tally */}
                <div className="flex gap-2 justify-center text-xs mt-2">
                  <span className="bg-[#33cc66]/20 text-[#33cc66] px-2 py-1 rounded border border-[#33cc66]/30">
                    A: {activeDeliberation.tally?.approve || 0}
                  </span>
                  <span className="bg-[#cc3333]/20 text-[#cc3333] px-2 py-1 rounded border border-[#cc3333]/30">
                    R: {activeDeliberation.tally?.reject || 0}
                  </span>
                  <span className="bg-[#ffaa00]/20 text-[#ffaa00] px-2 py-1 rounded border border-[#ffaa00]/30">
                    Abs: {activeDeliberation.tally?.abstain || 0}
                  </span>
                </div>
              </div>
            ) : (
              <div className="text-gray-500 text-xs text-center py-4">No active deliberation</div>
            )}
          </div>
        </div>

        {/* Center & Right Column: Active Proposal & Deliberation */}
        <div className="lg:col-span-2 flex flex-col gap-6">
          
          {/* Active Proposal */}
          {activeDeliberation ? (
            <div className="lcars-card bg-white/5 border border-[#7c4dff]/40 rounded-xl p-5 shadow-[0_0_15px_rgba(124,77,255,0.1)] relative">
              <div className="absolute top-0 right-0 bg-[#7c4dff]/20 text-[#7c4dff] text-[10px] px-2 py-1 rounded-bl-lg rounded-tr-xl border-l border-b border-[#7c4dff]/40">
                ACTIVE PROPOSAL
              </div>
              <h2 className="text-2xl font-bold text-white mb-2">{activeDeliberation.topic}</h2>
              <div className="flex gap-3 mb-4">
                <span className="text-xs bg-white/10 px-2 py-1 rounded text-[#66ccff]">
                  Cat: {activeDeliberation.category}
                </span>
                <span className="text-xs bg-white/10 px-2 py-1 rounded text-[#ff9900]">
                  By: {activeDeliberation.proposer}
                </span>
              </div>
              <p className="text-sm text-gray-300 mb-6 bg-black/30 p-3 rounded border border-white/5 min-h-[60px]">
                {activeDeliberation.description}
              </p>

              {/* Phase Progress */}
              <div className="flex items-center justify-between mb-4 mt-2 px-2 relative">
                <div className="absolute left-0 right-0 top-1/2 h-px bg-white/10 -z-10" />
                {phases.map((phase, idx) => (
                  <div key={phase} className="flex flex-col items-center gap-1 z-10 bg-[#0a0a12] px-2">
                    <div className={`w-4 h-4 rounded-full border-2 flex items-center justify-center 
                      ${idx <= currentPhaseIndex ? 'border-[#33cc66] bg-[#33cc66]/20' : 'border-gray-600 bg-black'}`}>
                      {idx < currentPhaseIndex && <Check size={10} className="text-[#33cc66]" />}
                      {idx === currentPhaseIndex && <div className="w-1.5 h-1.5 bg-[#33cc66] rounded-full animate-pulse" />}
                    </div>
                    <span className={`text-[9px] tracking-wider ${idx <= currentPhaseIndex ? 'text-[#33cc66]' : 'text-gray-500'}`}>
                      {phase}
                    </span>
                  </div>
                ))}
              </div>

              {/* Manual Triggers (Mocking the orchestration if needed) */}
              <div className="flex justify-end gap-2 mt-4 pt-4 border-t border-white/10">
                <button 
                  onClick={() => handleAction('deliberate', activeDeliberation.id)}
                  disabled={currentPhaseIndex > 0}
                  className="px-3 py-1 bg-white/10 hover:bg-[#66ccff]/20 text-[#66ccff] rounded text-xs transition-colors disabled:opacity-30 border border-transparent hover:border-[#66ccff]/50">
                  Trigger Deliberation
                </button>
                <button 
                  onClick={() => handleAction('vote', activeDeliberation.id)}
                  disabled={currentPhaseIndex > 1}
                  className="px-3 py-1 bg-white/10 hover:bg-[#ff9900]/20 text-[#ff9900] rounded text-xs transition-colors disabled:opacity-30 border border-transparent hover:border-[#ff9900]/50">
                  Trigger Vote
                </button>
                <button 
                  onClick={() => handleAction('enact', activeDeliberation.id)}
                  disabled={currentPhaseIndex > 3}
                  className="px-3 py-1 bg-[#7c4dff]/20 hover:bg-[#7c4dff]/40 text-white rounded text-xs transition-colors disabled:opacity-30 border border-[#7c4dff]/50">
                  Enact Consensus
                </button>
              </div>
            </div>
          ) : (
            <div className="lcars-card bg-white/5 border border-white/10 rounded-xl p-8 flex flex-col items-center justify-center text-gray-500 h-[300px]">
              <AlertTriangle size={32} className="mb-4 opacity-50" />
              <p>NO ACTIVE PROPOSALS IN THE CHAMBER</p>
            </div>
          )}

          {/* Deliberation Feed */}
          <div className="lcars-card bg-white/5 border border-white/10 rounded-xl p-4 flex-1 min-h-[300px] flex flex-col">
            <h2 className="text-[#66ccff] text-sm tracking-wider mb-3 flex items-center gap-2">
              <Activity size={16} /> DELIBERATION FEED
            </h2>
            <div className="flex-1 bg-black/40 rounded border border-white/5 p-3 overflow-y-auto space-y-3 relative">
              <AnimatePresence>
                {activeDeliberation?.feed?.map((item, idx) => (
                  <motion.div 
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    key={idx} 
                    className="text-xs bg-white/5 p-2 rounded border border-white/5"
                  >
                    <div className="flex justify-between items-center mb-1 text-[10px]">
                      <span className="font-bold text-[#ff9900]">{item.agent}</span>
                      <span className="text-gray-500">{new Date(item.timestamp).toLocaleTimeString()}</span>
                    </div>
                    <p className="text-gray-300">{item.comment}</p>
                  </motion.div>
                ))}
              </AnimatePresence>
              <div ref={feedEndRef} />
              {!activeDeliberation?.feed?.length && (
                <div className="absolute inset-0 flex items-center justify-center text-gray-600 text-xs">
                  Waiting for agent insights...
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Bottom Row: Propose Form & History */}
      <div className="mt-6 grid grid-cols-1 lg:grid-cols-2 gap-6">
        
        {/* Submit Form */}
        <div className="lcars-card bg-white/5 border border-white/10 rounded-xl overflow-hidden">
          <button 
            onClick={() => setIsFormOpen(!isFormOpen)}
            className="w-full p-4 flex items-center justify-between text-[#ff9900] hover:bg-white/5 transition-colors border-b border-white/10"
          >
            <div className="flex items-center gap-2 text-sm tracking-wider">
              <Send size={16} /> SUBMIT NEW PROPOSAL
            </div>
            {isFormOpen ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
          </button>
          
          <AnimatePresence>
            {isFormOpen && (
              <motion.div 
                initial={{ height: 0 }}
                animate={{ height: 'auto' }}
                exit={{ height: 0 }}
                className="overflow-hidden"
              >
                <form onSubmit={handlePropose} className="p-4 space-y-4">
                  <div>
                    <label className="block text-[10px] text-[#66ccff] mb-1">TOPIC</label>
                    <input 
                      required
                      value={formData.topic}
                      onChange={e => setFormData({...formData, topic: e.target.value})}
                      className="w-full bg-black/50 border border-white/10 rounded p-2 text-sm text-white focus:border-[#66ccff] outline-none" 
                    />
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-[10px] text-[#66ccff] mb-1">CATEGORY</label>
                      <select 
                        value={formData.category}
                        onChange={e => setFormData({...formData, category: e.target.value})}
                        className="w-full bg-black/50 border border-white/10 rounded p-2 text-sm text-white focus:border-[#66ccff] outline-none"
                      >
                        {CATEGORIES.map(c => <option key={c} value={c}>{c}</option>)}
                      </select>
                    </div>
                    <div>
                      <label className="block text-[10px] text-[#66ccff] mb-1">PROPOSER</label>
                      <select 
                        value={formData.proposer}
                        onChange={e => setFormData({...formData, proposer: e.target.value})}
                        className="w-full bg-black/50 border border-white/10 rounded p-2 text-sm text-white focus:border-[#66ccff] outline-none"
                      >
                        {Object.keys(AGENT_ROLES).map(c => <option key={c} value={c}>{c}</option>)}
                      </select>
                    </div>
                  </div>
                  <div>
                    <label className="block text-[10px] text-[#66ccff] mb-1">DESCRIPTION / REASONING</label>
                    <textarea 
                      required
                      rows={3}
                      value={formData.description}
                      onChange={e => setFormData({...formData, description: e.target.value})}
                      className="w-full bg-black/50 border border-white/10 rounded p-2 text-sm text-white focus:border-[#66ccff] outline-none" 
                    />
                  </div>
                  <button type="submit" className="w-full py-2 bg-[#ff9900]/20 hover:bg-[#ff9900]/30 border border-[#ff9900]/50 text-[#ff9900] rounded text-sm transition-colors mt-2">
                    SUBMIT TO COUNCIL
                  </button>
                </form>
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        {/* History */}
        <div className="lcars-card bg-white/5 border border-white/10 rounded-xl p-4">
          <h2 className="text-gray-400 text-sm tracking-wider mb-4 border-b border-white/10 pb-2">PROPOSAL HISTORY</h2>
          <div className="space-y-2 max-h-[200px] overflow-y-auto pr-2">
            {proposals.length > 0 ? proposals.filter(p => p.id !== activeProposalId).map(p => (
              <div 
                key={p.id} 
                className="flex items-center justify-between p-2 rounded bg-black/30 border border-white/5 hover:border-white/20 cursor-pointer transition-colors"
                onClick={() => setActiveProposalId(p.id)}
              >
                <div>
                  <div className="text-xs text-white">{p.topic}</div>
                  <div className="text-[10px] text-gray-500">{p.category} • {p.phase}</div>
                </div>
                {p.consensus_score !== undefined && (
                  <div className={`text-xs px-2 py-1 rounded border ${
                    p.consensus_score > 0 ? 'text-[#33cc66] border-[#33cc66]/30 bg-[#33cc66]/10' : 
                    p.consensus_score < 0 ? 'text-[#cc3333] border-[#cc3333]/30 bg-[#cc3333]/10' : 
                    'text-[#ffaa00] border-[#ffaa00]/30 bg-[#ffaa00]/10'
                  }`}>
                    {(p.consensus_score > 0 ? 'PASSED ' : p.consensus_score < 0 ? 'REJECTED ' : 'MIXED ')} 
                    {Math.round(Math.abs(p.consensus_score) * 100)}%
                  </div>
                )}
              </div>
            )) : (
              <div className="text-xs text-gray-600 text-center py-4">No historical proposals found</div>
            )}
          </div>
        </div>

      </div>
    </div>
  );
};

export default CouncilChamber;
