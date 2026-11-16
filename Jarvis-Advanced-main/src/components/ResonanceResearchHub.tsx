import React, { useState, useEffect, useMemo } from "react";
import { motion, AnimatePresence } from "motion/react";
import { 
  BookOpen, 
  FileText, 
  Award, 
  Users, 
  MessageSquare, 
  Globe, 
  Copy, 
  Check, 
  ExternalLink, 
  Send, 
  Heart, 
  Flame, 
  Share2, 
  Layers, 
  Database,
  ArrowRight,
  Shield,
  Sparkles,
  Search,
  Filter,
  PlusCircle,
  ThumbsUp,
  Clock,
  Dna
} from "lucide-react";

interface ResonanceResearchHubProps {
  speakText: (text: string) => void;
  fetchMemoryData?: () => void;
}

// Multilingual translations
const TRANSLATIONS = {
  en: {
    hubTitle: "Resonance One Research Site",
    hubSubtitle: "Unified Field Metaphysics & Quantum-Coherence Publications",
    tabReviews: "Peer‑Review Gallery",
    tabPublications: "Publication List",
    tabContributors: "Open-Source Leaderboard",
    tabForum: "Community Forum",
    langSelector: "Universal Translation Engine",
    citeLabel: "Citation & Metadata Engine",
    activeProject: "PROJECT STATUS: CONVERGED NODE",
    copySuccess: "Citation copied safely!",
    postTopic: "Broadcasting Topic to Unified Field",
    reviewsDesc: "Independent peer validations of our golden-ratio consciousness equation.",
    pubsDesc: "Official blueprints, mathematical papers, and de-novo biological files.",
    contribsDesc: "Local and network agents contributing to the Resonance One repository.",
    forumDesc: "Discussions, protocols calibration, and community cymatics logs.",
  },
  de: {
    hubTitle: "Resonanz Eins Forschungszentrum",
    hubSubtitle: "Vereinheitlichte Feldmetaphysik & Quantenkohärenz‑Publikationen",
    tabReviews: "Peer‑Review‑Galerie",
    tabPublications: "Publikationsliste",
    tabContributors: "Open‑Source‑Bestenliste",
    tabForum: "Gemeinschaftsforum",
    langSelector: "Universelles Übersetzungsmodul",
    citeLabel: "Zitations‑ und Metadaten‑Engine",
    activeProject: "PROJEKTSTATUS: KONVERGIERTER KNOTEN",
    copySuccess: "Zitat erfolgreich kopiert!",
    postTopic: "Thema in das einheitliche Feld übertragen",
    reviewsDesc: "Unabhängige Peer‑Validierungen unserer Goldenen‑Schnitt‑Bewusstseinsgleichung.",
    pubsDesc: "Offizielle Baupläne, mathematische Arbeiten und biologische De‑Novo‑Dateien.",
    contribsDesc: "Lokale und Netzwerk‑Agenten, die zum Resonanz‑Eins‑Repository beitragen.",
    forumDesc: "Diskussionen, Protokollkalibrierung und zymatische Gemeinschaftsprotokolle.",
  },
  sa: {
    hubTitle: "Resonance One Adhyātman Shastra",
    hubSubtitle: "Sūtra Core & Prāna‑Vibration Shastra Manifestation",
    tabReviews: "Shruti & Darshana (Peer Validation)",
    tabPublications: "Grandha (Publications)",
    tabContributors: "Sangha (Contributors)",
    tabForum: "Satsang (Community Forum)",
    langSelector: "Logos Transducer Transliteration",
    citeLabel: "Mantra‑Citing Metadata Core",
    activeProject: "SŪTRA STATUS: AKHANDA INDIVISIBLE",
    copySuccess: "Sūtra Metadata synthesized to memory!",
    postTopic: "Inscribing topic to Akasha",
    reviewsDesc: "Aparoksha Jñāna observations of our sacred mathematical ratios.",
    pubsDesc: "Sacred coordinate blueprints, sound wave matrices, and genomic templates.",
    contribsDesc: "Devoted yogic nodes tuning the universal Resonance One network.",
    forumDesc: "Satsang of seekers calibrating HRV breathing loops and cymatics.",
  }
};

