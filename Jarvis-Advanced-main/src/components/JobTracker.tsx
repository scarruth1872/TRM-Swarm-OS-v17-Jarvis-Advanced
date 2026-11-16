import React, { useState, useEffect, useRef } from "react";
import {
  Briefcase,
  Plus,
  Trash2,
  ExternalLink,
  Award,
  Check,
  Search,
  Activity,
  TrendingUp,
  FileText,
  X,
  Database,
  Brain,
  Cloud,
  RefreshCw,
  Wifi,
} from "lucide-react";

interface Job {
  id: string;
  role: string;
  company: string;
  salary: string;
  status: "applied" | "phone_screen" | "interviewing" | "offer";
  url: string;
  dateApplied: string;
  notes: string;
  matchScore: number;
  matchAnalysis: string[];
}

const INITIAL_JOBS: Job[] = [
  {
    id: "job_1",
    role: "Senior AV Systems Integrator",
    company: "ProMedia Tech",
    salary: "$95,000 - $110,000",
    status: "interviewing",
    url: "https://linkedin.com",
    dateApplied: "2026-06-25",
    notes:
      "Second round interview scheduled. Focusing on corporate AV layouts and network infrastructure setups.",
    matchScore: 94,
    matchAnalysis: [
      "AV Training & installation background matches perfectly",
      "Network Administration experience matches infrastructure requirements",
      "Systems design alignment is high",
    ],
  },
  {
    id: "job_2",
    role: "Corporate Wellness Director & Lead Trainer",
    company: "Vanguard Synergy",
    salary: "$85,000",
    status: "phone_screen",
    url: "https://indeed.com",
    dateApplied: "2026-06-28",
    notes:
      "Phone screen scheduled for Friday morning. Recruiter was highly interested in well-rounded fitness program designs.",
    matchScore: 89,
    matchAnalysis: [
      "Inspired-Fitness program design (cardio, strength, flexibility) aligns 100%",
      "Personal Fitness Training background matches leadership requirements",
      "Excellent fit for corporate wellness programs",
    ],
  },
  {
    id: "job_3",
    role: "Network Solutions Engineer",
    company: "NexaStream Solutions",
    salary: "$120,000",
    status: "applied",
    url: "https://ziprecruiter.com",
    dateApplied: "2026-06-29",
    notes:
      "Applied via quick-apply. Tailored resume to highlight local server deployments and network routing architecture.",
    matchScore: 92,
    matchAnalysis: [
      "Network Administration capabilities match deployment profile",
      "Systems Architect alignment is strong",
      "TypeScript/Python scripting experience is highly valued",
    ],
  },
];

