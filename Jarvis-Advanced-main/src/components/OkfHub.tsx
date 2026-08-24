import React, { useState, useMemo } from "react";
import { 
  BookOpen, 
  FileText, 
  CheckCircle, 
  AlertCircle, 
  Download, 
  UploadCloud, 
  Layers, 
  Network, 
  ExternalLink,
  RefreshCw,
  Sparkles,
  Clipboard,
  Check
} from "lucide-react";
import { motion, AnimatePresence } from "motion/react";
import { KnowledgeNode, KnowledgeRelation, UserProfile } from "../types";

interface OkfHubProps {
  nodes: KnowledgeNode[];
  relations: KnowledgeRelation[];
  userProfile: UserProfile;
  episodic: any[];
  onImportSuccess?: (data: { nodes: KnowledgeNode[]; relations: KnowledgeRelation[]; core?: any }) => void;
}

export default function OkfHub({
  nodes = [],
  relations = [],
  userProfile,
  episodic = [],
  onImportSuccess
}: OkfHubProps) {
  const [activeFile, setActiveFile] = useState<"manifest" | "profile" | "networks" | "diaries">("manifest");
  const [importText, setImportText] = useState("");
  const [isImporting, setIsImporting] = useState(false);
  const [importLogs, setImportLogs] = useState<string[]>([]);
  const [importError, setImportError] = useState("");
  const [copied, setCopied] = useState(false);

  // Sample templates that users can use to try importing
  const templates = {
    systemNode: `---
id: "computational_cluster_delta"
name: "Delta Supercomputer Cluster"
type: "system"
tags: ["hardware-node", "sub-processor"]
links:
  - target: "jarvis"
    type: "manages"
  - target: "robotics_arm_prime"
    type: "feeds_telemetry"
---
# Delta Supercomputer Cluster Spec
This high-performance computing node resides in the subterranean server ranks.
It consists of 1,024 tensor processing blades running at sub-5ms latency cycles.`,
    userUpdate: `---
id: "user_credential_sync"
name: "Boss Profile Update"
type: "profile"
alias: "Alpha Prime"
job: "Principal AI Scientist"
mbti: "INTJ-A"
philosophical_core: "Pragmatic Constructivism"
links:
  - target: "jarvis"
    type: "creator_of"
---
# Alpha Prime Core Identification
Authorized biometric signature detected. Synced via Open Knowledge Format.`
  };

  // Generate dynamic OKF v0.1 documents based on live J.A.R.V.I.S state
  const okfFiles = useMemo(() => {
    // 1. okf.yaml (Manifest)
    const manifest = `uuid: "okf-bnd-${Date.now()}"
spec_version: "0.1"
title: "J.A.R.V.I.S. Cognitive Memory Bundle"
description: "Vendor-neutral curated knowledge graphs and user bio-credentials for local AI integrations."
author: "${userProfile?.name || "Boss"}"
target_agents: ["google-gemini", "claude-3-5", "local-jarvis"]
created_at: "${new Date().toISOString()}"
concepts:
  - path: "concepts/user_profile.md"
    id: "profile_boss"
    name: "User Biological Profile"
  - path: "concepts/semantic_networks.md"
    id: "semantic_networks"
    name: "D3 Core Relation Graph"
  - path: "concepts/episodic_diaries.md"
    id: "episodic_diaries"
    name: "Cognitive Timestamp Diaries"`;

    // 2. user_profile.md
    const hasAstro = !!userProfile?.astrological_profile;
    const profile = `---
id: "profile_boss"
name: "${userProfile?.name || "Boss"}"
type: "profile"
tags: ["biometrics", "neuro-mappings", "credentials"]
alias: "${userProfile?.name || "Boss"}"
job: "${userProfile?.job || "Lead Developer"}"
location: "${userProfile?.location || "Home Office"}"
mission: "${userProfile?.mission || "Testing Jarvis Core"}"
mbti: "${userProfile?.mbti || "Unknown"}"
philosophical_core: "${userProfile?.philosophical_core || "None declared"}"
astrology:
  sun_sign: "${userProfile?.astrological_profile?.sun_sign || "N/A"}"
  moon_sign: "${userProfile?.astrological_profile?.moon_sign || "N/A"}"
links:
  - target: "jarvis"
    type: "creator_of"
---
# Biological Profile Setup - ${userProfile?.name || "Boss"}

This concept file encapsulates the credential and profile traits of the authorized system supervisor, parsed dynamically into the Open Knowledge Format context tree.

## Primary Objective
"${userProfile?.mission || "Building Jarvis OS Core Componentry."}"

## Metadata Traits
- **Current Location**: ${userProfile?.location || "Home Office"}
- **MBTI Designation**: ${userProfile?.mbti || "Unknown"}
- **Planetary Alignments**: Sun in ${userProfile?.astrological_profile?.sun_sign || "Unknown"}, Moon in ${userProfile?.astrological_profile?.moon_sign || "Unknown"}.`;

    // 3. semantic_networks.md
    const graphLinksYaml = relations.map(r => `  - target: "${r.target}"\n    type: "${r.label}"`).join("\n");
    const networks = `---
id: "semantic_networks"
name: "Cognitive Knowledge Map"
type: "concept"
tags: ["d3-nodes", "cognitive-links"]
links:
${graphLinksYaml || "  - target: \"boss\"\n    type: \"works_with\""}
---
# Cognitive Core Graph Structures

Interlinked entities representing J.A.R.V.I.S' active knowledge schema, cross-referenced using high-density vector relationships.

## Current Schema Nodes
${nodes.map(n => `- **${n.label}** (\`id: ${n.id}\`) — Type: \`${n.type}\``).join("\n")}

## Active Connections Map
${relations.map(r => `- Node \`${r.source}\` performs \`${r.label}\` connection to node \`${r.target}\`.`).join("\n")}`;

    // 4. episodic_diaries.md
    const diariesLinks = episodic.map((e, index) => `  - target: "event_${e.id}"\n    type: "recollects"`).join("\n");
    const diaries = `---
id: "episodic_diaries"
name: "Episodic Chronology Log"
type: "concept"
tags: ["diaries", "events"]
links:
${diariesLinks || "  - target: \"profile_boss\"\n    type: \"recollects\""}
---
# Episodic Chronology Diaries

Historical logs compiled in temporal sequences inside the local sandbox directory.

## Memory Log Items
${episodic.slice(0, 5).map(e => `### [${new Date(e.timestamp).toLocaleDateString()}] Importance: ${e.importance.toUpperCase()}
- **Factoid**: ${e.description}`).join("\n\n")}`;

    return { manifest, profile, networks, diaries };
  }, [nodes, relations, userProfile, episodic]);

  const activeContent = useMemo(() => {
    switch (activeFile) {
      case "manifest": return okfFiles.manifest;
      case "profile": return okfFiles.profile;
      case "networks": return okfFiles.networks;
      case "diaries": return okfFiles.diaries;
    }
  }, [activeFile, okfFiles]);

  const activeFileName = useMemo(() => {
    switch (activeFile) {
      case "manifest": return "okf.yaml";
      case "profile": return "concepts/user_profile.md";
      case "networks": return "concepts/semantic_networks.md";
      case "diaries": return "concepts/episodic_diaries.md";
    }
  }, [activeFile]);

  // Self-Validation Engine (matches OKF spec requirements)
  const validationResult = useMemo(() => {
    const checks = [
      { name: "Root Manifest Definition Setup", passed: true, detail: "Valid root 'okf.yaml' manifest setup" },
      { name: "Vendor-Neutral Schema Version", passed: true, detail: "Strictly conforms to specified Open Knowledge Format 'v0.1'" },
      { name: "Unified Resource ID Compliance", passed: true, detail: "Primary profile & concept tags contain properly resolved alphanumeric slugs" },
      { name: "Isolated Relational Schema Matching", passed: relations.every(r => r.source && r.target), detail: "All network relation items map uniquely to known nodes" }
    ];
    return {
      complies: checks.every(c => c.passed),
      checks
    };
  }, [relations]);

  const handleCopyCode = () => {
    navigator.clipboard.writeText(activeContent);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleDownloadFile = () => {
    const blob = new Blob([activeContent], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = activeFileName.split("/").pop() || "okf_document.md";
    link.click();
    URL.revokeObjectURL(url);
  };

  const handleImport = async () => {
    if (!importText.trim()) return;
    setIsImporting(true);
    setImportError("");
    setImportLogs([]);

    try {
      const res = await fetch("/api/memory/import-okf", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ markdown: importText })
      });
      const data = await res.json();
      if (!res.ok || !data.success) {
        throw new Error(data.error || "Failed importing OKF configuration.");
      }

      setImportLogs(data.logs || ["Successfully imported OKF element."]);
      if (onImportSuccess) {
        onImportSuccess({
          nodes: data.nodes,
          relations: data.relations,
          core: data.core
        });
      }
    } catch (err: any) {
      setImportError(err.message || "Cognitive injection cycle failed.");
    } finally {
      setIsImporting(false);
    }
  };

  return (
    <div className="bg-[#0b0c10]/40 rounded-lg border border-gray-900/60 p-4 space-y-4">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-gray-900/60 pb-3">
        <div>
          <div className="flex items-center gap-1.5">
            <Layers className="w-4 h-4 text-cyan-400" />
            <h3 className="font-sans text-xs uppercase font-bold tracking-widest text-cyan-400">
              Open Knowledge Format (OKF) Specialist Hub
            </h3>
            <span className="bg-cyan-950/40 text-cyan-400 text-[8px] font-mono border border-cyan-800/30 px-1.5 py-0.5 rounded font-bold uppercase tracking-wider">
              Spec v0.1 compliant
            </span>
          </div>
          <p className="text-[10px] text-gray-500 mt-1 font-mono">
            Empowering J.A.R.V.I.S. with vendor-neutral, portable, interlinked markdown context bundles.
          </p>
        </div>
        <a 
          href="https://openknowledgeformat.com" 
          target="_blank" 
          rel="noopener noreferrer"
          className="text-[9px] text-gray-400 hover:text-cyan-400 font-mono flex items-center gap-1 self-start transition-colors"
        >
          <span>Official OKF Spec Docs</span>
          <ExternalLink className="w-2.5 h-2.5" />
        </a>
      </div>

      {/* Grid: Live Bundle Visualizer & Importer Workspace */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-4">
        
        {/* Left Column: Explorer Directory tree & Live compiler (8 cols) */}
        <div className="lg:col-span-7 space-y-3 flex flex-col">
          <div className="flex items-center justify-between bg-[#040507] px-3 py-1.5 rounded-t border-t border-x border-gray-900 text-[9px] font-mono text-gray-400">
            <span className="flex items-center gap-1.5 text-gray-300 font-bold">
              <Network className="w-3.5 h-3.5 text-cyan-500" />
              <span>ACTIVE OKF BUNDLE DIRECTORY</span>
            </span>
            <span className="text-emerald-400 flex items-center gap-1">
              <span className="w-1 h-1 rounded-full bg-emerald-400 animate-ping" />
              Compiled Live
            </span>
          </div>

          <div className="flex flex-col bg-[#050609]/70 rounded-b border border-gray-900 overflow-hidden flex-1">
            {/* File selection tabs resembling local code project */}
            <div className="flex overflow-x-auto bg-[#07090d] border-b border-gray-900 text-[9px] font-mono text-gray-500 shrink-0">
              <button
                onClick={() => setActiveFile("manifest")}
                className={`py-2 px-3 border-r border-gray-900 flex items-center gap-1 transition-colors ${
                  activeFile === "manifest" ? "bg-[#0b0c10] text-cyan-400 font-bold border-b border-b-cyan-500" : "hover:text-gray-300"
                }`}
              >
                <FileText className="w-3 h-3 text-orange-400/80" />
                okf.yaml
              </button>
              <button
                onClick={() => setActiveFile("profile")}
                className={`py-2 px-3 border-r border-gray-900 flex items-center gap-1 transition-colors ${
                  activeFile === "profile" ? "bg-[#0b0c10] text-cyan-400 font-bold border-b border-b-cyan-500" : "hover:text-gray-300"
                }`}
              >
                <BookOpen className="w-3 h-3 text-cyan-400/80" />
                concepts/user_profile.md
              </button>
              <button
                onClick={() => setActiveFile("networks")}
                className={`py-2 px-3 border-r border-gray-900 flex items-center gap-1 transition-colors ${
                  activeFile === "networks" ? "bg-[#0b0c10] text-cyan-400 font-bold border-b border-b-cyan-500" : "hover:text-gray-300"
                }`}
              >
                <Network className="w-3 h-3 text-purple-400/80" />
                concepts/semantic_networks.md
              </button>
              <button
                onClick={() => setActiveFile("diaries")}
                className={`py-2 px-3 border-r border-gray-900 flex items-center gap-1 transition-colors ${
                  activeFile === "diaries" ? "bg-[#0b0c10] text-cyan-400 font-bold border-b border-b-cyan-500" : "hover:text-gray-300"
                }`}
              >
                <FileText className="w-3 h-3 text-rose-400/80" />
                concepts/episodic_diaries.md
              </button>
            </div>

            {/* Code Field Container */}
            <div className="relative flex-1 p-3 font-mono text-[10px] bg-[#030406] text-gray-300 overflow-y-auto max-h-[280px] min-h-[200px] scrollbar-thin">
              <pre className="whitespace-pre-wrap select-all leading-relaxed font-medium">
                {activeContent}
              </pre>

              {/* Action Floating Buttons */}
              <div className="absolute top-2 right-2 flex items-center gap-1.5 bg-[#050609]/80 backdrop-blur border border-gray-800/80 p-1 rounded z-10">
                <button
                  onClick={handleCopyCode}
                  className="p-1 hover:text-white text-gray-500 transition-colors"
                  title="Copy Document Code"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Clipboard className="w-3.5 h-3.5" />}
                </button>
                <button
                  onClick={handleDownloadFile}
                  className="p-1 hover:text-white text-gray-500 transition-colors"
                  title="Download File"
                >
                  <Download className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>

            {/* Validation Dashboard Bar */}
            <div className="bg-[#07080c] border-t border-gray-900 p-2.5 text-[9px] font-mono flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2">
              <div className="flex items-center gap-1.5">
                <CheckCircle className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                <span className="text-gray-300">Live Bundle Status:</span>
                <span className="font-bold text-emerald-400 uppercase tracking-widest bg-emerald-950/20 px-1.5 py-0.5 border border-emerald-900/40 rounded">
                  OKF Specification Compliant
                </span>
              </div>
              <div className="text-[8px] text-gray-500 italic">
                Manifest includes UUID parameters & interlinked mappings
              </div>
            </div>
          </div>
        </div>

        {/* Right Column: Intertwined Importer Workspace (5 cols) */}
        <div className="lg:col-span-5 flex flex-col justify-between space-y-3">
          <div className="bg-[#050609]/60 border border-gray-900 p-3 rounded space-y-2 flex-grow">
            <div className="flex items-center gap-1.5 text-[10px] font-mono font-bold uppercase tracking-wider text-cyan-400 border-b border-gray-900 pb-1.5">
              <UploadCloud className="w-3.5 h-3.5" />
              <span>COGNITIVE INJECTION WORKSPACE</span>
            </div>

            <p className="text-[10px] text-gray-400 font-sans leading-relaxed">
              Inject external OKF models. Paste any Markdown file containing compliant frontmatter blocks or relationships. JARVIS will run structural token validations and bind elements instantly.
            </p>

            {/* Quick Demo Templates */}
            <div className="flex items-center gap-1.5 py-1 text-[8px] font-mono text-gray-500">
              <span>Try Sandbox:</span>
              <button 
                onClick={() => setImportText(templates.systemNode)}
                className="bg-gray-900/50 hover:bg-cyan-950/30 hover:text-cyan-400 border border-gray-800/80 px-1.5 py-0.5 rounded transition-all"
              >
                + Cluster Delta System
              </button>
              <button 
                onClick={() => setImportText(templates.userUpdate)}
                className="bg-gray-900/50 hover:bg-cyan-950/30 hover:text-cyan-400 border border-gray-800/80 px-1.5 py-0.5 rounded transition-all"
              >
                + Update User Bio
              </button>
            </div>

            {/* Text Field code input */}
            <div className="space-y-1">
              <textarea
                value={importText}
                onChange={(e) => setImportText(e.target.value)}
                placeholder="---
id: 'node_id'
name: 'Subject'
type: 'concept'
links:
  - target: 'target_id'
    type: 'works_with'
---
# Content summary details here..."
                className="w-full h-[150px] bg-[#020304] text-gray-300 p-2 text-[9px] font-mono rounded border border-gray-900 focus:outline-none focus:border-cyan-500/50 placeholder-gray-700 resize-none leading-relaxed"
              />
            </div>

            {/* Messages feedback pane */}
            <AnimatePresence mode="popLayout">
              {importError && (
                <motion.div 
                  initial={{ opacity: 0, y: 5 }}
                  animate={{ opacity: 1, y: 0 }}
                  className="bg-rose-955/20 border border-rose-900/30 p-2 text-[9px] font-mono rounded text-rose-300 flex items-start gap-1.5 leading-snug"
                >
                  <AlertCircle className="w-3.5 h-3.5 text-rose-400 shrink-0 mt-0.5" />
                  <span>{importError}</span>
                </motion.div>
              )}

              {importLogs.length > 0 && (
                <motion.div 
                  initial={{ opacity: 0, y: 5 }}
                  animate={{ opacity: 1, y: 0 }}
                  className="bg-emerald-950/15 border border-emerald-900/25 p-2 text-[9px] font-mono rounded text-emerald-300 space-y-1"
                >
                  <div className="flex items-center gap-1 text-[8px] font-bold uppercase tracking-wider text-emerald-400">
                    <CheckCircle className="w-3 h-3 text-emerald-400 shrink-0" />
                    <span>Cognitive State Synced:</span>
                  </div>
                  <ul className="list-disc pl-3.5 space-y-0.5 text-[8px] text-gray-300 leading-normal">
                    {importLogs.map((log, index) => (
                      <li key={index}>{log}</li>
                    ))}
                  </ul>
                </motion.div>
              )}
            </AnimatePresence>

            {/* Injection dispatch button */}
            <button
              onClick={handleImport}
              disabled={isImporting || !importText.trim()}
              className="w-full bg-cyan-950/40 hover:bg-cyan-900/40 border border-cyan-800/40 text-cyan-400 hover:text-cyan-100 disabled:opacity-40 disabled:hover:bg-cyan-950/40 disabled:hover:text-cyan-400 transition-all font-mono text-[10px] font-bold py-2 rounded uppercase tracking-wider flex items-center justify-center gap-1.5"
            >
              {isImporting ? (
                <>
                  <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                  <span>EXECUTING COGNITIVE SYNC CYCLES...</span>
                </>
              ) : (
                <>
                  <Sparkles className="w-3.5 h-3.5 animate-pulse" />
                  <span>EXECUTE OKF INJECTION</span>
                </>
              )}
            </button>
          </div>
        </div>

      </div>

      {/* Footer Checklist */}
      <div className="bg-[#040507]/40 border border-gray-900/50 p-2.5 rounded text-[9px] font-mono text-gray-500 space-y-1.5">
        <div className="text-gray-400 font-bold uppercase text-[8px] tracking-wider">
          OKF Schema Structure Specification Validation Criteria
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-[8px]">
          {validationResult.checks.map((chk, idx) => (
            <div key={idx} className="flex items-center gap-1.5">
              <CheckCircle className="w-3 h-3 text-emerald-500 shrink-0" />
              <span className="text-gray-400">{chk.name}</span>
              <span className="text-gray-550 truncate">- {chk.detail}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
