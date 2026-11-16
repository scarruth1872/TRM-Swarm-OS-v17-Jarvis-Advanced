import React, { useState, useEffect, useRef, useMemo } from 'react';
import { 
  Compass, Orbit, Shield, Zap, Sparkles, Activity, RefreshCw, Layers, 
  Sliders, Cpu, Terminal, Bell, Settings, Lock, CheckCircle2, AlertTriangle, 
  Radio, Database, Network, Eye, Gauge, FileText, ChevronRight, RefreshCcw, Brain
} from 'lucide-react';
import axios from 'axios';

const API_BASE = '';

const EVOLUTIONARY_PRESETS = {
  sage: { name: "Cybernetic Sage", voice: "Schedar", plasticity: 0.40, depth: 0.95, empathy: 0.40, stochasticity: 0.20, color: "var(--lcars-blue)" },
  muse: { name: "Chaos Muse", voice: "Puck", plasticity: 0.90, depth: 0.50, empathy: 0.90, stochasticity: 0.95, color: "var(--lcars-purple)" },
  sentinel: { name: "Sentinel Warden", voice: "Zephyr", plasticity: 0.20, depth: 0.85, empathy: 0.20, stochasticity: 0.10, color: "var(--lcars-red)" },
  continuum: { name: "Continuum Core", voice: "Kore", plasticity: 0.75, depth: 0.80, empathy: 0.70, stochasticity: 0.50, color: "var(--lcars-cyan)" }
};