export default function JobTracker() {
  const [jobs, setJobs] = useState<Job[]>(INITIAL_JOBS);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newJob, setNewJob] = useState({
    role: "",
    company: "",
    salary: "",
    url: "",
    notes: "",
  });
  const [recommendations, setRecommendations] = useState<Job[]>([]);
  const [loadingRecs, setLoadingRecs] = useState(false);
  const [syncIndicator, setSyncIndicator] = useState<string | null>(null);
  const [persistReady, setPersistReady] = useState(false);
  const initialLoadDone = useRef(false);

  // Load persisted jobs on mount, merge with INITIAL_JOBS if API returns empty
  useEffect(() => {
    const loadPersistedJobs = async () => {
      try {
        const res = await fetch("/api/jobs");
        if (res.ok) {
          const data = await res.json();
          if (data.jobs && data.jobs.length > 0) {
            setJobs(data.jobs);
          }
        }
      } catch (err) {
        console.error("Failed to load persisted jobs:", err);
      } finally {
        initialLoadDone.current = true;
        setPersistReady(true);
      }
    };
    loadPersistedJobs();
  }, []);

  // Auto-save jobs array to server whenever jobs state changes (debounced 1s)
  useEffect(() => {
    if (!initialLoadDone.current) return;
    const timer = setTimeout(async () => {
      try {
        const res = await fetch("/api/jobs", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ jobs }),
        });
        if (res.ok) {
          setSyncIndicator("💾 Auto-saved");
          setTimeout(() => setSyncIndicator(null), 2000);
        }
      } catch (err) {
        console.error("Failed to auto-save jobs:", err);
      }
    }, 1000);
    return () => clearTimeout(timer);
  }, [jobs]);

  const syncSingleJob = async (job: Job) => {
    try {
      await fetch("/api/jobs/sync", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ job }),
      });
    } catch (err) {
      console.error("Failed to sync single job:", err);
    }
  };

  const fetchRecommendations = async () => {
    setLoadingRecs(true);
    try {
      const res = await fetch("/api/jobs/recommendations");
      if (res.ok) {
        const data = await res.json();
        setRecommendations(data.jobs || []);
      }
    } catch (err) {
      console.error("Failed to load recommendations:", err);
    } finally {
      setLoadingRecs(false);
    }
  };

  React.useEffect(() => {
    fetchRecommendations();
  }, []);

  const addRecommendedJob = async (recJob: Job) => {
    if (jobs.some((j) => j.url === recJob.url)) return;

    // Run semantic scoring to potentially update matchScore
    let matchScore = recJob.matchScore;
    let matchAnalysis = recJob.matchAnalysis;
    try {
      const result = await calculateAIMatch(recJob.role, recJob.notes);
      matchScore = result.score;
      matchAnalysis = result.analysis;
    } catch {
      // Keep existing keyword scores on failure
    }

    const newTracked: Job = {
      ...recJob,
      id: "job_" + Date.now(),
      status: "applied",
      matchScore,
      matchAnalysis,
    };
    setJobs((prev) => [newTracked, ...prev]);
    setRecommendations((prev) => prev.filter((r) => r.id !== recJob.id));
    syncSingleJob(newTracked);
  };

  const calculateAIMatch = async (
    roleName: string,
    notesText: string,
  ): Promise<{ score: number; analysis: string[] }> => {
    const description = `${roleName} ${notesText}`;

    // Try Gemini semantic matching with a 5s timeout
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 5000);

      const res = await fetch("/api/jobs/match", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jobDescription: description,
          jobTitle: roleName,
        }),
        signal: controller.signal,
      });
      clearTimeout(timeoutId);

      if (res.ok) {
        const data = await res.json();
        if (typeof data.matchScore === "number") {
          return {
            score: data.matchScore,
            analysis: data.matchAnalysis || [],
          };
        }
      }
    } catch {
      // Fall through to keyword-based scoring on API failure or timeout
    }

    // Keyword-based fallback scoring
    const roleLower = roleName.toLowerCase() + " " + notesText.toLowerCase();
    let score = 50;
    const analysis: string[] = [];

    if (/fit|train|coach|well|athlet|strength|flex|cardio/i.test(roleLower)) {
      score += 22;
      analysis.push("Inspired-Fitness athletic training alignment verified.");
    }
    if (/av|install|audio|video|sound|acoust/i.test(roleLower)) {
      score += 24;
      analysis.push("AV installation & systems design alignment verified.");
    }
    if (/network|admin|server|sys|routing|cloud|infras/i.test(roleLower)) {
      score += 20;
      analysis.push("Network Administration & infrastructure setup verified.");
    }
    if (/script|code|develop|engineer|typescript|python/i.test(roleLower)) {
      score += 15;
      analysis.push("Scripting languages compatibility checked.");
    }

    score = Math.min(score, 100);

    if (analysis.length === 0) {
      analysis.push(
        "Generic profile match based on core architectural principles.",
      );
    }

    return { score, analysis };
  };

  const handleAddJob = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newJob.role || !newJob.company) return;

    const { score, analysis } = await calculateAIMatch(
      newJob.role,
      newJob.notes,
    );

    const created: Job = {
      id: "job_" + Date.now(),
      role: newJob.role,
      company: newJob.company,
      salary: newJob.salary || "N/A",
      status: "applied",
      url: newJob.url || "https://google.com",
      dateApplied: new Date().toISOString().split("T")[0],
      notes: newJob.notes || "No additional context recorded.",
      matchScore: score,
      matchAnalysis: analysis,
    };

    setJobs((prev) => [created, ...prev]);
    setNewJob({ role: "", company: "", salary: "", url: "", notes: "" });
    setShowAddForm(false);
    syncSingleJob(created);
  };

  const moveJob = (id: string, newStatus: Job["status"]) => {
    setJobs((prev) =>
      prev.map((j) => (j.id === id ? { ...j, status: newStatus } : j)),
    );
  };

  const deleteJob = (id: string) => {
    setJobs((prev) => prev.filter((j) => j.id !== id));
  };

  // Pipeline count helpers
  const countStatus = (status: Job["status"]) =>
    jobs.filter((j) => j.status === status).length;

  return (
    <div className="w-full bg-[#07090e]/75 border border-gray-900 rounded-lg p-5 font-mono text-xs text-gray-300 relative overflow-hidden backdrop-blur-md">
      {/* Decorative cyber grid accent */}
      <div className="absolute top-0 right-0 w-32 h-32 bg-cyan-500/5 rounded-full blur-2xl pointer-events-none" />

      {/* Main Panel Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center border-b border-gray-800 pb-4 mb-4 gap-3">
        <div>
          <h2 className="text-sm font-bold tracking-wider text-cyan-400 flex items-center gap-1.5 uppercase">
            <Briefcase className="w-4 h-4" /> J.A.R.V.I.S. Interactive Job
            Tracker
          </h2>
          <p className="text-[10px] text-gray-500 mt-0.5">
            Synchronized with Inspired-Fitness LLC profile & Systems
            Architecture skills.
          </p>
          <div className="flex items-center gap-2 mt-1">
            {persistReady && (
              <span className="inline-flex items-center gap-1 text-[8px] text-emerald-500/60 font-semibold uppercase tracking-wider">
                <Database className="w-2.5 h-2.5" /> Persist Active
              </span>
            )}
            {syncIndicator && (
              <span className="inline-block text-[10px] text-emerald-400/80 animate-pulse">
                💾 {syncIndicator}
              </span>
            )}
          </div>
        </div>
        <button
          onClick={() => setShowAddForm(true)}
          className="flex items-center gap-1.5 bg-cyan-950/40 text-cyan-400 border border-cyan-800/60 hover:bg-cyan-900/30 px-3 py-1.5 rounded font-bold cursor-pointer transition-all"
        >
          <Plus className="w-3.5 h-3.5" /> TRACK NEW JOB
        </button>
      </div>

      {/* Statistics Rings Banner */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <div className="bg-black/30 border border-gray-900/60 rounded p-2.5 flex items-center justify-between">
          <div>
            <span className="text-[9px] text-gray-500 uppercase">
              Total Tracked
            </span>
            <div className="text-lg font-bold text-gray-200 mt-0.5">
              {jobs.length}
            </div>
          </div>
          <Activity className="w-6 h-6 text-gray-600" />
        </div>
        <div className="bg-black/30 border border-gray-900/60 rounded p-2.5 flex items-center justify-between">
          <div>
            <span className="text-[9px] text-gray-500 uppercase">
              Interviewing
            </span>
            <div className="text-lg font-bold text-purple-400 mt-0.5">
              {countStatus("interviewing")}
            </div>
          </div>
          <TrendingUp className="w-6 h-6 text-purple-550/50" />
        </div>
        <div className="bg-black/30 border border-gray-900/60 rounded p-2.5 flex items-center justify-between">
          <div>
            <span className="text-[9px] text-gray-500 uppercase">
              Offers Made
            </span>
            <div className="text-lg font-bold text-emerald-400 mt-0.5">
              {countStatus("offer")}
            </div>
          </div>
          <Award className="w-6 h-6 text-emerald-555/50" />
        </div>
        <div className="bg-black/30 border border-gray-900/60 rounded p-2.5 flex items-center justify-between">
          <div>
            <span className="text-[9px] text-gray-500 uppercase">
              Match Avg
            </span>
            <div className="text-lg font-bold text-cyan-400 mt-0.5">
              {jobs.length
                ? Math.round(
                    jobs.reduce((acc, curr) => acc + curr.matchScore, 0) /
                      jobs.length,
                  )
                : 0}
              %
            </div>
          </div>
          <FileText className="w-6 h-6 text-cyan-600" />
        </div>
      </div>

      {/* J.A.R.V.I.S. Active Recommendations Feed */}
      <div className="bg-black/35 border border-cyan-900/30 rounded p-4 mb-5 space-y-3 relative">
        <div className="flex justify-between items-center border-b border-gray-800/80 pb-2">
          <span className="text-[10px] text-cyan-400 font-bold uppercase tracking-wider flex items-center gap-1.5 animate-pulse">
            <Award className="w-3.5 h-3.5" /> J.A.R.V.I.S. Active Job
            Recommendations
            <span className="text-[7px] font-mono font-normal text-gray-500 bg-gray-950/60 border border-gray-800 px-1 py-0.5 rounded tracking-wider">
              <Cloud className="w-2.5 h-2.5 inline-block mr-0.5" />
              Arbeitnow + Remotive
            </span>
          </span>
          <button
            onClick={fetchRecommendations}
            disabled={loadingRecs}
            className="text-[9px] text-cyan-400 border border-cyan-800/50 bg-cyan-950/20 hover:bg-cyan-900/10 px-2 py-0.5 rounded cursor-pointer transition-colors"
          >
            {loadingRecs ? "SCANNING..." : "REFRESH FEED"}
          </button>
        </div>

        {loadingRecs ? (
          <div className="text-center py-4 text-gray-500 text-[10px] animate-pulse">
            Querying free api streams and matching suitability scores...
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {recommendations.map((rec) => (
              <div
                key={rec.id}
                className="bg-black/40 border border-gray-850 hover:border-cyan-950/50 rounded p-3 flex flex-col justify-between space-y-2"
              >
                <div className="flex justify-between items-start gap-1">
                  <div className="flex-1 min-w-0">
                    <h5 className="font-bold text-gray-200 leading-snug truncate">
                      {rec.role}
                    </h5>
                    <span className="text-[9px] text-gray-500 truncate block">
                      {rec.company}
                    </span>
                  </div>
                  <span className="text-[10px] font-bold text-emerald-400 bg-emerald-950/25 border border-emerald-900/35 px-1.5 py-0.5 rounded shrink-0 flex items-center gap-1">
                    {rec.matchScore}% FIT
                    <Brain
                      className="w-2.5 h-2.5 text-indigo-400/70"
                      title="AI-scored"
                    />
                  </span>
                </div>
                <p className="text-[9px] text-gray-500 line-clamp-2 italic leading-normal">
                  {rec.notes}
                </p>
                <div className="flex gap-2 pt-1.5 border-t border-gray-900">
                  <button
                    onClick={() => addRecommendedJob(rec)}
                    className="flex-1 bg-cyan-950/30 text-cyan-400 border border-cyan-800/40 hover:bg-cyan-900/20 text-[9px] py-1 rounded cursor-pointer font-bold uppercase transition-colors"
                  >
                    Track Job
                  </button>
                  <a
                    href={rec.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="px-2 text-gray-500 hover:text-white border border-gray-850 rounded flex items-center justify-center shrink-0"
                  >
                    <ExternalLink className="w-3 h-3" />
                  </a>
                </div>
              </div>
            ))}
            {recommendations.length === 0 && (
              <div className="col-span-full text-center py-6 text-gray-500 text-[10px] flex flex-col items-center gap-2">
                <Cloud className="w-6 h-6 text-gray-700" />
                <div>
                  No live job suggestions found in cache.
                  <br />
                  <span className="text-gray-600">
                    Click <strong>REFRESH FEED</strong> above to poll Arbeitnow
                    &amp; Remotive APIs for live listings.
                  </span>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Search Engine Direct Link Hub */}
      {/* Source & Scoring Info Bar */}
      <div className="bg-black/40 border border-gray-850/80 rounded p-3 mb-5">
        <div className="flex flex-wrap items-center gap-3">
          {/* API Source indicators */}
          <div className="flex items-center gap-2">
            <span className="text-[8px] text-gray-500 uppercase tracking-wider font-bold">
              Data Sources:
            </span>
            <span className="text-[9px] font-mono bg-blue-950/20 text-blue-400 border border-blue-900/40 px-1.5 py-0.5 rounded inline-flex items-center gap-1">
              <Cloud className="w-2.5 h-2.5" /> Arbeitnow
            </span>
            <span className="text-[9px] font-mono bg-violet-950/20 text-violet-400 border border-violet-900/40 px-1.5 py-0.5 rounded inline-flex items-center gap-1">
              <Wifi className="w-2.5 h-2.5" /> Remotive (Remote)
            </span>
          </div>
          {/* Scoring indicator */}
          <div className="flex items-center gap-2">
            <span className="text-[8px] text-gray-500 uppercase tracking-wider font-bold">
              Scoring:
            </span>
            <span className="text-[9px] font-mono bg-indigo-950/20 text-indigo-400 border border-indigo-900/40 px-1.5 py-0.5 rounded inline-flex items-center gap-1">
              <Brain className="w-2.5 h-2.5" /> Gemini AI Semantic
            </span>
            <span className="text-[8px] text-gray-600">
              (falls back to keyword matching when offline)
            </span>
          </div>
          {/* Persistence indicator */}
          <div className="flex items-center gap-2 ml-auto">
            <span className="text-[9px] font-mono text-emerald-500/70 inline-flex items-center gap-1">
              <Database className="w-2.5 h-2.5" />
              {persistReady ? "Auto-save active" : "Loading..."}
            </span>
          </div>
        </div>
      </div>

      {/* Search Engine Direct Link Hub */}
      <div className="bg-black/40 border border-gray-850/80 rounded p-3 mb-5">
        <span className="text-[9px] text-cyan-400 font-bold uppercase tracking-wider flex items-center gap-1">
          <Search className="w-3 h-3" /> Quick Job Query Links
        </span>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 mt-2">
          <a
            href="https://www.linkedin.com/jobs/search/?keywords=Personal+Trainer+Systems+Architect"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-between bg-blue-950/20 border border-blue-900/40 hover:bg-blue-900/30 text-blue-400 py-1.5 px-2.5 rounded text-[10px] tracking-wide"
          >
            <span>LINKEDIN: FITNESS/SYSTEMS</span>
            <ExternalLink className="w-3.5 h-3.5" />
          </a>
          <a
            href="https://www.indeed.com/jobs?q=Fitness+Trainer+Network+Installation"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-between bg-[#ff5810]/5 border border-[#ff5810]/30 hover:bg-[#ff5810]/15 text-[#ff5810] py-1.5 px-2.5 rounded text-[10px] tracking-wide"
          >
            <span>INDEED: FIT/NETWORK INSTALL</span>
            <ExternalLink className="w-3.5 h-3.5" />
          </a>
          <a
            href="https://www.ziprecruiter.com/jobs-search?search=Fitness+Coach+AV+Technician"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-between bg-emerald-950/20 border border-emerald-900/40 hover:bg-emerald-900/30 text-emerald-400 py-1.5 px-2.5 rounded text-[10px] tracking-wide"
          >
            <span>ZIPRECRUITER: COACH/AV TECH</span>
            <ExternalLink className="w-3.5 h-3.5" />
          </a>
        </div>
      </div>

      {/* Kanban Application Pipeline Columns */}
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-4">
        {/* COLUMN 1: APPLIED */}
        <div className="bg-black/35 border border-gray-900/80 rounded-lg p-3">
          <div className="flex justify-between items-center border-b border-gray-800 pb-1.5 mb-3">
            <span className="font-bold text-gray-400 uppercase tracking-wider">
              Applied ({countStatus("applied")})
            </span>
            <span className="w-1.5 h-1.5 rounded-full bg-yellow-500 animate-pulse" />
          </div>
          <div className="space-y-3">
            {jobs
              .filter((j) => j.status === "applied")
              .map((job) => (
                <JobCard
                  key={job.id}
                  job={job}
                  onMove={moveJob}
                  onDelete={deleteJob}
                />
              ))}
            {countStatus("applied") === 0 && <EmptyState />}
          </div>
        </div>

        {/* COLUMN 2: PHONE SCREEN */}
        <div className="bg-black/35 border border-gray-900/80 rounded-lg p-3">
          <div className="flex justify-between items-center border-b border-gray-800 pb-1.5 mb-3">
            <span className="font-bold text-cyan-400 uppercase tracking-wider">
              Phone Screen ({countStatus("phone_screen")})
            </span>
            <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-pulse" />
          </div>
          <div className="space-y-3">
            {jobs
              .filter((j) => j.status === "phone_screen")
              .map((job) => (
                <JobCard
                  key={job.id}
                  job={job}
                  onMove={moveJob}
                  onDelete={deleteJob}
                />
              ))}
            {countStatus("phone_screen") === 0 && <EmptyState />}
          </div>
        </div>

        {/* COLUMN 3: INTERVIEWING */}
        <div className="bg-black/35 border border-gray-900/80 rounded-lg p-3">
          <div className="flex justify-between items-center border-b border-gray-800 pb-1.5 mb-3">
            <span className="font-bold text-purple-400 uppercase tracking-wider">
              Interviewing ({countStatus("interviewing")})
            </span>
            <span className="w-1.5 h-1.5 rounded-full bg-purple-400 animate-pulse" />
          </div>
          <div className="space-y-3">
            {jobs
              .filter((j) => j.status === "interviewing")
              .map((job) => (
                <JobCard
                  key={job.id}
                  job={job}
                  onMove={moveJob}
                  onDelete={deleteJob}
                />
              ))}
            {countStatus("interviewing") === 0 && <EmptyState />}
          </div>
        </div>

        {/* COLUMN 4: OFFER / CLOSED */}
        <div className="bg-black/35 border border-gray-900/80 rounded-lg p-3">
          <div className="flex justify-between items-center border-b border-gray-800 pb-1.5 mb-3">
            <span className="font-bold text-emerald-400 uppercase tracking-wider">
              Offer / Offer Loop ({countStatus("offer")})
            </span>
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
          </div>
          <div className="space-y-3">
            {jobs
              .filter((j) => j.status === "offer")
              .map((job) => (
                <JobCard
                  key={job.id}
                  job={job}
                  onMove={moveJob}
                  onDelete={deleteJob}
                />
              ))}
            {countStatus("offer") === 0 && <EmptyState />}
          </div>
        </div>
      </div>

      {/* Add Job Modal Dialog */}
      {showAddForm && (
        <div className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4 backdrop-blur-sm">
          <div className="bg-[#0b0e14] border border-cyan-800/40 rounded-lg p-5 w-full max-w-md font-mono text-xs text-gray-300">
            <div className="flex justify-between items-center border-b border-gray-800 pb-3 mb-4">
              <span className="font-bold text-cyan-400 uppercase tracking-wider flex items-center gap-1.5">
                <Briefcase className="w-4 h-4" /> Track New Job Application
              </span>
              <button
                onClick={() => setShowAddForm(false)}
                className="text-gray-500 hover:text-gray-300 cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
            <form onSubmit={handleAddJob} className="space-y-3.5">
              <div>
                <label className="block text-[10px] text-gray-500 uppercase mb-1">
                  Role Title *
                </label>
                <input
                  type="text"
                  required
                  placeholder="e.g. Lead Network Deployment Installer"
                  value={newJob.role}
                  onChange={(e) =>
                    setNewJob((prev) => ({ ...prev, role: e.target.value }))
                  }
                  className="w-full bg-[#050608] border border-gray-850 rounded px-2.5 py-1.5 text-gray-200 placeholder-gray-650 focus:outline-none focus:border-cyan-700"
                />
              </div>
              <div>
                <label className="block text-[10px] text-gray-500 uppercase mb-1">
                  Company *
                </label>
                <input
                  type="text"
                  required
                  placeholder="e.g. Inspired Tech Group"
                  value={newJob.company}
                  onChange={(e) =>
                    setNewJob((prev) => ({ ...prev, company: e.target.value }))
                  }
                  className="w-full bg-[#050608] border border-gray-850 rounded px-2.5 py-1.5 text-gray-200 placeholder-gray-650 focus:outline-none focus:border-cyan-700"
                />
              </div>
              <div>
                <label className="block text-[10px] text-gray-500 uppercase mb-1">
                  Estimated Compensation
                </label>
                <input
                  type="text"
                  placeholder="e.g. $105,000 / year"
                  value={newJob.salary}
                  onChange={(e) =>
                    setNewJob((prev) => ({ ...prev, salary: e.target.value }))
                  }
                  className="w-full bg-[#050608] border border-gray-850 rounded px-2.5 py-1.5 text-gray-200 placeholder-gray-650 focus:outline-none focus:border-cyan-700"
                />
              </div>
              <div>
                <label className="block text-[10px] text-gray-500 uppercase mb-1">
                  Listing URL
                </label>
                <input
                  type="text"
                  placeholder="e.g. https://linkedin.com/jobs/..."
                  value={newJob.url}
                  onChange={(e) =>
                    setNewJob((prev) => ({ ...prev, url: e.target.value }))
                  }
                  className="w-full bg-[#050608] border border-gray-850 rounded px-2.5 py-1.5 text-gray-200 placeholder-gray-650 focus:outline-none focus:border-cyan-700"
                />
              </div>
              <div>
                <label className="block text-[10px] text-gray-500 uppercase mb-1">
                  Application Notes / Description
                </label>
                <textarea
                  rows={3}
                  placeholder="Add details about interview processes or key resume focus points..."
                  value={newJob.notes}
                  onChange={(e) =>
                    setNewJob((prev) => ({ ...prev, notes: e.target.value }))
                  }
                  className="w-full bg-[#050608] border border-gray-850 rounded px-2.5 py-1.5 text-gray-200 placeholder-gray-650 focus:outline-none focus:border-cyan-700"
                />
              </div>
              <div className="flex gap-2 pt-2">
                <button
                  type="submit"
                  className="flex-1 bg-cyan-950/40 text-cyan-400 border border-cyan-800/60 hover:bg-cyan-900/30 py-2 rounded font-bold cursor-pointer transition-all"
                >
                  SAVE & ANALYZE FIT
                </button>
                <button
                  type="button"
                  onClick={() => setShowAddForm(false)}
                  className="bg-gray-900 border border-gray-800 text-gray-400 hover:text-white px-4 py-2 rounded cursor-pointer transition-all"
                >
                  CANCEL
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}

function EmptyState() {
  return (
    <div className="border border-dashed border-gray-850 rounded p-4 text-center text-gray-600 text-[10px]">
      No active entries in column.
    </div>
  );
}

interface CardProps {
  key?: string;
  job: Job;
  onMove: (id: string, newStatus: Job["status"]) => void;
  onDelete: (id: string) => void;
}

function JobCard({ job, onMove, onDelete }: CardProps) {
  // Score color picker
  const getScoreColor = (score: number) => {
    if (score >= 90) return "text-emerald-400";
    if (score >= 80) return "text-cyan-400";
    return "text-yellow-450";
  };

  return (
    <div className="bg-[#090b10] border border-gray-850 rounded p-3 space-y-2.5 hover:border-gray-700 transition-all">
      {/* Card Header */}
      <div className="flex justify-between items-start gap-1">
        <div>
          <h4 className="font-bold text-gray-200 leading-tight">{job.role}</h4>
          <span className="text-[10px] text-gray-455">{job.company}</span>
        </div>
        <button
          onClick={() => onDelete(job.id)}
          className="text-gray-600 hover:text-red-400 cursor-pointer shrink-0 transition-colors"
          title="Remove entry"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Analytics Fit Metric */}
      <div className="bg-black/35 border border-gray-900/50 rounded px-2 py-1.5 flex items-center justify-between text-[10px]">
        <span className="text-gray-500 uppercase">AI SUITABILITY:</span>
        <span
          className={`font-bold flex items-center gap-1 ${getScoreColor(job.matchScore)}`}
        >
          {job.matchScore}%
        </span>
      </div>

      {/* Match details bullets */}
      <div className="space-y-0.5 border-l border-gray-800 pl-1.5 py-0.5">
        {job.matchAnalysis.slice(0, 2).map((item, idx) => (
          <div
            key={idx}
            className="text-[9px] text-gray-500 flex items-center gap-1 leading-normal"
          >
            <Check className="w-2.5 h-2.5 text-cyan-500 shrink-0" />
            <span className="truncate">{item}</span>
          </div>
        ))}
      </div>

      {/* Salary & Meta info */}
      <div className="flex justify-between items-center text-[10px] text-gray-500">
        <span>Est: {job.salary}</span>
        <span>{job.dateApplied}</span>
      </div>

      {/* Notes container */}
      <p className="text-[10px] text-gray-450 italic leading-snug bg-black/10 p-1.5 rounded border border-gray-900/40">
        {job.notes}
      </p>

      {/* Active Progression Controls */}
      <div className="flex gap-1.5 pt-1 border-t border-gray-900/60">
        {job.status !== "applied" && (
          <button
            onClick={() => {
              const stages: Job["status"][] = [
                "applied",
                "phone_screen",
                "interviewing",
                "offer",
              ];
              const prevIndex = stages.indexOf(job.status) - 1;
              onMove(job.id, stages[prevIndex]);
            }}
            className="flex-1 text-[9px] font-bold text-gray-400 bg-gray-900 hover:bg-gray-800 border border-gray-800 py-1 rounded cursor-pointer uppercase transition-colors"
          >
            Back
          </button>
        )}

        {job.status !== "offer" && (
          <button
            onClick={() => {
              const stages: Job["status"][] = [
                "applied",
                "phone_screen",
                "interviewing",
                "offer",
              ];
              const nextIndex = stages.indexOf(job.status) + 1;
              onMove(job.id, stages[nextIndex]);
            }}
            className="flex-1 text-[9px] font-bold text-cyan-400 bg-cyan-950/20 hover:bg-cyan-900/10 border border-cyan-900/40 py-1 rounded cursor-pointer uppercase transition-colors"
          >
            Advance
          </button>
        )}

        {job.url && (
          <a
            href={job.url}
            target="_blank"
            rel="noopener noreferrer"
            className="p-1 text-gray-500 hover:text-white border border-gray-850 rounded flex items-center justify-center shrink-0"
            title="View job posting URL"
          >
            <ExternalLink className="w-3 h-3" />
          </a>
        )}
      </div>
    </div>
  );
}
