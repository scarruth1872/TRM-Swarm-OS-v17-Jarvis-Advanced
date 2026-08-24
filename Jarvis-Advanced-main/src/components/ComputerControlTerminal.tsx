/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import React, { useState, useRef, useEffect } from "react";
import { Terminal, Play, Shield, RefreshCw, Layers } from "lucide-react";

interface ComputerControlTerminalProps {
  onExecuteCommand: (command: string) => Promise<string[]>;
}

export default function ComputerControlTerminal({ onExecuteCommand }: ComputerControlTerminalProps) {
  const [inputVal, setInputVal] = useState("");
  const [logs, setLogs] = useState<string[]>([
    "[12:00:00 AM] Bootloader v4.11 initialized safely on virtual machine.",
    "[12:00:01 AM] Mounting system execution sandboxes under container process...",
    "[12:00:01 AM] READY -- Type a command or select a pre-mapped automation script below."
  ]);
  const [running, setRunning] = useState(false);
  const shellBottomRef = useRef<HTMLDivElement>(null);

  const runningRef = useRef(running);
  useEffect(() => {
    runningRef.current = running;
  }, [running]);

  useEffect(() => {
    shellBottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  useEffect(() => {
    const handleMacroEvent = async (e: Event) => {
      const customEvent = e as CustomEvent<{ name: string; commands: string[] }>;
      const { name, commands } = customEvent.detail;
      if (!commands || commands.length === 0) return;

      if (runningRef.current) {
        setLogs((prev) => [
          ...prev,
          `\n🚨 [WARNING] Macro execution requested but device is busy with another process. Request dropped.`
        ]);
        return;
      }

      setRunning(true);
      const startTimestamp = new Date().toLocaleTimeString();
      setLogs((prev) => [
        ...prev,
        `\n⚡ [${startTimestamp}] INITIALIZING MACRO SEQUENCE: "${name}"`,
        `[${startTimestamp}] Queueing ${commands.length} terminal operations...`
      ]);

      for (let i = 0; i < commands.length; i++) {
        const cmd = commands[i];
        const stepTimestamp = new Date().toLocaleTimeString();
        setLogs((prev) => [
          ...prev,
          `\n$ ${cmd} [Macro Step ${i + 1}/${commands.length}]`,
          `[${stepTimestamp}] Dispatching subprocess query...`
        ]);

        // Small delay for micro-step realism
        await new Promise((resolve) => setTimeout(resolve, 850));

        try {
          const outputLines = await onExecuteCommand(cmd);
          setLogs((prev) => [...prev, ...outputLines]);
        } catch (err: any) {
          const errorTimestamp = new Date().toLocaleTimeString();
          setLogs((prev) => [
            ...prev,
            `[${errorTimestamp}] [SUBSTEP ERROR] Command sequence failed: ${err.message || "Execution exception."}`
          ]);
          setLogs((prev) => [
            ...prev,
            `\n🚨 [${new Date().toLocaleTimeString()}] MACRO SEQUENCE ABORTED due to step failure.`
          ]);
          setRunning(false);
          return;
        }
      }

      setLogs((prev) => [
        ...prev,
        `\n✅ [${new Date().toLocaleTimeString()}] MACRO SEQUENCE "${name}" EXECUTED SUCCESSFULLY.`
      ]);
      setRunning(false);
    };

    window.addEventListener("jarvis-trigger-macro", handleMacroEvent);
    return () => {
      window.removeEventListener("jarvis-trigger-macro", handleMacroEvent);
    };
  }, [onExecuteCommand]);

  // Pre-mapped script execution launcher
  const launchScript = async (scriptTitle: string, commandStr: string) => {
    if (running) return;
    setRunning(true);
    setInputVal("");
    
    // Add prompt user logs to list immediately
    const tempTimestamp = new Date().toLocaleTimeString();
    setLogs((prev) => [
      ...prev,
      `\n$ sh run_macro --task="${scriptTitle}"`,
      `[${tempTimestamp}] Dispatched terminal subprocess to Express daemon...`
    ]);

    try {
      const outputLines = await onExecuteCommand(commandStr);
      setLogs((prev) => [...prev, ...outputLines]);
    } catch (err: any) {
      setLogs((prev) => [
        ...prev,
        `[${new Date().toLocaleTimeString()}] [ERROR] Process crashed: ${err.message || "Unknown error"}`
      ]);
    } finally {
      setRunning(false);
    }
  };

  const handleFormSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputVal.trim() || running) return;
    const currentMsg = inputVal;
    setInputVal("");
    setRunning(true);

    const tempTimestamp = new Date().toLocaleTimeString();
    setLogs((prev) => [...prev, `\n$ ${currentMsg}`]);

    try {
      const outputLines = await onExecuteCommand(currentMsg);
      setLogs((prev) => [...prev, ...outputLines]);
    } catch (err: any) {
      setLogs((prev) => [
        ...prev,
        `[${tempTimestamp}] [CMD ERR] Failed to resolve system automation: ${err.message || 'Subprocess fault.'}`
      ]);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="bg-[#0b0f19] border border-[#1e293b] rounded-xl flex flex-col shadow-2xl overflow-hidden h-[450px]">
      {/* Terminal Title Bar */}
      <div className="bg-[#0f172a] px-4 py-2 border-b border-[#1e293b] flex items-center justify-between pointer-events-none">
        <div className="flex items-center gap-2">
          <Terminal className="w-4 h-4 text-emerald-400" />
          <span className="text-xs font-semibold font-mono tracking-wider text-slate-300">
            SYSTEM_EXECUTION_TERMINAL_V4
          </span>
        </div>
        <div className="flex items-center gap-1.5">
          <div className="w-2 h-2 rounded-full bg-red-500/80" />
          <div className="w-2 h-2 rounded-full bg-amber-500/80" />
          <div className="w-2 h-2 rounded-full bg-emerald-500/80" />
        </div>
      </div>

      {/* Terminal Monospace Stream Scroll Panel */}
      <div className="flex-1 p-4 overflow-y-auto font-mono text-[11px] text-[#34d399] leading-relaxed space-y-1 select-all bg-[#080b11]">
        {logs.map((log, index) => {
          let customClass = "text-slate-300";
          if (log.startsWith("$ ")) customClass = "text-[#38bdf8] font-bold text-xs mt-2 block";
          else if (log.includes("[ERROR]") || log.includes("[CMD ERR]") || log.includes("WARNING:")) {
            customClass = "text-red-400 font-medium bg-red-950/20 px-1 rounded border border-red-900/30";
          } else if (log.includes("COMPLETE") || log.includes("successfully") || log.includes("NOMINAL") || log.includes("success")) {
            customClass = "text-emerald-400 font-semibold";
          } else if (log.includes("⚡") || log.includes("⚡ Jarvis")) {
            customClass = "text-yellow-400 font-medium";
          }

          return (
            <div key={index} className={`${customClass} whitespace-pre-wrap break-all`}>
              {log}
            </div>
          );
        })}
        {running && (
          <div className="text-emerald-400/75 flex items-center gap-2 animate-pulse mt-1">
            <RefreshCw className="w-3 h-3 animate-spin text-emerald-400" />
            <span>Executing background automation package...</span>
          </div>
        )}
        <div ref={shellBottomRef} />
      </div>

      {/* Quick macro action hot-deck */}
      <div className="p-3 bg-[#0f172a]/50 border-t border-[#1e293b] grid grid-cols-2 md:grid-cols-4 gap-2">
        <button
          onClick={() => launchScript("CPULatencyOptimizer", "Optimize system memory footprint")}
          disabled={running}
          className="flex items-center justify-center gap-1.5 px-2.5 py-1.5 rounded text-[10px] uppercase font-semibold font-mono tracking-wider border border-[#334155]/40 bg-[#1e293b]/40 hover:bg-sky-950/20 hover:text-sky-400 hover:border-sky-900/50 text-slate-300 transition-all disabled:opacity-45 disabled:pointer-events-none"
        >
          <Layers className="w-3 h-3" />
          <span>Optimize Core</span>
        </button>

        <button
          onClick={() => launchScript("VirtualPyAutoGUIClicker", "Simulating clicking coordinate x520 y710")}
          disabled={running}
          className="flex items-center justify-center gap-1.5 px-2.5 py-1.5 rounded text-[10px] uppercase font-semibold font-mono tracking-wider border border-[#334155]/40 bg-[#1e293b]/40 hover:bg-amber-950/20 hover:text-amber-400 hover:border-amber-900/50 text-slate-300 transition-all disabled:opacity-45 disabled:pointer-events-none"
        >
          <Shield className="w-3 h-3" />
          <span>Virtual Click</span>
        </button>

        <button
          onClick={() => launchScript("InstallBlueprintsZip", "Compile blueprint installation zip for antigravity")}
          disabled={running}
          className="flex items-center justify-center gap-1.5 px-2.5 py-1.5 rounded text-[10px] uppercase font-semibold font-mono tracking-wider border border-[#334155]/40 bg-[#1e293b]/40 hover:bg-emerald-950/20 hover:text-emerald-400 hover:border-emerald-900/50 text-slate-300 transition-all disabled:opacity-45 disabled:pointer-events-none"
        >
          <Layers className="w-3 h-3" />
          <span>Zip Install</span>
        </button>

        <button
          onClick={() => launchScript("RebootNeuralBrain", "Reboot neural brain routing matrix")}
          disabled={running}
          className="flex items-center justify-center gap-1.5 px-2.5 py-1.5 rounded text-[10px] uppercase font-semibold font-mono tracking-wider border border-[#334155]/40 bg-[#1e293b]/40 hover:bg-red-950/20 hover:text-red-400 hover:border-red-900/50 text-slate-300 transition-all disabled:opacity-45 disabled:pointer-events-none"
        >
          <RefreshCw className="w-3 h-3" />
          <span>Reboot Brain</span>
        </button>
      </div>

      {/* Manual Terminal Input Form */}
      <form onSubmit={handleFormSubmit} className="bg-[#0b0f19] border-t border-[#1e293b] flex items-center p-2.5">
        <span className="text-[#38bdf8] font-mono text-xs pl-2 pr-1.5 select-none font-bold">$</span>
        <input
          type="text"
          value={inputVal}
          onChange={(e) => setInputVal(e.target.value)}
          disabled={running}
          placeholder={running ? "Device locked. Running subprocess..." : "Type custom CLI automation (e.g., 'optimize system' or 'reboot brain')..."}
          className="flex-1 bg-transparent border-0 outline-none focus:outline-none focus:ring-0 text-xs font-mono text-[#34d399] placeholder-slate-500 disabled:opacity-50"
        />
        <button
          type="submit"
          disabled={running || !inputVal.trim()}
          className="bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/20 hover:border-emerald-500/40 text-emerald-400 p-1.5 rounded transition-all disabled:opacity-40 disabled:pointer-events-none size-7 flex items-center justify-center"
        >
          <Play className="w-3.5 h-3.5 fill-emerald-400/25" />
        </button>
      </form>
    </div>
  );
}
