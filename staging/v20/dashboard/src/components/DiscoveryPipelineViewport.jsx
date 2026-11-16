import React, { useState, useEffect } from 'react';
import { 
  Dna, Beaker, GitMerge, Activity, ShieldCheck, 
  Layers, Play, RefreshCw, Zap, Award, CheckCircle2,
  FileCode, ArrowRight, Download, Crosshair, Search
} from 'lucide-react';
import axios from 'axios';

export default function DiscoveryPipelineViewport({ apiBase = 'http://127.0.0.1:8022' }) {
  const [pipelineStatus, setPipelineStatus] = useState(null);
  const [ampCandidates, setAmpCandidates] = useState([]);
  const [neoantigens, setNeoantigens] = useState([]);
  const [repurposedDrugs, setRepurposedDrugs] = useState([]);
  const [adcDesigns, setAdcDesigns] = useState([]);
  const [circuits, setCircuits] = useState([]);
  const [activeSubTab, setActiveSubTab] = useState('pipeline');
  const [loading, setLoading] = useState(true);
  const [executing, setExecuting] = useState(false);
  const [filterTarget, setFilterTarget] = useState('');

  const fetchAllData = async () => {
    try {
      setLoading(true);
      const [pipeRes, ampRes, neoRes, drugRes, adcRes, circRes] = await Promise.all([
        axios.get(`${apiBase}/discovery/pipeline/status`).catch(() => ({ data: null })),
        axios.get(`${apiBase}/discovery/amp-candidates`).catch(() => ({ data: { candidates: [] } })),
        axios.get(`${apiBase}/discovery/cancer-neoantigens`).catch(() => ({ data: { top_candidates: [] } })),
        axios.get(`${apiBase}/discovery/drug-repurposing/candidates`).catch(() => ({ data: { top_candidates: [] } })),
        axios.get(`${apiBase}/discovery/adc-designs`).catch(() => ({ data: { top_designs: [] } })),
        axios.get(`${apiBase}/discovery/synthetic-circuits`).catch(() => ({ data: { circuits: [] } }))
      ]);
      setPipelineStatus(pipeRes.data);
      setAmpCandidates(ampRes.data.candidates || []);
      setNeoantigens(neoRes.data.top_candidates || []);
      setRepurposedDrugs(drugRes.data.top_candidates || []);
      setAdcDesigns(adcRes.data.top_designs || []);
      setCircuits(circRes.data.circuits || []);
    } catch (err) {
      console.error("Failed to load discovery data", err);
    } finally {
      setLoading(false);
    }
  };

  const runPipelineCampaign = async () => {
    try {
      setExecuting(true);
      await axios.post(`${apiBase}/discovery/pipeline/run`, {
        campaign_name: "Autonomous_PHEIC_AMR_Master_Discovery"
      });
      await fetchAllData();
    } catch (err) {
      alert(`Discovery pipeline unavailable: ${err.response?.data?.message || err.message}`);
    } finally {
      setExecuting(false);
    }
  };

  useEffect(() => {
    fetchAllData();
  }, []);

  return (
    <div className="space-y-6 font-mono text-sm text-text-primary">
      {/* ── LCARS Header ── */}
      <div className="p-5 bg-background-secondary/80 border border-white/10 rounded-2xl backdrop-blur-md flex flex-wrap items-center justify-between gap-4 shadow-xl">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-xl bg-purple-500/20 border border-purple-500/40 flex items-center justify-center text-purple-400 shadow-[0_0_15px_rgba(168,85,247,0.3)]">
            <Dna size={24} />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-lg font-black tracking-wider uppercase text-white font-mono">
                DDR Discovery Engine & Bio-Pipeline (Tasks 11–20)
              </h2>
              <span className="px-2 py-0.5 text-[10px] rounded-full bg-purple-500/20 text-purple-300 border border-purple-500/30">
                DUAL-ROLE ANTIBODIES
              </span>
            </div>
            <p className="text-xs text-text-secondary mt-0.5">
              Monad Truth Anchor: <span className="text-accent-primary">0x4D4F4E4144000001</span> | 9 Autonomous Stages
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={fetchAllData}
            disabled={loading}
            className="px-3 py-2 bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl text-xs flex items-center gap-2 transition-colors text-text-secondary hover:text-white"
          >
            <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
            Refresh
          </button>
          <button
            onClick={runPipelineCampaign}
            disabled={executing}
            className="px-4 py-2 bg-purple-500/20 hover:bg-purple-500/30 border border-purple-500/40 text-purple-300 rounded-xl text-xs font-bold flex items-center gap-2 shadow-[0_0_20px_rgba(168,85,247,0.2)] transition-all"
          >
            {executing ? <RefreshCw size={14} className="animate-spin" /> : <Play size={14} />}
            Execute Discovery Campaign
          </button>
        </div>
      </div>

      {/* ── Sub-Tab Navigation ── */}
      <div className="flex items-center gap-2 border-b border-white/10 pb-3 overflow-x-auto">
        {[
          { id: 'pipeline', label: '01 MASTER PIPELINE (TASK 20)', icon: Activity },
          { id: 'amps', label: '02 AMP LEADS (TASK 11)', icon: Zap },
          { id: 'neoantigens', label: '03 CANCER NEOANTIGENS (TASK 17)', icon: Crosshair },
          { id: 'repurposed', label: '04 DRUG REPURPOSING (TASK 15)', icon: Beaker },
          { id: 'adcs', label: '05 ADC ARCHITECTURES (TASK 19)', icon: GitMerge },
          { id: 'circuits', label: '06 SYNTHETIC CIRCUITS (TASK 16)', icon: Layers },
        ].map(tab => {
          const TabIcon = tab.icon;
          const isActive = activeSubTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveSubTab(tab.id)}
              className={`px-3 py-2 rounded-xl text-xs font-bold flex items-center gap-2 whitespace-nowrap transition-all ${
                isActive
                  ? 'bg-purple-500/20 text-purple-300 border border-purple-500/40 shadow-[0_0_15px_rgba(168,85,247,0.2)]'
                  : 'bg-white/5 hover:bg-white/10 text-text-secondary hover:text-white border border-transparent'
              }`}
            >
              <TabIcon size={14} />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* ── 01 Master Pipeline View ── */}
      {activeSubTab === 'pipeline' && (
        <div className="space-y-6">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="p-4 bg-white/5 border border-white/10 rounded-xl">
              <div className="text-[10px] text-text-secondary uppercase">Campaign Status</div>
              <div className="text-base font-bold text-emerald-400 mt-1 flex items-center gap-2">
                <CheckCircle2 size={16} />
                {pipelineStatus?.pipeline_status || '—'}
              </div>
            </div>
            <div className="p-4 bg-white/5 border border-white/10 rounded-xl">
              <div className="text-[10px] text-text-secondary uppercase">Epitopes Screened</div>
              <div className="text-base font-bold text-accent-primary mt-1 flex items-center gap-2">
                <Dna size={16} />
                {pipelineStatus?.summary_metrics?.pathogen_epitopes_indexed ?? '—'} Signatures
              </div>
            </div>
            <div className="p-4 bg-white/5 border border-white/10 rounded-xl">
              <div className="text-[10px] text-text-secondary uppercase">AMP Leads Found</div>
              <div className="text-base font-bold text-cyan-400 mt-1 flex items-center gap-2">
                <Zap size={16} />
                {pipelineStatus?.summary_metrics?.amp_candidates_discovered ?? '—'} Nanomolar Leads
              </div>
            </div>
            <div className="p-4 bg-white/5 border border-white/10 rounded-xl">
              <div className="text-[10px] text-text-secondary uppercase">Peak ADC Affinity</div>
              <div className="text-base font-bold text-purple-400 mt-1 flex items-center gap-2">
                <Award size={16} />
                TI: — / Kd: —
              </div>
            </div>
          </div>

          <div className="p-5 bg-background-secondary/80 border border-white/10 rounded-2xl space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <span className="text-xs font-bold uppercase tracking-wider text-white">9-Stage Autonomous Campaign Pipeline</span>
              <span className="text-[10px] text-accent-primary font-mono">{pipelineStatus?.campaign_id || '—'}</span>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              {pipelineStatus?.stage_breakdowns && Object.entries(pipelineStatus.stage_breakdowns).map(([stg, info]) => (
                <div key={stg} className="p-3.5 bg-white/5 border border-white/5 rounded-xl space-y-1">
                  <div className="text-xs font-bold text-accent-primary uppercase truncate">{stg.replace(/_/g, ' ')}</div>
                  <div className="text-[11px] text-text-secondary">
                    {Object.entries(info).slice(0, 2).map(([k, v]) => (
                      <div key={k} className="flex justify-between">
                        <span className="capitalize">{k.replace(/_/g, ' ')}:</span>
                        <span className="text-white font-bold">{typeof v === 'object' ? JSON.stringify(v).slice(0, 15) : String(v)}</span>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* ── 02 AMP Candidates View ── */}
      {activeSubTab === 'amps' && (
        <div className="p-5 bg-background-secondary/80 border border-white/10 rounded-2xl space-y-4">
          <div className="flex items-center justify-between pb-3 border-b border-white/10">
            <div>
              <h3 className="text-sm font-bold text-white uppercase">Nanomolar Antimicrobial Peptides (AMPs)</h3>
              <p className="text-xs text-text-secondary">DDR Antibodies mapped to bactericidal peptide sequences</p>
            </div>
            <span className="text-xs px-2.5 py-1 rounded bg-accent-primary/20 text-accent-primary font-mono font-bold">
              {ampCandidates.length} Leads Grounded
            </span>
          </div>
          <div className="space-y-2 max-h-[500px] overflow-y-auto pr-1 custom-scrollbar">
            {ampCandidates.map((amp, idx) => (
              <div key={idx} className="p-3 bg-white/5 border border-white/5 rounded-xl flex items-center justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-accent-primary text-xs">{amp.candidate_id || amp.id || `AMP-${idx + 1}`}</span>
                    <span className="text-[10px] px-2 py-0.2 rounded bg-emerald-500/20 text-emerald-300">
                      Kd: {amp.predicted_kd_nm ?? amp.kd_nm ?? '—'} nM
                    </span>
                    <span className="text-[10px] text-text-secondary">{amp.target_protein || amp.target}</span>
                  </div>
                  <p className="text-[11px] font-mono text-text-secondary mt-1 truncate max-w-xl">
                    Seq: {amp.sequence || amp.peptide_sequence}
                  </p>
                </div>
                <div className="text-right">
                  <span className="tag text-[9px] tag-success">K3: —</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ── 03 Cancer Neoantigens View ── */}
      {activeSubTab === 'neoantigens' && (
        <div className="p-5 bg-background-secondary/80 border border-white/10 rounded-2xl space-y-4">
          <div className="flex items-center justify-between pb-3 border-b border-white/10">
            <div>
              <h3 className="text-sm font-bold text-white uppercase">Personalized Cancer Neoantigen Candidates</h3>
              <p className="text-xs text-text-secondary">HLA-restricted neoepitopes ranked by TAP transport & immunogenicity</p>
            </div>
            <span className="text-xs px-2.5 py-1 rounded bg-purple-500/20 text-purple-300 font-mono font-bold">
              {neoantigens.length} Epitopes Available
            </span>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-[500px] overflow-y-auto pr-1 custom-scrollbar">
            {neoantigens.map((neo, idx) => (
              <div key={idx} className="p-3 bg-white/5 border border-white/5 rounded-xl space-y-1">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-xs text-purple-300">{neo.gene} ({neo.mutation || neo.driver_mutation})</span>
                  <span className="text-[10px] px-2 py-0.5 rounded bg-white/10 text-white font-mono">{neo.hla_allele}</span>
                </div>
                <div className="text-[11px] font-mono text-accent-primary truncate">Peptide: {neo.peptide || neo.neoepitope}</div>
                <div className="flex items-center justify-between text-[10px] text-text-secondary pt-1">
                  <span>Kd: <strong className="text-white">{neo.affinity_nm || neo.binding_affinity_nm} nM</strong></span>
                  <span>Foreignness: <strong className="text-emerald-400">{neo.foreignness_score ?? '—'}</strong></span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ── 04 Drug Repurposing View ── */}
      {activeSubTab === 'repurposed' && (
        <div className="p-5 bg-background-secondary/80 border border-white/10 rounded-2xl space-y-4">
          <div className="flex items-center justify-between pb-3 border-b border-white/10">
            <div>
              <h3 className="text-sm font-bold text-white uppercase">Pharmacophore Match & Drug Repurposing</h3>
              <p className="text-xs text-text-secondary">FDA approved small-molecules docked against emerging virulence targets</p>
            </div>
            <span className="text-xs px-2.5 py-1 rounded bg-cyan-500/20 text-cyan-300 font-mono font-bold">
              {repurposedDrugs.length} Leads Screened
            </span>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-[500px] overflow-y-auto pr-1 custom-scrollbar">
            {repurposedDrugs.map((d, idx) => (
              <div key={idx} className="p-3 bg-white/5 border border-white/5 rounded-xl space-y-1">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-xs text-cyan-300">{d.drug_name}</span>
                  <span className="tag text-[8px] tag-success">Kd: {d.predicted_kd_nm ?? '—'} nM</span>
                </div>
                <div className="text-[11px] text-text-secondary">Target: <strong className="text-white">{d.target_name || d.target_symbol}</strong></div>
                <div className="text-[10px] text-text-secondary font-mono truncate">Mechanism: {d.repurposed_indication || 'Targeted Modulator'}</div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ── 05 ADC Designs View ── */}
      {activeSubTab === 'adcs' && (
        <div className="p-5 bg-background-secondary/80 border border-white/10 rounded-2xl space-y-4">
          <div className="flex items-center justify-between pb-3 border-b border-white/10">
            <div>
              <h3 className="text-sm font-bold text-white uppercase">Antibody-Drug Conjugate (ADC) Architectures</h3>
              <p className="text-xs text-text-secondary">Cleavable Val-Cit linkers, MMAE/DM1 warheads, DAR optimization</p>
            </div>
            <span className="text-xs px-2.5 py-1 rounded bg-amber-500/20 text-amber-300 font-mono font-bold">
              {adcDesigns.length} Engineered
            </span>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-[500px] overflow-y-auto pr-1 custom-scrollbar">
            {adcDesigns.map((adc, idx) => (
              <div key={idx} className="p-3 bg-white/5 border border-white/5 rounded-xl space-y-1">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-xs text-amber-300">{adc.adc_id || `ADC-ARCH-${idx+1}`}</span>
                  <span className="text-[10px] px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 font-mono">
                    TI: {adc.therapeutic_index ?? '—'}
                  </span>
                </div>
                <div className="text-[11px] text-text-secondary">Payload: <strong className="text-white">{adc.payload_warhead}</strong> (DAR: {adc.drug_to_antibody_ratio ?? '—'})</div>
                <div className="text-[10px] text-text-secondary font-mono">Linker: {adc.linker_type} | Target: {adc.target_antigen}</div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ── 06 Synthetic Circuits View ── */}
      {activeSubTab === 'circuits' && (
        <div className="p-5 bg-background-secondary/80 border border-white/10 rounded-2xl space-y-4">
          <div className="flex items-center justify-between pb-3 border-b border-white/10">
            <div>
              <h3 className="text-sm font-bold text-white uppercase">Synthetic Biology Genetic Logic Circuits</h3>
              <p className="text-xs text-text-secondary">Hill kinetic transfer functions compiled into physical DNA plasmid maps</p>
            </div>
            <span className="text-xs px-2.5 py-1 rounded bg-indigo-500/20 text-indigo-300 font-mono font-bold">
              {circuits.length} Circuits Compiled
            </span>
          </div>
          <div className="space-y-3 max-h-[500px] overflow-y-auto pr-1 custom-scrollbar">
            {circuits.map((c, idx) => (
              <div key={idx} className="p-3.5 bg-white/5 border border-white/5 rounded-xl space-y-2">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-xs text-indigo-300">{c.circuit_id}</span>
                    <span className="text-[10px] px-2 py-0.5 rounded bg-white/10 text-white font-mono">{c.logic_type} Gate</span>
                  </div>
                  <span className="text-[10px] text-emerald-400 font-mono font-bold">Dynamic Range: {c.dynamic_range_fold ? `${c.dynamic_range_fold}x` : '—'}</span>
                </div>
                <p className="text-xs text-text-secondary">{c.description}</p>
                <div className="bg-black/30 p-2 rounded text-[10px] font-mono text-accent-primary truncate">
                  Plasmid DNA ({c.plasmid_length_bp ?? '—'} bp): {c.dna_sequence?.slice(0, 70)}...
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
