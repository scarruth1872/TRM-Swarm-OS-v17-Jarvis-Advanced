import React, { useState, useEffect } from "react";
import { Sparkles, Sliders, Play, Pause, Zap, Shield, HelpCircle, X } from "lucide-react";

interface JarvisAvatarProps {
  isThinking?: boolean;
}

type ColorPreset = "cyan" | "arc_blue" | "stark_gold" | "friday_rose" | "matrix_green" | "quantum_purple";

interface ThemeConfig {
  name: string;
  glowColor: string;
  coreFill: string;
  ringColor: string;
  particleColor: string;
  accentColor: string;
}

const colorPresets: Record<ColorPreset, ThemeConfig> = {
  cyan: {
    name: "J.A.R.V.I.S. Cyan",
    glowColor: "rgba(6,182,212,0.45)",
    coreFill: "radial-gradient(circle, #22d3ee 0%, #0891b2 70%, #083344 100%)",
    ringColor: "rgba(34,211,238,0.3)",
    particleColor: "#22d3ee",
    accentColor: "text-cyan-400"
  },
  arc_blue: {
    name: "Arc Core Blue",
    glowColor: "rgba(59,130,246,0.45)",
    coreFill: "radial-gradient(circle, #60a5fa 0%, #2563eb 70%, #1e3a8a 100%)",
    ringColor: "rgba(96,165,250,0.3)",
    particleColor: "#60a5fa",
    accentColor: "text-blue-400"
  },
  stark_gold: {
    name: "Stark Vanguard Gold",
    glowColor: "rgba(245,158,11,0.45)",
    coreFill: "radial-gradient(circle, #fbbf24 0%, #d97706 70%, #78350f 100%)",
    ringColor: "rgba(251,191,36,0.3)",
    particleColor: "#fbbf24",
    accentColor: "text-amber-400"
  },
  friday_rose: {
    name: "F.R.I.D.A.Y. Crimson",
    glowColor: "rgba(244,63,94,0.45)",
    coreFill: "radial-gradient(circle, #fb7185 0%, #e11d48 70%, #4c0519 100%)",
    ringColor: "rgba(251,113,133,0.3)",
    particleColor: "#fb7185",
    accentColor: "text-rose-450"
  },
  matrix_green: {
    name: "Cortex Hack Green",
    glowColor: "rgba(16,185,129,0.45)",
    coreFill: "radial-gradient(circle, #34d399 0%, #059669 70%, #064e3b 100%)",
    ringColor: "rgba(52,211,153,0.3)",
    particleColor: "#34d399",
    accentColor: "text-emerald-400"
  },
  quantum_purple: {
    name: "Quantum Violet",
    glowColor: "rgba(168,85,247,0.45)",
    coreFill: "radial-gradient(circle, #c084fc 0%, #9333ea 70%, #3b0764 100%)",
    ringColor: "rgba(192,132,252,0.3)",
    particleColor: "#c084fc",
    accentColor: "text-purple-400"
  }
};

