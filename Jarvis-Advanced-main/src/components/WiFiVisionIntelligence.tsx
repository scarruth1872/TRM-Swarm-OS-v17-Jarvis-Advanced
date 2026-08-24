import React, { useState, useEffect, useRef } from "react";
import { motion } from "motion/react";
import { 
  Wifi, 
  Activity, 
  Sparkles, 
  Heart, 
  RefreshCw, 
  ShieldAlert, 
  Database, 
  AlertCircle, 
  Eye, 
  Filter, 
  Atom, 
  Settings, 
  Compass, 
  Play, 
  Square, 
  Bell, 
  Sliders, 
  Brain, 
  Sparkle,
  Zap
} from "lucide-react";

interface WiFiVisionIntelligenceProps {
  speakText: (text: string) => void;
  fetchMemoryData?: () => void;
}

// 3D joint representation relative to body center
interface Vector3D {
  x: number;
  y: number;
  z: number;
}

interface SkeletonPreset {
  id: string;
  name: string;
  description: string;
  pulseBpm: number;
  respRate: number;
  motionVelocity: number;
  interferenceLevel: number;
  postureStatus: string;
  guardRisk: "LOW" | "NORMAL" | "HIGH" | "CRITICAL";
  animationStyle: "yoga" | "workout" | "idle" | "fall" | "noise";
}

const PRESETS: SkeletonPreset[] = [
  {
    id: "empty_room",
    name: "Empty Calibration Baseline",
    description: "Quiet room state. Static furniture reflections filtered out. No biological waveforms detected.",
    pulseBpm: 0,
    respRate: 0,
    motionVelocity: 0.02,
    interferenceLevel: 0.12,
    postureStatus: "Empty Space",
    guardRisk: "LOW",
    animationStyle: "idle"
  },
  {
    id: "yoga_breath",
    name: "Yoga Breathing Alignment",
    description: "Slow-moving human form. Stable cardiac rhythm, consistent deep breathing, high pose posture alignment.",
    pulseBpm: 64,
    respRate: 11,
    motionVelocity: 0.18,
    interferenceLevel: 0.15,
    postureStatus: "Limb Extension / Perfect Form (94%)",
    guardRisk: "LOW",
    animationStyle: "yoga"
  },
  {
    id: "workout_squat",
    name: "High Intensity Squat Cycles",
    description: "Deep cyclic vertical displacement. Elevated cardiac flow and breathing depth under Forge analysis.",
    pulseBpm: 138,
    respRate: 24,
    motionVelocity: 0.95,
    interferenceLevel: 0.28,
    postureStatus: "Cyclic Knee Flexion (Squat Routine - Rep 12)",
    guardRisk: "NORMAL",
    animationStyle: "workout"
  },
  {
    id: "emergency_fall",
    name: "Anomalous Fall & Immobilization",
    description: "Violent skeletal displacement followed by static flatline posture feedback. Vanguard alert priority.",
    pulseBpm: 48,
    respRate: 6,
    motionVelocity: 0.85,
    interferenceLevel: 0.35,
    postureStatus: "HORIZONTAL COLLAPSE / SECURING BIO-STABILITY",
    guardRisk: "CRITICAL",
    animationStyle: "fall"
  }
];

