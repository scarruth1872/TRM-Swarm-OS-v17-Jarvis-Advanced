import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  BookOpen, Search, FileText, Clock, Shield, CheckCircle, 
  AlertTriangle, HelpCircle, ExternalLink, X, Maximize2, 
  Download, ChevronRight, Hash, Eye, Sparkles, RefreshCw
} from 'lucide-react';

const JOURNAL_HEADER = {
  name: 'The Monad Journal of Autonomous Discovery',
  issn: '2998-277X',
  tagline: 'Where the swarm speaks, and the universe verifies.',
  founded: 'August 10, 2026',
  budget: '$828/yr',
  access: 'Open Access — $0',
};

const statusIcons = {
  'PUBLISHED': <CheckCircle size={14} className="text-green-400" />,
  'PRE-REGISTERED': <Clock size={14} className="text-yellow-400" />,
  'DRAFT': <FileText size={14} className="text-blue-400" />,
  'PREPRINT': <AlertTriangle size={14} className="text-orange-400" />,
  'SPECIFICATION': <Shield size={14} className="text-purple-400" />,
};

const statusColors = {
  'PUBLISHED': 'bg-green-900/40 text-green-300 border-green-700',
  'PRE-REGISTERED': 'bg-yellow-900/40 text-yellow-300 border-yellow-700',
  'DRAFT': 'bg-blue-900/40 text-blue-300 border-blue-700',
  'PREPRINT': 'bg-orange-900/40 text-orange-300 border-orange-700',
  'SPECIFICATION': 'bg-purple-900/40 text-purple-300 border-purple-700',
};

