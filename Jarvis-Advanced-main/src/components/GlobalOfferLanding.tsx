import React, { useState, useMemo } from "react";
import { 
  BookOpen, 
  Sparkles, 
  DollarSign, 
  CheckCircle, 
  TrendingUp, 
  Coins, 
  Download, 
  Copy, 
  FileText, 
  Layers, 
  Shield, 
  Globe, 
  Users, 
  ArrowRight, 
  Lock,
  Flame,
  Check,
  RefreshCw,
  Mail,
  Activity,
  Heart,
  Zap,
  Clock,
  Dumbbell
} from "lucide-react";
import { motion, AnimatePresence } from "motion/react";

interface GlobalOfferLandingProps {
  speakText?: (text: string) => void;
}

export default function GlobalOfferLanding({ speakText }: GlobalOfferLandingProps) {
  // Navigation tabs: now including Inspired-Fitness LLC master program & trajectory
  const [activeSubTab, setActiveSubTab] = useState<"inspired-fitness" | "trajectory" | "manifest" | "profitability">("inspired-fitness");
  
  // State for waitlist sign-up
  const [email, setEmail] = useState("");
  const [waitlistStatus, setWaitlistStatus] = useState<"idle" | "loading" | "success" | "error">("idle");
  const [copiedManifest, setCopiedManifest] = useState(false);

  // States for interactive profitability calculator
  const [licenseType, setLicenseType] = useState<"self-hosted" | "managed" | "enterprise">("managed");
  const [targetUsers, setTargetUsers] = useState<number>(2500);
  const [monthlyPrice, setMonthlyPrice] = useState<number>(19);
  const [infrastructureCostPercent, setInfrastructureCostPercent] = useState<number>(15);

  // Active day selection for the 7-Day Bio-Geometric Trajectory
  const [selectedDay, setSelectedDay] = useState<number>(1);

  // 1. Profitability Calculations based on selected values
  const financialMetrics = useMemo(() => {
    const basePrice = monthlyPrice;
    const rawARR = targetUsers * basePrice * 12;
    const infrastructureCost = (rawARR * infrastructureCostPercent) / 100;
    
    // Support costs scale logarithmically with users
    const supportCosts = Math.log10(targetUsers) * 350 * 12;
    const administrativeCosts = 12000; // General fixed annual cost
    
    const totalAnnualCosts = infrastructureCost + supportCosts + administrativeCosts;
    const annualNetProfit = Math.max(0, rawARR - totalAnnualCosts);
    const grossMarginPercent = rawARR > 0 ? ((rawARR - infrastructureCost) / rawARR) * 100 : 100;
    const netMarginPercent = rawARR > 0 ? (annualNetProfit / rawARR) * 100 : 0;
    
    return {
      arr: rawARR,
      costs: totalAnnualCosts,
      profit: annualNetProfit,
      grossMargin: grossMarginPercent,
      netMargin: netMarginPercent,
      mrr: (targetUsers * basePrice),
      monthlyProfit: annualNetProfit / 12
    };
  }, [licenseType, targetUsers, monthlyPrice, infrastructureCostPercent]);

  // Adjust defaults when the user changes license type
  const handleLicenseTypeChange = (type: "self-hosted" | "managed" | "enterprise") => {
    setLicenseType(type);
    if (type === "self-hosted") {
      setMonthlyPrice(5);
      setInfrastructureCostPercent(5);
    } else if (type === "managed") {
      setMonthlyPrice(19);
      setInfrastructureCostPercent(22);
    } else {
      setMonthlyPrice(299);
      setInfrastructureCostPercent(10);
    }
  };

  const handleWaitlistSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!email || !email.includes("@")) {
      setWaitlistStatus("error");
      return;
    }
    setWaitlistStatus("loading");
    setTimeout(() => {
      setWaitlistStatus("success");
      if (speakText) {
        speakText("Biometrics logged. You are now entered into the sovereign waitlist pipeline.");
      }
    }, 1200);
  };

  const formalManifestText = `========================================================================
                      UNIFIED CODEX & JARVIS OS MANIFEST
                  "A Sovereign Digital Interface for Humanity"
========================================================================

1. PHILOSOPHICAL FOUNDATION
------------------
We build at the intersection of logical processing frameworks and the deep intuitive
subconscious. Recognizing the Law of One, we perceive that all technology must serve 
to integrate individual creative agency rather than centralize control. J.A.R.V.I.S. OS 
exists as a localized, sovereign companion bridging human intentionality with systemic wisdom.

2. THE SOVEREIGN PRAGMACY FRAMEWORK
------------------
• Absolute Localism: Your cognitive memory graphs, episodic history logs, and profile traits 
  belong entirely to your local physical boundary.
• Context Autonomy: The artificial companion executes instructions based on vendor-neutral, 
  human-readable schemas (Open Knowledge Format) to guarantee platform portability.
• Aesthetic Honesty: Removing cognitive noise, telemetry trackers, and unrequested metrics. 
  Technology should be silent, precise, and beautiful.

3. THE PROFITABLE VALUE ENGINE
------------------
To sustain real-world impact and accelerate our co-creative journey, we establish 
a robust, self-funding business model that respects human sovereignty:

A. Independent Self-Hosted Sovereign Nodes (The Developer Tier)
   Providing easy, plug-and-play local binaries for private servers. 
   Model: Pay-as-you-own perpetual licenses with optional developer updates.

B. Zero-Knowledge Sovereign Persona Cloud (The Companion Tier)
   Safe, fully end-to-end encrypted remote metadata sync. 
   Model: Premium recurring subscription model supporting cloud operations without harvesting user context.

C. Crystalline Visual Codex Consulting (The Architectural Tier)
   Bespoke system designs, custom knowledge graphs, and theme assets for enterprises.
   Model: Standard professional advisory contracts mapping enterprise information into unified visual architectures.

We invite the world to collaborate and build. 
Signed in alignment with Shawn Carruth & J.A.R.V.I.S. OS Core.`;

  const copyManifestToClipboard = () => {
    navigator.clipboard.writeText(formalManifestText);
    setCopiedManifest(true);
    setTimeout(() => setCopiedManifest(false), 2000);
  };

  // Trajectory data for interactive viewer
  const trajectoryDays = [
    {
      day: 1,
      title: "Posterior Chain Manifold (Hinge/Pull)",
      focus: "Establishing the 'Saddle Point' of the spinal column.",
      exercises: [
        { name: "Isometric Dead-Stop RDLs", prescription: "3 sets of 8 (4-3-1 tempo)", notes: "Hold the bottom position (the saddle) for 3 seconds to cancel momentum and isolate the hamstring-glute junction." },
        { name: "Laminar Flow Lat Pull-downs", prescription: "4 sets of 12", notes: "Focus on maintaining consistent 'vorticity' in the latissimus dorsi without using momentum or shoulder shrugs." },
        { name: "Hyperbolic Face Pulls", prescription: "3 sets of 20", notes: "High-rep 'holographic' finish to maximize local blood volume and rear deltoid/rotator cuff alignment." }
      ]
    },
    {
      day: 2,
      title: "Anterior Structure (Squat/Push)",
      focus: "Scaling the lower body manifold while protecting joint complexes.",
      exercises: [
        { name: "Goblet Squats (Mostow Rigidity)", prescription: "4 sets of 10", notes: "Pause for 2 seconds at the bottom. Ensure the femoral alignment is perfectly aligned to maintain structural integrity." },
        { name: "Incline Press (Vector Optimization)", prescription: "3 sets of 10", notes: "Controlled 4-second eccentric phase to protect the rotator cuff and shoulder capsule." },
        { name: "BFR Leg Extensions", prescription: "3 sets of 30", notes: "Blood Flow Restriction. Induce localized hypoxia for metabolic 'Blow-up' without joint load." }
      ]
    },
    {
      day: 3,
      title: "Active Recovery / 'The Vacuum'",
      focus: "Reducing metabolic entropy and restoring cellular balance.",
      exercises: [
        { name: "Low-Intensity Steady State (LISS)", prescription: "30 minutes", notes: "Keep heart rate strictly in the 'Laminar Flow' zone (approx 110-120 BPM) to clear waste products." },
        { name: "Dynamic Mobility Protocol", prescription: "15 minutes", notes: "Focus on hip and thoracic 'tessellation'—opening closed joint spaces through rotational mechanics." }
      ]
    },
    {
      day: 4,
      title: "Upper Body Density (Vertical Press/Pull)",
      focus: "Geometric centering and kinetic chain reinforcement.",
      exercises: [
        { name: "Overhead Press (The Pillar)", prescription: "3 sets of 8", notes: "Treat the core as a 'rigid structure'. Absolute zero hip flex or knee-bend helper momentum." },
        { name: "Neutral Grip Weighted Pull-ups", prescription: "3 sets to failure", notes: "Control the stretch at the bottom. Avoid dead-hanging to protect the joint capsule." },
        { name: "Dips", prescription: "3 sets of 15", notes: "Slow 4-second eccentrics, stopping at 90 degrees to maintain the shoulder manifold's optimal range." }
      ]
    },
    {
      day: 5,
      title: "Lower Body Resilience (Unilateral/Lunge)",
      focus: "Breaking unilateral symmetry to find unified structural stability.",
      exercises: [
        { name: "Bulgarian Split Squats", prescription: "3 sets of 10 per leg", notes: "Address 'unstable ranges' to build deep knee and hip stabilizer resilience." },
        { name: "Single-Leg RDLs", prescription: "3 sets of 12 per side", notes: "Proprioceptive filtering. Keeps the pelvis level and improves ankle-to-hip kinetic chain." },
        { name: "Calf Raises (The Foundation)", prescription: "4 sets of 20", notes: "Hold the top contraction for 2 seconds to reinforce the Achilles tendon." }
      ]
    },
    {
      day: 6,
      title: "The 'Vortex' (Full Body Metabolic Stress)",
      focus: "High-frequency signals to drive overall hypertrophy.",
      exercises: [
        { name: "Kettlebell Swings", prescription: "45s Work / 15s Rest", notes: "Part of a continuous circuit. Hinge explosively to drive posterior power." },
        { name: "Push-ups & Goblet Squats", prescription: "45s Work / 15s Rest", notes: "Keep tension constant. Do not lock out joints completely to maintain continuous load." },
        { name: "Pull-ups", prescription: "45s Work / 15s Rest", notes: "Sustain structural form despite mounting systemic 'turbulence'." }
      ]
    },
    {
      day: 7,
      title: "The 'Sabbath' (Total Recovery)",
      focus: "Re-establishing the Tautological Ring of cellular synthesis.",
      exercises: [
        { name: "Complete Physical Rest", prescription: "24 Hours", notes: "Zero intensive physical training. Focus entirely on structural decompression." },
        { name: "Newton-Hodge Nutrition", prescription: "Continuous", notes: "High protein pulses (3g Leucine every 4 hours) paired with nutrient-dense foods to rebuild." },
        { name: "Mind-Muscle Meditation", prescription: "20 minutes", notes: "Visualize the biological continuum returning to a state of perfect, unified integrity." }
      ]
    }
  ];

  return (
    <div id="global-offer-root" className="space-y-6">
      {/* Header Banner - Custom Styled in Midnight Emerald & Gold for Inspired Fitness */}
      <div id="offer-header" className="relative p-6 rounded-lg bg-gradient-to-r from-[#042f1a]/80 via-slate-950 to-[#022c22]/80 border border-emerald-900/40 overflow-hidden shadow-[0_4px_20px_rgba(4,47,26,0.3)]">
        <div className="absolute top-0 right-0 w-64 h-64 bg-emerald-500/5 rounded-full blur-3xl pointer-events-none" />
        <div className="absolute bottom-0 left-12 w-64 h-64 bg-amber-500/5 rounded-full blur-3xl pointer-events-none" />
        
        <div className="relative flex flex-col lg:flex-row items-start lg:items-center justify-between gap-5">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <span className="px-2 py-0.5 bg-emerald-950/80 text-emerald-400 text-[9px] font-mono rounded border border-emerald-500/30 uppercase tracking-widest font-bold">
                Bio-Geometric Platform Active
              </span>
              <span className="text-[10px] text-amber-500 font-mono tracking-wider">
                INSPIRED-FITNESS LLC • SHAWN CARRUTH
              </span>
            </div>
            <h1 className="text-2xl font-black text-gray-100 tracking-tight flex items-center gap-2 font-sans">
              <Dumbbell className="w-6 h-6 text-emerald-400" /> INSPIRED-FITNESS LLC
            </h1>
            <p className="text-xs text-slate-300 max-w-2xl leading-relaxed">
              Applying the rigorous mechanics of mathematical manifolds, fluid dynamics, and nutritional algebra to prevent entropic decay and reinforce physical architecture.
            </p>
          </div>
          
          <div className="flex flex-wrap bg-[#05070a]/90 p-1 rounded border border-slate-800 shrink-0 font-mono text-[10px]">
            <button
              onClick={() => {
                setActiveSubTab("inspired-fitness");
                if (speakText) speakText("Opening the Inspired Fitness master program.");
              }}
              className={`px-3 py-1.5 rounded transition-all flex items-center gap-1.5 ${
                activeSubTab === "inspired-fitness" ? "bg-emerald-950/80 text-emerald-400 font-bold border border-emerald-500/30" : "text-gray-400 hover:text-gray-200"
              }`}
            >
              <Activity className="w-3.5 h-3.5" /> Master Program
            </button>
            <button
              onClick={() => {
                setActiveSubTab("trajectory");
                if (speakText) speakText("Loading the 7-day bio-geometric training trajectory.");
              }}
              className={`px-3 py-1.5 rounded transition-all flex items-center gap-1.5 ${
                activeSubTab === "trajectory" ? "bg-emerald-950/80 text-emerald-400 font-bold border border-emerald-500/30" : "text-gray-400 hover:text-gray-200"
              }`}
            >
              <Clock className="w-3.5 h-3.5" /> 7-Day Trajectory
            </button>
            <button
              onClick={() => {
                setActiveSubTab("manifest");
                if (speakText) speakText("Opening the formal JARVIS OS manifest.");
              }}
              className={`px-3 py-1.5 rounded transition-all flex items-center gap-1.5 ${
                activeSubTab === "manifest" ? "bg-amber-500/15 text-amber-400 font-bold border border-amber-500/25" : "text-gray-400 hover:text-gray-200"
              }`}
            >
              <FileText className="w-3.5 h-3.5" /> JARVIS Manifest
            </button>
            <button
              onClick={() => {
                setActiveSubTab("profitability");
                if (speakText) speakText("Synthesizing interactive business simulation metrics.");
              }}
              className={`px-3 py-1.5 rounded transition-all flex items-center gap-1.5 ${
                activeSubTab === "profitability" ? "bg-amber-500/15 text-amber-400 font-bold border border-amber-500/25" : "text-gray-400 hover:text-gray-200"
              }`}
            >
              <Coins className="w-3.5 h-3.5" /> Profit Calculator
            </button>
          </div>
        </div>
      </div>

      <AnimatePresence mode="wait">
        {/* SUBTAB 1: INSPIRED FITNESS MASTER PROGRAM */}
        {activeSubTab === "inspired-fitness" && (
          <motion.div
            key="inspired-fitness-tab"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
            className="grid grid-cols-1 lg:grid-cols-12 gap-6"
          >
            {/* Main Interactive Marketing & Philosophy Display */}
            <div className="lg:col-span-8 space-y-6">
              {/* Philosophy Hero Card */}
              <div className="p-6 rounded-xl bg-gradient-to-br from-slate-950 via-[#032012] to-slate-950 border border-emerald-900/30 space-y-4 relative shadow-lg">
                <div className="absolute top-4 right-4 text-emerald-500/15 font-mono text-7xl font-bold select-none">
                  01
                </div>
                <div className="flex items-center gap-2">
                  <div className="p-1.5 bg-emerald-500/10 rounded text-emerald-400 border border-emerald-500/20">
                    <Shield className="w-5 h-5" />
                  </div>
                  <h2 className="text-lg font-bold text-gray-100 font-sans tracking-wide">
                    I. Structure Precedes Phenomenon
                  </h2>
                </div>
                <p className="text-sm text-slate-300 leading-relaxed">
                  At Inspired-Fitness LLC, we reject the archaic, brute-force model of training. Your body is not a machine to be worn down; it is a persistent geometric manifold. Guided by the principles of <strong>Mostow Rigidity</strong>, we believe that physical phenomenon—muscle growth, strength, and vitality—cannot exist without absolute structural integrity. We design movement patterns that act as a stabilizer against the entropic decay of time, ensuring your joint alignment remains pristine while your muscular capability expands.
                </p>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3 pt-2 font-mono text-xs">
                  <div className="p-3 bg-black/40 border border-emerald-950 rounded-lg">
                    <span className="text-emerald-400 font-bold block mb-1">The Tautological Core</span>
                    <p className="text-slate-400 text-[11px]">Movement patterns that are universal and fundamental (Squat, Hinge, Push, Pull).</p>
                  </div>
                  <div className="p-3 bg-black/40 border border-emerald-950 rounded-lg">
                    <span className="text-emerald-400 font-bold block mb-1">The Dissipative Defense</span>
                    <p className="text-slate-400 text-[11px]">Using structured exercise to filter out the natural entropy of aging, preserving the human manifold.</p>
                  </div>
                </div>
              </div>

              {/* Protocol Section Card */}
              <div className="p-6 rounded-xl bg-gradient-to-br from-slate-950 via-[#032012] to-slate-950 border border-emerald-900/30 space-y-4 relative shadow-lg">
                <div className="absolute top-4 right-4 text-emerald-500/15 font-mono text-7xl font-bold select-none">
                  02
                </div>
                <div className="flex items-center gap-2">
                  <div className="p-1.5 bg-emerald-500/10 rounded text-emerald-400 border border-emerald-500/20">
                    <Activity className="w-5 h-5" />
                  </div>
                  <h2 className="text-lg font-bold text-gray-100 font-sans tracking-wide">
                    II. The Vortex-Strain Protocol
                  </h2>
                </div>
                <p className="text-sm text-slate-300 leading-relaxed">
                  We treat human physiology as a fluid, dynamic system. Traditional training often sacrifices the joints for the sake of the muscle, leading to premature wear. Our signature <strong>Vortex-Strain Hypertrophy</strong> protocol utilizes targeted isometric holds to eliminate momentum and isolate tension, followed by controlled laminar flow tempos that maximize local metabolic stress. This triggers a powerful hormonal response and rapid recovery, bypassing the joint strain of heavy, unstructured lifting.
                </p>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3 pt-2 font-mono text-[11px]">
                  <div className="p-3 bg-black/40 border border-emerald-950/60 rounded-lg space-y-1">
                    <span className="text-amber-500 font-bold block">1. Stationary Phase</span>
                    <p className="text-slate-400">5-second isometric holds at the point of maximum mechanical disadvantage to cancel momentum noise.</p>
                  </div>
                  <div className="p-3 bg-black/40 border border-emerald-950/60 rounded-lg space-y-1">
                    <span className="text-amber-500 font-bold block">2. Laminar Flow</span>
                    <p className="text-slate-400">A strict 4-0-2-0 tempo (4s down, 2s up) to keep the vorticity of tension inside the muscle manifold.</p>
                  </div>
                  <div className="p-3 bg-black/40 border border-emerald-950/60 rounded-lg space-y-1">
                    <span className="text-amber-500 font-bold block">3. Holographic Flush</span>
                    <p className="text-slate-400">High-repetition (20+ reps) finishers that saturate the muscle with nutrient-rich blood to signal growth.</p>
                  </div>
                </div>
              </div>

              {/* Nutrition Slope Card */}
              <div className="p-6 rounded-xl bg-gradient-to-br from-slate-950 via-[#032012] to-slate-950 border border-emerald-900/30 space-y-4 relative shadow-lg">
                <div className="absolute top-4 right-4 text-emerald-500/15 font-mono text-7xl font-bold select-none">
                  03
                </div>
                <div className="flex items-center gap-2">
                  <div className="p-1.5 bg-emerald-500/10 rounded text-emerald-400 border border-emerald-500/20">
                    <Zap className="w-5 h-5" />
                  </div>
                  <h2 className="text-lg font-bold text-gray-100 font-sans tracking-wide">
                    III. The Newton-Hodge Metabolic Slope
                  </h2>
                </div>
                <p className="text-sm text-slate-300 leading-relaxed">
                  True physical preservation is a matter of arithmetic stability. To maintain an elite, muscular 150lb frame or to build resilient tissue in the 50+ demographic, your anabolic potential must consistently sit above catabolic breakdown. Through our precise nutrient-timing strategy, we establish a consistent "pulse-resolution" protocol, triggering protein synthesis and keeping your biological lattice perfectly stabilized.
                </p>
                <div className="p-4 bg-black/50 border border-emerald-950 rounded-lg flex flex-col md:flex-row items-center gap-4">
                  <div className="text-center bg-emerald-950/40 px-4 py-3 rounded-lg border border-emerald-500/20 shrink-0 font-mono">
                    <span className="text-[10px] text-slate-400 block uppercase">Anabolic Ratio</span>
                    <span className="text-xl font-bold text-emerald-400">Newton &gt; Hodge</span>
                  </div>
                  <div className="text-xs text-slate-400 leading-relaxed font-mono">
                    <strong className="text-slate-200">Protein Frequency:</strong> To maintain your specific "Lattice," we initiate a protein "pulse" of approx. 3g of Leucine every 3-4 hours to trigger the mTor pathway (the biological tautological ring).
                  </div>
                </div>
              </div>
            </div>

            {/* Sticky Sidebar with Marketing Copy and Call-To-Action */}
            <div className="lg:col-span-4 space-y-4">
              <div className="p-5 bg-slate-950 border border-emerald-900/40 rounded-xl space-y-4 relative overflow-hidden shadow-md">
                <div className="absolute inset-0 bg-gradient-to-b from-emerald-950/10 to-transparent pointer-events-none" />
                <p className="text-[10px] text-emerald-400 font-mono font-bold uppercase tracking-widest">Polished Website Showcase</p>
                
                <div className="p-4 bg-black/40 border border-emerald-950 rounded-lg space-y-3 font-serif italic text-slate-200 text-sm leading-relaxed text-center">
                  "At Inspired-Fitness, we don't just train bodies; we preserve human architecture. Led by Shawn Carruth—author, researcher, and trainer—we apply the principles of Bio-Geometric Optimization to ensure your physical structure remains as resilient as the mathematics that governs the universe. From 150lb elite athletes to the 50+ demographic, we move beyond the phenomenon of fitness into the science of stability."
                </div>

                <div className="space-y-2 pt-2">
                  <label className="text-[10px] text-slate-400 font-mono uppercase">Request Consultation / Waitlist</label>
                  <form onSubmit={handleWaitlistSubmit} className="space-y-2">
                    <input
                      type="email"
                      placeholder="Enter email to secure access..."
                      value={email}
                      onChange={(e) => setEmail(e.target.value)}
                      className="w-full px-3 py-2 bg-black text-xs text-white border border-emerald-950 focus:border-emerald-500 focus:outline-none rounded"
                    />
                    <button
                      type="submit"
                      className="w-full py-2 bg-gradient-to-r from-emerald-600 to-emerald-700 hover:from-emerald-500 hover:to-emerald-600 text-white font-bold font-mono text-[10px] tracking-wider uppercase rounded transition-all border border-emerald-950 cursor-pointer"
                    >
                      {waitlistStatus === "loading" ? "TRANSMITTING..." : "SECURE MY SPACE"}
                    </button>
                  </form>
                  {waitlistStatus === "success" && (
                    <p className="text-[10px] text-emerald-400 font-mono text-center mt-1">✓ Connection established. Welcome to the stable manifold.</p>
                  )}
                </div>

                <div className="pt-2 border-t border-emerald-950/60">
                  <button
                    onClick={() => setActiveSubTab("trajectory")}
                    className="w-full py-2 bg-[#05070a] hover:bg-emerald-950/20 text-emerald-400 hover:text-emerald-300 font-bold font-mono text-[10px] tracking-wider uppercase rounded border border-emerald-900/30 transition-all flex items-center justify-center gap-1"
                  >
                    <span>View 7-Day Trajectory</span>
                    <ArrowRight className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              {/* Quick Bios Stats */}
              <div className="p-4 bg-[#090b11]/80 border border-slate-800 rounded-xl font-mono text-xs space-y-2 text-slate-400">
                <span className="text-[10px] text-emerald-400 font-bold block uppercase">Architect Profile</span>
                <div className="flex justify-between border-b border-slate-900 pb-1">
                  <span>Name:</span>
                  <span className="text-slate-200">Shawn Carruth</span>
                </div>
                <div className="flex justify-between border-b border-slate-900 pb-1">
                  <span>Age:</span>
                  <span className="text-slate-200">53 Years</span>
                </div>
                <div className="flex justify-between border-b border-slate-900 pb-1">
                  <span>Weight:</span>
                  <span className="text-slate-200">150 lbs (Stable Manifold)</span>
                </div>
                <div className="flex justify-between">
                  <span>Philosophy:</span>
                  <span className="text-emerald-400 font-bold">Law of One</span>
                </div>
              </div>
            </div>
          </motion.div>
        )}

        {/* SUBTAB 2: 7-DAY TRAJECTORY VIEWER */}
        {activeSubTab === "trajectory" && (
          <motion.div
            key="trajectory-tab"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
            className="space-y-6"
          >
            {/* Day Selector Buttons */}
            <div className="flex flex-wrap gap-2 p-1 bg-slate-950 border border-emerald-900/30 rounded-lg">
              {trajectoryDays.map((tDay) => (
                <button
                  key={tDay.day}
                  onClick={() => {
                    setSelectedDay(tDay.day);
                    if (speakText) speakText(`Loading training protocols for Day ${tDay.day}`);
                  }}
                  className={`flex-1 min-w-[80px] py-2 px-3 text-center rounded text-xs font-mono transition-all border ${
                    selectedDay === tDay.day
                      ? "bg-emerald-950/80 text-emerald-400 border-emerald-500/30 font-bold shadow-sm"
                      : "bg-transparent text-slate-400 border-transparent hover:text-slate-200"
                  }`}
                >
                  Day {tDay.day}
                </button>
              ))}
            </div>

            {/* Selected Day Details */}
            <div className="p-6 rounded-xl bg-gradient-to-br from-slate-950 via-[#032012] to-slate-950 border border-emerald-900/40 relative overflow-hidden shadow-lg">
              <div className="absolute top-4 right-4 text-emerald-500/10 font-mono text-8xl font-black select-none">
                D0{selectedDay}
              </div>

              <div className="relative space-y-4">
                <div className="space-y-1">
                  <span className="px-2 py-0.5 bg-emerald-950/80 text-emerald-400 text-[9px] font-mono rounded border border-emerald-500/30 uppercase tracking-widest font-bold">
                    Trajectory Node
                  </span>
                  <h2 className="text-xl font-bold text-gray-100 font-sans tracking-tight">
                    {trajectoryDays[selectedDay - 1].title}
                  </h2>
                  <p className="text-xs text-amber-500 font-mono italic">
                    Focus: {trajectoryDays[selectedDay - 1].focus}
                  </p>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-3">
                  {trajectoryDays[selectedDay - 1].exercises.map((ex, idx) => (
                    <div key={idx} className="p-4 bg-black/40 border border-emerald-950 rounded-lg space-y-2 hover:border-emerald-800/30 transition-all flex flex-col justify-between">
                      <div className="space-y-1">
                        <span className="text-[10px] text-emerald-400 font-mono block">EXERCISE 0{idx + 1}</span>
                        <h3 className="text-sm font-bold text-slate-200 font-sans">{ex.name}</h3>
                        <span className="inline-block px-1.5 py-0.5 bg-amber-500/10 text-amber-400 border border-amber-500/20 rounded font-mono text-[10px] mt-1">
                          {ex.prescription}
                        </span>
                      </div>
                      <p className="text-[11px] text-slate-400 leading-relaxed font-mono mt-3 pt-2 border-t border-emerald-950/40">
                        {ex.notes}
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Trajectory Blueprint Export/Sync */}
            <div className="p-4 bg-slate-950 border border-slate-800 rounded-lg flex flex-col sm:flex-row justify-between items-center gap-4 text-xs font-mono">
              <div className="space-y-0.5">
                <p className="font-bold text-slate-200">The entire 7-day trajectory has been synchronized to your Workspace.</p>
                <p className="text-slate-400 text-[11px]">It resides in a beautiful Google Doc in your Google Drive.</p>
              </div>
              <a
                href="https://docs.google.com/document/d/1VKh431iKwxYo-1-p-wbFNK4gl5NTgFH8VhvOk45eQwI/edit"
                target="_blank"
                rel="noreferrer"
                className="px-4 py-2 bg-emerald-950/80 hover:bg-emerald-900 text-emerald-400 font-bold text-[10px] rounded border border-emerald-500/30 transition-all flex items-center gap-1.5"
              >
                <span>OPEN WORKSPACE DOCUMENT</span>
                <FileText className="w-3.5 h-3.5" />
              </a>
            </div>
          </motion.div>
        )}

        {/* SUBTAB 3: FORMAL MANIFEST */}
        {activeSubTab === "manifest" && (
          <motion.div
            key="manifest-tab"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
            className="grid grid-cols-1 lg:grid-cols-12 gap-5"
          >
            {/* Manifest Markdown & Visual Details */}
            <div id="manifest-details" className="lg:col-span-8 space-y-4">
              <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4">
                <div className="flex justify-between items-center border-b border-gray-800 pb-3">
                  <div className="space-y-0.5">
                    <h2 className="text-sm font-bold text-gray-200 uppercase tracking-wider flex items-center gap-2 font-mono">
                      <BookOpen className="w-4 h-4 text-amber-400" /> Executive Decree & Concept Statement
                    </h2>
                    <p className="text-[10px] text-gray-500 font-mono">ESTABLISHED JUN 2026 • AUTHOR: SUPERVISOR & JARVIS OS</p>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={copyManifestToClipboard}
                      className="px-2.5 py-1 bg-gray-900 border border-gray-800 hover:bg-gray-850 hover:border-gray-700 text-slate-350 hover:text-slate-200 text-[10px] font-mono rounded flex items-center gap-1 transition-all"
                    >
                      {copiedManifest ? (
                        <>
                          <Check className="w-3.5 h-3.5 text-emerald-400" />
                          <span>COPIED SPEC</span>
                        </>
                      ) : (
                        <>
                          <Copy className="w-3.5 h-3.5" />
                          <span>COPY CODEX</span>
                        </>
                      )}
                    </button>
                  </div>
                </div>

                <div className="relative font-mono text-xs text-gray-300 leading-relaxed max-h-[420px] overflow-y-auto whitespace-pre p-4 rounded bg-black/40 border border-gray-900/80 scrollbar-thin scrollbar-thumb-gray-800/80">
                  {formalManifestText}
                </div>

                <div className="p-3 bg-amber-500/5 rounded border border-amber-900/20 text-[11px] text-amber-200 leading-relaxed">
                  <span className="font-bold uppercase tracking-wider block mb-1">PROFITABILITY STRATEGY THESIS:</span>
                  By charging premium tier pricing for encrypted zero-knowledge companion synchronization, we bypass the need to analyze, harvest, or resell user data. The model treats privacy not as a compliant checkbox, but as our primary high-margin feature.
                </div>
              </div>
            </div>

            {/* Side Metrics/Highlights */}
            <div id="manifest-sidebar" className="lg:col-span-4 space-y-4">
              <div className="p-4 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4 font-mono">
                <p className="text-[10px] text-sky-400 font-bold uppercase tracking-widest">Business Model Pillars</p>
                
                <div className="space-y-3.5 text-xs text-gray-300">
                  <div id="pillar-1" className="p-3 bg-gray-950/50 border border-gray-900 rounded-lg space-y-1 hover:border-amber-900/30 transition-all">
                    <div className="flex items-center justify-between text-amber-400 font-bold text-[10px]">
                      <span>01 / DEVELOPER LICENSE</span>
                      <span className="text-[9px] px-1 bg-amber-400/10 text-amber-300 rounded border border-amber-400/20">$50 One-Time</span>
                    </div>
                    <p className="text-[11px] text-gray-400 leading-normal">
                      Local terminal binary pack, fully portable. Direct D3 graph export and local sqlite synchronization.
                    </p>
                  </div>

                  <div id="pillar-2" className="p-3 bg-gray-950/50 border border-gray-900 rounded-lg space-y-1 hover:border-sky-900/30 transition-all">
                    <div className="flex items-center justify-between text-sky-400 font-bold text-[10px]">
                      <span>02 / SOVEREIGN SYNC</span>
                      <span className="text-[9px] px-1 bg-sky-400/10 text-sky-300 rounded border border-sky-400/20">$19/Mo Recurring</span>
                    </div>
                    <p className="text-[11px] text-gray-400 leading-normal">
                      Zero-knowledge decentralized metadata synchronization across dynamic personal environments.
                    </p>
                  </div>

                  <div id="pillar-3" className="p-3 bg-gray-950/50 border border-gray-900 rounded-lg space-y-1 hover:border-emerald-900/30 transition-all">
                    <div className="flex items-center justify-between text-emerald-400 font-bold text-[10px]">
                      <span>03 / ADVISORY DECK</span>
                      <span className="text-[9px] px-1 bg-emerald-400/10 text-emerald-300 rounded border border-emerald-400/20">$10,000+ Contract</span>
                    </div>
                    <p className="text-[11px] text-gray-400 leading-normal">
                      Integrating structural Law of One conceptual frameworks and tarot diagnostics into organizational information schemas.
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </motion.div>
        )}

        {/* SUBTAB 4: THE INTERACTIVE PROFITABILITY ENGINE */}
        {activeSubTab === "profitability" && (
          <motion.div
            key="profitability-tab"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
            className="grid grid-cols-1 lg:grid-cols-12 gap-5"
          >
            {/* Left Controls */}
            <div id="calc-controls" className="lg:col-span-5 space-y-4">
              <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-4">
                <p className="text-xs font-bold text-amber-400 font-mono uppercase tracking-wider flex items-center gap-1.5">
                  <TrendingUp className="w-4 h-4" /> Venture Model Synthesizer
                </p>

                {/* Model Selector Card */}
                <div className="space-y-2">
                  <label className="text-[10px] text-gray-400 font-mono uppercase">Value Offering Channel</label>
                  <div className="grid grid-cols-3 gap-2">
                    <button
                      onClick={() => handleLicenseTypeChange("self-hosted")}
                      className={`py-1.5 px-2 text-center rounded text-[10px] font-mono border transition-all ${
                        licenseType === "self-hosted"
                          ? "bg-amber-500/10 text-amber-300 border-amber-500/40 font-bold"
                          : "bg-gray-950/40 text-gray-500 border-gray-900 hover:text-gray-300"
                      }`}
                    >
                      Self-Hosted Node
                    </button>
                    <button
                      onClick={() => handleLicenseTypeChange("managed")}
                      className={`py-1.5 px-2 text-center rounded text-[10px] font-mono border transition-all ${
                        licenseType === "managed"
                          ? "bg-amber-500/10 text-amber-300 border-amber-500/40 font-bold"
                          : "bg-gray-950/40 text-gray-500 border-gray-900 hover:text-gray-300"
                      }`}
                    >
                      Sovereign Sync
                    </button>
                    <button
                      onClick={() => handleLicenseTypeChange("enterprise")}
                      className={`py-1.5 px-2 text-center rounded text-[10px] font-mono border transition-all ${
                        licenseType === "enterprise"
                          ? "bg-amber-500/10 text-amber-300 border-amber-500/40 font-bold"
                          : "bg-gray-950/40 text-gray-500 border-gray-900 hover:text-gray-300"
                      }`}
                    >
                      Enterprise Codex
                    </button>
                  </div>
                </div>

                {/* SLIDER: Active Sovereign Users */}
                <div className="space-y-1 pt-2">
                  <div className="flex justify-between items-center text-[10px] font-mono">
                    <span className="text-gray-400 uppercase">Estimated Active Users</span>
                    <span className="text-sky-400 font-bold">{targetUsers.toLocaleString()} Members</span>
                  </div>
                  <input
                    type="range"
                    min="100"
                    max="20000"
                    step="100"
                    value={targetUsers}
                    onChange={(e) => setTargetUsers(parseInt(e.target.value))}
                    className="w-full accent-amber-500 bg-gray-950"
                  />
                  <span className="text-[9px] text-gray-500 block font-mono italic">Adjust the count of private sovereign nodes active in the world.</span>
                </div>

                {/* SLIDER: Subscription Price ($ value) */}
                <div className="space-y-1 pt-2">
                  <div className="flex justify-between items-center text-[10px] font-mono">
                    <span className="text-gray-400 uppercase">Monthly Price Node / Membership</span>
                    <span className="text-amber-400 font-bold">${monthlyPrice} / Month</span>
                  </div>
                  <input
                    type="range"
                    min="1"
                    max={licenseType === "enterprise" ? "2000" : "150"}
                    step={licenseType === "enterprise" ? "20" : "1"}
                    value={monthlyPrice}
                    onChange={(e) => setMonthlyPrice(parseInt(e.target.value))}
                    className="w-full accent-amber-500 bg-gray-950"
                  />
                </div>

                {/* SLIDER: Resource Maintenance Overhead (%) */}
                <div className="space-y-1 pt-2">
                  <div className="flex justify-between items-center text-[10px] font-mono">
                    <span className="text-gray-400 uppercase">Marginal Infrastructure Cost</span>
                    <span className="text-red-400 font-bold">{infrastructureCostPercent}%</span>
                  </div>
                  <input
                    type="range"
                    min="2"
                    max="50"
                    step="1"
                    value={infrastructureCostPercent}
                    onChange={(e) => setInfrastructureCostPercent(parseInt(e.target.value))}
                    className="w-full accent-amber-500 bg-gray-950"
                  />
                  <span className="text-[9px] text-gray-550 block font-mono">Sovereign peer networks and edge models reduce standard hosting server loads significantly!</span>
                </div>
              </div>
            </div>

            {/* Right Financial Dashboard */}
            <div id="calc-dashboard" className="lg:col-span-7 space-y-4 font-mono">
              <div className="p-5 bg-[#090b11]/80 border border-gray-800 rounded-lg space-y-5">
                <p className="text-xs font-bold text-sky-400 uppercase tracking-wider flex items-center gap-1.5">
                  <DollarSign className="w-4 h-4" /> Live Projections (Aesthetic Balance)
                </p>

                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 pt-1">
                  
                  <div id="m-arr" className="p-3 bg-gray-950/60 border border-gray-900 rounded-lg">
                    <p className="text-[9px] text-gray-500 uppercase font-semibold">Projected ARR</p>
                    <p className="text-lg font-bold text-gray-200 mt-1">
                      ${financialMetrics.arr.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                    </p>
                    <p className="text-[8px] text-emerald-400 mt-1 italic font-sans flex items-center gap-0.5">
                      <TrendingUp className="w-2.5 h-2.5" /> Annual Run
                    </p>
                  </div>

                  <div id="m-mrr" className="p-3 bg-gray-950/60 border border-gray-900 rounded-lg">
                    <p className="text-[9px] text-gray-500 uppercase font-semibold">Monthly MRR</p>
                    <p className="text-lg font-bold text-amber-400 mt-1">
                      ${financialMetrics.mrr.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                    </p>
                    <p className="text-[8px] text-gray-400 mt-1 italic font-sans">
                      Active Yield
                    </p>
                  </div>

                  <div id="m-margin" className="p-3 bg-gray-950/60 border border-gray-900 rounded-lg">
                    <p className="text-[9px] text-gray-500 uppercase font-semibold">Net Profit</p>
                    <p className="text-lg font-bold text-emerald-400 mt-1">
                      ${financialMetrics.monthlyProfit.toLocaleString(undefined, { maximumFractionDigits: 0 })}/mo
                    </p>
                    <p className="text-[8px] text-emerald-400 mt-1 italic font-sans flex items-center gap-0.5">
                      Net: {financialMetrics.netMargin.toFixed(1)}%
                    </p>
                  </div>

                  <div id="m-cost" className="p-3 bg-gray-950/60 border border-gray-900 rounded-lg">
                    <p className="text-[9px] text-gray-500 uppercase font-semibold">Maint. Cost</p>
                    <p className="text-lg font-bold text-red-400 mt-1">
                      ${(financialMetrics.costs / 12).toLocaleString(undefined, { maximumFractionDigits: 0 })}/mo
                    </p>
                    <p className="text-[8px] text-red-400 mt-1 italic font-sans">
                      Infrastructure + Ops
                    </p>
                  </div>

                </div>

                {/* Progress Visualizer of Margin Efficiency */}
                <div className="space-y-2 pt-2">
                  <div className="flex justify-between items-center text-[10px]">
                    <span className="text-gray-400 uppercase">Operational Margin Blueprint</span>
                    <span className="text-emerald-400 font-bold">{financialMetrics.netMargin.toFixed(1)}% Yield Efficiency</span>
                  </div>
                  <div className="h-2.5 bg-gray-950 rounded-full overflow-hidden border border-gray-900 p-[1.5px]">
                    <div 
                      className="h-full bg-gradient-to-r from-amber-600 to-emerald-400 rounded-full transition-all duration-350" 
                      style={{ width: `${Math.min(100, Math.max(5, financialMetrics.netMargin))}%` }}
                    />
                  </div>
                  <div className="flex justify-between text-[8px] text-gray-500">
                    <span className="uppercase">High Maintenance overhead (Red Zone)</span>
                    <span className="uppercase font-semibold text-emerald-500">Optimum Profit Margin (Green Zone)</span>
                  </div>
                </div>

                {/* Business Mechanics Explanation Panel */}
                <div className="p-3.5 bg-[#05070a]/90 rounded border border-gray-900 text-xs text-gray-400 space-y-2 leading-relaxed">
                  <p className="text-[10px] font-bold text-gray-300 block uppercase tracking-wider">
                    How JARVIS achieves {financialMetrics.netMargin.toFixed(0)}% Profit Margin:
                  </p>
                  <ul className="list-disc pl-4 space-y-1 text-[11px] font-mono text-gray-400">
                    <li>
                      <strong className="text-gray-300">Decentralized SQLite and OKF Graphs:</strong> Local client-side processing removes expensive central databases and API calls.
                    </li>
                    <li>
                      <strong className="text-gray-300">Self-Directed Software Deployments:</strong> Providing Dockerized server configurations drastically limits typical server lifecycle overhead.
                    </li>
                    <li>
                      <strong className="text-gray-300">Values-Based Encryption Subscriptions:</strong> Zero-knowledge syncing reduces server operations liability; we are not responsible for metadata compliance pipelines.
                    </li>
                  </ul>
                </div>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