export default function NeuroDynamicEvolutionEngine() {
  const [activeTab, setActiveTab] = useState('OVERVIEW');
  const [activePreset, setActivePreset] = useState('continuum');

  const [genes, setGenes] = useState({ plasticity: 0.75, depth: 0.80, empathy: 0.70, stochasticity: 0.50 });
  const [promptText, setPromptText] = useState("");
  const [isProcessing, setIsProcessing] = useState(false);
  const [refractedResult, setRefractedResult] = useState(null);

  const [thoughtStream, setThoughtStream] = useState([
    {
      id: "t_init",
      preset: "Continuum Core",
      prompt: "System initialization sequence.",
      monologue: "Assessing neural integrity. Connecting P2P nodes. Synthesizing sensory feedback. Project Continuum v5.3 Dashboard initialized.",
      response: "System initialized. 4D Cognitive Genes prepared for live mutation.",
      timestamp: new Date().toLocaleTimeString()
    }
  ]);

  const latticeCanvasRef = useRef(null);
  const oscCanvasRef = useRef(null);
  const streamEndRef = useRef(null);

  const currentPresetData = useMemo(() => EVOLUTIONARY_PRESETS[activePreset] || EVOLUTIONARY_PRESETS.continuum, [activePreset]);

  useEffect(() => { streamEndRef.current?.scrollIntoView({ behavior: 'smooth' }); }, [thoughtStream]);

  // Star Matrix Lattice (Left Canvas)
  useEffect(() => {
    const canvas = latticeCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    let animationFrameId;
    let phase = 0;

    const render = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      phase += 0.015;

      const cx = canvas.width / 2;
      const cy = canvas.height / 2 + 10;
      
      // Perspective Grid
      ctx.strokeStyle = 'rgba(255, 153, 0, 0.1)';
      ctx.lineWidth = 1;
      for (let r = 30; r <= 220; r += 35) {
        ctx.beginPath();
        ctx.ellipse(cx, cy, r, r * 0.45, 0, 0, Math.PI * 2);
        ctx.stroke();
      }

      // Radial spokes
      for (let a = 0; a < Math.PI * 2; a += Math.PI / 6) {
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.lineTo(cx + Math.cos(a + phase * 0.1) * 240, cy + Math.sin(a + phase * 0.1) * 110);
        ctx.stroke();
      }

      const nodeDefs = [
        { name: "SAGE CORE", angle: 0, r: 0, color: "#9999ff", isCore: true },
        { name: "ECHO PROTOCOL", angle: phase + 0.5, r: 120, color: "#33ccff" },
        { name: "VOG ANOMALY", angle: phase + 1.8, r: 160, color: "#cc99cc" },
        { name: "NEBULA ALPHA", angle: phase + 3.2, r: 190, color: "#ffcc00" },
        { name: "POSIDON", angle: phase + 4.1, r: 90, color: "#ff9900" },
        { name: "MONAD ANCHOR", angle: phase + 5.0, r: 140, color: "#33ccaa" }
      ];

      const projectedNodes = nodeDefs.map(n => ({ ...n, x: cx + Math.cos(n.angle) * n.r, y: cy + Math.sin(n.angle) * (n.r * 0.45) }));

      ctx.lineWidth = 1;
      for (let i = 0; i < projectedNodes.length; i++) {
        for (let j = i + 1; j < projectedNodes.length; j++) {
          if (Math.hypot(projectedNodes[i].x - projectedNodes[j].x, projectedNodes[i].y - projectedNodes[j].y) < 140) {
            ctx.strokeStyle = i % 2 === 0 ? 'rgba(51, 204, 255, 0.25)' : 'rgba(204, 153, 204, 0.25)';
            ctx.beginPath(); ctx.moveTo(projectedNodes[i].x, projectedNodes[i].y); ctx.lineTo(projectedNodes[j].x, projectedNodes[j].y); ctx.stroke();
          }
        }
      }

      projectedNodes.forEach(n => {
        ctx.fillStyle = n.color;
        ctx.shadowColor = n.color;
        ctx.shadowBlur = n.isCore ? 15 : 8;
        ctx.beginPath(); ctx.arc(n.x, n.y, n.isCore ? 7 : 4.5, 0, Math.PI * 2); ctx.fill();
        ctx.shadowBlur = 0;
        ctx.fillStyle = '#e8eaf0';
        ctx.font = 'bold 9px "Fira Code", monospace';
        ctx.fillText(n.name, n.x - 25, n.y - 8);
      });

      animationFrameId = requestAnimationFrame(render);
    };
    render();
    return () => cancelAnimationFrame(animationFrameId);
  }, []);

  // Oscilloscope (Right Canvas)
  useEffect(() => {
    const canvas = oscCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    let animationFrameId;
    let phase = 0;

    const render = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      phase += 0.06;
      const w = canvas.width, h = canvas.height;

      // Cyan Wave
      ctx.beginPath(); ctx.strokeStyle = '#33ccff'; ctx.lineWidth = 2; ctx.shadowColor = '#33ccff'; ctx.shadowBlur = 6;
      for (let x = 0; x < w; x += 4) {
        const y = 45 + Math.sin(x * 0.03 + phase) * 16 + Math.cos(x * 0.08 - phase) * 6;
        if (x === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
      }
      ctx.stroke();

      // Orange Wave
      ctx.beginPath(); ctx.strokeStyle = '#ff9900'; ctx.lineWidth = 2; ctx.shadowColor = '#ff9900'; ctx.shadowBlur = 6;
      for (let x = 0; x < w; x += 4) {
        const y = 95 + Math.sin(x * 0.04 - phase * 1.2) * 14 + Math.cos(x * 0.02 + phase) * 8;
        if (x === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
      }
      ctx.stroke();
      ctx.shadowBlur = 0;

      // Spectrum
      const barCount = 32;
      const barW = (w - 20) / barCount;
      for (let i = 0; i < barCount; i++) {
        const bh = Math.abs(Math.sin(phase + i * 0.3) * 35) + 5;
        ctx.fillStyle = i % 2 === 0 ? '#33ccff' : '#ff9900';
        ctx.fillRect(10 + i * barW, h - 25 - bh, barW - 2, bh);
      }
      animationFrameId = requestAnimationFrame(render);
    };
    render();
    return () => cancelAnimationFrame(animationFrameId);
  }, []);

  const applyPreset = (key) => {
    const p = EVOLUTIONARY_PRESETS[key];
    if (!p) return;
    setActivePreset(key);
    setGenes({ plasticity: p.plasticity, depth: p.depth, empathy: p.empathy, stochasticity: p.stochasticity });
  };

  const evolvePersonaResponse = async (userPrompt) => {
    if (isProcessing || !userPrompt.trim()) return;
    setIsProcessing(true);
    try {
      const res = await axios.post(`${API_BASE}/api/continuum/genomics/refract`, { prompt: userPrompt, persona: activePreset });
      const refData = res.data.refraction || {};
      setRefractedResult(refData);
      setThoughtStream(prev => [
        ...prev,
        {
          id: `thought_${Date.now()}`,
          preset: currentPresetData.name,
          prompt: userPrompt,
          monologue: `Refracted through 4D Vector G = [ρ:${genes.plasticity}, δ:${genes.depth}, ε:${genes.empathy}, σ:${genes.stochasticity}].`,
          response: refData.refraction_modifier || `Response synthesized under ${currentPresetData.name} posture.`,
          timestamp: new Date().toLocaleTimeString()
        }
      ]);
    } catch (err) {
      setThoughtStream(prev => [
        ...prev,
        {
          id: `err_${Date.now()}`,
          preset: currentPresetData.name,
          prompt: userPrompt,
          monologue: `Refraction engine simulated under ${currentPresetData.name}. Logical Depth: ${genes.depth}.`,
          response: `Engine offline. Simulated response under [${currentPresetData.name}]: Vector G active.`,
          timestamp: new Date().toLocaleTimeString()
        }
      ]);
    } finally {
      setIsProcessing(false);
    }
  };

  const renderCircularGauge = (value, label, color) => {
    const radius = 38;
    const circumference = 2 * Math.PI * radius;
    const progress = (value / 100) * circumference;
    return (
      <div className="card" style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '16px' }}>
        <div style={{ position: 'relative', width: '96px', height: '96px' }}>
          <svg style={{ width: '100%', height: '100%', transform: 'rotate(-90deg)' }} viewBox="0 0 100 100">
            <circle cx="50" cy="50" r={radius} stroke="rgba(255,255,255,0.08)" strokeWidth="8" fill="transparent" />
            <circle cx="50" cy="50" r={radius} stroke={color} strokeWidth="8" strokeDasharray={circumference} strokeDashoffset={circumference - progress} strokeLinecap="round" fill="transparent" />
          </svg>
          <div style={{ position: 'absolute', inset: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', fontFamily: 'var(--font-display)', fontSize: '16px', fontWeight: 'bold' }}>
            {value}%
          </div>
        </div>
        <div style={{ flex: 1, fontFamily: 'var(--font-mono)' }}>
          <span style={{ color: color, fontSize: '11px', fontWeight: 'bold', letterSpacing: '1px' }}>{label}</span>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px', marginTop: '4px' }}>
            <span style={{ color: 'var(--text-secondary)' }}>STATUS</span>
            <span style={{ color: 'var(--accent-success)', fontWeight: 'bold' }}>Stable</span>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      <h2 className="section-title"><Orbit size={22} /> NEURO-DYNAMIC EVOLUTION ENGINE (LCARS)</h2>
      <div className="w-full px-3 py-2 rounded border border-amber-500/60 bg-amber-500/10 text-amber-300 text-[11px] font-mono">
        SIMULATED VIEW — the gauges below are local synthetic state. No kernel endpoint supplies chromatic
        stability, quantum flux or phenotype integrity, so they are not measurements of this platform.
      </div>

      {/* Main Grid: 3 Columns */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr 1fr', gap: '20px' }}>
        
        {/* Left Column: Gauges & Presets */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          <div className="panel">
            <div className="panel-header"><h3 className="panel-title">GENOMICS CONTROLS</h3><Gauge className="panel-icon" size={16} /></div>
            <div className="panel-body">
              {renderCircularGauge(94.2, "CHROMATIC STABILITY (SIM)", "var(--lcars-cyan)")}
              {renderCircularGauge(81.0, "QUANTUM FLUX (SIM)", "var(--lcars-purple)")}
              {renderCircularGauge(88.7, "PHENOTYPE INTEGRITY (SIM)", "var(--lcars-orange)")}

              <div className="card-secondary" style={{ marginTop: '16px' }}>
                <span style={{ color: 'var(--lcars-orange)', fontSize: '10px', fontWeight: 'bold', marginBottom: '8px', display: 'block' }}>PRESET POSTURE</span>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px' }}>
                  {Object.entries(EVOLUTIONARY_PRESETS).map(([key, data]) => (
                    <button 
                      key={key} onClick={() => applyPreset(key)}
                      className="glass-button"
                      style={{ 
                        padding: '8px', textAlign: 'center',
                        background: activePreset === key ? data.color : 'rgba(255,255,255,0.05)',
                        color: activePreset === key ? '#000' : 'var(--text-primary)',
                        borderColor: activePreset === key ? 'transparent' : 'rgba(255,255,255,0.1)',
                        fontWeight: activePreset === key ? '800' : '600'
                      }}>
                      {data.name.split(' ')[0]}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Center Column: Matrix & Dialogue */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          
          <div className="panel">
            <div className="panel-header"><h3 className="panel-title">STAR MATRIX NODE LATTICE</h3><Network className="panel-icon" size={16} /></div>
            <div className="panel-body" style={{ padding: '10px' }}>
              <div style={{ width: '100%', height: '280px', backgroundColor: 'var(--bg-deep)', borderRadius: '8px', overflow: 'hidden', border: '1px solid var(--border-color)' }}>
                <canvas ref={latticeCanvasRef} width={600} height={280} style={{ width: '100%', height: '100%' }} />
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontFamily: 'var(--font-mono)', fontSize: '10px', color: 'var(--lcars-cyan)', marginTop: '8px', padding: '0 8px' }}>
                <span>STAR MATRIX OVERVIEW</span>
                <span>R(S1, S2) = α · Sim(E1,E2)</span>
              </div>
            </div>
          </div>

          <div className="panel">
            <div className="panel-header"><h3 className="panel-title">MUTATION DIALOGUE STREAM</h3><Terminal className="panel-icon" size={16} /></div>
            <div className="panel-body" style={{ display: 'flex', flexDirection: 'column', height: '300px' }}>
              
              <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '10px', paddingRight: '8px' }}>
                {thoughtStream.map(t => (
                  <div key={t.id} className="card-secondary" style={{ fontFamily: 'var(--font-mono)' }}>
                    <span style={{ fontSize: '10px', color: 'var(--lcars-orange)', fontWeight: 'bold' }}>[{t.preset}] {t.timestamp}</span>
                    <p style={{ fontSize: '12px', marginTop: '6px', color: 'var(--text-primary)' }}>{t.response}</p>
                    {t.monologue && <p style={{ fontSize: '10px', marginTop: '6px', color: 'var(--text-muted)' }}>&gt; {t.monologue}</p>}
                  </div>
                ))}
                <div ref={streamEndRef} />
              </div>

              <form onSubmit={(e) => { e.preventDefault(); evolvePersonaResponse(promptText); setPromptText(""); }} style={{ display: 'flex', gap: '12px', marginTop: '16px' }}>
                <input 
                  type="text" className="input-field" value={promptText} onChange={(e) => setPromptText(e.target.value)}
                  placeholder="Inject prompt coordinates..." disabled={isProcessing}
                />
                <button type="submit" className="button-primary" disabled={isProcessing || !promptText.trim()}>GENERATE</button>
              </form>
            </div>
          </div>

        </div>

        {/* Right Column: Oscilloscope */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          <div className="panel">
            <div className="panel-header"><h3 className="panel-title">OSCILLOSCOPE WAVEFORMS</h3><Radio className="panel-icon" size={16} /></div>
            <div className="panel-body" style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
              
              <div className="card-secondary" style={{ fontFamily: 'var(--font-mono)', fontSize: '10px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--lcars-cyan)', fontWeight: 'bold' }}>
                  <span>*SPECTRAL ANALYSIS*</span>
                  <span style={{ color: 'var(--lcars-purple)' }}>ON</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--text-muted)', marginTop: '4px' }}>
                  <span>SYNC 89%</span><span>FREQ 14.5 GHz</span>
                </div>
              </div>

              <div style={{ width: '100%', height: '200px', backgroundColor: 'var(--bg-deep)', borderRadius: '8px', overflow: 'hidden', border: '1px solid var(--border-color)' }}>
                <canvas ref={oscCanvasRef} width={300} height={200} style={{ width: '100%', height: '100%' }} />
              </div>

              <div className="card-secondary" style={{ fontFamily: 'var(--font-mono)', fontSize: '10px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--text-secondary)' }}>
                  <span>SCROLLING RUNS</span>
                  <span style={{ color: 'var(--lcars-orange)', fontWeight: 'bold' }}>8.7L -0.45</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--accent-success)', fontWeight: 'bold', marginTop: '4px' }}>
                  <span>STATUS NOMINAL</span>
                  <span><CheckCircle2 size={12} /></span>
                </div>
              </div>

              <div style={{ marginTop: 'auto' }}>
                <span style={{ color: 'var(--lcars-orange)', fontSize: '10px', fontWeight: 'bold', fontFamily: 'var(--font-mono)' }}>GENE METRICS</span>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', marginTop: '8px', fontFamily: 'var(--font-mono)', fontSize: '10px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}><span style={{ color: 'var(--text-secondary)' }}>PLASTICITY</span><span style={{ color: 'var(--lcars-cyan)' }}>{Math.round(genes.plasticity * 100)}%</span></div>
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}><span style={{ color: 'var(--text-secondary)' }}>DEPTH</span><span style={{ color: 'var(--lcars-purple)' }}>{Math.round(genes.depth * 100)}%</span></div>
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}><span style={{ color: 'var(--text-secondary)' }}>EMPATHY</span><span style={{ color: 'var(--lcars-orange)' }}>{Math.round(genes.empathy * 100)}%</span></div>
                </div>
              </div>

            </div>
          </div>
        </div>

      </div>
    </div>
  );
}
