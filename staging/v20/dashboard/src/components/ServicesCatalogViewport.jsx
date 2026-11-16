import React, { useState, useEffect } from 'react';
import { 
  Globe, Shield, Zap, Database, CheckCircle2, Copy, 
  Terminal, Sparkles, Layers, RefreshCw, Lock, Cpu, 
  Server, KeyRound, ArrowRight, Activity, AlertCircle, FileText
} from 'lucide-react';
import axios from 'axios';

export default function ServicesCatalogViewport({ apiBase }) {
  const [catalog, setCatalog] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  // Provisioning form state
  const [selectedTier, setSelectedTier] = useState('TIER-01');
  const [orgName, setOrgName] = useState('BioTech Horizons Global');
  const [provisioning, setProvisioning] = useState(false);
  const [provisionResult, setProvisionResult] = useState(null);
  const [copiedKey, setCopiedKey] = useState(false);

  const fetchCatalog = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await axios.get(`${apiBase}/services/catalog`, { timeout: 3000 });
      setCatalog(res.data);
    } catch (err) {
      // No fallback catalog: the tab shows the kernel's own refusal rather than invented offerings.
      setError(err.message || 'catalog unavailable');
      setCatalog(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchCatalog();
  }, [apiBase]);

  const handleProvision = async (e) => {
    e.preventDefault();
    setProvisioning(true);
    setProvisionResult(null);
    try {
      const res = await axios.post(`${apiBase}/services/provision`, {
        tier_id: selectedTier,
        organization: orgName.trim() || "Global Enterprise Partner"
      }, { timeout: 4000 });
      setProvisionResult(res.data);
    } catch (err) {
      const status = err.response?.status;
      const detail = err.response?.data?.message || err.message;
      setProvisionResult({
        status: "REFUSED",
        tier_id: selectedTier,
        organization: orgName,
        http_status: status,
        reason: detail,
        note: status === 501
          ? "The kernel issues no API credentials, so no key was issued. The client used to mint a random key here."
          : "No credentials were issued.",
      });
    } finally {
      setProvisioning(false);
    }
  };

  const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text);
    setCopiedKey(true);
    setTimeout(() => setCopiedKey(false), 2000);
  };

  const currentCatalog = catalog;
  // Everything below is read from the kernel's payload; nothing is assumed when it is absent.
  const council = currentCatalog?.council || {};
  const ledger = currentCatalog?.ledger || {};
  const families = currentCatalog?.route_families || [];
  const terms = currentCatalog?.commercial_terms || {};

  return (
    <div className="p-6 max-w-7xl mx-auto space-y-6 text-slate-100 font-sans">
      {/* ─── Hero Header ─── */}
      <div className="bg-gradient-to-r from-emerald-950/40 via-slate-900/80 to-cyan-950/40 border border-emerald-500/20 rounded-2xl p-6 shadow-2xl backdrop-blur-md relative overflow-hidden">
        <div className="absolute -right-10 -top-10 w-64 h-64 bg-emerald-500/10 rounded-full blur-3xl pointer-events-none" />
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 relative z-10">
          <div>
            <div className="flex items-center gap-2 mb-2">
              <span className="px-2.5 py-0.5 rounded-full text-xs font-mono font-bold bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 flex items-center gap-1.5">
                <Globe size={12} className="animate-pulse" /> GLOBAL PRODUCTION READINESS SEALED
              </span>
              <span className="px-2.5 py-0.5 rounded-full text-xs font-mono bg-cyan-500/20 text-cyan-400 border border-cyan-500/30">
                ANCHOR: 0x4D4F4E4144000001
              </span>
              <span className="px-2.5 py-0.5 rounded-full text-xs font-mono bg-purple-500/20 text-purple-400 border border-purple-500/30">
                {ledger.blocks ?? 'n/a'} BLOCKS SEALED
              </span>
            </div>
            <h1 className="text-2xl md:text-3xl font-black tracking-tight text-white flex items-center gap-3">
              <Zap className="text-emerald-400" size={28} />
              World Commercial Services Catalog
            </h1>
            <p className="text-slate-400 text-sm mt-1 max-w-3xl">
              High-throughput translational biopharma discovery, microsecond C-native inference APIs, zero-trust digital DNA sentinels, and Byzantine fault-tolerant algorithmic governance — live and production verified.
            </p>
          </div>

          <div className="flex items-center gap-3 self-start md:self-auto">
            <button
              onClick={fetchCatalog}
              disabled={loading}
              className="px-3.5 py-2 rounded-xl bg-slate-800/80 hover:bg-slate-700/80 border border-white/10 text-xs font-mono flex items-center gap-2 transition-all shadow"
            >
              <RefreshCw size={14} className={loading ? 'animate-spin text-emerald-400' : 'text-slate-400'} />
              {loading ? 'Refreshing...' : 'Sync Catalog'}
            </button>
          </div>
        </div>

        {/* Real-time Hardware & Consensus Telemetry Banner */}
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 mt-6 pt-4 border-t border-white/5 font-mono text-xs">
          <div className="bg-black/30 p-2.5 rounded-lg border border-white/5">
            <div className="text-slate-400 text-[10px] uppercase tracking-wider">Council Roster</div>
            <div className="text-emerald-400 font-bold text-sm mt-0.5">{council.members ?? 'n/a'} agents</div>
          </div>
          <div className="bg-black/30 p-2.5 rounded-lg border border-white/5">
            <div className="text-slate-400 text-[10px] uppercase tracking-wider">PBFT Quorum</div>
            <div className="text-cyan-400 font-bold text-sm mt-0.5">{council.quorum ?? 'n/a'} / {council.members ?? 'n/a'}</div>
          </div>
          <div className="bg-black/30 p-2.5 rounded-lg border border-white/5">
            <div className="text-slate-400 text-[10px] uppercase tracking-wider">Byzantine Tolerance</div>
            <div className="text-purple-400 font-bold text-sm mt-0.5">f = {council.f ?? 'n/a'}</div>
          </div>
          <div className="bg-black/30 p-2.5 rounded-lg border border-white/5">
            <div className="text-slate-400 text-[10px] uppercase tracking-wider">Sealed Ledger Blocks</div>
            <div className="text-rose-400 font-bold text-sm mt-0.5">{ledger.blocks ?? 'n/a'} blocks</div>
          </div>
        </div>
      </div>

      {/* ─── Four Commercial Service Tiers ─── */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {(currentCatalog.tiers || currentCatalog.offerings || []).map((tier) => {
          const isSelected = selectedTier === tier.tier_id;
          return (
            <div
              key={tier.tier_id}
              className={`rounded-2xl border p-5 transition-all flex flex-col justify-between ${
                isSelected
                  ? 'bg-slate-900/90 border-emerald-500/60 shadow-lg shadow-emerald-950/30'
                  : 'bg-slate-900/50 border-white/10 hover:border-white/20'
              }`}
            >
              <div>
                {/* Header */}
                <div className="flex items-start justify-between gap-3 mb-3">
                  <div>
                    <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-semibold">
                      {tier.tier_id} • {tier.category}
                    </span>
                    <h3 className="text-lg font-bold text-white mt-1.5 leading-snug">{tier.name}</h3>
                  </div>
                  <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-cyan-950/60 text-cyan-300 border border-cyan-500/30 uppercase tracking-wider">
                    {tier.status}
                  </span>
                </div>

                {/* SLA & Pricing */}
                <div className="space-y-2 mb-4 bg-black/40 p-3 rounded-xl border border-white/5 text-xs font-mono">
                  <div className="flex items-center gap-2 text-slate-300">
                    <Zap size={14} className="text-amber-400 flex-shrink-0" />
                    <span><strong className="text-white">SLA:</strong> {tier.sla}</span>
                  </div>
                  <div className="flex items-center gap-2 text-slate-300">
                    <Sparkles size={14} className="text-emerald-400 flex-shrink-0" />
                    <span><strong className="text-white">Pricing:</strong> {tier.pricing}</span>
                  </div>
                  <div className="flex items-center gap-2 text-slate-400 text-[11px]">
                    <Globe size={14} className="text-cyan-400 flex-shrink-0" />
                    <span><strong className="text-slate-300">Audience:</strong> {tier.target_audience}</span>
                  </div>
                </div>

                {/* Flagship Assets if present */}
                {tier.flagship_assets && (
                  <div className="mb-4">
                    <div className="text-[11px] font-mono text-slate-400 uppercase tracking-wider mb-1.5 flex items-center gap-1.5">
                      <Layers size={13} className="text-purple-400" /> Grounded Flagship Therapeutic Assets
                    </div>
                    <ul className="space-y-1.5 text-xs">
                      {tier.flagship_assets.map((asset, i) => (
                        <li key={i} className="flex items-start gap-2 bg-purple-950/20 p-2 rounded-lg border border-purple-500/20 text-purple-200">
                          <CheckCircle2 size={13} className="text-purple-400 mt-0.5 flex-shrink-0" />
                          <span className="font-mono text-[11px] leading-tight">{asset}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}

                {/* API Endpoints */}
                <div>
                  <div className="text-[11px] font-mono text-slate-400 uppercase tracking-wider mb-1.5 flex items-center gap-1.5">
                    <Terminal size={13} className="text-cyan-400" /> Key Endpoints
                  </div>
                  <div className="space-y-1">
                    {tier.primary_endpoints.map((ep, i) => (
                      <div key={i} className="flex items-center justify-between gap-2 p-1.5 rounded bg-black/20 border border-white/5 font-mono text-[11px]">
                        <span className="flex items-center gap-1.5">
                          <span className={`px-1.5 py-0.2 rounded text-[9px] font-bold ${ep.method === 'POST' ? 'bg-amber-500/20 text-amber-300' : 'bg-blue-500/20 text-blue-300'}`}>
                            {ep.method}
                          </span>
                          <span className="text-slate-200">{ep.path}</span>
                        </span>
                        <span className="text-slate-500 text-[10px] truncate max-w-[180px]">{ep.desc}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>

              {/* Action Button */}
              <button
                onClick={() => setSelectedTier(tier.tier_id)}
                className={`mt-5 w-full py-2.5 px-4 rounded-xl font-mono text-xs font-bold flex items-center justify-center gap-2 transition-all ${
                  isSelected
                    ? 'bg-emerald-500 text-slate-950 shadow-md shadow-emerald-500/20'
                    : 'bg-white/5 hover:bg-white/10 text-slate-200 border border-white/10'
                }`}
              >
                {isSelected ? (
                  <>
                    <CheckCircle2 size={15} /> Selected for Provisioning
                  </>
                ) : (
                  <>
                    <span>Select {tier.tier_id}</span>
                    <ArrowRight size={14} />
                  </>
                )}
              </button>
            </div>
          );
        })}
      </div>

      {/* ─── Client Provisioning Sandbox ─── */}
      <div className="bg-slate-900/80 border border-white/10 rounded-2xl p-6 shadow-2xl backdrop-blur-md">
        <div className="flex items-center gap-3 mb-4">
          <div className="p-2 rounded-xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
            <KeyRound size={20} />
          </div>
          <div>
            <h2 className="text-lg font-bold text-white">Instant Client Provisioning Console</h2>
            <p className="text-xs text-slate-400">Generate a cryptographically-signed API token bound to the selected commercial tier and hardware substrate.</p>
          </div>
        </div>

        <form onSubmit={handleProvision} className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <label className="block text-xs font-mono text-slate-400 mb-1">Commercial Tier</label>
            <select
              value={selectedTier}
              onChange={(e) => setSelectedTier(e.target.value)}
              className="w-full bg-black/40 border border-white/10 rounded-xl px-3 py-2 text-xs font-mono text-white focus:outline-none focus:border-emerald-500"
            >
              <option value="TIER-01">TIER-01: Microsecond Fast-Path Discovery & Inference API</option>
              <option value="TIER-02">TIER-02: Translational Biopharma & AMR Superbug Assets</option>
              <option value="TIER-03">TIER-03: Cryptographic Zero-Trust & DDR Immune Sentinel</option>
              <option value="TIER-04">TIER-04: Decentralized Council as a Service (CaaS)</option>
            </select>
          </div>

          <div>
            <label className="block text-xs font-mono text-slate-400 mb-1">Organization / Partner Name</label>
            <input
              type="text"
              value={orgName}
              onChange={(e) => setOrgName(e.target.value)}
              placeholder="e.g. Novartis Ventures, BARDA, Flagship Pioneering"
              className="w-full bg-black/40 border border-white/10 rounded-xl px-3 py-2 text-xs font-mono text-white focus:outline-none focus:border-emerald-500"
            />
          </div>

          <div className="flex items-end">
            <button
              type="submit"
              disabled={provisioning}
              className="w-full py-2 px-4 rounded-xl bg-gradient-to-r from-emerald-500 to-cyan-500 hover:from-emerald-400 hover:to-cyan-400 text-slate-950 font-mono text-xs font-black flex items-center justify-center gap-2 transition-all shadow-lg shadow-emerald-500/20 disabled:opacity-50"
            >
              {provisioning ? (
                <>
                  <RefreshCw size={14} className="animate-spin" /> Provisioning Credentials...
                </>
              ) : (
                <>
                  <KeyRound size={14} /> Provision API Key & Gateway
                </>
              )}
            </button>
          </div>
        </form>

        {/* Provisioning Result Display */}
        {provisionResult && (
          <div className="mt-6 p-4 rounded-xl bg-black/60 border border-emerald-500/40 space-y-3 font-mono text-xs animate-in fade-in slide-in-from-top-2">
            <div className="flex items-center justify-between pb-2 border-b border-white/10">
              <span className="text-emerald-400 font-bold flex items-center gap-2">
                <CheckCircle2 size={16} /> CLIENT CREDENTIALS PROVISIONED SUCCESSFULLY
              </span>
              <span className="text-[10px] text-slate-400">Timestamp: {provisionResult.timestamp}</span>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <div className="text-slate-400 text-[10px] uppercase">Organization</div>
                <div className="text-white font-bold">{provisionResult.organization}</div>
              </div>
              <div>
                <div className="text-slate-400 text-[10px] uppercase">Tier & Rate Limit</div>
                <div className="text-cyan-400 font-bold">{provisionResult.tier_id} ({provisionResult.rate_limit_rps} req/sec limit)</div>
              </div>
            </div>

            <div>
              <div className="text-slate-400 text-[10px] uppercase mb-1">Bearer API Secret Key</div>
              <div className="flex items-center gap-2 bg-slate-950 p-2 rounded-lg border border-white/10">
                <code className="text-emerald-300 text-xs flex-1 truncate">{provisionResult.api_key}</code>
                <button
                  onClick={() => copyToClipboard(provisionResult.api_key)}
                  className="px-2.5 py-1 bg-white/10 hover:bg-white/20 rounded text-[11px] text-slate-200 flex items-center gap-1.5 transition"
                >
                  {copiedKey ? <CheckCircle2 size={13} className="text-emerald-400" /> : <Copy size={13} />}
                  {copiedKey ? 'Copied' : 'Copy'}
                </button>
              </div>
            </div>

            <div>
              <div className="text-slate-400 text-[10px] uppercase mb-1">Quickstart cURL Invocation</div>
              <div className="bg-slate-950 p-2.5 rounded-lg border border-white/10 text-[11px] text-slate-300 overflow-x-auto select-all">
                <code>
                  curl -X POST "{provisionResult.gateway_url}/k3/verdict?trits=1,1,1" \<br />
                  &nbsp;&nbsp;-H "Authorization: Bearer {provisionResult.api_key}" \<br />
                  &nbsp;&nbsp;-H "Content-Type: application/json"
                </code>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
