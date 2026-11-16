import React, { useState, useEffect, useRef } from "react";
import { 
  Bot, 
  Settings, 
  Play, 
  Pause, 
  RotateCcw, 
  Compass, 
  Map, 
  Eye, 
  Cpu, 
  Network, 
  ChevronRight, 
  Info, 
  ShieldAlert,
  Sliders,
  Sparkles,
  Activity,
  Zap,
  CheckCircle,
  AlertTriangle
} from "lucide-react";

// Types for Path Planning Grid
type CellType = "empty" | "obstacle" | "start" | "goal" | "path" | "visited";

export const RoboticsLab: React.FC<{ speakText?: (text: string) => void }> = ({ speakText }) => {
  // Sub-tabs of Robotics Lab
  const [activeTab, setActiveTab] = useState<"kinematics" | "localization" | "planning" | "avoidance" | "edge_mpc">("kinematics");

  // ==========================================
  // STATE DEFINITIONS
  // ==========================================
  
  // 1. Kinematics Simulation State
  const [kinSimRunning, setKinSimRunning] = useState(false);
  const [wheelLeft, setWheelLeft] = useState(2.0); // rad/s
  const [wheelRight, setWheelRight] = useState(3.0); // rad/s
  const [wheelRadius, setWheelRadius] = useState(1.5); // cm
  const [trackWidth, setTrackWidth] = useState(6.0); // L, distance between wheels (cm)
  const kinCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const kinPoseRef = useRef({ x: 125, y: 125, theta: 0 });
  const kinPathRef = useRef<{ x: number; y: number }[]>([]);

  // 2. Particle Filter Localization State
  const [locSimRunning, setLocSimRunning] = useState(false);
  const [particleCount] = useState(80);
  const [sensorNoise, setSensorNoise] = useState(2.0); // Standard deviation
  const locCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const robotPoseRef = useRef({ x: 50, y: 125 });
  const odometryPoseRef = useRef({ x: 50, y: 125 }); // accumulated noise (dead-reckoning)
  const particlesRef = useRef<{ x: number; y: number; weight: number }[]>([]);
  // Landmarks points on the map
  const landmarks = [
    { id: "A", x: 40, y: 40, color: "#ef4444" },
    { id: "B", x: 210, y: 40, color: "#3b82f6" },
    { id: "C", x: 40, y: 210, color: "#10b981" },
    { id: "D", x: 210, y: 210, color: "#f59e0b" }
  ];

  // 3. Grid Path Planning (A* vs Bug Algorithm) State
  const [planningAlgo, setPlanningAlgo] = useState<"AStar" | "Bug2">("AStar");
  const [gridSize] = useState(15);
  const [startPos, setStartPos] = useState({ r: 2, c: 2 });
  const [goalPos, setGoalPos] = useState({ r: 12, c: 12 });
  const [grid, setGrid] = useState<CellType[][]>([]);
  const [planningSteps, setPlanningSteps] = useState(0);
  const [planningActive, setPlanningActive] = useState(false);

  // 4. Reactive Obstacle Avoidance (Field / Lidar scan) state
  const [avoidSimRunning, setAvoidSimRunning] = useState(false);
  const [avoidSpeed, setAvoidSpeed] = useState(1.5);
  const avoidCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const obstaclePosRef = useRef({ x: 125, y: 125, r: 25 });
  const targetPosRef = useRef({ x: 210, y: 125 });
  const activeAvoidPoseRef = useRef({ x: 40, y: 125, theta: 0 });
  const avoidPathRef = useRef<{ x: number; y: number }[]>([]);

  // 5. Edge MPC & ROS2 DDS State variables
  const [mpcLatency, setMpcLatency] = useState(2.4); // ms, target < 5ms
  const [recalculatingMpc, setRecalculatingMpc] = useState(false);
  const [jointTorques, setJointTorques] = useState([42.1, 58.4, 89.2, 22.1, 14.8, 8.5]); // Nm
  const [predictiveMaintenanceActive, setPredictiveMaintenanceActive] = useState(true);
  const [taskReroutingStatus, setTaskReroutingStatus] = useState("Nominal - All Systems Green");
  const [ddsQosReliability, setDdsQosReliability] = useState<"RELIABLE" | "BEST_EFFORT">("RELIABLE");
  const [ddsQosDurability, setDdsQosDurability] = useState<"TRANSIENT_LOCAL" | "VOLATILE">("TRANSIENT_LOCAL");
  const [packetLossPercent, setPacketLossPercent] = useState(0.00); // % loss
  const [edgeMpcSimRunning, setEdgeMpcSimRunning] = useState(false);

  // Speaking announcer
  const triggerVoiceGreeting = (tab: string) => {
    if (!speakText) return;
    switch(tab) {
      case "kinematics":
        speakText("Displaying differential drive kinematics lab. Adjust wheel coordinates to compute angular trajectories, Boss.");
        break;
      case "localization":
        speakText("Initiating particle filter state localization. Demonstrating probabilistic Monte Carlo resampling and sensor drift convergence.");
        break;
      case "planning":
        speakText("Opening autonomous path planning grid. Toggle between grid-based A-star routing and boundary-following bug algorithms.");
        break;
      case "avoidance":
        speakText("Rendering potential fields reactive avoidance simulator. Actively resolving vector force vectors from target attraction and obstacle density.");
        break;
      case "edge_mpc":
        speakText("Accessing Edge-based Model Predictive Control and standardized ROS2 DDS configuration nodes. Standing by to deploy sub-5ms localized trajectory recalculations, Boss.");
        break;
    }
  };

  useEffect(() => {
    triggerVoiceGreeting(activeTab);
  }, [activeTab]);

  // ==========================================
  // 1. KINEMATICS ENGINE / CANVAS LOOPS
  // ==========================================
  useEffect(() => {
    if (activeTab !== "kinematics") return;
    const canvas = kinCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animFrame: number;
    let lastTime = performance.now();

    const draw = (time: number) => {
      const dt = Math.min((time - lastTime) / 1000, 0.1); // cap dt at 100ms
      lastTime = time;

      ctx.fillStyle = "#020306";
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      // Drawing Grid lines
      ctx.strokeStyle = "#111827";
      ctx.lineWidth = 1;
      for (let i = 0; i < canvas.width; i += 25) {
        ctx.beginPath();
        ctx.moveTo(i, 0);
        ctx.lineTo(i, canvas.height);
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(0, i);
        ctx.lineTo(canvas.width, i);
        ctx.stroke();
      }

      // Update differential drive physics if running
      if (kinSimRunning) {
        const pose = kinPoseRef.current;
        const R = wheelRadius;
        const L = trackWidth;
        const wL = wheelLeft;
        const wR = wheelRight;

        // Linear velocity (v) and angular velocity (w)
        const v = (R * (wR + wL)) / 2.0;
        const w = (R * (wR - wL)) / L;

        // Runge-Kutta integration
        pose.theta += w * dt;
        pose.x += v * Math.cos(pose.theta) * dt * 10; // multiplier to scale view
        pose.y += v * Math.sin(pose.theta) * dt * 10;

        // Wrap or rebound off walls
        if (pose.x < 10) { pose.x = 10; pose.theta = Math.PI - pose.theta; }
        if (pose.x > canvas.width - 10) { pose.x = canvas.width - 10; pose.theta = Math.PI - pose.theta; }
        if (pose.y < 10) { pose.y = 10; pose.theta = -pose.theta; }
        if (pose.y > canvas.height - 10) { pose.y = canvas.height - 10; pose.theta = -pose.theta; }

        kinPathRef.current.push({ x: pose.x, y: pose.y });
        if (kinPathRef.current.length > 300) kinPathRef.current.shift();
      }

      // Draw Path Traced
      if (kinPathRef.current.length > 1) {
        ctx.beginPath();
        ctx.strokeStyle = "rgba(14, 165, 233, 0.45)";
        ctx.lineWidth = 1.5;
        ctx.moveTo(kinPathRef.current[0].x, kinPathRef.current[0].y);
        for (let j = 1; j < kinPathRef.current.length; j++) {
          ctx.lineTo(kinPathRef.current[j].x, kinPathRef.current[j].y);
        }
        ctx.stroke();
      }

      // Draw Robot body
      const robot = kinPoseRef.current;
      ctx.save();
      ctx.translate(robot.x, robot.y);
      ctx.rotate(robot.theta);

      // Chassis
      ctx.fillStyle = "rgba(15, 23, 42, 0.85)";
      ctx.strokeStyle = "#38bdf8";
      ctx.lineWidth = 2;
      ctx.beginPath();
      // Circle Chassis
      ctx.arc(0, 0, 14, 0, 2 * Math.PI);
      ctx.fill();
      ctx.stroke();

      // Heading indicator line / pointer
      ctx.strokeStyle = "#f39c12";
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(0, 0);
      ctx.lineTo(18, 0);
      ctx.stroke();

      // Wheels
      ctx.fillStyle = "#ef4444";
      ctx.fillRect(-6, -16, 12, 4); // Left wheel
      ctx.fillStyle = "#10b981";
      ctx.fillRect(-6, 12, 12, 4);  // Right wheel

      ctx.restore();

      // Instantaneous Center of Curvature (ICC) estimation display text
      ctx.fillStyle = "rgba(156, 163, 175, 0.7)";
      ctx.font = "8px monospace";
      const wL = wheelLeft;
      const wR = wheelRight;
      if (Math.abs(wL - wR) < 0.01) {
        ctx.fillText("ICC: INFINITE (STRAIGHT TRAJECTORY)", 8, canvas.height - 8);
      } else {
        const R = wheelRadius;
        const L = trackWidth;
        // ICC Radius to center of base: R_icc = L/2 * (w_R + w_L)/(w_R - w_L)
        const R_icc = (L / 2.0) * (wR + wL) / (wR - wL);
        ctx.fillText(`ICC Radius: ${R_icc.toFixed(2)} cm`, 8, canvas.height - 8);
      }

      animFrame = requestAnimationFrame(draw);
    };

    animFrame = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(animFrame);
  }, [activeTab, kinSimRunning, wheelLeft, wheelRight, wheelRadius, trackWidth]);

  // ==========================================
  // 2. LOCALIZATION ENGINE & PARTICLE SYSTEM
  // ==========================================
  const initParticles = () => {
    const arr: { x: number; y: number; weight: number }[] = [];
    const basePose = odometryPoseRef.current;
    for (let i = 0; i < particleCount; i++) {
      // Scatter uniformly first, or Gaussian around true position
      arr.push({
        x: basePose.x + (Math.random() - 0.5) * 120,
        y: basePose.y + (Math.random() - 0.5) * 120,
        weight: 1.0 / particleCount
      });
    }
    particlesRef.current = arr;
  };

  useEffect(() => {
    if (activeTab !== "localization") return;
    initParticles();

    const canvas = locCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animFrame: number;
    let lastTime = performance.now();
    let moveAngle = 0;

    const draw = (time: number) => {
      const dt = Math.min((time - lastTime) / 1000, 0.1);
      lastTime = time;

      ctx.fillStyle = "#020306";
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      // Draw Grid
      ctx.strokeStyle = "#0f172a";
      ctx.lineWidth = 1;
      for (let i = 0; i < canvas.width; i += 50) {
        ctx.beginPath();
        ctx.moveTo(i, 0);
        ctx.lineTo(i, canvas.height);
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(0, i);
        ctx.lineTo(canvas.width, i);
        ctx.stroke();
      }

      // Draw Landmarks
      landmarks.forEach((lm) => {
        ctx.fillStyle = lm.color;
        ctx.beginPath();
        ctx.arc(lm.x, lm.y, 6, 0, 2 * Math.PI);
        ctx.fill();
        ctx.strokeStyle = "#ffffff";
        ctx.lineWidth = 1.5;
        ctx.stroke();

        ctx.fillStyle = "#fff";
        ctx.font = "bold 8px monospace";
        ctx.fillText(`LM-${lm.id}`, lm.x - 11, lm.y - 10);
      });

      if (locSimRunning) {
        // Move robot in smooth circular motion around center
        moveAngle += 0.8 * dt;
        const centerX = 125, centerY = 125, radius = 70;
        
        const oldTruePose = { ...robotPoseRef.current };
        robotPoseRef.current.x = centerX + Math.cos(moveAngle) * radius;
        robotPoseRef.current.y = centerY + Math.sin(moveAngle) * radius;

        // Motion displacement vector
        const dx = robotPoseRef.current.x - oldTruePose.x;
        const dy = robotPoseRef.current.y - oldTruePose.y;

        // 1. Prediction Step (Odometry Update with sensorNoise / drift)
        const driftX = dx + (Math.random() - 0.5) * sensorNoise * 0.45;
        const driftY = dy + (Math.random() - 0.5) * sensorNoise * 0.45;
        odometryPoseRef.current.x += driftX;
        odometryPoseRef.current.y += driftY;

        // Propagate motion to all particles with noise expansion
        particlesRef.current.forEach((p) => {
          p.x += dx + (Math.random() - 0.5) * sensorNoise * 1.5;
          p.y += dy + (Math.random() - 0.5) * sensorNoise * 1.5;
        });

        // 2. Correction Step (Measurement Likelihood updating weights based on landmark distance checks)
        // Measure real distances
        const trueDistances = landmarks.map((lm) => {
          return Math.hypot(robotPoseRef.current.x - lm.x, robotPoseRef.current.y - lm.y);
        });

        // Calculate weights
        let totalWeight = 0;
        particlesRef.current.forEach((p) => {
          let likelihood = 1.0;
          landmarks.forEach((lm, idx) => {
            const pDist = Math.hypot(p.x - lm.x, p.y - lm.y);
            const error = pDist - trueDistances[idx];
            // Gaussian likelihood distribution
            const variance = sensorNoise * sensorNoise + 3.0;
            likelihood *= Math.exp(-(error * error) / (2.0 * variance)) / Math.sqrt(2 * Math.PI * variance);
          });
          p.weight = likelihood;
          totalWeight += likelihood;
        });

        // Normalize weights and Resample (Low Variance Sampling helper)
        if (totalWeight > 0) {
          particlesRef.current.forEach((p) => p.weight /= totalWeight);
          
          // Resampling selection
          const newParticles: typeof particlesRef.current = [];
          const N = particlesRef.current.length;
          let index = Math.floor(Math.random() * N);
          let beta = 0.0;
          let maxW = Math.max(...particlesRef.current.map(p => p.weight));

          for (let i = 0; i < N; i++) {
            beta += Math.random() * 2.0 * maxW;
            while (beta > particlesRef.current[index].weight) {
              beta -= particlesRef.current[index].weight;
              index = (index + 1) % N;
            }
            // Add resampled particle with minor perturbation
            newParticles.push({
              x: particlesRef.current[index].x + (Math.random() - 0.5) * 1.2,
              y: particlesRef.current[index].y + (Math.random() - 0.5) * 1.2,
              weight: 1.0 / N
            });
          }
          particlesRef.current = newParticles;
        } else {
          // Fallback to random distribution near odometry
          initParticles();
        }
      }

      // Draw Particles
      particlesRef.current.forEach((p) => {
        ctx.fillStyle = "rgba(249, 115, 22, 0.65)"; // Orange dot
        ctx.beginPath();
        ctx.arc(p.x, p.y, 2, 0, 2 * Math.PI);
        ctx.fill();
      });

      // Draw Dead-Reckoning (Odometry Pose) as highly drifted red hollow circle
      ctx.strokeStyle = "rgba(239, 68, 68, 0.75)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(odometryPoseRef.current.x, odometryPoseRef.current.y, 8, 0, 2 * Math.PI);
      ctx.stroke();
      ctx.fillStyle = "rgba(239, 68, 68, 0.2)";
      ctx.fill();
      ctx.font = "8px monospace";
      ctx.fillStyle = "#f87171";
      ctx.fillText("Odometry (Drifting)", odometryPoseRef.current.x + 10, odometryPoseRef.current.y + 3);

      // Draw True Robot Pose as striking solid emerald circle with target heading
      ctx.fillStyle = "#10b981";
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.arc(robotPoseRef.current.x, robotPoseRef.current.y, 7, 0, 2 * Math.PI);
      ctx.fill();
      ctx.stroke();
      ctx.font = "9px monospace";
      ctx.fillStyle = "#34d399";
      ctx.fillText("True Location", robotPoseRef.current.x + 10, robotPoseRef.current.y - 4);

      // Highlight sensor ranging lines to landmarks
      if (locSimRunning) {
        ctx.strokeStyle = "rgba(16, 185, 129, 0.15)";
        ctx.lineWidth = 1;
        landmarks.forEach((lm) => {
          ctx.beginPath();
          ctx.moveTo(robotPoseRef.current.x, robotPoseRef.current.y);
          ctx.lineTo(lm.x, lm.y);
          ctx.stroke();
        });
      }

      animFrame = requestAnimationFrame(draw);
    };

    animFrame = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(animFrame);
  }, [activeTab, locSimRunning, sensorNoise]);

  // ==========================================
  // 3. GRID PATH PLANNING GENERATOR (A*)
  // ==========================================
  const initPlanningGrid = () => {
    const tempGrid: CellType[][] = Array.from({ length: gridSize }, () => 
      Array(gridSize).fill("empty")
    );

    // Set obstacles (a couple of walls matching canonical path problem)
    for (let c = 4; c <= 10; c++) {
      tempGrid[5][c] = "obstacle";
    }
    for (let r = 8; r <= 13; r++) {
      tempGrid[r][7] = "obstacle";
    }

    tempGrid[startPos.r][startPos.c] = "start";
    tempGrid[goalPos.r][goalPos.c] = "goal";

    setGrid(tempGrid);
    setPlanningSteps(0);
  };

  useEffect(() => {
    if (activeTab === "planning") {
      initPlanningGrid();
    }
  }, [activeTab, startPos, goalPos]);

  // Simplified A* Pathfinding animation engine
  const runPathFinding = async () => {
    if (planningActive) return;
    setPlanningActive(true);
    
    if (speakText) {
      speakText(`Searching grid routes using J.A.R.V.I.S. pathfinder. Applying heuristic priority queues, Boss.`);
    }

    // Reset grid representation
    const freshGrid: CellType[][] = Array.from({ length: gridSize }, () => Array(gridSize).fill("empty"));
    for (let c = 4; c <= 10; c++) freshGrid[5][c] = "obstacle";
    for (let r = 8; r <= 13; r++) freshGrid[r][7] = "obstacle";
    freshGrid[startPos.r][startPos.c] = "start";
    freshGrid[goalPos.r][goalPos.c] = "goal";

    // Priority Queue item interface: row, col, f_score, g_score, parent
    type Node = { r: number; c: number; f: number; g: number; parent: Node | null };
    
    const openList: Node[] = [{ r: startPos.r, c: startPos.c, f: 0, g: 0, parent: null }];
    const closedSet = new Set<string>();

    const h = (r: number, c: number) => {
      // Manhattan distance heuristic
      return Math.abs(r - goalPos.r) + Math.abs(c - goalPos.c);
    };

    let steps = 0;
    while (openList.length > 0) {
      steps++;
      // Sort to get node with lowest f_score
      openList.sort((a, b) => a.f - b.f);
      const current = openList.shift()!;
      const key = `${current.r},${current.c}`;

      if (closedSet.has(key)) continue;
      closedSet.add(key);

      // Animation speed throttle
      await new Promise((resolve) => setTimeout(resolve, 35));

      // Visualise progress
      if (!(current.r === startPos.r && current.c === startPos.c) && !(current.r === goalPos.r && current.c === goalPos.c)) {
        freshGrid[current.r][current.c] = "visited";
        setGrid([...freshGrid.map(row => [...row])]);
        setPlanningSteps(steps);
      }

      // Check if reached Goal
      if (current.r === goalPos.r && current.c === goalPos.c) {
        // Backtrack path
        let pathNode = current.parent;
        while (pathNode && !(pathNode.r === startPos.r && pathNode.c === startPos.c)) {
          freshGrid[pathNode.r][pathNode.c] = "path";
          pathNode = pathNode.parent;
        }
        setGrid([...freshGrid.map(row => [...row])]);
        setPlanningActive(false);

        if (speakText) {
          speakText(`Converged optimized path with heuristic cost score. Waypoint calculation committed successfully.`);
        }
        return;
      }

      // 4-connected grid neighbors
      const directions = [
        { dr: -1, dc: 0 },
        { dr: 1, dc: 0 },
        { dr: 0, dc: -1 },
        { dr: 0, dc: 1 }
      ];

      for (const dir of directions) {
        const nr = current.r + dir.dr;
        const nc = current.c + dir.dc;

        if (nr >= 0 && nr < gridSize && nc >= 0 && nc < gridSize) {
          if (freshGrid[nr][nc] === "obstacle" || closedSet.has(`${nr},${nc}`)) continue;

          const gScore = current.g + 1;
          const hScore = h(nr, nc);
          const fScore = gScore + hScore;

          // Check if already in openList with better score
          const existing = openList.find(node => node.r === nr && node.c === nc);
          if (!existing || gScore < existing.g) {
            if (existing) {
              existing.g = gScore;
              existing.f = fScore;
              existing.parent = current;
            } else {
              openList.push({ r: nr, c: nc, f: fScore, g: gScore, parent: current });
            }
          }
        }
      }
    }

    setPlanningActive(false);
    if (speakText) {
      speakText(`Pathplanning grid traversal failed Boss. No convergent pathway detected.`);
    }
  };

  // Bug algorithm path planner simulation
  const simulateBugAlgo = async () => {
    if (planningActive) return;
    setPlanningActive(true);

    if (speakText) {
      speakText(`Initiating Bug-2 obstacle-contour navigation mode, Boss.`);
    }

    const freshGrid: CellType[][] = Array.from({ length: gridSize }, () => Array(gridSize).fill("empty"));
    for (let c = 4; c <= 10; c++) freshGrid[5][c] = "obstacle";
    for (let r = 8; r <= 13; r++) freshGrid[r][7] = "obstacle";
    freshGrid[startPos.r][startPos.c] = "start";
    freshGrid[goalPos.r][goalPos.c] = "goal";

    let cr = startPos.r;
    let cc = startPos.c;
    let steps = 0;
    const maxSteps = 100;

    // Line from start to goal is modeled. In Bug, robot tracks this line, and if hits obstacle, outlines it.
    // For visual simulation, let's trace a line directly, contour clockwise when hitting obstacles.
    while (steps < maxSteps) {
      steps++;
      await new Promise((resolve) => setTimeout(resolve, 90));

      if (cr === goalPos.r && cc === goalPos.c) {
        setPlanningActive(false);
        if (speakText) {
          speakText(`Bug traversal converged successfully at coordinates.`);
        }
        return;
      }

      // Determine ideal direction to goal
      const diffR = goalPos.r - cr;
      const diffC = goalPos.c - cc;

      let nextR = cr + (diffR !== 0 ? Math.sign(diffR) : 0);
      let nextC = cc + (diffC !== 0 ? Math.sign(diffC) : 0);

      // If next is obstacle, trace around it
      if (freshGrid[nextR][nextC] === "obstacle" || (diffR !== 0 && freshGrid[cr + Math.sign(diffR)][cc] === "obstacle")) {
        // Step to side (outline clockwise)
        if (cc < gridSize - 1 && freshGrid[cr][cc + 1] !== "obstacle") {
          cc += 1;
        } else if (cr < gridSize - 1 && freshGrid[cr + 1][cc] !== "obstacle") {
          cr += 1;
        } else if (cc > 0 && freshGrid[cr][cc - 1] !== "obstacle") {
          cc -= 1;
        } else {
          cr -= 1;
        }
      } else {
        // Safe step
        cr = nextR;
        cc = nextC;
      }

      if (!(cr === startPos.r && cc === startPos.c) && !(cr === goalPos.r && cc === goalPos.c)) {
        freshGrid[cr][cc] = "path";
        setGrid([...freshGrid.map(row => [...row])]);
        setPlanningSteps(steps);
      }
    }

    setPlanningActive(false);
  };


  // ==========================================
  // 4. POTENTIAL FIELDS & LIDAR SCAN OBSTACLE AVOIDANCE
  // ==========================================
  useEffect(() => {
    if (activeTab !== "avoidance") return;
    const canvas = avoidCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animFrame: number;
    let lastTime = performance.now();

    const draw = (time: number) => {
      const dt = Math.min((time - lastTime) / 1000, 0.1);
      lastTime = time;

      ctx.fillStyle = "#020306";
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      // Draw Grid
      ctx.strokeStyle = "#0d1424";
      ctx.lineWidth = 1;
      for (let i = 0; i < canvas.width; i += 25) {
        ctx.beginPath();
        ctx.moveTo(i, 0);
        ctx.lineTo(i, canvas.height);
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(0, i);
        ctx.lineTo(canvas.width, i);
        ctx.stroke();
      }

      // Draw Target Location
      const target = targetPosRef.current;
      ctx.fillStyle = "#10b981";
      ctx.beginPath();
      ctx.arc(target.x, target.y, 6, 0, 2 * Math.PI);
      ctx.fill();
      ctx.font = "bold 8px monospace";
      ctx.fillText("TARGET GOAL", target.x - 22, target.y - 10);

      // Draw Obstacle (Repulsive source)
      const obs = obstaclePosRef.current;
      const grad = ctx.createRadialGradient(obs.x, obs.y, 5, obs.x, obs.y, obs.r + 20);
      grad.addColorStop(0, "rgba(239, 68, 68, 0.45)");
      grad.addColorStop(1, "rgba(239, 68, 68, 0.0)");
      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.arc(obs.x, obs.y, obs.r + 20, 0, 2 * Math.PI);
      ctx.fill();

      ctx.fillStyle = "#ef4444";
      ctx.beginPath();
      ctx.arc(obs.x, obs.y, obs.r, 0, 2 * Math.PI);
      ctx.fill();
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = 1;
      ctx.stroke();
      ctx.fillStyle = "#ffffff";
      ctx.fillText("OBSTACLE", obs.x - 20, obs.y + 3);

      // Potential Field Control Vector physics
      const robot = activeAvoidPoseRef.current;
      const speed = avoidSpeed;

      // 1. Attraction Vector (F_att = k_att * (Target_pos - Robot_pos))
      const dxTarget = target.x - robot.x;
      const dyTarget = target.y - robot.y;
      const distTarget = Math.hypot(dxTarget, dyTarget);
      
      const kAtt = 0.5;
      let fX_att = kAtt * (dxTarget / (distTarget || 1)) * 30;
      let fY_att = kAtt * (dyTarget / (distTarget || 1)) * 30;

      // 2. Repulsion Vector (F_rep = k_rep * (1/d - 1/rho_0) * (1/d^2) * (Robot_pos - Obs_pos))
      const dxObs = robot.x - obs.x;
      const dyObs = robot.y - obs.y;
      const distObs = Math.hypot(dxObs, dyObs);
      const threshold = obs.r + 30; // activation distance field

      let fX_rep = 0;
      let fY_rep = 0;
      if (distObs < threshold) {
        const kRep = 40.0;
        const repForce = kRep * (1.0 / Math.max(distObs - obs.r, 1.0) - 1.0 / (threshold - obs.r));
        fX_rep = (dxObs / distObs) * repForce * 15;
        fY_rep = (dyObs / distObs) * repForce * 15;
      }

      // Total Forces
      const netX = fX_att + fX_rep;
      const netY = fY_att + fY_rep;

      if (avoidSimRunning) {
        // Integrate forces into velocity
        robot.theta = Math.atan2(netY, netX);
        robot.x += Math.cos(robot.theta) * speed * dt * 25;
        robot.y += Math.sin(robot.theta) * speed * dt * 25;

        // If very close to target, reset to start loop
        if (distTarget < 10) {
          robot.x = 40;
          robot.y = 125 + (Math.random() - 0.5) * 50; // scatter starting lane
          avoidPathRef.current = [];
        }

        avoidPathRef.current.push({ x: robot.x, y: robot.y });
        if (avoidPathRef.current.length > 250) avoidPathRef.current.shift();
      }

      // Draw trailing path
      if (avoidPathRef.current.length > 1) {
        ctx.beginPath();
        ctx.strokeStyle = "rgba(16, 185, 129, 0.4)";
        ctx.lineWidth = 1.5;
        ctx.moveTo(avoidPathRef.current[0].x, avoidPathRef.current[0].y);
        for (let j = 1; j < avoidPathRef.current.length; j++) {
          ctx.lineTo(avoidPathRef.current[j].x, avoidPathRef.current[j].y);
        }
        ctx.stroke();
      }

      // Simulated LIDAR Scanning beams
      const beamCount = 36;
      ctx.strokeStyle = "rgba(37, 99, 235, 0.1)"; 
      ctx.lineWidth = 0.5;
      for (let b = 0; b < beamCount; b++) {
        const phi = (b * (2 * Math.PI)) / beamCount;
        const beamMax = 45;
        
        // Calculate collision intersection of beam with circular obstacle
        let reach = beamMax;
        // Ray equation: rx(s) = robot.x + s*cos(phi), ry(s) = robot.y + s*sin(phi)
        // distance to obs.x, obs.y equals obs.r
        // (rx(s) - obs.x)^2 + (ry(s) - obs.y)^2 = r^2 -> quadratic equation in s
        const ex = robot.x - obs.x;
        const ey = robot.y - obs.y;
        const cosPhi = Math.cos(phi);
        const sinPhi = Math.sin(phi);

        const A_quad = 1;
        const B_quad = 2 * (ex * cosPhi + ey * sinPhi);
        const C_quad = ex * ex + ey * ey - obs.r * obs.r;
        const discr = B_quad * B_quad - 4 * A_quad * C_quad;

        if (discr >= 0) {
          const s1 = (-B_quad - Math.sqrt(discr)) / (2 * A_quad);
          if (s1 > 0 && s1 < beamMax) {
            reach = s1;
          }
        }

        ctx.beginPath();
        ctx.moveTo(robot.x, robot.y);
        ctx.lineTo(robot.x + cosPhi * reach, robot.y + sinPhi * reach);
        ctx.stroke();

        // If hitting obstacle, add small particle bounce point
        if (reach < beamMax) {
          ctx.fillStyle = "rgba(220, 38, 38, 0.7)";
          ctx.beginPath();
          ctx.arc(robot.x + cosPhi * reach, robot.y + sinPhi * reach, 1.5, 0, 2 * Math.PI);
          ctx.fill();
        }
      }

      // Draw control vectors (Arrow vectors over robot)
      if (avoidSimRunning) {
        ctx.lineWidth = 1.5;

        // Attraction vector (Green Arrow)
        ctx.strokeStyle = "#10b981";
        ctx.beginPath();
        ctx.moveTo(robot.x, robot.y);
        ctx.lineTo(robot.x + fX_att, robot.y + fY_att);
        ctx.stroke();

        // Repulsion vector (Red Arrow)
        if (Math.hypot(fX_rep, fY_rep) > 0.5) {
          ctx.strokeStyle = "#ef4444";
          ctx.beginPath();
          ctx.moveTo(robot.x, robot.y);
          ctx.lineTo(robot.x + fX_rep, robot.y + fY_rep);
          ctx.stroke();
        }

        // Output Result vector (Blue Arrow)
        ctx.strokeStyle = "#38bdf8";
        ctx.beginPath();
        ctx.moveTo(robot.x, robot.y);
        ctx.lineTo(robot.x + netX, robot.y + netY);
        ctx.stroke();
      }

      // Draw Robot Body
      ctx.save();
      ctx.translate(robot.x, robot.y);
      ctx.rotate(robot.theta);
      ctx.fillStyle = "#ffffff";
      ctx.strokeStyle = "#2563eb";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(0, 0, 7, 0, 2 * Math.PI);
      ctx.fill();
      ctx.stroke();
      // Arrow indicator inside robot
      ctx.fillStyle = "#1e40af";
      ctx.beginPath();
      ctx.moveTo(5, 0);
      ctx.lineTo(-4, -4);
      ctx.lineTo(-4, 4);
      ctx.closePath();
      ctx.fill();
      ctx.restore();

      animFrame = requestAnimationFrame(draw);
    };

    animFrame = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(animFrame);
  }, [activeTab, avoidSimRunning, avoidSpeed]);


  // ==========================================
  // 5. EDGE MPC & JOINT-TORQUE MONITORING LOOP
  // ==========================================
  useEffect(() => {
    if (activeTab !== "edge_mpc" || !edgeMpcSimRunning) return;
    
    const interval = setInterval(() => {
      // Simulate sub-5ms MPC calculations slightly fluctuating
      setMpcLatency(parseFloat((1.8 + Math.random() * 1.5).toFixed(2)));

      // Simulate real-time joint-torque monitoring
      setJointTorques(prev => {
        const next = [...prev];
        // Add slight physical jitter to Joint 1 - 6
        for(let i = 0; i < 6; i++) {
          next[i] = parseFloat(Math.max(5, Math.min(150, next[i] + (Math.random() - 0.5) * 8)).toFixed(1));
        }
        
        // Trigger simulated predictive maintenance re-routing if joint 3 exceeds load threshold (115 Nm)
        if (next[2] > 115) {
          setTaskReroutingStatus("WARNING: Joint 3 load threshold exceeded! Actively re-routing trajectory to secondary manipulator.");
          if (speakText && Math.random() > 0.85) {
            speakText("Warning, joint 3 torque load threshold exceeded. Initiating dynamic task re-routing protocols.");
          }
        } else {
          setTaskReroutingStatus("Nominal - Edge MPC Balancing Trajectories");
        }
        return next;
      });

      // Update simulated packet loss based on QoS settings
      if (ddsQosReliability === "RELIABLE" && ddsQosDurability === "TRANSIENT_LOCAL") {
        setPacketLossPercent(0.00); // Perfect QoS
      } else {
        setPacketLossPercent(parseFloat((Math.random() * 2.8).toFixed(2))); // Drifting packet loss
      }

    }, 350);

    return () => clearInterval(interval);
  }, [activeTab, edgeMpcSimRunning, ddsQosReliability, ddsQosDurability]);

  const triggerMpcRecalculation = () => {
    setRecalculatingMpc(true);
    if (speakText) {
      speakText("Initiating edge-based model predictive control trajectory recalculation.");
    }
    setTimeout(() => {
      setRecalculatingMpc(false);
      setMpcLatency(parseFloat((1.5 + Math.random() * 0.8).toFixed(2)));
      if (speakText) {
        speakText("Recalculation complete. Localized trajectory converged in 2.1 milliseconds.");
      }
    }, 600);
  };


  return (
    <div className="space-y-4 animate-fadeIn text-gray-300 font-sans">
      {/* Header Panel */}
      <div className="border-b border-gray-900 pb-2.5 flex justify-between items-center">
        <p className="text-[10px] text-blue-400 font-bold uppercase tracking-widest flex items-center gap-1.5">
          <Bot className="w-3.5 h-3.5 text-blue-400 animate-pulse" />
          <span>Autonomous Robotics Simulation Lab</span>
        </p>
        <span className="text-[8px] font-mono text-amber-500 uppercase bg-amber-950/45 border border-amber-900/60 px-1.5 py-0.5 rounded font-bold">
          CORELL_TEXTBOOK_SANDBOX
        </span>
      </div>

      {/* Course Context Header */}
      <div className="bg-gradient-to-r from-blue-950/20 to-black/40 border border-gray-900/80 rounded p-3 text-[10px] leading-relaxed text-gray-400 font-mono flex items-start gap-2.5 shadow-md">
        <Info className="w-4 h-4 text-blue-400 shrink-0 mt-0.5" />
        <div className="space-y-1">
          <div className="text-gray-200 uppercase font-semibold text-[11px] tracking-wider">
            Introduction to Autonomous Robots Sandbox
          </div>
          <p className="leading-normal">
            This workspace dynamically simulates the mathematical, probabilistic, and algorithmic paradigms defined by 
            <strong> Nikolaus Correll, Bradley Hayes, and Alessandro Roncone</strong>. Toggle sub-modules below to run active kinematic, localization, pathfinding, or collision avoidance tests in real-time.
          </p>
        </div>
      </div>

      {/* Simulator Selector Headers */}
      <div className="flex bg-[#030508] border border-gray-900 rounded-lg p-0.5 text-[9px] uppercase font-mono font-bold">
        {[
          { id: "kinematics", label: "Wheel Kinematics", icon: Compass },
          { id: "localization", label: "Particle filter", icon: Eye },
          { id: "planning", label: "Path Planning grid", icon: Map },
          { id: "avoidance", label: "Potential fields", icon: Network },
          { id: "edge_mpc", label: "Edge MPC & ROS2 DDS", icon: Cpu }
        ].map((tab) => {
          const IconComp = tab.icon;
          const isSelected = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`flex-1 py-1.5 rounded-md flex items-center justify-center gap-1.5 transition-all cursor-pointer ${
                isSelected 
                  ? "bg-blue-600 font-bold text-white shadow-md shadow-blue-900/30" 
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              <IconComp className="w-3 h-3 shrink-0" />
              <span className="hidden sm:inline">{tab.label}</span>
            </button>
          );
        })}
      </div>

      {/* Main Grid: Control & Visual canvas */}
      <div className="grid grid-cols-1 md:grid-cols-12 gap-3.5">
        
        {/* Left Side: Parameters sliders */}
        <div className="md:col-span-5 bg-[#05070a] border border-gray-900 rounded-lg p-3.5 space-y-3.5 select-none flex flex-col justify-between">
          
          {/* TAB 1 CONTENT: DIFFERENTIAL DRIVE PARAMS */}
          {activeTab === "kinematics" && (
            <div className="space-y-3">
              <label className="text-[10px] text-blue-400 font-bold uppercase tracking-wider block font-mono pb-1 border-b border-gray-950">
                Chassis Geometry & Speeds
              </label>

              <div className="space-y-3.5 text-xs font-mono">
                {/* Left Wheel velocity */}
                <div className="space-y-1">
                  <div className="flex justify-between items-center text-[10px]">
                    <span className="text-gray-500 uppercase">Left Wheel Angular Speed (ωL)</span>
                    <span className="text-red-400 font-bold">{wheelLeft.toFixed(1)} rad/s</span>
                  </div>
                  <input
                    type="range"
                    min={-6}
                    max={6}
                    step={0.2}
                    value={wheelLeft}
                    onChange={(e) => setWheelLeft(parseFloat(e.target.value))}
                    className="w-full h-1 bg-black rounded outline-none accent-red-500"
                  />
                </div>

                {/* Right Wheel velocity */}
                <div className="space-y-1">
                  <div className="flex justify-between items-center text-[10px]">
                    <span className="text-gray-500 uppercase">Right Wheel Angular Speed (ωR)</span>
                    <span className="text-emerald-400 font-bold">{wheelRight.toFixed(1)} rad/s</span>
                  </div>
                  <input
                    type="range"
                    min={-6}
                    max={6}
                    step={0.2}
                    value={wheelRight}
                    onChange={(e) => setWheelRight(parseFloat(e.target.value))}
                    className="w-full h-1 bg-black rounded outline-none accent-emerald-500"
                  />
                </div>

                {/* Wheel Diameter */}
                <div className="space-y-1 pt-1.5 border-t border-gray-950">
                  <div className="flex justify-between items-center text-[10px]">
                    <span className="text-gray-500 uppercase">Wheel Radius (R)</span>
                    <span className="text-gray-300 font-bold">{wheelRadius.toFixed(2)} cm</span>
                  </div>
                  <input
                    type="range"
                    min={0.5}
                    max={3.0}
                    step={0.1}
                    value={wheelRadius}
                    onChange={(e) => setWheelRadius(parseFloat(e.target.value))}
                    className="w-full h-1 bg-black rounded outline-none accent-blue-500"
                  />
                </div>

                {/* Track Width */}
                <div className="space-y-1">
                  <div className="flex justify-between items-center text-[10px]">
                    <span className="text-gray-500 uppercase">Track Width (L)</span>
                    <span className="text-gray-300 font-bold">{trackWidth.toFixed(1)} cm</span>
                  </div>
                  <input
                    type="range"
                    min={3.0}
                    max={12.0}
                    step={0.5}
                    value={trackWidth}
                    onChange={(e) => setTrackWidth(parseFloat(e.target.value))}
                    className="w-full h-1 bg-black rounded outline-none accent-blue-500"
                  />
                </div>
              </div>

              <div className="pt-3 border-t border-gray-950 text-[10px] leading-relaxed text-gray-500 font-mono space-y-1">
                <span className="text-amber-500 uppercase font-bold text-[8px] tracking-widest block">Chapter 3 Kinematics Formula</span>
                <p>v_linear = R * (ωR + ωL) / 2</p>
                <p>ω_angular = R * (ωR - ωL) / L</p>
              </div>
            </div>
          )}

          {/* TAB 2 CONTENT: LOCALIZATION DRIVER */}
          {activeTab === "localization" && (
            <div className="space-y-3">
              <label className="text-[10px] text-blue-400 font-bold uppercase tracking-wider block font-mono pb-1 border-b border-gray-950">
                Monte Carlo Filtering Drivers
              </label>

              <div className="space-y-3.5 text-xs font-mono">
                {/* Noise parameters */}
                <div className="space-y-1">
                  <div className="flex justify-between items-center text-[10px]">
                    <span className="text-gray-500 uppercase">Sensor Standard Deviation (σ)</span>
                    <span className="text-blue-400 font-bold">{sensorNoise.toFixed(1)} cm</span>
                  </div>
                  <input
                    type="range"
                    min={0.5}
                    max={8.0}
                    step={0.5}
                    value={sensorNoise}
                    onChange={(e) => setSensorNoise(parseFloat(e.target.value))}
                    className="w-full h-1 bg-black rounded outline-none accent-blue-500"
                  />
                </div>

                <div className="bg-black/40 border border-gray-950 p-2.5 rounded font-mono text-[9px] text-gray-400 space-y-1.5 leading-snug">
                  <span className="text-orange-400 font-bold text-[8px] uppercase block">Resampling Loop Details:</span>
                  <p>1. **Predict**: Push particles forward based on circular displacement vector with added noise expansion.</p>
                  <p>2. **Measure**: Query LIDAR sensor distances to Landmarks A, B, C, D.</p>
                  <p>3. **Correct**: Weight particles via Gaussian probability density function. Resample with replacement.</p>
                </div>
              </div>

              <div className="pt-3 border-t border-gray-950 text-[10px] leading-relaxed text-gray-500 font-mono space-y-1">
                <span className="text-amber-500 uppercase font-bold text-[8px] tracking-widest block">Chapter 6 Probability</span>
                <p>p(x_t | u_t, x_(t-1)) ~ N(mean, covariance)</p>
              </div>
            </div>
          )}

          {/* TAB 3 CONTENT: PATH PLANNING GRID SELECTOR */}
          {activeTab === "planning" && (
            <div className="space-y-3">
              <label className="text-[10px] text-blue-400 font-bold uppercase tracking-wider block font-mono pb-1 border-b border-gray-950">
                Planning Algorithm configuration
              </label>

              <div className="space-y-3 font-mono">
                {/* Method selector */}
                <div className="grid grid-cols-2 gap-2 text-[9px]">
                  <button
                    onClick={() => { setPlanningAlgo("AStar"); initPlanningGrid(); }}
                    className={`py-1.5 rounded uppercase font-bold transition-all border ${
                      planningAlgo === "AStar"
                        ? "bg-blue-950/50 text-blue-400 border-blue-900/60"
                        : "bg-black/30 text-gray-500 border-gray-950 hover:text-gray-400"
                    }`}
                  >
                    A* Grid Search
                  </button>
                  <button
                    onClick={() => { setPlanningAlgo("Bug2"); initPlanningGrid(); }}
                    className={`py-1.5 rounded uppercase font-bold transition-all border ${
                      planningAlgo === "Bug2"
                        ? "bg-blue-950/50 text-blue-400 border-blue-900/60"
                        : "bg-black/30 text-gray-500 border-gray-950 hover:text-gray-400"
                    }`}
                  >
                    Bug-2 Algorithm
                  </button>
                </div>

                <div className="space-y-2 text-[10px] bg-black/40 border border-gray-950 p-2.5 rounded text-gray-400 leading-relaxed">
                  {planningAlgo === "AStar" ? (
                    <p>
                      <strong>A* Heuristic Search</strong> utilizes a priority queue tracking <strong>f(n) = g(n) + h(n)</strong>. Manhattan distance yields extremely cohesive, optimal shortest path routes around boundaries.
                    </p>
                  ) : (
                    <p>
                      <strong>Bug-2 Navigation</strong> utilizes boundary-following loops. The robot maintains a straight-line vector to destination, tracing obstacle obstacles clockwise on obstacle collision, then returning to line path.
                    </p>
                  )}
                </div>

                <div className="grid grid-cols-2 gap-2 border-t border-gray-950 pt-2.5">
                  <div className="bg-black/30 p-2 rounded border border-gray-950 text-center">
                    <span className="text-[7px] text-gray-500 block uppercase">Traversed steps</span>
                    <span className="text-[13px] font-bold text-gray-200 mt-0.5">{planningSteps}</span>
                  </div>
                  <div className="bg-black/30 p-2 rounded border border-gray-950 text-center">
                    <span className="text-[7px] text-gray-500 block uppercase">Solver status</span>
                    <span className="text-[8px] font-bold text-emerald-400 block uppercase mt-0.5">
                      {planningActive ? "solving map..." : "ready"}
                    </span>
                  </div>
                </div>
              </div>

              <div className="pt-1.5 text-[10px] leading-relaxed text-gray-500 font-mono space-y-1">
                <span className="text-amber-500 uppercase font-bold text-[8px] tracking-widest block">Chapter 10/11 Routing</span>
              </div>
            </div>
          )}

          {/* TAB 4 CONTENT: POTENTIAL FIELDS & REACTIVE AVOIDANCE */}
          {activeTab === "avoidance" && (
            <div className="space-y-3">
              <label className="text-[10px] text-blue-400 font-bold uppercase tracking-wider block font-mono pb-1 border-b border-gray-950">
                Vector Force Parameters
              </label>

              <div className="space-y-3.5 text-xs font-mono">
                {/* speed slider */}
                <div className="space-y-1">
                  <div className="flex justify-between items-center text-[10px]">
                    <span className="text-gray-500 uppercase">Movement Vector Speed (V)</span>
                    <span className="text-blue-400 font-bold">{avoidSpeed.toFixed(1)} cm/s</span>
                  </div>
                  <input
                    type="range"
                    min={0.5}
                    max={4.0}
                    step={0.5}
                    value={avoidSpeed}
                    onChange={(e) => setAvoidSpeed(parseFloat(e.target.value))}
                    className="w-full h-1 bg-black rounded outline-none accent-blue-500"
                  />
                </div>

                <div className="bg-black/40 border border-gray-950 p-2.5 rounded text-gray-400 font-mono text-[9px] space-y-1.5 leading-snug">
                  <span className="text-blue-400 font-bold text-[8px] uppercase block">Virtual Vector Fields:</span>
                  <p className="text-emerald-400 font-semibold">• Green indicator line: Attractive force vector steering robot directly toward Target Goal.</p>
                  <p className="text-red-400 font-semibold">• Red indicator line: Repulsive vector pushing away from center obstacle.</p>
                  <p className="text-sky-400 font-semibold">• Blue indicator line: Summed emergent pilot trajectory vector.</p>
                </div>
              </div>

              <div className="pt-3 border-t border-gray-950 text-[10px] leading-relaxed text-gray-500 font-mono space-y-1">
                <span className="text-amber-500 uppercase font-bold text-[8px] tracking-widest block">Chapter 12 Control</span>
                <p>U_total = U_attraction + U_repulsion</p>
                <p>Vector Force = -∇ U_total</p>
              </div>
            </div>
          )}

          {/* TAB 5 CONTENT: EDGE MPC & ROS2 DDS MIDDLEWARE */}
          {activeTab === "edge_mpc" && (
            <div className="space-y-3">
              <label className="text-[10px] text-blue-400 font-bold uppercase tracking-wider block font-mono pb-1 border-b border-gray-950">
                ROS2 QoS & MPC Parameters
              </label>

              <div className="space-y-3 text-xs font-mono">
                {/* QoS Reliability Selector */}
                <div className="space-y-1">
                  <span className="text-[9px] text-gray-500 uppercase">ROS2 DDS Reliability Policy</span>
                  <div className="grid grid-cols-2 gap-1.5">
                    <button
                      onClick={() => setDdsQosReliability("RELIABLE")}
                      className={`py-1 rounded text-[9px] font-bold border transition-all ${
                        ddsQosReliability === "RELIABLE"
                          ? "bg-emerald-950/40 text-emerald-400 border-emerald-500/30"
                          : "bg-black/30 text-gray-500 border-transparent hover:text-gray-400"
                      }`}
                    >
                      RELIABLE
                    </button>
                    <button
                      onClick={() => setDdsQosReliability("BEST_EFFORT")}
                      className={`py-1 rounded text-[9px] font-bold border transition-all ${
                        ddsQosReliability === "BEST_EFFORT"
                          ? "bg-amber-950/40 text-amber-400 border-amber-500/30"
                          : "bg-black/30 text-gray-500 border-transparent hover:text-gray-400"
                      }`}
                    >
                      BEST_EFFORT
                    </button>
                  </div>
                </div>

                {/* QoS Durability Selector */}
                <div className="space-y-1 pt-1">
                  <span className="text-[9px] text-gray-500 uppercase">ROS2 DDS Durability Policy</span>
                  <div className="grid grid-cols-2 gap-1.5">
                    <button
                      onClick={() => setDdsQosDurability("TRANSIENT_LOCAL")}
                      className={`py-1 rounded text-[9px] font-bold border transition-all ${
                        ddsQosDurability === "TRANSIENT_LOCAL"
                          ? "bg-emerald-950/40 text-emerald-400 border-emerald-500/30"
                          : "bg-black/30 text-gray-500 border-transparent hover:text-gray-400"
                      }`}
                    >
                      TRANSIENT LOCAL
                    </button>
                    <button
                      onClick={() => setDdsQosDurability("VOLATILE")}
                      className={`py-1 rounded text-[9px] font-bold border transition-all ${
                        ddsQosDurability === "VOLATILE"
                          ? "bg-amber-950/40 text-amber-400 border-amber-500/30"
                          : "bg-black/30 text-gray-500 border-transparent hover:text-gray-400"
                      }`}
                    >
                      VOLATILE
                    </button>
                  </div>
                </div>

                {/* Live Latency & Packet Loss Display */}
                <div className="grid grid-cols-2 gap-2 pt-2 border-t border-gray-950">
                  <div className="bg-black/40 p-2 rounded border border-gray-950">
                    <span className="text-[8px] text-gray-500 block uppercase">MPC Latency</span>
                    <span className={`text-[12px] font-bold block ${mpcLatency < 5 ? "text-emerald-400" : "text-amber-500"}`}>
                      {mpcLatency.toFixed(2)} ms
                    </span>
                    <span className="text-[7px] text-gray-400">Target: &lt; 5.0ms</span>
                  </div>
                  <div className="bg-black/40 p-2 rounded border border-gray-950">
                    <span className="text-[8px] text-gray-500 block uppercase">DDS Packet Loss</span>
                    <span className={`text-[12px] font-bold block ${packetLossPercent === 0 ? "text-emerald-400" : "text-amber-400"}`}>
                      {packetLossPercent.toFixed(2)} %
                    </span>
                    <span className="text-[7px] text-gray-400">Standardized QoS</span>
                  </div>
                </div>

                {/* Manual Recalculation Trigger */}
                <button
                  onClick={triggerMpcRecalculation}
                  disabled={recalculatingMpc}
                  className="w-full py-1.5 bg-blue-950/50 hover:bg-blue-900/40 text-blue-400 border border-blue-900/40 rounded text-[9px] font-bold tracking-wider uppercase transition-all"
                >
                  {recalculatingMpc ? "RECOMPUTING TRAJECTORY..." : "FORCE TRAJECTORY RECALCULATION"}
                </button>
              </div>

              <div className="pt-2 border-t border-gray-950 text-[10px] leading-relaxed text-gray-500 font-mono space-y-0.5">
                <span className="text-amber-500 uppercase font-bold text-[8px] tracking-widest block">Predictive Maintenance</span>
                <p className="text-[9px] text-slate-400 italic font-sans">
                  Status: <span className="text-emerald-400 font-bold">{taskReroutingStatus}</span>
                </p>
              </div>
            </div>
          )}

          {/* Universal control deck buttons */}
          <div className="pt-4 border-t border-gray-900/80 flex items-center justify-between">
            <span className="text-[8px] text-gray-550 font-mono uppercase block">
              STATE CONTROLLERS
            </span>

            <div className="flex gap-2">
              <button
                onClick={() => {
                  if (activeTab === "kinematics") {
                    setKinSimRunning(!kinSimRunning);
                  } else if (activeTab === "localization") {
                    setLocSimRunning(!locSimRunning);
                  } else if (activeTab === "planning") {
                    if (planningAlgo === "AStar") {
                      runPathFinding();
                    } else {
                      simulateBugAlgo();
                    }
                  } else if (activeTab === "avoidance") {
                    setAvoidSimRunning(!avoidSimRunning);
                  } else if (activeTab === "edge_mpc") {
                    setEdgeMpcSimRunning(!edgeMpcSimRunning);
                  }
                }}
                disabled={activeTab === "planning" && planningActive}
                className={`px-3 py-1.5 text-[9px] uppercase font-bold font-mono tracking-wider rounded transition-all flex items-center gap-1 cursor-pointer select-none ${
                  (activeTab === "kinematics" && kinSimRunning) || 
                  (activeTab === "localization" && locSimRunning) || 
                  (activeTab === "avoidance" && avoidSimRunning) ||
                  (activeTab === "planning" && planningActive) ||
                  (activeTab === "edge_mpc" && edgeMpcSimRunning)
                    ? "bg-red-950/60 text-red-400 border border-red-900/40"
                    : "bg-blue-600 hover:bg-blue-500 text-white"
                }`}
              >
                {(activeTab === "kinematics" && kinSimRunning) || 
                (activeTab === "localization" && locSimRunning) || 
                (activeTab === "avoidance" && avoidSimRunning) ||
                (activeTab === "planning" && planningActive) ||
                (activeTab === "edge_mpc" && edgeMpcSimRunning) ? (
                  <>
                    <Pause className="w-3 h-3 text-red-400" />
                    <span>PAUSE SIMULATION</span>
                  </>
                ) : (
                  <>
                    <Play className="w-3 h-3" />
                    <span>{activeTab === "planning" ? "RUN ROUTING WORKER" : "START SIMULATION"}</span>
                  </>
                )}
              </button>

              <button
                onClick={() => {
                  if (activeTab === "kinematics") {
                    setKinSimRunning(false);
                    kinPoseRef.current = { x: 125, y: 125, theta: 0 };
                    kinPathRef.current = [];
                  } else if (activeTab === "localization") {
                    setLocSimRunning(false);
                    robotPoseRef.current = { x: 50, y: 125 };
                    odometryPoseRef.current = { x: 50, y: 125 };
                    initParticles();
                  } else if (activeTab === "planning") {
                    initPlanningGrid();
                  } else if (activeTab === "avoidance") {
                    setAvoidSimRunning(false);
                    activeAvoidPoseRef.current = { x: 40, y: 125, theta: 0 };
                    avoidPathRef.current = [];
                  } else if (activeTab === "edge_mpc") {
                    setEdgeMpcSimRunning(false);
                    setJointTorques([42.1, 58.4, 89.2, 22.1, 14.8, 8.5]);
                    setMpcLatency(2.4);
                    setPacketLossPercent(0.00);
                  }
                }}
                className="px-2.5 py-1.5 bg-gray-900 hover:bg-gray-800 text-gray-400 rounded text-[9px] uppercase font-mono tracking-wider border border-gray-800/80 transition-all cursor-pointer"
                title="Reset simulation parameters"
              >
                <RotateCcw className="w-3 h-3" />
              </button>
            </div>
          </div>
        </div>

        {/* Right Side: Active HTML5 Drawing Stage Canvas / Edge Monitoring Panel */}
        <div className="md:col-span-7 bg-[#020407] border border-gray-900 rounded-lg p-3.5 flex flex-col items-center justify-center min-h-[300px]">
          
          {activeTab === "planning" ? (
            /* Custom Cell-based SVG or CSS Grid for planning module */
            <div className="flex flex-col items-center space-y-3.5 w-full">
              <div 
                className="grid gap-0.5 bg-gray-950 border border-gray-900 rounded p-1 w-full max-w-[250px] aspect-square"
                style={{ gridTemplateColumns: `repeat(${gridSize}, minmax(0, 1fr))` }}
              >
                {grid.map((rowArr, rIdx) => 
                  rowArr.map((cellState, cIdx) => {
                    let bg = "bg-[#05060a] border-[0.5px] border-gray-950";
                    if (cellState === "obstacle") bg = "bg-red-500/85 shadow-sm";
                    if (cellState === "start") bg = "bg-blue-500 animate-pulse font-bold text-[8px]";
                    if (cellState === "goal") bg = "bg-emerald-500 animate-pulse font-bold";
                    if (cellState === "visited") bg = "bg-blue-950/40 border-[0.5px] border-blue-900/30";
                    if (cellState === "path") bg = "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.5)] border-[0.5px] border-emerald-300/40";

                    return (
                      <div 
                        key={`${rIdx}-${cIdx}`} 
                        className={`aspect-square w-full rounded-[1px] transition-colors ${bg} flex items-center justify-center`}
                        title={`Row ${rIdx}, Col ${cIdx}: ${cellState}`}
                      />
                    );
                  })
                )}
              </div>

              {/* Grid Legends */}
              <div className="flex flex-wrap items-center justify-center gap-3 text-[8px] font-mono text-gray-500 uppercase border-t border-gray-950 pt-2.5 w-full">
                <div className="flex items-center gap-1"><span className="w-2 h-2 rounded bg-blue-500 shrink-0" /> Start</div>
                <div className="flex items-center gap-1"><span className="w-2 h-2 rounded bg-emerald-500 shrink-0" /> Target</div>
                <div className="flex items-center gap-1"><span className="w-2 h-2 rounded bg-red-500 shrink-0" /> Obstacles</div>
                <div className="flex items-center gap-1"><span className="w-2 h-2 rounded bg-blue-950 shrink-0" /> explored</div>
                <div className="flex items-center gap-1"><span className="w-2 h-2 rounded bg-emerald-400 shrink-0" /> route</div>
              </div>
            </div>
          ) : activeTab === "edge_mpc" ? (
            /* Custom Joint-Torque Monitoring Panel and ROS2 QoS Status */
            <div className="w-full space-y-4 font-mono text-xs">
              <div className="p-3 bg-gradient-to-r from-emerald-950/20 to-transparent border border-emerald-900/20 rounded-lg flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Activity className="w-4 h-4 text-emerald-400 animate-pulse" />
                  <div>
                    <span className="text-[10px] text-emerald-400 font-bold block uppercase">Edge MPC Status</span>
                    <span className="text-[11px] text-gray-300">Continuous Sub-5ms Trajectory Tracking Active</span>
                  </div>
                </div>
                <span className="px-2 py-0.5 bg-emerald-950/80 text-emerald-400 border border-emerald-500/30 rounded text-[9px]">
                  MPC ACTIVE
                </span>
              </div>

              {/* Joint Torques Grid */}
              <div className="space-y-2">
                <span className="text-[9px] text-gray-500 uppercase tracking-wider block">Real-time Joint Torque Sensors (Nm)</span>
                <div className="grid grid-cols-2 gap-3">
                  {jointTorques.map((torque, idx) => {
                    const maxTorque = 150;
                    const percent = Math.min(100, (torque / maxTorque) * 100);
                    const isHigh = torque > 115;
                    return (
                      <div key={idx} className="p-2.5 bg-black/40 border border-gray-950 rounded-md space-y-1.5">
                        <div className="flex justify-between items-center text-[10px]">
                          <span className="text-gray-400 uppercase font-semibold">Joint {idx + 1}</span>
                          <span className={`font-bold ${isHigh ? "text-red-400" : "text-emerald-400"}`}>
                            {torque} Nm
                          </span>
                        </div>
                        <div className="h-1.5 bg-gray-950 rounded-full overflow-hidden p-[1px]">
                          <div 
                            className={`h-full rounded-full transition-all duration-300 ${isHigh ? "bg-red-500" : "bg-emerald-500"}`}
                            style={{ width: `${percent}%` }}
                          />
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>

              {/* QoS Middleware Standard Status */}
              <div className="p-3 bg-black/40 border border-gray-950 rounded-lg space-y-2">
                <span className="text-[9px] text-gray-500 uppercase tracking-wider block">Standardized ROS2 DDS Middleware QoS</span>
                <div className="grid grid-cols-3 gap-2 text-[9px] text-center">
                  <div className="p-1.5 bg-gray-950 rounded border border-gray-900">
                    <span className="text-gray-500 block text-[8px] uppercase">Reliability</span>
                    <span className="text-emerald-400 font-bold block mt-0.5">{ddsQosReliability}</span>
                  </div>
                  <div className="p-1.5 bg-gray-950 rounded border border-gray-900">
                    <span className="text-gray-500 block text-[8px] uppercase">Durability</span>
                    <span className="text-emerald-400 font-bold block mt-0.5">{ddsQosDurability}</span>
                  </div>
                  <div className="p-1.5 bg-gray-950 rounded border border-gray-900">
                    <span className="text-gray-500 block text-[8px] uppercase">DDS History</span>
                    <span className="text-emerald-400 font-bold block mt-0.5">KEEP_LAST (10)</span>
                  </div>
                </div>
                <p className="text-[9px] text-slate-500 text-center italic mt-1 leading-normal">
                  Tailored QoS profiles ensure 0% packet loss in high-density wireless environments.
                </p>
              </div>
            </div>
          ) : (
            /* HTML5 Canvas views for Kinematics, Particles and Avoidance */
            <div className="flex flex-col items-center space-y-2.5 w-full">
              <canvas
                ref={
                  activeTab === "kinematics" 
                    ? kinCanvasRef 
                    : activeTab === "localization" 
                    ? locCanvasRef 
                    : avoidCanvasRef
                }
                width={250}
                height={250}
                className="bg-[#020305] border border-gray-900 rounded-lg shadow-inner max-w-full"
              />
              
              {/* Contextual legends for Loc */}
              {activeTab === "localization" && (
                <div className="flex flex-wrap items-center justify-center gap-3 text-[8px] font-mono text-gray-500 uppercase">
                  <div className="flex items-center gap-1"><span className="w-1.5 h-1.5 rounded-full bg-emerald-400 block shrink-0" /> True Posture</div>
                  <div className="flex items-center gap-1"><span className="w-1.5 h-1.5 rounded-full bg-red-400 block shrink-0" /> Odometry Drift</div>
                  <div className="flex items-center gap-1"><span className="w-1.5 h-1.5 rounded-full bg-orange-400 block shrink-0" /> Belief Particles</div>
                </div>
              )}

              {/* Contextual legends for avoid */}
              {activeTab === "avoidance" && (
                <div className="flex flex-wrap items-center justify-center gap-3 text-[8px] font-mono text-gray-500 uppercase">
                  <div className="flex items-center gap-1"><span className="w-2 h-0.5 bg-emerald-500 block shrink-0" /> Attraction force</div>
                  <div className="flex items-center gap-1"><span className="w-2 h-0.5 bg-red-500 block shrink-0" /> Repulsive force</div>
                  <div className="flex items-center gap-1"><span className="w-2 h-0.5 bg-sky-500 block shrink-0" /> Combined trajectory</div>
                </div>
              )}
            </div>
          )}

        </div>
      </div>
    </div>
  );
};