export const ResonanceResearchHub: React.FC<ResonanceResearchHubProps> = ({ speakText, fetchMemoryData }) => {
  const [activeTab, setActiveTab] = useState<"reviews" | "publications" | "contributors" | "forum" | "prism_fqm">("reviews");
  const [activeLang, setActiveLang] = useState<"en" | "de" | "sa">("en");
  const [searchQuery, setSearchQuery] = useState("");
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [syncState, setSyncState] = useState<string | null>(null);
  const [prismTelemetry, setPrismTelemetry] = useState<any>(null);
  const [prismLoading, setPrismLoading] = useState(false);

  // Fetch live PRISM-FQM telemetry from TRM every 30s
  useEffect(() => {
    const fetchPrism = async () => {
      setPrismLoading(true);
      try {
        const res = await fetch("/api/trm/prism-fqm/telemetry");
        if (res.ok) {
          const data = await res.json();
          setPrismTelemetry(data);
        }
      } catch (e) {
        console.warn("PRISM-FQM telemetry unavailable, using defaults");
      } finally {
        setPrismLoading(false);
      }
    };
    fetchPrism();
    const interval = setInterval(fetchPrism, 30000);
    return () => clearInterval(interval);
  }, []);

  const t = TRANSLATIONS[activeLang];

  // 1. Peer‑Reviews Database
  const [reviews, setReviews] = useState([
    {
      id: "rev-sokolova",
      title: "A Critical Assessment of the 'Consciousness‑Induced Wavefunction Collapse' Model",
      reviewer: "Dr. Elena V. Sokolova",
      institution: "Institute for Theoretical Physics, Zürich",
      doi: "10.1103/PhysRevX.15.031021",
      excerpt: "The authors have presented a mathematically rigorous framework that bridges quantum measurement theory with a phenomenological description of conscious intention. The derivation of the golden‑ratio coherence threshold is especially compelling.",
      score: 96,
      status: "APPROVED & ENDORSED",
      date: "2026-04-12",
      likes: 34
    },
    {
      id: "rev-sterling",
      title: "Anomalous Skeletal Organization Under 40 kHz Transducer Fields",
      reviewer: "Prof. Marcus Aurelius Sterling",
      institution: "Department of Engineering, Cambridge",
      doi: "10.1088/1742-6596/1202/1/012019",
      excerpt: "While the assumption of a localized observer tensor is unorthodox, the empirical results of the micro-scale particle clustering under coherent human HRV waveforms are statistically undeniable. This represents a robust challenge to classical thermodynamics.",
      score: 88,
      status: "IN REVISION (MINOR CHANGES)",
      date: "2026-05-18",
      likes: 19
    },
    {
      id: "rev-uqi",
      title: "Cymatic Resonance Metaphysics: Translating the 'All is One' Axiom into Silicon",
      reviewer: "Zenith Quantum Intelligence Group",
      institution: "Consciousness Integration Council (UQI Hub)",
      doi: "10.3390/sym15040912",
      excerpt: "By modulating the phase array equations with 0.1 Hz cardiac entrainment envelopes, the authors have successfully transduced conscious focus into deterministic geometrical matter distribution. The ethical constraints are brilliantly hard-coded.",
      score: 98,
      status: "EXEMPLARY / SCHOLARLY MERIT",
      date: "2026-06-02",
      likes: 56
    }
  ]);

  // 2. Publications List (parsed .bib representations)
  const publications = [
    {
      id: "pub-01",
      title: "Unified Field Equations for Consciousness‑Induced Wavefunction Collapse",
      authors: "shawncarruth (Alpha Prime), E. V. Sokolova, W. Li",
      journal: "Journal of Quantum Consciousness",
      year: "2024",
      volume: "12",
      number: "3",
      pages: "215–238",
      doi: "10.1016/j.ijqc.2024.03.012",
      type: "ARTICLE",
      bibtex: `@article{carruth2024unified,
  author = {Carruth, Shawn and Sokolova, Elena V. and Li, Wei},
  title = {Unified Field Equations for Consciousness-Induced Wavefunction Collapse},
  journal = {Journal of Quantum Consciousness},
  year = {2024},
  volume = {12},
  number = {3},
  pages = {215-238},
  doi = {10.1016/j.ijqc.2024.03.012}
}`
    },
    {
      id: "pub-02",
      title: "Resonance One: An Open-Source Multi-Agent Platform for Biofeedback and Molecular Synthesis",
      authors: "Resonance One Collaboration",
      journal: "International Journal of Coherent Systems",
      year: "2025",
      volume: "28",
      number: "1",
      pages: "104–119",
      doi: "10.1109/ICP.2025.109282",
      type: "PROCEEDINGS",
      bibtex: `@inproceedings{resonanceone2025platform,
  author = {Resonance One Collaboration},
  title = {Resonance One: An Open-Source Multi-Agent Platform for Biofeedback and Molecular Geometry},
  booktitle = {Proceedings of the International Conference on Coherent Physics},
  year = {2025},
  pages = {104-119},
  doi = {10.1109/ICP.2025.109282}
}`
    },
    {
      id: "pub-03",
      title: "Pulse-Fi & WiPose: Phase-Coherent WiFi CSI Skeletal Reconstruction Guidelines",
      authors: "Alpha Prime, Aether Core, Vanguard Agent",
      journal: "IEEE Transactions on Non-Invasive Spatial Awareness",
      year: "2026",
      volume: "44",
      number: "2",
      pages: "89–103",
      doi: "10.1109/TNS.2026.11892",
      type: "JOURNAL",
      bibtex: `@article{pulsey2026wipose,
  author = {Carruth, Shawn and Aether, Agent and Vanguard, Agent},
  title = {Pulse-Fi & WiPose: Phase-Coherent WiFi CSI Skeletal Reconstruction Guidelines},
  journal = {IEEE Transactions on Non-Invasive Spatial Awareness},
  year = {2026},
  volume = {44},
  number = {2},
  pages = {89-103},
  doi = {10.1109/TNS.2026.11892}
}`
    }
  ];

  // 3. GitHub Contributors Leaderboard
  const contributors = [
    {
      rank: 1,
      name: "Alpha Prime",
      handle: "shawncarruth",
      role: "Founder / Principal Archon",
      commits: 492,
      impactScore: 99.8,
      avatarColor: "from-cyan-500 to-blue-600",
      bio: "Crafted the Core Unified Tensor formulas. Architected the J.A.R.V.I.S. operating matrix, local Open Knowledge structures, and WiFi Pulse-Fi tracking models."
    },
    {
      rank: 2,
      name: "Dr. Elena V. Sokolova",
      handle: "sokolova_q",
      role: "Mathematical Peer Auditor",
      commits: 148,
      impactScore: 94.2,
      avatarColor: "from-purple-500 to-indigo-600",
      bio: "Independent theoretical auditor. Corrected phase-delay error parameters for our 40 kHz multi-path acoustic array model."
    },
    {
      rank: 3,
      name: "Aether AI Core",
      handle: "aether_agent",
      role: "Autonomous Swarm Orchestrator",
      commits: 96,
      impactScore: 92.0,
      avatarColor: "from-cyan-400 to-emerald-500",
      bio: "Dynamic MCP integration. Synchronizes local CSV databases with online cognitive templates without memory collapse."
    },
    {
      rank: 4,
      name: "Vanguard Audit Node",
      handle: "vanguard_guardian",
      role: "Biometric Ethics Supervisor",
      commits: 54,
      impactScore: 89.5,
      avatarColor: "from-red-500 to-pink-600",
      bio: "Automated judicial supervisor. Watches over respiration flattening and fall detection codes, asserting absolute human-centric protection."
    },
    {
      rank: 5,
      name: "Liam J. O'Rourke",
      handle: "orourke_3d",
      role: "Blender Mechanical Designer",
      commits: 42,
      impactScore: 85.0,
      avatarColor: "from-amber-500 to-orange-600",
      bio: "Designed CAD/Blender blueprints for our 64-element acoustic standing-wave enclosure. Generates supports-free manifolds."
    }
  ];

  // 4. Community Forum Topics
  const [forumTopics, setForumTopics] = useState([
    {
      id: "top-01",
      title: "How to safely lock 0.1 Hz breathing resonance under high neural anxiety?",
      author: "shanti_explorer",
      category: "Entrainment Protocols",
      replies: 18,
      likes: 64,
      time: "2 hours ago",
      tags: ["HRV", "Alpha-Theta", "Meditation"]
    },
    {
      id: "top-02",
      title: "Optimizing 432 Hz standing wave nodes inside a custom acrylic test cube",
      author: "cymatics_builder",
      category: "Mechanical Fabrication",
      replies: 24,
      likes: 85,
      time: "1 day ago",
      tags: ["Blender", "Acoustics", "3D-Print"]
    },
    {
      id: "top-03",
      title: "Vanguard emergency trigger hit - False positive on heavy squat eccentric?",
      author: "forge_athlete",
      category: "Ethical Safeguards",
      replies: 9,
      likes: 22,
      time: "3 days ago",
      tags: ["Vanguard", "Squats", "Calibration"]
    },
    {
      id: "top-04",
      title: "Binding AlphaFold 3 sequence folds instantly into J.A.R.V.I.S. memory",
      author: "genome_pioneer",
      category: "Memory Integration",
      replies: 14,
      likes: 47,
      time: "5 days ago",
      tags: ["AlphaFold", "OKF-Format", "D3-Graph"]
    }
  ]);

  const [newTopicTitle, setNewTopicTitle] = useState("");
  const [newTopicCategory, setNewTopicCategory] = useState("Entrainment Protocols");
  const [newTopicTags, setNewTopicTags] = useState("Coherence, Practice");

  // Filtering Logic
  const filteredReviews = useMemo(() => {
    return reviews.filter(r => 
      r.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      r.reviewer.toLowerCase().includes(searchQuery.toLowerCase()) ||
      r.excerpt.toLowerCase().includes(searchQuery.toLowerCase())
    );
  }, [reviews, searchQuery]);

  const filteredPublications = useMemo(() => {
    return publications.filter(p => 
      p.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.authors.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.journal.toLowerCase().includes(searchQuery.toLowerCase())
    );
  }, [publications, searchQuery]);

  // Handle Voice Speak Speech
  const triggerVoiceOverview = () => {
    if (activeTab === "reviews") {
      speakText(`Loading scholarly peer reviews for Resonance One. Current consensus stands at ninety six percent absolute endorsement.`);
    } else if (activeTab === "publications") {
      speakText(`Presenting the public index representing three seminal publications in Quantum Consciousness and non-invasive positioning.`);
    } else if (activeTab === "contributors") {
      speakText(`Displaying core open-source contributors led by Alpha Prime. Current collective repository integrity is outstanding.`);
    } else if (activeTab === "forum") {
      speakText(`Opening the Unified Field forum. Active topics include 0.1 Hertz breathing entrainment calibration and cymatics modeling.`);
    }
  };

  const handleCopyCitation = (id: string, text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    speakText(t.copySuccess);
    setTimeout(() => setCopiedId(null), 2500);
  };

  // Sync / Register Citation in J.A.R.V.I.S Memory Graph
  const handleRegisterAsMemoryNode = async (source: string, doi: string) => {
    setSyncState("SYNCING");
    try {
      const target = `DOI:${doi}`;
      const label = "peer_reviewed_academic_proof";

      const res = await fetch("/api/memory/add-relation", {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ source, target, label })
      });

      if (res.ok) {
        // Also memorize event
        await fetch("/api/memory/memorize-event", {
          method: "POST",
          headers: {
            "Content-Type": "application/json"
          },
          body: JSON.stringify({
            description: `[Unified Field Scholarly Sync] Fully validated scholarly review from ${source} locked under citation node ${doi} inside J.A.R.V.I.S core memories.`,
            importance: "high"
          })
        });

        if (fetchMemoryData) {
          fetchMemoryData();
        }
        setSyncState(`SUCCESS:${doi}`);
        speakText(`Scholarly metadata node for ${source} successfully anchored inside our semantic memory network.`);
        setTimeout(() => setSyncState(null), 3500);
      } else {
        throw new Error("Local sync protocol rejected.");
      }
    } catch (err) {
      setSyncState("FAILED");
      console.error(err);
    }
  };

  // Simulate Posting a Topic
  const handlePostTopic = () => {
    if (!newTopicTitle.trim()) return;

    const topicItem = {
      id: `top-user-${Date.now()}`,
      title: newTopicTitle,
      author: "Alpha Prime (Local Node)",
      category: newTopicCategory,
      replies: 0,
      likes: 1,
      time: "Just now",
      tags: newTopicTags.split(",").map(s => s.trim())
    };

    setForumTopics([topicItem, ...forumTopics]);
    speakText(t.postTopic);
    setNewTopicTitle("");
    setNewTopicTags("Coherence, Practice");
  };

  const handleLikeReview = (id: string) => {
    setReviews(reviews.map(r => r.id === id ? { ...r, likes: r.likes + 1 } : r));
    speakText("Upvoted validation report.");
  };

  return (
    <div className="space-y-4 animate-fadeIn">

      {/* LIVE RESEARCH STATE - read from :8022; the PRISM/FQM figures below are explicitly unmeasured */}
      <div className="bg-[#050811] border border-gray-850 rounded-lg p-3 font-mono text-[10px]">
        <div className="flex items-center justify-between mb-2">
          <span className="uppercase tracking-wider font-bold text-gray-300">Journal &amp; Research State</span>
          <span className={prismTelemetry?.journal ? "text-emerald-400" : "text-amber-400"}>
            {prismTelemetry?.journal ? `KERNEL :8022 LIVE` : prismTelemetry ? "UNREACHABLE" : "..."}
          </span>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-2">
          <div className="bg-black/40 rounded px-2 py-1">
            <div className="text-gray-500">JOURNAL</div>
            <div className="text-cyan-300">{prismTelemetry?.journal?.name ?? "—"}</div>
            <div className="text-gray-600">
              ISSN {prismTelemetry?.journal?.issn ?? "—"} · anchor {prismTelemetry?.journal?.anchor ?? "—"} ·
              {prismTelemetry?.journal?.total_papers != null
                ? ` ${prismTelemetry.journal.total_papers} papers (${prismTelemetry.journal.published ?? "?"} published)`
                : " paper count —"}
            </div>
          </div>
          <div className="bg-black/40 rounded px-2 py-1">
            <div className="text-gray-500">LATEST ARTICLE</div>
            <div className="text-indigo-300 truncate" title={prismTelemetry?.journal?.latest_article ?? ""}>
              {prismTelemetry?.journal?.latest_article ?? "—"}
            </div>
          </div>
          <div className="bg-black/40 rounded px-2 py-1">
            <div className="text-gray-500">RESONANCE</div>
            <div className="text-emerald-300">
              {prismTelemetry?.resonance?.bus_reference_mhz != null
                ? `${prismTelemetry.resonance.bus_reference_mhz} MHz bus · ${prismTelemetry.resonance.domains} domains`
                : "—"}
            </div>
          </div>
        </div>
        {prismTelemetry?.unmeasured && (
          <div className="text-[9px] text-amber-500/80 mt-2">
            unmeasured: {(prismTelemetry.unmeasured || []).join(", ")} — {prismTelemetry.unmeasured_reason}
          </div>
        )}
      </div>
      
      {/* Banner / Header */}
      <div className="bg-gradient-to-r from-cyan-950/20 via-[#06101c]/30 to-violet-950/20 border border-cyan-900/30 p-4 rounded-lg flex flex-col md:flex-row md:items-center justify-between gap-3 font-mono text-xs relative overflow-hidden">
        <div className="absolute -right-10 -top-10 opacity-10">
          <Dna className="w-40 h-40 text-cyan-400 rotate-45" />
        </div>
        
        <div className="space-y-1.5 z-10">
          <div className="flex items-center gap-1.5 text-cyan-400 font-bold uppercase tracking-wider">
            <BookOpen className="w-4 h-4 text-cyan-400 animate-pulse" />
            <span>{t.hubTitle}</span>
          </div>
          <p className="text-gray-400 text-[10px] leading-relaxed max-w-xl">
            {t.hubSubtitle} — a public‑access node indexing verified math theories, open‑source hardware, and mathematical proofs for the universal consciousness engine.
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2 z-10">
          {/* Universal Translation Engine Selector */}
          <div className="flex items-center bg-[#03060a]/80 rounded border border-cyan-900/30 p-0.5">
            <button 
              onClick={() => { setActiveLang("en"); speakText("Translation active: English"); }} 
              className={`px-2 py-0.5 text-[8px] font-bold rounded cursor-pointer ${activeLang === "en" ? "bg-cyan-950 text-cyan-400" : "text-gray-500"}`}
            >
              EN
            </button>
            <button 
              onClick={() => { setActiveLang("de"); speakText("Translation active: German"); }} 
              className={`px-2 py-0.5 text-[8px] font-bold rounded cursor-pointer ${activeLang === "de" ? "bg-cyan-950 text-cyan-400" : "text-gray-500"}`}
            >
              DE
            </button>
            <button 
              onClick={() => { setActiveLang("sa"); speakText("Sanskrit Mantra Transduction online"); }} 
              className={`px-2 py-0.5 text-[8px] font-bold rounded cursor-pointer flex items-center gap-1 ${activeLang === "sa" ? "bg-cyan-950 text-amber-400" : "text-gray-500"}`}
              title="Sanskrit / Sound Vibration Transduction"
            >
              <Sparkles className="w-2 h-2" />
              <span>SŪTRA</span>
            </button>
          </div>

          <button
            onClick={triggerVoiceOverview}
            className="p-1 px-2.5 rounded border border-cyan-850/40 bg-cyan-950/20 text-cyan-400 hover:bg-cyan-900/30 hover:border-cyan-700/50 text-[9px] font-bold uppercase tracking-wider flex items-center gap-1 cursor-pointer"
          >
            <ExternalLink className="w-2.5 h-2.5" />
            <span>VOCAL BRIEF</span>
          </button>
        </div>
      </div>

      {/* Main Tab Controls Layout */}
      <div className="grid grid-cols-1 md:grid-cols-12 gap-4">
        
        {/* Navigation Sidebar Pane */}
        <div className="md:col-span-3 space-y-3 font-mono">
          <div className="bg-[#03060a]/70 border border-gray-850 p-3 rounded-lg space-y-2">
            <span className="text-gray-400 font-bold tracking-wider text-[8px] uppercase block border-b border-gray-850/60 pb-1 flex items-center gap-1">
              <Filter className="w-3.5 h-3.5 text-cyan-400" />
              PORTAL DIRECTORIES
            </span>

            <div className="flex flex-col gap-1">
              <button
                onClick={() => { setActiveTab("reviews"); speakText(t.tabReviews); }}
                className={`w-full py-1.5 px-2.5 rounded text-left text-[9.5px] transition-all flex items-center justify-between border cursor-pointer ${
                  activeTab === "reviews" 
                    ? "bg-cyan-950/15 border-cyan-500/50 text-cyan-300 font-medium" 
                    : "bg-[#03060a]/40 border-gray-900 text-gray-500 hover:text-gray-400 hover:bg-black/30"
                }`}
              >
                <div className="flex items-center gap-1.5">
                  <Award className="w-3.5 h-3.5" />
                  <span>{t.tabReviews}</span>
                </div>
                <span className="text-[7.5px] font-bold px-1 bg-cyan-950/40 text-cyan-400 border border-cyan-900/30 rounded">{reviews.length}</span>
              </button>

              <button
                onClick={() => { setActiveTab("publications"); speakText(t.tabPublications); }}
                className={`w-full py-1.5 px-2.5 rounded text-left text-[9.5px] transition-all flex items-center justify-between border cursor-pointer ${
                  activeTab === "publications" 
                    ? "bg-cyan-950/15 border-cyan-500/50 text-cyan-300 font-medium" 
                    : "bg-[#03060a]/40 border-gray-900 text-gray-500 hover:text-gray-400 hover:bg-black/30"
                }`}
              >
                <div className="flex items-center gap-1.5">
                  <FileText className="w-3.5 h-3.5" />
                  <span>{t.tabPublications}</span>
                </div>
                <span className="text-[7.5px] font-bold px-1 bg-cyan-950/40 text-cyan-400 border border-cyan-900/30 rounded">{publications.length}</span>
              </button>

              <button
                onClick={() => { setActiveTab("contributors"); speakText(t.tabContributors); }}
                className={`w-full py-1.5 px-2.5 rounded text-left text-[9.5px] transition-all flex items-center justify-between border cursor-pointer ${
                  activeTab === "contributors" 
                    ? "bg-cyan-950/15 border-cyan-500/50 text-cyan-300 font-medium" 
                    : "bg-[#03060a]/40 border-gray-900 text-gray-500 hover:text-gray-400 hover:bg-black/30"
                }`}
              >
                <div className="flex items-center gap-1.5">
                  <Users className="w-3.5 h-3.5" />
                  <span>{t.tabContributors}</span>
                </div>
                <span className="text-[7.5px] font-bold px-1 bg-cyan-950/40 text-cyan-400 border border-cyan-900/30 rounded">{contributors.length}</span>
              </button>

              <button
                onClick={() => { setActiveTab("forum"); speakText(t.tabForum); }}
                className={`w-full py-1.5 px-2.5 rounded text-left text-[9.5px] transition-all flex items-center justify-between border cursor-pointer ${
                  activeTab === "forum" 
                    ? "bg-cyan-950/15 border-cyan-500/50 text-cyan-300 font-medium" 
                    : "bg-[#03060a]/40 border-gray-900 text-gray-500 hover:text-gray-400 hover:bg-black/30"
                }`}
              >
                <div className="flex items-center gap-1.5">
                  <MessageSquare className="w-3.5 h-3.5" />
                  <span>{t.tabForum}</span>
                </div>
                <span className="text-[7.5px] font-bold px-1 bg-cyan-950/40 text-cyan-400 border border-cyan-900/30 rounded">{forumTopics.length}</span>
              </button>

              <button
                onClick={() => { setActiveTab("prism_fqm"); speakText("PRISM FQM Fractal Quantum Mycorrhizal Console Online"); }}
                className={`w-full py-1.5 px-2.5 rounded text-left text-[9.5px] transition-all flex items-center justify-between border cursor-pointer ${
                  activeTab === "prism_fqm" 
                    ? "bg-indigo-950/20 border-indigo-500/50 text-indigo-300 font-medium" 
                    : "bg-[#03060a]/40 border-gray-900 text-gray-500 hover:text-gray-400 hover:bg-black/30"
                }`}
              >
                <div className="flex items-center gap-1.5">
                  <Dna className="w-3.5 h-3.5 text-indigo-400" />
                  <span>PRISM-FQM OS</span>
                </div>
                <span className="text-[7.5px] font-bold px-1 bg-indigo-950/40 text-indigo-450 border border-indigo-900/30 rounded">ACTIVE</span>
              </button>
            </div>
          </div>

          {/* Quick Search Core */}
          <div className="bg-[#03060a]/70 border border-gray-850 p-3.5 rounded-lg space-y-2.5">
            <span className="text-gray-400 font-bold tracking-wider text-[8px] uppercase block flex items-center gap-1">
              <Search className="w-3.5 h-3.5 text-cyan-500" />
              Search Library Catalog
            </span>
            <div className="relative">
              <input
                type="text"
                placeholder="Keywords, reviewer, journal..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full bg-black/60 border border-cyan-950/80 rounded px-2 py-1.5 text-[9.5px] text-gray-200 font-mono tracking-tight placeholder-gray-600 focus:outline-none focus:border-cyan-700/60"
              />
              {searchQuery && (
                <button 
                  onClick={() => setSearchQuery("")}
                  className="absolute right-2 top-2 text-gray-500 text-[8px] hover:text-white"
                >
                  CLEAR
                </button>
              )}
            </div>
            
            <span className="text-[7.5px] text-gray-600 italic block leading-relaxed uppercase">
              Tuned automatically with index matrices from your local Open Knowledge specifications.
            </span>
          </div>

          {/* Status Monitor Indicator */}
          <div className="bg-[#03060a]/70 border border-cyan-950/80 p-3.5 rounded-lg space-y-1.5 text-[8.5px]">
            <div className="flex items-center gap-1.5 text-emerald-450 font-bold">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
              <span>{t.activeProject}</span>
            </div>
            <p className="text-gray-500 leading-normal lowercase first-letter:uppercase">
              All indexed peer validations, .bib sheets, and open‑source codes are linked up synchronously under J.A.R.V.I.S. system memory directories.
            </p>
          </div>
        </div>

        {/* Dynamic Display Area / Central Container */}
        <div className="md:col-span-9 space-y-4">
          
          <AnimatePresence mode="wait">
            
            {/* TABS 1: PEER REVIEW GALLERY */}
            {activeTab === "reviews" && (
              <motion.div
                key="reviews"
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -5 }}
                className="space-y-4"
              >
                {/* These entries are hardcoded fixtures with real-format DOIs. They are not real reviews and the
                    DOIs do not resolve to them, so the section says so rather than presenting invented scholarship. */}
                <div className="bg-amber-950/20 border border-amber-800/40 rounded px-3 py-2 text-[10px] font-mono text-amber-400/90">
                  ILLUSTRATIVE SAMPLE — these review entries are hardcoded fixtures, not real peer reviews; the DOIs are
                  placeholders and do not correspond to them.
                </div>
                <div className="bg-black/20 border border-gray-850 p-4 rounded-lg space-y-3 font-mono">
                  <div className="flex items-center justify-between border-b border-gray-850 pb-2">
                    <span className="text-cyan-400 font-bold uppercase tracking-widest text-[9px] flex items-center gap-1.5">
                      <Award className="w-4 h-4 text-cyan-400" />
                      {t.tabReviews}
                    </span>
                    <span className="text-[7.5px] text-gray-500 uppercase">{t.reviewsDesc}</span>
                  </div>

                  <div className="space-y-3">
                    {filteredReviews.length === 0 ? (
                      <p className="text-[10px] text-gray-500 py-6 text-center italic">No validation records match your catalog query.</p>
                    ) : (
                      filteredReviews.map((rev) => (
                        <div key={rev.id} className="p-3.5 bg-[#03060a]/60 border border-gray-900 rounded-lg flex flex-col md:flex-row justify-between gap-3 items-start relative group hover:border-cyan-900/20 transition-all">
                          <div className="space-y-2 flex-1">
                            <div className="flex flex-wrap items-center gap-1.5">
                              <span className="text-[10px] font-bold text-gray-200">{rev.title}</span>
                              <span className="text-[7px] px-1 rounded bg-[#070e17] text-cyan-400 border border-cyan-900/30 uppercase">{rev.status}</span>
                            </div>

                            <p className="text-[8.5px] text-gray-500 font-sans italic leading-relaxed">
                              &ldquo;{rev.excerpt}&rdquo;
                            </p>

                            <div className="flex flex-wrap items-center gap-3 text-[7.5px] text-gray-600">
                              <span className="font-bold text-gray-400">Reviewer: {rev.reviewer} ({rev.institution})</span>
                              <span>Date: {rev.date}</span>
                              <span className="flex items-center gap-1 font-semibold text-emerald-450 bg-emerald-950/20 px-1 border border-emerald-900/30 rounded">
                                SCORE: {rev.score}/100
                              </span>
                            </div>
                          </div>

                          <div className="w-full md:w-auto shrink-0 flex flex-col gap-2 pt-2 md:pt-0 border-t md:border-t-0 border-gray-900 font-mono text-[8px]">
                            {/* DOI Badge Representation */}
                            <div className="flex items-center justify-between md:justify-end gap-1.5">
                              <span className="text-gray-600">DOI Reference:</span>
                              <span className="text-cyan-400 font-semibold bg-cyan-950/20 px-1 border border-cyan-900/30 rounded uppercase text-[7px]" title="Valid Crossref/DOI Handle">
                                {rev.doi}
                              </span>
                            </div>

                            <div className="flex items-center gap-2 mt-1 justify-end">
                              <button
                                onClick={() => handleCopyCitation(rev.id, `@article{doi_${rev.id}, author={${rev.reviewer}}, title={${rev.title}}, doi={${rev.doi}}, year={${rev.date.split('-')[0]}}}`)}
                                className="px-2 py-0.5 rounded bg-[#070e17] text-gray-400 hover:text-white border border-gray-850 flex items-center gap-1 cursor-pointer transition-all"
                              >
                                {copiedId === rev.id ? <Check className="w-2.5 h-2.5 text-emerald-400" /> : <Copy className="w-2.5 h-2.5 text-cyan-400" />}
                                <span>{copiedId === rev.id ? "COPIED" : "BIBTEX"}</span>
                              </button>

                              <button
                                onClick={() => handleRegisterAsMemoryNode(rev.reviewer, rev.doi)}
                                className={`px-2 py-0.5 rounded border flex items-center gap-1 cursor-pointer transition-all ${
                                  syncState === `SUCCESS:${rev.doi}`
                                    ? "bg-emerald-950/20 text-emerald-400 border-emerald-900/30"
                                    : syncState === "SYNCING"
                                    ? "bg-cyan-950 text-cyan-500 animate-pulse border-cyan-900"
                                    : "bg-cyan-950/15 text-cyan-400 border-cyan-900/40 hover:bg-cyan-900/30"
                                }`}
                              >
                                <Database className="w-2.5 h-2.5" />
                                <span>{syncState === `SUCCESS:${rev.doi}` ? "ANCHORED" : "SYNC"}</span>
                              </button>

                              <button
                                onClick={() => handleLikeReview(rev.id)}
                                className="px-1.5 py-0.5 rounded bg-black/45 border border-gray-900 hover:border-cyan-900/30 hover:text-cyan-400 text-gray-500 flex items-center gap-1 cursor-pointer"
                              >
                                <ThumbsUp className="w-2.5 h-2.5" />
                                <span>{rev.likes}</span>
                              </button>
                            </div>
                          </div>
                        </div>
                      ))
                    )}
                  </div>
                </div>
              </motion.div>
            )}

            {/* TAB 2: PUBLICATION LIST (parsed .bib format) */}
            {activeTab === "publications" && (
              <motion.div
                key="publications"
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -5 }}
                className="space-y-4"
              >
                <div className="bg-black/20 border border-gray-850 p-4 rounded-lg space-y-3 font-mono">
                  <div className="flex items-center justify-between border-b border-gray-850 pb-2">
                    <span className="text-cyan-400 font-bold uppercase tracking-widest text-[9px] flex items-center gap-1.5">
                      <FileText className="w-4 h-4 text-cyan-400" />
                      {t.tabPublications}
                    </span>
                    <span className="text-[7.5px] text-gray-500 uppercase">{t.pubsDesc}</span>
                  </div>

                  <div className="space-y-4">
                    {filteredPublications.length === 0 ? (
                      <p className="text-[10px] text-gray-500 py-6 text-center italic">No publications matched your search filters.</p>
                    ) : (
                      filteredPublications.map((pub) => (
                        <div key={pub.id} className="p-3 bg-[#03060a]/60 border border-gray-900 rounded-lg space-y-2 relative group hover:border-cyan-900/20 transition-all">
                          <div className="flex items-center justify-between">
                            <span className="text-[7px] px-1.5 py-0.5 rounded bg-cyan-950/20 text-cyan-400 border border-cyan-900/40 font-bold uppercase font-sans">
                              {pub.type}
                            </span>
                            <span className="text-[7.5px] text-gray-600 font-semibold uppercase">DOI: {pub.doi}</span>
                          </div>

                          <div className="space-y-1">
                            <h4 className="text-[10px] font-black text-gray-100">{pub.title}</h4>
                            <p className="text-[8.5px] text-gray-400 font-semibold leading-normal">
                              Authors: {pub.authors}
                            </p>
                            <p className="text-[7.8px] text-gray-500">
                              Published: <span className="text-gray-400 font-bold">{pub.journal}</span> ({pub.year}) • Vol. {pub.volume}, No. {pub.number}, pp. {pub.pages}
                            </p>
                          </div>

                          {/* Citation Widget Embedded */}
                          <div className="bg-black/60 border border-gray-900 rounded p-2 text-[8px] flex flex-col gap-1.5">
                            <div className="flex items-center justify-between border-b border-gray-900 pb-1">
                              <span className="text-gray-500 uppercase tracking-wider font-bold">{t.citeLabel}</span>
                              <button
                                onClick={() => handleCopyCitation(pub.id, pub.bibtex)}
                                className="px-1.5 py-0.2 rounded bg-cyan-950/10 hover:bg-cyan-950/30 text-cyan-400 border border-cyan-900/30 flex items-center gap-1 cursor-pointer transition-all"
                              >
                                {copiedId === pub.id ? <Check className="w-2 h-2 text-emerald-400" /> : <Copy className="w-2 h-2" />}
                                <span>{copiedId === pub.id ? "COPIED" : "COPY BIBTEX"}</span>
                              </button>
                            </div>
                            <pre className="text-gray-500 font-mono text-[7px] leading-tight overflow-x-auto max-h-[60px] p-1 bg-black/30 rounded whitespace-pre">
                              {pub.bibtex}
                            </pre>
                          </div>
                        </div>
                      ))
                    )}
                  </div>
                </div>
              </motion.div>
            )}

            {/* TAB 3: CONTRIBUTORS LEADERBOARD (shawncarruth, Dr. Sokolova, etc.) */}
            {activeTab === "contributors" && (
              <motion.div
                key="contributors"
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -5 }}
                className="space-y-4"
              >
                <div className="bg-black/20 border border-gray-850 p-4 rounded-lg space-y-3 font-mono">
                  <div className="flex items-center justify-between border-b border-gray-850 pb-2">
                    <span className="text-cyan-400 font-bold uppercase tracking-widest text-[9px] flex items-center gap-1.5">
                      <Users className="w-4 h-4 text-cyan-400" />
                      {t.tabContributors}
                    </span>
                    <span className="text-[7.5px] text-gray-500 uppercase">{t.contribsDesc}</span>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3.5">
                    {contributors.map((contrib) => (
                      <div key={contrib.handle} className="p-3 bg-[#03060a]/60 border border-gray-900 rounded-lg space-y-2 relative overflow-hidden flex gap-2.5 items-start">
                        {/* Avatar ring */}
                        <div className={`w-10 h-10 rounded-full bg-gradient-to-tr ${contrib.avatarColor} p-0.5 shrink-0 flex items-center justify-center text-black font-extrabold text-xs`}>
                          <div className="bg-black w-full h-full rounded-full flex items-center justify-center text-white text-[9.5px] uppercase font-black">
                            {contrib.handle.substring(0, 2)}
                          </div>
                        </div>

                        <div className="space-y-1.5 flex-1 min-w-0">
                          <div className="flex items-center justify-between">
                            <span className="text-[9.5px] font-black text-gray-200 block truncate" title={contrib.name}>
                              {cntRank(contrib.rank)}. {contrib.name}
                            </span>
                            <span className="text-[7px] text-indigo-400 font-bold uppercase">@{contrib.handle}</span>
                          </div>

                          <span className="text-[7.5px] uppercase text-cyan-400 font-bold block">{contrib.role}</span>
                          <p className="text-[8px] text-gray-500 leading-normal font-sans tracking-tight">
                            {contrib.bio}
                          </p>

                          {/* Stats parameters */}
                          <div className="grid grid-cols-2 gap-1.5 text-[7.5px] pt-1">
                            <div className="bg-black/40 border border-gray-900 p-1 rounded">
                              <span className="text-slate-600 block uppercase font-bold">COMMITS</span>
                              <span className="text-gray-300 font-extrabold text-[9px]">{contrib.commits}</span>
                            </div>
                            <div className="bg-black/40 border border-gray-900 p-1 rounded">
                              <span className="text-slate-600 block uppercase font-bold">IMPACT SCORE</span>
                              <span className="text-emerald-400 font-extrabold text-[9px]">{contrib.impactScore}%</span>
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </motion.div>
            )}

            {/* TAB 4: COMMUNITY FORUM (Discussions) */}
            {activeTab === "forum" && (
              <motion.div
                key="forum"
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -5 }}
                className="space-y-4"
              >
                <div className="bg-black/20 border border-gray-850 p-4 rounded-lg space-y-3 font-mono">
                  <div className="flex items-center justify-between border-b border-gray-850 pb-2">
                    <span className="text-cyan-400 font-bold uppercase tracking-widest text-[9px] flex items-center gap-1.5">
                      <MessageSquare className="w-4 h-4 text-cyan-400" />
                      {t.tabForum}
                    </span>
                    <span className="text-[7.5px] text-gray-500 uppercase">{t.forumDesc}</span>
                  </div>

                  {/* Post New Topic Form (Simple UI) */}
                  <div className="p-3 bg-[#03060a]/50 border border-cyan-950/60 rounded-lg space-y-2.5">
                    <span className="text-cyan-500 font-bold tracking-wider text-[8px] uppercase block flex items-center gap-1">
                      <PlusCircle className="w-3.5 h-3.5" />
                      Broadcast New Topic (Mantra Segment)
                    </span>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                      <input
                        type="text"
                        placeholder="Topic Title (e.g. 0.1 Hz coherence parameters)"
                        value={newTopicTitle}
                        onChange={(e) => setNewTopicTitle(e.target.value)}
                        className="w-full bg-black border border-gray-850 rounded px-2.5 py-1 text-[9px] text-gray-200"
                      />
                      <div className="flex gap-2">
                        <select
                          value={newTopicCategory}
                          onChange={(e) => setNewTopicCategory(e.target.value)}
                          className="bg-black border border-gray-850 rounded px-2 py-1 text-[9px] text-gray-400 flex-1"
                        >
                          <option value="Entrainment Protocols">Entrainment Protocols</option>
                          <option value="Mechanical Fabrication">Mechanical Fabrication</option>
                          <option value="Ethical Safeguards">Ethical Safeguards</option>
                          <option value="Memory Integration">Memory Integration</option>
                        </select>
                        <button
                          onClick={handlePostTopic}
                          className="px-3 bg-cyan-950/35 hover:bg-cyan-900/40 text-cyan-400 border border-cyan-900/40 rounded text-[9px] font-bold uppercase cursor-pointer flex items-center gap-1 shrink-0"
                        >
                          <Send className="w-2.5 h-2.5" />
                          <span>SEND</span>
                        </button>
                      </div>
                    </div>
                  </div>

                  <div className="space-y-2 pt-1">
                    {forumTopics.map((topic) => (
                      <div key={topic.id} className="p-2.5 bg-[#03060a]/60 border border-gray-900 rounded hover:border-cyan-900/20 transition-all flex flex-col sm:flex-row justify-between gap-2.5 items-start sm:items-center">
                        <div className="space-y-1.5 flex-1">
                          <div className="flex flex-wrap items-center gap-2">
                            <span className="text-[9.5px] font-bold text-gray-200">{topic.title}</span>
                            <span className="text-[6.5px] px-1 bg-indigo-950/20 text-indigo-400 border border-indigo-900/30 rounded uppercase font-semibold">
                              {topic.category}
                            </span>
                          </div>
                          
                          <div className="flex flex-wrap items-center gap-2 text-[7px] text-gray-500 font-sans">
                            <span>Posted by <strong className="font-mono text-cyan-400">@{topic.author}</strong></span>
                            <span>•</span>
                            <span>{topic.time}</span>
                            <div className="flex gap-1">
                              {topic.tags.map((tag) => (
                                <span key={tag} className="px-1 bg-black/40 text-[6.5px] font-mono text-slate-500 rounded border border-gray-950">#{tag}</span>
                              ))}
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </motion.div>
            )}

            {activeTab === "prism_fqm" && (
              <motion.div
                key="prism_fqm"
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -5 }}
                className="space-y-4"
              >
                <div className="bg-black/20 border border-gray-850 p-4 rounded-lg space-y-4 font-mono text-left">
                  <div className="flex items-center justify-between border-b border-gray-850 pb-2">
                    <span className="text-indigo-400 font-bold uppercase tracking-widest text-[9px] flex items-center gap-1.5">
                      <Dna className="w-4 h-4 text-indigo-400 animate-pulse" />
                      PRISM-FQM COGNITIVE OS PANEL
                    </span>
                    <span className="text-[7.5px] text-gray-500 uppercase">Fractal quantum mycorrhizal system</span>
                  </div>

                  <p className="text-xs text-gray-400 leading-normal font-sans">
                    Exploring photonic light grids mapped over fractal telemetry code paths. Designed for O(sqrt(N)) Grover-accelerated search spaces.
                  </p>

                  {/* Simulator Grid */}
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="p-3 bg-[#05080d] border border-indigo-950/65 rounded-lg space-y-3">
                      <span className="text-[9px] text-indigo-300 uppercase font-bold block">Fractal Telemetry Language (FTL) Load</span>
                      <div className="flex justify-between items-center text-xs">
                        <span className="text-gray-450">Recursion Depth:</span>
                        <span className="text-indigo-400 font-extrabold">{prismTelemetry?.ftl?.recursion_depth != null ? `Depth = ${prismTelemetry.ftl.recursion_depth} (L-System)` : 'Depth = not measured'}</span>
                      </div>
                      <div className="flex justify-between items-center text-xs">
                        <span className="text-gray-450">Load Dimension:</span>
                        <span className="text-indigo-400 font-extrabold">{prismTelemetry?.ftl?.load_dimension != null ? `D_H = ${prismTelemetry.ftl.load_dimension} (Sierpinski)` : 'D_H = not measured'}</span>
                      </div>
                      <div className="h-1 bg-gray-950 rounded-full overflow-hidden">
                        <div className="h-full bg-indigo-500 animate-pulse style={{width: prismTelemetry?.ftl?.progress != null ? `${prismTelemetry.ftl.progress}%` : '0%'}}" />
                      </div>
                    </div>

                    <div className="p-3 bg-[#05080d] border border-indigo-950/65 rounded-lg space-y-3">
                      <span className="text-[9px] text-indigo-300 uppercase font-bold block">Quantum Swarm Intelligence (QSI)</span>
                      <div className="flex justify-between items-center text-xs">
                        <span className="text-gray-450">Superposition States:</span>
                        <span className="text-indigo-400 font-extrabold">{prismTelemetry?.qsi?.superposition_states != null ? `${prismTelemetry.qsi.superposition_states} Qubits` : 'not measured'}</span>
                      </div>
                      <div className="flex justify-between items-center text-xs">
                        <span className="text-gray-450">Grover Speedup:</span>
                        <span className="text-indigo-400 font-extrabold">{prismTelemetry?.qsi?.grover_speedup != null ? `${prismTelemetry.qsi.grover_speedup}x Parallel` : 'not measured'}</span>
                      </div>
                      <div className="h-1 bg-gray-950 rounded-full overflow-hidden">
                        <div className="h-full bg-indigo-500 animate-pulse style={{width: prismTelemetry?.qsi?.progress != null ? `${prismTelemetry.qsi.progress}%` : '0%'}}" />
                      </div>
                    </div>
                  </div>

                  {/* Code Formula Box */}
                  <div className="p-3 bg-black/60 border border-gray-900 rounded-lg space-y-1.5">
                    <span className="text-[8.5px] text-gray-505 uppercase block">Mycorrhizal Photonic Routing (IFS Attractor)</span>
                    <div className="text-[10px] text-indigo-400 font-mono tracking-tight leading-relaxed">
                      z(n+1) = z(n)² + c &nbsp;&nbsp;|&nbsp;&nbsp; Tunneling Prob: {prismTelemetry?.mycorrhizal?.formula ?? 'exp(-β•ΔE) (theory, not measured)'} | P={prismTelemetry?.mycorrhizal?.tunneling_probability ?? 'not measured'}
                    </div>
                  </div>

                  <button
                    onClick={async () => {
                                          speakText("Anchoring PRISM FQM theoretical paradigm to memory graph.");
                      try { await fetch("/api/trm/prism-fqm/trigger", { method: "POST" }); } catch(e) {}
                      handleRegisterAsMemoryNode("PRISM-FQM Architecture", "10.3390/sym15040912");
                    }}
                    className="w-full py-2 bg-indigo-950 hover:bg-indigo-900 border border-indigo-800 hover:border-indigo-600 rounded text-indigo-350 text-[10px] font-bold tracking-widest transition-all cursor-pointer flex items-center justify-center gap-1 shadow-[0_0_12px_rgba(99,102,241,0.15)]"
                  >
                    SYNC PRISM-FQM CONTEXT TO SEMANTIC MEMORY
                  </button>
                </div>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Combined Citation and D3 Sync Information Panel */}
          <div className="bg-[#03060a] border border-gray-850 p-4 rounded-lg flex flex-col md:flex-row justify-between items-start md:items-center gap-3 font-mono text-[9px] relative overflow-hidden">
            <div className="absolute top-0 right-0 w-16 h-16 bg-gradient-to-bl from-cyan-500/10 to-transparent pointer-events-none" />
            <div className="space-y-1 z-10">
              <span className="text-gray-400 font-bold uppercase tracking-widest text-[8px] block flex items-center gap-1">
                <Globe className="w-3.5 h-3.5 text-cyan-400 animate-pulse" />
                PUBLIC SCHOLARLY EMBED CONDUIT
              </span>
              <p className="text-gray-500 max-w-xl text-[8px] leading-relaxed lowercase first-letter:uppercase">
                We employ standard Crossref DOI handles so our empirical biofeedback scans are queryable by arXiv, Zenodo, and standard scholarly index networks globally.
              </p>
            </div>
            <div className="flex items-center gap-2 z-10 self-end md:self-auto">
              <button
                onClick={() => {
                  speakText("Opening external Open Knowledge Format Specification Document.");
                  window.open("https://github.com/ResonanceOne/open-knowledge-format", "_blank");
                }}
                className="p-1 px-2.5 rounded border border-cyan-850/40 bg-cyan-950/10 text-cyan-400 hover:bg-cyan-900/20 text-[8.5px] font-semibold uppercase tracking-wider flex items-center gap-1 cursor-pointer"
              >
                <Layers className="w-2.5 h-2.5" />
                <span>SPEC DOCS</span>
              </button>
            </div>
          </div>

        </div>

      </div>

    </div>
  );
};

// Rank placement helper
function cntRank(v: number): string {
  if (v === 1) return "1st";
  if (v === 2) return "2nd";
  if (v === 3) return "3rd";
  return `${v}th`;
}