export const WiFiVisionIntelligence: React.FC<WiFiVisionIntelligenceProps> = ({ speakText, fetchMemoryData }) => {
  const [activePreset, setActivePreset] = useState<string>("yoga_breath");
  const [isCapturing, setIsCapturing] = useState<boolean>(true);
  
  // Interactive filters
  const [petFilter, setPetFilter] = useState<boolean>(true);
  const [fanFilter, setFanFilter] = useState<boolean>(true);
  const [multipathFilter, setMultipathFilter] = useState<boolean>(true);
  
  // Custom threshold configurations
  const [sensitivity, setSensitivity] = useState<number>(85);
  const [suppressionDb, setSuppressionDb] = useState<number>(38);
  const [resScale, setResScale] = useState<number>(75);

  // Synaptic Sync states
  const [syncingState, setSyncingState] = useState<"IDLE" | "SYNCING" | "COMPLETED" | "FAIL">("IDLE");
  const [syncNodeResult, setSyncNodeResult] = useState<string | null>(null);

  // 3D camera angles
  const [yaw, setYaw] = useState<number>(20);
  const [pitch, setPitch] = useState<number>(10);
  const [zoom, setZoom] = useState<number>(180);
  const [isDragging, setIsDragging] = useState<boolean>(false);
  const [autoRotate, setAutoRotate] = useState<boolean>(true);
  const dragStartPos = useRef({ x: 0, y: 0 });

  const skeletonCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const waveCanvasRef = useRef<HTMLCanvasElement | null>(null);

  const selectedPresetObj = PRESETS.find(p => p.id === activePreset) || PRESETS[1];

  const [ambientState, setAmbientState] = useState<{
    lighting_level: number;
    audio_volume: number;
    audio_mode: string;
  }>({
    lighting_level: 100,
    audio_volume: 50,
    audio_mode: "Normal"
  });

  // Fetch current ambient state periodically
  useEffect(() => {
    const fetchAmbient = async () => {
      try {
        const res = await fetch("/api/swarm/spatial/ambient");
        if (res.ok) {
          const data = await res.json();
          setAmbientState(data);
        }
      } catch (err) {
        console.error("Failed to fetch ambient feedback status", err);
      }
    };
    fetchAmbient();
    const interval = setInterval(fetchAmbient, 3000);
    return () => clearInterval(interval);
  }, []);

  // Update feed whenever activePreset changes
  useEffect(() => {
    let focusScore = 0.5;
    let presence = true;
    if (activePreset === "empty_room") {
      focusScore = 0.0;
      presence = false;
    } else if (activePreset === "yoga_breath") {
      focusScore = 0.85;
    } else if (activePreset === "workout_squat") {
      focusScore = 0.95;
    } else if (activePreset === "emergency_fall") {
      focusScore = 0.10;
    }

    const pushFeed = async () => {
      try {
        const res = await fetch("/api/swarm/spatial/feed", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            user_focus_score: focusScore,
            presence_detected: presence,
            raw_csi_amplitude: Math.random() * 50 + 100
          })
        });
        if (res.ok) {
          const data = await res.json();
          setAmbientState(data);
        }
      } catch (err) {
        console.error("Failed to push spatial feed", err);
      }
    };
    pushFeed();
  }, [activePreset]);

  // Sound and UI response triggers
  const triggerVoiceGreeting = () => {
    if (activePreset === "empty_room") {
      speakText("Ambient Wi-Fi vision has recalibrated. The studio airspace is clear and vacant.");
    } else if (activePreset === "yoga_breath") {
      speakText("Wi-Fi pulse detection reports slow, deep diaphragmatic flow at eleven breaths per minute. Form alignment is nominal.");
    } else if (activePreset === "workout_squat") {
      speakText("Forge is analyzing your active posture. Squat alignment is exceptional at ninety-four percent accuracy. Keep pushing.");
    } else if (activePreset === "emergency_fall") {
      speakText("Warning. Vanguard biometric safeties detected abrupt acceleration followed by absolute immobilization. Initiating emergency wellness protocols.");
    }
  };

  // Synchronise finding to J.A.R.V.I.S semantic graphs
  const handlePersistToSynapticGraph = async () => {
    setSyncingState("SYNCING");
    try {
      // Add a knowledge relationship
      const source = "Conscious WiFi Presence";
      const target = selectedPresetObj.name;
      const label = `Biometric status (${selectedPresetObj.pulseBpm} BPM, RR: ${selectedPresetObj.respRate})`;

      const res = await fetch("/api/memory/add-relation", {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ source, target, label })
      });

      if (res.ok) {
        // Also memorize event to diaries
        await fetch("/api/memory/memorize-event", {
          method: "POST",
          headers: {
            "Content-Type": "application/json"
          },
          body: JSON.stringify({
            description: `[Wi-Fi Vision Ingestion] Synced biological telemetry from spatial envelope: ${selectedPresetObj.name} (Risk Factor: ${selectedPresetObj.guardRisk}).`,
            importance: selectedPresetObj.guardRisk === "CRITICAL" ? "high" : "medium"
          })
        });

        if (fetchMemoryData) {
          fetchMemoryData();
        }
        setSyncingState("COMPLETED");
        setSyncNodeResult(`Successfully mapped core node: '${target}' aligned as protective biometric anchor.`);
        speakText(`Wi-Fi vision indices for ${selectedPresetObj.name} have been fully synthesized as memory graph parameters.`);
      } else {
        throw new Error("Handshake rejected.");
      }
    } catch (err) {
      setSyncingState("FAIL");
      console.error(err);
    }
  };

  // Dragging action on 3D viewport
  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    dragStartPos.current = { x: e.clientX, y: e.clientY };
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDragging) return;
    const deltaX = e.clientX - dragStartPos.current.x;
    const deltaY = e.clientY - dragStartPos.current.y;
    setYaw(prev => prev + deltaX * 0.5);
    setPitch(prev => Math.max(-60, Math.min(60, prev + deltaY * 0.5)));
    dragStartPos.current = { x: e.clientX, y: e.clientY };
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  // Render loop for 3D skeleton and real-time waves
  useEffect(() => {
    let animationId: number;
    let tickCount = 0;

    const render = () => {
      tickCount++;
      const canvasTextSelectionColors = {
        cyan: "#22d3ee",
        violet: "#a78bfa",
        grid: "#0f172a",
        border: "#1e293b",
        alert: "#f87171"
      };

      const activeYaw = autoRotate ? yaw + tickCount * 0.35 : yaw;

      // ------------------------------------
      // DRAW PART 1: 3D SKELETON ENVIRONMENT
      // ------------------------------------
      const canvas1 = skeletonCanvasRef.current;
      if (canvas1) {
        const ctx = canvas1.getContext("2d");
        if (ctx) {
          const w = canvas1.width;
          const h = canvas1.height;
          ctx.fillStyle = "#03070a";
          ctx.fillRect(0, 0, w, h);

          // Draw ground calibration grids
          ctx.strokeStyle = "#08101a";
          ctx.lineWidth = 1;
          for (let i = -4; i <= 4; i++) {
            // Draw floor mesh in 3D projection
            projectLine(ctx, { x: i * 0.5, y: -0.2, z: -2 }, { x: i * 0.5, y: -0.2, z: 2 }, w, h, activeYaw, pitch, zoom);
            projectLine(ctx, { x: -2, y: -0.2, z: i * 0.5 }, { x: 2, y: -0.2, z: i * 0.5 }, w, h, activeYaw, pitch, zoom);
          }

          // Draw grid wall in background for depth
          ctx.strokeStyle = "rgba(139, 92, 246, 0.04)";
          for (let i = -1; i <= 1; i += 0.5) {
            projectLine(ctx, { x: -2, y: i + 0.5, z: 2 }, { x: 2, y: i + 0.5, z: 2 }, w, h, activeYaw, pitch, zoom);
            projectLine(ctx, { x: i * 2, y: -0.2, z: 2 }, { x: i * 2, y: 1.5, z: 2 }, w, h, activeYaw, pitch, zoom);
          }

          // Draw Concentric Radar Scanner rings on the ground
          ctx.strokeStyle = "rgba(6, 182, 212, 0.08)";
          ctx.lineWidth = 1;
          for (let r = 0.5; r <= 2.5; r += 0.5) {
            ctx.beginPath();
            const points = [];
            for (let a = 0; a < 360; a += 10) {
              const rad = (a * Math.PI) / 180;
              const p = renderProject3D({ x: Math.cos(rad) * r, y: -0.2, z: Math.sin(rad) * r }, w, h, activeYaw, pitch, zoom);
              if (p) points.push(p);
            }
            if (points.length > 0) {
              ctx.moveTo(points[0].x, points[0].y);
              for (let pi = 1; pi < points.length; pi++) {
                ctx.lineTo(points[pi].x, points[pi].y);
              }
              ctx.closePath();
              ctx.stroke();
            }
          }

          // Draw sweeping radar line on floor
          if (isCapturing && selectedPresetObj.id !== "empty_room") {
            const sweepAngle = (tickCount * 0.02) % (2 * Math.PI);
            const sweepEnd = {
              x: Math.cos(sweepAngle) * 2,
              y: -0.2,
              z: Math.sin(sweepAngle) * 2
            };
            ctx.strokeStyle = selectedPresetObj.guardRisk === "CRITICAL" ? "rgba(248, 113, 113, 0.25)" : "rgba(3, 180, 220, 0.25)";
            ctx.lineWidth = 1.5;
            projectLine(ctx, { x: 0, y: -0.2, z: 0 }, sweepEnd, w, h, activeYaw, pitch, zoom);
          }

          // Generate animated joint coordinates based on preset style
          const joints: Record<string, Vector3D> = {};
          let angleTimer = tickCount * 0.03;

          // Head / Spine center movement
          let bodyY = 0.7;
          let bodyX = 0;
          let bodyZ = 0;

          if (selectedPresetObj.animationStyle === "yoga") {
            // Gentle side sweep & breathing lift
            bodyY = 0.5 + Math.sin(tickCount * 0.02) * 0.05;
            bodyX = Math.sin(tickCount * 0.015) * 0.15;
            joints["head"] = { x: bodyX, y: bodyY + 0.75, z: bodyZ };
            joints["neck"] = { x: bodyX, y: bodyY + 0.6, z: bodyZ };
            joints["shoulderL"] = { x: bodyX - 0.35, y: bodyY + 0.53 + Math.sin(tickCount * 0.02) * 0.04, z: bodyZ };
            joints["shoulderR"] = { x: bodyX + 0.35, y: bodyY + 0.53 - Math.sin(tickCount * 0.02) * 0.04, z: bodyZ };
            joints["elbowL"] = { x: bodyX - 0.5, y: bodyY + 0.3 + Math.sin(tickCount * 0.03) * 0.1, z: bodyZ + 0.2 };
            joints["elbowR"] = { x: bodyX + 0.5, y: bodyY + 0.3 - Math.sin(tickCount * 0.03) * 0.1, z: bodyZ - 0.2 };
            joints["handL"] = { x: bodyX - 0.65, y: bodyY + 0.2 + Math.sin(tickCount * 0.02) * 0.15, z: bodyZ + 0.3 };
            joints["handR"] = { x: bodyX + 0.65, y: bodyY + 0.2 - Math.sin(tickCount * 0.02) * 0.15, z: bodyZ - 0.3 };
            joints["midHip"] = { x: bodyX * 0.5, y: bodyY + 0.1, z: bodyZ };
            joints["hipL"] = { x: bodyX * 0.5 - 0.18, y: bodyY, z: bodyZ };
            joints["hipR"] = { x: bodyX * 0.5 + 0.18, y: bodyY, z: bodyZ };
            joints["kneeL"] = { x: bodyX * 0.4 - 0.22, y: 0.12, z: bodyZ + 0.15 };
            joints["kneeR"] = { x: bodyX * 0.4 + 0.22, y: 0.12, z: bodyZ - 0.15 };
            joints["footL"] = { x: bodyX * 0.3 - 0.25, y: -0.15, z: bodyZ + 0.25 };
            joints["footR"] = { x: bodyX * 0.3 + 0.25, y: -0.15, z: bodyZ - 0.25 };

          } else if (selectedPresetObj.animationStyle === "workout") {
            // Rapid vertical squats
            const squatDepth = 0.35 * (1 + Math.sin(tickCount * 0.05)); // 0.0 to 0.7 squat cyclic
            bodyY = 0.5 - squatDepth * 0.6;
            joints["head"] = { x: bodyX, y: bodyY + 0.75, z: bodyZ };
            joints["neck"] = { x: bodyX, y: bodyY + 0.6, z: bodyZ };
            joints["shoulderL"] = { x: bodyX - 0.35, y: bodyY + 0.55, z: bodyZ };
            joints["shoulderR"] = { x: bodyX + 0.35, y: bodyY + 0.55, z: bodyZ };
            // Raise arms in front for balance squat
            joints["elbowL"] = { x: bodyX - 0.35, y: bodyY + 0.5, z: bodyZ - 0.45 };
            joints["elbowR"] = { x: bodyX + 0.35, y: bodyY + 0.5, z: bodyZ - 0.45 };
            joints["handL"] = { x: bodyX - 0.35, y: bodyY + 0.55, z: bodyZ - 0.7 };
            joints["handR"] = { x: bodyX + 0.35, y: bodyY + 0.55, z: bodyZ - 0.7 };
            joints["midHip"] = { x: bodyX, y: bodyY + 0.08, z: bodyZ };
            joints["hipL"] = { x: bodyX - 0.18, y: bodyY, z: bodyZ };
            joints["hipR"] = { x: bodyX + 0.18, y: bodyY, z: bodyZ };
            // Knee bend outwards/forwards
            joints["kneeL"] = { x: bodyX - 0.33, y: bodyY * 0.5 + 0.05, z: bodyZ - 0.2 };
            joints["kneeR"] = { x: bodyX + 0.33, y: bodyY * 0.5 + 0.05, z: bodyZ - 0.2 };
            joints["footL"] = { x: bodyX - 0.28, y: -0.15, z: bodyZ };
            joints["footR"] = { x: bodyX + 0.28, y: -0.15, z: bodyZ };

          } else if (selectedPresetObj.animationStyle === "fall") {
            // horizontal body collapse flatline simulation
            const fallTimer = Math.min(tickCount * 0.04, 3.5);
            // Translate from vertical coordinates downwards to floor level horizontal layout
            const fallProgress = Math.min(1, fallTimer / 2.5); // transition
            
            bodyY = 0.5 * (1 - fallProgress) - 0.12;
            const tiltZ = -0.7 * fallProgress;
            const tiltX = -0.5 * fallProgress;

            joints["head"] = { x: tiltX - 0.65, y: bodyY + 0.1, z: tiltZ - 0.1 };
            joints["neck"] = { x: tiltX - 0.4, y: bodyY + 0.08, z: tiltZ };
            joints["shoulderL"] = { x: tiltX - 0.35, y: bodyY + 0.18, z: tiltZ + 0.25 };
            joints["shoulderR"] = { x: tiltX - 0.35, y: bodyY - 0.05, z: tiltZ - 0.25 };
            joints["elbowL"] = { x: tiltX - 0.15, y: bodyY + 0.05, z: tiltZ + 0.4 };
            joints["elbowR"] = { x: tiltX - 0.25, y: bodyY - 0.1, z: tiltZ - 0.4 };
            joints["handL"] = { x: tiltX, y: bodyY, z: tiltZ + 0.35 };
            joints["handR"] = { x: tiltX - 0.1, y: bodyY - 0.12, z: tiltZ - 0.55 };
            joints["midHip"] = { x: tiltX + 0.15, y: bodyY + 0.02, z: tiltZ };
            joints["hipL"] = { x: tiltX + 0.1, y: bodyY + 0.1, z: tiltZ + 0.15 };
            joints["hipR"] = { x: tiltX + 0.1, y: bodyY - 0.06, z: tiltZ - 0.15 };
            joints["kneeL"] = { x: tiltX + 0.45, y: bodyY + 0.08, z: tiltZ + 0.1 };
            joints["kneeR"] = { x: tiltX + 0.5, y: bodyY - 0.1, z: tiltZ - 0.18 };
            joints["footL"] = { x: tiltX + 0.75, y: bodyY, z: tiltZ + 0.15 };
            joints["footR"] = { x: tiltX + 0.8, y: bodyY - 0.15, z: tiltZ - 0.1 };

          } else {
            // idle / empty room noise perturbation
            bodyY = 0.5 + Math.sin(tickCount * 0.005) * 0.01;
            const jitter = () => selectedPresetObj.id === "empty_room" ? 0 : (Math.random() - 0.5) * 0.011;
            joints["head"] = { x: bodyX + jitter(), y: bodyY + 0.75, z: bodyZ };
            joints["neck"] = { x: bodyX, y: bodyY + 0.6, z: bodyZ };
            joints["shoulderL"] = { x: bodyX - 0.35, y: bodyY + 0.55, z: bodyZ };
            joints["shoulderR"] = { x: bodyX + 0.35, y: bodyY + 0.55, z: bodyZ };
            joints["elbowL"] = { x: bodyX - 0.45, y: bodyY + 0.35, z: bodyZ };
            joints["elbowR"] = { x: bodyX + 0.45, y: bodyY + 0.35, z: bodyZ };
            joints["handL"] = { x: bodyX - 0.55, y: bodyY + 0.15, z: bodyZ };
            joints["handR"] = { x: bodyX + 0.55, y: bodyY + 0.15, z: bodyZ };
            joints["midHip"] = { x: bodyX, y: bodyY + 0.08, z: bodyZ };
            joints["hipL"] = { x: bodyX - 0.18, y: bodyY, z: bodyZ };
            joints["hipR"] = { x: bodyX + 0.18, y: bodyY, z: bodyZ };
            joints["kneeL"] = { x: bodyX - 0.22, y: bodyY * 0.4, z: bodyZ };
            joints["kneeR"] = { x: bodyX + 0.22, y: bodyY * 0.4, z: bodyZ };
            joints["footL"] = { x: bodyX - 0.25, y: -0.15, z: bodyZ };
            joints["footR"] = { x: bodyX + 0.25, y: -0.15, z: bodyZ };
          }

          // Render connections if capturing and biological presence exists
          if (isCapturing && selectedPresetObj.id !== "empty_room") {
            // Setup dynamic skeleton lines
            const bones = [
              ["head", "neck"],
              ["neck", "shoulderL"],
              ["neck", "shoulderR"],
              ["shoulderL", "elbowL"],
              ["shoulderR", "elbowR"],
              ["elbowL", "handL"],
              ["elbowR", "handR"],
              ["neck", "midHip"],
              ["midHip", "hipL"],
              ["midHip", "hipR"],
              ["hipL", "kneeL"],
              ["hipR", "kneeR"],
              ["kneeL", "footL"],
              ["kneeR", "footR"]
            ];

            // Render bones
            ctx.lineWidth = 2.5;
            ctx.shadowBlur = 4;
            ctx.shadowColor = selectedPresetObj.guardRisk === "CRITICAL" ? canvasTextSelectionColors.alert : canvasTextSelectionColors.cyan;
            ctx.strokeStyle = selectedPresetObj.guardRisk === "CRITICAL" ? "rgba(248, 113, 113, 0.85)" : "rgba(34, 211, 238, 0.85)";

            bones.forEach(([jA, jB]) => {
              if (joints[jA] && joints[jB]) {
                projectLine(ctx, joints[jA], joints[jB], w, h, activeYaw, pitch, zoom);
              }
            });

            // Draw breathing expansion sphere around midHip
            if (joints["midHip"] && selectedPresetObj.respRate > 0) {
              const sphereRad = 0.45 + Math.sin(tickCount * 0.05) * 0.05;
              ctx.strokeStyle = selectedPresetObj.guardRisk === "CRITICAL" ? "rgba(239, 68, 68, 0.14)" : "rgba(34, 211, 238, 0.12)";
              ctx.lineWidth = 0.75;
              ctx.beginPath();
              const points = [];
              for (let a = 0; a < 360; a += 15) {
                const rad = (a * Math.PI) / 180;
                const spPoint = {
                  x: joints["midHip"].x + Math.cos(rad) * sphereRad,
                  y: joints["midHip"].y + Math.sin(rad) * sphereRad * 1.5, // slightly ellipsoidal
                  z: joints["midHip"].z
                };
                const p = renderProject3D(spPoint, w, h, activeYaw, pitch, zoom);
                if (p) points.push(p);
              }
              if (points.length > 0) {
                ctx.moveTo(points[0].x, points[0].y);
                for (let pi = 1; pi < points.length; pi++) {
                  ctx.lineTo(points[pi].x, points[pi].y);
                }
                ctx.closePath();
                ctx.stroke();
              }
            }

            // Render joint points/nodes
            ctx.shadowBlur = 6;
            ctx.fillStyle = selectedPresetObj.guardRisk === "CRITICAL" ? "#ef4444" : "#06b6d4";
            Object.entries(joints).forEach(([key, point]) => {
              const proj = renderProject3D(point, w, h, activeYaw, pitch, zoom);
              if (proj) {
                ctx.beginPath();
                ctx.arc(proj.x, proj.y, 4, 0, Math.PI * 2);
                ctx.fill();
              }
            });
            ctx.shadowBlur = 0;

            // Render joint text annotations
            ctx.fillStyle = "rgba(34, 211, 238, 0.7)";
            ctx.font = "5px monospace";
            const labeledJoints = ["head", "handL", "handR", "footL", "footR"];
            labeledJoints.forEach(key => {
              if (joints[key]) {
                const proj = renderProject3D(joints[key], w, h, activeYaw, pitch, zoom);
                if (proj) {
                  ctx.fillText(key.toUpperCase(), proj.x + 8, proj.y - 2);
                }
              }
            });

            // Render HUD metrics inside skeleton space
            ctx.fillStyle = "rgba(167, 139, 250, 0.15)";
            ctx.strokeStyle = "rgba(167, 139, 250, 0.4)";
            ctx.lineWidth = 1;
            ctx.fillRect(8, 8, 110, 48);
            ctx.strokeRect(8, 8, 110, 48);

            ctx.fillStyle = "#a78bfa";
            ctx.font = "bold 7px monospace";
            ctx.fillText("ACTIVE RF RADAR", 12, 18);
            ctx.fillStyle = "#94a3b8";
            ctx.font = "6.5px monospace";
            ctx.fillText(`FPS: 60.0 S/C`, 12, 27);
            ctx.fillText(`VELOCITY: ${selectedPresetObj.motionVelocity} m/s`, 12, 35);
            ctx.fillStyle = selectedPresetObj.guardRisk === "CRITICAL" ? "#f87171" : "#22d3ee";
            ctx.fillText(`JOINT ACCU: ${(sensitivity * 1.1).toFixed(1)}%`, 12, 44);

            // Draw bounding radar tracking circle around head
            const headProj = renderProject3D(joints["head"], w, h, activeYaw, pitch, zoom);
            if (headProj) {
              ctx.strokeStyle = selectedPresetObj.guardRisk === "CRITICAL" ? "rgba(248, 113, 113, 0.35)" : "rgba(34, 211, 238, 0.35)";
              ctx.beginPath();
              ctx.arc(headProj.x, headProj.y, 22, 0, Math.PI * 2);
              ctx.stroke();
              ctx.font = "5px monospace";
              ctx.fillStyle = "rgba(34, 211, 238, 0.7)";
              ctx.fillText("TRACKING_NODE_H01", headProj.x + 25, headProj.y - 12);
            }
          } else {
            // Draw scanning baseline text
            ctx.fillStyle = "rgba(71, 85, 105, 0.6)";
            ctx.font = "9px monospace";
            ctx.textAlign = "center";
            ctx.fillText(
              isCapturing ? "AWAITING SPATIAL WAVE HANDSHAKE..." : "RADAR POWEROFF",
              w / 2,
              h / 2
            );
            ctx.textAlign = "left";
          }
        }
      }

      // ------------------------------------
      // DRAW PART 2: RF CSI WAVEFORMS
      // ------------------------------------
      const canvas2 = waveCanvasRef.current;
      if (canvas2) {
        const ctx2 = canvas2.getContext("2d");
        if (ctx2) {
          const w = canvas2.width;
          const h = canvas2.height;
          ctx2.fillStyle = "#010406";
          ctx2.fillRect(0, 0, w, h);

          // Draw gridlines
          ctx2.strokeStyle = "#04090f";
          ctx2.lineWidth = 1;
          for (let col = 0; col < w; col += 30) {
            ctx2.beginPath();
            ctx2.moveTo(col, 0);
            ctx2.lineTo(col, h);
            ctx2.stroke();
          }
          for (let row = 0; row < h; row += 20) {
            ctx2.beginPath();
            ctx2.moveTo(0, row);
            ctx2.lineTo(w, row);
            ctx2.stroke();
          }

          if (isCapturing) {
            // Setup multi-antenna receiver channels (MIMO representation)
            const rxColors = ["rgba(139, 92, 246, 0.35)", "rgba(59, 130, 246, 0.3)", "rgba(236, 72, 153, 0.25)"];
            const amplitudeScale = 1.0 - (suppressionDb / 100);

            // Draw MIMO raw streams (Antennas Rx1, Rx2, Rx3)
            for (let ch = 0; ch < 3; ch++) {
              ctx2.beginPath();
              ctx2.lineWidth = 0.8;
              ctx2.strokeStyle = rxColors[ch];
              
              for (let x = 0; x < w; x++) {
                let val = 0;
                // Add slight phase shift for each channel
                const phaseShift = ch * 12;
                
                if (selectedPresetObj.respRate > 0) {
                  val += Math.sin((x * 0.05) - (tickCount * 0.04) + phaseShift) * 12 * (resScale / 100);
                }
                if (selectedPresetObj.pulseBpm > 0) {
                  const heartbeatPulse = Math.max(0, Math.sin((x * 0.12) - (tickCount * 0.12) + phaseShift));
                  val += Math.pow(heartbeatPulse, 4) * 10;
                }
                if (!petFilter && selectedPresetObj.id !== "empty_room") {
                  val += Math.sin(x * 1.1 + tickCount * 0.5 + phaseShift) * 15 * selectedPresetObj.interferenceLevel;
                }
                if (!fanFilter && selectedPresetObj.id !== "empty_room") {
                  val += Math.sin(x * 0.8 - tickCount * 0.8 + phaseShift) * 12 * selectedPresetObj.interferenceLevel;
                }
                if (!multipathFilter) {
                  val += Math.sin(x * 0.02 - tickCount * 0.01 + phaseShift) * 18 * selectedPresetObj.interferenceLevel;
                }
                
                const y = h / 2 + val * amplitudeScale;
                if (x === 0) ctx2.moveTo(x, y);
                else ctx2.lineTo(x, y);
              }
              ctx2.stroke();
            }

            // Render a SECOND channel (glowing cyan stream representing the "Consensus Clustered Bio-Signal")
            ctx2.beginPath();
            ctx2.lineWidth = 1.8;
            ctx2.strokeStyle = selectedPresetObj.guardRisk === "CRITICAL" ? "#f87171" : "#22d3ee";
            ctx2.shadowBlur = 4;
            ctx2.shadowColor = selectedPresetObj.guardRisk === "CRITICAL" ? "#ef4444" : "#06b6d4";

            for (let x = 0; x < w; x++) {
              let cleanBioValue = 0;
              if (selectedPresetObj.respRate > 0) {
                cleanBioValue += Math.sin((x * 0.05) - (tickCount * 0.045)) * 11;
              }
              if (selectedPresetObj.pulseBpm > 0) {
                const heartbeatPulse = Math.max(0, Math.sin((x * 0.12) - (tickCount * 0.125)));
                cleanBioValue += Math.pow(heartbeatPulse, 6) * 8;
              }

              // Subtle baseline noise even when filtered
              cleanBioValue += (Math.random() - 0.5) * 0.8;

              const y2 = h / 2 + cleanBioValue;
              if (x === 0) {
                ctx2.moveTo(x, y2);
              } else {
                ctx2.lineTo(x, y2);
              }
            }
            ctx2.stroke();
            ctx2.shadowBlur = 0;

            // Draw mini FFT spectrum bar chart overlay
            const barWidth = 4;
            const barGap = 2;
            const barCount = 14;
            ctx2.fillStyle = "rgba(4, 9, 15, 0.8)";
            ctx2.strokeStyle = "rgba(30, 41, 59, 0.8)";
            ctx2.fillRect(w - 100, h - 55, 92, 48);
            ctx2.strokeRect(w - 100, h - 55, 92, 48);
            
            ctx2.fillStyle = "rgba(148, 163, 184, 0.4)";
            ctx2.font = "5px monospace";
            ctx2.fillText("FFT SPECTRUM (dB)", w - 95, h - 48);

            for (let i = 0; i < barCount; i++) {
              let barHeight = 0;
              // Simulate spectrum levels depending on active filters
              if (selectedPresetObj.id !== "empty_room") {
                if (i === 1) { // Resp rate (slow ~0.2Hz)
                  barHeight = 12 + Math.sin(tickCount * 0.08) * 4;
                } else if (i === 3) { // Cardiac rate (~1.2Hz)
                  barHeight = 22 + Math.cos(tickCount * 0.14) * 6;
                } else if (i === 8 && !fanFilter) { // Fan noise (~3Hz)
                  barHeight = 16 + Math.random() * 8;
                } else if (i === 12 && !petFilter) { // Pet jitter (~12Hz)
                  barHeight = 24 + Math.random() * 12;
                } else {
                  barHeight = 3 + Math.random() * 3;
                }
              } else {
                barHeight = 2 + Math.random() * 1.5;
              }
              
              ctx2.fillStyle = (i === 1 || i === 3) ? "rgba(34, 211, 238, 0.7)" : "rgba(139, 92, 246, 0.4)";
              ctx2.fillRect(w - 94 + i * (barWidth + barGap), h - 8 - barHeight, barWidth, barHeight);
            }
          } else {
            // Draw flat dead static waves
            ctx2.strokeStyle = "rgba(148, 163, 184, 0.15)";
            ctx2.beginPath();
            ctx2.moveTo(0, h / 2);
            ctx2.lineTo(w, h / 2);
            ctx2.stroke();
          }
        }
      }

      animationId = requestAnimationFrame(render);
    };

    // Helper functions for math projections
    const projectLine = (
      ctx: CanvasRenderingContext2D,
      p1: Vector3D,
      p2: Vector3D,
      w: number,
      h: number,
      yaw: number,
      pitch: number,
      zoom: number
    ) => {
      const proj1 = renderProject3D(p1, w, h, yaw, pitch, zoom);
      const proj2 = renderProject3D(p2, w, h, yaw, pitch, zoom);
      if (proj1 && proj2) {
        ctx.beginPath();
        ctx.moveTo(proj1.x, proj1.y);
        ctx.lineTo(proj2.x, proj2.y);
        ctx.stroke();
      }
    };

    const renderProject3D = (
      v: Vector3D,
      w: number,
      h: number,
      yawDeg: number,
      pitchDeg: number,
      scale: number
    ) => {
      const radYaw = (yawDeg * Math.PI) / 180;
      const radPitch = (pitchDeg * Math.PI) / 180;

      // 1. Orbit Rotation around Y axis (Yaw)
      let rX = v.x * Math.cos(radYaw) - v.z * Math.sin(radYaw);
      let rZ = v.x * Math.sin(radYaw) + v.z * Math.cos(radYaw);

      // 2. Orbit Rotation around X axis (Pitch)
      let rY = v.y * Math.cos(radPitch) - rZ * Math.sin(radPitch);
      rZ = v.y * Math.sin(radPitch) + rZ * Math.cos(radPitch);

      // Orthographic projection offset with deep center focus
      const scrX = w / 2 + rX * scale;
      const scrY = h * 0.7 - rY * scale; // anchor base at approx bottom-half
      return { x: scrX, y: scrY };
    };

    render();

    return () => {
      cancelAnimationFrame(animationId);
    };
  }, [activePreset, isCapturing, petFilter, fanFilter, multipathFilter, yaw, pitch, zoom, sensitivity, suppressionDb, resScale, selectedPresetObj, autoRotate]);

  return (
    <div className="space-y-4 animate-fadeIn">
      {/* Overview Intro Banner */}
      <div className="bg-gradient-to-r from-cyan-950/20 via-[#06101c]/30 to-violet-950/20 border border-cyan-900/30 p-4 rounded-lg flex flex-col md:flex-row md:items-center justify-between gap-3 font-mono text-xs">
        <div className="space-y-1.5">
          <div className="flex items-center gap-1.5 text-cyan-400 font-bold uppercase tracking-wider">
            <Wifi className="w-4 h-4 text-cyan-400 animate-pulse" />
            <span>Biometric Wave-Space Vision Layer</span>
          </div>
          <p className="text-gray-400 text-[10px] leading-relaxed max-w-xl">
            Synthesizing <strong>Pulse-Fi &amp; WiPose algorithms</strong> to recognize skeletal geometry, metabolic respiration rates, and heart performance metrics. Harnessing non-invasive RF reflections instead of cameras to sustain pristine absolute privacy.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => {
              setIsCapturing(!isCapturing);
              speakText(isCapturing ? "Wi-fi spatial capture stream halted." : "Resuming ambient Wi-Fi optical radar tracking.");
            }}
            className={`py-1 px-2.5 rounded border font-bold text-[9px] uppercase cursor-pointer flex items-center gap-1 ${
              isCapturing 
                ? "bg-emerald-950/20 text-emerald-400 border-emerald-900/40 hover:bg-emerald-900/30" 
                : "bg-red-950/20 text-red-400 border-red-900/40 hover:bg-red-900/30"
            }`}
          >
            {isCapturing ? <Play className="w-2.5 h-2.5 fill-emerald-400" /> : <Square className="w-2.5 h-2.5 fill-red-400" />}
            <span>{isCapturing ? "RADAR: ON" : "RADAR: SLEEP"}</span>
          </button>
          
          <button
            onClick={triggerVoiceGreeting}
            className="p-1 px-2.5 rounded border border-cyan-850/40 bg-cyan-950/20 text-cyan-400 hover:bg-cyan-900/30 hover:border-cyan-700/50 text-[9px] font-bold uppercase tracking-wider flex items-center gap-1 cursor-pointer"
          >
            <Activity className="w-2.5 h-2.5" />
            <span>VOICE SUMMARIZE</span>
          </button>
        </div>
      </div>

      {/* Main split grid: Viewport controls vs diagnostics */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-4">

        {/* Column 1: Left Grid - 3D Render Canvas and Wave plots */}
        <div className="lg:col-span-8 space-y-4">
          
          {/* 3D Skeletal Mesh Viewport */}
          <div className="relative bg-[#020509] border border-gray-850 rounded-lg overflow-hidden group">
            {/* Viewport header tags */}
            <div className="absolute top-2.5 left-2.5 z-10 font-mono text-[8px] tracking-widest flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-full bg-cyan-500 animate-ping" />
              <span className="text-gray-300 font-bold bg-black/60 px-1.5 py-0.5 border border-gray-850 rounded uppercase">
                WiPose 3D Skeleton Proj // Space: ACTIVE
              </span>
            </div>

            <div className="absolute top-2.5 right-2.5 z-10 font-mono text-[8px] tracking-tight text-gray-500 bg-black/60 px-1.5 py-0.5 rounded border border-gray-850">
              ROTATE: Drag mouse over canvas // YAW: {Math.round(yaw)}° PITCH: {Math.round(pitch)}°
            </div>

            {/* Skeleton Canvas */}
            <canvas
              ref={skeletonCanvasRef}
              width={540}
              height={260}
              onMouseDown={handleMouseDown}
              onMouseMove={handleMouseMove}
              onMouseUp={handleMouseUp}
              onMouseLeave={handleMouseUp}
              className="w-full h-[260px] block cursor-grab active:cursor-grabbing"
            />

            {/* Viewport controls overlay bottom left */}
            <div className="absolute bottom-2.5 left-2.5 z-10 font-mono text-[8px] flex items-center gap-2 bg-black/40 p-1 rounded">
              <span className="text-gray-500">CAMERA ZOOM:</span>
              <button 
                onClick={() => setZoom(prev => Math.max(100, prev - 20))}
                className="px-1.5 bg-gray-900 border border-gray-800 text-gray-300 rounded cursor-pointer hover:bg-gray-800"
              >
                -
              </button>
              <span className="text-cyan-400 font-bold">{zoom}pt</span>
              <button 
                onClick={() => setZoom(prev => Math.min(300, prev + 20))}
                className="px-1.5 bg-gray-900 border border-gray-800 text-gray-300 rounded cursor-pointer hover:bg-gray-800"
              >
                +
              </button>

              <button 
                onClick={() => setAutoRotate(!autoRotate)}
                className={`ml-2 px-1.5 py-0.5 rounded cursor-pointer border font-bold text-[7px] tracking-wider uppercase transition-all ${
                  autoRotate 
                    ? "bg-cyan-950/40 text-cyan-450 border-cyan-900/50" 
                    : "bg-gray-900 text-gray-500 border-gray-850 hover:text-gray-300"
                }`}
              >
                {autoRotate ? "ORBIT: AUTOROT" : "ORBIT: STATIC"}
              </button>
            </div>
            
            <div className="absolute bottom-2.5 right-2.5 z-10 font-mono text-[8px] text-gray-400 font-semibold bg-black/60 px-2 py-0.5 border border-cyan-900/40 rounded uppercase bg-gradient-to-r from-transparent to-cyan-950/10">
              Active Presets Status: <span className="text-cyan-400 font-bold">{selectedPresetObj.postureStatus}</span>
            </div>
          </div>

          {/* CSI Stream Plotting Waveform */}
          <div className="bg-[#020509] border border-gray-850 rounded-lg overflow-hidden relative">
            <div className="p-2 border-b border-gray-850 bg-black/25 flex items-center justify-between font-mono text-[8px] tracking-wider uppercase">
              <span className="text-violet-400 font-bold flex items-center gap-1">
                <Filter className="w-3 h-3 text-violet-400" />
                Real-time CSI Amplitude Waveform Stream (Raw vs. Decoded)
              </span>
              <div className="flex items-center gap-2.5">
                <span className="flex items-center gap-1"><span className="w-1.5 h-1.5 rounded-full bg-violet-500" /> Raw Multi-path</span>
                <span className="flex items-center gap-1"><span className="w-1.5 h-1.5 rounded-full bg-cyan-400" /> Clustered Vitals Consensus</span>
              </div>
            </div>

            <canvas
              ref={waveCanvasRef}
              width={540}
              height={100}
              className="w-full h-[100px] block"
            />
          </div>

        </div>

        {/* Column 2: Right Grid - Presets, Swarm Filters, Orchestrator agents */}
        <div className="lg:col-span-4 space-y-4">
          
          {/* Target Presets selection */}
          <div className="bg-black/20 border border-gray-850/80 p-3.5 rounded-lg space-y-2.5 font-mono">
            <span className="text-gray-400 font-bold tracking-wider text-[9px] uppercase block border-b border-gray-850/60 pb-1 flex items-center gap-1">
              <Sliders className="w-3.5 h-3.5 text-cyan-400" />
              Calibration Targets Presets
            </span>
            <div className="grid grid-cols-1 gap-1.5">
              {PRESETS.map((preset) => (
                <button
                  key={preset.id}
                  onClick={() => {
                    setActivePreset(preset.id);
                    // Autofills or adjusts suppression triggers
                    if (preset.id === "empty_room") {
                      setSensitivity(10);
                      setSuppressionDb(80);
                    } else if (preset.id === "workout_squat") {
                      setSensitivity(92);
                      setSuppressionDb(20);
                    } else {
                      setSensitivity(85);
                      setSuppressionDb(38);
                    }
                    speakText(`Synthesizing calibration matrix: ${preset.name}`);
                  }}
                  className={`w-full p-2 rounded text-left text-[9.5px] transition-all flex flex-col gap-1 border cursor-pointer ${
                    activePreset === preset.id
                      ? "bg-cyan-950/15 border-cyan-500/50 text-cyan-300"
                      : "bg-[#03060a]/40 border-gray-900 text-gray-500 hover:text-gray-300 hover:bg-black/30"
                  }`}
                >
                  <div className="flex items-center justify-between w-full font-bold">
                    <span>{preset.name}</span>
                    <span className={`text-[7px] px-1 rounded border ${
                      preset.guardRisk === "CRITICAL" 
                        ? "bg-red-950/20 text-red-400 border-red-900/40" 
                        : preset.guardRisk === "NORMAL"
                        ? "bg-sky-950/20 text-sky-400 border-sky-900/40"
                        : "bg-emerald-950/20 text-emerald-400 border-emerald-900/40"
                    }`}>
                      {preset.guardRisk} RISK
                    </span>
                  </div>
                  <p className="text-[7.5px] text-gray-500 leading-normal lowercase first-letter:uppercase">
                    {preset.description}
                  </p>
                </button>
              ))}
            </div>
          </div>

          {/* Swarm Filter Controls */}
          <div className="bg-black/20 border border-gray-850 p-3.5 rounded-lg space-y-2.5 font-mono text-[9px]">
            <span className="text-gray-400 font-bold tracking-wider text-[9px] uppercase block border-b border-gray-850 pb-1 flex items-center gap-1">
              <Atom className="w-3.5 h-3.5 text-violet-400" />
              Active Swarm Noise Resolution Filters
            </span>

            <div className="space-y-2 pt-0.5">
              <div className="flex items-center justify-between p-1.5 bg-[#03060a]/50 rounded border border-gray-900">
                <div className="flex flex-col">
                  <span className="text-gray-300 font-semibold uppercase text-[8.5px]">PetFilter Module</span>
                  <span className="text-gray-600 text-[7px]">Squelch high-frequency rapid jitter (&gt;4.5 Hz)</span>
                </div>
                <button
                  onClick={() => setPetFilter(!petFilter)}
                  className={`px-1.5 py-0.5 rounded text-[8px] font-bold border transition-all cursor-pointer ${
                    petFilter ? "bg-cyan-950/20 text-cyan-400 border-cyan-900/50" : "bg-black/30 text-gray-600 border-gray-850"
                  }`}
                >
                  {petFilter ? "ENFORCED" : "HALTED"}
                </button>
              </div>

              <div className="flex items-center justify-between p-1.5 bg-[#03060a]/50 rounded border border-gray-900">
                <div className="flex flex-col">
                  <span className="text-gray-300 font-semibold uppercase text-[8.5px]">FanOscillation Filter</span>
                  <span className="text-gray-600 text-[7px]">Harmonic suppression of cyclic rotating fans</span>
                </div>
                <button
                  onClick={() => setFanFilter(!fanFilter)}
                  className={`px-1.5 py-0.5 rounded text-[8px] font-bold border transition-all cursor-pointer ${
                    fanFilter ? "bg-cyan-950/20 text-cyan-400 border-cyan-900/50" : "bg-black/30 text-gray-600 border-gray-850"
                  }`}
                >
                  {fanFilter ? "ENFORCED" : "HALTED"}
                </button>
              </div>

              <div className="flex items-center justify-between p-1.5 bg-[#03060a]/50 rounded border border-gray-900">
                <div className="flex flex-col">
                  <span className="text-gray-300 font-semibold uppercase text-[8.5px]">MultiPathResolver</span>
                  <span className="text-gray-600 text-[7px]">Compensate spatial phase-delay fades</span>
                </div>
                <button
                  onClick={() => setMultipathFilter(!multipathFilter)}
                  className={`px-1.5 py-0.5 rounded text-[8px] font-bold border transition-all cursor-pointer ${
                    multipathFilter ? "bg-cyan-950/20 text-cyan-400 border-cyan-900/50" : "bg-black/30 text-gray-600 border-gray-850"
                  }`}
                >
                  {multipathFilter ? "ENFORCED" : "HALTED"}
                </button>
              </div>
            </div>

            {/* Extra manual dials */}
            <div className="space-y-2 pt-1 border-t border-gray-900">
              <div className="space-y-1">
                <div className="flex justify-between text-[7px] text-gray-500">
                  <span>ANTENNA SENSITIVITY MULTIPLIER</span>
                  <span className="text-cyan-400">{sensitivity}%</span>
                </div>
                <input 
                  type="range" 
                  min="0" 
                  max="100" 
                  value={sensitivity} 
                  onChange={(e) => setSensitivity(Number(e.target.value))}
                  className="w-full accent-cyan-400 h-1 bg-gray-900 rounded-lg cursor-pointer"
                />
              </div>

              <div className="space-y-1">
                <div className="flex justify-between text-[7px] text-gray-500">
                  <span>MULTIPATH SUPPRESSION DEPTH</span>
                  <span className="text-violet-400">{suppressionDb} dB</span>
                </div>
                <input 
                  type="range" 
                  min="5" 
                  max="95" 
                  value={suppressionDb} 
                  onChange={(e) => setSuppressionDb(Number(e.target.value))}
                  className="w-full accent-violet-400 h-1 bg-gray-900 rounded-lg cursor-pointer"
                />
              </div>
            </div>
          </div>

        </div>

      </div>

      {/* Multi-Agent Orchestration Logs & Diagnostic readout */}
      <div className="grid grid-cols-1 md:grid-cols-12 gap-4">
        
        {/* Core Vitals readout */}
        <div className="md:col-span-5 bg-[#03060a] border border-gray-850 p-4 rounded-lg flex flex-col justify-between font-mono">
          <div className="space-y-2">
            <span className="text-gray-400 font-bold uppercase tracking-widest text-[8px] block border-b border-gray-850 pb-1 flex items-center gap-1">
              <Heart className="w-3.5 h-3.5 text-red-500 animate-pulse" />
              BIOMETRIC VITALS RESOLVER (PULSE-FI)
            </span>

            {/* Cardiac & Respiration grid */}
            {selectedPresetObj.id !== "empty_room" && isCapturing ? (
              <div className="grid grid-cols-2 gap-3 pt-2">
                <div className="bg-black/50 border border-gray-900 p-2.5 rounded text-center relative overflow-hidden">
                  <div className="absolute top-1 right-2"><span className="text-[6.5px] text-red-500 font-bold animate-ping">♥</span></div>
                  <span className="text-gray-550 text-[7px] font-bold block uppercase">HEART RATE</span>
                  <span className="text-2xl font-black text-gray-100 block tracking-tight my-1">{selectedPresetObj.pulseBpm} <span className="text-[10px] font-medium text-gray-500">BPM</span></span>
                  <span className="text-[7px] font-semibold bg-red-950/20 text-red-400 border border-red-900/30 px-1 rounded inline-block uppercase">Cardiac Flow</span>
                </div>

                <div className="bg-black/50 border border-gray-900 p-2.5 rounded text-center relative overflow-hidden">
                  <div className="absolute top-1 right-2"><span className="text-[6.5px] text-cyan-400 font-bold animate-pulse">~</span></div>
                  <span className="text-gray-555 text-[7px] font-bold block uppercase">RESPIRATORY LEVEL</span>
                  <span className="text-2xl font-black text-gray-100 block tracking-tight my-1">{selectedPresetObj.respRate} <span className="text-[10px] font-medium text-gray-500">BREATHS</span></span>
                  <span className="text-[7px] font-semibold bg-cyan-950/20 text-cyan-400 border border-cyan-900/30 px-1 rounded inline-block uppercase">Diaphragmatic</span>
                </div>
              </div>
            ) : (
              <div className="py-8 text-center text-[10px] text-gray-600 block uppercase font-bold tracking-widest animate-pulse">
                - NO LIVE BIO-WAVES REGISTERED IN AIRSPACE -
              </div>
            )}
          </div>

          <div className="pt-3 border-t border-gray-900 mt-3 flex items-center justify-between text-[10px]">
            <span className="text-gray-600 uppercase text-[8px]">Synapse persistence state:</span>
            <button
              onClick={handlePersistToSynapticGraph}
              disabled={syncingState === "SYNCING" || selectedPresetObj.id === "empty_room"}
              className={`p-1 px-2.5 rounded border font-bold text-[8.5px] uppercase flex items-center gap-1 transition-all cursor-pointer ${
                selectedPresetObj.id === "empty_room"
                  ? "bg-gray-950 border-gray-900 text-gray-700 cursor-not-allowed"
                  : syncingState === "SYNCING"
                  ? "bg-cyan-950 text-cyan-500 animate-pulse border-cyan-900"
                  : syncingState === "COMPLETED"
                  ? "bg-emerald-950/20 text-emerald-400 border-emerald-900/40"
                  : "bg-cyan-950/20 text-cyan-400 border-cyan-900/40 hover:bg-cyan-900/30"
              }`}
            >
              <Database className="w-2.5 h-2.5" />
              <span>{syncingState === "SYNCING" ? "SYNCING..." : syncingState === "COMPLETED" ? "SYNCHRONIZED" : "SYNC TO J.A.R.V.I.S"}</span>
            </button>
          </div>
          
          {syncNodeResult && (
            <p className="mt-2 text-[7.5px] p-1.5 bg-black/40 text-emerald-400 border border-emerald-900/20 rounded">
              ✔ {syncNodeResult}
            </p>
          )}
        </div>

        {/* Cognitive swarm agent telemetry logs */}
        <div className="md:col-span-7 bg-[#03060a] border border-gray-850 p-4 rounded-lg font-mono text-[9px] space-y-3">
          <div className="flex items-center justify-between border-b border-gray-850 pb-1.5">
            <span className="text-gray-400 font-bold uppercase tracking-widest text-[8px] block flex items-center gap-1">
              <Brain className="w-3.5 h-3.5 text-violet-400 animate-pulse" />
              UQI Multi-Agent Orchestration &amp; Ethics Audit
            </span>
            <span className="text-[7.5px] text-gray-500 bg-black px-1.5 rounded border border-gray-900 font-semibold">COGNITIVE SWARM v1.0.9</span>
          </div>

          <div className="grid grid-cols-1 select-none text-[8.5px] md:grid-cols-2 gap-3">
            {/* Aether */}
            <div className="bg-black/50 border border-gray-900/80 p-2 rounded">
              <div className="flex justify-between items-center mb-1">
                <span className="text-cyan-400 font-semibold uppercase text-[8px]">AETHER (Orchestrator)</span>
                <span className="text-[7px] text-emerald-400">ONLINE</span>
              </div>
              <p className="text-gray-600 font-mono text-[7px] text-slate-400 bg-slate-900/20 p-1 rounded font-bold">Endpoint: /api/agents/orchestrator/signal_fusion</p>
              <p className="text-gray-500 leading-tight mt-1">Fusing multi-path router signals under ESP32 spatial waveforms.</p>
            </div>

            {/* Vanguard */}
            <div className="bg-black/50 border border-gray-900/80 p-2 rounded">
              <div className="flex justify-between items-center mb-1">
                <span className="text-red-400 font-semibold uppercase text-[8px]">VANGUARD (Guardian)</span>
                <span className="text-[7px] text-emerald-400">ONLINE</span>
              </div>
              <p className="text-gray-600 font-mono text-[7px] text-slate-400 bg-slate-900/20 p-1 rounded font-bold">Endpoint: /api/agents/judicial/biometric_ethics</p>
              <p className="text-gray-500 leading-tight mt-1">Ethical enforcement: scans for fall triggers and respiration arrest risks.</p>
            </div>

            {/* Forge */}
            <div className="bg-black/50 border border-gray-900/80 p-2 rounded">
              <div className="flex justify-between items-center mb-1">
                <span className="text-violet-400 font-semibold uppercase text-[8px]">FORGE (Trainer)</span>
                <span className="text-[7px] text-emerald-400">ONLINE</span>
              </div>
              <p className="text-gray-600 font-mono text-[7px] text-slate-400 bg-slate-900/20 p-1 rounded font-bold">Endpoint: /api/agents/executor/form_analysis</p>
              <p className="text-gray-500 leading-tight mt-1">Grading 3D skeleton frames against postural trajectory nodes.</p>
            </div>

            {/* Synaptic Navigator */}
            <div className="bg-black/50 border border-gray-900/80 p-2 rounded">
              <div className="flex justify-between items-center mb-1">
                <span className="text-amber-400 font-semibold uppercase text-[8px]">AMBIENT PRESENCE (Navigator)</span>
                <span className="text-[7px] text-amber-500">ATTUNING</span>
              </div>
              <p className="text-gray-600 font-mono text-[7px] text-slate-400 bg-slate-900/20 p-1 rounded font-bold">Endpoint: /api/emergence/synaptic_navigator/ambient_presence</p>
              <p className="text-gray-500 leading-tight mt-1">Adapting localized audio dialogue to active exercise intent patterns.</p>
            </div>

            {/* Ambient Environment Actuators */}
            <div className="bg-black/50 border border-gray-900/80 p-2 rounded md:col-span-2">
              <div className="flex justify-between items-center mb-1">
                <span className="text-emerald-400 font-semibold uppercase text-[8.5px] tracking-wider block">💡 Ambient Environment Actuators (Physical IoT Feedback)</span>
                <span className="text-[7.5px] text-emerald-400 font-bold bg-emerald-950/30 px-1 border border-emerald-900/30 rounded">ACTIVE LOOP</span>
              </div>
              <p className="text-gray-600 font-mono text-[7px] text-slate-400 bg-slate-900/20 p-1 rounded font-bold">Endpoint: /api/swarm/spatial/ambient</p>
              <div className="grid grid-cols-3 gap-2 mt-2 font-mono text-[9px]">
                <div className="bg-slate-950/40 p-1.5 border border-slate-900 rounded">
                  <span className="text-gray-500 block text-[7.5px] uppercase">Lighting Actuator</span>
                  <span className="text-cyan-400 font-bold">{ambientState.lighting_level}% Intensity</span>
                </div>
                <div className="bg-slate-950/40 p-1.5 border border-slate-900 rounded">
                  <span className="text-gray-500 block text-[7.5px] uppercase">Audio Actuator</span>
                  <span className="text-cyan-400 font-bold">{ambientState.audio_volume}% Volume</span>
                </div>
                <div className="bg-slate-950/40 p-1.5 border border-slate-900 rounded">
                  <span className="text-gray-500 block text-[7.5px] uppercase">Acoustic Mode</span>
                  <span className="text-violet-400 font-bold">{ambientState.audio_mode}</span>
                </div>
              </div>
            </div>
          </div>
          
          {selectedPresetObj.guardRisk === "CRITICAL" && (
            <div className="p-2 border border-red-900/30 bg-red-950/15 rounded flex items-start gap-2 animate-bounce">
              <ShieldAlert className="w-5 h-5 text-red-500 shrink-0" />
              <div>
                <span className="text-red-400 font-bold block uppercase text-[8.5px]">IMMEDIATE VANGUARD ETHICAL ESCALATION TRIPPED!</span>
                <p className="text-[7.5px] text-gray-400 leading-tight">Horizontal static immobilization detected. Firing safety interrupts to emergency alert endpoints and updating conscious presence priorities. Immediate vocal handshake synthesized.</p>
              </div>
            </div>
          )}
        </div>

      </div>
    </div>
  );
};
