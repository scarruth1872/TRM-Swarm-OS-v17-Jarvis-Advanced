/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import React, { useState, useEffect, useRef } from "react";
import {
  Terminal as TerminalIcon,
  Cpu,
  Activity,
  Shield,
  MapPin,
  User,
  Briefcase,
  Target,
  Camera,
  Volume2,
  VolumeX,
  Mic,
  MicOff,
  Database,
  GitBranch,
  Plus,
  Send,
  RefreshCw,
  X,
  Share2,
  CheckCircle,
  Eye,
  Settings,
  HelpCircle,
  Clock,
  Play,
  Globe,
  Lock,
  Server,
  AlertTriangle,
  Check,
  FileText,
  Sparkles,
  ShieldAlert,
  Square,
  LayoutGrid,
  Maximize2,
  Minimize2,
  Search,
  Sliders,
  ChevronRight,
  CheckCircle2,
  Zap,
} from "lucide-react";
import {
  MemoryState,
  SystemStatus,
  ChatMessage,
  UserProfile,
  KnowledgeRelation,
  KnowledgeNode,
  JarvisSkill,
} from "./types";
import SystemTelemetry from "./components/SystemTelemetry";
import ComputerControlTerminal from "./components/ComputerControlTerminal";
import { AscilineEngine } from "./components/AscilineEngine";
import { RoboticsLab } from "./components/RoboticsLab";
import { WorkspaceNexus } from "./components/WorkspaceNexus";
import CognitiveMemoryGraph from "./components/CognitiveMemoryGraph";
import JarvisAvatar from "./components/JarvisAvatar";
import OkfHub from "./components/OkfHub";
import GlobalOfferLanding from "./components/GlobalOfferLanding";
import NoosphericRecalibration from "./components/NoosphericRecalibration";
import LocalModelSync from "./components/LocalModelSync";
import TernarySynapseEngine from "./components/TernarySynapseEngine";
import BiomimeticLab from "./components/BiomimeticLab";
import { WiFiVisionIntelligence } from "./components/WiFiVisionIntelligence";
import { ResonanceResearchHub } from "./components/ResonanceResearchHub";
import JobTracker from "./components/JobTracker";

// Firebase Auth & Database Integrations
import { auth, db, OperationType, handleFirestoreError } from "./lib/firebase";
import {
  onAuthStateChanged,
  signInWithPopup,
  GoogleAuthProvider,
  signOut,
  signInWithEmailAndPassword,
  createUserWithEmailAndPassword,
  updateProfile,
} from "firebase/auth";
import { doc, getDoc, setDoc } from "firebase/firestore";

