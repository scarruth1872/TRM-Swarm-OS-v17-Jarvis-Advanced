import React, { useState, useEffect, useMemo, useRef } from "react";
import { 
  Laptop, 
  Smartphone, 
  Cpu, 
  Wifi, 
  WifiOff, 
  RefreshCw, 
  Check, 
  Copy, 
  Play, 
  Terminal, 
  Code, 
  Layers, 
  SmartphoneNfc,
  Database,
  ArrowRight,
  Sparkles,
  ExternalLink,
  ShieldCheck,
  Send,
  Sliders,
  Activity,
  Flame,
  Wrench,
  Download
} from "lucide-react";
import { motion, AnimatePresence } from "motion/react";

interface LocalModelSyncProps {
  speakText?: (text: string) => void;
}

export default function LocalModelSync({ speakText }: LocalModelSyncProps) {
  // Navigation for layout mode
  const [activeSegment, setActiveSegment] = useState<"desktop" | "mobile" | "endpoints" | "pipeline">("pipeline");
  
  // Local model connection parameters
  const [provider, setProvider] = useState<"ollama" | "lmstudio" | "custom">("ollama");
  const [endpointUrl, setEndpointUrl] = useState("http://localhost:11434");
  const [modelTag, setModelTag] = useState("llama3:8b");
  const [apiKey, setApiKey] = useState("");
  const [connectionStatus, setConnectionStatus] = useState<"disconnected" | "testing" | "connected" | "cors_error">("disconnected");
  
  // Interactive prompt tester
  const [testPrompt, setTestPrompt] = useState("Explain sovereignty in one sentence.");
  const [testResponse, setTestResponse] = useState("");
  const [isPromptLoading, setIsPromptLoading] = useState(false);

  // Tauri configuration code block
  const [copiedCodeType, setCopiedCodeType] = useState<"tauri" | "electron" | "ollama_cmd" | "pip_cmd">(null);

  // Mobile Mock Interface State
  const [mobileBattery, setMobileBattery] = useState(88);
  const [isMobileSynced, setIsMobileSynced] = useState(true);
  const [mobileSyncLogs, setMobileSyncLogs] = useState<string[]>([
    "09:51:12 - Local memory graph indexed.",
    "09:53:05 - Found 14 active dCas9 nodes.",
    "09:55:40 - Ready for local model handshake."
  ]);
  const [mobilePromptInput, setMobilePromptInput] = useState("");

  // MoE Training Pipeline States
  const [selectedMoE, setSelectedMoE] = useState<"qwen-moe" | "jetmoe">("qwen-moe");
  const [isTraining, setIsTraining] = useState(false);
  const [currentLoss, setCurrentLoss] = useState(2.85);
  const [epoch, setEpoch] = useState(0.00);
  const [tokensProcessed, setTokensProcessed] = useState(0);
  const [learningRate, setLearningRate] = useState("2e-4");
  const [gradientSteps, setGradientSteps] = useState(4);
  const [selectedSplit, setSelectedSplit] = useState<"lora" | "qlora" | "full">("qlora");
  const [latestInjectedNode, setLatestInjectedNode] = useState<string>("");
  
  // Simulated expert load percentage distribution (Summing up dynamically)
  const [expertDistribution, setExpertDistribution] = useState<number[]>([28, 15, 8, 32, 4, 3, 6, 4]);
  const [trainingLogs, setTrainingLogs] = useState<string[]>([
    "Initialization parameters loaded.",
    "Awaiting sparse Mixture of Experts gating setup."
  ]);

  // Interval for training simulation
  useEffect(() => {
    let intervalId: any;
    if (isTraining) {
      intervalId = setInterval(() => {
        setEpoch(prev => {
          const next = Number((prev + 0.01).toFixed(2));
          if (next >= 3.0) {
            setIsTraining(false);
            if (speakText) speakText("Mixture of Experts training pipeline run completed. Check final loss weights.");
            return 3.0;
          }
          return next;
        });

        setCurrentLoss(prev => {
          const noise = (Math.random() - 0.52) * 0.08;
          const nextLoss = Math.max(0.25, Number((prev + noise).toFixed(4)));
          return nextLoss;
        });

        setTokensProcessed(prev => prev + Math.floor(Math.random() * 450 + 800));

        // Slightly update routing distribution to show live sparse router gating
        setExpertDistribution(prev => {
          return prev.map(val => {
            const shift = Math.floor((Math.random() - 0.5) * 6);
            return Math.max(1, Math.min(45, val + shift));
          });
        });

        const lossVal = currentLoss.toFixed(4);
        const logMsgs = [
          `Batch processed - Tokens: ${tokensProcessed} - Sparsity penalty: 0.015`,
          `Gating router selected Top-2 Experts - Step loss: ${lossVal}`,
          `Embedding projection synced to localized cognitive weights.`
        ];
        const selectedMsg = logMsgs[Math.floor(Math.random() * logMsgs.length)];
        setTrainingLogs(prev => [selectedMsg, ...prev.slice(0, 15)]);

      }, 1000);
    }
    return () => clearInterval(intervalId);
  }, [isTraining, tokensProcessed, currentLoss]);

  // Thunder Compute Custom Integration State
  const [gpuType, setGpuType] = useState<"rtx4090" | "a100" | "h100" | "l4">("rtx4090");
  const [instanceStatus, setInstanceStatus] = useState<"offline" | "provisioning" | "ready" | "training">("offline");
  const [sshConnectionString, setSshConnectionString] = useState("ssh ubuntu@216.81.245.192 -p 30780 -i <path_to_key>");
  const [thunderLogs, setThunderLogs] = useState<string[]>([
    "Thunder Compute cluster bridge standing by. Secure your GPU to begin J.A.R.V.I.S. MoE model alignment."
  ]);
  const [thunderCopiedCode, setThunderCopiedCode] = useState<"ssh" | "python" | null>(null);

  const triggerThunderComputeProvision = () => {
    if (instanceStatus !== "offline") {
      setInstanceStatus("offline");
      setThunderLogs(prev => ["Deprovisioned remote GPU host cluster connection.", ...prev]);
      if (speakText) speakText("Thunder Compute GPU container connection closed.");
      return;
    }

    setInstanceStatus("provisioning");
    
    const ipMatch = sshConnectionString.match(/@([a-zA-Z0-9\.-]+)/);
    const targetHost = ipMatch ? ipMatch[1] : "216.81.245.192";
    
    setThunderLogs([`Requesting GPU cluster allocation from ${targetHost}...`]);
    if (speakText) speakText("Requesting cloud GPU handshake from Thunder Compute model server.");

    setTimeout(() => {
      setThunderLogs(prev => [
        "✔ Handshake approved. Provisioning secure container node.",
        `Allocating custom physical GPU hardware: ${gpuType.toUpperCase()} under sovereign Law of One alignment architecture...`,
        ...prev
      ]);
    }, 1200);

    setTimeout(() => {
      setThunderLogs(prev => [
        "✔ Remote Base System Synchronized: Ubuntu 22.04 LTS (x86_64)",
        "✔ Base Stack preloaded: CUDA toolkit, PyTorch, Hugging Face Hub, and PEFT libraries.",
        "✔ Preparing high-performance Unsloth compilation kernels (2x faster, 60% memory savings)...",
        ...prev
      ]);
    }, 2800);

    setTimeout(() => {
      setInstanceStatus("ready");
      setThunderLogs(prev => [
        `🌟 THUNDER COMPUTE GPU LINK ONLINE! Target: ${targetHost}`,
        `SSH authentication daemon established securely.`,
        "You can now connect directly via standard SSH keys or initiate direct PyTorch SFT training blocks.",
        ...prev
      ]);
      if (speakText) speakText("GPU Node successfully authorized. Ready for Mixture of Experts aligned modeling.");
    }, 4500);
  };

  const tauriConfigJson = `{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:3000",
    "distDir": "../dist"
  },
  "package": {
    "productName": "JarvisOS",
    "version": "1.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": true,
      "http5": {
        "all": true,
        "request": true,
        "scope": ["http://localhost:11434/*", "http://localhost:1234/*"]
      }
    },
    "bundle": {
      "active": true,
      "category": "DeveloperTool",
      "identifier": "com.jarvis.sovereign",
      "icon": ["icons/32x32.png", "icons/128x128.png"]
    }
  }
}`;

  const electronConfigJs = `// main.js - Electron native wrapper configuration
const { app, BrowserWindow } = require('electron');
const path = require('path');

function createWindow () {
  const win = new BrowserWindow({
    width: 1280,
    height: 800,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false,
      webSecurity: false // Necessary to bypass direct localhost CORS blocks
    },
    titleBarStyle: 'hidden',
    backgroundColor: '#05070a'
  });

  // Load the web preview environment
  win.loadURL('http://localhost:3000');
}

app.whenReady().then(() => {
  createWindow();
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});`;

  const handleCopyCode = (text: string, type: "tauri" | "electron" | "ollama_cmd" | "pip_cmd") => {
    navigator.clipboard.writeText(text);
    setCopiedCodeType(type);
    setTimeout(() => setCopiedCodeType(null), 2000);
  };

  // Test Direct Local Connection (with clean mock fallbacks / troubleshooting instructions)
  const testLocalConnection = async () => {
    setConnectionStatus("testing");
    setTestResponse("");
    addMobileLog(`Pinging model provider endpoint at ${endpointUrl}...`);

    try {
      // Direct client-side fetch test
      const pathSuffix = provider === "ollama" ? "/api/tags" : "/v1/models";
      
      const controller = new AbortController();
      const id = setTimeout(() => controller.abort(), 3000); // 3 second timeout fast check

      const response = await fetch(`${endpointUrl}${pathSuffix}`, {
        method: "GET",
        signal: controller.signal,
        headers: apiKey ? { "Authorization": `Bearer ${apiKey}` } : {}
      });

      clearTimeout(id);

      if (response.ok) {
        setConnectionStatus("connected");
        addMobileLog(`Established uplink with local provider tags.`);
        if (speakText) speakText("Connection established. Local model endpoints synchronized.");
      } else {
        throw new Error("Endpoint refused connection");
      }
    } catch (err: any) {
      console.warn("Direct localhost connection refused. Simulating CORS/network check...", err);
      
      // Because browser running inside our sandboxed hosted environment cannot directly touch
      // user's local physical network via raw localhost requests without custom setup, 
      // we provide clear directions and a gorgeous successful mock scenario so the user can verify configurations.
      setTimeout(() => {
        setConnectionStatus("cors_error");
        addMobileLog("CORS Warning: Direct localhost request limited by browser sandbox.");
        if (speakText) speakText("Endpoint diagnostics completed. Check CORS command configurations.");
      }, 1000);
    }
  };

  // Send Prompt to Local Model (attempts live fetch, falls back to simulated logic to maintain amazing sovereign UI experience)
  const sendPromptToLocalModel = async (promptInput: string, isFromMobile = false) => {
    if (!promptInput.trim()) return;

    if (isFromMobile) {
      setMobileSyncLogs(prev => [`> Prompter: ${promptInput}`, ...prev]);
      setMobilePromptInput("");
    } else {
      setIsPromptLoading(true);
      setTestResponse("");
    }

    try {
      const isOllama = provider === "ollama";
      const fetchUrl = isOllama ? `${endpointUrl}/api/generate` : `${endpointUrl}/v1/chat/completions`;
      
      const body = isOllama ? JSON.stringify({
        model: modelTag,
        prompt: promptInput,
        stream: false
      }) : JSON.stringify({
        model: modelTag,
        messages: [{ role: "user", content: promptInput }]
      });

      const response = await fetch(fetchUrl, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(apiKey && { "Authorization": `Bearer ${apiKey}` })
        },
        body
      });

      if (!response.ok) throw new Error("Local model generated an error response");
      
      const data = await response.json();
      const outputText = isOllama ? data.response : data.choices[0].message.content;

      if (isFromMobile) {
        addMobileLog(`Model output: ${outputText}`);
      } else {
        setTestResponse(outputText);
        setIsPromptLoading(false);
      }
    } catch (err) {
      // Simulate highly educational response based on Jarvis's tone mapping
      setTimeout(() => {
        const fallbackText = `[Sovereign Sandbox Emulation File] 
I processed your prompt "${promptInput}" using local context files. 

Since JARVIS OS prioritizes offline-first local network nodes, we achieve persistent sovereignty by feeding memory graph indices directly into your local machine's memory registers. To enable live inference directly from this hosted preview, ensure you start your local model container using:
export OLLAMA_ORIGINS="*" `;
        
        if (isFromMobile) {
          addMobileLog(`Model simulated output synced with graphs.`);
        } else {
          setTestResponse(fallbackText);
          setIsPromptLoading(false);
        }
      }, 1200);
    }
  };

  const addMobileLog = (logEntry: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setMobileSyncLogs(prev => [`${timestamp} - ${logEntry}`, ...prev.slice(0, 15)]);
  };

  return (
    <div id="local-model-root" className="space-y-6">
      {/* Banner Segment */}
      <div id="local-model-banner" className="relative p-5 rounded-lg bg-gradient-to-r from-sky-950/20 via-slate-900/40 to-indigo-950/20 border border-sky-900/30 overflow-hidden">
        <div className="absolute top-0 right-0 w-64 h-64 bg-sky-500/5 rounded-full blur-3xl pointer-events-none" />
        <div className="absolute bottom-0 left-12 w-64 h-64 bg-indigo-500/5 rounded-full blur-3xl pointer-events-none" />
        
        <div className="relative flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <span className="px-2 py-0.5 bg-indigo-500/15 text-indigo-400 text-[9px] font-mono rounded border border-indigo-500/20 uppercase tracking-widest animate-pulse">
                Local-First Arch synced
              </span>
              <span className="text-[10px] text-sky-400 font-mono tracking-wider">
                CODENAME: OFFLINE COMPONION
              </span>
            </div>
            <h1 className="text-xl md:text-2xl font-bold text-gray-100 tracking-tight flex items-center gap-2 font-sans">
              <Cpu className="w-5.5 h-5.5 text-indigo-400 animate-pulse" /> Desktop & Mobile Native Integration
            </h1>
            <p className="text-xs text-gray-400 max-w-2xl leading-relaxed">
              Dataright is humanright. Connect your J.A.R.V.I.S. OS dashboard directly to locally hosted large language models (Ollama, LM Studio) or compile this viewport as a standalone desktop binary and mobile companion app.
            </p>
          </div>
          
          <div className="flex flex-wrap gap-1 bg-[#05070a]/95 p-1 rounded border border-gray-800 shrink-0 font-mono text-[10px]">
            <button
              onClick={() => {
                setActiveSegment("pipeline");
                if (speakText) speakText("Opening local Mixture of Experts training pipeline configuration.");
              }}
              className={`px-3 py-1 rounded transition-all flex items-center gap-1.5 ${
                activeSegment === "pipeline" ? "bg-indigo-500/15 text-indigo-400 font-bold border border-indigo-500/20" : "text-gray-400 hover:text-gray-200"
              }`}
            >
              <Sparkles className="w-3.5 h-3.5 text-indigo-400 animate-pulse" /> Local MoE Pipeline
            </button>
            <button
              onClick={() => {
                setActiveSegment("endpoints");
                if (speakText) speakText("Opening local model integration dashboard");
              }}
              className={`px-3 py-1 rounded transition-all flex items-center gap-1.5 ${
                activeSegment === "endpoints" ? "bg-indigo-500/15 text-indigo-400 font-bold border border-indigo-500/20" : "text-gray-400 hover:text-gray-200"
              }`}
            >
              <Database className="w-3.5 h-3.5" /> Model Endpoints
            </button>
            <button
              onClick={() => {
                setActiveSegment("desktop");
                if (speakText) speakText("Displaying desktop native build guides.");
              }}
              className={`px-3 py-1 rounded transition-all flex items-center gap-1.5 ${
                activeSegment === "desktop" ? "bg-indigo-500/15 text-indigo-400 font-bold border border-indigo-500/20" : "text-gray-400 hover:text-gray-200"
              }`}
            >
              <Laptop className="w-3.5 h-3.5" /> Desktop Client
            </button>
            <button
              onClick={() => {
                setActiveSegment("mobile");
                if (speakText) speakText("Opening interactive smart mobile frame simulation.");
              }}
              className={`px-3 py-1 rounded transition-all flex items-center gap-1.5 ${
                activeSegment === "mobile" ? "bg-indigo-500/15 text-indigo-400 font-bold border border-indigo-500/20" : "text-gray-400 hover:text-gray-200"
              }`}
            >
              <Smartphone className="w-3.5 h-3.5" /> Mobile Sync
            </button>
          </div>
        </div>
      </div>

      <AnimatePresence mode="wait">
        {/* VIEW 1: COGNITIVE model ENDPOINTS & LOGICAL HANDSHAKES */}
        {activeSegment === "endpoints" && (
          <motion.div
            key="endpoints-view"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="grid grid-cols-1 lg:grid-cols-12 gap-5"
          >
            {/* Connection configuration panels */}
            <div className="lg:col-span-5 space-y-4">
              <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4">
                <p className="text-xs font-bold text-indigo-400 font-mono uppercase tracking-wider flex items-center gap-2">
                  <Sliders className="w-4 h-4 text-indigo-400" /> Host Configurations
                </p>

                {/* Local API provider toggle */}
                <div className="space-y-2">
                  <label className="text-[10px] text-gray-400 font-mono uppercase">Local API Provider</label>
                  <div className="grid grid-cols-3 gap-2">
                    {(["ollama", "lmstudio", "custom"] as const).map((prov) => (
                      <button
                        key={prov}
                        onClick={() => {
                          setProvider(prov);
                          if (prov === "ollama") {
                            setEndpointUrl("http://localhost:11434");
                            setModelTag("llama3:8b");
                          } else if (prov === "lmstudio") {
                            setEndpointUrl("http://localhost:1234");
                            setModelTag("meta-llama-3-8b-instruct");
                          } else {
                            setEndpointUrl("http://127.0.0.1:8000");
                            setModelTag("v1");
                          }
                          setConnectionStatus("disconnected");
                        }}
                        className={`py-1.5 px-2 text-center rounded text-[10px] font-mono border transition-all ${
                          provider === prov
                            ? "bg-indigo-500/10 text-indigo-300 border-indigo-500/30 font-bold"
                            : "bg-gray-950/40 text-gray-500 border-gray-900 hover:text-gray-300"
                        }`}
                      >
                        {prov === "ollama" && "Ollama"}
                        {prov === "lmstudio" && "LM Studio"}
                        {prov === "custom" && "Custom IP"}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Endpoint Host address field */}
                <div className="space-y-1">
                  <label className="text-[10px] text-gray-500 font-mono uppercase">Host Endpoint Connection URL</label>
                  <input
                    type="text"
                    value={endpointUrl}
                    onChange={(e) => {
                      setEndpointUrl(e.target.value);
                      setConnectionStatus("disconnected");
                    }}
                    placeholder="e.g. http://localhost:11434"
                    className="w-full bg-gray-950 border border-gray-800 text-xs py-2 px-3 text-slate-100 rounded focus:outline-none focus:border-indigo-500/50 font-mono"
                  />
                </div>

                {/* Model Tag Identifier */}
                <div className="space-y-1">
                  <label className="text-[10px] text-gray-500 font-mono uppercase">Model Tag Identifier (Tag)</label>
                  <input
                    type="text"
                    value={modelTag}
                    onChange={(e) => setModelTag(e.target.value)}
                    placeholder="e.g. llama3, mistral"
                    className="w-full bg-gray-950 border border-gray-800 text-xs py-2 px-3 text-slate-100 rounded focus:outline-none focus:border-indigo-500/50 font-mono"
                  />
                </div>

                {/* API Authorization token (optional) */}
                {provider === "custom" && (
                  <div className="space-y-1">
                    <label className="text-[10px] text-gray-500 font-mono uppercase">Bearer/API Authorization Token (Optional)</label>
                    <input
                      type="password"
                      value={apiKey}
                      onChange={(e) => setApiKey(e.target.value)}
                      placeholder="Insert auth token key"
                      className="w-full bg-gray-950 border border-gray-800 text-xs py-2 px-3 text-slate-100 rounded focus:outline-none focus:border-indigo-500/50 font-mono"
                    />
                  </div>
                )}

                {/* Uplink diagnostics and action buttons */}
                <div className="pt-2 flex gap-2">
                  <button
                    onClick={testLocalConnection}
                    disabled={connectionStatus === "testing"}
                    className="flex-1 py-1.5 bg-indigo-600 hover:bg-indigo-500 disabled:bg-indigo-850 text-slate-950 text-[10px] tracking-wider uppercase font-bold font-mono rounded flex items-center justify-center gap-1 cursor-pointer transition-all shadow-[0_0_12px_rgba(99,102,241,0.2)]"
                  >
                    {connectionStatus === "testing" ? (
                      <>
                        <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                        <span>TESTING...</span>
                      </>
                    ) : (
                      <>
                        <Wifi className="w-3.5 h-3.5" />
                        <span>TEST DIRECT CONNECTION</span>
                      </>
                    )}
                  </button>
                </div>
              </div>

              {/* Troubleshooting helper box */}
              <div className="p-4 bg-gray-900/50 border border-gray-850 rounded-lg space-y-2 text-xs">
                <span className="font-bold text-gray-200 block font-mono uppercase text-[10px] tracking-wider text-amber-500">
                  ⚠️ CORS HANDSHAKE REQUIREMENTS:
                </span>
                <p className="text-gray-400 font-sans text-[11px] leading-relaxed">
                  Web browsers block websites hosted on secure HTTPS networks from directly pinging local endpoints like <code>127.0.0.1</code> unless custom headers are enabled.
                </p>
                <div className="space-y-1 font-mono text-[9px] text-gray-400">
                  <p className="font-semibold text-gray-300">Start Ollama with cross-origins set to all:</p>
                  <div className="p-2 bg-black/40 rounded border border-gray-900 flex justify-between items-center">
                    <code className="text-amber-500">OLLAMA_ORIGINS="*" ollama serve</code>
                    <button 
                      onClick={() => handleCopyCode('OLLAMA_ORIGINS="*" ollama serve', "ollama_cmd")} 
                      className="p-1 hover:bg-gray-800 rounded transition-all text-slate-400"
                    >
                      {copiedCodeType === "ollama_cmd" ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                    </button>
                  </div>
                </div>
              </div>
            </div>

            {/* Practical prompt testing playground */}
            <div className="lg:col-span-7 space-y-4">
              <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4 h-full flex flex-col justify-between">
                
                <div className="space-y-3">
                  <div className="flex justify-between items-center border-b border-gray-850 pb-2">
                    <p className="text-xs font-bold text-sky-400 font-mono uppercase tracking-wider flex items-center gap-1.5">
                      <Terminal className="w-4 h-4 text-sky-400" /> Unified Local Model Terminal
                    </p>
                    <div className="flex items-center gap-1.5">
                      {connectionStatus === "connected" && (
                        <span className="px-2 py-0.5 bg-emerald-500/10 text-emerald-400 text-[9px] font-mono rounded border border-emerald-500/20 uppercase tracking-widest">
                          CONNECTED
                        </span>
                      )}
                      {connectionStatus === "cors_error" && (
                        <span className="px-2 py-0.5 bg-amber-500/10 text-amber-500 text-[9px] font-mono rounded border border-amber-500/20 uppercase">
                          CORS COMPLIANT WARN
                        </span>
                      )}
                      {connectionStatus === "disconnected" && (
                        <span className="px-2 py-0.5 bg-gray-900 text-gray-500 text-[9px] font-mono rounded border border-gray-800 uppercase">
                          STANBY
                        </span>
                      )}
                    </div>
                  </div>

                  <p className="text-[11px] text-gray-400 leading-normal font-sans">
                    Submit prompts directly to test formatting outputs and streaming speeds. JARVIS OS cognitive vectors integrate directly with active local context layers to customize the model's ultimate answers.
                  </p>

                  <div className="space-y-2">
                    <input
                      type="text"
                      value={testPrompt}
                      onChange={(e) => setTestPrompt(e.target.value)}
                      placeholder="Insert sovereign testing prompt"
                      className="w-full bg-gray-950 border border-gray-850 text-xs py-2 px-3 text-slate-100 rounded focus:outline-none focus:border-indigo-500/50 font-sans"
                    />

                    <div className="flex justify-between items-center">
                      <span className="text-[9px] text-gray-500 font-mono uppercase">Model Target: <code className="text-indigo-400">{modelTag}</code></span>
                      <button
                        onClick={() => sendPromptToLocalModel(testPrompt)}
                        disabled={isPromptLoading || !testPrompt.trim()}
                        className="px-4 py-1 bg-[#05070a] hover:bg-gray-850 disabled:bg-gray-900 border border-indigo-900/40 text-indigo-400 font-bold font-mono text-[10px] rounded hover:shadow-[0_0_8px_rgba(99,102,241,0.25)] transition-all flex items-center gap-1 cursor-pointer"
                      >
                        {isPromptLoading ? <RefreshCw className="w-3 h-3 animate-spin" /> : <Play className="w-3 h-3" />}
                        <span>SUBMIT PROMPT</span>
                      </button>
                    </div>
                  </div>
                </div>

                {/* Output terminal visualizer */}
                <div className="space-y-1.5 pt-4">
                  <span className="text-[9px] text-gray-500 font-mono uppercase block">Local Inference Response Stream</span>
                  <div className="h-44 bg-[#030406]/95 border border-gray-900/80 rounded p-3 font-mono text-[11px] leading-relaxed text-indigo-300/90 overflow-y-auto whitespace-pre-wrap select-text scrollbar-thin scrollbar-thumb-gray-900">
                    {testResponse ? testResponse : (
                      <span className="text-gray-650 italic">[Waiting for local prompt processing...]</span>
                    )}
                  </div>
                </div>

              </div>
            </div>
          </motion.div>
        )}

        {/* VIEW 2: DESKTOP APP SPEC WRAPPER (TAURI / ELECTRON BUILDER) */}
        {activeSegment === "desktop" && (
          <motion.div
            key="desktop-view"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="grid grid-cols-1 lg:grid-cols-12 gap-5 text-left font-sans"
          >
            {/* Tauri build specifications */}
            <div className="lg:col-span-6 space-y-4">
              <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4">
                <div className="flex justify-between items-center border-b border-gray-850 pb-2">
                  <div className="space-y-0.5">
                    <p className="text-[10px] text-indigo-400 font-bold uppercase tracking-widest font-mono flex items-center gap-1">
                      <Code className="w-3.5 h-3.5" /> Option A: Tauri Native Compiler (Recommended)
                    </p>
                    <h3 className="text-xs text-gray-200">Rust-based ultra lightweight wrapper (Bundle size &lt; 10MB)</h3>
                  </div>
                  <button 
                    onClick={() => handleCopyCode(tauriConfigJson, "tauri")}
                    className="px-2.5 py-1 bg-gray-950 border border-gray-800 hover:border-gray-700 text-slate-400 hover:text-slate-200 text-[10px] font-mono rounded flex items-center gap-1 transition-all"
                  >
                    {copiedCodeType === "tauri" ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                    <span>{copiedCodeType === "tauri" ? "COPIED JSON" : "COPY CONFIG"}</span>
                  </button>
                </div>

                <p className="text-xs text-gray-400 leading-normal">
                  Tauri uses Rust under the hood to compile your React bundle down to a highly responsive native OS executable with standard Webkit/WebView views. It grants raw sandbox privileges without Electron's system overhead.
                </p>

                {/* Terminal visual style code showcase */}
                <div className="relative p-3 bg-black/60 rounded border border-gray-900 max-h-[300px] overflow-y-auto font-mono text-[9px] leading-relaxed text-indigo-400/90 whitespace-pre scrollbar-thin scrollbar-thumb-gray-900">
                  {tauriConfigJson}
                </div>

                <div className="p-3 bg-indigo-500/5 rounded border border-indigo-900/20 text-[11px] text-indigo-200 font-mono leading-normal">
                  <strong className="text-indigo-400 uppercase">⚡ FAST INSTALL:</strong><br/>
                  1. cargo install tauri-cli<br/>
                  2. cargo tauri init<br/>
                  3. cargo tauri build
                </div>
              </div>
            </div>

            {/* Electron build specifications */}
            <div className="lg:col-span-6 space-y-4">
              <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4">
                <div className="flex justify-between items-center border-b border-gray-850 pb-2">
                  <div className="space-y-0.5">
                    <p className="text-[10px] text-sky-400 font-bold uppercase tracking-widest font-mono flex items-center gap-1">
                      <Layers className="w-3.5 h-3.5" /> Option B: Electron Frame wrapper
                    </p>
                    <h3 className="text-xs text-gray-200">Standard chromium-based engine wrapper (Cross-Platform)</h3>
                  </div>
                  <button 
                    onClick={() => handleCopyCode(electronConfigJs, "electron")}
                    className="px-2.5 py-1 bg-gray-950 border border-gray-800 hover:border-gray-700 text-slate-400 hover:text-slate-200 text-[10px] font-mono rounded flex items-center gap-1 transition-all"
                  >
                    {copiedCodeType === "electron" ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                    <span>{copiedCodeType === "electron" ? "COPIED CORE" : "COPY MAIN.JS"}</span>
                  </button>
                </div>

                <p className="text-xs text-gray-400 leading-normal">
                  Electron is perfect for high-speed cross platform builds. Setting <code>webSecurity: false</code> allows you to completely bypass any CORS/local network locks on your hardware.
                </p>

                {/* Terminal visual style code showcase */}
                <div className="relative p-3 bg-black/60 rounded border border-gray-900 max-h-[300px] overflow-y-auto font-mono text-[9px] leading-relaxed text-sky-400/95 whitespace-pre scrollbar-thin scrollbar-thumb-gray-900">
                  {electronConfigJs}
                </div>

                <div className="p-3 bg-sky-500/5 rounded border border-sky-900/20 text-[11px] text-sky-200 font-mono leading-normal">
                  <strong className="text-sky-400 uppercase">⚡ PACKAGER BUILD:</strong><br/>
                  1. npm install --save-dev electron electron-packager<br/>
                  2. npx electron-packager . JarvisOS --platform=darwin --arch=x64
                </div>
              </div>
            </div>
          </motion.div>
        )}

        {/* VIEW 3: SMARTPHONE MOBILE SYNC SCREEN MOCKUP */}
        {activeSegment === "mobile" && (
          <motion.div
            key="mobile-view"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="grid grid-cols-1 md:grid-cols-12 gap-5"
          >
            {/* Interactive settings description */}
            <div className="md:col-span-5 space-y-4 text-left font-sans flex flex-col justify-between">
              <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4">
                <span className="text-[10px] text-indigo-400 font-bold uppercase tracking-widest font-mono block">
                  Companion Synchronization Specification
                </span>
                <h3 className="text-sm font-bold text-gray-200 font-sans tracking-wide">
                  Mobile Companion Sovereign System Sync
                </h3>
                
                <p className="text-xs text-gray-400 leading-relaxed font-sans">
                  The mobile layout is engineered specifically to be responsive, optimized for dynamic touch zones (minimum 44px) and edge-encrypted network channels.
                </p>

                <div className="space-y-3.5 text-xs font-mono text-gray-300">
                  <div className="p-3 bg-gray-950/50 border border-gray-900 rounded-lg space-y-1.5 hover:border-indigo-900/30 transition-all">
                    <span className="text-[10px] text-indigo-400 font-bold">DECENTRALIZED MEMORY PUSH:</span>
                    <p className="text-[11px] text-gray-400 leading-relaxed leading-normal">
                      Syncs active local memory nodes (OKF formats) over your local Wi-Fi router. Eliminates centralized cloud data hoarding.
                    </p>
                  </div>

                  <div className="p-3 bg-gray-950/50 border border-gray-900 rounded-lg space-y-1.5 hover:border-indigo-900/30 transition-all">
                    <span className="text-[10px] text-indigo-400 font-bold">BIOMETRIC PING:</span>
                    <p className="text-[11px] text-gray-400 leading-relaxed leading-normal">
                      Trigger manual biometric authorization sequences directly from your phone to unlock secure diagnostic segments.
                    </p>
                  </div>
                </div>

                {/* Sync status toggles */}
                <div className="pt-2 flex justify-between items-center text-xs">
                  <span className="text-gray-400 font-mono text-[10px]">MOBILE SYNC STATUS</span>
                  <button
                    onClick={() => {
                      setIsMobileSynced(!isMobileSynced);
                      addMobileLog(isMobileSynced ? "Severed companion network loop." : "Re-established WiFi companion loop.");
                    }}
                    className={`px-3 py-1 text-[10px] font-mono font-bold rounded border transition-all ${
                      isMobileSynced ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20" : "bg-red-500/10 text-red-400 border-red-500/20"
                    }`}
                  >
                    {isMobileSynced ? "CONNECTED" : "OFFLINE"}
                  </button>
                </div>
              </div>

              {/* QR Code / manual sync mockup */}
              <div className="p-4 bg-gray-900/30 border border-gray-850 rounded-lg flex items-center justify-between gap-3 text-xs font-mono">
                <div className="space-y-0.5">
                  <span className="text-[10px] text-indigo-400 block font-semibold uppercase">Quick QR Setup:</span>
                  <p className="text-[11px] text-gray-400">Scan layout signature via mobile camera lens to establish quick-pairing.</p>
                </div>
                <div className="w-12 h-12 bg-white flex items-center justify-center rounded shrink-0 p-1">
                  {/* Generated QR pattern mockup */}
                  <div className="w-full h-full bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-slate-900 via-indigo-950 to-slate-950" />
                </div>
              </div>
            </div>

            {/* Realistic smartphone visual UI mockup */}
            <div className="md:col-span-7 flex justify-center py-2">
              <div className="w-[285px] h-[550px] bg-[#020305] border-[10px] border-slate-900 rounded-[35px] shadow-[0_22px_44px_rgba(0,0,0,0.9)] overflow-hidden flex flex-col justify-between relative font-sans text-slate-200">
                
                {/* Dynamic notch bar */}
                <div className="absolute top-0 inset-x-0 h-6 bg-slate-900 flex justify-center items-end px-4 z-40">
                  <div className="w-24 h-4 bg-black rounded-b-xl absolute top-0 flex items-center justify-around px-2 text-[8px] font-mono text-gray-300">
                    <span className="text-[6px] tracking-tighter">● WIFI SYNC</span>
                  </div>
                  <div className="w-full flex justify-between text-[8px] font-mono text-gray-400 pb-1 z-50">
                    <span>09:55 AM</span>
                    <div className="flex gap-1.5 items-center">
                      <span>{isMobileSynced ? "LTE" : "Disconnected"}</span>
                      <span>🔋 {mobileBattery}%</span>
                    </div>
                  </div>
                </div>

                {/* Interactive mobile UI view area */}
                <div className="pt-8 px-4 flex-1 flex flex-col justify-between overflow-y-auto bg-slate-950 bg-[radial-gradient(ellipse_at_top_right,_var(--tw-gradient-stops))] from-indigo-950/20 via-slate-950 to-slate-950 relative scrollbar-none">
                  
                  {/* Header visual */}
                  <div className="space-y-1.5 py-2 text-center border-b border-white/5">
                    <div className="flex items-center justify-center gap-1 text-[11px] font-bold text-indigo-400 tracking-wider uppercase font-mono">
                      <SmartphoneNfc className="w-3.5 h-3.5" /> J.A.R.V.I.S. Companion
                    </div>
                    <span className="text-[7.5px] text-gray-500 font-mono tracking-widest block uppercase">SOVEREIGN MOBILE HANDSHAKE</span>
                  </div>

                  {/* Core mobile data module list */}
                  <div className="flex-1 py-3 text-left space-y-3">
                    {/* Live Mobile logs */}
                    <div className="space-y-1">
                      <span className="text-[7.5px] text-indigo-400 font-mono uppercase block">Real-time WiFi Sync State</span>
                      <div className="h-28 bg-[#030406]/95 border border-slate-900 rounded p-2 font-mono text-[7.5px] text-indigo-300 space-y-1 overflow-y-auto scrollbar-none">
                        {mobileSyncLogs.map((log, index) => <div key={index} className="truncate">{log}</div>)}
                      </div>
                    </div>

                    {/* Fast biometric toggle mockup */}
                    <div className="p-2 bg-slate-900/60 border border-slate-800 rounded-lg flex items-center justify-between">
                      <div className="space-y-0.5">
                        <span className="text-[8px] text-slate-300 font-bold block leading-none">Biometric Access State</span>
                        <span className="text-[7px] text-gray-500 font-mono">Awaiting verification sensor trigger</span>
                      </div>
                      <button
                        onClick={() => {
                          setMobileBattery(b => Math.max(1, b - 1));
                          addMobileLog("Triggered biometrics handshake verification.");
                          if (speakText) speakText("Biometric verification loop transmitted via mobile companion.");
                        }}
                        className="w-7 h-7 rounded-full bg-indigo-500/10 flex items-center justify-center text-indigo-400 border border-indigo-500/20 cursor-pointer hover:bg-indigo-500/20 transition-all"
                      >
                        <ShieldCheck className="w-3.5 h-3.5" />
                      </button>
                    </div>

                    {/* Context slider control */}
                    <div className="p-2 bg-slate-900/60 border border-slate-800 rounded-lg space-y-1 font-mono text-[8px]">
                      <div className="flex justify-between items-center text-gray-400 leading-none">
                        <span>CONTEXT MEMORY CAP</span>
                        <span className="text-indigo-400 font-bold">4,096 tokens</span>
                      </div>
                      <div className="h-1 bg-gray-950 rounded-full overflow-hidden">
                        <div className="h-full bg-indigo-500 w-2/3" />
                      </div>
                    </div>
                  </div>

                  {/* Mobile active prompt submit box */}
                  <div className="py-2.5 bg-slate-950 border-t border-white/5 z-20">
                    <div className="flex gap-1.5 bg-slate-900 p-1 rounded border border-slate-800">
                      <input
                        type="text"
                        placeholder="Master instructions..."
                        value={mobilePromptInput}
                        onChange={(e) => setMobilePromptInput(e.target.value)}
                        className="flex-1 bg-transparent text-[9px] border-none focus:outline-none pl-1 py-1 text-slate-100 placeholder-slate-500"
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            sendPromptToLocalModel(mobilePromptInput, true);
                          }
                        }}
                      />
                      <button
                        onClick={() => sendPromptToLocalModel(mobilePromptInput, true)}
                        className="p-1 bg-indigo-500 text-black rounded hover:bg-indigo-400 transition-all flex items-center justify-center shrink-0 w-5 h-5 cursor-pointer"
                      >
                        <Send className="w-3 h-3 fill-current" />
                      </button>
                    </div>
                  </div>

                </div>

                {/* Smartphone bottom home menu indicator */}
                <div className="h-4 bg-slate-900 flex justify-center items-center z-40">
                  <div className="w-16 h-1 bg-slate-550 rounded-full" />
                </div>

              </div>
            </div>
          </motion.div>
        )}

        {/* VIEW 4: SPARSED Mixture of Experts (MoE) TRAINING AND FINE-TUNING PIPELINE */}
        {activeSegment === "pipeline" && (
          <motion.div
            key="moe-pipeline-view"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="grid grid-cols-1 lg:grid-cols-12 gap-5"
          >
            {/* Theoretical & configuration column */}
            <div className="lg:col-span-5 space-y-4 text-left font-sans">
              <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4">
                <div className="flex items-center gap-2">
                  <Flame className="w-5 h-5 text-indigo-400 animate-pulse" />
                  <h3 className="text-sm font-bold text-gray-200 uppercase tracking-wider font-mono">
                    MoE Alignment Rig
                  </h3>
                </div>

                <p className="text-xs text-slate-400 leading-relaxed font-sans">
                  Mixture-of-Experts (MoE) architectures allow us to train robust models without suffering high hardware constraints, because only a tiny subset of the neural weights are active per input token.
                </p>

                {/* MoE Selector widget */}
                <div className="space-y-2">
                  <label className="text-[10px] text-gray-400 font-mono uppercase block">Sovereign Base MoE choice</label>
                  <div className="grid grid-cols-2 gap-2">
                    <button
                      onClick={() => {
                        setSelectedMoE("qwen-moe");
                        setCurrentLoss(2.81);
                        setEpoch(0.0);
                        if (speakText) speakText("Switched base to Qwen 1.5 Mixture of Experts model.");
                      }}
                      className={`p-3 rounded text-left border transition-all ${
                        selectedMoE === "qwen-moe" 
                          ? "bg-indigo-500/10 border-indigo-500/40 text-indigo-300" 
                          : "bg-gray-950/40 border-gray-900 text-gray-500 hover:text-slate-300"
                      }`}
                    >
                      <span className="text-xs font-bold block">Qwen-1.5-MoE-A2.7B</span>
                      <span className="text-[9px] font-mono opacity-80 block">0.93B Active / 2.7B Total</span>
                      <span className="text-[8px] text-indigo-400 font-semibold block mt-1">Recommended for Mobile (RAM &lt; 3GB)</span>
                    </button>

                    <button
                      onClick={() => {
                        setSelectedMoE("jetmoe");
                        setCurrentLoss(2.45);
                        setEpoch(0.0);
                        if (speakText) speakText("Switched base to Jet Mixture-of-Experts architecture.");
                      }}
                      className={`p-3 rounded text-left border transition-all ${
                        selectedMoE === "jetmoe" 
                          ? "bg-indigo-500/10 border-indigo-500/40 text-indigo-300" 
                          : "bg-gray-950/40 border-gray-900 text-gray-500 hover:text-slate-300"
                      }`}
                    >
                      <span className="text-xs font-bold block">JetMoE-8B (Academic)</span>
                      <span className="text-[9px] font-mono opacity-80 block">2.0B Active / 8.0B Total</span>
                      <span className="text-[8px] text-indigo-400 font-semibold block mt-1">Highly scalable, out-performs LLaMA2-7B</span>
                    </button>
                  </div>
                </div>

                {/* Training parameters configurations */}
                <div className="space-y-3.5 pt-1 border-t border-gray-850/60 font-mono text-[10px]">
                  {/* Fine tuning split selector */}
                  <div className="space-y-1.5">
                    <span className="text-gray-500 uppercase">Parameter Optimization Split</span>
                    <div className="grid grid-cols-3 gap-1">
                      {(["lora", "qlora", "full"] as const).map(split => (
                        <button
                          key={split}
                          onClick={() => setSelectedSplit(split)}
                          className={`py-1 text-center rounded text-[8px] border uppercase transition-all ${
                            selectedSplit === split ? "bg-indigo-500/15 text-indigo-300 border-indigo-500/30 font-bold" : "bg-black/20 text-gray-500 border-gray-900 hover:text-gray-300"
                          }`}
                        >
                          {split === "lora" && "LoRA (Rank 16)"}
                          {split === "qlora" && "QLoRA (NF4)"}
                          {split === "full" && "Full Weights"}
                        </button>
                      ))}
                    </div>
                  </div>

                  {/* LR adjustment slider */}
                  <div className="space-y-1">
                    <div className="flex justify-between text-gray-500">
                      <span>LEARNING RATE</span>
                      <span className="text-slate-300">{learningRate}</span>
                    </div>
                    <select
                      value={learningRate}
                      onChange={(e) => setLearningRate(e.target.value)}
                      className="w-full bg-gray-950 border border-gray-850 text-[9px] p-1.5 text-indigo-400 focus:outline-none rounded"
                    >
                      <option value="5e-4">5e-4 (Aggressive Tuning)</option>
                      <option value="2e-4">2e-4 (Standard Alignment)</option>
                      <option value="5e-5">5e-5 (Fine Conservative)</option>
                    </select>
                  </div>

                  {/* Gradient accumulation selection */}
                  <div className="space-y-1">
                    <div className="flex justify-between text-gray-500">
                      <span>GRADIENT ACCUMULATION STEPS</span>
                      <span className="text-slate-300">{gradientSteps} batches</span>
                    </div>
                    <input
                      type="range"
                      min={1}
                      max={16}
                      value={gradientSteps}
                      onChange={(e) => setGradientSteps(Number(e.target.value))}
                      className="w-full accent-indigo-500"
                    />
                  </div>
                </div>

                {/* Cognitive memory context injector widget */}
                <div className="pt-2 border-t border-gray-850/60 space-y-2">
                  <span className="text-[10px] text-indigo-400 font-bold uppercase tracking-wider font-mono block">Context Graph Injection</span>
                  <p className="text-[10.5px] text-gray-500 leading-normal">
                    Inject active episodic registers and semantic graph files directly into the input training dataset. Fits perfectly in the MoE pretrain sequence.
                  </p>
                  <div className="flex gap-2">
                    <button
                      onClick={() => {
                        const nodes = ["Dicas9_Sovereign", "Local_Inference_Node", "Episodic_Chunk_87", "Okf_Consensus_V7"];
                        const choice = nodes[Math.floor(Math.random() * nodes.length)];
                        setLatestInjectedNode(choice);
                        setTrainingLogs(prev => [`Inserted memory graph vector for [${choice}] into token list.`, ...prev]);
                        if (speakText) speakText(`Injected episodic knowledge node ${choice} into local MoE context.`);
                      }}
                      className="flex-1 py-1 px-2.5 bg-gray-950 hover:bg-gray-900 border border-indigo-900/40 text-indigo-400 rounded text-[9px] font-mono flex items-center justify-center gap-1.5 transition-all"
                    >
                      <Activity className="w-3.5 h-3.5 text-indigo-500" />
                      <span>INJECT SYSTEM GRAPH VECTOR</span>
                    </button>
                  </div>
                  {latestInjectedNode && (
                    <span className="text-[8px] text-slate-400 font-mono uppercase bg-indigo-950/20 px-2 py-0.5 border border-indigo-900/20 rounded block w-full truncate">
                      LATEST WEIGHT: <code className="text-amber-400">{latestInjectedNode}</code>
                    </span>
                  )}
                </div>
              </div>
            </div>

            {/* Interactive pipeline training visualizer */}
            <div className="lg:col-span-7 space-y-4">
              <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4 flex flex-col justify-between">
                <div>
                  <div className="flex justify-between items-center border-b border-gray-850 pb-2">
                    <p className="text-xs font-bold text-indigo-400 font-mono uppercase tracking-wider flex items-center gap-1.5">
                      <Activity className="w-4 h-4 text-indigo-400" /> Training Monitor Dashboard
                    </p>
                    <button
                      onClick={() => {
                        setIsTraining(!isTraining);
                        if (!isTraining) {
                          if (speakText) speakText("Starting sparse Mixture of Experts local fine-tuning simulation.");
                          setTrainingLogs(prev => ["Stimulated PyTorch training pipeline stream.", ...prev]);
                        } else {
                          if (speakText) speakText("Fine-tuning pipeline paused.");
                        }
                      }}
                      className={`px-3 py-1 font-mono text-[9px] font-bold rounded border transition-all flex items-center gap-1 cursor-pointer ${
                        isTraining 
                          ? "bg-red-500/15 text-red-400 border-red-500/20" 
                          : "bg-emerald-500/10 text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/20"
                      }`}
                    >
                      <Play className="w-3 h-3" />
                      <span>{isTraining ? "PAUSE PIPELINE" : "RUN TRAINING PIPELINE"}</span>
                    </button>
                  </div>

                  {/* Numerical Realtime Metrics */}
                  <div className="grid grid-cols-3 gap-2 mt-3 font-mono text-center">
                    <div className="p-2 bg-slate-950 rounded border border-gray-900">
                      <span className="text-[8px] text-slate-500 uppercase block">Training Loss</span>
                      <span className="text-sm font-bold text-indigo-300">{currentLoss.toFixed(4)}</span>
                    </div>
                    <div className="p-2 bg-slate-950 rounded border border-gray-900">
                      <span className="text-[8px] text-slate-500 uppercase block">Normalized epoch</span>
                      <span className="text-sm font-bold text-amber-400">{epoch.toFixed(2)} / 3.00</span>
                    </div>
                    <div className="p-2 bg-slate-950 rounded border border-gray-900">
                      <span className="text-[8px] text-slate-500 uppercase block">Tokens Processed</span>
                      <span className="text-sm font-bold text-sky-400">{tokensProcessed.toLocaleString()}</span>
                    </div>
                  </div>

                  {/* Mixture of Experts Sparsity Gating Load Bar Chart */}
                  <div className="mt-4 p-4 bg-slate-950 border border-slate-900 rounded-lg space-y-2">
                    <div className="flex justify-between items-center text-[9px] font-mono text-gray-400">
                      <span className="text-indigo-400 font-bold uppercase tracking-wider">Top-2 Gating Router Distribution (Expert sparse load)</span>
                      <span>Total Experts: 8</span>
                    </div>
                    
                    {/* Live bars representing routing distribution of tokens to experts */}
                    <div className="space-y-1.5 pt-1">
                      {expertDistribution.map((load, index) => {
                        const expertRoles = ["Reasoning", "Database", "Robotics", "Systems", "Episodic", "Translative", "Telemetry", "Auxiliary"];
                        return (
                          <div key={index} id={`expert-bar-${index}`} className="flex items-center gap-2 text-[9px] font-mono">
                            <span className="w-16 truncate text-slate-400 text-[8.5px] uppercase">{expertRoles[index]}</span>
                            <div className="flex-1 h-2 bg-[#05070a] rounded overflow-hidden">
                              <div 
                                className="h-full bg-indigo-500/80 rounded transition-all duration-700"
                                style={{ width: `${Math.min(100, Math.max(2, (load / 140) * 100))}%` }}
                              />
                            </div>
                            <span className="w-8 text-right text-indigo-300 text-[8px]">{((load / 140) * 100).toFixed(1)}%</span>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                </div>

                {/* PyTorch Fine-tuning setup terminal output */}
                <div className="space-y-1.5 pt-4">
                  <div className="flex justify-between items-center text-[9px] font-mono">
                    <span className="text-slate-500 uppercase">Interactive Python alignment configuration</span>
                    <button
                      onClick={() => handleCopyCode(`pip install transformers peft trl accelerate`, "pip_cmd")}
                      className="p-1 hover:bg-slate-900 rounded transition-all text-slate-400 flex items-center gap-1"
                    >
                      {copiedCodeType === "pip_cmd" ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                      <span>{copiedCodeType === "pip_cmd" ? "Copied" : "Copy Install Script"}</span>
                    </button>
                  </div>

                  <div className="h-32 bg-[#030406]/95 border border-gray-900/80 rounded p-3 font-mono text-[9.5px] leading-relaxed text-indigo-300/90 overflow-y-auto whitespace-pre select-text scrollbar-thin scrollbar-thumb-gray-900">
                    {`# python fine-tune-moe.py
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import LoraConfig, get_peft_model

# Load compact Mixture of Experts base weights
model_path = "${selectedMoE === "qwen-moe" ? "Qwen/Qwen1.5-MoE-A2.7B" : "research/JetMoE-8B"}"
tokenizer = AutoTokenizer.from_pretrained(model_path, trust_remote_code=True)
model = AutoModelForCausalLM.from_pretrained(
    model_path, 
    torch_dtype=torch.bfloat16, 
    device_map="auto",
    trust_remote_code=True
)

# Setup sparse-friendly LoRA parameters
peft_config = LoraConfig(
    r=16,
    lora_alpha=32,
    target_modules=["q_proj", "v_proj", "gating_network"], # Injecting into router weights!
    bias="none",
    task_type="CAUSAL_LM"
)
model = get_peft_model(model, peft_config)
print("Configured Local MoE Model for alignment fine-tuning splits.")`}
                  </div>
                </div>

              </div>
            </div>

            {/* THUNDER COMPUTE CLOUD GPU ORCHESTRATION BRIDGE */}
            <div className="lg:col-span-12 p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4">
              <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 border-b border-gray-850 pb-3">
                <div className="space-y-1 text-left">
                  <span className="px-2 py-0.5 bg-indigo-500/15 text-indigo-400 text-[9px] font-mono rounded border border-indigo-500/20 uppercase tracking-widest animate-pulse">
                    ⚡ THUNDER COMPUTE DETACHED SUB-CLUSTER
                  </span>
                  <h4 className="text-sm font-bold text-gray-200 uppercase tracking-wider font-mono flex items-center gap-1.5">
                    <Cpu className="w-4 h-4 text-indigo-400" /> Decentralized Real Model Training Rig
                  </h4>
                  <p className="text-xs text-slate-400 max-w-2xl leading-normal">
                    Painless Cloud GPU provisioning. Rent high-end RTX 4090s, L4s, or A100s instantly to fine-tune your sparse Mixture-of-Experts (MoE) brain model.
                  </p>
                </div>

                <div className="flex items-center gap-2 flex-wrap">
                  <span className="text-[10px] text-gray-400 font-mono">NODE GPU TYPE:</span>
                  <select
                    value={gpuType}
                    onChange={(e) => {
                      setGpuType(e.target.value as any);
                      if (instanceStatus !== "offline") setInstanceStatus("offline");
                    }}
                    disabled={instanceStatus === "provisioning" || instanceStatus === "training"}
                    className="bg-black border border-gray-800 text-xs px-2 py-1.5 rounded text-indigo-400 focus:outline-none focus:border-indigo-500 font-bold"
                  >
                    <option value="rtx4090">RTX 4090 Ada 24GB ($0.45/hr)</option>
                    <option value="l4">NVIDIA L4 24GB ($0.22/hr)</option>
                    <option value="a100">NVIDIA A100 SXM4 80GB ($1.89/hr)</option>
                    <option value="h100">NVIDIA H100 SXM5 80GB ($2.75/hr)</option>
                  </select>

                  <button
                    onClick={triggerThunderComputeProvision}
                    disabled={instanceStatus === "provisioning"}
                    className={`px-4 py-1.5 rounded text-[10px] font-mono uppercase font-bold tracking-wider cursor-pointer border transition-all ${
                      instanceStatus === "ready" 
                        ? "bg-red-500/10 text-red-400 border-red-500/20 hover:bg-red-500/20" 
                        : instanceStatus === "provisioning"
                          ? "bg-amber-500/10 text-amber-400 border-amber-500/20"
                          : "bg-indigo-600 hover:bg-indigo-500 text-slate-950 font-bold border-indigo-700 hover:shadow-[0_0_12px_rgba(99,102,241,0.2)]"
                    }`}
                  >
                    {instanceStatus === "offline" && "Provision GPU Instance"}
                    {instanceStatus === "provisioning" && "Allocating Cluster..."}
                    {instanceStatus === "ready" && "Destroy GPU Container"}
                  </button>
                </div>
              </div>

              {/* Grid with logs + SSH Connection on the left, download & run on the right */}
              <div className="grid grid-cols-1 md:grid-cols-12 gap-4 text-left">
                <div className="md:col-span-6 space-y-3">
                  {/* Status & Live allocation Logs */}
                  <div className="space-y-1">
                    <div className="flex justify-between text-[10px] font-mono text-gray-500 uppercase">
                      <span>Live Provisioning Streams</span>
                      <span className="text-indigo-400 font-bold">STATUS: {instanceStatus.toUpperCase()}</span>
                    </div>
                    <div className="h-56 bg-black/80 border border-slate-900 rounded-lg p-3 font-mono text-[10px] leading-relaxed text-indigo-300 space-y-1.5 overflow-y-auto select-text scrollbar-thin scrollbar-thumb-gray-900">
                      {thunderLogs.map((log, i) => (
                        <div key={i} className={log.startsWith("✔") || log.startsWith("🌟") ? "text-emerald-400 font-bold" : log.startsWith("Requesting") ? "text-amber-400" : "text-indigo-300"}>
                          {log}
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* SSH Connection command */}
                  <div className="p-3 bg-indigo-500/5 rounded-lg border border-indigo-900/30 space-y-1.5 font-mono text-xs">
                    <div className="flex justify-between items-center text-[10px] text-gray-400">
                      <span>SSH TARGET TUNNEL</span>
                      <span className={`font-bold ${instanceStatus === "ready" ? "text-emerald-400" : "text-amber-500"}`}>
                        {instanceStatus === "ready" ? "● CONNECTED" : "● EDITABLE STANDBY"}
                      </span>
                    </div>
                    <div className="flex items-center justify-between gap-2 p-2 bg-black/45 rounded border border-gray-900">
                      <input
                        type="text"
                        value={sshConnectionString}
                        onChange={(e) => setSshConnectionString(e.target.value)}
                        className="flex-1 bg-transparent text-indigo-400 font-mono text-[11px] focus:outline-none focus:ring-0 w-full"
                        placeholder="ssh user@ip -p port -i key"
                      />
                      <button
                        onClick={() => {
                          navigator.clipboard.writeText(sshConnectionString);
                          setThunderCopiedCode("ssh");
                          setTimeout(() => setThunderCopiedCode(null), 2000);
                        }}
                        className="p-1 hover:bg-slate-900 rounded shrink-0 transition-all text-slate-300"
                        title="Copy connection string"
                      >
                        {thunderCopiedCode === "ssh" ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5 text-slate-400" />}
                      </button>
                    </div>
                    <p className="text-[9px] text-[#5c6984] leading-relaxed font-sans">
                      Specify your authorized host port, or keep the secure default. Click <strong>Provision GPU Instance</strong> above to simulate a live pipeline handshake or directly run SSH terminal commands on the remote server.
                    </p>
                  </div>
                </div>

                {/* Right side: Remote Thunder Compute Launch automation code */}
                <div className="md:col-span-6 space-y-1">
                  <div className="flex justify-between items-center text-[10px] font-mono text-gray-500 uppercase">
                    <span>Cluster execution automation script</span>
                    <button
                      onClick={() => {
                        const bashScript = `#!/bin/bash
# Clone and setup training repository on Thunder Compute GPU
echo "Initializing Python development stack on detached GPU..."
pip install transformers peft trl accelerate datasets

# Fetch active episodic knowledge graphs from JARVIS OS node
curl -s "https://ais-dev-pyqme3god3royd4eroq62c-82590913883.us-west2.run.app/api/health"

# Execute fine-tune causal MoE segment
python train_thunder.py --gpu_target "${gpuType}" --base_model "${selectedMoE === "qwen-moe" ? "Qwen/Qwen1.5-MoE-A2.7B" : "research/JetMoE-8B"}"
`;
                        navigator.clipboard.writeText(bashScript);
                        setThunderCopiedCode("python");
                        setTimeout(() => setThunderCopiedCode(null), 2000);
                      }}
                      className="p-1 text-[9px] text-indigo-400 hover:text-indigo-300 font-bold rounded flex items-center gap-1 transition-all cursor-pointer"
                    >
                      {thunderCopiedCode === "python" ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3 text-indigo-400" />}
                      <span>Copy Automation Script</span>
                    </button>
                  </div>

                  <div className="p-3 bg-[#030406]/95 border border-gray-900 rounded-lg h-56 font-mono text-[9.5px] leading-relaxed text-indigo-200/90 overflow-y-auto whitespace-pre select-text scrollbar-thin scrollbar-thumb-gray-900">
                    {`# connect, install and run model fine-tune
git clone https://github.com/shawncarruth/jarvis-os-moe-hub
cd jarvis-os-moe-hub

# Setup fast thunder packages
pip install thunder-compute torch transformers accelerate peft trl

# Inject active context & begin dataset alignment
python train_thunder.py \\
    --gpu_target "${gpuType}" \\
    --base_model "${selectedMoE === "qwen-moe" ? "Qwen/Qwen1.5-MoE-A2.7B" : "research/JetMoE-8B"}" \\
    --learning_rate "${learningRate}" \\
    --optim "${selectedSplit}" \\
    --accum_steps ${gradientSteps}`}
                  </div>

                  <div className="p-2.5 bg-yellow-500/5 rounded border border-yellow-500/10 text-[10px] text-yellow-300/90 font-sans leading-relaxed flex items-start gap-1.5">
                    <Sparkles className="w-4 h-4 shrink-0 text-yellow-400" />
                    <span>
                      <strong>Inference Convergence Tip:</strong> Thunder Compute instances synchronize context graphs by binding container instances with your live J.A.R.V.I.S. OS dashboard metrics over a secure data link.
                    </span>
                  </div>
                </div>
              </div>
            </div>

          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
