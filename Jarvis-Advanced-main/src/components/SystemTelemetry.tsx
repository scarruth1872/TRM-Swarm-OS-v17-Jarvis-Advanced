/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import React, { useEffect, useState } from "react";
import { Cpu, HardDrive, Wifi, Activity, AlertTriangle, RefreshCw } from "lucide-react";
import { SystemStatus } from "../types";

interface SystemTelemetryProps {
  stats: SystemStatus;
  loading: boolean;
  onRefresh: () => void;
  proactiveMuted: boolean;
  onToggleProactive: () => void;
}

export default function SystemTelemetry({
  stats,
  loading,
  onRefresh,
  proactiveMuted,
  onToggleProactive
}: SystemTelemetryProps) {
  const [pulse, setPulse] = useState(false);

  useEffect(() => {
    const interval = setInterval(() => {
      setPulse((p) => !p);
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="bg-[#0f172a] border border-[#1e293b] rounded-xl p-5 shadow-2xl relative overflow-hidden h-full flex flex-col justify-between">
      {/* Visual neon grid background filter */}
      <div className="absolute inset-0 bg-[radial-gradient(#1e293b_1px,transparent_1px)] [background-size:16px_16px] opacity-10 pointer-events-none" />

      <div>
        <div className="flex items-center justify-between border-b border-[#1e293b] pb-3 mb-4">
          <div className="flex items-center gap-2">
            <div className={`w-2.5 h-2.5 rounded-full ${stats.cpu > 80 ? 'bg-red-500' : 'bg-[#10b981] animate-pulse'}`} />
            <h2 className="text-sm font-semibold tracking-wider font-sans text-slate-300 uppercase">
              Proactive Cognitive Scan
            </h2>
          </div>
          <button
            onClick={onRefresh}
            className={`text-slate-400 hover:text-white transition-colors p-1.5 rounded bg-[#1e293b]/70 hover:bg-[#1e293b] ${loading ? 'animate-spin' : ''}`}
            title="Force Hardware Diagnostic"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Dynamic Telemetry Metric Pairs */}
        <div className="grid grid-cols-2 gap-4">
          {/* CPU Scan Panel */}
          <div className="bg-[#1e293b]/50 border border-[#334155]/45 rounded-lg p-3.5 relative">
            <div className="flex items-center justify-between text-xs text-slate-400 mb-1 pointer-events-none">
              <span>CPU PROCESSES</span>
              <Cpu className="w-3.5 h-3.5 text-[#38bdf8]" />
            </div>
            <div className="flex items-baseline gap-1 mt-1">
              <span className="text-2xl font-bold font-mono tracking-tight text-white">{stats.cpu}%</span>
              <span className="text-[10px] text-[#38bdf8] font-mono leading-none">V-CORES</span>
            </div>
            {/* Visual Mini Progress Gauge */}
            <div className="w-full bg-[#334155]/60 rounded-full h-1 mt-2.5 overflow-hidden">
              <div
                className={`h-full transition-all duration-700 ${stats.cpu > 75 ? 'bg-red-500' : 'bg-[#38bdf8]'}`}
                style={{ width: `${stats.cpu}%` }}
              />
            </div>
          </div>

          {/* Memory Storage Scan Panel */}
          <div className="bg-[#1e293b]/50 border border-[#334155]/45 rounded-lg p-3.5 relative">
            <div className="flex items-center justify-between text-xs text-slate-400 mb-1 pointer-events-none">
              <span>RAM COGNITION</span>
              <Activity className="w-3.5 h-3.5 text-[#a855f7]" />
            </div>
            <div className="flex items-baseline gap-1 mt-1">
              <span className="text-2xl font-bold font-mono tracking-tight text-white">{stats.ram}%</span>
              <span className="text-[10px] text-[#a855f7] font-mono leading-none">RESIDENT</span>
            </div>
            <div className="w-full bg-[#334155]/60 rounded-full h-1 mt-2.5 overflow-hidden">
              <div
                className="h-full bg-[#a855f7] transition-all duration-700"
                style={{ width: `${stats.ram}%` }}
              />
            </div>
          </div>
        </div>

        {/* Status indicator metrics */}
        <div className="mt-4 space-y-2.5">
          <div className="flex items-center justify-between text-xs bg-[#1e293b]/30 p-2 rounded border border-[#334155]/20">
            <div className="flex items-center gap-2 text-slate-400">
              <HardDrive className="w-3.5 h-3.5 text-[#cbd5e1]" />
              <span>Cognitive State Cache</span>
            </div>
            <span className="font-mono text-slate-300 font-medium text-xs">{stats.storage}</span>
          </div>

          <div className="flex items-center justify-between text-xs bg-[#1e293b]/30 p-2 rounded border border-[#334155]/20">
            <div className="flex items-center gap-2 text-slate-400">
              <Wifi className="w-3.5 h-3.5 text-[#10b981]" />
              <span>Dev Environment Tunnel</span>
            </div>
            <span className="font-mono text-[#10b981] font-medium text-xs">PORT 3000 :: LOCALHOST</span>
          </div>
        </div>
      </div>

      {/* Proactive Speech Alert Diagnostics Box */}
      <div className="mt-4 pt-4 border-t border-[#1e293b]">
        <div className="flex items-center justify-between mb-2">
          <span className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
            Proactive Warning Interceptor
          </span>
          <button
            onClick={onToggleProactive}
            className={`text-[10px] px-2 py-0.5 rounded transition-all font-mono border ${
              proactiveMuted
                ? "bg-red-950/20 text-red-400 border-red-900/50"
                : "bg-emerald-950/20 text-emerald-400 border-emerald-900/50"
            }`}
          >
            {proactiveMuted ? "INTERCEPTOR DEACTIVATED" : "INTERCEPTOR ARMED"}
          </button>
        </div>

        <div className="bg-[#1e293b]/40 rounded-lg p-3 border border-[#334155]/30">
          {stats.cpu > 50 && !proactiveMuted ? (
            <div className="flex items-start gap-2.5 text-xs text-amber-400 animate-pulse">
              <AlertTriangle className="w-4 h-4 shrink-0 mt-0.5" />
              <div>
                <p className="font-semibold">Thread Utilization Spiking ({stats.cpu}%)</p>
                <p className="text-[10px] text-slate-400 mt-0.5">
                  Jarvis is analyzing active compiler nodes. Voice alert triggered.
                </p>
              </div>
            </div>
          ) : (
            <div className="flex items-start gap-2.5 text-xs text-slate-400">
              <Activity className="w-4 h-4 shrink-0 mt-0.5 text-[#10b981]" />
              <div>
                <p className="font-semibold text-slate-300">All Core Subsystems Green</p>
                <p className="text-[10px] text-slate-500 mt-0.5">
                  Proactive vocal engine monitoring system performance thresholds.
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