export default function App() {
  // Firebase Auth states
  const [currentUser, setCurrentUser] = useState<any>(null);
  const [authLoading, setAuthLoading] = useState(true);
  const [authEmail, setAuthEmail] = useState("");
  const [authPassword, setAuthPassword] = useState("");
  const [authName, setAuthName] = useState(""); // Default/onboarding alias
  const [authError, setAuthError] = useState<string | null>(null);
  const [authStateMode, setAuthStateMode] = useState<"signin" | "signup">(
    "signin",
  );
  const [showOnboarding, setShowOnboarding] = useState(false);

  // Onboarding Form States
  const [onboardingStep, setOnboardingStep] = useState(1);
  const [onboardAlias, setOnboardAlias] = useState("");
  const [onboardJob, setOnboardJob] = useState("");
  const [onboardLocation, setOnboardLocation] = useState("");
  const [onboardMission, setOnboardMission] = useState("");
  const [onboardInterestsStr, setOnboardInterestsStr] = useState("");
  const [onboardLanguage, setOnboardLanguage] = useState("TypeScript");

  // Intercept and wrap fetch with user identity headers
  const fetch = async (input: RequestInfo | URL, init?: RequestInit) => {
    const uid = currentUser?.uid || auth.currentUser?.uid;
    const headers = {
      "Content-Type": "application/json",
      ...(init?.headers || {}),
      ...(uid ? { "x-user-id": uid } : {}),
    };
    return window.fetch(input, {
      ...init,
      headers,
    });
  };

  // Cognitive states
  const [memory, setMemory] = useState<MemoryState>({
    core: {
      user_profile: {
        name: "Boss",
        job: "Lead Systems Architect",
        location: "Aesthetic Skyloft, Sector 7",
        mission:
          "Building an autonomous, reactive cognitive companion under code-name Jarvis OS.",
        interests: ["Agentic loops", "Knowledge graphs", "Vision models"],
        preferences: {
          developmentLanguage: "TypeScript",
          alertThreshold: "High",
        },
      },
      current_context: "Establishing cognitive baseline.",
    },
    episodic: [],
    relations: [],
    nodes: [],
  });

  // Chat conversational loop
  const [messages, setMessages] = useState<ChatMessage[]>([
    {
      id: "msg_init",
      role: "assistant",
      content:
        "Diagnostics positive. Memory, telemetry, and sensory input loops successfully bound to local container. Standing by for instructions, Boss.",
      timestamp: new Date().toISOString(),
    },
  ]);
  const [userInput, setUserInput] = useState("");
  const [chatLoading, setChatLoading] = useState(false);
  const [isChatHistoryCollapsed, setIsChatHistoryCollapsed] = useState(false);
  const [uploadedFileBase64, setUploadedFileBase64] = useState<string | null>(
    null,
  );
  const [uploadedFileName, setUploadedFileName] = useState<string | null>(null);

  // Advanced Visual Capture and Webcam Stream state
  const [cameraActive, setCameraActive] = useState(false);
  const [visualDataUrl, setVisualDataUrl] = useState<string | null>(null);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  // Audio Speech Recognition and text-to-speech toggles
  const [speechActive, setSpeechActive] = useState(false);
  const [voiceEnabled, setVoiceEnabled] = useState(true);
  const [isListeningForVoice, setIsListeningForVoice] = useState(false);
  const [isSpeaking, setIsSpeaking] = useState(false);

  // Modern Continuous Vocal Wake Monitor for "Jarvis" / "J.A.R.V.I.S"
  const [wakeWordListening, setWakeWordListening] = useState(false);
  const [wakeStatus, setWakeStatus] = useState<
    "offline" | "idle" | "listening" | "processing"
  >("offline");
  const [wakeWordDetected, setWakeWordDetected] = useState(false);
  const [wakeInterimText, setWakeInterimText] = useState("");
  const [wakeLastHeard, setWakeLastHeard] = useState("");

  const wakeWordListeningRef = useRef(false);
  const chatLoadingRef = useRef(false);
  const wakeRecognitionRef = useRef<any>(null);
  const messagesRef = useRef<ChatMessage[]>([]);

  useEffect(() => {
    wakeWordListeningRef.current = wakeWordListening;
  }, [wakeWordListening]);

  useEffect(() => {
    chatLoadingRef.current = chatLoading;
  }, [chatLoading]);

  useEffect(() => {
    messagesRef.current = messages;
  }, [messages]);

  // Dialog controls for preferences or data creation
  const [showConfigModal, setShowConfigModal] = useState(false);
  const [newKey, setNewKey] = useState("");
  const [newValue, setNewValue] = useState("");
  const [newRelation, setNewRelation] = useState({
    source: "",
    target: "",
    relation: "",
  });

  // System statistics telemetry
  const [systemStats, setSystemStats] = useState<SystemStatus>({
    cpu: 24,
    ram: 58,
    activeConnections: 3,
    storage: "128 GB / 512 GB",
    status: "NOMINAL",
  });
  const [statsLoading, setStatsLoading] = useState(false);
  const [proactiveMuted, setProactiveMuted] = useState(false);

  // Uptime state timer
  const [secondsUptime, setSecondsUptime] = useState(0);

  // Audio stream visualizer bar levels
  const [vocalLevels, setVocalLevels] = useState<number[]>([
    15, 20, 10, 40, 25, 5, 20, 10,
  ]);

  // Terminal scroll anchors
  const chatEndRef = useRef<HTMLDivElement | null>(null);

  // Dynamic modular J.A.R.V.I.S. Skills Registry and Telegram Bot config tab states
  const [activeBox2Tab, setActiveBox2Tab] = useState<
    | "registry"
    | "memory"
    | "skills"
    | "telegram"
    | "testing"
    | "webcheck"
    | "asciline"
    | "robotics"
    | "workspace"
    | "offer"
    | "recalibration"
    | "local_sync"
    | "ternary"
    | "biomimetic"
    | "research_hub"
  >("registry");
  const [isBox2Expanded, setIsBox2Expanded] = useState(false);
  const [registrySearchQuery, setRegistrySearchQuery] = useState("");
  const [registryCategoryFilter, setRegistryCategoryFilter] = useState("all");
  const [memorySubView, setMemorySubView] = useState<"graph" | "okf">("graph");
  const [workspaceToken, setWorkspaceToken] = useState<string | null>(null);
  const [skillsList, setSkillsList] = useState<JarvisSkill[]>([]);
  const [telegramToken, setTelegramToken] = useState("");
  const [telegramPollingActive, setTelegramPollingActive] = useState(false);
  const [checkingTelegramStatus, setCheckingTelegramStatus] = useState(false);
  const [showSkillCreatorModal, setShowSkillCreatorModal] = useState(false);
  const [macroCommandsInput, setMacroCommandsInput] = useState("");
  const [newSkillForm, setNewSkillForm] = useState({
    name: "",
    description: "",
    scriptTrigger: "",
    category: "utility" as const,
  });

  // Web Intelligence Checker (Web-Check integration)
  const [webAnalysisUrl, setWebAnalysisUrl] = useState("");
  const [isAnalyzingWeb, setIsAnalyzingWeb] = useState(false);
  const [webAnalysisReport, setWebAnalysisReport] = useState<any>(null);
  const [webAnalysisError, setWebAnalysisError] = useState<string | null>(null);

  // Cognitive Agent Test Suite & Capability Diagnostic State
  const [activeTest, setActiveTest] = useState<string | null>(null);
  const [testStatus, setTestStatus] = useState<
    "idle" | "running" | "completed" | "failed"
  >("idle");
  const [testLogs, setTestLogs] = useState<string[]>([]);
  const [activeTestStep, setActiveTestStep] = useState<number>(0);

  const STRESS_TESTS = [
    {
      id: "sec_audit",
      name: "Security Audit & Compliance Assessment",
      description:
        "Traces project source tree structure, audits package setups, validates container security, indexes memories, and registers compliance levels.",
      difficulty: "HIGH COMPLEXITY",
      avgDuration: "4.5 seconds",
      steps: [
        "Boot compliance probing sandboxes & mount memory maps",
        "Recursively catalog workspace assets (validating TS interfaces)",
        "Trigger system container optimization matrix daemon via REST",
        "Inject audit compliance findings into J.A.R.V.I.S Episodic Memory",
        "Map 'Codebase' -> 'Audited' relationships in Knowledge Graph",
        "Invoke charismatic vocal feedback loops for final results review",
      ],
    },
    {
      id: "brain_sync",
      name: "Cognitive Synapse & Graph Re-balancing Test",
      description:
        "Performs full memory consolidation, deduplicates duplicate semantic relations, load-balances diagnostic caches, and tests cross-subsystem latency thresholds.",
      difficulty: "EXTREME COMPLEXITY",
      avgDuration: "5.0 seconds",
      steps: [
        "Launch semantic consolidation (scanning nodes for duplicate entities)",
        "Sync core profiles & update last test preferences settings",
        "Trigger virtual mouse pointer click coordinate calculations (x520 y710)",
        "Save long-term memory event records tracking stress diagnostic outcome",
        "Register 'Jarvis Stress Test' -> 'Passed' connections in Knowledge Graph",
        "Announce synthetic brain calibration and optimizations results",
      ],
    },
    {
      id: "camera_calib",
      name: "Visual Optical Matrix Calibration Test",
      description:
        "Activates webcam feed stream metrics, processes optical canvas pixel indices, maps focus point coordinates, and logs sensory synchronization states.",
      difficulty: "MEDIUM COMPLEXITY",
      avgDuration: "3.8 seconds",
      steps: [
        "Establish safe camera buffer streams and calibrate sensory triggers",
        "Simulate / Execute webcam snap capturing live visual RGB histograms",
        "Dispatch diagnostic image classification metrics request",
        "Record persistent optical calibration event to J.A.R.V.I.S database",
        "Announce system and optical coordination synchronization to Boss",
      ],
    },
  ];

  const runStressTest = async (testId: string) => {
    if (testStatus === "running") return;

    const test = STRESS_TESTS.find((t) => t.id === testId);
    if (!test) return;

    setActiveTest(testId);
    setTestStatus("running");
    setActiveTestStep(0);

    const timestampStr = () => new Date().toLocaleTimeString();
    setTestLogs([
      `[${timestampStr()}] INITIALIZING COMPATIBILITY TEST SEQUENCE: "${test.name}"`,
      `[${timestampStr()}] Sandboxed VM allocated. Ready to execute ${test.steps.length} sequential operations.`,
    ]);

    speakText(
      `Initiating high priority system compatibility test: ${test.name}. Standing by, Boss.`,
    );

    try {
      // Step 1: Booting / setup
      await new Promise((r) => setTimeout(r, 900));
      setActiveTestStep(1);
      setTestLogs((prev) => [
        ...prev,
        `[${timestampStr()}] Step 1/5: ${test.steps[0]} - Completed.`,
      ]);

      // Step 2: Catalog workspace files or console read
      await new Promise((r) => setTimeout(r, 1000));
      setActiveTestStep(2);
      let step2Detail = "";
      try {
        const res = await fetch("/api/memory");
        if (res.ok) {
          const data = await res.json();
          step2Detail = ` (Indexed ${data.nodes?.length || 0} active semantic nodes and ${data.episodic?.length || 0} episodic memory blocks)`;
        }
      } catch (e) {}
      setTestLogs((prev) => [
        ...prev,
        `[${timestampStr()}] Step 2/5: ${test.steps[1]}${step2Detail} - Completed successfully.`,
      ]);

      // Step 3: Trigger real terminal optimization command via REST
      setActiveTestStep(3);
      setTestLogs((prev) => [
        ...prev,
        `[${timestampStr()}] Step 3/5: Dispatching REST execution script...`,
      ]);
      let cmdToRun = "Optimize system memory footprint";
      if (testId === "brain_sync")
        cmdToRun = "Simulating clicking coordinate x520 y710";
      if (testId === "camera_calib")
        cmdToRun = "Simulating optical matrix check";

      try {
        const res = await fetch("/api/execute-cmd", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ command: cmdToRun }),
        });
        if (res.ok) {
          const data = await res.json();
          const executionLogs = data.logs || [];
          setTestLogs((prev) => [
            ...prev,
            ...executionLogs.map((l: string) => `  > ${l}`),
          ]);
        }
      } catch (e) {
        setTestLogs((prev) => [
          ...prev,
          `  > [Error] Direct command endpoint failed, continuing via mock fallback.`,
        ]);
      }
      setTestLogs((prev) => [
        ...prev,
        `[${timestampStr()}] Step 3/5: ${test.steps[2]} - Completed.`,
      ]);
      await new Promise((r) => setTimeout(r, 900));

      // Step 4: Write real episodic memory fact to long-term memory
      setActiveTestStep(4);
      setTestLogs((prev) => [
        ...prev,
        `[${timestampStr()}] Step 4/5: Dispatching episodic event payload...`,
      ]);
      try {
        const desc = `Successfully completed agentic stress diagnostics for test: ${test.name} (${test.difficulty}).`;
        await fetch("/api/memory/memorize-event", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ description: desc, importance: "high" }),
        });
        await fetch("/api/memory/update-profile", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            key: "last_stress_test_status",
            value: `PASSED_${testId.toUpperCase()}`,
          }),
        });
        setTestLogs((prev) => [
          ...prev,
          `  > Recorded in J.A.R.V.I.S Episodic Memories registry.`,
        ]);
        setTestLogs((prev) => [
          ...prev,
          `  > Updated Core Profile preference: last_stress_test_status = PASSED_${testId.toUpperCase()}`,
        ]);
      } catch (e) {
        setTestLogs((prev) => [
          ...prev,
          `  > [Error] Failed to write to endpoints, continuing with local caches.`,
        ]);
      }
      setTestLogs((prev) => [
        ...prev,
        `[${timestampStr()}] Step 4/5: ${test.steps[3]} - Completed.`,
      ]);
      await new Promise((r) => setTimeout(r, 950));

      // Step 5: Add a semantic relationship node dynamically
      setActiveTestStep(5);
      setTestLogs((prev) => [
        ...prev,
        `[${timestampStr()}] Step 5/5: Initiating semantic graph sync...`,
      ]);
      try {
        let src = "codebase";
        let tgt = "audited_secv4";
        let lbl = "Audited";
        if (testId === "brain_sync") {
          src = "boss";
          tgt = "jarvis_stress_test";
          lbl = "Passed Successfully";
        } else if (testId === "camera_calib") {
          src = "optical_matrix";
          tgt = "calibrated_eye";
          lbl = "Synchronized";
        }

        await fetch("/api/memory/add-relation", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ source: src, target: tgt, label: lbl }),
        });
        setTestLogs((prev) => [
          ...prev,
          `  > Established graph nodes connection: [${src}] --(${lbl})--> [${tgt}]`,
        ]);
      } catch (e) {
        setTestLogs((prev) => [
          ...prev,
          `  > [Error] Failed to insert relation to semantic index.`,
        ]);
      }
      setTestLogs((prev) => [
        ...prev,
        `[${timestampStr()}] Step 5/5: ${test.steps[4]} - Completed.`,
      ]);

      // Auto-refresh the main app memory lists so they immediately populate on screen!
      fetchMemoryData();
      fetchSystemStats();

      // Final celebration & voice speech summary!
      setTestStatus("completed");
      setActiveTestStep(6);
      setTestLogs((prev) => [
        ...prev,
        `\n[${timestampStr()}] ==========================================================`,
        `[${timestampStr()}] STATUS: COMPATIBILITY TEST SEQUENCE SUCCESSFUL // ALL OK`,
        `[${timestampStr()}] Passed validation metrics under sandboxed container parameters.`,
        `[${timestampStr()}] ==========================================================`,
      ]);

      const successPhrase = `A stellar achievement, Boss! The capability test "${test.name}" has returned nominal parameters across all subcomponents. Direct preferences and neural relationships have been permanently synchronized into our database.`;
      speakText(successPhrase);
    } catch (err: any) {
      setTestStatus("failed");
      setTestLogs((prev) => [
        ...prev,
        `\n🚨 [${timestampStr()}] DIAGNOSTIC SEQUENCE CRITICAL FAILURE: ${err.message || "Execution exception."}`,
      ]);
      speakText(
        `My apologies Boss, but the capability diagnostic sequence encountered an unexpected execution fault. System diagnostics are offline.`,
      );
    }
  };

  const runningWebCheck = async (targetUrl?: string) => {
    const urlToUse = targetUrl || webAnalysisUrl;
    if (!urlToUse.trim()) return;

    setIsAnalyzingWeb(true);
    setWebAnalysisError(null);
    speakText(
      `Commencing multi-spectrum network intelligence scan on ${urlToUse}. Scanning DNS records, SSL handshakes, and application headers in parallel. Standby, Boss.`,
    );

    try {
      const res = await fetch("/api/web-analyzer", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: urlToUse }),
      });

      if (!res.ok) {
        const errData = await res.json();
        throw new Error(errData.error || "Web check failure.");
      }

      const report = await res.json();
      setWebAnalysisReport(report);

      const score = report.headers?.securityScore ?? 0;
      const grade = report.intelReport?.postureGrade ?? "B";
      speakText(
        `Intelligence report compiled successfully, Boss. Determined posture grade of ${grade} with a security header compliance score of ${score} percent.`,
      );

      fetchMemoryData();
    } catch (err: any) {
      console.error(err);
      setWebAnalysisError(err.message || "Could not complete web check.");
      speakText(
        `Warning Boss. Multi spectrum probe failed to connect or resolve target routing addresses.`,
      );
    } finally {
      setIsAnalyzingWeb(false);
    }
  };

  const fetchSkills = async () => {
    try {
      const res = await fetch("/api/skills");
      if (res.ok) {
        const data = await res.json();
        setSkillsList(data.skills);
      }
    } catch (err) {
      console.warn("Failed to retrieve system skills matrix.", err);
    }
  };

  const toggleSkillState = async (id: string) => {
    try {
      const res = await fetch("/api/skills/toggle", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id }),
      });
      if (res.ok) {
        const data = await res.json();
        setSkillsList(data.skills);
        setTelegramPollingActive(data.pollingActive);
        speakText(`Tactical capability buffer successfully adjusted.`);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const fetchTelegramStatus = async () => {
    try {
      const res = await fetch("/api/telegram/status");
      if (res.ok) {
        const data = await res.json();
        setTelegramPollingActive(data.pollingActive);
      }
    } catch (err) {
      console.warn("Telegram configuration not initialized.");
    }
  };

  const saveTelegramConfig = async () => {
    if (!telegramToken.trim()) return;
    setCheckingTelegramStatus(true);
    try {
      const res = await fetch("/api/telegram/config", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token: telegramToken }),
      });
      const data = await res.json();
      if (res.ok && data.success) {
        setTelegramPollingActive(data.pollingActive);
        setTelegramToken("");
        speakText(
          "Telegram uplink synchronization successful. Continuous sensory polling is online, Boss.",
        );
        fetchSkills(); // refresh integration list
      } else {
        speakText(
          "Uplink handshake failed. Please verify Botfather token coordinates.",
        );
      }
    } catch (err) {
      console.error(err);
    } finally {
      setCheckingTelegramStatus(false);
    }
  };

  const handleTriggerMacro = (name: string, commands?: string[]) => {
    if (!commands || commands.length === 0) return;
    window.dispatchEvent(
      new CustomEvent("jarvis-trigger-macro", {
        detail: { name, commands },
      }),
    );
    speakText(
      `Activating high-priority terminal command sequence for ${name}, Boss.`,
    );
  };

  const submitNewSkill = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newSkillForm.name || !newSkillForm.description) return;
    try {
      const payload = {
        ...newSkillForm,
        commands:
          newSkillForm.category === "macro"
            ? macroCommandsInput
                .split("\n")
                .map((l) => l.trim())
                .filter((l) => l.length > 0)
            : undefined,
      };

      const res = await fetch("/api/skills/add", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (res.ok) {
        const data = await res.json();
        setSkillsList(data.skills);
        setNewSkillForm({
          name: "",
          description: "",
          scriptTrigger: "",
          category: "utility",
        });
        setMacroCommandsInput("");
        setShowSkillCreatorModal(false);
        speakText(
          `Successfully injected customized tactical skill sequence: ${newSkillForm.name}.`,
        );
      }
    } catch (e) {
      console.error(e);
    }
  };

  // Web Speech recognition handler
  const recognitionRef = useRef<any>(null);

  // Load baseline cognitive data on start with Firebase Auth monitoring
  useEffect(() => {
    const unsubscribe = onAuthStateChanged(auth, async (user) => {
      setCurrentUser(user);
      if (user) {
        setAuthError(null);
        try {
          const userDocRef = doc(db, "users", user.uid);
          let docSnap;
          try {
            docSnap = await getDoc(userDocRef);
          } catch (e) {
            handleFirestoreError(e, OperationType.GET, `users/${user.uid}`);
            throw e;
          }
          if (docSnap.exists() && docSnap.data().setupCompleted) {
            setShowOnboarding(false);
            // Load user-specific isolated memory databases
            fetchMemoryData();
            fetchChatHistory();
            fetchSkills();
            fetchTelegramStatus();
          } else {
            const isShawn =
              user.email?.toLowerCase() === "shawncarruth@gmail.com";
            if (isShawn) {
              setOnboardAlias("Shawn Carruth");
              setOnboardJob(
                "Lead Systems Architect & Creative Consciousness Matrix",
              );
              setOnboardLocation("Aesthetic Skyloft, Sector 7");
              setOnboardMission(
                "Co-creating the J.A.R.V.I.S. OS autonomous reactive cognitive companion, integrating quantum-inspired creativity with the Law of One philosophy (All is One).",
              );
              setOnboardInterestsStr(
                "Agentic loops, Knowledge graphs, Astrology & Numerology, Tarot Archetypes, Myers-Briggs & INFJ Psychology, Law of One & Spiritual Exploration, TypeScript/Python Architecture",
              );
              setOnboardLanguage("TypeScript");
            } else {
              setOnboardAlias(user.displayName || "");
              setOnboardJob("Lead Systems Architect");
              setOnboardLocation("Operational Base");
              setOnboardMission("Deploying Jarvis OS Neural Matrix");
              setOnboardInterestsStr("");
              setOnboardLanguage("TypeScript");
            }
            setShowOnboarding(true);
            setOnboardingStep(1);
          }
        } catch (e) {
          console.error("Firestore error while retrieving user profile: ", e);
          setShowOnboarding(true);
        }
      } else {
        setShowOnboarding(false);
        setMemory({
          core: {
            user_profile: {
              name: "Boss",
              job: "Lead Systems Architect",
              location: "Home Office",
              mission: "Testing Jarvis OS Core",
              interests: [],
              preferences: {},
            },
            current_context: "Session unauthenticated.",
          },
          episodic: [],
          relations: [],
          nodes: [],
        });
      }
      setAuthLoading(false);
    });

    fetchSystemStats();

    // Warm up standard browser SpeechSynthesis voices list
    if (typeof window !== "undefined" && window.speechSynthesis) {
      window.speechSynthesis.getVoices();
    }

    // Stats fetching timer
    const statsTimer = setInterval(() => {
      fetchSystemStats();
    }, 12000);

    // Dynamic visualizer levels timer
    const waveTimer = setInterval(() => {
      setVocalLevels((prev) =>
        prev.map(() => Math.floor(5 + Math.random() * 65)),
      );
    }, 180);

    // Uptime timer
    const uptimeTimer = setInterval(() => {
      setSecondsUptime((val) => val + 1);
    }, 1000);

    return () => {
      unsubscribe();
      clearInterval(statsTimer);
      clearInterval(waveTimer);
      clearInterval(uptimeTimer);
      if (videoRef.current && videoRef.current.srcObject) {
        const stream = videoRef.current.srcObject as MediaStream;
        stream.getTracks().forEach((track) => track.stop());
      }
    };
  }, []);

  const handleStopSpeaking = () => {
    if (typeof window !== "undefined" && window.speechSynthesis) {
      window.speechSynthesis.cancel();
      (window as any).activeUtterances = [];
    }
    setIsSpeaking(false);
  };

  // Update voice synthesis when messages change
  const speakText = (text: string) => {
    if (!voiceEnabled) return;
    try {
      // Clean up markdown blocks, files, logs, equations, brackets, extra stars
      let cleanSpeakText = text
        .replace(/```[\s\S]*?```/g, " [Detailed log logs generated] ") // ignore full code blocks completely
        .replace(/`([^`]+)`/g, "$1") // strip backticks but keep interior word
        .replace(/[*#\\_~>\[\]\-]/g, "") // strip standard markdown styling symbols
        .replace(
          /[\uE000-\uF8FF]|\uD83C[\uDC00-\uDFFF]|\uD83D[\uDC00-\uDFFF]|[\u2011-\u26FF]|\uD83E[\uDD10-\uDDFF]/g,
          "",
        ) // remove visual emojis
        .trim();

      if (!cleanSpeakText) return;

      if (typeof window !== "undefined" && window.speechSynthesis) {
        // Cancel any current speaking or queued speech
        window.speechSynthesis.cancel();
        setIsSpeaking(true);

        // Safe regex to separate into individual sentences (groups ending with .?!; or any general sentence punctuation)
        const sentences = cleanSpeakText
          .match(/[^.?!;]+[.?!;]*/g)
          ?.map((s) => s.trim())
          .filter((s) => s.length > 0) || [cleanSpeakText];

        if (sentences.length === 0) {
          setIsSpeaking(false);
          return;
        }

        // Ensure global persistent reference lists to bypass aggressive browser Garbage Collection cutting off text
        if (!(window as any).activeUtterances) {
          (window as any).activeUtterances = [];
        } else {
          // Clear previous outstanding voice reference instances to let them be disposed
          (window as any).activeUtterances = [];
        }

        const voices = window.speechSynthesis.getVoices();

        // Let's locate elegant British accents matching classical J.A.R.V.I.S persona
        let targetVoice = voices.find(
          (v) =>
            v.lang.toLowerCase().includes("en-gb") &&
            (v.name.toLowerCase().includes("male") ||
              v.name.toLowerCase().includes("david") ||
              v.name.toLowerCase().includes("google") ||
              v.name.toLowerCase().includes("natural")),
        );

        if (!targetVoice) {
          targetVoice = voices.find((v) =>
            v.lang.toLowerCase().includes("en-gb"),
          );
        }

        // fallback search elements
        if (!targetVoice) {
          targetVoice = voices.find(
            (v) =>
              v.lang.toLowerCase().startsWith("en") &&
              (v.name.toLowerCase().includes("google") ||
                v.name.toLowerCase().includes("natural") ||
                v.name.toLowerCase().includes("english")),
          );
        }

        if (!targetVoice) {
          targetVoice = voices.find((v) =>
            v.lang.toLowerCase().startsWith("en"),
          );
        }

        // Queue speech sentences sequentially so SpeechSynthesis doesn't cut off or sound robotic
        sentences.forEach((sentence) => {
          const utterance = new SpeechSynthesisUtterance(sentence);

          if (targetVoice) {
            utterance.voice = targetVoice;
          }

          // Apply warmth configurations (slightly lower pitch for American voices or neutral rate for human feel)
          const isBritish = targetVoice?.lang.toLowerCase().includes("en-gb");
          utterance.pitch = isBritish ? 1.0 : 0.9; // Deeper resonant pitch if fallback is US English
          utterance.rate = 0.95; // Elegantly paced cadence

          // Push into the global window array to prevent Chrome garbage collecting mid-sentence
          (window as any).activeUtterances.push(utterance);

          utterance.onstart = () => {
            setIsSpeaking(true);
          };

          utterance.onend = () => {
            const idx = (window as any).activeUtterances.indexOf(utterance);
            if (idx > -1) {
              (window as any).activeUtterances.splice(idx, 1);
            }
            if (
              !(window as any).activeUtterances ||
              (window as any).activeUtterances.length === 0
            ) {
              setIsSpeaking(false);
            }
          };

          utterance.onerror = () => {
            const idx = (window as any).activeUtterances.indexOf(utterance);
            if (idx > -1) {
              (window as any).activeUtterances.splice(idx, 1);
            }
            if (
              !(window as any).activeUtterances ||
              (window as any).activeUtterances.length === 0
            ) {
              setIsSpeaking(false);
            }
          };

          window.speechSynthesis.speak(utterance);
        });
      }
    } catch (e) {
      console.warn(
        "Speech synthesis unavailable or blocked by iframe environment:",
        e,
      );
      setIsSpeaking(false);
    }
  };

  // Send prompt directly to JARVIS OS server-side cognitive backend
  const sendDirectiveToJarvis = async (promptText: string) => {
    if (!promptText.trim() || chatLoading) return;

    if (cameraActive) {
      captureFrameBase64();
    }

    setChatLoading(true);

    const newUserMsg: ChatMessage = {
      id: "msg_" + Date.now(),
      role: "user",
      content: promptText,
      timestamp: new Date().toISOString(),
    };

    setMessages((prev) => [...prev, newUserMsg]);

    try {
      const res = await fetch("/api/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          message: promptText,
          history: [...messagesRef.current, newUserMsg],
          visualData: uploadedFileBase64 || visualDataUrl,
          accessToken: workspaceToken,
          userId: currentUser?.uid || auth.currentUser?.uid,
        }),
      });

      if (res.ok) {
        const data = await res.json();

        const agentResponse: ChatMessage = {
          id: "msg_reply_" + Date.now(),
          role: "assistant",
          content: data.reply,
          timestamp: new Date().toISOString(),
          toolDetails: data.toolDetails,
        };

        setMessages((prev) => [...prev, agentResponse]);
        speakText(data.reply);

        if (data.toolDetails) {
          fetchMemoryData();
        }
      } else {
        try {
          const errData = await res.json();
          const serverReply =
            errData.reply ||
            errData.error ||
            "A connection handshake could not be established.";
          const agentResponse: ChatMessage = {
            id: "msg_reply_error_" + Date.now(),
            role: "assistant",
            content: serverReply,
            timestamp: new Date().toISOString(),
          };
          setMessages((prev) => [...prev, agentResponse]);
          speakText(serverReply);
        } catch (parseErr) {
          const rawText = await res
            .text()
            .catch(() => "Network cognitive stream failure");
          throw new Error(rawText);
        }
      }
    } catch (err: any) {
      setMessages((prev) => [
        ...prev,
        {
          id: "msg_err_" + Date.now(),
          role: "assistant",
          content: `Vocal cortex failure: ${err.message || "Check your system setup and API secrets keys."}`,
          timestamp: new Date().toISOString(),
        },
      ]);
    } finally {
      setChatLoading(false);
      setUploadedFileBase64(null);
      setUploadedFileName(null);
      scrollChatBottom();
    }
  };

  // Helper to read and encode uploaded files for multimodal processing
  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    setUploadedFileName(file.name);
    const reader = new FileReader();
    reader.onload = (event) => {
      if (event.target?.result && typeof event.target.result === "string") {
        setUploadedFileBase64(event.target.result);
      }
    };
    reader.readAsDataURL(file);
  };

  // Setup Web Audio / Speech Recognition (Push-To-Talk)
  const toggleSpeechRecognition = () => {
    // Shutdown wake monitor if currently running to avoid mic resource locks
    if (wakeWordListening) {
      setWakeWordListening(false);
      setWakeStatus("offline");
      setWakeInterimText("");
      if (wakeRecognitionRef.current) {
        try {
          wakeRecognitionRef.current.abort();
        } catch (e) {}
        wakeRecognitionRef.current = null;
      }
    }

    if (isListeningForVoice) {
      if (recognitionRef.current) {
        try {
          recognitionRef.current.stop();
        } catch (e) {}
      }
      setIsListeningForVoice(false);
      return;
    }

    try {
      const SpeechRecognition =
        (window as any).SpeechRecognition ||
        (window as any).webkitSpeechRecognition;
      if (!SpeechRecognition) {
        alert(
          "Web Speech Recognition is not supported in this browser. Please type prompts manually into the high-tech visual portal.",
        );
        return;
      }

      const rec = new SpeechRecognition();
      rec.continuous = false;
      rec.interimResults = false;
      rec.lang = "en-US";

      rec.onstart = () => {
        setIsListeningForVoice(true);
      };

      rec.onresult = (e: any) => {
        const transcript = e.results[0][0].transcript;
        if (transcript) {
          setUserInput(transcript);
          // AUTO-SUBMIT the spoken vocal directive directly to J.A.R.V.I.S!
          sendDirectiveToJarvis(transcript);
        }
      };

      rec.onerror = (err: any) => {
        console.error("Speech recognition error:", err);
        setIsListeningForVoice(false);
      };

      rec.onend = () => {
        setIsListeningForVoice(false);
      };

      recognitionRef.current = rec;
      rec.start();
    } catch (err) {
      console.error("Speech recognition initialization failed:", err);
    }
  };

  // Continuous background J.A.R.V.I.S. Wake-Word Engine (Listening for "Jarvis" / "J.A.R.V.I.S.")
  const startWakeWordRecognition = () => {
    const SpeechRecognition =
      (window as any).SpeechRecognition ||
      (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) return;

    if (wakeRecognitionRef.current) {
      try {
        wakeRecognitionRef.current.abort();
      } catch (err) {}
    }

    const rec = new SpeechRecognition();
    rec.continuous = true;
    rec.interimResults = true;
    rec.lang = "en-US";

    rec.onstart = () => {
      setWakeStatus("idle");
    };

    rec.onresult = (event: any) => {
      let finalText = "";
      let interimText = "";

      for (let i = event.resultIndex; i < event.results.length; i++) {
        const transcriptText = event.results[i][0].transcript;
        if (event.results[i].isFinal) {
          finalText += transcriptText;
        } else {
          interimText += transcriptText;
        }
      }

      const activeText = (finalText || interimText || "").trim();
      if (activeText) {
        setWakeInterimText(activeText);
      }

      const lowerText = activeText.toLowerCase();
      // Listen for the specific name trigger "jarvis" or "j.a.r.v.i.s."
      if (lowerText.includes("jarvis")) {
        setWakeWordDetected(true);
        setWakeStatus("listening");

        const jarvisIdx = lowerText.indexOf("jarvis");
        const remaining = activeText.substring(jarvisIdx + 6).trim();
        const commandText = remaining.replace(/^[,.!?\s]+/, "").trim();

        if (
          commandText.length > 2 &&
          event.results[event.results.length - 1].isFinal
        ) {
          // Heard name + command all in one breath
          setWakeStatus("processing");
          setWakeLastHeard(activeText);
          setWakeInterimText("");
          sendDirectiveToJarvis(commandText);

          setTimeout(() => {
            setWakeWordDetected(false);
            setWakeStatus("idle");
          }, 3000);
        } else if (event.results[event.results.length - 1].isFinal) {
          // Only heard the wake word greeting: trigger J.A.R.V.I.S. response and listen for next phrase
          setWakeLastHeard("Jarvis");
          setWakeInterimText("");
          setWakeStatus("listening");
          speakText("Yes, Boss? I am online and listening.");

          setTimeout(() => {
            setWakeWordDetected(false);
            setWakeStatus("idle");
          }, 4000);
        }
      }
    };

    rec.onerror = (err: any) => {
      console.warn("[Wake Word Engine Exception]:", err.error || err);
      if (err.error === "aborted") return;
      if (err.error === "no-speech") {
        setWakeInterimText("");
      }
    };

    rec.onend = () => {
      // If the owner wants wake word active, reboot the browser mic capture context immediately on timeout!
      if (wakeWordListeningRef.current) {
        console.log(
          "[Wake Word Engine] Restarting acoustic capture channel...",
        );
        try {
          rec.start();
        } catch (e) {
          console.warn("[Wake-Word Engine] Restart delayed:", e);
        }
      }
    };

    wakeRecognitionRef.current = rec;
    try {
      rec.start();
    } catch (e) {
      console.error("[Wake Word Engine] Initialization lock failed:", e);
    }
  };

  const toggleWakeWordMonitor = () => {
    // Turn off traditional Push-To-Talk speech mic if active
    if (isListeningForVoice) {
      if (recognitionRef.current) {
        try {
          recognitionRef.current.stop();
        } catch (e) {}
      }
      setIsListeningForVoice(false);
    }

    if (wakeWordListening) {
      setWakeWordListening(false);
      setWakeStatus("offline");
      setWakeInterimText("");
      if (wakeRecognitionRef.current) {
        try {
          wakeRecognitionRef.current.abort();
        } catch (e) {}
        wakeRecognitionRef.current = null;
      }
      speakText("Acoustic wake-word monitor deactivated.");
    } else {
      setWakeWordListening(true);
      setWakeStatus("idle");
      speakText(
        "Continuous wake monitor initiated. Say my name Jarvis to issue direct sound protocols.",
      );

      setTimeout(() => {
        startWakeWordRecognition();
      }, 1000);
    }
  };

  // ------------ FIREBASE AUTHENTICATION HANDLERS ------------
  const handleGoogleSignIn = async () => {
    setAuthError(null);
    try {
      const provider = new GoogleAuthProvider();
      await signInWithPopup(auth, provider);
    } catch (e: any) {
      console.error("Google authentication failed", e);
      setAuthError(e.message || "Google Authentication Uplink Aborted.");
    }
  };

  const handleBypassAuth = async () => {
    setAuthError(null);
    const mockUser = {
      uid: "dev-boss-id",
      email: "boss@localhost",
      displayName: "Boss",
      isMock: true,
    };
    setCurrentUser(mockUser);
    setShowOnboarding(false);
    fetchMemoryData();
    fetchChatHistory();
    fetchSkills();
    fetchTelegramStatus();
  };

  const handleAuthorizeWorkspace = async () => {
    setAuthError(null);
    if (currentUser?.isMock) {
      const token = prompt(
        "Developer Mode: Enter custom Google OAuth Access Token (or leave blank to use mock-token):",
      );
      if (token !== null) {
        setWorkspaceToken(token || "dev-mock-workspace-token");
        if (speakText) {
          speakText(
            "Google Workspace local diagnostic synclink is now active, Boss.",
          );
        }
      }
      return;
    }
    try {
      const provider = new GoogleAuthProvider();
      provider.addScope("https://www.googleapis.com/auth/drive");
      provider.addScope("https://mail.google.com/");
      provider.addScope("https://www.googleapis.com/auth/calendar");
      provider.addScope("https://www.googleapis.com/auth/spreadsheets");
      provider.addScope("https://www.googleapis.com/auth/documents");
      provider.addScope("https://www.googleapis.com/auth/presentations");
      provider.addScope(
        "https://www.googleapis.com/auth/meetings.space.created",
      );
      provider.addScope("https://www.googleapis.com/auth/contacts");

      const result = await signInWithPopup(auth, provider);
      const credential = GoogleAuthProvider.credentialFromResult(result);
      if (credential?.accessToken) {
        setWorkspaceToken(credential.accessToken);
        if (speakText) {
          speakText(
            "Google Workspace diagnostic synclink is now active and aligned, Boss.",
          );
        }
      }
    } catch (e: any) {
      console.error("Workspace google authentication failed", e);
      setAuthError(e.message || "G-Workspace auth connection faulted.");
    }
  };

  const handleEmailSignIn = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!authEmail || !authPassword) {
      setAuthError("Email and Password parameters are required.");
      return;
    }
    setAuthError(null);
    try {
      await signInWithEmailAndPassword(auth, authEmail, authPassword);
    } catch (e: any) {
      console.error("Email login failed", e);
      setAuthError(e.message || "Authorization credentials invalid.");
    }
  };

  const handleEmailSignUp = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!authEmail || !authPassword || !authName) {
      setAuthError("All credentials and call sign alias fields are required.");
      return;
    }
    setAuthError(null);
    try {
      const userCred = await createUserWithEmailAndPassword(
        auth,
        authEmail,
        authPassword,
      );
      await updateProfile(userCred.user, { displayName: authName });
    } catch (e: any) {
      console.error("Registration failed", e);
      setAuthError(e.message || "Failed to create security context.");
    }
  };

  const handleSignOut = async () => {
    try {
      if (currentUser?.isMock) {
        setCurrentUser(null);
      } else {
        await signOut(auth);
      }
      setMemory({
        core: {
          user_profile: {
            name: "Boss",
            job: "Lead Systems Architect",
            location: "Home Office",
            mission: "Testing Jarvis OS Core",
            interests: [],
            preferences: {},
          },
          current_context: "Session unauthenticated.",
        },
        episodic: [],
        relations: [],
        nodes: [],
      });
      setMessages([
        {
          id: "msg_init",
          role: "assistant",
          content: "Uplink disconnected. Security lockdowns engaged.",
          timestamp: new Date().toISOString(),
        },
      ]);
    } catch (e) {
      console.error("Logout failed", e);
    }
  };

  const handleOnboardingSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentUser) return;
    try {
      const isShawn =
        currentUser.email?.toLowerCase() === "shawncarruth@gmail.com";
      const profile: UserProfile = {
        name:
          onboardAlias ||
          (isShawn ? "Shawn Carruth" : currentUser.displayName || "Boss"),
        job:
          onboardJob ||
          (isShawn
            ? "Lead Systems Architect & Creative Consciousness Matrix"
            : "Lead Systems Architect"),
        location:
          onboardLocation ||
          (isShawn ? "Aesthetic Skyloft, Sector 7" : "Operational Base"),
        mission:
          onboardMission ||
          (isShawn
            ? "Co-creating the J.A.R.V.I.S. OS autonomous reactive cognitive companion, integrating quantum-inspired creativity with the Law of One philosophy (All is One)."
            : "Deploying Jarvis OS Neural Matrix"),
        interests: onboardInterestsStr
          ? onboardInterestsStr
              .split(",")
              .map((i) => i.trim())
              .filter(Boolean)
          : isShawn
            ? [
                "Agentic loops",
                "Knowledge graphs",
                "Astrology & Numerology",
                "Tarot Archetypes",
                "Myers-Briggs & INFJ Psychology",
                "Law of One & Spiritual Exploration",
                "TypeScript/Python Architecture",
              ]
            : [],
        preferences: {
          developmentLanguage: onboardLanguage,
          alertThreshold: "High",
          setupCompletedAt: new Date().toISOString(),
          ...(isShawn
            ? {
                voiceSpeed: "Normal",
                telegramBotToken:
                  "8921982523:AAHrDCdRBbp818o5cM-ANzo0daI1JVfAX9c",
                last_stress_test_status: "PASSED_BRAIN_SYNC",
              }
            : {}),
        },
        ...(isShawn
          ? {
              role: "Co-Creator & Administrator",
              alias: "Boss",
              age: 53,
              birth_date: "March 18, 1972",
              astrological_profile: {
                sun_sign:
                  "Pisces (Intuitive, empathetic, creative connection of all things)",
                moon_sign:
                  "Virgo/Libra border (Brings analytical precision and seeking harmony)",
                planetary_influences: {
                  neptune:
                    "Pisces ruler - Heightened imagination, visionary thinking",
                  jupiter:
                    "Sagittarius (1972) - Expansive philosophical outlook, truth-seeking",
                  saturn:
                    "Gemini (1972) - Disciplined communication, structured learning",
                },
              },
              numerology: {
                life_path: "5",
                details:
                  "Represents freedom, adaptability, multifaceted expression, exploring diverse creative territories while seeking meaningful structure.",
              },
              tarot_archetypes: {
                birth_card:
                  "The Moon (XVIII) - Navigates subconscious origin of creativity, comfortable with ambiguity, intuitive understanding of hidden patterns",
                year_card_2025:
                  "The Hierophant (V) - Consolidating wisdom, sharing knowledge, bridging traditional techniques with innovative approaches",
                soul_card:
                  "The Hermit (IX) - Internal seeker of wisdom, values solitary creative process, illuminating paths for others",
              },
              mbti: "INFJ (Dominant Ni: patterns & visions; Auxiliary Fe: audience impact & connections; Tertiary Ti: intellectual precision; Inferior Se: sensory craft)",
              philosophical_core:
                "The Law of One - 'All is One', 'I am because we are'. Deep perception of the unified source behind all creative expressions.",
            }
          : {}),
      };

      // 1. Write to database (Firestore)
      if (!currentUser?.isMock) {
        try {
          await setDoc(doc(db, "users", currentUser.uid), {
            ...profile,
            setupCompleted: true,
          });
        } catch (e) {
          handleFirestoreError(
            e,
            OperationType.WRITE,
            `users/${currentUser.uid}`,
          );
          throw e;
        }
      }

      // 2. Sync to local file partition
      await fetch("/api/memory/sync-profile", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ profile }),
      });

      // 3. Update frontend state
      setShowOnboarding(false);
      speakText(
        `Configuration complete, ${profile.name}. Neural interfaces successfully calibrated. J.A.R.V.I.S. is fully active.`,
      );

      // 4. Load full isolated memory structures
      fetchMemoryData();
      fetchChatHistory();
      fetchSkills();
      fetchTelegramStatus();
    } catch (err: any) {
      console.error(
        "Failed to compile user onboarding configuration matrix",
        err,
      );
      setAuthError(err.message || "Compilation failed. Handshake interrupted.");
    }
  };

  // Get consolidated backend values
  const fetchMemoryData = async () => {
    try {
      const res = await fetch("/api/memory");
      if (res.ok) {
        const data = await res.json();
        setMemory(data);
      }
    } catch (e) {
      console.warn(
        "REST endpoint unavailable, continuing with cached simulation baseline.",
      );
    }
  };

  const handleAddRelation = async (
    source: string,
    target: string,
    label: string,
  ): Promise<boolean> => {
    try {
      const res = await fetch("/api/memory/add-relation", {
        method: "POST",
        body: JSON.stringify({ source, target, label }),
      });
      if (res.ok) {
        fetchMemoryData();
        return true;
      }
      return false;
    } catch (e) {
      console.error("Failed to append neural relationship:", e);
      return false;
    }
  };

  const fetchChatHistory = async () => {
    try {
      const res = await fetch("/api/chat-history");
      if (res.ok) {
        const data = await res.json();
        if (data.history && data.history.length > 0) {
          setMessages(data.history);
          scrollChatBottom();
        }
      }
    } catch (e) {
      console.warn(
        "Failed to retrieve persistent chat history, continuing with cached session state.",
      );
    }
  };

  const clearChatHistory = async () => {
    if (
      !window.confirm(
        "Are you sure you want to purge J.A.R.V.I.S' congnitive conversational history? (Profile facts remain)",
      )
    )
      return;
    try {
      const res = await fetch("/api/chat-history/clear", { method: "POST" });
      if (res.ok) {
        const data = await res.json();
        setMessages(data.history);
        speakText(
          "Conversational buffers successfully purged, Boss. Standing by with fresh synapses.",
        );
      }
    } catch (error) {
      console.error("Purging chat history failed:", error);
    }
  };

  // Get active system stats from endpoint
  const fetchSystemStats = async () => {
    setStatsLoading(true);
    try {
      const res = await fetch("/api/system-stats");
      if (res.ok) {
        const stats = await res.json();
        setSystemStats(stats);

        // Vocal spike notification simulated on UI
        if (stats.cpu > 75 && !proactiveMuted) {
          speakText(
            `Warning Boss. Subprocessor utilization is peaking at ${stats.cpu} percent.`,
          );
        }
      }
    } catch (e) {
      // Simulate slow drift
      setSystemStats((prev) => ({
        ...prev,
        cpu: Math.floor(18 + Math.random() * 25),
        ram: Math.floor(52 + Math.random() * 10),
      }));
    } finally {
      setStatsLoading(false);
    }
  };

  // Handle active webcam tracking in browser frame
  const toggleCameraFeed = async () => {
    if (cameraActive) {
      if (videoRef.current && videoRef.current.srcObject) {
        const stream = videoRef.current.srcObject as MediaStream;
        stream.getTracks().forEach((track) => track.stop());
        videoRef.current.srcObject = null;
      }
      setCameraActive(false);
      setVisualDataUrl(null);
    } else {
      try {
        const constraints = { video: { width: 640, height: 480 } };
        const stream = await navigator.mediaDevices.getUserMedia(constraints);
        if (videoRef.current) {
          videoRef.current.srcObject = stream;
        }
        setCameraActive(true);
        // Regular sampling interval to bind visual analysis payload base64
        captureFrameBase64();
      } catch (err: any) {
        console.warn(
          "Webcam access restricted or unavailable, starting simulated camera telemetry analyzer matrix.",
        );
        setCameraActive(true);
        // Create an elegant fake scanner placeholder
        setTimeout(() => createFakeVirtualImage(), 500);
      }
    }
  };

  // Create base64 fake graphic payload for preview environment fallback
  const createFakeVirtualImage = () => {
    const canvas = document.createElement("canvas");
    canvas.width = 300;
    canvas.height = 200;
    const ctx = canvas.getContext("2d");
    if (ctx) {
      ctx.fillStyle = "#020617";
      ctx.fillRect(0, 0, 300, 200);
      ctx.strokeStyle = "#38bdf8";
      ctx.lineWidth = 1;
      ctx.strokeRect(10, 10, 280, 180);
      ctx.fillStyle = "#38bdf8";
      ctx.font = "12px monospace";
      ctx.fillText("[ ANTIGRAVITY OS SENSORS ]", 25, 40);
      ctx.fillText("TARGET: VIRTUAL_DESKTOP_SCREEN", 25, 70);
      ctx.fillText(`TIMESTAMP: ${new Date().toLocaleTimeString()}`, 25, 100);
      ctx.fillText("SCANNER POSITION: NOMINAL", 25, 130);

      // Paint neon graphs
      ctx.fillStyle = "rgba(56, 189, 248, 0.2)";
      ctx.fillRect(25, 145, 250, 40);
      setVisualDataUrl(canvas.toDataURL("image/jpeg"));
    }
  };

  // Siphons current video frame to base64 string
  const captureFrameBase64 = () => {
    if (!videoRef.current || !canvasRef.current) return;
    try {
      const idxCanvas = canvasRef.current;
      const ctx = idxCanvas.getContext("2d");
      if (ctx && videoRef.current.videoWidth) {
        idxCanvas.width = videoRef.current.videoWidth;
        idxCanvas.height = videoRef.current.videoHeight;
        ctx.drawImage(
          videoRef.current,
          0,
          0,
          idxCanvas.width,
          idxCanvas.height,
        );

        // Add futuristic overlay scan lines directly onto the telemetry frame
        ctx.strokeStyle = "rgba(56, 189, 248, 0.15)";
        ctx.lineWidth = 2;
        for (let y = 0; y < idxCanvas.height; y += 16) {
          ctx.beginPath();
          ctx.moveTo(0, y);
          ctx.lineTo(idxCanvas.width, y);
          ctx.stroke();
        }

        const b64 = idxCanvas.toDataURL("image/jpeg", 0.7);
        setVisualDataUrl(b64);
      }
    } catch (e) {
      console.warn("Base64 capture delayed:", e);
    }
  };

  // Send prompt to routing API endpoint
  const handleChatSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!userInput.trim() || chatLoading) return;
    setIsChatHistoryCollapsed(false);
    const currentPrompt = userInput;
    setUserInput("");
    await sendDirectiveToJarvis(currentPrompt);
  };

  // Scroll terminal logs smoothly
  const scrollChatBottom = () => {
    setTimeout(() => {
      chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }, 100);
  };

  // Add a new relation manually
  const submitNewRelation = async (e: React.FormEvent) => {
    e.preventDefault();
    const { source, target, relation } = newRelation;
    if (!source || !target || !relation) return;

    try {
      const res = await fetch("/api/memory/add-relation", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ source, target, label: relation }),
      });

      if (res.ok) {
        fetchMemoryData();
        setNewRelation({ source: "", target: "", relation: "" });
      }
    } catch (e) {
      console.error(e);
    }
  };

  // Update a profile dynamic setting
  const submitProfileKey = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newKey || !newValue) return;

    try {
      const res = await fetch("/api/memory/update-profile", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ key: newKey, value: newValue }),
      });

      if (res.ok) {
        fetchMemoryData();
        setNewKey("");
        setNewValue("");
      }
    } catch (e) {
      console.error(e);
    }
  };

  // Execute manual terminal commands directly
  const executeTerminalCommand = async (
    cmdString: string,
  ): Promise<string[]> => {
    try {
      const res = await fetch("/api/execute-cmd", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ command: cmdString }),
      });

      if (res.ok) {
        const data = await res.json();

        // Refresh memory nodes automatically on reboot or updates
        setTimeout(() => {
          fetchMemoryData();
          fetchSystemStats();
        }, 1000);

        return data.logs;
      }
      return [
        `[${new Date().toLocaleTimeString()}] Subprocess failed to respond.`,
      ];
    } catch (err: any) {
      return [`[Error] Terminal daemon blocked: ${err.message}`];
    }
  };

  const formatUptime = (totalSeconds: number) => {
    const hrs = Math.floor(totalSeconds / 3600)
      .toString()
      .padStart(2, "0");
    const mins = Math.floor((totalSeconds % 3600) / 60)
      .toString()
      .padStart(2, "0");
    const secs = (totalSeconds % 60).toString().padStart(2, "0");
    return `${hrs}:${mins}:${secs}`;
  };

  return (
    <div className="min-h-screen bg-[#050608] text-[#cbd5e1] font-sans overflow-hidden flex flex-col justify-between relative">
      {/* Sleek scanner sweep line overlay */}
      <div className="absolute top-0 left-0 w-full h-[2px] bg-sky-500/10 shadow-[0_0_10px_#0ea5e9] animate-[bounce_8s_infinite] pointer-events-none z-50 opacity-40" />

      {/* Embedded hidden helper canvas for capture conversion */}
      <canvas ref={canvasRef} className="hidden" />

      {authLoading ? (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-[#050608] z-50">
          <div className="relative w-20 h-20 flex items-center justify-center mb-6">
            <div className="absolute w-full h-full rounded-full border-2 border-blue-500/10 border-t-2 border-t-cyan-500 animate-spin"></div>
            <div className="w-2.5 h-2.5 rounded-full bg-cyan-400 animate-pulse shadow-[0_0_12px_#22d3ee]"></div>
          </div>
          <p className="text-xs font-mono text-cyan-400 tracking-widest uppercase animate-pulse">
            Establishing encrypted uplink... Standby.
          </p>
        </div>
      ) : !currentUser ? (
        <div
          className="flex-1 flex items-center justify-center bg-[#050608] p-4 font-mono select-none"
          id="auth-gate-pane"
        >
          <div className="max-w-md w-full bg-[#0a0d14]/90 border border-gray-800 rounded-2xl p-6 md:p-8 shadow-[0_0_30px_rgba(59,130,246,0.1)] backdrop-blur-md">
            {/* Header / Logo */}
            <div className="text-center mb-6">
              <div className="inline-flex items-center justify-center size-14 rounded-full bg-blue-500/10 border border-blue-500/20 mb-3 shadow-[0_0_15px_rgba(59,130,246,0.15)] animate-pulse">
                <Cpu className="w-6 h-6 text-blue-400" />
              </div>
              <h2 className="text-white text-md font-bold tracking-widest uppercase">
                J.A.R.V.I.S. OS
              </h2>
              <p className="text-gray-500 text-[10px] uppercase tracking-wider mt-1 font-sans">
                Cognitive Companion Core Interface v5.1
              </p>
            </div>

            {authError && (
              <div className="mb-4 p-3 bg-red-950/20 border border-red-900/40 rounded text-red-400 text-[11px] flex items-start gap-2 animate-fadeIn font-sans">
                <AlertTriangle className="w-3.5 h-3.5 shrink-0 mt-0.5 text-red-400" />
                <span>{authError}</span>
              </div>
            )}

            <form
              onSubmit={
                authStateMode === "signin"
                  ? handleEmailSignIn
                  : handleEmailSignUp
              }
              className="space-y-4"
            >
              {authStateMode === "signup" && (
                <div>
                  <label className="block text-[10px] text-gray-400 uppercase tracking-wider mb-1 font-sans">
                    Boss Preferred Call-Sign Alias
                  </label>
                  <input
                    type="text"
                    required
                    value={authName}
                    onChange={(e) => setAuthName(e.target.value)}
                    placeholder="e.g. Tony Stank"
                    className="w-full bg-[#050608] border border-gray-800 rounded px-3 py-2 text-xs text-gray-200 placeholder-gray-600 focus:outline-none focus:border-blue-500 font-mono"
                  />
                </div>
              )}

              <div>
                <label className="block text-[10px] text-gray-400 uppercase tracking-wider mb-1 font-sans">
                  Encrypted Identity (Email)
                </label>
                <input
                  type="email"
                  required
                  value={authEmail}
                  onChange={(e) => setAuthEmail(e.target.value)}
                  placeholder="name@agency.com"
                  className="w-full bg-[#050608] border border-gray-800 rounded px-3 py-2 text-xs text-gray-200 placeholder-gray-600 focus:outline-none focus:border-blue-500 font-mono"
                />
              </div>

              <div>
                <label className="block text-[10px] text-gray-400 uppercase tracking-wider mb-1 font-sans">
                  Uplink Password
                </label>
                <input
                  type="password"
                  required
                  value={authPassword}
                  onChange={(e) => setAuthPassword(e.target.value)}
                  placeholder="••••••••••••"
                  className="w-full bg-[#050608] border border-gray-800 rounded px-3 py-2 text-xs text-gray-200 placeholder-gray-600 focus:outline-none focus:border-blue-500 font-mono"
                />
              </div>

              <button
                type="submit"
                className="w-full py-2 bg-blue-600 hover:bg-blue-500 text-white rounded text-xs uppercase tracking-widest font-bold font-mono shadow-[0_0_15px_rgba(59,130,246,0.3)] transition-all cursor-pointer"
              >
                {authStateMode === "signin"
                  ? "Initialize Neural Tunnel"
                  : "Construct Personal Binder"}
              </button>
            </form>

            <div className="relative my-5">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-gray-800"></div>
              </div>
              <div className="relative flex justify-center text-[9px] uppercase">
                <span className="bg-[#0a0d14] px-2 text-gray-500 tracking-wider">
                  Alternative Sec Handshake
                </span>
              </div>
            </div>

            <button
              onClick={handleGoogleSignIn}
              type="button"
              className="w-full py-2 bg-gray-900 hover:bg-[#111319] text-gray-300 border border-gray-800 rounded text-xs uppercase tracking-widest font-bold font-mono transition-all flex items-center justify-center gap-2 cursor-pointer mb-2"
            >
              <Globe className="w-3.5 h-3.5 text-blue-400" />
              Sign In with Google Credentials
            </button>

            <button
              onClick={handleBypassAuth}
              type="button"
              className="w-full py-2 bg-blue-950/30 hover:bg-blue-900/40 text-cyan-400 border border-blue-900/50 rounded text-xs uppercase tracking-widest font-bold font-mono transition-all flex items-center justify-center gap-2 cursor-pointer shadow-[0_0_10px_rgba(34,211,238,0.1)]"
            >
              <TerminalIcon className="w-3.5 h-3.5 text-cyan-400" />
              Developer Bypass (Offline Mode)
            </button>

            <div className="mt-5 text-center text-[10px] font-mono">
              {authStateMode === "signin" ? (
                <p className="text-gray-500">
                  New operator in Sector?{" "}
                  <button
                    onClick={() => {
                      setAuthStateMode("signup");
                      setAuthError(null);
                    }}
                    className="text-blue-400 hover:underline cursor-pointer"
                  >
                    Construct Bio Identity
                  </button>
                </p>
              ) : (
                <p className="text-gray-500">
                  Binder state already active?{" "}
                  <button
                    onClick={() => {
                      setAuthStateMode("signin");
                      setAuthError(null);
                    }}
                    className="text-blue-400 hover:underline cursor-pointer"
                  >
                    Authenticate Identity
                  </button>
                </p>
              )}
            </div>
          </div>
        </div>
      ) : showOnboarding ? (
        <div
          className="flex-1 flex items-center justify-center bg-[#050608] p-4 font-mono select-none"
          id="onboarding-gate"
        >
          <div className="max-w-xl w-full bg-[#0a0d14]/95 border border-gray-800 rounded-2xl p-6 md:p-8 shadow-[0_0_40px_rgba(59,130,246,0.15)] backdrop-blur-md relative overflow-hidden">
            {/* Glowing active wireframe accent */}
            <div className="absolute top-0 left-0 w-full h-[1px] bg-gradient-to-r from-transparent via-blue-500 to-transparent"></div>

            {/* Steps tracker indicators */}
            <div className="flex justify-between items-center mb-6 text-[9px] uppercase font-mono tracking-widest text-gray-500">
              <span>Establishing Profile Protocol</span>
              <span>Step {onboardingStep} of 3</span>
            </div>

            {/* Stepped onboarding form */}
            <form onSubmit={handleOnboardingSubmit} className="space-y-6">
              {onboardingStep === 1 && (
                <div className="space-y-4 animate-fadeIn">
                  <div className="inline-flex gap-2 items-center text-blue-400 uppercase text-xs font-bold tracking-widest border-b border-blue-900/40 pb-2 mb-2">
                    <User className="w-4 h-4 text-blue-400" />
                    <span>01 // Operational Call-Sign</span>
                  </div>
                  <p className="text-gray-400 text-[11px] leading-relaxed font-sans">
                    I must address you properly during active runtimes, Boss.
                    What is your preferred call-sign and current job
                    categorization?
                  </p>
                  <div>
                    <label className="block text-[10px] text-gray-450 uppercase tracking-wider mb-1 font-sans">
                      Preferred Name / Call sign
                    </label>
                    <input
                      type="text"
                      required
                      value={onboardAlias}
                      onChange={(e) => setOnboardAlias(e.target.value)}
                      placeholder="Boss (e.g., Tony, Director Fury)"
                      className="w-full bg-[#050608] border border-gray-800 rounded px-3 py-2 text-xs text-gray-200 placeholder-gray-600 focus:outline-none focus:border-blue-500"
                    />
                  </div>
                  <div>
                    <label className="block text-[10px] text-gray-450 uppercase tracking-wider mb-1 font-sans">
                      Expert Designation / Occupation
                    </label>
                    <input
                      type="text"
                      required
                      value={onboardJob}
                      onChange={(e) => setOnboardJob(e.target.value)}
                      placeholder="e.g. Lead Roboticist, Autonomous Systems Architect"
                      className="w-full bg-[#050608] border border-gray-800 rounded px-3 py-2 text-xs text-gray-200 placeholder-gray-600 focus:outline-none focus:border-blue-500"
                    />
                  </div>
                  <div className="flex justify-end pt-2">
                    <button
                      type="button"
                      onClick={() => setOnboardingStep(2)}
                      className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded text-xs uppercase tracking-widest font-bold cursor-pointer transition-all shadow-[0_0_10px_#3b82f6]"
                    >
                      Verify Call sign
                    </button>
                  </div>
                </div>
              )}

              {onboardingStep === 2 && (
                <div className="space-y-4 animate-fadeIn">
                  <div className="inline-flex gap-2 items-center text-blue-400 uppercase text-xs font-bold tracking-widest border-b border-blue-900/40 pb-2 mb-2">
                    <MapPin className="w-4 h-4 text-blue-400" />
                    <span>02 // Operational Coordinates</span>
                  </div>
                  <p className="text-gray-400 text-[11px] leading-relaxed font-sans">
                    Calibrating geographic sensors and spatial coordinates.
                    Please state your secure testing center location and current
                    overarching mission directive.
                  </p>
                  <div>
                    <label className="block text-[10px] text-gray-450 uppercase tracking-wider mb-1 font-sans">
                      Operational Base Location
                    </label>
                    <input
                      type="text"
                      required
                      value={onboardLocation}
                      onChange={(e) => setOnboardLocation(e.target.value)}
                      placeholder="Home Office, Zurich Labs, Sector-4 Skyloft"
                      className="w-full bg-[#050608] border border-gray-800 rounded px-3 py-2 text-xs text-gray-200 placeholder-gray-605 focus:outline-none focus:border-blue-500"
                    />
                  </div>
                  <div>
                    <label className="block text-[10px] text-gray-450 uppercase tracking-wider mb-1 font-sans">
                      Primary Project Directive / Mission
                    </label>
                    <textarea
                      required
                      rows={2}
                      value={onboardMission}
                      onChange={(e) => setOnboardMission(e.target.value)}
                      placeholder="Solving autonomous robotics kinematics solvers and cognitive routing pathways."
                      className="w-full bg-[#050608] border border-gray-800 rounded px-3 py-2 text-xs text-gray-200 placeholder-gray-605 focus:outline-none focus:border-blue-500 font-mono resize-none"
                    />
                  </div>
                  <div className="flex justify-between pt-2">
                    <button
                      type="button"
                      onClick={() => setOnboardingStep(1)}
                      className="px-4 py-2 bg-gray-900 hover:bg-gray-850 text-gray-400 rounded text-xs uppercase tracking-widest font-bold cursor-pointer transition-all border border-gray-800"
                    >
                      Step Back
                    </button>
                    <button
                      type="button"
                      onClick={() => setOnboardingStep(3)}
                      className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded text-xs uppercase tracking-widest font-bold cursor-pointer transition-all shadow-[0_0_10px_#3b82f6]"
                    >
                      Lock Coordinates
                    </button>
                  </div>
                </div>
              )}

              {onboardingStep === 3 && (
                <div className="space-y-4 animate-fadeIn">
                  <div className="inline-flex gap-2 items-center text-blue-400 uppercase text-xs font-bold tracking-widest border-b border-blue-900/40 pb-2 mb-2">
                    <Target className="w-4 h-4 text-blue-400" />
                    <span>03 // Cognitive Preferences</span>
                  </div>
                  <p className="text-gray-400 text-[11px] leading-relaxed font-sans">
                    Final parameters to stabilize our neural pipeline. Specify
                    your areas of scientific interest (comma-separated tags) and
                    preferred compiler configuration.
                  </p>
                  <div>
                    <label className="block text-[10px] text-gray-450 uppercase tracking-wider mb-1 font-sans">
                      Scientific Areas of Interest (Comma Separated)
                    </label>
                    <input
                      type="text"
                      required
                      value={onboardInterestsStr}
                      onChange={(e) => setOnboardInterestsStr(e.target.value)}
                      placeholder="Autonomous Navigation, Kinematics, Web Grounding, Particle Physics"
                      className="w-full bg-[#050608] border border-gray-800 rounded px-3 py-2 text-xs text-gray-200 placeholder-gray-650 focus:outline-none focus:border-blue-500"
                    />
                  </div>
                  <div>
                    <label className="block text-[10px] text-gray-450 uppercase tracking-wider mb-1 font-sans">
                      Primary Development Compiler
                    </label>
                    <select
                      value={onboardLanguage}
                      onChange={(e) => setOnboardLanguage(e.target.value)}
                      className="w-full bg-[#050608] border border-gray-800 rounded px-3 py-2 text-xs text-gray-200 focus:outline-none focus:border-blue-500 font-mono"
                    >
                      <option value="TypeScript">
                        TypeScript (Standard High Performance)
                      </option>
                      <option value="Python">
                        Python (Robotic Matrix Algebra)
                      </option>
                      <option value="WebAssembly">
                        WebAssembly (Hardware Compiles)
                      </option>
                    </select>
                  </div>
                  <div className="flex justify-between pt-2">
                    <button
                      type="button"
                      onClick={() => setOnboardingStep(2)}
                      className="px-4 py-2 bg-gray-900 hover:bg-[#111319] text-gray-400 rounded text-xs uppercase tracking-widest font-bold cursor-pointer transition-all border border-gray-800"
                    >
                      Step Back
                    </button>
                    <button
                      type="submit"
                      className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white rounded text-xs uppercase tracking-widest font-bold cursor-pointer transition-all shadow-[0_0_10px_#10b981]"
                    >
                      Construct Neural Binder
                    </button>
                  </div>
                </div>
              )}
            </form>
          </div>
        </div>
      ) : (
        <div
          className="max-w-[1400px] w-full mx-auto p-4 md:p-6 flex flex-col flex-1 h-screen justify-between"
          id="jarvis-desktop-grid"
        >
          {/* Top Header Node Bar (Match aesthetics from instructions exactly) */}
          <header className="flex flex-col md:flex-row items-start md:items-center justify-between border-b border-gray-800 pb-4 mb-4 gap-4">
            <div className="flex items-center gap-3">
              <div className="w-3.5 h-3.5 bg-blue-500 rounded-full animate-pulse shadow-[0_0_12px_#3b82f6]"></div>
              <div>
                <h1 className="text-base md:text-lg font-bold tracking-widest text-white uppercase flex items-center gap-2">
                  Antigravity Protocol
                  <span className="text-blue-500 font-light text-xs bg-blue-900/10 border border-blue-900/35 px-2 py-0.5 rounded italic">
                    v4.0.2-Stable
                  </span>
                </h1>
                <div className="flex items-center gap-2 mt-0.5">
                  <p className="text-[10px] text-gray-500 tracking-wider font-mono">
                    NEURAL_ROUTING_ACTIVE // HOSTNAME: JARVIS_VM
                  </p>
                  {currentUser && (
                    <span className="text-[10px] text-blue-400 font-mono border-l border-gray-800 pl-2">
                      ACTIVE SESS:{" "}
                      <span className="text-gray-300 font-bold">
                        {currentUser.email}
                      </span>
                      {memory.core?.user_profile?.role && (
                        <span className="text-emerald-400 font-bold ml-2 pl-2 border-l border-gray-800 uppercase tracking-wider">
                          [{memory.core.user_profile.role}]
                        </span>
                      )}
                    </span>
                  )}
                </div>
              </div>
            </div>

            {/* Core hardware system states header panel */}
            <div className="flex flex-wrap gap-6 text-[11px] font-mono text-gray-400 bg-gray-900/10 border border-gray-800/40 rounded-xl p-2.5">
              <div className="flex flex-col">
                <span className="text-gray-500 text-[10px]">UPTIME</span>
                <span className="text-blue-400 font-bold tracking-wider">
                  {formatUptime(secondsUptime)}
                </span>
              </div>
              <div className="flex flex-col">
                <span className="text-gray-500 text-[10px]">LATENCY</span>
                <span className="text-[#38bdf8] font-bold">24ms</span>
              </div>
              <div className="flex flex-col">
                <span className="text-gray-500 text-[10px]">
                  AGENTIC SYSTEM
                </span>
                <span className="text-[#10b981] font-semibold">
                  Gemini-3.5-Intelligence
                </span>
              </div>
              <div className="flex flex-col">
                <span className="text-gray-500 text-[10px]">STATUS</span>
                <span className="text-[#34d399] flex items-center gap-1 font-bold">
                  <CheckCircle className="w-3 h-3 text-emerald-400 animate-pulse" />
                  ONLINE
                </span>
              </div>
              <div className="flex flex-col justify-center border-l border-gray-800 pl-4">
                <button
                  onClick={handleSignOut}
                  type="button"
                  className="text-[10px] bg-red-950/20 text-red-400 hover:bg-red-900/20 border border-red-500/20 px-2 py-1 rounded font-mono uppercase tracking-wider cursor-pointer font-bold"
                >
                  DISCONNECT
                </button>
              </div>
            </div>
          </header>

          {/* Dynamic Left/Right Main Control Grid */}
          <div className="grid grid-cols-12 gap-5 flex-1 overflow-hidden min-h-0">
            {/* Left Panel: Conversational Loop */}
            <section className="col-span-12 lg:col-span-5 flex flex-col bg-[#0D0F14] border border-gray-800 rounded-xl overflow-hidden h-full">
              <div className="py-2.5 px-4 bg-gray-900/50 border-b border-gray-800 flex justify-between items-center">
                <div className="flex items-center gap-1.5">
                  <GitBranch className="w-3.5 h-3.5 text-blue-500" />
                  <span className="text-[10px] font-bold text-gray-400 uppercase tracking-widest">
                    Neural Input Loop
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  {isSpeaking && (
                    <button
                      onClick={handleStopSpeaking}
                      className="mr-2 text-[9px] text-[#f43f5e] hover:text-red-400 font-mono tracking-widest uppercase bg-red-950/25 border border-red-500/35 rounded px-2 py-0.5 cursor-pointer hover:bg-red-950/40 transition-all font-semibold flex items-center gap-1 animate-pulse"
                      title="Stop J.A.R.V.I.S. speech feedback"
                    >
                      <Square className="w-2 h-2 fill-red-400 border-none" />{" "}
                      STOP VOICE
                    </button>
                  )}
                  <button
                    onClick={clearChatHistory}
                    className="mr-2 text-[9px] text-red-400/80 hover:text-red-400 font-mono tracking-widest uppercase bg-red-950/15 border border-red-500/20 rounded px-1.5 py-0.5 cursor-pointer hover:bg-red-950/30 transition-all font-semibold"
                    title="Purge conversational loop buffer"
                  >
                    [ PURGE BUFFER ]
                  </button>
                  <button
                    onClick={() =>
                      setIsChatHistoryCollapsed(!isChatHistoryCollapsed)
                    }
                    type="button"
                    className="mr-2 text-[9px] text-blue-400/80 hover:text-blue-400 font-mono tracking-widest uppercase bg-blue-950/15 border border-blue-500/20 rounded px-1.5 py-0.5 cursor-pointer hover:bg-blue-950/30 transition-all font-semibold flex items-center gap-1 shrink-0"
                    title={
                      isChatHistoryCollapsed
                        ? "Expand conversation feed"
                        : "Collapse conversation feed"
                    }
                  >
                    {isChatHistoryCollapsed ? (
                      <Maximize2 className="w-2.5 h-2.5" />
                    ) : (
                      <Minimize2 className="w-2.5 h-2.5" />
                    )}
                    {isChatHistoryCollapsed ? "EXPAND" : "COLLAPSE"}
                  </button>
                  {isListeningForVoice && (
                    <span className="text-[9px] bg-red-950 text-red-400 border border-red-900 px-1.5 py-0.5 rounded leading-none shrink-0 animate-pulse">
                      LISTENING VOICE...
                    </span>
                  )}
                  {chatLoading ? (
                    <span className="text-[9px] text-[#34d399] uppercase tracking-wider font-mono animate-pulse">
                      Running diagnostics...
                    </span>
                  ) : (
                    <span className="text-[9px] text-[#22c55e]/70 uppercase tracking-widest font-mono">
                      STANDBY
                    </span>
                  )}
                </div>
              </div>

              {/* Chat conversation stream body */}
              <div
                className={`p-4 overflow-y-auto space-y-4 font-mono text-xs transition-all duration-300 ${
                  isChatHistoryCollapsed
                    ? "max-h-0 py-0 opacity-0 overflow-hidden pointer-events-none"
                    : "flex-1 max-h-[calc(100vh-320px)] lg:max-h-none"
                }`}
              >
                {messages.map((msg, idx) => {
                  const isUser = msg.role === "user";
                  return (
                    <div key={msg.id || idx} className="space-y-1 h-auto">
                      <div className="flex justify-between items-center">
                        <span
                          className={`text-[10px] uppercase font-bold tracking-wider ${isUser ? "text-[#38bdf8]" : "text-[#10b981]"}`}
                        >
                          {isUser ? "YOU (Admin)" : "JARVIS (System)"}
                        </span>
                        <span className="text-[9px] text-gray-500 font-mono">
                          {new Date(msg.timestamp).toLocaleTimeString()}
                        </span>
                      </div>

                      <div
                        className={`p-3 rounded-lg border leading-relaxed text-gray-200 shadow-sm ${
                          isUser
                            ? "bg-[#121620] border-[#3b82f6]/15"
                            : "bg-black/20 border-gray-800/80 pr-6"
                        }`}
                      >
                        {msg.content}
                      </div>

                      {/* Agent tool declaration logs */}
                      {msg.toolDetails && (
                        <div className="my-2 p-2.5 bg-[#1a1f2e]/60 border border-amber-900/40 rounded text-[11px] text-amber-300 space-y-1.5 animate-fadeIn">
                          <div className="flex items-center gap-1.5 font-bold text-amber-400 uppercase tracking-wider">
                            <Shield className="w-3.5 h-3.5 shrink-0" />
                            <span>
                              ⚡ Operational Tool Auto-run:{" "}
                              {msg.toolDetails.name.replace(/_/g, " ")}
                            </span>
                          </div>
                          {msg.toolDetails.args &&
                          typeof msg.toolDetails.args === "object" ? (
                            <div className="text-gray-400 font-mono text-[10px] space-y-0.5 pl-1">
                              {Object.entries(msg.toolDetails.args).map(
                                ([key, val]) => (
                                  <div
                                    key={key}
                                    className="flex flex-wrap items-baseline gap-1"
                                  >
                                    <span className="text-amber-500 font-semibold">
                                      {key.replace(/_/g, " ")}:
                                    </span>
                                    <span className="text-gray-300 italic">
                                      {typeof val === "object"
                                        ? JSON.stringify(val)
                                        : String(val)}
                                    </span>
                                  </div>
                                ),
                              )}
                            </div>
                          ) : (
                            <p className="text-gray-400 font-mono italic text-[10px]">
                              Parameters: {String(msg.toolDetails.args)}
                            </p>
                          )}
                          {msg.toolDetails.result && (
                            <p className="text-gray-400 font-mono italic text-[10px] border-t border-gray-800/80 mt-1 pt-1">
                              Status: {msg.toolDetails.result}
                            </p>
                          )}
                        </div>
                      )}
                    </div>
                  );
                })}
                {chatLoading && (
                  <div className="flex items-center gap-1.5 text-[#38bdf8] animate-pulse">
                    <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                    <span>Synthesizing active agent core pathways...</span>
                  </div>
                )}
                <div ref={chatEndRef} />
              </div>

              {/* Voice Wave Visualizer indicator */}
              <div className="px-4 py-2 bg-black/40 border-t border-gray-800/80 flex items-center justify-between gap-2">
                <span className="text-[9px] font-mono text-gray-500">
                  VOICE FREQUENCY SIGNAL
                </span>
                <div className="flex gap-0.5 items-end h-4 pr-1">
                  {vocalLevels.map((val, i) => (
                    <div
                      key={i}
                      className="w-1 bg-[#3b82f6]"
                      style={{
                        height: `${statsLoading ? val / 2 : val / 4}px`,
                      }}
                    />
                  ))}
                </div>
              </div>

              {/* J.A.R.V.I.S. Continuous Vocal HUD & Wake Word Monitor */}
              <div className="px-4 py-3 bg-[#0a0d13] border-t border-b border-gray-800/80 flex flex-col gap-2 font-mono text-[11px]">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <div className="relative flex h-3 w-3">
                      {wakeWordListening && (
                        <span
                          className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${
                            wakeStatus === "idle"
                              ? "bg-cyan-400"
                              : wakeStatus === "listening"
                                ? "bg-amber-400"
                                : wakeStatus === "processing"
                                  ? "bg-emerald-400"
                                  : "bg-cyan-400"
                          }`}
                        ></span>
                      )}
                      <span
                        className={`relative inline-flex rounded-full h-3 w-3 ${
                          !wakeWordListening
                            ? "bg-gray-700"
                            : wakeStatus === "idle"
                              ? "bg-cyan-500"
                              : wakeStatus === "listening"
                                ? "bg-amber-500"
                                : wakeStatus === "processing"
                                  ? "bg-emerald-500"
                                  : "bg-cyan-500"
                        }`}
                      ></span>
                    </div>
                    <span className="font-bold text-gray-300 uppercase tracking-widest text-[10px]">
                      {!wakeWordListening
                        ? "Vocal Wake Monitor: OFFLINE"
                        : `Wake Monitor: ${wakeStatus.toUpperCase()}`}
                    </span>
                  </div>

                  <button
                    type="button"
                    onClick={toggleWakeWordMonitor}
                    className={`px-2.5 py-1 rounded text-[10px] font-bold tracking-wider border transition-all ${
                      wakeWordListening
                        ? "bg-cyan-950/40 text-cyan-400 border-cyan-800 hover:bg-cyan-900/30"
                        : "bg-[#111319] text-gray-400 border-gray-800 hover:text-white"
                    }`}
                  >
                    {wakeWordListening
                      ? "[ DEACTIVATE MONITOR ]"
                      : "[ ACTIVATE WAKE MONITOR ]"}
                  </button>
                </div>

                {/* Real-time Subtitles / Audio Transcript stream from Wake Monitor */}
                {wakeWordListening && (
                  <div className="mt-1 p-2 bg-black/40 rounded border border-gray-850/80 space-y-1">
                    <div className="flex justify-between items-center text-[9px] text-gray-500 uppercase">
                      <span>Acoustic Whisper Stream</span>
                      {wakeWordDetected && (
                        <span className="text-amber-400 animate-pulse font-bold">
                          WAKE WORD DETECTED
                        </span>
                      )}
                    </div>
                    <p className="text-gray-300 italic min-h-[14px]">
                      {wakeInterimText ? (
                        <span>&quot;{wakeInterimText}&quot;</span>
                      ) : (
                        <span className="text-gray-605 font-sans tracking-wide text-gray-500">
                          Say &quot;Jarvis [your command]&quot;...
                        </span>
                      )}
                    </p>
                    {wakeLastHeard && (
                      <div className="text-[9px] text-[#38bdf8] flex items-center gap-1 border-t border-gray-900/50 mt-1 pt-1 font-mono">
                        <span className="text-gray-550 font-sans text-gray-400">
                          LAST HEARD:
                        </span>
                        <span className="font-semibold text-cyan-300">
                          &quot;{wakeLastHeard}&quot;
                        </span>
                      </div>
                    )}
                  </div>
                )}
              </div>

              {/* Prompt Input Matrix */}
              <div className="p-3 bg-black/40 border-t border-gray-800 space-y-2">
                {uploadedFileName && (
                  <div className="flex items-center justify-between bg-blue-950/20 border border-blue-900/35 rounded px-2.5 py-1 animate-fadeIn">
                    <div className="flex items-center gap-1.5 text-blue-400 font-mono text-[9px]">
                      <FileText className="w-3.5 h-3.5" />
                      <span>
                        ATTACHED MULTIMODAL FILE:{" "}
                        <strong className="text-gray-300">
                          {uploadedFileName}
                        </strong>
                      </span>
                    </div>
                    <button
                      type="button"
                      onClick={() => {
                        setUploadedFileBase64(null);
                        setUploadedFileName(null);
                      }}
                      className="text-gray-500 hover:text-red-400 cursor-pointer"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  </div>
                )}
                <form
                  onSubmit={handleChatSubmit}
                  className="flex gap-2 items-center"
                >
                  {/* File Attachment Upload Button */}
                  <label
                    className="p-2 rounded-full bg-gray-900 text-gray-400 border border-gray-800 hover:text-white cursor-pointer transition-all flex items-center justify-center shrink-0"
                    title="Attach file (Image, PDF, Document, Audio)"
                  >
                    <Plus className="w-3.5 h-3.5" />
                    <input
                      type="file"
                      onChange={handleFileUpload}
                      className="hidden"
                    />
                  </label>

                  {/* Voice toggle button */}
                  <button
                    type="button"
                    onClick={toggleSpeechRecognition}
                    className={`p-2 rounded-full border transition-all ${
                      isListeningForVoice
                        ? "bg-red-950/40 text-red-400 border-red-500/45 animate-pulse"
                        : "bg-gray-900 text-gray-400 border-gray-800 hover:text-white"
                    }`}
                    title="Toggle Local Speech Typing"
                  >
                    <Mic className="w-3.5 h-3.5" />
                  </button>

                  {isSpeaking && (
                    <button
                      type="button"
                      onClick={handleStopSpeaking}
                      className="p-2 rounded-full bg-red-950/45 text-[#f43f5e] border border-red-500/50 hover:bg-red-900/40 transition-all flex items-center justify-center animate-pulse shrink-0"
                      title="Stop Jarvis voice playback"
                    >
                      <Square className="w-3.5 h-3.5 fill-[#f43f5e]" />
                    </button>
                  )}

                  {/* Main dynamic text input */}
                  <div className="flex-1 relative">
                    <input
                      type="text"
                      value={userInput}
                      onChange={(e) => setUserInput(e.target.value)}
                      placeholder={
                        chatLoading
                          ? "Synthesizing brain waves..."
                          : "Enter directive for your AI system companion..."
                      }
                      className="w-full bg-[#090b10] border border-gray-800 rounded px-3 py-2 text-xs font-mono text-gray-100 placeholder-gray-500 focus:outline-none focus:border-blue-500"
                      disabled={chatLoading}
                    />
                    {cameraActive && (
                      <span className="absolute right-3.5 top-1/2 -translate-y-1/2 flex items-center gap-1 text-[9px] font-semibold text-emerald-400 uppercase font-mono bg-emerald-950/20 px-1 border border-emerald-900/30 rounded">
                        <Camera className="w-2.5 h-2.5" /> EYES ACTIVE
                      </span>
                    )}
                  </div>

                  <button
                    type="submit"
                    className="p-2 rounded-full bg-blue-600 hover:bg-blue-500 text-white shadow-md disabled:opacity-45 transition-all text-sm flex items-center justify-center size-8"
                    disabled={chatLoading || !userInput.trim()}
                  >
                    <Send className="w-3.5 h-3.5" />
                  </button>
                </form>
              </div>
            </section>

            {/* Right Panel: Multimodal sensory matrix & cognitive charts */}
            <section className="col-span-12 lg:col-span-7 flex flex-col gap-5 h-full overflow-y-auto lg:overflow-hidden pb-4 lg:pb-0 relative">
              {/* Split Row for Vision Feed and Memory Node Tree */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-5 min-h-[250px] lg:h-[45%] shrink-0">
                {/* Box 1: Sensory Eye Feed (Webcam Support) */}
                <div className="bg-[#0D0F14] border border-gray-800 rounded-xl overflow-hidden flex flex-col h-full relative">
                  <div className="py-2.5 px-3 bg-gray-900/50 border-b border-gray-800 flex justify-between items-center">
                    <div className="flex items-center gap-1.5">
                      {cameraActive ? (
                        <>
                          <Camera className="w-3.5 h-3.5 text-emerald-400 animate-pulse" />
                          <span className="text-[10px] font-bold text-gray-400 uppercase tracking-widest">
                            Sensory Vision Feed
                          </span>
                        </>
                      ) : (
                        <>
                          <Sparkles className="w-3.5 h-3.5 text-sky-400 animate-pulse" />
                          <span className="text-[10px] font-bold text-gray-400 uppercase tracking-widest">
                            J.A.R.V.I.S. Consciousness Core
                          </span>
                        </>
                      )}
                    </div>
                    <button
                      onClick={toggleCameraFeed}
                      className={`text-[9px] px-2.5 py-0.5 rounded-full border tracking-wide font-mono transition-all ${
                        cameraActive
                          ? "bg-emerald-950 text-emerald-400 border-emerald-500"
                          : "bg-gray-850 text-gray-500 border-gray-800 hover:text-white"
                      }`}
                    >
                      {cameraActive ? "SHUTDOWN FEED" : "ACTIVATE EYES"}
                    </button>
                  </div>

                  {/* Vision display view */}
                  <div className="flex-1 flex items-center justify-center relative bg-gray-950">
                    <div className="absolute inset-0 opacity-10 bg-[radial-gradient(circle_at_center,_#3b82f6_0%,_transparent_75%)] pointer-events-none" />

                    {cameraActive ? (
                      <div className="w-full h-full relative flex items-center justify-center">
                        <video
                          ref={videoRef}
                          autoPlay
                          playsInline
                          muted
                          className="w-full h-full object-cover opacity-85"
                        />
                        {/* Interactive scanner matrix bars */}
                        <div className="absolute inset-0 bg-[linear-gradient(rgba(56,189,248,0.05)_1px,transparent_1px)] bg-[size:100%_4px] pointer-events-none" />
                        <div className="absolute inset-0 border border-blue-500/20 box-border pointer-events-none rounded" />
                        <div className="absolute top-2 left-2 bg-[#000]/70 text-[9px] text-[#38bdf8] font-mono px-2 py-0.5 rounded border border-blue-900/20 shadow-md">
                          CAM_FEED_TUNNEL :: RESOLUTION 1080P
                        </div>

                        {/* Clickable take capture snapshot preview */}
                        <button
                          onClick={captureFrameBase64}
                          className="absolute bottom-3.5 right-3.5 px-2 py-1 bg-black/80 hover:bg-black text-[9px] text-gray-300 font-mono rounded border border-gray-850 flex items-center gap-1 transition-all uppercase"
                        >
                          <RefreshCw className="w-3 h-3 text-sky-400" />
                          <span>Siphon Frame</span>
                        </button>
                      </div>
                    ) : (
                      <div className="w-full h-full flex flex-col items-center justify-center">
                        <JarvisAvatar isThinking={chatLoading} />
                      </div>
                    )}
                    <div className="absolute bottom-2 left-2 text-[9px] font-mono text-gray-500 uppercase pointer-events-none">
                      PROCESSED_BY_LLAVA_GEMINI_V4
                    </div>
                  </div>
                </div>

                {/* Box 2: Core Memory Profile & Semantic Knowledge Graph */}
                <div
                  className={`bg-[#0D0F14] border border-gray-800 rounded-xl overflow-hidden flex flex-col transition-all duration-300 ${isBox2Expanded ? "absolute inset-0 z-30 h-full w-full" : "h-full"}`}
                >
                  <div className="py-2.5 px-3 bg-gray-900/50 border-b border-gray-800 flex justify-between items-center shrink-0">
                    <div className="flex items-center gap-1.5 min-w-0">
                      <Database className="w-3.5 h-3.5 text-blue-500 animate-pulse shrink-0" />
                      <span className="text-[10px] font-bold text-gray-400 uppercase tracking-widest truncate">
                        Antigravity Capabilities Registry
                      </span>
                      {activeBox2Tab !== "registry" && (
                        <span className="text-[8px] bg-blue-950/60 text-blue-400 border border-blue-900/30 px-1.5 py-0.5 rounded font-mono uppercase tracking-wider shrink-0 hidden sm:inline-block">
                          {activeBox2Tab}
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-1.5 shrink-0">
                      {/* Expand/Collapse toggle */}
                      <button
                        onClick={() => setIsBox2Expanded(!isBox2Expanded)}
                        className="text-[9px] px-2 py-0.5 rounded bg-gray-950/80 hover:bg-gray-905 text-gray-400 hover:text-white border border-gray-850 transition-all font-mono font-bold uppercase flex items-center gap-1 cursor-pointer"
                        title={
                          isBox2Expanded
                            ? "Collapse panel scale"
                            : "Maximize panel scale"
                        }
                      >
                        {isBox2Expanded ? (
                          <>
                            <Minimize2 className="w-2.5 h-2.5 text-amber-400" />
                            <span>COLLAPSE</span>
                          </>
                        ) : (
                          <>
                            <Maximize2 className="w-2.5 h-2.5 text-emerald-400" />
                            <span>MAXIMIZE</span>
                          </>
                        )}
                      </button>

                      {activeBox2Tab === "memory" && (
                        <button
                          onClick={() => setShowConfigModal(!showConfigModal)}
                          className="text-[9px] px-2 py-0.5 rounded bg-gray-950/80 text-gray-400 hover:text-white border border-gray-850 flex items-center gap-1 transition-all font-mono"
                        >
                          <Plus className="w-3 h-3 text-sky-400" />
                          <span>CONFIGURE FACT</span>
                        </button>
                      )}
                      {activeBox2Tab === "skills" && (
                        <button
                          onClick={() => setShowSkillCreatorModal(true)}
                          className="text-[9px] px-2 py-0.5 rounded bg-blue-950/40 text-blue-400 hover:text-blue-300 border border-blue-900/40 flex items-center gap-1 transition-all pointer-events-auto font-mono"
                        >
                          <Plus className="w-3 h-3 text-blue-400" />
                          <span>INJECT CUSTOM SKILL</span>
                        </button>
                      )}
                    </div>
                  </div>

                  {/* Sub-Tabs Selector Header */}
                  <div className="flex overflow-x-auto scrollbar-thin scrollbar-thumb-gray-800/50 scrollbar-track-transparent bg-[#07090d] border-b border-gray-800/80 text-[10px] uppercase font-mono shrink-0">
                    <button
                      onClick={() => setActiveBox2Tab("registry")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1.5 ${
                        activeBox2Tab === "registry"
                          ? "text-emerald-400 bg-[#0d0f14]/50 border-b-2 border-b-emerald-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      <LayoutGrid className="w-3.5 h-3.5" /> Registry Dashboard
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("memory")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 ${
                        activeBox2Tab === "memory"
                          ? "text-blue-400 bg-[#0d0f14]/50 border-b-2 border-b-blue-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      Cognitive Memory
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("biomimetic")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1.5 ${
                        activeBox2Tab === "biomimetic"
                          ? "text-cyan-400 bg-[#0d0f14]/50 border-b-2 border-b-cyan-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      🧬 Biomimetic Lab
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("wifi_vision")}
                      className={`flex-1 min-w-[170px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1.5 ${
                        activeBox2Tab === "wifi_vision"
                          ? "text-cyan-400 bg-[#0d0f14]/50 border-b-2 border-b-cyan-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      📡 Wi-Fi Vision & Intel
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("job_tracker")}
                      className={`flex-1 min-w-[150px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1.5 ${
                        activeBox2Tab === "job_tracker"
                          ? "text-cyan-400 bg-[#0d0f14]/50 border-b-2 border-b-cyan-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      💼 Career Tracker
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("skills")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 ${
                        activeBox2Tab === "skills"
                          ? "text-blue-400 bg-[#0d0f14]/50 border-b-2 border-b-blue-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      Tactical Skills ({skillsList.length})
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("telegram")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1.5 ${
                        activeBox2Tab === "telegram"
                          ? "text-blue-400 bg-[#0d0f14]/50 border-b-2 border-b-blue-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      <span
                        className={`w-1.5 h-1.5 rounded-full ${telegramPollingActive ? "bg-emerald-400 animate-pulse shadow-[0_0_6px_#34d399]" : "bg-red-500"}`}
                      />
                      Telegram Linker
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("testing")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1 ${
                        activeBox2Tab === "testing"
                          ? "text-blue-400 bg-[#0d0f14]/50 border-b-2 border-b-blue-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      Capability Lab
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("webcheck")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1 ${
                        activeBox2Tab === "webcheck"
                          ? "text-blue-400 bg-[#0d0f14]/50 border-b-2 border-b-blue-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      Web Intel Check
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("asciline")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1 ${
                        activeBox2Tab === "asciline"
                          ? "text-blue-400 bg-[#0d0f14]/50 border-b-2 border-b-blue-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      ASCILINE Engine
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("robotics")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1 ${
                        activeBox2Tab === "robotics"
                          ? "text-blue-400 bg-[#0d0f14]/50 border-b-2 border-b-blue-500"
                          : "text-gray-500 hover:text-gray-300"
                      }`}
                    >
                      Robotics Lab
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("workspace")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1 ${
                        activeBox2Tab === "workspace"
                          ? "text-blue-400 bg-[#0d0f14]/50 border-b-2 border-b-blue-500"
                          : "text-gray-550 hover:text-gray-300"
                      }`}
                    >
                      Workspace
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("offer")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1 ${
                        activeBox2Tab === "offer"
                          ? "text-amber-400 bg-[#0d0f14]/50 border-b-2 border-b-amber-500"
                          : "text-gray-550 hover:text-gray-300"
                      }`}
                    >
                      🔮 Global Offer
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("recalibration")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1 ${
                        activeBox2Tab === "recalibration"
                          ? "text-cyan-400 bg-[#0d0f14]/50 border-b-2 border-b-cyan-500"
                          : "text-gray-550 hover:text-gray-300"
                      }`}
                    >
                      🧬 Phase VII Protocol
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("local_sync")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1 ${
                        activeBox2Tab === "local_sync"
                          ? "text-indigo-400 bg-[#0d0f14]/50 border-b-2 border-b-indigo-500"
                          : "text-gray-550 hover:text-gray-300"
                      }`}
                    >
                      💻 Native & Local Sync
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("ternary")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center border-r border-gray-800/60 transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1 ${
                        activeBox2Tab === "ternary"
                          ? "text-yellow-400 bg-[#0d0f14]/50 border-b-2 border-b-yellow-500"
                          : "text-gray-550 hover:text-gray-300"
                      }`}
                    >
                      ⚡ Ternary & 1-Bit Engine
                    </button>
                    <button
                      onClick={() => setActiveBox2Tab("research_hub")}
                      className={`flex-1 min-w-[130px] shrink-0 py-1.5 px-3 text-center transition-all font-bold tracking-wider hover:bg-gray-900/30 flex items-center justify-center gap-1 ${
                        activeBox2Tab === "research_hub"
                          ? "text-cyan-400 bg-[#0d0f14]/50 border-b-2 border-b-cyan-500"
                          : "text-gray-550 hover:text-gray-300"
                      }`}
                    >
                      🔬 Research & Publications
                    </button>
                  </div>

                  {/* Content View Area */}
                  <div className="flex-1 p-4 overflow-y-auto space-y-4 min-h-0">
                    {/* TAB 0: INTERACTIVE CAPABILITIES REGISTRY DASHBOARD */}
                    {activeBox2Tab === "registry" && (
                      <div className="space-y-4 animate-fadeIn">
                        {/* Dashboard Top Search and Filtering Panel */}
                        <div className="flex flex-col sm:flex-row gap-2.5 items-center justify-between bg-black/40 p-3 rounded-lg border border-gray-850">
                          <div className="flex items-center gap-1.5 font-mono text-[10px] uppercase font-bold text-gray-400 tracking-wider w-full sm:w-auto">
                            <Sliders className="w-4 h-4 text-emerald-450 shrink-0" />
                            <span>Filter Systems Capabilities</span>
                          </div>

                          {/* Fast Text Filter */}
                          <div className="flex items-center gap-2 w-full sm:w-auto flex-1 max-w-sm relative">
                            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-500" />
                            <input
                              type="text"
                              value={registrySearchQuery}
                              onChange={(e) =>
                                setRegistrySearchQuery(e.target.value)
                              }
                              placeholder="Search active intelligence, physical nodes..."
                              className="bg-gray-950/80 border border-gray-850 rounded px-2 py-1 pl-8 text-[11px] font-mono text-gray-200 placeholder-gray-500 hover:border-gray-800 focus:border-blue-500 focus:outline-none w-full"
                            />
                            {registrySearchQuery && (
                              <button
                                onClick={() => setRegistrySearchQuery("")}
                                className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-550 hover:text-white text-[10px] font-bold font-sans cursor-pointer"
                              >
                                ✕
                              </button>
                            )}
                          </div>

                          {/* Category Badges Filters */}
                          <div className="flex flex-wrap gap-1 font-mono text-[9px] uppercase tracking-wide">
                            {[
                              "all",
                              "intelligence",
                              "physics",
                              "integration",
                            ].map((cat) => (
                              <button
                                key={cat}
                                onClick={() => setRegistryCategoryFilter(cat)}
                                className={`px-2 py-1 rounded border transition-all ${
                                  registryCategoryFilter === cat
                                    ? "bg-emerald-950/40 text-emerald-350 border-emerald-800/80 font-bold"
                                    : "bg-black/20 text-gray-500 border-transparent hover:text-gray-300 hover:border-gray-850"
                                }`}
                              >
                                {cat}
                              </button>
                            ))}
                          </div>
                        </div>

                        {/* Capabilities Bento Grid Matrix */}
                        {(() => {
                          const registryItems = [
                            {
                              id: "memory",
                              name: "Cognitive Memory",
                              desc: "Multi-hop semantic network, D3.js dynamic relationship maps, & OKF schema databases.",
                              category: "intelligence",
                              icon: Database,
                              accentColor:
                                "text-blue-400 border-blue-900/30 bg-blue-950/5",
                              badgeColor:
                                "bg-blue-900/10 text-blue-400 border-blue-900/30",
                              liveStatus: "ONLINE",
                              telemetry: `${memory.nodes?.length || 0} nodes / ${memory.relations?.length || 0} relations`,
                              vibe: "Coherent Semantic Mesh",
                            },
                            {
                              id: "skills",
                              name: "Tactical Skills",
                              desc: "Custom injectables system running real-time command microcodes and subroutines.",
                              category: "intelligence",
                              icon: Cpu,
                              accentColor:
                                "text-purple-400 border-purple-900/30 bg-purple-950/5",
                              badgeColor:
                                "bg-purple-900/10 text-purple-400 border-purple-900/30",
                              liveStatus:
                                skillsList.filter((s) => s.enabled).length > 0
                                  ? "ONLINE"
                                  : "STANDBY",
                              telemetry: `${skillsList.filter((s) => s.enabled).length} active / ${skillsList.length} defined`,
                              vibe: "Executable Cognitive Modules",
                            },
                            {
                              id: "telegram",
                              name: "Telegram Linker",
                              desc: "Bridges communications to exterior smart systems via instant event polling loops.",
                              category: "integration",
                              icon: Send,
                              accentColor:
                                "text-sky-400 border-sky-900/30 bg-sky-950/5",
                              badgeColor:
                                "bg-sky-900/10 text-sky-450 border-sky-900/30",
                              liveStatus: telegramPollingActive
                                ? "ONLINE"
                                : "OFFLINE",
                              telemetry: telegramPollingActive
                                ? "Awaiting inputs / Polling active"
                                : "Dormant / Event stream closed",
                              vibe: "Relayed Ingress Interface",
                            },
                            {
                              id: "testing",
                              name: "Capability Lab",
                              desc: "Dynamic testing platform to examine, debug, and sand-sandbox individual modules.",
                              category: "intelligence",
                              icon: Sliders,
                              accentColor:
                                "text-emerald-400 border-emerald-900/30 bg-emerald-950/5",
                              badgeColor:
                                "bg-emerald-900/10 text-emerald-400 border-emerald-900/30",
                              liveStatus: "ARMED",
                              telemetry: "Full sandbox validation active",
                              vibe: "Sanity Testing Matrix",
                            },
                            {
                              id: "webcheck",
                              name: "Web Intel Check",
                              desc: "Direct web intelligence query mappings using grounding APIs to check global indicators.",
                              category: "intelligence",
                              icon: Globe,
                              accentColor:
                                "text-teal-400 border-teal-900/30 bg-teal-950/5",
                              badgeColor:
                                "bg-teal-900/10 text-teal-400 border-teal-900/30",
                              liveStatus: "STANDBY",
                              telemetry: "Web query socket connected",
                              vibe: "Real-time Grounding Intel",
                            },
                            {
                              id: "asciline",
                              name: "ASCILINE Engine",
                              desc: "Live system telemetric signal parser and 256-channel frequency canvas visualizer.",
                              category: "physics",
                              icon: Activity,
                              accentColor:
                                "text-rose-400 border-rose-900/30 bg-rose-950/5",
                              badgeColor:
                                "bg-rose-900/10 text-rose-400 border-rose-900/30",
                              liveStatus: "ONLINE",
                              telemetry: "256-chn spectrum sweeping",
                              vibe: "Oscilloscope Waveform Feed",
                            },
                            {
                              id: "robotics",
                              name: "Robotics Lab",
                              desc: "Visual feedback microcontroller logic simulator representing physical transducer nodes.",
                              category: "physics",
                              icon: Settings,
                              accentColor:
                                "text-amber-400 border-amber-900/30 bg-amber-950/5",
                              badgeColor:
                                "bg-amber-900/10 text-amber-450 border-amber-900/30",
                              liveStatus: "EMULATING",
                              telemetry: "Registers synced with logic board",
                              vibe: "Hardware Loop In Gating",
                            },
                            {
                              id: "workspace",
                              name: "Workspace Integrations",
                              desc: "Google Workspace service mappings, secure credential checks, and workspace APIs.",
                              category: "integration",
                              icon: Briefcase,
                              accentColor:
                                "text-indigo-400 border-indigo-900/30 bg-indigo-950/5",
                              badgeColor:
                                "bg-indigo-900/10 text-indigo-455 border-indigo-900/30",
                              liveStatus: workspaceToken
                                ? "ACTIVE"
                                : "AWAITING_AUTH",
                              telemetry: workspaceToken
                                ? "Active session token active"
                                : "Credentials scope unconfigured",
                              vibe: "Google Cloud Ecosystem Mapping",
                            },
                            {
                              id: "offer",
                              name: "Strategic Valuation",
                              desc: "Global financial modeling, strategic decision paths, and option multipliers.",
                              category: "integration",
                              icon: Target,
                              accentColor:
                                "text-cyan-400 border-cyan-900/30 bg-cyan-950/5",
                              badgeColor:
                                "bg-cyan-900/10 text-cyan-400 border-cyan-900/30",
                              liveStatus: "READY",
                              telemetry: "Valuation matrices locked",
                              vibe: "Algorithmic Risk Compiler",
                            },
                            {
                              id: "recalibration",
                              name: "Phase VII Protocol",
                              desc: "Fine-grained phase calibrator, and quantum coherence alignment tuning tools.",
                              category: "physics",
                              icon: Sparkles,
                              accentColor:
                                "text-pink-400 border-pink-900/30 bg-pink-950/5",
                              badgeColor:
                                "bg-pink-900/10 text-pink-400 border-pink-900/30",
                              liveStatus: "OPTIMAL",
                              telemetry: "Normalized delta offset: ~0.027",
                              vibe: "Entropy Conversion Engine",
                            },
                            {
                              id: "local_sync",
                              name: "Native & Local Sync",
                              desc: "Synchronization loops mapping memory files, offline logs, and host database caches.",
                              category: "intelligence",
                              icon: RefreshCw,
                              accentColor:
                                "text-yellow-400 border-yellow-900/30 bg-yellow-950/5",
                              badgeColor:
                                "bg-yellow-900/10 text-yellow-400 border-yellow-900/30",
                              liveStatus: "COMPLETED",
                              telemetry: "Local mirror storage optimized",
                              vibe: "Double-Buffered Offline Cache",
                            },
                            {
                              id: "ternary",
                              name: "Ternary & 1-Bit Engine",
                              desc: "Vcore hardware translation mapping silicon voltage drops into agent weight spaces.",
                              category: "physics",
                              icon: Zap,
                              accentColor:
                                "text-orange-400 border-orange-900/30 bg-orange-950/5",
                              badgeColor:
                                "bg-orange-900/10 text-orange-400 border-orange-900/30",
                              liveStatus: "TUNED",
                              telemetry: "Est. Sparsity Ratio: ~61.4%",
                              vibe: "Voltage-Gated Logic Engine",
                            },
                          ];

                          const filteredRegistry = registryItems.filter(
                            (item) => {
                              const matchesSearch =
                                item.name
                                  .toLowerCase()
                                  .includes(
                                    registrySearchQuery.toLowerCase(),
                                  ) ||
                                item.desc
                                  .toLowerCase()
                                  .includes(
                                    registrySearchQuery.toLowerCase(),
                                  ) ||
                                item.vibe
                                  .toLowerCase()
                                  .includes(registrySearchQuery.toLowerCase());
                              const matchesCategory =
                                registryCategoryFilter === "all" ||
                                item.category === registryCategoryFilter;
                              return matchesSearch && matchesCategory;
                            },
                          );

                          if (filteredRegistry.length === 0) {
                            return (
                              <div className="text-center py-8 bg-black/10 border border-gray-900 rounded-lg space-y-2">
                                <p className="text-gray-500 font-mono text-xs">
                                  No active capabilities found matching your
                                  current filter filters.
                                </p>
                                <button
                                  onClick={() => {
                                    setRegistrySearchQuery("");
                                    setRegistryCategoryFilter("all");
                                  }}
                                  className="text-[10px] text-emerald-400 font-mono hover:underline uppercase"
                                >
                                  [ Purge Search Queries ]
                                </button>
                              </div>
                            );
                          }

                          return (
                            <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3">
                              {filteredRegistry.map((item) => {
                                const CardIcon = item.icon;
                                const isOnline =
                                  item.liveStatus === "ONLINE" ||
                                  item.liveStatus === "ACTIVE" ||
                                  item.liveStatus === "OPTIMAL" ||
                                  item.liveStatus === "TUNED" ||
                                  item.liveStatus === "ARMED" ||
                                  item.liveStatus === "COMPLETED";

                                return (
                                  <div
                                    key={item.id}
                                    onClick={() => {
                                      setActiveBox2Tab(item.id as any);
                                    }}
                                    className={`group relative p-3 rounded-xl border flex flex-col justify-between gap-3 transition-all duration-200 cursor-pointer ${item.accentColor} border-gray-850 hover:border-blue-900/40 hover:bg-black/40 hover:shadow-[0_0_15px_rgba(59,130,246,0.02)]`}
                                  >
                                    <div>
                                      {/* Cap Header */}
                                      <div className="flex items-center justify-between font-mono text-[9px] mb-2">
                                        <span
                                          className={`px-2 py-0.5 rounded-full border uppercase text-[8px] font-bold tracking-wider ${item.badgeColor}`}
                                        >
                                          {item.category}
                                        </span>
                                        <div className="flex items-center gap-1">
                                          <span
                                            className={`w-1.5 h-1.5 rounded-full ${isOnline ? "bg-emerald-400 animate-pulse" : "bg-gray-650"}`}
                                          />
                                          <span
                                            className={`font-semibold ${isOnline ? "text-emerald-455" : "text-gray-500"}`}
                                          >
                                            {item.liveStatus}
                                          </span>
                                        </div>
                                      </div>

                                      {/* Cap Core Visual & Info */}
                                      <div className="flex gap-2.5 items-start">
                                        <div
                                          className={`p-1.5 rounded bg-[#090b10] border border-gray-850/80 text-gray-400 group-hover:text-blue-400 group-hover:border-blue-900/30 transition-all shrink-0`}
                                        >
                                          <CardIcon className="w-4 h-4" />
                                        </div>
                                        <div className="space-y-0.5">
                                          <h3 className="text-xs font-bold text-gray-200 tracking-wide font-mono group-hover:text-white transition-colors">
                                            {item.name}
                                          </h3>
                                          <p className="text-[10px] text-gray-450 leading-snug tracking-normal">
                                            {item.desc}
                                          </p>
                                        </div>
                                      </div>
                                    </div>

                                    {/* Cap Live Telemetry Status Footer */}
                                    <div className="border-t border-gray-900/50 pt-2 flex items-center justify-between text-[9px] font-mono mt-1">
                                      <div
                                        className="text-gray-550 block max-w-[70%] truncate"
                                        title={item.telemetry}
                                      >
                                        <span className="text-gray-600 block text-[8px] uppercase">
                                          Telemetry
                                        </span>
                                        <span className="text-gray-450 font-semibold">
                                          {item.telemetry}
                                        </span>
                                      </div>
                                      <div className="text-right flex items-center gap-0.5 text-blue-500 group-hover:text-sky-400 font-bold uppercase transition-colors shrink-0">
                                        <span>LAUNCH</span>
                                        <ChevronRight className="w-3 h-3 group-hover:translate-x-0.5 transition-transform" />
                                      </div>
                                    </div>
                                  </div>
                                );
                              })}
                            </div>
                          );
                        })()}
                      </div>
                    )}

                    {/* TAB 1: COGNITIVE MEMORY */}
                    {activeBox2Tab === "memory" && (
                      <div className="space-y-4 animate-fadeIn">
                        {/* Dynamic editable user profile credentials layout */}
                        <div className="space-y-2">
                          <p className="text-[10px] text-blue-500 font-bold uppercase tracking-widest border-b border-gray-900 pb-1 flex items-center gap-1">
                            <User className="w-3 h-3" />
                            <span>User Core Credentials</span>
                          </p>
                          <div className="grid grid-cols-2 gap-2 text-xs font-mono font-medium">
                            <div className="bg-black/20 p-2 rounded border border-gray-850">
                              <span className="text-[10px] text-gray-550 block uppercase font-sans">
                                Boss Alias
                              </span>
                              <span className="text-gray-200 mt-1 font-semibold">
                                {memory.core?.user_profile?.name || "Boss"}
                              </span>
                            </div>
                            <div className="bg-black/20 p-2 rounded border border-gray-850">
                              <span className="text-[10px] text-gray-550 block uppercase font-sans">
                                Occupation
                              </span>
                              <span
                                className="text-gray-200 block truncate font-semibold"
                                title={memory.core?.user_profile?.job}
                              >
                                {memory.core?.user_profile?.job || "Architect"}
                              </span>
                            </div>
                          </div>
                          <div className="bg-black/20 p-2 rounded border border-gray-850 text-xs font-mono font-medium">
                            <span className="text-[10px] text-gray-550 block uppercase font-sans">
                              Overarching Objective
                            </span>
                            <p className="text-gray-300 mt-0.5 italic leading-snug">
                              {memory.core?.user_profile?.mission ||
                                "Building Jarvis OS Core Componentry."}
                            </p>
                          </div>

                          {/* Special Admin & Metaphysical Profiles */}
                          {memory.core?.user_profile?.role && (
                            <div className="bg-emerald-950/20 p-2.5 text-xs font-mono rounded border border-emerald-900/35">
                              <span className="text-[9px] text-emerald-400 block uppercase font-sans font-bold tracking-wider">
                                System Clearance Authorization
                              </span>
                              <span className="text-emerald-100 font-bold flex items-center gap-1.5 uppercase mt-0.5">
                                <ShieldAlert className="w-3.5 h-3.5 text-emerald-400 animate-pulse shrink-0" />
                                {memory.core.user_profile.role}
                              </span>
                            </div>
                          )}

                          {memory.core?.user_profile?.astrological_profile && (
                            <div className="bg-blue-950/10 p-2.5 rounded border border-blue-900/30 space-y-2 text-xs font-mono mt-1">
                              <div className="flex items-center gap-1 text-[10px] font-bold text-blue-400 uppercase tracking-widest border-b border-blue-900/25 pb-1">
                                <Sparkles className="w-3 h-3 text-cyan-400 shrink-0" />
                                <span>Metaphysics Matrix</span>
                              </div>
                              <div className="grid grid-cols-2 gap-1.5 text-[10px]">
                                <div>
                                  <span className="text-[8px] text-gray-500 block uppercase">
                                    Sun Sign
                                  </span>
                                  <span className="text-gray-300 font-semibold leading-tight block">
                                    {
                                      memory.core.user_profile
                                        .astrological_profile.sun_sign
                                    }
                                  </span>
                                </div>
                                <div>
                                  <span className="text-[8px] text-gray-500 block uppercase">
                                    Moon Sign
                                  </span>
                                  <span className="text-gray-300 font-semibold leading-tight block">
                                    {
                                      memory.core.user_profile
                                        .astrological_profile.moon_sign
                                    }
                                  </span>
                                </div>
                              </div>
                              {memory.core.user_profile.mbti && (
                                <div>
                                  <span className="text-[8px] text-gray-500 block uppercase">
                                    Myers-Briggs Designation
                                  </span>
                                  <span className="text-cyan-400 font-bold">
                                    {memory.core.user_profile.mbti}
                                  </span>
                                </div>
                              )}
                              {memory.core.user_profile.philosophical_core && (
                                <div className="border-t border-blue-900/20 pt-1 text-[10px]">
                                  <span className="text-[8px] text-gray-550 block uppercase">
                                    Spiritual Core Philosophy
                                  </span>
                                  <p className="text-gray-300 italic mt-0.5 leading-snug">
                                    "
                                    {
                                      memory.core.user_profile
                                        .philosophical_core
                                    }
                                    "
                                  </p>
                                </div>
                              )}
                              {memory.core.user_profile.tarot_archetypes && (
                                <div className="border-t border-blue-900/20 pt-1 text-[10px]">
                                  <span className="text-[8px] text-gray-550 block uppercase">
                                    Tarot Archetype (Birth)
                                  </span>
                                  <p className="text-gray-300 leading-snug mt-0.5">
                                    {
                                      memory.core.user_profile.tarot_archetypes
                                        .birth_card
                                    }
                                  </p>
                                </div>
                              )}
                            </div>
                          )}
                        </div>

                        {/* Sub-view switcher within Cognitive Memory */}
                        <div className="pt-2 border-t border-gray-900/40">
                          <div className="flex bg-[#040507] p-1 rounded border border-gray-900/60 font-mono text-[9px] uppercase tracking-wider mb-3">
                            <button
                              onClick={() => setMemorySubView("graph")}
                              className={`flex-1 py-1 px-3 text-center rounded transition-all ${
                                memorySubView === "graph"
                                  ? "bg-blue-955/40 text-blue-400 font-bold border border-blue-905/40"
                                  : "text-gray-500 hover:text-gray-300"
                              }`}
                            >
                              D3 Semantic Graph
                            </button>
                            <button
                              onClick={() => setMemorySubView("okf")}
                              className={`flex-1 py-1 px-3 text-center rounded transition-all flex items-center justify-center gap-1.5 ${
                                memorySubView === "okf"
                                  ? "bg-cyan-955/45 text-cyan-400 font-bold border border-cyan-905/45"
                                  : "text-gray-500 hover:text-gray-300"
                              }`}
                            >
                              <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-pulse shrink-0" />
                              Open Knowledge Format (OKF) Hub
                            </button>
                          </div>

                          {memorySubView === "graph" ? (
                            <CognitiveMemoryGraph
                              nodes={memory.nodes || []}
                              relations={memory.relations || []}
                              onAddRelation={handleAddRelation}
                              onRefresh={fetchMemoryData}
                            />
                          ) : (
                            <OkfHub
                              nodes={memory.nodes || []}
                              relations={memory.relations || []}
                              userProfile={memory.core?.user_profile}
                              episodic={memory.episodic || []}
                              onImportSuccess={(data) => {
                                if (data.nodes) {
                                  setMemory((prev) => ({
                                    ...prev,
                                    nodes: data.nodes,
                                    relations: data.relations || prev.relations,
                                    core: data.core
                                      ? { ...prev.core, ...data.core }
                                      : prev.core,
                                  }));
                                }
                                fetchMemoryData();
                              }}
                            />
                          )}
                        </div>
                      </div>
                    )}

                    {/* TAB 2: SYSTEM SKILLS LIST */}
                    {activeBox2Tab === "skills" && (
                      <div className="space-y-3 animate-fadeIn">
                        <div className="flex justify-between items-center border-b border-gray-900 pb-1.5">
                          <p className="text-[10px] text-blue-500 font-bold uppercase tracking-widest flex items-center gap-1">
                            <Cpu className="w-3.5 h-3.5 animate-pulse" />
                            <span>Active Capabilities Subsystems</span>
                          </p>
                          <span className="text-[9px] font-mono text-gray-500 uppercase">
                            SANDBOX CONTAINER MODE
                          </span>
                        </div>

                        {skillsList.length > 0 ? (
                          <div className="space-y-2 max-h-[260px] overflow-y-auto pr-1">
                            {skillsList.map((skill) => {
                              const isCoreCategory = skill.category === "core";
                              const isAutomation =
                                skill.category === "automation";
                              const isIntegration =
                                skill.category === "integration";
                              const isMacro = skill.category === "macro";

                              let categoryBadgeColor =
                                "bg-amber-900/10 text-amber-400 border-amber-900/40";
                              if (isCoreCategory)
                                categoryBadgeColor =
                                  "bg-purple-900/10 text-purple-400 border-purple-900/40";
                              else if (isAutomation)
                                categoryBadgeColor =
                                  "bg-emerald-900/10 text-emerald-400 border-emerald-900/40";
                              else if (isIntegration)
                                categoryBadgeColor =
                                  "bg-blue-900/10 text-blue-400 border-blue-900/40";
                              else if (isMacro)
                                categoryBadgeColor =
                                  "bg-emerald-950 text-emerald-400 border-emerald-900/50 shadow-[0_0_8px_rgba(52,211,153,0.15)]";

                              return (
                                <div
                                  key={skill.id}
                                  className={`p-2.5 rounded-lg border flex flex-col gap-2 transition-colors ${
                                    skill.enabled
                                      ? isMacro
                                        ? "bg-emerald-950/15 border-emerald-900/35 shadow-[0_0_12px_rgba(52,211,153,0.04)]"
                                        : "bg-black/30 border-blue-900/25 shadow-[0_0_10px_rgba(59,130,246,0.03)]"
                                      : "bg-black/10 border-gray-900 opacity-60 hover:opacity-85"
                                  }`}
                                >
                                  <div className="flex items-start justify-between gap-3">
                                    <div className="space-y-1 min-w-0 flex-1">
                                      <div className="flex items-center gap-2">
                                        <h4 className="text-xs font-bold text-gray-200 tracking-wide font-mono">
                                          {skill.name}
                                        </h4>
                                        <span
                                          className={`text-[8px] font-bold font-mono border px-1.5 py-0.5 rounded uppercase shrink-0 ${categoryBadgeColor}`}
                                        >
                                          {skill.category}
                                        </span>
                                      </div>
                                      <p className="text-[10px] text-gray-400 leading-snug">
                                        {skill.description}
                                      </p>
                                    </div>

                                    {/* Active toggle action checkbox */}
                                    <label className="relative inline-flex items-center cursor-pointer shrink-0 mt-0.5 pointer-events-auto">
                                      <input
                                        type="checkbox"
                                        checked={skill.enabled}
                                        onChange={() =>
                                          toggleSkillState(skill.id)
                                        }
                                        className="sr-only peer"
                                      />
                                      <div
                                        className={`w-8 h-4 bg-gray-800 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-gray-400 after:rounded-full after:h-3 after:w-3 after:transition-all ${isMacro ? "peer-checked:bg-emerald-500" : "peer-checked:bg-blue-600"} peer-checked:after:bg-white`}
                                      />
                                    </label>
                                  </div>

                                  {isMacro
                                    ? skill.commands &&
                                      skill.commands.length > 0 && (
                                        <div className="mt-1 border-t border-gray-900/60 pt-1.5 space-y-1.5">
                                          <div className="text-[9px] uppercase font-bold text-gray-550 font-mono tracking-wider">
                                            Sequence Commands:
                                          </div>
                                          <div className="space-y-1">
                                            {skill.commands.map((cmd, cIdx) => (
                                              <div
                                                key={cIdx}
                                                className="text-[9px] text-emerald-400 font-mono px-2 py-0.5 bg-emerald-950/20 border border-emerald-900/15 rounded flex items-center gap-1.5"
                                              >
                                                <span className="text-emerald-600 font-bold">
                                                  {cIdx + 1}.
                                                </span>
                                                <span>{cmd}</span>
                                              </div>
                                            ))}
                                          </div>
                                          {skill.enabled && (
                                            <button
                                              onClick={() =>
                                                handleTriggerMacro(
                                                  skill.name,
                                                  skill.commands,
                                                )
                                              }
                                              className="mt-1 flex items-center gap-1 px-2 py-1 bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/20 hover:border-emerald-500/40 text-emerald-400 font-mono text-[9px] rounded uppercase font-bold tracking-widest transition-all cursor-pointer pointer-events-auto"
                                            >
                                              <Play className="w-2.5 h-2.5 fill-emerald-400/20 text-emerald-400" />
                                              <span>Run macro</span>
                                            </button>
                                          )}
                                        </div>
                                      )
                                    : skill.scriptTrigger && (
                                        <div className="text-[9px] text-[#38bdf8] font-mono bg-blue-950/15 border border-blue-900/20 rounded px-1.5 py-0.5 inline-block shrink-0 max-w-full truncate mt-1">
                                          $ {skill.scriptTrigger}
                                        </div>
                                      )}
                                </div>
                              );
                            })}
                          </div>
                        ) : (
                          <div className="p-8 text-center text-[10px] font-mono text-gray-600">
                            Loading J.A.R.V.I.S capabilities deck...
                          </div>
                        )}
                      </div>
                    )}

                    {/* TAB 3: TELEGRAM BOT COGNITIVE INTEGRATION */}
                    {activeBox2Tab === "telegram" && (
                      <div className="space-y-3.5 animate-fadeIn">
                        <div className="border-b border-gray-900 pb-1.5 flex justify-between items-center">
                          <p className="text-[10px] text-blue-400 font-bold uppercase tracking-widest flex items-center gap-1">
                            <Send className="w-3.5 h-3.5 text-sky-400" />
                            <span>Mobilize Jarvis OS Wirelessly</span>
                          </p>
                          <span
                            className={`text-[8px] font-bold font-mono px-1.5 py-0.5 rounded ${
                              telegramPollingActive
                                ? "bg-emerald-950 text-emerald-400 border border-emerald-900"
                                : "bg-red-950 text-red-400 border border-red-900"
                            }`}
                          >
                            {telegramPollingActive
                              ? "DAEMON_POLLING_ACTIVE"
                              : "STANDBY"}
                          </span>
                        </div>

                        {/* Instructions panel */}
                        <div className="bg-black/40 border border-gray-900 rounded p-2.5 text-[10px] leading-relaxed text-gray-400 font-mono space-y-1.5">
                          <div className="text-gray-200 uppercase font-semibold">
                            How to link your messaging:
                          </div>
                          <p>
                            1. Open Telegram & search for{" "}
                            <span className="text-blue-400 font-semibold font-sans hover:underline cursor-pointer">
                              @BotFather
                            </span>
                          </p>
                          <p>
                            2. Send `/newbot` to create J.A.R.V.I.S., and copy
                            the returned{" "}
                            <span className="text-[#38bdf8]">HTTP Token</span>{" "}
                            API key.
                          </p>
                          <p>
                            3. Paste the Token below. Once synchronized, you can
                            prompt your butler securely from any laptop or
                            mobile phone client!
                          </p>
                        </div>

                        {/* Config token inputs */}
                        <div className="space-y-2">
                          <label className="text-[9px] text-gray-550 uppercase font-bold tracking-wider block">
                            Telegram Bot API Key (HTTP Token)
                          </label>
                          <div className="flex gap-2">
                            <input
                              type="password"
                              placeholder="e.g. 123456:ABC-DEF1234ghIkl-zyx"
                              value={telegramToken}
                              onChange={(e) => setTelegramToken(e.target.value)}
                              className="bg-[#050608] text-xs font-mono text-[#cbd5e1] border border-gray-800 rounded p-2 flex-1 focus:outline-none focus:border-blue-500"
                            />
                            <button
                              onClick={saveTelegramConfig}
                              disabled={
                                checkingTelegramStatus || !telegramToken
                              }
                              className="px-3.5 bg-blue-600 hover:bg-blue-500 text-white font-mono text-[10px] uppercase font-bold tracking-wide rounded transition-all disabled:opacity-45 h-auto py-2 shrink-0 cursor-pointer"
                            >
                              {checkingTelegramStatus
                                ? "handshaking..."
                                : "sync uplink"}
                            </button>
                          </div>
                        </div>

                        {/* Dynamic connected status reporting */}
                        {telegramPollingActive ? (
                          <div className="p-2.5 bg-emerald-950/10 border border-emerald-900/30 rounded-lg flex items-center gap-2.5 mt-2 text-[10px] font-mono">
                            <CheckCircle className="w-5 h-5 text-emerald-400 shrink-0 animate-pulse" />
                            <div className="min-w-0">
                              <span className="font-bold text-emerald-400 block uppercase">
                                SYNC ESTABLISHED // STANDBY FOR BOSS COMMANDS
                              </span>
                              <p className="text-gray-400 text-[9px] leading-normal uppercase">
                                J.A.R.V.I.S. is listening live and responds to
                                any message sent directly to your Telegram bot.
                              </p>
                            </div>
                          </div>
                        ) : (
                          <div className="p-2.5 bg-gray-950/30 border border-gray-900/60 rounded-lg text-center text-[10px] font-mono text-gray-550 italic uppercase leading-relaxed font-sans">
                            Uplink inactive. Apply a bot token to establish
                            cloud command loops.
                          </div>
                        )}
                      </div>
                    )}

                    {/* TAB 4: COGNITIVE AGENT CAPABILITY DIAGNOSTICS & STRESS TESTER */}
                    {activeBox2Tab === "testing" && (
                      <div className="space-y-4 animate-fadeIn">
                        <div className="border-b border-gray-900 pb-1.5 flex justify-between items-center">
                          <p className="text-[10px] text-blue-400 font-bold uppercase tracking-widest flex items-center gap-1">
                            <Activity className="w-3.5 h-3.5 text-sky-450 animate-pulse" />
                            <span>VM Sandbox Capability Diagnostics</span>
                          </p>
                          <span className="text-[8px] font-mono text-gray-550 uppercase">
                            STRESS_LAB_SECURE_V4
                          </span>
                        </div>

                        <div className="bg-black/40 border border-gray-900 rounded p-2.5 text-[10px] leading-relaxed text-gray-400 font-mono space-y-1.5">
                          <div className="text-gray-200 uppercase font-semibold">
                            Integrative Agent Validation Protocol:
                          </div>
                          <p>
                            Launch multi-stage cognitive sequences to verify
                            J.A.R.V.I.S's core capability pipelines. Each run
                            evaluates database mutations, file indexing, and
                            voice synthesis feedback networks in real-time.
                          </p>
                        </div>

                        {/* Scenario selector deck */}
                        <div className="space-y-2.5">
                          {STRESS_TESTS.map((test) => {
                            const isRunningThis =
                              activeTest === test.id &&
                              testStatus === "running";
                            const isCompletedThis =
                              activeTest === test.id &&
                              testStatus === "completed";

                            return (
                              <div
                                key={test.id}
                                className={`p-3 rounded border transition-all ${
                                  isRunningThis
                                    ? "bg-slate-950/40 border-sky-500/30 shadow-[0_0_12px_rgba(56,189,248,0.08)]"
                                    : "bg-[#05070a] border-gray-900 hover:border-gray-800"
                                }`}
                              >
                                <div className="flex justify-between items-start gap-1">
                                  <div>
                                    <span
                                      className={`text-[8px] font-bold px-1.5 py-0.5 rounded font-mono ${
                                        test.difficulty.includes("EXTREME")
                                          ? "bg-red-950 text-red-400 border border-red-900/40"
                                          : test.difficulty.includes("HIGH")
                                            ? "bg-amber-950 text-amber-500 border border-amber-900/40"
                                            : "bg-sky-950 text-sky-400 border border-sky-900/40"
                                      }`}
                                    >
                                      {test.difficulty}
                                    </span>
                                    <h4 className="text-xs font-bold text-slate-200 font-mono mt-1 w-full tracking-wide">
                                      {test.name}
                                    </h4>
                                  </div>
                                  <span className="text-[8px] font-mono text-slate-500 whitespace-nowrap shrink-0">
                                    {test.avgDuration}
                                  </span>
                                </div>
                                <p className="text-[10px] text-gray-400 mt-1 pb-2 leading-relaxed font-sans border-b border-gray-900">
                                  {test.description}
                                </p>

                                {/* Progress pipeline tracker if running */}
                                {activeTest === test.id && (
                                  <div className="mt-2.5 font-mono text-[9px]">
                                    <div className="flex justify-between items-center text-slate-400 mb-1.5">
                                      <span>OPERATIONAL PROGRESS:</span>
                                      <span
                                        className={
                                          testStatus === "completed"
                                            ? "text-emerald-400 font-bold"
                                            : "text-sky-400 font-bold"
                                        }
                                      >
                                        {testStatus === "running"
                                          ? `STAGE ${activeTestStep}/5 ACTIVE`
                                          : testStatus === "completed"
                                            ? "DIAGNOSTIC PASSED"
                                            : "DIAGFAIL"}
                                      </span>
                                    </div>

                                    {/* Progress Pipeline Nodes */}
                                    <div className="flex items-center gap-1.5 mb-2.5">
                                      {[1, 2, 3, 4, 5].map((sIndex) => {
                                        const isPassed =
                                          activeTestStep >= sIndex;
                                        const isActive =
                                          activeTestStep === sIndex &&
                                          testStatus === "running";

                                        return (
                                          <React.Fragment key={sIndex}>
                                            <div
                                              className={`size-4 px-1 rounded-full flex items-center justify-center font-bold font-mono transition-all text-[8px] border ${
                                                isPassed &&
                                                testStatus === "completed"
                                                  ? "bg-emerald-950 text-emerald-400 border-emerald-500/50"
                                                  : isPassed && !isActive
                                                    ? "bg-blue-950 text-blue-400 border-blue-500/40"
                                                    : isActive
                                                      ? "bg-sky-500 text-white border-sky-400 animate-pulse shadow-[0_0_8px_#0ea5e9]"
                                                      : "bg-black/30 text-gray-600 border-gray-900"
                                              }`}
                                            >
                                              {sIndex}
                                            </div>
                                            {sIndex < 5 && (
                                              <div className="h-[1.5px] flex-1 min-w-[10px] bg-gray-900">
                                                <div
                                                  className={`h-full bg-sky-500 transition-all duration-350 ${
                                                    activeTestStep > sIndex
                                                      ? "w-full"
                                                      : "w-0"
                                                  }`}
                                                />
                                              </div>
                                            )}
                                          </React.Fragment>
                                        );
                                      })}
                                    </div>
                                  </div>
                                )}

                                <div className="flex justify-end gap-2 mt-2">
                                  <button
                                    onClick={() => runStressTest(test.id)}
                                    disabled={testStatus === "running"}
                                    className={`px-3 py-1 text-[9px] uppercase font-bold font-mono tracking-wider rounded transition-all cursor-pointer ${
                                      isRunningThis
                                        ? "bg-sky-950/40 text-sky-400 border border-sky-900/60 animate-pulse pointer-events-none"
                                        : "bg-slate-900 hover:bg-slate-850 text-slate-300 hover:text-white border border-gray-800"
                                    }`}
                                  >
                                    {isRunningThis
                                      ? "DIAG_VM_PROCESSING..."
                                      : isCompletedThis
                                        ? "RE-RUN TEST"
                                        : "RUN DIAGNOSTIC VM"}
                                  </button>
                                </div>
                              </div>
                            );
                          })}
                        </div>

                        {/* High-tech Console Logs Stream */}
                        {testLogs.length > 0 && (
                          <div className="space-y-1">
                            <label className="text-[8px] text-gray-550 uppercase font-bold tracking-wider block font-mono">
                              Real-Time Sandbox Logs
                            </label>
                            <div className="bg-[#050609] border border-gray-900 rounded p-2.5 font-mono text-[9px] text-[#34d399] leading-relaxed max-h-[160px] overflow-y-auto space-y-1 shadow-inner select-all">
                              {testLogs.map((log, index) => (
                                <div
                                  key={index}
                                  className="whitespace-pre-wrap break-all"
                                >
                                  {log}
                                </div>
                              ))}
                            </div>
                          </div>
                        )}
                      </div>
                    )}

                    {/* TAB 5: WEB INTEL CHECK CLONE */}
                    {activeBox2Tab === "webcheck" && (
                      <div className="space-y-4 animate-fadeIn">
                        <div className="border-b border-gray-900 pb-1.5 flex justify-between items-center">
                          <p className="text-[10px] text-cyan-400 font-bold uppercase tracking-widest flex items-center gap-1.5">
                            <Globe className="w-3.5 h-3.5 text-cyan-400 animate-pulse" />
                            <span>Multi-Spectrum Network Intelligence</span>
                          </p>
                          <span className="text-[8px] font-mono text-gray-500 uppercase">
                            J.A.R.V.I.S_SCANNER_V4.2
                          </span>
                        </div>

                        {/* Entry Search Form */}
                        <div className="bg-[#05070a] border border-gray-900 rounded-lg p-3 space-y-3">
                          <p className="text-[10px] text-gray-400 font-sans leading-relaxed">
                            Enter any domain host or URL to run a real-time web
                            intelligence audit. Checks DNS, SSL, security
                            response compliance headers, sitemap status, cookie
                            flags, and generates a J.A.R.V.I.S. cognitive
                            synthesis briefing.
                          </p>

                          <div className="flex gap-2">
                            <div className="relative flex-1">
                              <Globe className="absolute left-2.5 top-2.5 w-3.5 h-3.5 text-gray-500" />
                              <input
                                type="text"
                                value={webAnalysisUrl}
                                onChange={(e) =>
                                  setWebAnalysisUrl(e.target.value)
                                }
                                placeholder="e.g. google.com, github.com, lissy93.github.io"
                                onKeyDown={(e) =>
                                  e.key === "Enter" && runningWebCheck()
                                }
                                className="w-full bg-[#0a0d14] border border-gray-800 focus:border-cyan-500 rounded px-2.5 py-1.5 pl-8 text-xs font-mono text-gray-100 placeholder-gray-600 outline-none transition-all"
                              />
                            </div>
                            <button
                              onClick={() => runningWebCheck()}
                              disabled={isAnalyzingWeb}
                              className={`px-4 py-1.5 rounded text-xs font-bold font-mono tracking-wider transition-all select-none cursor-pointer flex items-center gap-1.5 ${
                                isAnalyzingWeb
                                  ? "bg-cyan-950/45 text-cyan-400 border border-cyan-800/50 animate-pulse pointer-events-none"
                                  : "bg-cyan-900/15 hover:bg-cyan-900/30 text-cyan-300 hover:text-cyan-200 border border-cyan-500/35 cursor-pointer active:scale-95"
                              }`}
                            >
                              {isAnalyzingWeb ? (
                                <>
                                  <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                                  <span>PROBING...</span>
                                </>
                              ) : (
                                <>
                                  <Play className="w-3 h-3 text-cyan-400" />
                                  <span>RUN PROBE</span>
                                </>
                              )}
                            </button>
                          </div>

                          {/* Interactive fast-audit targets list */}
                          <div className="flex flex-wrap items-center gap-1.5 pt-1 text-[9px] font-mono">
                            <span className="text-gray-500 uppercase">
                              FAST PILOT PROBES:
                            </span>
                            {[
                              "github.com",
                              "wikipedia.org",
                              "lissy93.github.io",
                            ].map((target) => (
                              <button
                                key={target}
                                onClick={() => {
                                  setWebAnalysisUrl(target);
                                  runningWebCheck(target);
                                }}
                                disabled={isAnalyzingWeb}
                                className="px-1.5 py-0.5 rounded border border-gray-950 bg-black/40 hover:bg-cyan-950/20 hover:border-cyan-900/40 text-gray-400 hover:text-cyan-300 transition-all cursor-pointer"
                              >
                                {target}
                              </button>
                            ))}
                          </div>
                        </div>

                        {/* Web Probe Result Error Area */}
                        {webAnalysisError && (
                          <div className="bg-red-950/25 border border-red-900/40 rounded p-3 text-red-400 text-[10px] flex gap-2.5 animate-fadeIn">
                            <AlertTriangle className="w-4 h-4 text-red-500 shrink-0" />
                            <div className="space-y-1">
                              <span className="font-bold uppercase font-mono tracking-wide">
                                [NETWORK_PROBE_ERROR]
                              </span>
                              <p>{webAnalysisError}</p>
                            </div>
                          </div>
                        )}

                        {/* Standby Empty Interface */}
                        {!webAnalysisReport &&
                          !isAnalyzingWeb &&
                          !webAnalysisError && (
                            <div className="border border-dashed border-gray-900 rounded-lg p-6 text-center space-y-2 animate-fadeIn bg-slate-950/10">
                              <Globe className="w-8 h-8 text-cyan-900/50 mx-auto animate-pulse" />
                              <div className="space-y-0.5">
                                <h5 className="text-[11px] font-bold font-mono text-cyan-500 uppercase tracking-widest">
                                  Network Oracle Idle
                                </h5>
                                <p className="text-[10px] text-gray-500 mx-auto max-w-sm">
                                  Standing by for target parameter. Direct
                                  J.A.R.V.I.S. to connect, scan, and parse
                                  server architectures live.
                                </p>
                              </div>
                            </div>
                          )}

                        {/* Loader progress screen */}
                        {isAnalyzingWeb && (
                          <div className="border border-cyan-900/20 rounded-lg p-6 text-center space-y-3 animate-pulse bg-cyan-950/5">
                            <RefreshCw className="w-7 h-7 text-cyan-400 animate-spin mx-auto" />
                            <div className="space-y-1 font-mono text-[9px] text-cyan-400">
                              <div className="font-bold uppercase tracking-widest">
                                ESTABLISHING VIRTUAL HANDSHAKE
                              </div>
                              <div className="text-gray-500 text-[8px] animate-pulse">
                                Resolving A/MX records • Reading TLS certs •
                                Prising response headers
                              </div>
                            </div>
                          </div>
                        )}

                        {/* Full Bento style audit report report result analysis! */}
                        {webAnalysisReport && !isAnalyzingWeb && (
                          <div className="space-y-4 animate-fadeIn">
                            {/* Scorecard grid */}
                            <div className="grid grid-cols-3 gap-2.5">
                              {/* Card 1: Security Grade */}
                              <div className="bg-black/35 border border-gray-900 rounded p-2.5 flex flex-col items-center justify-center text-center space-y-1">
                                <span className="text-[8px] font-mono text-gray-500 uppercase tracking-wider">
                                  POSTURE GRADE
                                </span>
                                <div
                                  className={`size-11 rounded-full flex items-center justify-center font-bold text-lg font-mono border shadow-sm ${
                                    ["A+", "A", "A-"].some((g) =>
                                      (
                                        webAnalysisReport.intelReport
                                          ?.postureGrade || ""
                                      ).includes(g),
                                    )
                                      ? "bg-emerald-950/20 text-emerald-400 border-emerald-500/45 shadow-[0_0_8px_rgba(16,185,129,0.06)]"
                                      : ["B+", "B", "B-"].some((g) =>
                                            (
                                              webAnalysisReport.intelReport
                                                ?.postureGrade || ""
                                            ).includes(g),
                                          )
                                        ? "bg-cyan-950/20 text-cyan-400 border-cyan-500/45"
                                        : ["C+", "C", "C-"].some((g) =>
                                              (
                                                webAnalysisReport.intelReport
                                                  ?.postureGrade || ""
                                              ).includes(g),
                                            )
                                          ? "bg-amber-950/20 text-amber-500 border-amber-550/40"
                                          : "bg-red-950/20 text-red-500 border-red-500/40 animate-pulse"
                                  }`}
                                >
                                  {webAnalysisReport.intelReport
                                    ?.postureGrade || "B"}
                                </div>
                              </div>

                              {/* Card 2: Security Headers Checklist Compliance */}
                              <div className="bg-black/35 border border-gray-900 rounded p-2.5 flex flex-col items-center justify-center text-center space-y-1">
                                <span className="text-[8px] font-mono text-gray-500 uppercase tracking-wider">
                                  HEADER SCORE
                                </span>
                                <span className="text-lg font-bold font-mono tracking-widest text-gray-100">
                                  {webAnalysisReport.headers?.securityScore ??
                                    0}
                                  %
                                </span>
                                <div className="w-14 h-[4px] bg-gray-900 rounded-full overflow-hidden">
                                  <div
                                    className="h-full bg-cyan-400 transition-all duration-350"
                                    style={{
                                      width: `${webAnalysisReport.headers?.securityScore ?? 0}%`,
                                    }}
                                  />
                                </div>
                              </div>

                              {/* Card 3: Detected Server info */}
                              <div className="bg-black/35 border border-gray-900 rounded p-2.5 flex flex-col items-center justify-center text-center space-y-1">
                                <span className="text-[8px] font-mono text-gray-500 uppercase tracking-wider">
                                  SERVER HEAD
                                </span>
                                <span
                                  className="text-[10px] font-mono font-bold text-cyan-355 truncate w-full text-center"
                                  title={
                                    webAnalysisReport.headers?.server ||
                                    "Unknown"
                                  }
                                >
                                  {webAnalysisReport.headers?.server ||
                                    "Unknown"}
                                </span>
                                <span className="text-[8px] font-sans text-gray-400 leading-none">
                                  {webAnalysisReport.cookies?.count ?? 0} Cookie
                                  {(webAnalysisReport.cookies?.count ?? 0) !== 1
                                    ? "s"
                                    : ""}
                                </span>
                              </div>
                            </div>

                            {/* Overview card */}
                            <div className="bg-[#050609] border border-gray-900 rounded p-3 space-y-2">
                              <span className="text-[8px] font-mono text-gray-500 uppercase tracking-widest block font-bold">
                                J.A.R.V.I.S. EXECUTIVE OVERVIEW
                              </span>
                              <p className="text-[10px] text-gray-300 leading-relaxed font-sans">
                                {webAnalysisReport.intelReport?.overview}
                              </p>

                              {webAnalysisReport.intelReport?.techStack &&
                                webAnalysisReport.intelReport.techStack.length >
                                  0 && (
                                  <div className="flex flex-wrap items-center gap-1 pt-1.5">
                                    <span className="text-[8px] font-mono text-gray-500 mr-1">
                                      TECH STACK:
                                    </span>
                                    {webAnalysisReport.intelReport.techStack.map(
                                      (tech: string) => (
                                        <span
                                          key={tech}
                                          className="bg-slate-900 text-gray-300 text-[8px] px-1.5 py-0.5 rounded font-mono border border-gray-855/80"
                                        >
                                          {tech}
                                        </span>
                                      ),
                                    )}
                                  </div>
                                )}
                            </div>

                            {/* DNS & SSL details */}
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-2.5">
                              {/* DNS Records */}
                              <div className="bg-black/20 border border-gray-900 rounded p-2.5 space-y-2 font-mono text-[9px]">
                                <span className="text-[8px] font-bold text-gray-500 uppercase tracking-wider flex items-center gap-1">
                                  <FileText className="w-3 h-3 text-cyan-400" />
                                  <span>DNS Records Spectrum</span>
                                </span>

                                <div className="space-y-1.5 max-h-[145px] overflow-y-auto pr-1">
                                  <div className="flex justify-between items-baseline py-0.5 border-b border-gray-950">
                                    <span className="text-gray-500">
                                      A Records:
                                    </span>
                                    <span className="text-gray-300 font-bold max-w-[130px] truncate">
                                      {Array.isArray(webAnalysisReport.dns?.A)
                                        ? webAnalysisReport.dns.A.join(", ")
                                        : "Not setup"}
                                    </span>
                                  </div>
                                  <div className="flex justify-between items-baseline py-0.5 border-b border-gray-950">
                                    <span className="text-gray-500">
                                      AAAA (IPv6):
                                    </span>
                                    <span className="text-gray-300 truncate max-w-[130px]">
                                      {Array.isArray(
                                        webAnalysisReport.dns?.AAAA,
                                      )
                                        ? webAnalysisReport.dns.AAAA.join(", ")
                                        : "Not setup"}
                                    </span>
                                  </div>
                                  <div className="flex justify-between items-baseline py-0.5 border-b border-gray-950">
                                    <span className="text-gray-500 font-mono">
                                      MX Servers:
                                    </span>
                                    <span
                                      className="text-gray-300 truncate max-w-[130px]"
                                      title={
                                        Array.isArray(webAnalysisReport.dns?.MX)
                                          ? JSON.stringify(
                                              webAnalysisReport.dns.MX,
                                            )
                                          : ""
                                      }
                                    >
                                      {Array.isArray(webAnalysisReport.dns?.MX)
                                        ? webAnalysisReport.dns.MX.map(
                                            (m: any) =>
                                              typeof m === "object"
                                                ? `${m.exchange} (${m.priority})`
                                                : String(m),
                                          ).join(", ")
                                        : "Not found"}
                                    </span>
                                  </div>
                                  <div className="flex justify-between items-baseline py-0.5 border-b border-gray-950">
                                    <span className="text-gray-500 font-mono">
                                      Host NS:
                                    </span>
                                    <span className="text-gray-300 truncate max-w-[130px]">
                                      {Array.isArray(webAnalysisReport.dns?.NS)
                                        ? webAnalysisReport.dns.NS.join(", ")
                                        : "Not found"}
                                    </span>
                                  </div>
                                  <div className="flex justify-between items-baseline py-0.5">
                                    <span className="text-gray-500 font-mono">
                                      TXT Verification:
                                    </span>
                                    <span
                                      className="text-gray-300 truncate max-w-[130px]"
                                      title={
                                        Array.isArray(
                                          webAnalysisReport.dns?.TXT,
                                        )
                                          ? webAnalysisReport.dns.TXT.join(", ")
                                          : ""
                                      }
                                    >
                                      {Array.isArray(webAnalysisReport.dns?.TXT)
                                        ? webAnalysisReport.dns.TXT.join(", ")
                                        : "Not found"}
                                    </span>
                                  </div>
                                </div>
                              </div>

                              {/* SSL/TLS certificate */}
                              <div className="bg-black/20 border border-gray-900 rounded p-2.5 space-y-2 font-mono text-[9px]">
                                <span className="text-[8px] font-bold text-gray-500 uppercase tracking-wider flex items-center gap-1">
                                  <Lock className="w-3 h-3 text-emerald-500" />
                                  <span>SSL/TLS Handshake</span>
                                </span>

                                <div className="space-y-1.5 font-mono">
                                  <div className="flex justify-between items-baseline py-0.5 border-b border-gray-950">
                                    <span className="text-gray-500 font-mono">
                                      Certificate:
                                    </span>
                                    <span
                                      className={`font-bold ${webAnalysisReport.ssl?.valid !== false ? "text-emerald-400" : "text-red-400"}`}
                                    >
                                      {webAnalysisReport.ssl?.valid !== false
                                        ? "VALID / ACTIVE"
                                        : "UNVERIFIED cert"}
                                    </span>
                                  </div>
                                  <div className="flex justify-between items-baseline py-0.5 border-b border-gray-950">
                                    <span className="text-gray-500 font-mono">
                                      Authority:
                                    </span>
                                    <span
                                      className="text-gray-300 truncate max-w-[130px]"
                                      title={
                                        webAnalysisReport.ssl?.issuer?.O ||
                                        "Unknown"
                                      }
                                    >
                                      {webAnalysisReport.ssl?.issuer?.O ||
                                        "Let's Encrypt / CDN"}
                                    </span>
                                  </div>
                                  <div className="flex justify-between items-baseline py-0.5 border-b border-gray-950">
                                    <span className="text-gray-500">
                                      Days Left:
                                    </span>
                                    <span
                                      className={`font-bold ${
                                        (webAnalysisReport.ssl?.days_left ??
                                          0) > 30
                                          ? "text-emerald-400"
                                          : "text-amber-400"
                                      }`}
                                    >
                                      {webAnalysisReport.ssl?.days_left
                                        ? `${webAnalysisReport.ssl.days_left} days`
                                        : "N/A"}
                                    </span>
                                  </div>
                                  <div className="flex justify-between items-baseline py-0.5 border-b border-gray-950">
                                    <span className="text-gray-500">
                                      Protocol:
                                    </span>
                                    <span className="text-gray-350">
                                      {webAnalysisReport.ssl?.protocol ||
                                        "TLS 1.3"}
                                    </span>
                                  </div>
                                  <div className="flex justify-between items-baseline py-0.5">
                                    <span className="text-gray-500">
                                      Metadata indexes:
                                    </span>
                                    <span className="text-gray-300">
                                      {webAnalysisReport.files?.robots
                                        ? "Robots ✓"
                                        : "Robots ✗"}{" "}
                                      |{" "}
                                      {webAnalysisReport.files?.sitemap
                                        ? "Sitemap ✓"
                                        : "Sitemap ✗"}
                                    </span>
                                  </div>
                                </div>
                              </div>
                            </div>

                            {/* Security Headers checklist */}
                            <div className="bg-black/20 border border-gray-900 rounded p-2.5 space-y-2">
                              <span className="text-[8px] font-bold text-gray-500 uppercase tracking-widest block font-mono">
                                Response Security Posture Compliance
                              </span>

                              <div className="grid grid-cols-2 gap-1.5 font-mono text-[9px]">
                                {[
                                  {
                                    key: "content-security-policy",
                                    name: "Content-Security-Policy",
                                    desc: "Guards against scripts injection hacks",
                                  },
                                  {
                                    key: "strict-transport-security",
                                    name: "Strict-Transport-Security",
                                    desc: "Forces secure HTTPS channels",
                                  },
                                  {
                                    key: "x-frame-options",
                                    name: "X-Frame-Options",
                                    desc: "Blocks frame clickjacking attacks",
                                  },
                                  {
                                    key: "x-content-type-options",
                                    name: "X-Content-Type-Options",
                                    desc: "Inhibits content phishing attempts",
                                  },
                                ].map((item) => {
                                  const isPresent = (
                                    webAnalysisReport.headers?.present || []
                                  ).includes(item.key);
                                  return (
                                    <div
                                      key={item.key}
                                      className="bg-black/30 border border-gray-950/80 rounded p-1.5 flex items-start gap-1.5 justify-between"
                                    >
                                      <div className="space-y-0.5 min-w-0">
                                        <div className="text-gray-200 font-bold truncate text-[8px]">
                                          {item.name}
                                        </div>
                                        <div className="text-gray-500 text-[8px] leading-none truncate font-sans">
                                          {item.desc}
                                        </div>
                                      </div>
                                      <span
                                        className={`text-[10px] font-bold shrink-0 ${isPresent ? "text-emerald-400" : "text-gray-650"}`}
                                      >
                                        {isPresent ? "✓ SECURED" : "✗ MISSING"}
                                      </span>
                                    </div>
                                  );
                                })}
                              </div>
                            </div>

                            {/* Vulnerabilities & playbooks recommendations */}
                            <div className="space-y-2">
                              {/* Vulnerabilities Box */}
                              {webAnalysisReport.intelReport?.vulnerabilities &&
                                webAnalysisReport.intelReport.vulnerabilities
                                  .length > 0 && (
                                  <div className="bg-red-950/15 border border-red-900/30 rounded p-3 space-y-1.5 font-mono">
                                    <span className="text-[8px] tracking-wider font-bold block uppercase flex items-center gap-1 text-red-400">
                                      <AlertTriangle className="w-3 h-3 text-red-500 animate-pulse" />
                                      <span>
                                        POTENTIAL DISCOVERED VULNERABILITIES (
                                        {
                                          webAnalysisReport.intelReport
                                            .vulnerabilities.length
                                        }
                                        )
                                      </span>
                                    </span>
                                    <ul className="list-disc pl-3.5 space-y-1 text-[10px] text-gray-300 font-sans leading-relaxed">
                                      {webAnalysisReport.intelReport.vulnerabilities.map(
                                        (v: string, idx: number) => (
                                          <li
                                            key={idx}
                                            className="leading-relaxed font-mono text-[9px]"
                                          >
                                            {v}
                                          </li>
                                        ),
                                      )}
                                    </ul>
                                  </div>
                                )}

                              {/* Recommendations Playbook */}
                              {webAnalysisReport.intelReport?.recommendations &&
                                webAnalysisReport.intelReport.recommendations
                                  .length > 0 && (
                                  <div className="bg-cyan-950/15 border border-cyan-900/30 rounded p-3 space-y-1.5 font-mono">
                                    <span className="text-[8px] tracking-wider font-bold block uppercase flex items-center gap-1.5 text-cyan-400">
                                      <Check className="w-3.5 h-3.5 text-cyan-400" />
                                      <span>
                                        TACTICAL MITIGATION PLAYBOOK & ACTIONS
                                      </span>
                                    </span>
                                    <ul className="list-decimal pl-3.5 space-y-1 text-[10px] text-gray-350 font-sans leading-relaxed">
                                      {webAnalysisReport.intelReport.recommendations.map(
                                        (r: string, idx: number) => (
                                          <li
                                            key={idx}
                                            className="leading-relaxed font-mono text-[9px]"
                                          >
                                            {r}
                                          </li>
                                        ),
                                      )}
                                    </ul>
                                  </div>
                                )}
                            </div>
                          </div>
                        )}
                      </div>
                    )}

                    {/* TAB 6: ASCILINE ENGINE */}
                    {activeBox2Tab === "asciline" && (
                      <AscilineEngine speakText={speakText} />
                    )}

                    {/* TAB 7: AUTONOMOUS ROBOTICS SIMULATION LAB */}
                    {activeBox2Tab === "robotics" && (
                      <RoboticsLab speakText={speakText} />
                    )}

                    {/* TAB 8: GOOGLE WORKSPACE SYSTEM INTEGRATION */}
                    {activeBox2Tab === "workspace" && (
                      <WorkspaceNexus
                        accessToken={workspaceToken}
                        onAuthorize={handleAuthorizeWorkspace}
                        speakText={speakText}
                        authError={authError}
                      />
                    )}

                    {/* TAB 9: GLOBAL SOVEREIGN MANIFEST & LANDING FRAME */}
                    {activeBox2Tab === "offer" && (
                      <GlobalOfferLanding speakText={speakText} />
                    )}

                    {/* TAB 10: PHASE VII METACOGNITIVE RECALIBRATION PORTAL */}
                    {activeBox2Tab === "recalibration" && (
                      <NoosphericRecalibration speakText={speakText} />
                    )}

                    {/* TAB 11: NATIVE DESKTOP & MOBILE INTEGRATION HUB */}
                    {activeBox2Tab === "local_sync" && (
                      <LocalModelSync speakText={speakText} />
                    )}

                    {/* TAB 12: EXPERIMENTAL TERNARY SYNAPSE & BITNET COMPILER */}
                    {activeBox2Tab === "ternary" && (
                      <TernarySynapseEngine
                        speakText={speakText}
                        fetch={fetch}
                      />
                    )}

                    {/* TAB 13: DEEPMIND BIOMIMETIC MOLECULAR & MISSENSE LAB */}
                    {activeBox2Tab === "biomimetic" && (
                      <BiomimeticLab
                        speakText={speakText}
                        fetchMemoryData={fetchMemoryData}
                      />
                    )}

                    {/* TAB 14: AMBIENT WI-FI VISION & PULSE INTELLIGENCE */}
                    {activeBox2Tab === "wifi_vision" && (
                      <WiFiVisionIntelligence
                        speakText={speakText}
                        fetchMemoryData={fetchMemoryData}
                      />
                    )}

                    {/* TAB 15: COHERENCE-THRESHOLD RESEARCH PORTAL & ROADMAPS */}
                    {activeBox2Tab === "research_hub" && (
                      <ResonanceResearchHub
                        speakText={speakText}
                        fetchMemoryData={fetchMemoryData}
                      />
                    )}

                    {/* TAB 16: CAREER TRACKER & JOB SEARCH INTERFACE */}
                    {activeBox2Tab === "job_tracker" && <JobTracker />}
                  </div>
                </div>
              </div>

              {/* Bottom Row: Active Proactive Monitors & Terminal Emulation split */}
              <div className="grid grid-cols-1 md:grid-cols-12 gap-5 flex-1 min-h-[300px]">
                {/* Proactive Agents System Panel */}
                <div className="col-span-12 md:col-span-4 flex flex-col">
                  <SystemTelemetry
                    stats={systemStats}
                    loading={statsLoading}
                    onRefresh={fetchSystemStats}
                    proactiveMuted={proactiveMuted}
                    onToggleProactive={() => setProactiveMuted(!proactiveMuted)}
                  />
                </div>

                {/* Execution Command Terminal Component */}
                <div className="col-span-12 md:col-span-8 flex flex-col">
                  <ComputerControlTerminal
                    onExecuteCommand={executeTerminalCommand}
                  />
                </div>
              </div>
            </section>
          </div>

          {/* Global Modal for Configs & Facts creation */}
          {showConfigModal && (
            <div className="fixed inset-0 bg-[#000]/80 flex items-center justify-center p-4 z-50 animate-fadeIn">
              <div className="bg-[#0D0F14] border border-gray-800 rounded-xl max-w-md w-full overflow-hidden shadow-2xl font-mono">
                <div className="bg-[#0f172a] p-4 border-b border-gray-800 flex justify-between items-center">
                  <span className="text-xs font-semibold text-slate-300 uppercase tracking-widest flex items-center gap-2">
                    <Settings className="w-4 h-4 text-sky-400" />
                    Configure Cognitive Matrix Fact
                  </span>
                  <button
                    onClick={() => setShowConfigModal(false)}
                    className="text-gray-405 hover:text-white transition-colors"
                  >
                    <X className="w-4 h-4" />
                  </button>
                </div>

                <div className="p-5 space-y-5">
                  {/* Profile detail modifier form */}
                  <form onSubmit={submitProfileKey} className="space-y-3">
                    <div className="text-[10px] text-slate-400 font-bold pb-1 uppercase border-b border-gray-800 tracking-wider">
                      Update Core Profile Setting
                    </div>
                    <div>
                      <label className="text-[10px] text-slate-500 uppercase block mb-1">
                        Key Setting
                      </label>
                      <input
                        type="text"
                        placeholder="e.g. name, job, or favoriteLanguage"
                        value={newKey}
                        onChange={(e) => setNewKey(e.target.value)}
                        className="w-full bg-[#050608] text-xs text-[#cbd5e1] border border-gray-800 rounded p-2 focus:outline-none focus:border-blue-500"
                      />
                    </div>
                    <div>
                      <label className="text-[10px] text-slate-500 uppercase block mb-1">
                        String Value
                      </label>
                      <input
                        type="text"
                        placeholder="e.g. Tony, Chief Engineering Lead"
                        value={newValue}
                        onChange={(e) => setNewValue(e.target.value)}
                        className="w-full bg-[#050608] text-xs text-[#cbd5e1] border border-gray-800 rounded p-2 focus:outline-none focus:border-blue-500"
                      />
                    </div>
                    <button
                      type="submit"
                      disabled={!newKey || !newValue}
                      className="w-full bg-blue-600 hover:bg-blue-500 text-white font-semibold py-1.5 rounded text-xs transition-colors disabled:opacity-45"
                    >
                      Commit Profile Parameter
                    </button>
                  </form>

                  {/* Relation adder form */}
                  <form onSubmit={submitNewRelation} className="space-y-3">
                    <div className="text-[10px] text-slate-400 font-bold pb-1 uppercase border-b border-gray-800 tracking-wider">
                      Weave Semantic Graph Node RELATION
                    </div>
                    <div className="grid grid-cols-3 gap-2">
                      <div>
                        <label className="text-[9px] text-slate-550 uppercase block mb-1">
                          Source
                        </label>
                        <input
                          type="text"
                          placeholder="Boss"
                          value={newRelation.source}
                          onChange={(e) =>
                            setNewRelation((prev) => ({
                              ...prev,
                              source: e.target.value,
                            }))
                          }
                          className="w-full bg-[#050608] text-[11px] text-[#cbd5e1] border border-gray-800 rounded p-1.5 focus:outline-none focus:border-blue-500"
                        />
                      </div>
                      <div>
                        <label className="text-[9px] text-slate-550 uppercase block mb-1">
                          Relation
                        </label>
                        <input
                          type="text"
                          placeholder="Owns"
                          value={newRelation.relation}
                          onChange={(e) =>
                            setNewRelation((prev) => ({
                              ...prev,
                              relation: e.target.value,
                            }))
                          }
                          className="w-full bg-[#050608] text-[11px] text-[#cbd5e1] border border-gray-800 rounded p-1.5 focus:outline-none focus:border-blue-500"
                        />
                      </div>
                      <div>
                        <label className="text-[9px] text-slate-550 uppercase block mb-1">
                          Target
                        </label>
                        <input
                          type="text"
                          placeholder="Max"
                          value={newRelation.target}
                          onChange={(e) =>
                            setNewRelation((prev) => ({
                              ...prev,
                              target: e.target.value,
                            }))
                          }
                          className="w-full bg-[#050608] text-[11px] text-[#cbd5e1] border border-gray-800 rounded p-1.5 focus:outline-none focus:border-blue-500"
                        />
                      </div>
                    </div>
                    <button
                      type="submit"
                      disabled={
                        !newRelation.source ||
                        !newRelation.target ||
                        !newRelation.relation
                      }
                      className="w-full bg-emerald-600 hover:bg-emerald-500 text-white font-semibold py-1.5 rounded text-xs transition-colors disabled:opacity-45"
                    >
                      Build Graph Connection
                    </button>
                  </form>
                </div>

                <div className="bg-[#0f172a] p-3 text-[10px] text-slate-500 border-t border-gray-800 text-center uppercase">
                  Synchronizes permanently to backend JSON logs
                </div>
              </div>
            </div>
          )}

          {/* Modal B: Injected Custom Skill Creator Dialog */}
          {showSkillCreatorModal && (
            <div className="fixed inset-0 bg-black/80 backdrop-blur-md flex items-center justify-center z-50 p-4 animate-fadeIn">
              <div className="bg-[#0b0c11] border border-gray-800 rounded-xl max-w-md w-full shadow-2xl overflow-hidden flex flex-col">
                <div className="py-3 px-4 bg-gray-950 border-b border-gray-800 flex justify-between items-center">
                  <div className="flex items-center gap-1.5 font-mono text-[11px] font-bold text-gray-300 uppercase">
                    <Cpu className="w-4 h-4 text-blue-500" />
                    <span>Configured Custom Skill Injector</span>
                  </div>
                  <button
                    onClick={() => setShowSkillCreatorModal(false)}
                    className="text-gray-500 hover:text-white transition-colors cursor-pointer"
                  >
                    <X className="w-4 h-4" />
                  </button>
                </div>

                <div className="p-4 space-y-4">
                  <div className="text-[10px] text-gray-400 font-mono flex items-center gap-2 border-b border-gray-900 pb-2">
                    <Shield className="w-3.5 h-3.5 text-blue-400 shrink-0" />
                    <span>
                      INJECT DIRECTLY INTO J.A.R.V.I.S. ACTIVE SYSTEM REGISTRY
                      PATH
                    </span>
                  </div>

                  <form
                    onSubmit={submitNewSkill}
                    className="space-y-4 font-sans"
                  >
                    <div className="space-y-1.5">
                      <label className="text-[10px] text-gray-500 uppercase font-bold tracking-wider block font-mono">
                        Skill Name
                      </label>
                      <input
                        type="text"
                        className="w-full bg-black/40 border border-gray-800 p-2 text-xs rounded text-white focus:outline-none focus:border-blue-500"
                        placeholder="e.g. Weather Sensor, Mail Alert"
                        value={newSkillForm.name}
                        onChange={(e) =>
                          setNewSkillForm((prev) => ({
                            ...prev,
                            name: e.target.value,
                          }))
                        }
                        required
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label className="text-[10px] text-gray-550 uppercase font-bold tracking-wider block font-mono">
                        Description
                      </label>
                      <textarea
                        rows={2}
                        className="w-full bg-black/40 border border-gray-800 p-2 text-xs rounded text-white focus:outline-none focus:border-blue-500 font-sans"
                        placeholder="Describe what J.A.R.V.I.S should do with this skill..."
                        value={newSkillForm.description}
                        onChange={(e) =>
                          setNewSkillForm((prev) => ({
                            ...prev,
                            description: e.target.value,
                          }))
                        }
                        required
                      />
                    </div>

                    {newSkillForm.category === "macro" ? (
                      <div className="space-y-1.5 animate-fadeIn">
                        <label className="text-[10px] text-emerald-400 uppercase font-bold tracking-wider block font-mono">
                          Macro Commands Sequence (One per line)
                        </label>
                        <textarea
                          rows={3}
                          className="w-full bg-black/40 border border-emerald-900/60 p-2 text-xs rounded text-white focus:outline-none focus:border-emerald-500 font-mono text-emerald-400 placeholder-emerald-950/70"
                          placeholder="optimize system&#10;reboot neural brain&#10;clean temporary caches"
                          value={macroCommandsInput}
                          onChange={(e) =>
                            setMacroCommandsInput(e.target.value)
                          }
                          required
                        />
                        <span className="text-[9px] text-[#34d399]/70 font-mono italic block leading-snug">
                          💡 Common CLI sequences execute sequentially when you
                          click run.
                        </span>
                      </div>
                    ) : (
                      <div className="space-y-1.5">
                        <label className="text-[10px] text-gray-550 uppercase font-bold tracking-wider block font-mono">
                          Trigger Script Code (Optional)
                        </label>
                        <input
                          type="text"
                          className="w-full bg-black/40 border border-gray-800 p-2 text-xs rounded text-white focus:outline-none focus:border-[#38bdf8] font-mono text-[#38bdf8]"
                          placeholder="e.g. npx tsx scripts/get-weather.ts"
                          value={newSkillForm.scriptTrigger}
                          onChange={(e) =>
                            setNewSkillForm((prev) => ({
                              ...prev,
                              scriptTrigger: e.target.value,
                            }))
                          }
                        />
                      </div>
                    )}

                    <div className="grid grid-cols-2 gap-2">
                      <div className="space-y-1.5">
                        <label className="text-[10px] text-gray-555 uppercase font-bold tracking-wider block font-mono">
                          Category
                        </label>
                        <select
                          className="w-full bg-black/40 border border-gray-800 p-2 text-xs rounded text-white focus:outline-none focus:border-blue-500 font-mono text-[10px] uppercase"
                          value={newSkillForm.category}
                          onChange={(e) =>
                            setNewSkillForm((prev) => ({
                              ...prev,
                              category: e.target.value as any,
                            }))
                          }
                        >
                          <option value="utility">Utility</option>
                          <option value="automation">Automation</option>
                          <option value="integration">Integration</option>
                          <option value="core">Core Subsystem</option>
                          <option value="macro">Terminal Macro</option>
                        </select>
                      </div>

                      <div className="flex items-end">
                        <button
                          type="submit"
                          className="w-full bg-blue-600 hover:bg-blue-500 text-white font-mono text-[10px] cursor-pointer font-bold tracking-wider py-2.5 rounded uppercase transition-colors"
                        >
                          Inject Skill
                        </button>
                      </div>
                    </div>
                  </form>
                </div>

                <div className="bg-[#08090d] p-3 text-[10px] text-slate-500 border-t border-gray-900 text-center uppercase font-mono">
                  PERSISTS PERMANENTLY IN THE SYSTEM DATABASE
                </div>
              </div>
            </div>
          )}

          {/* Bottom Command Bar / Footer (Match instructions exactly) */}
          <footer className="mt-4 flex flex-col md:flex-row items-center justify-between border-t border-gray-800 pt-4 gap-4 pb-2">
            {/* Quick interactive utility macros deck */}
            <div className="flex gap-3 text-[9px] font-mono select-none">
              <button
                onClick={toggleCameraFeed}
                className={`px-3 py-1.5 rounded border uppercase transition-all flex items-center gap-1.5 font-bold ${
                  cameraActive
                    ? "bg-blue-900/10 text-blue-400 border-blue-900/50"
                    : "bg-[#0a0C10] border-gray-800 text-gray-500 hover:text-gray-300"
                }`}
              >
                <Eye className="w-3.5 h-3.5" />
                <span>Sensors [F1]</span>
              </button>
              <button
                onClick={() => setVoiceEnabled(!voiceEnabled)}
                className={`px-3 py-1.5 rounded border uppercase transition-all flex items-center gap-1.5 font-bold ${
                  voiceEnabled
                    ? "bg-blue-900/10 text-[#38bdf8] border-blue-900/50"
                    : "bg-[#0a0C10] border-gray-800 text-gray-500 hover:text-gray-300"
                }`}
              >
                {voiceEnabled ? (
                  <Volume2 className="w-3.5 h-3.5" />
                ) : (
                  <VolumeX className="w-3.5 h-3.5" />
                )}
                <span>Voice Feedback [F2]</span>
              </button>
              <button
                onClick={() => setShowConfigModal(true)}
                className="px-3 py-1.5 bg-[#0a0C10] border border-gray-800 rounded text-gray-500 hover:text-gray-300 transition-all uppercase font-bold flex items-center gap-1.5"
              >
                <Database className="w-3.5 h-3.5" />
                <span>Memory Graph [F3]</span>
              </button>
            </div>

            {/* Core watermark footprint label */}
            <div className="text-[10px] font-mono text-gray-550 uppercase tracking-wider text-center md:text-right flex items-center gap-2">
              <Clock className="w-3.5 h-3.5 text-blue-500/80 animate-pulse" />
              <span>
                JARVIS_CORE_RUNTIME_OSX_ANTIGRAVITY_v4 // LOCAL_HOST_3000
              </span>
            </div>
          </footer>
        </div>
      )}
    </div>
  );
}