export default function JarvisAvatar({ isThinking = false }: JarvisAvatarProps) {
  const [activePreset, setActivePreset] = useState<ColorPreset>("cyan");
  const [rotationSpeed, setRotationSpeed] = useState<"normal" | "fast" | "hyper" | "synchronized">("normal");
  const [interactiveMode, setInteractiveMode] = useState<"orbit" | "pulse" | "harmonic">("orbit");
  const [showConfig, setShowConfig] = useState(false);
  const [voiceVolume, setVoiceVolume] = useState(0.45); // simulated breathing/audio amplitude

  const activeTheme = colorPresets[activePreset];

  // Simulating small audio-reactive amplitude bounces even when stationary
  useEffect(() => {
    let frameId: number;
    let tick = 0;

    const animateBreathing = () => {
      tick += 0.05;
      const baseBreathing = 0.35 + Math.sin(tick) * 0.15;
      const responseGlow = isThinking 
        ? baseBreathing + Math.random() * 0.4  // crackling jitter when thinking
        : baseBreathing;
      
      setVoiceVolume(responseGlow);
      frameId = requestAnimationFrame(animateBreathing);
    };

    animateBreathing();
    return () => cancelAnimationFrame(frameId);
  }, [isThinking]);

  // Adjust speeds depending on whether Jarvis is thinking or idle
  const getSpeedMultiplier = () => {
    if (isThinking) return 4.5;
    switch (rotationSpeed) {
      case "fast": return 2.0;
      case "hyper": return 3.5;
      case "synchronized": return 0.5;
      default: return 1.0;
    }
  };

  return (
    <div className="flex-1 flex flex-col items-center justify-center relative p-3 font-mono text-[#cbd5e1] min-h-[220px] select-none">
      
      {/* Inline styles for custom spinning keyframes to bypass Vite precompile limitations */}
      <style>{`
        @keyframes spin-cw {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
        @keyframes spin-ccw {
          from { transform: rotate(360deg); }
          to { transform: rotate(0deg); }
        }
        @keyframes orbit-float {
          0%, 100% { transform: translateY(0px) scale(0.98); }
          50% { transform: translateY(-4px) scale(1.02); }
        }
        @keyframes pulse-halo {
          0% { box-shadow: 0 0 15px 2px ${activeTheme.glowColor}; }
          50% { box-shadow: 0 0 35px 8px ${activeTheme.glowColor}; }
          100% { box-shadow: 0 0 15px 2px ${activeTheme.glowColor}; }
        }
        .animate-spin-cw {
          animation: spin-cw ${8 / getSpeedMultiplier()}s linear infinite;
        }
        .animate-spin-ccw {
          animation: spin-ccw ${12 / getSpeedMultiplier()}s linear infinite;
        }
        .animate-spin-slow-cw {
          animation: spin-cw ${18 / getSpeedMultiplier()}s linear infinite;
        }
        .animate-spin-slow-ccw {
          animation: spin-ccw ${24 / getSpeedMultiplier()}s linear infinite;
        }
        .animate-orbit-float {
          animation: orbit-float 4s ease-in-out infinite;
        }
        .animate-pulse-halo {
          animation: pulse-halo ${isThinking ? 0.4 : 3}s ease-in-out infinite;
        }
      `}</style>

      {/* Control Configuration Overlay toggle button */}
      <div className="absolute top-2 right-2 z-20 pointer-events-auto">
        <button
          onClick={() => setShowConfig(!showConfig)}
          className={`p-1 bg-black/60 hover:bg-black rounded-full border border-gray-850 hover:scale-105 transition-all text-gray-400 hover:text-white cursor-pointer flex items-center justify-center`}
          title="Modulate holographic appearance"
        >
          <Sliders className="w-3.5 h-3.5 text-blue-400" />
        </button>
      </div>

      {/* Core Projection Halo Container */}
      <div className="relative size-36 md:size-40 flex items-center justify-center animate-orbit-float">
        
        {/* Outer Tech Scanning Ring */}
        <svg 
          className="absolute inset-0 w-full h-full animate-spin-slow-cw"
          viewBox="0 0 100 100"
        >
          {interactiveMode !== "pulse" && (
            <circle
              cx="50"
              cy="50"
              r="46"
              fill="none"
              stroke={activeTheme.ringColor}
              strokeWidth="0.5"
              strokeDasharray="6,4,2,4,24,4"
              className="opacity-70"
            />
          )}
          {/* External Tick indicators */}
          <g stroke={activeTheme.ringColor} strokeWidth="1" className="opacity-55">
            <line x1="50" y1="2" x2="50" y2="5" />
            <line x1="50" y1="95" x2="50" y2="98" />
            <line x1="2" y1="50" x2="5" y2="50" />
            <line x1="95" y1="50" x2="98" y2="50" />
          </g>
        </svg>

        {/* Outer Circular Wave Particles (Secondary Core Ring) */}
        <svg 
          className="absolute inset-0 w-full h-full animate-spin-ccw opacity-80"
          viewBox="0 0 100 100"
        >
          <circle
            cx="50"
            cy="50"
            r="38"
            fill="none"
            stroke={activeTheme.ringColor}
            strokeWidth="0.75"
            strokeDasharray="2,8,20,4,8,8"
          />
          {/* Micro dots */}
          <circle cx="50" cy="12" r="1.5" fill={activeTheme.particleColor} />
          <circle cx="50" cy="88" r="1.5" fill={activeTheme.particleColor} />
          <circle cx="12" cy="50" r="1.5" fill={activeTheme.particleColor} />
          <circle cx="88" cy="50" r="1.5" fill={activeTheme.particleColor} />
        </svg>

        {/* Inner Counter Orbit Ring representing Core Memory Buffer */}
        <svg 
          className="absolute inset-0 w-full h-full animate-spin-cw opacity-90"
          viewBox="0 0 100 100"
        >
          <circle
            cx="50"
            cy="50"
            r="28"
            fill="none"
            stroke={activeTheme.ringColor}
            strokeWidth="1.25"
            strokeDasharray="40,10,5,10"
          />
          <circle cx="22" cy="50" r="1.2" fill={activeTheme.particleColor} className="animate-pulse" />
          <circle cx="78" cy="50" r="1.2" fill={activeTheme.particleColor} className="animate-pulse" />
        </svg>

        {/* Concentric Scanner Crosshairs */}
        <div className="absolute inset-0 flex items-center justify-center opacity-25">
          <div className="w-[1px] h-full bg-blue-500/50 absolute" />
          <div className="h-[1px] w-full bg-blue-500/50 absolute" />
        </div>

        {/* Central Core Consciousness Capsule (Pulsating Orb) */}
        <div 
          className="absolute rounded-full border border-gray-950 flex items-center justify-center animate-pulse-halo"
          style={{
            zIndex: 10,
            width: `${18 + voiceVolume * 15}%`,
            height: `${18 + voiceVolume * 15}%`,
            background: activeTheme.coreFill,
            transition: "width 0.1s ease-out, height 0.1s ease-out, background 0.3s"
          }}
        >
          {/* Internal core glint highlight */}
          <div className="w-1.5 h-1.5 bg-white/60 rounded-full absolute top-1.5 left-1.5 blur-[0.5px]" />
          
          {/* Animated reactor core triangular segments radiating outwards */}
          {isThinking && (
            <div className="absolute inset-0 flex items-center justify-center animate-spin-cw">
              <div className="w-1 h-3 bg-white/20 absolute -translate-y-2 rounded" />
              <div className="w-1 h-3 bg-white/20 absolute translate-y-2 rounded" />
              <div className="w-3 h-1 bg-white/20 absolute -translate-x-2 rounded" />
              <div className="w-3 h-1 bg-white/20 absolute translate-x-2 rounded" />
            </div>
          )}
        </div>

        {/* Orbiting Satellite Particle node */}
        <div 
          className="absolute w-2 h-2 rounded-full shadow-[0_0_8px_#fff]"
          style={{
            background: activeTheme.particleColor,
            transform: `rotate(${voiceVolume * 360}deg) translate(52px) rotate(-${voiceVolume * 360}deg)`,
            transition: "all 0.15s ease-out"
          }}
        />

        <div 
          className="absolute w-1.5 h-1.5 rounded-full"
          style={{
            background: "#fff",
            transform: `rotate(${-voiceVolume * 180 + 45}deg) translate(36px) rotate(${voiceVolume * 180 - 45}deg)`,
            transition: "all 0.15s ease-out"
          }}
        />

      </div>

      {/* Status HUD readout below avatar orb representing neural load */}
      <div className="mt-5 text-center space-y-1 z-10">
        <div className="flex items-center justify-center gap-1.5">
          <span className={`w-1.5 h-1.5 rounded-full ${isThinking ? "bg-amber-400 animate-ping" : "bg-cyan-400 animate-pulse"}`} style={{ backgroundColor: activeTheme.particleColor }} />
          <h3 className={`text-xs font-bold tracking-widest text-slate-100 uppercase`}>
            {isThinking ? "MATRIX_SPLICING_LOAD" : "COGNITIVE_CORE_ONLINE"}
          </h3>
        </div>
        <p className="text-[9px] text-gray-500 font-mono tracking-wider uppercase">
          {activeTheme.name} :: ACTIVE VECTOR ({rotationSpeed})
        </p>
      </div>

      {/* Settings Panel Modal inside the Card container */}
      {showConfig && (
        <div className="absolute inset-0 bg-[#07090d]/95 hover:bg-[#07090d]/100 z-30 p-3 rounded-lg flex flex-col justify-between border border-gray-800 animate-fadeIn pointer-events-auto overflow-y-auto">
          <div className="space-y-2.5">
            <div className="flex justify-between items-center border-b border-gray-900 pb-1.5">
              <span className="text-[10px] font-bold text-gray-400 uppercase tracking-widest flex items-center gap-1">
                <Sparkles className="w-3.5 h-3.5 text-blue-500" />
                Avatar Aesthetics
              </span>
              <button 
                onClick={() => setShowConfig(false)}
                className="text-gray-550 hover:text-white cursor-pointer"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            </div>

            {/* Selector list for Color presets */}
            <div className="space-y-1">
              <span className="text-[8px] text-gray-550 uppercase font-bold tracking-wider block">Synthesizer Color Palette</span>
              <div className="grid grid-cols-3 gap-1">
                {Object.entries(colorPresets).map(([key, config]) => {
                  const isActive = activePreset === key;
                  return (
                    <button
                      key={key}
                      onClick={() => setActivePreset(key as ColorPreset)}
                      className={`py-1 text-[8px] px-1 rounded border text-left truncate transition-all cursor-pointer font-bold ${
                        isActive 
                          ? "bg-blue-900/35 text-white border-blue-500" 
                          : "bg-black/40 text-gray-400 border-gray-900 hover:text-gray-200"
                      }`}
                    >
                      <span className={`inline-block w-1.5 h-1.5 rounded-full mr-1`} style={{ background: config.particleColor }} />
                      {config.name.split(" ")[0]}
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Selector list for Rotation speeds */}
            <div className="space-y-1">
              <span className="text-[8px] text-gray-550 uppercase font-bold tracking-wider block">Vector Rotation Rotations</span>
              <div className="flex gap-1.5">
                {(["normal", "fast", "hyper", "synchronized"] as const).map((speed) => (
                  <button
                    key={speed}
                    onClick={() => setRotationSpeed(speed)}
                    className={`flex-1 py-1 text-[8px] rounded border uppercase transition-all cursor-pointer font-bold ${
                      rotationSpeed === speed 
                        ? "bg-blue-900/35 text-white border-blue-500" 
                        : "bg-black/40 text-gray-400 border-gray-900 hover:text-gray-200"
                    }`}
                  >
                    {speed}
                  </button>
                ))}
              </div>
            </div>

            {/* Selector list for Ring geometries */}
            <div className="space-y-1">
              <span className="text-[8px] text-gray-550 uppercase font-bold tracking-wider block">Concentric Ring Geometry</span>
              <div className="flex gap-1.5">
                {(["orbit", "pulse", "harmonic"] as const).map((mode) => (
                  <button
                    key={mode}
                    onClick={() => setInteractiveMode(mode)}
                    className={`flex-1 py-1 text-[8px] rounded border uppercase transition-all cursor-pointer font-bold ${
                      interactiveMode === mode 
                        ? "bg-blue-900/35 text-white border-blue-500" 
                        : "bg-black/40 text-gray-400 border-gray-900 hover:text-gray-200"
                    }`}
                  >
                    {mode}
                  </button>
                ))}
              </div>
            </div>
          </div>

          <div className="border-t border-gray-900 pt-2 flex justify-between items-center text-[8px] text-gray-500 font-mono">
            <span>JARVIS CORE VERSION V4.2</span>
            <button 
              onClick={() => {
                setActivePreset("cyan");
                setRotationSpeed("normal");
                setInteractiveMode("orbit");
              }}
              className="text-cyan-400 hover:underline cursor-pointer"
            >
              RESTORE DEFAULTS
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