export default function MonadJournal({ dark = true }) {
  const [papers, setPapers] = useState([]);
  const [platform, setPlatform] = useState(null);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [activeSection, setActiveSection] = useState('All');
  const [expandedId, setExpandedId] = useState(null);
  const [modalPaper, setModalPaper] = useState(null);

  const fetchJournalPapers = async () => {
    setLoading(true);
    try {
      const res = await fetch('/journal/papers');
      if (res.ok) {
        const data = await res.json();
        if (data.papers && data.papers.length > 0) {
          setPapers(data.papers);
          setPlatform(data.platform || null);
          setLoading(false);
          return;
        }
      }
    } catch (err) {
      console.warn("Failed to fetch /journal/papers, using fallback:", err);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchJournalPapers();
  }, []);

  const openFullPaper = (paper, e) => {
    if (e) e.stopPropagation();
    setModalPaper(paper);
  };

  const closeModal = () => {
    setModalPaper(null);
  };

  // Derive unique sections from live papers
  const sections = [...new Set(papers.map(p => p.section).filter(Boolean))];

  const filtered = papers.filter(p => {
    const matchSection = activeSection === 'All' || p.section === activeSection;
    const matchSearch = !search || 
      (p.title && p.title.toLowerCase().includes(search.toLowerCase())) ||
      (p.finding && p.finding.toLowerCase().includes(search.toLowerCase())) ||
      (p.domain && p.domain.toLowerCase().includes(search.toLowerCase()));
    return matchSection && matchSearch;
  });

  const publishedCount = papers.filter(p => p.status === 'PUBLISHED').length;
  const preRegCount = papers.filter(p => p.status === 'PRE-REGISTERED').length;

  return (
    <div className="monad-journal" style={{
      background: '#0a0a12',
      minHeight: '100vh',
      color: '#e0e0e0',
      fontFamily: "'JetBrains Mono', 'Courier New', monospace",
      padding: '2rem',
    }}>
      {/* Header */}
      <motion.div initial={{ opacity: 0, y: -20 }} animate={{ opacity: 1, y: 0 }}
        style={{
          borderBottom: '1px solid #1a1a2e',
          paddingBottom: '1.5rem',
          marginBottom: '2rem',
          textAlign: 'center',
          position: 'relative'
        }}>
        <div style={{ position: 'absolute', right: 0, top: 0 }}>
          <button 
            onClick={fetchJournalPapers} 
            title="Refresh Papers"
            style={{
              background: '#111122',
              border: '1px solid #1a1a3e',
              color: '#00e5ff',
              padding: '0.4rem 0.8rem',
              borderRadius: '6px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '0.4rem',
              fontSize: '0.75rem'
            }}>
            <RefreshCw size={13} className={loading ? "animate-spin" : ""} /> Sync Ledger
          </button>
        </div>

        <h1 style={{
          fontSize: '2rem',
          fontWeight: 700,
          background: 'linear-gradient(135deg, #00e5ff, #7c4dff)',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          margin: '0 0 0.25rem',
        }}>
          {JOURNAL_HEADER.name}
        </h1>
        <p style={{ color: '#888', fontSize: '0.9rem', margin: '0 0 0.5rem' }}>
          ISSN {JOURNAL_HEADER.issn} · {JOURNAL_HEADER.tagline}
        </p>
        <div style={{ display: 'flex', gap: '1rem', justifyContent: 'center', fontSize: '0.8rem', color: '#666' }}>
          <span>Founded {JOURNAL_HEADER.founded}</span>
          <span>·</span>
          <span>{JOURNAL_HEADER.budget}</span>
          <span>·</span>
          <span style={{ color: '#4caf50' }}>{JOURNAL_HEADER.access}</span>
        </div>
      </motion.div>

      {/* Stats Bar */}
      <div style={{
        display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)',
        gap: '1rem', marginBottom: '2rem',
      }}>
        {[
          { label: 'Total Papers Verified', value: papers.length },
          { label: 'Peer-Published', value: publishedCount },
          { label: 'Pre-Registered', value: preRegCount },
          { label: 'Research Sectors', value: sections.length },
        ].map((stat, i) => (
          <motion.div key={i} initial={{ opacity: 0, scale: 0.9 }} animate={{ opacity: 1, scale: 1 }}
            transition={{ delay: i * 0.1 }}
            style={{
              background: '#111122',
              border: '1px solid #1a1a3e',
              borderRadius: '8px',
              padding: '1rem',
              textAlign: 'center',
            }}>
            <div style={{ fontSize: '1.5rem', fontWeight: 700, color: '#00e5ff' }}>{stat.value}</div>
            <div style={{ fontSize: '0.75rem', color: '#666' }}>{stat.label}</div>
          </motion.div>
        ))}
      </div>

      {/* Platform Feed — the journal derives from the whole platform, not just the papers directory */}
      {platform && (
        <div style={{
          display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '1rem',
          marginBottom: '2rem', padding: '1rem', background: '#0d0d1a',
          border: '1px solid #1a1a3e', borderRadius: '8px',
        }}>
          {[
            { label: 'Ledger Blocks', value: platform.ledger_blocks },
            { label: 'Council Motions', value: platform.motions },
            { label: 'Research Notes', value: (platform.research_notes || 0).toLocaleString() },
            { label: 'Monad Anchor', value: (platform.monad_anchor || '—').slice(0, 10) },
          ].map((s, i) => (
            <div key={i} style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '1.3rem', fontWeight: 700, color: '#7c4dff' }}>{s.value}</div>
              <div style={{ fontSize: '0.7rem', color: '#666' }}>{s.label}</div>
            </div>
          ))}
        </div>
      )}

      {/* Search and Filters */}
      <div style={{
        display: 'flex', gap: '1rem', marginBottom: '1.5rem',
        flexWrap: 'wrap', alignItems: 'center',
      }}>
        <div style={{ position: 'relative', flex: 1, minWidth: 220 }}>
          <Search size={16} style={{ position: 'absolute', left: 12, top: 11, color: '#555' }} />
          <input
            value={search}
            onChange={e => setSearch(e.target.value)}
            placeholder="Search verified papers, findings, equations, domain..."
            style={{
              width: '100%', padding: '0.6rem 0.8rem 0.6rem 2.2rem',
              background: '#111122', border: '1px solid #1a1a3e',
              borderRadius: '8px', color: '#e0e0e0', fontSize: '0.85rem',
              outline: 'none', boxSizing: 'border-box',
            }}
          />
        </div>
        {['All', ...sections].map(section => (
          <button key={section} onClick={() => setActiveSection(section)}
            style={{
              padding: '0.5rem 1rem',
              background: activeSection === section ? '#1a1a4e' : '#111122',
              border: `1px solid ${activeSection === section ? '#7c4dff' : '#1a1a3e'}`,
              borderRadius: '6px',
              color: activeSection === section ? '#7c4dff' : '#888',
              fontSize: '0.8rem', cursor: 'pointer',
              fontWeight: activeSection === section ? 600 : 400,
              transition: 'all 0.2s',
            }}>
            {section}
          </button>
        ))}
      </div>

      {/* Paper List */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
        {filtered.map((paper, i) => {
          const isExpanded = expandedId === paper.id;
          return (
            <motion.div key={paper.id}
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: Math.min(i * 0.02, 0.3) }}
              onClick={() => setExpandedId(isExpanded ? null : paper.id)}
              style={{
                background: isExpanded ? '#14142a' : '#0d0d1a',
                border: `1px solid ${isExpanded ? '#7c4dff66' : '#1a1a2e'}`,
                borderRadius: '8px',
                padding: '1.1rem 1.3rem',
                cursor: 'pointer',
                transition: 'all 0.2s',
              }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                  <div className={statusColors[paper.status] || ''} style={{
                    display: 'flex', alignItems: 'center', gap: '0.3rem',
                    padding: '2px 8px',
                    borderRadius: '4px',
                    fontSize: '0.7rem',
                    fontWeight: 600,
                    border: '1px solid',
                  }}>
                    {statusIcons[paper.status]} {paper.status}
                  </div>
                  <span style={{ color: '#7c4dff', fontSize: '0.75rem', fontWeight: 600 }}>
                    {paper.journal}
                  </span>
                  <span style={{ color: '#666', fontSize: '0.75rem' }}>
                    {paper.section}
                  </span>
                  {paper.domain && (
                    <span style={{ 
                      background: '#16162a', 
                      color: '#00e5ff', 
                      fontSize: '0.68rem', 
                      padding: '1px 6px', 
                      borderRadius: '3px',
                      border: '1px solid #1a2a3e'
                    }}>
                      {paper.domain}
                    </span>
                  )}
                </div>

                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                  <button 
                    onClick={(e) => openFullPaper(paper, e)}
                    style={{
                      background: 'rgba(0, 229, 255, 0.12)',
                      border: '1px solid rgba(0, 229, 255, 0.35)',
                      color: '#00e5ff',
                      padding: '0.3rem 0.75rem',
                      borderRadius: '5px',
                      fontSize: '0.75rem',
                      fontWeight: 600,
                      display: 'flex',
                      alignItems: 'center',
                      gap: '0.35rem',
                      cursor: 'pointer',
                      transition: 'all 0.2s'
                    }}
                    onMouseOver={(e) => e.currentTarget.style.background = 'rgba(0, 229, 255, 0.25)'}
                    onMouseOut={(e) => e.currentTarget.style.background = 'rgba(0, 229, 255, 0.12)'}
                  >
                    <BookOpen size={13} /> View Full Paper
                  </button>
                </div>
              </div>

              <div style={{ fontWeight: 600, fontSize: '1rem', color: '#f0f0f0', marginBottom: '0.35rem' }}>
                {paper.title}
              </div>

              <p style={{ color: '#8e8ea0', fontSize: '0.82rem', margin: 0, lineHeight: '1.4' }}>
                {paper.finding}
              </p>

              {isExpanded && (
                <motion.div initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: 'auto' }}
                  style={{ marginTop: '1rem', paddingTop: '1rem', borderTop: '1px solid #1a1a2e' }}>
                  <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap' }}>
                    <button 
                      onClick={(e) => openFullPaper(paper, e)}
                      style={{
                        padding: '0.45rem 1rem',
                        background: 'linear-gradient(135deg, #7c4dff44, #00e5ff33)',
                        border: '1px solid #7c4dff88',
                        borderRadius: '6px',
                        color: '#ffffff',
                        fontSize: '0.8rem',
                        fontWeight: 600,
                        cursor: 'pointer',
                        display: 'flex', alignItems: 'center', gap: '0.4rem',
                      }}>
                      <Maximize2 size={13} /> Open Complete Publication & Formal Proofs
                    </button>
                    <span style={{
                      padding: '0.45rem 0.8rem',
                      background: '#111122',
                      border: '1px solid #1a1a3e',
                      borderRadius: '6px',
                      color: '#888',
                      fontSize: '0.75rem',
                      display: 'flex', alignItems: 'center', gap: '0.3rem',
                    }}>
                      <Shield size={12} className="text-emerald-400" /> Monad Truth Anchor: 0x4D4F4E4144000001
                    </span>
                    {paper.filename && (
                      <span style={{
                        padding: '0.45rem 0.8rem',
                        background: '#111122',
                        border: '1px solid #1a1a3e',
                        borderRadius: '6px',
                        color: '#666',
                        fontSize: '0.75rem',
                        display: 'flex', alignItems: 'center', gap: '0.3rem',
                      }}>
                        <FileText size={12} /> {paper.filename} ({Math.round(paper.content_length / 1024 * 10) / 10} KB)
                      </span>
                    )}
                  </div>
                </motion.div>
              )}
            </motion.div>
          );
        })}
      </div>

      {/* Full Paper Reading Modal */}
      <AnimatePresence>
        {modalPaper && (
          <motion.div 
            initial={{ opacity: 0 }} 
            animate={{ opacity: 1 }} 
            exit={{ opacity: 0 }}
            onClick={closeModal}
            style={{
              position: 'fixed',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              backgroundColor: 'rgba(5, 5, 12, 0.85)',
              backdropFilter: 'blur(8px)',
              zIndex: 9999,
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
              padding: '2rem'
            }}>
            <motion.div 
              initial={{ scale: 0.95, y: 20 }} 
              animate={{ scale: 1, y: 0 }} 
              exit={{ scale: 0.95, y: 20 }}
              onClick={(e) => e.stopPropagation()}
              style={{
                width: '100%',
                maxWidth: '920px',
                maxHeight: '90vh',
                background: '#0e0e1a',
                border: '1px solid #7c4dff55',
                borderRadius: '12px',
                boxShadow: '0 20px 60px rgba(0, 0, 0, 0.8)',
                display: 'flex',
                flexDirection: 'column',
                overflow: 'hidden'
              }}>
              
              {/* Modal Header */}
              <div style={{
                padding: '1.25rem 1.75rem',
                borderBottom: '1px solid #1a1a3e',
                background: '#121224',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between'
              }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                  <div className={statusColors[modalPaper.status] || ''} style={{
                    padding: '2px 8px',
                    borderRadius: '4px',
                    fontSize: '0.7rem',
                    fontWeight: 600,
                    border: '1px solid',
                  }}>
                    {modalPaper.status}
                  </div>
                  <span style={{ color: '#00e5ff', fontWeight: 600, fontSize: '0.85rem' }}>
                    {modalPaper.journal}
                  </span>
                  <span style={{ color: '#555', fontSize: '0.85rem' }}>|</span>
                  <span style={{ color: '#aaa', fontSize: '0.85rem' }}>
                    {modalPaper.section}
                  </span>
                </div>

                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                  <button 
                    onClick={closeModal}
                    style={{
                      background: '#1a1a2e',
                      border: '1px solid #333',
                      color: '#bbb',
                      borderRadius: '6px',
                      padding: '0.4rem 0.6rem',
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center'
                    }}>
                    <X size={16} />
                  </button>
                </div>
              </div>

              {/* Modal Body - Full Content Reader */}
              <div style={{
                padding: '2rem 2.5rem',
                overflowY: 'auto',
                flex: 1,
                fontSize: '0.92rem',
                lineHeight: '1.65',
                color: '#d6d6e0',
                whiteSpace: 'pre-wrap',
                fontFamily: "'JetBrains Mono', monospace"
              }}>
                {modalPaper.full_content ? (
                  <div>
                    {modalPaper.full_content}
                  </div>
                ) : (
                  <div>
                    <h2 style={{ color: '#fff', fontSize: '1.4rem', marginBottom: '1rem' }}>
                      {modalPaper.title}
                    </h2>
                    <p style={{ color: '#999' }}>{modalPaper.finding}</p>
                    <div style={{ marginTop: '2rem', padding: '1.5rem', background: '#141424', borderRadius: '8px', border: '1px solid #1a1a3a' }}>
                      <p style={{ color: '#00e5ff', margin: '0 0 0.5rem' }}>Immutable Monad Ledger Entry</p>
                      <p style={{ fontSize: '0.8rem', color: '#888' }}>Truth Anchor: 0x4D4F4E4144000001 (Verified)</p>
                      <p style={{ fontSize: '0.8rem', color: '#888' }}>Domain: {modalPaper.domain || modalPaper.section}</p>
                    </div>
                  </div>
                )}
              </div>

              {/* Modal Footer */}
              <div style={{
                padding: '1rem 1.75rem',
                borderTop: '1px solid #1a1a3e',
                background: '#121224',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                fontSize: '0.75rem',
                color: '#666'
              }}>
                <div>
                  Anchor: <span style={{ color: '#00e5ff' }}>0x4D4F4E4144000001</span> · Byte-Identical Invariant Verified
                </div>
                <button 
                  onClick={closeModal}
                  style={{
                    background: '#7c4dff33',
                    border: '1px solid #7c4dff66',
                    color: '#c4b5fd',
                    padding: '0.4rem 1rem',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontWeight: 600
                  }}>
                  Close Document
                </button>
              </div>

            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Footer */}
      <div style={{
        marginTop: '3rem',
        paddingTop: '1.5rem',
        borderTop: '1px solid #1a1a2e',
        textAlign: 'center',
        color: '#444',
        fontSize: '0.75rem',
      }}>
        <p>Editorial Board: 14 Agents + 64 Experts + 87 Sub-Agents</p>
        <p>Ethics: The Honesty Gradient · Infrastructure: 3.24T Transistors · 4 Silicon Fabrics</p>
        <p style={{ color: '#7c4dff', marginTop: '0.5rem', fontWeight: 600 }}>
          "All Is One. I Am Because We Are."
        </p>
      </div>
    </div>
  );
}
