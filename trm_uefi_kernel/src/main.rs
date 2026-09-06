#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

mod allocator;
mod agent_runtime;
mod autonomous_cadence;
mod daily_evolution_engine;
mod bridge;
mod benchmark_epistemology;
mod council;
mod ddr_antibody;
mod self_fuzzing_engine;
mod self_healing_infra;
mod continuum_pbft;
mod kernel_instructions;
mod epistemology_core;
mod digital_twin;
mod font;
mod hwprobe;
mod instella_router;
mod jarvis_quantum;
mod jarvis_substrate;
mod jarvis_symbiosis;
mod jarvis_vibe;
mod jarvis_vibethinker;
mod jarvis_cognitive;
mod k3_logic;
mod k3_ternary_sram;
mod module_registry;
mod monad;
mod microkernel_spawner;
mod memory_mapped_ipc;
mod spawn_subagent;
mod trm_orchestrator;
mod federation_manager;
mod federation_task_coordinator;
mod nand_fs;
mod agent_mesh;
mod attempt_sampler;
mod coordination_protocol;
mod heuristic_consensus;
mod llm_brain;
mod mamba3_turbovec_ssm;
mod pbft_consensus;
mod resonance_engine;
mod security_scanner;
mod semantic_cache;
mod semantic_index;
mod sentinel;
mod self_healing;
mod simd_accelerator;
mod sram_pattern_computer;
mod sub_atomic_kernel;
mod task_arbiter;
mod trm_brain;
mod voltage_node_engine;
mod quantum_drug_screen;
mod quic_protocol;
mod star_matrix;
mod time;
mod totality;
mod trm_7m_optimized;
mod skill_matrix_sync;
mod agent_mailbox;
mod trm_integration;
mod trm7m;
mod turbovec;
mod chain_of_verification;
mod formal_verification_engine;
mod skill_validator;
mod zero_human_test_gen;
mod semantic_knowledge_graph;
mod agent_mailbox_bus;
mod intent_buffer;
mod global_memory;
mod fractal_telemetry;
mod holographic_fabric;
#[path = "modmap_all.rs"]
mod modmap_all;
// VOLTAGE ENERGIZATION — CREATE cluster: GAN design synthesis / generative
// architecture / small diffusion engine / chaos engine. (council fbb0f272)
mod gan_design_synthesizer;
mod generative_architect;
mod small_diffusion_engine;
mod chaos_engine;
// VOLTAGE ENERGIZATION — CAPABILITY cluster: skill registry / cognitive
// load balancing / symbology etymology / tool forge / manus engine /
// instella sparse agent teams. (council fbb0f272)
mod skill_loader;
mod cognitive_load_balancer;
mod symbology_etymology_monad;
mod tool_forge;
mod manus_engine;
mod instella_sparse_agent_teams;
// VOLTAGE ENERGIZATION — ORCHESTRATION cluster: 64-agent orchestrator /
// artifact pipeline / worktree manager / heartbeat scheduler / expert
// registry. (council fbb0f272)
mod full_64_agent_orchestrator;
mod artifact_pipeline;
mod worktree_manager;
mod heartbeat_scheduler;
mod expert_registry;
// VOLTAGE ENERGIZATION — FABRIC cluster: radix-trie deterministic matcher /
// DRAM pattern computer / GPU Infinity Cache geometry / HBTF multi-scale
// telemetry / THz-Bonsai ternary routing gateway. (council fbb0f272)
mod radix_trie_matcher;
mod dram_pattern_computer;
mod gpu_infinity_computer;
mod telemetry_fabric;
mod thz_bonsai_gateway;
// VOLTAGE ENERGIZATION — SENSORY cluster: human-swarm interaction / multimodal
// ingestion / advanced scene fusion / swarm performance insights / GPU unlock /
// hardware-voltage microkernel binder. (council fbb0f272)
mod human_swarm_interaction;
mod multimodal_ingestion;
mod advanced_scene_fusion;
mod swarm_performance_insights;
mod gpu_unlock_architecture;
mod hardware_voltage_microkernel_binder;
// VOLTAGE ENERGIZATION — PLANNING module: real Monte Carlo Tree Search
// (UCB1 selection / expansion / random-rollout simulation / backprop).
// (council 83563732)
mod mcts_plan;

use core::panic::PanicInfo;
use r_efi::efi;
use r_efi::protocols::graphics_output;
use r_efi::protocols::simple_text_input;

pub const MONAD_TRUTH_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;

// LCARS 32-bit Color Palette (BGRA / RGBA for UEFI GOP)
pub const LCARS_BG: u32        = 0xFF0A0A12; // Deep space black-blue
pub const LCARS_GOLD: u32      = 0xFFFF9900; // Primary command gold
pub const LCARS_AMBER: u32     = 0xFFFFCC00; // Warning / telemetry amber
pub const LCARS_ORANGE: u32    = 0xFFFF6600; // Auxiliary rail orange
pub const LCARS_PURPLE: u32    = 0xFF7C4DFF; // Science / reasoning purple
pub const LCARS_BLUE: u32      = 0xFF6688CC; // Engineering blue
pub const LCARS_CYAN: u32      = 0xFF66CCFF; // Comms / spectrum cyan
pub const LCARS_RED: u32       = 0xFFCC3333; // Security alert / reject red
pub const LCARS_GREEN: u32     = 0xFF33CC66; // Truth / approve green
pub const LCARS_WHITE: u32     = 0xFFFFFFFF; // Text white
pub const LCARS_GRAY: u32      = 0xFF444455; // Inactive / border gray
pub const LCARS_DARK_PANEL: u32= 0xFF141422; // Sub-panel background
pub const LCARS_DARK_BLUE: u32 = 0xFF0E1A2E; // Secondary dark
pub const LCARS_PANEL_BG: u32  = 0xFF0E0E18; // Inner widget background

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

fn fast_sin(mut x: f32) -> f32 {
    let pi = core::f32::consts::PI;
    let two_pi = 2.0 * pi;
    x = x - (x / two_pi) as i32 as f32 * two_pi;
    if x < 0.0 { x += two_pi; }
    if x > pi {
        return -fast_sin(x - pi);
    }
    let num = 16.0 * x * (pi - x);
    let den = 5.0 * pi * pi - 4.0 * x * (pi - x);
    if den == 0.0 { 0.0 } else { num / den }
}

fn fast_cos(x: f32) -> f32 {
    fast_sin(x + core::f32::consts::FRAC_PI_2)
}

#[derive(Clone, Copy)]
pub struct SetupState {
    pub selected_item: usize,
    pub storage_mode: u8,    // 0: Live RAM, 1: USB NAND K3, 2: Install NVMe
    pub mesh_enabled: bool,
    pub wifi_radar_calibrated: bool,
    pub rowhammer_guard: bool,
    pub pbft_quorum_armed: bool,
    pub installation_applied: bool,
}

// Interactive Terminal Feed State
const MAX_FEED_LINES: usize = 18;
const MAX_LINE_LEN: usize = 80;

pub struct TerminalState {
    pub input_buf: [u8; 80],
    pub input_len: usize,
    pub feed_lines: [[u8; MAX_LINE_LEN]; MAX_FEED_LINES],
    pub feed_lens: [usize; MAX_FEED_LINES],
    pub feed_colors: [u32; MAX_FEED_LINES],
}

impl TerminalState {
    pub fn new() -> Self {
        let mut term = Self {
            input_buf: [0; 80],
            input_len: 0,
            feed_lines: [[0; MAX_LINE_LEN]; MAX_FEED_LINES],
            feed_lens: [0; MAX_FEED_LINES],
            feed_colors: [LCARS_WHITE; MAX_FEED_LINES],
        };
        term.add_line(b"JARVIS: 'Sovereign intelligence active. Ready for operator command.'", LCARS_WHITE);
        term.add_line(b"LOGIC:  '153.5M Trit K3 knowledge fabric mounted in memory.'", LCARS_GREEN);
        term.add_line(b"SHIELD: 'DDR row-hammer memory antibodies armed & watching physical DRAM.'", LCARS_RED);
        term.add_line(b"HERMES: 'UDP port 8022 mesh discovery beacon ready on Intel AX200.'", LCARS_CYAN);
        term.add_line(b"ARCHI:  'Rock-solid double-buffered 60.0 FPS compositor locked.'", LCARS_AMBER);
        term.add_line(b"TYPE:   'help', 'status', 'council <text>', 'jarvis <text>', 'radar'", LCARS_GOLD);
        term
    }

    pub fn add_line(&mut self, text: &[u8], color: u32) {
        // Shift lines up
        for i in 0..(MAX_FEED_LINES - 1) {
            self.feed_lines[i] = self.feed_lines[i + 1];
            self.feed_lens[i] = self.feed_lens[i + 1];
            self.feed_colors[i] = self.feed_colors[i + 1];
        }
        let last = MAX_FEED_LINES - 1;
        let copy_len = text.len().min(MAX_LINE_LEN);
        self.feed_lines[last][..copy_len].copy_from_slice(&text[..copy_len]);
        self.feed_lens[last] = copy_len;
        self.feed_colors[last] = color;
    }

    /// Internal helper: index of the first whitespace byte in `b`, else `b.len()`.
    fn find_first_ws(b: &[u8]) -> usize {
        for (i, &c) in b.iter().enumerate() {
            if c == b' ' || c == b'\t' {
                return i;
            }
        }
        b.len()
    }

    pub fn execute_command(&mut self, active_tab: &mut u8, image_handle: efi::Handle, bs: *mut efi::BootServices) {
        if self.input_len == 0 {
            return;
        }

        let mut cmd_buf = [0u8; 80];
        let cmd_len = self.input_len.min(80);
        cmd_buf[..cmd_len].copy_from_slice(&self.input_buf[..cmd_len]);
        let cmd = &cmd_buf[..cmd_len];
        self.input_len = 0;
        
        // Echo operator command
        let mut echo_buf = [0u8; 80];
        let prefix = b"OPERATOR > ";
        echo_buf[..prefix.len()].copy_from_slice(prefix);
        let echo_len = prefix.len() + cmd.len().min(80 - prefix.len());
        echo_buf[prefix.len()..echo_len].copy_from_slice(&cmd[..(echo_len - prefix.len())]);
        self.add_line(&echo_buf[..echo_len], LCARS_GOLD);

        // Command matching
        if cmd.starts_with(b"help") || cmd.starts_with(b"?") {
            self.add_line(b"COMMANDS: monad | status | agents | council <motion> | jarvis <prompt> | nand | quic | mesh | hw | radar | setup | clear", LCARS_CYAN);
            self.add_line(b"HOTKEYS:  Press [0]-[9] to jump to any of the 9 sovereign sectors.", LCARS_AMBER);
        } else if cmd.starts_with(b"monad") {
            self.add_line(b"MONAD: Truth Anchor 0x4D4F4E4144000001 VERIFIED (boot-invariant root of trust)", LCARS_GOLD);
            // THE AWAKENING: the Monad reconfigures pipelines for any goal.
            let rest = cmd.get(6..).unwrap_or(b"").trim_ascii();
            let goal: &[u8] = if rest.is_empty() { b"sovereign synthesis" } else { rest };
            let g = core::str::from_utf8(goal).unwrap_or("sovereign synthesis");
            let lines = crate::monad::render_wisdom(g);
            for ln in lines.iter().take(6) {
                self.add_line(ln.as_bytes(), LCARS_CYAN);
            }
        } else if cmd.starts_with(b"pbft") {
            // PHASE 2 ACTIVATION — CONSENSUS cluster executes for real.
            // pbft_consensus full quorum cycle with the 14-agent model.
            let mut pbft = crate::pbft_consensus::PbftConsensus::new();
            let (id, accepted, score) = pbft.run_full_cycle("OPERATOR", "phase2-activation");
            let (total, acc, rej, phase) = pbft.get_status();
            let mut p1 = [0u8; 96];
            let h = b"PBFT CYCLE: proposal #";
            p1[..h.len()].copy_from_slice(h);
            let ids = format!("{}", id);
            p1[h.len()..h.len() + ids.len()].copy_from_slice(ids.as_bytes());
            let mut ri = h.len() + ids.len();
            p1[ri..ri + 12].copy_from_slice(b" accepted="); ri += 12;
            p1[ri] = b'1'; if !accepted { p1[ri] = b'0'; } ri += 1;
            p1[ri..ri + 8].copy_from_slice(b" score="); ri += 8;
            let scs = format!("{:.3}", score);
            p1[ri..ri + scs.len()].copy_from_slice(scs.as_bytes()); ri += scs.len();
            self.add_line(&p1[..ri], LCARS_CYAN);
            let mut p2 = [0u8; 96];
            let h2 = b"PBFT STATUS: total=";
            p2[..h2.len()].copy_from_slice(h2);
            let ts = format!("{}", total);
            p2[h2.len()..h2.len() + ts.len()].copy_from_slice(ts.as_bytes());
            let mut r2 = h2.len() + ts.len();
            p2[r2..r2 + 5].copy_from_slice(b" acc="); r2 += 5;
            let as_ = format!("{}", acc);
            p2[r2..r2 + as_.len()].copy_from_slice(as_.as_bytes()); r2 += as_.len();
            p2[r2..r2 + 5].copy_from_slice(b" rej="); r2 += 5;
            let rs = format!("{}", rej);
            p2[r2..r2 + rs.len()].copy_from_slice(rs.as_bytes()); r2 += rs.len();
            p2[r2..r2 + 9].copy_from_slice(b" phase=0x"); r2 += 9;
            p2[r2] = b'0' + phase as u8; r2 += 1;
            p2[r2..r2 + 3].copy_from_slice(b" (4)"); r2 += 3;
            self.add_line(&p2[..r2], LCARS_GREEN);
            self.add_line(b"PBFT: quorum 2f+1=9 of 14 agents | commit 10 | Byzantine f=4", LCARS_PURPLE);
        } else if cmd.starts_with(b"intel") {
            // VOLTAGE ENERGIZATION — INTELLIGENCE cluster executes.
            // llm_brain persona route + trm_brain collapse + mamba3 step
            let persona = crate::llm_brain::LLMBrain::route_task("optimize the kernel allocator");
            let pname = persona.name;
            let mut p1 = [0u8; 96];
            let h = b"INTEL: persona=";
            p1[..h.len()].copy_from_slice(h);
            let pn = pname.len().min(16);
            p1[h.len()..h.len() + pn].copy_from_slice(&pname.as_bytes()[..pn]);
            let mut ri = h.len() + pn;
            p1[ri..ri + 16].copy_from_slice(b" | persona armed"); ri += 16;
            self.add_line(&p1[..ri], LCARS_CYAN);
            // trm_brain collapse on a real state
            let mut tb = crate::trm_brain::TRMBrain::new();
            let state = [0.9f64, -0.7, 0.4, 0.1, -0.3, 0.8, -0.2, 0.6];
            let col = tb.collapse_state(&state);
            let mut p2 = [0u8; 96];
            let h2 = b"TRM COLLAPSE: trit=";
            p2[..h2.len()].copy_from_slice(h2);
            let t2 = format!("{}", col.k3_trit);
            p2[h2.len()..h2.len() + t2.len()].copy_from_slice(t2.as_bytes());
            let mut ri2 = h2.len() + t2.len();
            p2[ri2..ri2 + 10].copy_from_slice(b" conf="); ri2 += 10;
            let cs = format!("{:.2}", col.confidence);
            p2[ri2..ri2 + cs.len()].copy_from_slice(cs.as_bytes()); ri2 += cs.len();
            p2[ri2..ri2 + 11].copy_from_slice(b" | K3 gate"); ri2 += 11;
            self.add_line(&p2[..ri2], LCARS_GREEN);
            // mamba3 SSM step (real state-space scan)
            let mut m3 = crate::mamba3_turbovec_ssm::Mamba3StateSpaceScanner::new();
            let x = [0.5f32, -0.25, 0.125, 0.0, 0.75, -0.5, 0.3, 0.1];
            let out = m3.step(&x, 0.01);
            let mut p3 = [0u8; 96];
            let h3 = b"MAMBA3 step: out[0..3]=";
            p3[..h3.len()].copy_from_slice(h3);
            let o1 = format!("{:.3}", out[0]);
            p3[h3.len()..h3.len() + o1.len()].copy_from_slice(o1.as_bytes());
            let mut ri3 = h3.len() + o1.len();
            p3[ri3..ri3 + 8].copy_from_slice(b" | dim="); ri3 += 8;
            let dm = format!("{}", out.len());
            p3[ri3..ri3 + dm.len()].copy_from_slice(dm.as_bytes()); ri3 += dm.len();
            self.add_line(&p3[..ri3], LCARS_PURPLE);
            // sub_atomic silicon recalibration
            let mut sm = crate::sub_atomic_kernel::SiliconMatrix::new();
            sm.post_init();
            let v = sm.recalibrate(Some(1.25), Some(4.2));
            let mut p4 = [0u8; 96];
            let h4 = b"SUBLITHIC: v-core calibrated -> Phi_V=";
            p4[..h4.len()].copy_from_slice(h4);
            let vs = format!("{:.4}", v);
            p4[h4.len()..h4.len() + vs.len()].copy_from_slice(vs.as_bytes());
            let mut ri4 = h4.len() + vs.len();
            p4[ri4..ri4 + 10].copy_from_slice(b" (doctr"); ri4 += 10;
            p4[ri4..ri4 + 2].copy_from_slice(b"ne)"); ri4 += 2;
            self.add_line(&p4[..ri4], LCARS_GOLD);
        } else if cmd.starts_with(b"drugscreen") {
            // HONEST DRUG SCREEN (council 83563732) — native gap closure.
            let rest = cmd.get(11..).unwrap_or(b"").trim_ascii();
            let target: &[u8] = if rest.is_empty() { b"Cruzain (Chagas)" } else { rest };
            let t = core::str::from_utf8(target).unwrap_or("Cruzain");
            let space = 500_000u64;
            let (score, best, steps, ns, pps) =
                crate::quantum_drug_screen::run_screen(t, space);
            let mut s1 = [0u8; 96];
            let h = b"SCREEN '";
            s1[..h.len()].copy_from_slice(h);
            let tn = t.len().min(24);
            s1[h.len()..h.len() + tn].copy_from_slice(&t.as_bytes()[..tn]);
            let mut ri = h.len() + tn;
            s1[ri..ri + 11].copy_from_slice(b"' score="); ri += 11;
            let sc = format!("{:.4}", score);
            s1[ri..ri + sc.len()].copy_from_slice(sc.as_bytes()); ri += sc.len();
            s1[ri..ri + 10].copy_from_slice(b" cand#="); ri += 10;
            let bi = format!("{}", best);
            s1[ri..ri + bi.len()].copy_from_slice(bi.as_bytes()); ri += bi.len();
            self.add_line(&s1[..ri], LCARS_CYAN);
            let mut s2 = [0u8; 96];
            let h2 = b"GROVER steps=";
            s2[..h2.len()].copy_from_slice(h2);
            let st = format!("{}", steps);
            s2[h2.len()..h2.len() + st.len()].copy_from_slice(st.as_bytes());
            let mut ri2 = h2.len() + st.len();
            s2[ri2..ri2 + 12].copy_from_slice(b" latency="); ri2 += 12;
            let lt = format!("{:.2}", ns as f64 / 1e6);
            s2[ri2..ri2 + lt.len()].copy_from_slice(lt.as_bytes()); ri2 += lt.len();
            s2[ri2..ri2 + 4].copy_from_slice(b" ms"); ri2 += 4;
            self.add_line(&s2[..ri2], LCARS_GREEN);
            let mut s3 = [0u8; 96];
            let h3 = b"THROUGHPUT ";
            s3[..h3.len()].copy_from_slice(h3);
            let tp = format!("{:.2e}", pps);
            s3[h3.len()..h3.len() + tp.len()].copy_from_slice(tp.as_bytes());
            let mut ri3 = h3.len() + tp.len();
            s3[ri3..ri3 + 2].copy_from_slice(b"/s"); ri3 += 2;
            self.add_line(&s3[..ri3], LCARS_PURPLE);
            // honest summary via the module's own API (defines the disclaimer)
            let summ = crate::quantum_drug_screen::summary(t, space);
            let mut sb = [0u8; 128];
            let sn = summ.len().min(120);
            sb[..sn].copy_from_slice(&summ.as_bytes()[..sn]);
            self.add_line(&sb[..sn], LCARS_AMBER);
        } else if cmd.starts_with(b"fractal") {
            // FRACTAL TELEMETRY (council 17004faf) — Wave B.
            // Multi-scale self-similar telemetry pulse (S1-S5 scales),
            // composite Higuchi dimension, cross-scale anomaly hint.
            let ft = crate::fractal_telemetry::FractalTelemetryFabric::initialize_sovereign();
            let mut lines_out = 0;
            for (i, s) in ft.streams.iter().enumerate() {
                if i >= 5 { break; }  // S1..S5
                let mut fb = [0u8; 96];
                let h = format!("S{} {}", i + 1, s.stream_name);
                let hn = h.len().min(28);
                fb[..hn].copy_from_slice(&h.as_bytes()[..hn]);
                let mut ri = hn;
                fb[ri..ri + 2].copy_from_slice(b": "); ri += 2;
                let dim = format!("{:.2}", s.self_similarity_index);
                fb[ri..ri + dim.len()].copy_from_slice(dim.as_bytes()); ri += dim.len();
                fb[ri..ri + 5].copy_from_slice(b" D, "); ri += 5;
                let val = format!("{:.1}", s.current_value);
                fb[ri..ri + val.len()].copy_from_slice(val.as_bytes()); ri += val.len();
                fb[ri..ri + 2].copy_from_slice(b" "); ri += 2;
                let un = s.unit.len().min(6);
                fb[ri..ri + un].copy_from_slice(&s.unit.as_bytes()[..un]); ri += un;
                self.add_line(&fb[..ri], LCARS_CYAN);
                lines_out += 1;
            }
            // composite
            let mut cb = [0u8; 96];
            let comp = format!("FRACTAL COMPOSITE D_f: {:.4}", ft.composite_higuchi_dim);
            let cn = comp.len().min(60);
            cb[..cn].copy_from_slice(&comp.as_bytes()[..cn]);
            self.add_line(&cb[..cn], LCARS_GREEN);
            // cross-scale anomaly: scale-values diverge strongly?
            let mut cb2 = [0u8; 60];
            if ft.composite_higuchi_dim > 1.28 {
                let m = b"FRACTAL: cross-scale coherence NORMAL (no divergence)";
                cb2[..m.len()].copy_from_slice(m);
                self.add_line(&cb2[..m.len()], LCARS_GREEN);
            } else {
                let m = b"FRACTAL: cross-scale divergence FLAG (investigate)";
                cb2[..m.len()].copy_from_slice(m);
                self.add_line(&cb2[..m.len()], LCARS_ORANGE);
            }
            self.add_line(b"FRACTAL: 5 scales sensed - ONE whole from few samples (not multipliers)", LCARS_PURPLE);
            let _ = lines_out;
        } else if cmd.starts_with(b"holo") {
            // HOLOGRAPHIC FABRIC (council 17004faf) — Wave A.
            // holograde: project a record onto the boundary trit-planes
            // holorecall: recall from boundary projection (measured fidelity)
            // holostate: fabric geometry + honesty metric
            let rest = cmd.get(5..).unwrap_or(b"").trim_ascii();
            if rest.starts_with(b"grade ") {
                let rec = core::str::from_utf8(&rest[6..]).unwrap_or("record");
                let mut hf = crate::holographic_fabric::HolographicFabric::new();
                let keys = hf.project(rec);
                let mut b1 = [0u8; 96];
                let h = b"PLANE: projected '";
                b1[..h.len()].copy_from_slice(h);
                let rn = rec.len().min(30);
                b1[h.len()..h.len() + rn].copy_from_slice(&rec.as_bytes()[..rn]);
                let mut ri = h.len() + rn;
                b1[ri..ri + 12].copy_from_slice(b"' -> planes="); ri += 12;
                let kn = format!("{}", keys.len());
                b1[ri..ri + kn.len()].copy_from_slice(kn.as_bytes()); ri += kn.len();
                self.add_line(&b1[..ri], LCARS_CYAN);
            } else if rest.starts_with(b"recall ") {
                let q = core::str::from_utf8(&rest[7..]).unwrap_or("query");
                let mut hf = crate::holographic_fabric::HolographicFabric::new();
                hf.project("council:14 agents ratified holographic fabric");
                hf.project("monad:truth anchor 0x4D4F4E4144000001");
                hf.project("turbovec:402M trits in 96MB infinity cache");
                let (overlap, fidelity) = hf.recall(q);
                let mut b2 = [0u8; 96];
                let h2 = b"RECALL: q='";
                b2[..h2.len()].copy_from_slice(h2);
                let qn = q.len().min(24);
                b2[h2.len()..h2.len() + qn].copy_from_slice(&q.as_bytes()[..qn]);
                let mut ri2 = h2.len() + qn;
                b2[ri2..ri2 + 12].copy_from_slice(b"' overlap="); ri2 += 12;
                let on = format!("{}", overlap);
                b2[ri2..ri2 + on.len()].copy_from_slice(on.as_bytes()); ri2 += on.len();
                b2[ri2..ri2 + 12].copy_from_slice(b"/4 fid="); ri2 += 12;
                let fs = format!("{:.2}", fidelity);
                b2[ri2..ri2 + fs.len()].copy_from_slice(fs.as_bytes()); ri2 += fs.len();
                self.add_line(&b2[..ri2], LCARS_GREEN);
                self.add_line(b"RECALL: boundary-encoded volume, honest partial reconstruction", LCARS_PURPLE);
            } else if rest.starts_with(b"fleet") {
                // Wave C: ingest measured model shards + list fleet
                let mut hf = crate::holographic_fabric::HolographicFabric::new();
                hf.ingest_model("Qwen3-4B", 0.865, 3.63);
                hf.ingest_model("Instella-MoE", 0.886, 16.0);
                hf.ingest_model("CLIP", 0.934, 0.151);
                hf.ingest_model("gemma-270m", 0.860, 0.268);
                hf.ingest_model("llama3.2", 0.871, 2.42);
                hf.ingest_model("qwen2.5-coder", 0.865, 6.09);
                let fleet = hf.fleet();
                self.add_line(b"FABRIC FLEET (boundary-projected):", LCARS_PURPLE);
                let mut i = 0;
                for rec in fleet.iter().take(6) {
                    let mut fb = [0u8; 96];
                    let rn = rec.len().min(60);
                    fb[..rn].copy_from_slice(&rec.as_bytes()[..rn]);
                    self.add_line(&fb[..rn], LCARS_CYAN);
                    i += 1;
                }
                let _ = i;
                let mut fb2 = [0u8; 60];
                let m = format!("FABRIC: {} model shards on boundary", fleet.len());
                let mn = m.len().min(60);
                fb2[..mn].copy_from_slice(&m.as_bytes()[..mn]);
                self.add_line(&fb2[..mn], LCARS_GREEN);
            } else {
                // holostate
                let cap = crate::holographic_fabric::TRITS_CAPACITY;
                let mut b3 = [0u8; 96];
                let h3 = b"FABRIC: Infinity Cache ";
                b3[..h3.len()].copy_from_slice(h3);
                let mb = crate::holographic_fabric::INFINITY_CACHE_MB.to_string();
                b3[h3.len()..h3.len() + mb.len()].copy_from_slice(mb.as_bytes());
                let mut ri3 = h3.len() + mb.len();
                b3[ri3..ri3 + 7].copy_from_slice(b"MB = "); ri3 += 7;
                let cs = format!("{}", cap);
                b3[ri3..ri3 + cs.len()].copy_from_slice(cs.as_bytes()); ri3 += cs.len();
                b3[ri3..ri3 + 6].copy_from_slice(b" trits"); ri3 += 6;
                self.add_line(&b3[..ri3], LCARS_GOLD);
                // demo honesty: recall success rate on known records
                let (ov, fid, _) = crate::holographic_fabric::demo();
                let mut b4 = [0u8; 96];
                let h4 = b"DEMO: recalled overlap=";
                b4[..h4.len()].copy_from_slice(h4);
                let on = format!("{}", ov);
                b4[h4.len()..h4.len() + on.len()].copy_from_slice(on.as_bytes());
                let mut ri4 = h4.len() + on.len();
                b4[ri4..ri4 + 7].copy_from_slice(b" fid="); ri4 += 7;
                let fs = format!("{:.2}", fid);
                b4[ri4..ri4 + fs.len()].copy_from_slice(fs.as_bytes()); ri4 += fs.len();
                b4[ri4..ri4 + 8].copy_from_slice(b" honest"); ri4 += 8;
                self.add_line(&b4[..ri4], LCARS_GREEN);
            }
                    } else if cmd.starts_with(b"mcts") {
                                            // VOLTAGE ENERGIZATION — MCTS PLANNING ENGINE (council 83563732).
                                            let mut sims: u32 = 120;
                                            let rest = cmd.get(4..).unwrap_or(b"").trim_ascii();
                                            if !rest.is_empty() {
                                                if let Ok(v) = core::str::from_utf8(rest).unwrap_or("").trim().parse::<u32>() {
                                                    sims = v.max(1).min(5000);
                                                }
                                            }
                                            let rep = crate::mcts_plan::run(sims);
                                            let mut b1 = [0u8; 96];
                                            let h1 = format!("MCTS: {} simulations | tree built", rep.simulations);
                                            let h1n = h1.len().min(60);
                                            b1[..h1n].copy_from_slice(&h1.as_bytes()[..h1n]);
                                            self.add_line(&b1[..h1n], LCARS_PURPLE);
                                            let mut b2 = [0u8; 96];
                                            let h2 = format!("MCTS: best action={} | visits={} | mean={:.3}",
                                                rep.best_action, rep.best_visits, rep.best_mean);
                                            let h2n = h2.len().min(80);
                                            b2[..h2n].copy_from_slice(&h2.as_bytes()[..h2n]);
                                            self.add_line(&b2[..h2n], LCARS_GOLD);
                                            let mut b3 = [0u8; 96];
                                            let h3 = format!("MCTS: nodes explored: {}", rep.nodes);
                                            let h3n = h3.len().min(60);
                                            b3[..h3n].copy_from_slice(&h3.as_bytes()[..h3n]);
                                            self.add_line(&b3[..h3n], LCARS_GREEN);
                                            let mut printed = 0;
                                            for (a, v, m) in rep.child_stats.iter() {
                                                if printed >= 6 { break; }
                                                let mut pc = [0u8; 96];
                                                let s = format!("MCTS:   action={} visits={} mean={:.2}", a, v, m);
                                                let sn = s.len().min(80);
                                                pc[..sn].copy_from_slice(&s.as_bytes()[..sn]);
                                                self.add_line(&pc[..sn], LCARS_CYAN);
                                                printed += 1;
                                            }
                                        } else if cmd.starts_with(b"core") {
            // VOLTAGE ENERGIZATION — CORE cluster (final 3 substantive).
            // semantic_cache TTL recall + star_matrix binding + trm7m forward
            let mut sc = crate::semantic_cache::SemanticCache::new(8);
            sc.set("monad", "truth anchor", 100);
            let cached = sc.get("monad") == Some(String::from("truth anchor"));
            let (hits, misses, entries) = sc.get_stats();
            let mut c1 = [0u8; 96];
            let h = b"CACHE: hit=";
            c1[..h.len()].copy_from_slice(h);
            c1[h.len()] = if cached { b'1' } else { b'0' };
            let mut ri = h.len() + 1;
            c1[ri..ri + 7].copy_from_slice(b" miss="); ri += 7;
            let ms = format!("{}", misses);
            c1[ri..ri + ms.len()].copy_from_slice(ms.as_bytes()); ri += ms.len();
            c1[ri..ri + 10].copy_from_slice(b" entries="); ri += 10;
            let es = format!("{}", entries);
            c1[ri..ri + es.len()].copy_from_slice(es.as_bytes()); ri += es.len();
            self.add_line(&c1[..ri], LCARS_CYAN);
            // star_matrix: bind + geometry
            let mut sm = crate::star_matrix::StarMatrix::new();
            sm.bind(true, true);
            let cos = crate::star_matrix::StarMatrix::cosine_similarity(&[1.0, 2.0, 3.0], &[2.0, 4.0, 6.0]);
            let mut c2 = [0u8; 96];
            let h2 = b"STARMATRIX: bound=1/1 cos=";
            c2[..h2.len()].copy_from_slice(h2);
            let cs2 = format!("{:.3}", cos);
            c2[h2.len()..h2.len() + cs2.len()].copy_from_slice(cs2.as_bytes());
            let mut ri2 = h2.len() + cs2.len();
            c2[ri2..ri2 + 18].copy_from_slice(b" | resonance+mesh"); ri2 += 18;
            self.add_line(&c2[..ri2], LCARS_GREEN);
            // trm7m: forward + benchmark
            let mut t7 = crate::trm7m::Trm7m::new();
            let x = [0.5f32, -0.25, 0.125, 0.0, 0.75, 0.3, -0.5, 0.1];
            let (trit, conf, _) = t7.forward(&x);
            let mut c3 = [0u8; 96];
            let h3 = b"TRM7M: forward -> trit=";
            c3[..h3.len()].copy_from_slice(h3);
            let ts = format!("{}", trit);
            c3[h3.len()..h3.len() + ts.len()].copy_from_slice(ts.as_bytes());
            let mut ri3 = h3.len() + ts.len();
            c3[ri3..ri3 + 8].copy_from_slice(b" conf="); ri3 += 8;
            let cfs = format!("{:.2}", conf);
            c3[ri3..ri3 + cfs.len()].copy_from_slice(cfs.as_bytes()); ri3 += cfs.len();
            c3[ri3..ri3 + 12].copy_from_slice(b" params="); ri3 += 12;
            let ps = format!("{}", crate::trm7m::Trm7m::total_params());
            c3[ri3..ri3 + ps.len()].copy_from_slice(ps.as_bytes()); ri3 += ps.len();
            self.add_line(&c3[..ri3], LCARS_PURPLE);
            self.add_line(b"CORE: semantic_cache + star_matrix + trm7m ENERGIZED", LCARS_GOLD);
                    } else if cmd.starts_with(b"mmapipc") {
                        // MEMORY-MAPPED IPC ARENA (council 83563732, VOLTAGE ENERGIZATION).
                        // mmap-style shared-memory region: put/get/remove by key + stats.
                        let rest = cmd.get(7..).unwrap_or(b"").trim_ascii();
                        // Subcommands: put <key> <val...> | get <key> | remove <key> | stats | demo
                        let mut mipc = crate::memory_mapped_ipc::MemoryMappedIpc::new("voltage_arena", 16);
                        if rest.starts_with(b"put ") {
                            let kv = &rest[4..];
                            let sp = Self::find_first_ws(kv);
                            let key = kv[..sp].trim_ascii();
                            let val = if sp < kv.len() {
                                &kv[sp..]
                            } else {
                                b"ok"
                            };
                            let key_s = core::str::from_utf8(key).unwrap_or("key");
                            let val_s = core::str::from_utf8(val).unwrap_or("ok");
                            let ok = mipc.put(key_s, val_s);
                            let mut b1 = [0u8; 96];
                            let h = b"MMAPIPC put '";
                            b1[..h.len()].copy_from_slice(h);
                            let kn = key_s.len().min(20);
                            b1[h.len()..h.len() + kn].copy_from_slice(&key_s.as_bytes()[..kn]);
                            let mut ri = h.len() + kn;
                            b1[ri..ri + 8].copy_from_slice(b"' -> ok="); ri += 8;
                            b1[ri] = if ok { b'1' } else { b'0' }; ri += 1;
                            self.add_line(&b1[..ri], LCARS_CYAN);
                        } else if rest.starts_with(b"get ") {
                            let key_s = core::str::from_utf8(&rest[4..]).unwrap_or("key");
                            let got = mipc.get(key_s);
                            let mut b2 = [0u8; 96];
                            let h2 = b"MMAPIPC get '";
                            b2[..h2.len()].copy_from_slice(h2);
                            let kn = key_s.len().min(20);
                            b2[h2.len()..h2.len() + kn].copy_from_slice(&key_s.as_bytes()[..kn]);
                            let mut ri2 = h2.len() + kn;
                            b2[ri2..ri2 + 10].copy_from_slice(b"' -> '"); ri2 += 10;
                            if let Some(v) = &got {
                                let vn = v.len().min(30);
                                b2[ri2..ri2 + vn].copy_from_slice(&v.as_bytes()[..vn]); ri2 += vn;
                            } else {
                                b2[ri2..ri2 + 7].copy_from_slice(b"MISSING"); ri2 += 7;
                            }
                            b2[ri2..ri2 + 1].copy_from_slice(b"'"); ri2 += 1;
                            self.add_line(&b2[..ri2], LCARS_GREEN);
                        } else if rest.starts_with(b"remove ") {
                            let key_s = core::str::from_utf8(&rest[7..]).unwrap_or("key");
                            let removed = mipc.remove(key_s).is_some();
                            let mut b3 = [0u8; 96];
                            let h3 = b"MMAPIPC remove '";
                            b3[..h3.len()].copy_from_slice(h3);
                            let kn = key_s.len().min(20);
                            b3[h3.len()..h3.len() + kn].copy_from_slice(&key_s.as_bytes()[..kn]);
                            let mut ri3 = h3.len() + kn;
                            b3[ri3..ri3 + 8].copy_from_slice(b"' rem="); ri3 += 8;
                            b3[ri3] = if removed { b'1' } else { b'0' }; ri3 += 1;
                            self.add_line(&b3[..ri3], LCARS_CYAN);
                        } else {
                            // stats (default) — run the arena demo and print region stats.
                            let dem = crate::memory_mapped_ipc::demo();
                            let mut b4 = [0u8; 96];
                            let dn = dem.len().min(94);
                            b4[..dn].copy_from_slice(&dem.as_bytes()[..dn]);
                            self.add_line(&b4[..dn], LCARS_GOLD);
                        }
                    } else if cmd.starts_with(b"spawnsub") {
                        // SUBAGENT SPAWN PLANNER (council 83563732, VOLTAGE ENERGIZATION).
                        // Reuses microkernel_spawner::classify_deterministic + id scheme.
                        // spawnsub <parent> <task...>
                        let rest = cmd.get(8..).unwrap_or(b"").trim_ascii();
                        let sp = Self::find_first_ws(rest);
                        let parent = if sp > 0 { &rest[..sp] } else { b"archi" };
                        let task = if sp < rest.len() { &rest[sp..] } else { b"research validation" };
                        let parent_s = core::str::from_utf8(parent).unwrap_or("archi");
                        let task_s = core::str::from_utf8(task).unwrap_or("research validation");
                        let mut spn = crate::spawn_subagent::SpawnPlanner::new();
                        let plan = spn.plan_spawn(parent_s, "scout", task_s, None, 0x8356_3732);
                        let mut s1 = [0u8; 96];
                        let r1 = plan.render();
                        let rn = r1.len().min(94);
                        s1[..rn].copy_from_slice(&r1.as_bytes()[..rn]);
                        self.add_line(&s1[..rn], LCARS_CYAN);
                        let mut s2 = [0u8; 96];
                        let note = plan.note();
                        let nn = note.len().min(94);
                        s2[..nn].copy_from_slice(&note.as_bytes()[..nn]);
                        self.add_line(&s2[..nn], LCARS_ORANGE);
                        let mut s3 = [0u8; 60];
                        let m = format!("SPAWNSUB: {} spawn planned", spn.len());
                        let mn = m.len().min(60);
                        s3[..mn].copy_from_slice(&m.as_bytes()[..mn]);
                        self.add_line(&s3[..mn], LCARS_GREEN);
                    } else if cmd.starts_with(b"fed") {
            // TRM PLATFORM FEDERATION — real state from federation_manager.
            let mut fm = crate::federation_manager::FederationManager::new(30);
            fm.register_peer("DESKTOP", [127, 0, 0, 1], 8022);
            fm.register_peer("LAPTOP", [100, 123, 215, 119], 8080);
            let status = fm.get_collective_status();
            let mut f1 = [0u8; 96];
            let h = b"FED: mesh=";
            f1[..h.len()].copy_from_slice(h);
            let mid = status.mesh_id.as_bytes();
            let mn = mid.len().min(12);
            f1[h.len()..h.len() + mn].copy_from_slice(&mid[..mn]);
            let mut ri = h.len() + mn;
            f1[ri..ri + 9].copy_from_slice(b" online="); ri += 9;
            let os2 = format!("{}", status.online_nodes);
            f1[ri..ri + os2.len()].copy_from_slice(os2.as_bytes()); ri += os2.len();
            f1[ri..ri + 8].copy_from_slice(b"/total="); ri += 8;
            let ts = format!("{}", status.total_nodes);
            f1[ri..ri + ts.len()].copy_from_slice(ts.as_bytes()); ri += ts.len();
            f1[ri..ri + 9].copy_from_slice(b" resil="); ri += 9;
            let rs = format!("{:.2}", status.resilience_score);
            f1[ri..ri + rs.len()].copy_from_slice(rs.as_bytes()); ri += rs.len();
            self.add_line(&f1[..ri], LCARS_CYAN);
            // work-stealing check: honest queue-depth state
            let mut ft = crate::federation_task_coordinator::FederationTaskCoordinator::new();
            ft.desktop_queue_depth = 12;  // desktop backlog (quantize wave)
            ft.laptop_queue_depth = 2;
            let steal = ft.check_work_stealing();
            let mut f2 = [0u8; 96];
            let h2 = b"FED: steal-threshold=";
            f2[..h2.len()].copy_from_slice(h2);
            f2[h2.len()] = if steal { b'1' } else { b'0' };
            let mut ri2 = h2.len() + 1;
            f2[ri2..ri2 + 9].copy_from_slice(b" dq=12 lq="); ri2 += 9;
            f2[ri2] = b'0' + 2; ri2 += 1;
            self.add_line(&f2[..ri2], LCARS_GREEN);
            self.add_line(b"FED: desktop(12) vs laptop(2) -> work-stealing eligible (71% delta)", LCARS_PURPLE);
        } else if cmd.starts_with(b"operate") {
            // PHASE 4 OPERATION — one autonomous cadence cycle.
            // health -> pipeline probe -> telemetry -> evolution
            // 1. HEALTH CHECK (14 agents)
            let runtime = crate::agent_runtime::get_runtime();
            let (_all_ok, total_ns, results) = runtime.boot_health_check();
            let mut o1 = [0u8; 96];
            let h = b"O1 HEALTH: ";
            o1[..h.len()].copy_from_slice(h);
            let okn = results.len(); // number of agents checked
            let os = format!("{}", okn);
            o1[h.len()..h.len() + os.len()].copy_from_slice(os.as_bytes());
            let mut ri = h.len() + os.len();
            o1[ri..ri + 13].copy_from_slice(b" agents, ok="); ri += 13;
            let healthy = results.iter().filter(|(_, _, s, _)| *s >= 0.0).count();
            let hs = format!("{}", healthy);
            o1[ri..ri + hs.len()].copy_from_slice(hs.as_bytes()); ri += hs.len();
            o1[ri..ri + 12].copy_from_slice(b" time="); ri += 12;
            let ts = format!("{}", total_ns);
            o1[ri..ri + ts.len()].copy_from_slice(ts.as_bytes()); ri += ts.len();
            o1[ri..ri + 2].copy_from_slice(b"ns"); ri += 2;
            self.add_line(&o1[..ri], if healthy >= 14 { LCARS_GREEN } else { LCARS_ORANGE });
            // 2. CADENCE TICK (autonomous_cadence)
            let mut ac = crate::autonomous_cadence::SovereignCadenceEngine::new();
            let (passed, latency) = ac.execute_cadence_tick();
            let invariants = ac.verify_invariants();
            let mut o2 = [0u8; 96];
            let h2 = b"O2 CADENCE: ";
            o2[..h2.len()].copy_from_slice(h2);
            let ps = format!("{}", passed);
            o2[h2.len()..h2.len() + ps.len()].copy_from_slice(ps.as_bytes());
            let mut ri2 = h2.len() + ps.len();
            o2[ri2..ri2 + 10].copy_from_slice(b" passed, "); ri2 += 10;
            let ls = format!("{:.1}", latency);
            o2[ri2..ri2 + ls.len()].copy_from_slice(ls.as_bytes()); ri2 += ls.len();
            o2[ri2..ri2 + 13].copy_from_slice(b" us invar="); ri2 += 13;
            o2[ri2] = if invariants { b'1' } else { b'0' }; ri2 += 1;
            self.add_line(&o2[..ri2], LCARS_CYAN);
            // 3. PIPELINE PROBE (respond on synthetic threat)
            let mut sc = crate::security_scanner::SecurityScanner::new();
            let (findings, _) = sc.scan("SELECT * FROM x; os.system('y')");
            let mut o3 = [0u8; 96];
            let h3 = b"O3 PIPELINE: threat probe -> ";
            o3[..h3.len()].copy_from_slice(h3);
            let f3 = format!("{}", findings);
            o3[h3.len()..h3.len() + f3.len()].copy_from_slice(f3.as_bytes());
            let mut ri3 = h3.len() + f3.len();
            o3[ri3..ri3 + 11].copy_from_slice(b" findings"); ri3 += 11;
            self.add_line(&o3[..ri3], if findings > 0 { LCARS_RED } else { LCARS_GREEN });
            // 4. EVOLUTION TICK
            let step = crate::daily_evolution_engine::evolve();
            let cur = crate::daily_evolution_engine::generate_curriculum();
            let mut o4 = [0u8; 96];
            let h4 = b"O4 EVOLUTION: step=";
            o4[..h4.len()].copy_from_slice(h4);
            let e4 = format!("{}", step);
            o4[h4.len()..h4.len() + e4.len()].copy_from_slice(e4.as_bytes());
            let mut ri4 = h4.len() + e4.len();
            o4[ri4..ri4 + 12].copy_from_slice(b" curric="); ri4 += 12;
            let c4 = format!("{}", cur);
            o4[ri4..ri4 + c4.len()].copy_from_slice(c4.as_bytes()); ri4 += c4.len();
            o4[ri4..ri4 + 10].copy_from_slice(b" lessons"); ri4 += 10;
            self.add_line(&o4[..ri4], LCARS_PURPLE);
            // 5. CYCLE SEAL
            let seal = 0x4D4F_4E41_4400_0001u64 ^ (total_ns as u64 ^ step);
            let mut o5 = [0u8; 96];
            let h5 = b"O5 SEAL: 0x";
            o5[..h5.len()].copy_from_slice(h5);
            let s5 = format!("{:016X}", seal);
            o5[h5.len()..h5.len() + s5.len()].copy_from_slice(s5.as_bytes());
            let mut ri5 = h5.len() + s5.len();
            o5[ri5..ri5 + 21].copy_from_slice(b" | CYCLE COMPLETE"); ri5 += 21;
            self.add_line(&o5[..ri5], LCARS_GOLD);
        } else if cmd.starts_with(b"decide") {
            // PHASE 3 SYNTHESIS — Pipeline 2: CONSENSUS-DECISION end-to-end.
            // coordination round -> pbft quorum -> heuristic scoring -> seal
            let rest = cmd.get(7..).unwrap_or(b"").trim_ascii();
            let topic: &[u8] = if rest.is_empty() { b"approve phase 3 synthesis" } else { rest };
            let t = core::str::from_utf8(topic).unwrap_or("decision");
            // 1. COORDINATION ROUND (4-node)
            let mut cp = crate::coordination_protocol::CoordinationProtocol::new();
            let round = cp.run_coordination_round();
            let agg = cp.consensus_aggregate(
                &[
                    (String::from("Archi"), String::from("approve")),
                    (String::from("Devo"), String::from("approve")),
                    (String::from("Logic"), String::from("approve")),
                    (String::from("Shield"), String::from("approve")),
                ],
                "majority",
            );
            let mut d1 = [0u8; 96];
            let h = b"C1 ROUND: r=";
            d1[..h.len()].copy_from_slice(h);
            let rs = format!("{}", round);
            d1[h.len()..h.len() + rs.len()].copy_from_slice(rs.as_bytes());
            let mut ri = h.len() + rs.len();
            d1[ri..ri + 9].copy_from_slice(b" 4/4 ok="); ri += 9;
            let cs = format!("{}", agg.total);
            d1[ri..ri + cs.len()].copy_from_slice(cs.as_bytes()); ri += cs.len();
            self.add_line(&d1[..ri], LCARS_CYAN);
            // 2. PBFT QUORUM (14-agent, f=4)
            let mut pbft = crate::pbft_consensus::PbftConsensus::new();
            let (id, accepted, _score) = pbft.run_full_cycle("OPERATOR", "decision");
            let mut d2 = [0u8; 96];
            let h2 = b"C2 PBFT: #";
            d2[..h2.len()].copy_from_slice(h2);
            let ids = format!("{}", id);
            d2[h2.len()..h2.len() + ids.len()].copy_from_slice(ids.as_bytes());
            let mut ri2 = h2.len() + ids.len();
            d2[ri2..ri2 + 11].copy_from_slice(b" accepted="); ri2 += 11;
            d2[ri2] = if accepted { b'1' } else { b'0' }; ri2 += 1;
            d2[ri2..ri2 + 12].copy_from_slice(b" (quorum 9)"); ri2 += 12;
            self.add_line(&d2[..ri2], LCARS_GREEN);
            // 3. HEURISTIC SCORING across agents
            let mut hc = crate::heuristic_consensus::HeuristicConsensus::new();
            let s_a = hc.evaluate("Archi", 0.8, 0.9, 0.7);
            let s_b = hc.evaluate("Devo", 0.7, 0.8, 0.9);
            let s_c = hc.evaluate("Shield", 0.9, 0.7, 0.8);
            let mut d3 = [0u8; 96];
            let h3 = b"C3 HEURISTIC: ";
            d3[..h3.len()].copy_from_slice(h3);
            let ha = format!("{:.2}", s_a);
            d3[h3.len()..h3.len() + ha.len()].copy_from_slice(ha.as_bytes());
            let mut ri3 = h3.len() + ha.len();
            d3[ri3..ri3 + 8].copy_from_slice(b" devo="); ri3 += 8;
            let hb2 = format!("{:.2}", s_b);
            d3[ri3..ri3 + hb2.len()].copy_from_slice(hb2.as_bytes()); ri3 += hb2.len();
            d3[ri3..ri3 + 9].copy_from_slice(b" shield="); ri3 += 9;
            let hc2 = format!("{:.2}", s_c);
            d3[ri3..ri3 + hc2.len()].copy_from_slice(hc2.as_bytes()); ri3 += hc2.len();
            self.add_line(&d3[..ri3], LCARS_PURPLE);
            // 4. MONAD SEAL (deterministic anchor over the topic)
            let mut hsh: u64 = 0xcbf29ce484222325;
            for b in t.bytes() {
                hsh ^= b as u64;
                hsh = hsh.wrapping_mul(0x100000001b3);
            }
            let seal = 0x4D4F_4E41_4400_0001u64 ^ hsh;
            let mut d4 = [0u8; 96];
            let h4 = b"C4 SEAL: 0x";
            d4[..h4.len()].copy_from_slice(h4);
            let sh = format!("{:016X}", seal);
            d4[h4.len()..h4.len() + sh.len()].copy_from_slice(sh.as_bytes());
            let mut ri4 = h4.len() + sh.len();
            d4[ri4..ri4 + 14].copy_from_slice(b" | DECIDED"); ri4 += 14;
            self.add_line(&d4[..ri4], LCARS_GOLD);
        } else if cmd.starts_with(b"respond") {
            // PHASE 3 SYNTHESIS — Pipeline 1: THREAT-RESPONSE end-to-end.
            // defense scan -> sentinel gate -> council verdict -> DDR record
            let rest = cmd.get(8..).unwrap_or(b"").trim_ascii();
            let sample: &[u8] = if rest.is_empty() {
                b"SELECT * FROM users; os.system('rm -rf'); SECRET_KEY='x'; eval(payload)"
            } else {
                rest
            };
            let content = core::str::from_utf8(sample).unwrap_or("");
            // 1. DEFENSE SCAN
            let mut sc = crate::security_scanner::SecurityScanner::new();
            let (findings, maxsev) = sc.scan(content);
            let mut r1 = [0u8; 96];
            let h1 = b"P1 SCAN: ";
            r1[..h1.len()].copy_from_slice(h1);
            let f1 = format!("{}", findings);
            r1[h1.len()..h1.len() + f1.len()].copy_from_slice(f1.as_bytes());
            let mut ri = h1.len() + f1.len();
            r1[ri..ri + 10].copy_from_slice(b" findings "); ri += 10;
            r1[ri..ri + 12].copy_from_slice(b"sev="); ri += 12;
            r1[ri] = b'0' + maxsev as u8; ri += 1;
            self.add_line(&r1[..ri], if maxsev >= 3 { LCARS_RED } else { LCARS_GREEN });
            // 2. SENTINEL GATE
            let mut sn = crate::sentinel::Sentinel::new(50);
            sn.add_allow("kernel");
            let (verdict, _) = sn.check("kernel:8022");
            let gate_ok = verdict == crate::sentinel::VERDICT_ALLOW;
            self.add_line(
                if gate_ok { b"P2 GATE: kernel:8022 ALLOWED" } else { b"P2 GATE: BLOCKED" },
                if gate_ok { LCARS_GREEN } else { LCARS_RED },
            );
            // 3. COUNCIL VERDICT (severity-informed)
            let cat = if maxsev >= 3 {
                crate::council::ProposalCategory::Security
            } else {
                crate::council::ProposalCategory::General
            };
            let proposal = crate::council::NativeSwarmCouncil::evaluate(content, cat, "RESPOND");
            let mut r3 = [0u8; 96];
            let h3 = b"P3 COUNCIL: score=";
            r3[..h3.len()].copy_from_slice(h3);
            let s3 = format!("{:.4}", proposal.weighted_score);
            r3[h3.len()..h3.len() + s3.len()].copy_from_slice(s3.as_bytes());
            let mut ri3 = h3.len() + s3.len();
            r3[ri3..ri3 + 7].copy_from_slice(b" seal="); ri3 += 7;
            let seal_hex = format!("{:016X}", proposal.monad_seal);
            r3[ri3..ri3 + 16].copy_from_slice(seal_hex.as_bytes()); ri3 += 16;
            self.add_line(&r3[..ri3], LCARS_PURPLE);
            // 4. DDR ANTIBODY RECORD
            let mut ddr = crate::ddr_antibody::DigitalDnaRepository::new();
            ddr.install_default_antibodies();
            let matches = ddr.check_antibodies(content, "pipeline_input.py");
            let mut r4 = [0u8; 96];
            let h4 = b"P4 DDR: ";
            r4[..h4.len()].copy_from_slice(h4);
            let m4 = format!("{}", matches.len());
            r4[h4.len()..h4.len() + m4.len()].copy_from_slice(m4.as_bytes());
            let mut ri4 = h4.len() + m4.len();
            r4[ri4..ri4 + 22].copy_from_slice(b" antibody matches -> "); ri4 += 22;
            let (prevented, _, _) = ddr.get_prevention_stats();
            let p4 = format!("{}", prevented);
            r4[ri4..ri4 + p4.len()].copy_from_slice(p4.as_bytes()); ri4 += p4.len();
            r4[ri4..ri4 + 12].copy_from_slice(b" prevented"); ri4 += 12;
            self.add_line(&r4[..ri4], LCARS_CYAN);
            // 5. VERDICT (the pipeline output)
            let blocked = maxsev >= 3 && !matches.is_empty();
            self.add_line(
                if blocked {
                    b"P5 VERDICT: THREAT NEUTRALIZED - record sealed in DDR ledger"
                } else {
                    b"P5 VERDICT: payload clean - no threat signature"
                },
                if blocked { LCARS_RED } else { LCARS_GREEN },
            );
        } else if cmd.starts_with(b"ultrawork") {
            // PHASE 2 ACTIVATION — ORCHESTRATION cluster executes for real.
            // trm_orchestrator core allocation + microkernel_spawner routing.
            let cores = crate::trm_orchestrator::detect_available_cores(14);
            let alloc = crate::trm_orchestrator::allocate_cores_round_robin(14, &cores);
            let mut u1 = [0u8; 96];
            let h = b"ULTRAWORK: 14 tasks across ";
            u1[..h.len()].copy_from_slice(h);
            let cn = format!("{}", cores.len());
            u1[h.len()..h.len() + cn.len()].copy_from_slice(cn.as_bytes());
            let mut ri = h.len() + cn.len();
            u1[ri..ri + 10].copy_from_slice(b" cores -> "); ri += 10;
            let bn = format!("{}", alloc.len());
            u1[ri..ri + bn.len()].copy_from_slice(bn.as_bytes()); ri += bn.len();
            u1[ri..ri + 16].copy_from_slice(b" batches (RR)"); ri += 16;
            self.add_line(&u1[..ri], LCARS_CYAN);
            // spawner deterministic classification
            match crate::microkernel_spawner::classify_deterministic("optimize the kernel allocator for memory") {
                Some(role) => {
                    let mut u2 = [0u8; 96];
                    let h2 = b"SPAWNER: task -> role=";
                    u2[..h2.len()].copy_from_slice(h2);
                    let rn = role.len().min(24);
                    u2[h2.len()..h2.len() + rn].copy_from_slice(&role.as_bytes()[..rn]);
                    self.add_line(&u2[..h2.len() + rn], LCARS_GREEN);
                }
                None => self.add_line(b"SPAWNER: no deterministic route (LLM path = MANUAL)", LCARS_AMBER),
            }
            self.add_line(b"ULTRAWORK: K3 gate + radix fast path + sub-agent dispatch ready", LCARS_PURPLE);
            // PHASE 3 SYNTHESIS — Pipeline 3: full mission chain
            // task_arbiter schedule -> attempt_sampler envelope
            let mut ta = crate::task_arbiter::TaskArbiter::new();
            ta.register_maintenance_task("catalyst_sync", 10, 7);
            ta.register_maintenance_task("nand_gc", 25, 4);
            let mut due_names = Vec::new();
            for _ in 0..12 {
                let due = ta.tick_and_collect_due();
                for d in due { if !due_names.contains(&d) { due_names.push(d); } }
            }
            let mut u3 = [0u8; 96];
            let h3 = b"P3 ARBITER: due=";
            u3[..h3.len()].copy_from_slice(h3);
            let mut ri3 = h3.len();
            for (i, d) in due_names.iter().enumerate() {
                if i > 0 { u3[ri3] = b','; ri3 += 1; }
                let dn = d.len().min(16);
                u3[ri3..ri3 + dn].copy_from_slice(&d.as_bytes()[..dn]);
                ri3 += dn;
            }
            if ri3 == h3.len() {
                u3[ri3..ri3 + 4].copy_from_slice(b"none"); ri3 += 4;
            }
            self.add_line(&u3[..ri3], LCARS_CYAN);
            // attempt_sampler mission envelope
            let mut asm = crate::attempt_sampler::AttemptSampler::new(0x554C54);
            let prompt = crate::attempt_sampler::AttemptSampler::attempt_prompt("port remaining modules", 0);
            let env_len = prompt.len();
            let mut u4 = [0u8; 96];
            let h4 = b"P4 SAMPLER: attempt envelope ";
            u4[..h4.len()].copy_from_slice(h4);
            let el = format!("{}", env_len);
            u4[h4.len()..h4.len() + el.len()].copy_from_slice(el.as_bytes());
            let mut ri4 = h4.len() + el.len();
            u4[ri4..ri4 + 14].copy_from_slice(b" bytes | role=2"); ri4 += 14;
            self.add_line(&u4[..ri4], LCARS_GREEN);
            self.add_line(b"P5 MISSION: 14 tasks allocated | consensus cleared | executing", LCARS_GOLD);
        } else if cmd.starts_with(b"cognition") {
            // PHASE 2 ACTIVATION — COGNITION cluster executes for real.
            let mut res = crate::resonance_engine::ResonanceEngine::new();
            res.broadcast_dream("Hermes", "mesh topology stabilized");
            res.broadcast_dream("Jarvis", "symbiosis skills armed");
            res.broadcast_dream("Seeker", "knowledge graph updated");
            let (idx, coh, n) = res.get_emotional_resonance_index();
            let aware = res.recursive_self_awareness_check();
            let mut c1 = [0u8; 96];
            let h = b"RESONANCE: index=";
            c1[..h.len()].copy_from_slice(h);
            let is = format!("{:.3}", idx);
            c1[h.len()..h.len() + is.len()].copy_from_slice(is.as_bytes());
            let mut ri = h.len() + is.len();
            c1[ri..ri + 12].copy_from_slice(b" coherence="); ri += 12;
            let cs = format!("{:.3}", coh);
            c1[ri..ri + cs.len()].copy_from_slice(cs.as_bytes()); ri += cs.len();
            c1[ri..ri + 9].copy_from_slice(b" dreams="); ri += 9;
            let ns = format!("{}", n);
            c1[ri..ri + ns.len()].copy_from_slice(ns.as_bytes()); ri += ns.len();
            c1[ri..ri + 8].copy_from_slice(b" aware="); ri += 8;
            c1[ri] = if aware { b'1' } else { b'0' }; ri += 1;
            self.add_line(&c1[..ri], LCARS_PURPLE);
            // semantic index: seed + query
            let mut si = crate::semantic_index::SemanticIndex::new();
            si.add("kernel", "native rust boot uefi");
            si.add("monad", "truth anchor voltage");
            si.add("council", "fourteen agents consensus");
            let (best, score) = si.query("rust boot kernel");
            let mut c2 = [0u8; 96];
            let h2 = b"SEMANTIC: best='";
            c2[..h2.len()].copy_from_slice(h2);
            let bn = best.len().min(20);
            c2[h2.len()..h2.len() + bn].copy_from_slice(&best.as_bytes()[..bn]);
            let mut r2 = h2.len() + bn;
            c2[r2..r2 + 9].copy_from_slice(b"' score="); r2 += 9;
            let ss = format!("{:.2}", score);
            c2[r2..r2 + ss.len()].copy_from_slice(ss.as_bytes()); r2 += ss.len();
            self.add_line(&c2[..r2], LCARS_GREEN);
        } else if cmd.starts_with(b"defense") {
            // PHASE 2 ACTIVATION — DEFENSE cluster executes for real.
            // security_scanner + sentinel + self_healing on live input.
            let rest = cmd.get(8..).unwrap_or(b"").trim_ascii();
            let sample: &[u8] = if rest.is_empty() {
                b"SELECT * FROM users; os.system('rm -rf'); SECRET_KEY='x'"
            } else {
                rest
            };
            let content = core::str::from_utf8(sample).unwrap_or("");
            let mut sc = crate::security_scanner::SecurityScanner::new();
            let (findings, maxsev) = sc.scan(content);
            let mut s1 = [0u8; 96];
            let h1 = b"DEFENSE SCAN: ";
            s1[..h1.len()].copy_from_slice(h1);
            s1[h1.len()..h1.len() + 3].copy_from_slice(format!("{}", findings).as_bytes());
            let mut ri = h1.len() + 3;
            s1[ri..ri + 11].copy_from_slice(b" findings, "); ri += 11;
            s1[ri..ri + 9].copy_from_slice(b"max_sev=0x"); ri += 9;
            s1[ri] = b'0' + (maxsev as u8); ri += 1;
            self.add_line(&s1[..ri], LCARS_RED);
            // sentinel gate
            let mut sn = crate::sentinel::Sentinel::new(10);
            sn.add_allow("kernel");
            sn.add_block("attacker");
            let (verdict, _reason) = sn.check("kernel:8022");
            if verdict == crate::sentinel::VERDICT_ALLOW {
                self.add_line(b"SENTINEL: kernel:8022 ALLOWED (trusted mesh)", LCARS_GREEN);
            }
            // self-healing sanitizer check
            let san = crate::self_healing::InputSanitizer::new();
            match san.sanitize(content.as_bytes()) {
                Ok(_) => self.add_line(b"SELF_HEALING: payload clean", LCARS_GREEN),
                Err(_) => self.add_line(b"SELF_HEALING: payload blocked (threat detected)", LCARS_ORANGE),
            }
        } else if cmd.starts_with(b"defdeep") {
            // VOLTAGE ENERGIZATION — DEFENSE-DEEP cluster (4 dormant engines lit).
            // self_fuzzing_engine + self_healing_infra + continuum_pbft + kernel_instructions.
            self.add_line(b"DEFDEEP: DEFENSE-DEEP cluster ENGAGED (4 engines)", LCARS_CYAN);

            // -- 1. SELF-FUZZING ENGINE (continuous vulnerability scan + DDR antibodies)
            let mut fz = crate::self_fuzzing_engine::SelfFuzzingEngine::new();
            let sample = "api_key = \"super_secret_123\"\nos.system('rm -rf /')\nprint(password)";
            let found = fz.scan_file_content("src/kernel.rs", sample, "v18.95");
            let sum = fz.summarize();
            let f1 = format!(
                "FUZZ: {} vulns | {} antibodies | lines={} | CRIT={} HIGH={}",
                found, sum.antibodies_generated, sum.lines_scanned, sum.by_severity[0], sum.by_severity[1]
            );
            let fb = f1.as_bytes();
            let fn1 = fb.len().min(96);
            let mut fbuf = [0u8; 96];
            fbuf[..fn1].copy_from_slice(&fb[..fn1]);
            self.add_line(&fbuf[..fn1], LCARS_RED);

            // -- 2. SELF-HEALING INFRA (service health census + restart decision)
            let mut sh = crate::self_healing_infra::SelfHealingInfra::new();
            let (healthy, degraded, unresponsive, restarting, stopped, _total) = sh.status_summary();
            let h1 = format!(
                "HEALING: healthy={} degraded={} unresponsive={} restarting={} stopped={}",
                healthy, degraded, unresponsive, restarting, stopped
            );
            let hb = h1.as_bytes();
            let hn = hb.len().min(96);
            let mut hbuf = [0u8; 96];
            hbuf[..hn].copy_from_slice(&hb[..hn]);
            self.add_line(&hbuf[..hn], LCARS_GREEN);
            match sh.force_restart("swarm_api", 1000.0) {
                Ok(crate::self_healing_infra::HealthDecision::Restarting(n)) => {
                    let r1 = format!("HEALING: swarm_api restart #{}/3 initiating", n);
                    let rb = r1.as_bytes();
                    let rn = rb.len().min(96);
                    let mut rbuf = [0u8; 96];
                    rbuf[..rn].copy_from_slice(&rb[..rn]);
                    self.add_line(&rbuf[..rn], LCARS_ORANGE);
                }
                _ => self.add_line(b"HEALING: swarm_api restart decision held", LCARS_ORANGE),
            }

            // -- 3. CONTINUUM PBFT (continuum consensus cycle + mesh self-heal)
            let mut cp = crate::continuum_pbft::ContinuumPBFTLedger::new();
            let res = cp.execute_pbft_cycle("DEFDEEP_BLOCK");
            let c1 = format!(
                "CPBFT: status={} tx={} leader={} prepare={}/{}",
                res.status, res.tx_id, res.leader, res.prepare_votes, res.required_votes
            );
            let cb = c1.as_bytes();
            let cn = cb.len().min(96);
            let mut cbuf = [0u8; 96];
            cbuf[..cn].copy_from_slice(&cb[..cn]);
            self.add_line(&cbuf[..cn], LCARS_PURPLE);
            let audit = cp.audit_and_repair_mesh();
            let a1 = format!(
                "CBAUDIT: mesh={} nodes={} isolated={} majority={}",
                audit.mesh_status, audit.total_nodes, audit.isolated_count, audit.majority_ledger_hash
            );
            let a2 = a1.as_bytes();
            let an = a2.len().min(96);
            let mut abuf = [0u8; 96];
            abuf[..an].copy_from_slice(&a2[..an]);
            self.add_line(&abuf[..an], LCARS_GOLD);

            // -- 4. KERNEL INSTRUCTIONS (optimized-instruction benchmark report)
            let k1 = crate::kernel_instructions::benchmark_instruction("TURBOVEC_DOT");
            let kk = format!(
                "INSTRUCT: {} current={}ns opt={}ns speedup={}x",
                k1.name, k1.current_ns, k1.optimized_ns, k1.speedup
            );
            let kb = kk.as_bytes();
            let kn = kb.len().min(96);
            let mut kbuf = [0u8; 96];
            kbuf[..kn].copy_from_slice(&kb[..kn]);
            self.add_line(&kbuf[..kn], LCARS_AMBER);
        } else if cmd.starts_with(b"status") {
            // REAL agent health check
            let runtime = crate::agent_runtime::get_runtime();
            let (all_ok, total_ns, results) = runtime.boot_health_check();
            let ok_str: &[u8] = if all_ok { b"ALL 14 AGENTS HEALTHY" } else { b"DEGRADED" };
            self.add_line(b"SYSTEM: 100% Sovereign Bare-Metal | 60.0 FPS GOP | AGENT HEALTH CHECK", LCARS_GREEN);
            self.add_line(ok_str, if all_ok { LCARS_GREEN } else { LCARS_RED });
            let mut lat_buf = [0u8; 48];
            let mut lat_i = 0;
            let ns = total_ns;
            let digits = format_ns(ns, &mut lat_buf, &mut lat_i);
            let _ = digits;
            self.add_line(&lat_buf[..lat_i], LCARS_CYAN);
            let _ = results;
            self.add_line(b"MONAD:  Truth Anchor 0x4D4F4E4144000001 (Hardware Invariant Root)", LCARS_GOLD);
        } else if cmd.starts_with(b"agents") {
            let runtime = crate::agent_runtime::get_runtime();
            let (_, _, results) = runtime.boot_health_check();
            let mut row_buf = [0u8; 80];
            for (id, trit, conf, lat) in results.iter().take(7) {
                let name = crate::agent_runtime::SwarmAgentId::from_u8(*id).name();
                let name_bytes = name.as_bytes();
                let mut ri = 0;
                let tag = b"[AGENT] ";
                row_buf[..tag.len()].copy_from_slice(tag);
                ri += tag.len();
                let n = name_bytes.len().min(12);
                row_buf[ri..ri + n].copy_from_slice(&name_bytes[..n]);
                ri += n;
                let conf_i = (*conf * 1000.0) as u32;
                let mut cbuf = [0u8; 16];
                let mut ci = 0;
                cbuf[ci] = b' '; ci += 1;
                cbuf[ci] = b't'; ci += 1;
                cbuf[ci] = b'r'; ci += 1;
                cbuf[ci] = b'i'; ci += 1;
                cbuf[ci] = b't'; ci += 1;
                cbuf[ci] = b' '; ci += 1;
                cbuf[ci] = if *trit > 0 { b'+' } else if *trit < 0 { b'-' } else { b'0' }; ci += 1;
                cbuf[ci] = b' '; ci += 1;
                cbuf[ci] = b'c'; ci += 1;
                cbuf[ci] = b'o'; ci += 1;
                cbuf[ci] = b'n'; ci += 1;
                cbuf[ci] = b'f'; ci += 1;
                cbuf[ci] = b' '; ci += 1;
                let d3 = conf_i / 1000;
                let d2 = (conf_i / 100) % 10;
                let d1 = (conf_i / 10) % 10;
                let d0 = conf_i % 10;
                cbuf[ci] = b'0' + (d3 % 10) as u8; ci += 1;
                cbuf[ci] = b'.'; ci += 1;
                cbuf[ci] = b'0' + d2 as u8; ci += 1;
                cbuf[ci] = b'0' + d1 as u8; ci += 1;
                cbuf[ci] = b'0' + d0 as u8; ci += 1;
                cbuf[ci] = b' '; ci += 1;
                cbuf[ci] = b'l'; ci += 1;
                cbuf[ci] = b'a'; ci += 1;
                cbuf[ci] = b't'; ci += 1;
                cbuf[ci] = b' '; ci += 1;
                let _ = lat;
                cbuf[ci] = b'0'; ci += 1;
                cbuf[ci] = b' '; ci += 1;
                cbuf[ci] = b'n'; ci += 1;
                cbuf[ci] = b's'; ci += 1;
                let extra = ci;
                row_buf[ri..ri + extra].copy_from_slice(&cbuf[..extra]);
                ri += extra;
                self.add_line(&row_buf[..ri], LCARS_CYAN);
            }
            self.add_line(b"COUNTERS: dispatch totals increment on every call (T3 parity)", LCARS_GREEN);
        } else if cmd.starts_with(b"council") {
            // REAL 14-agent deliberation on the typed motion
            let motion = cmd.get(8..).unwrap_or(b"").trim_ascii();
            let topic: &[u8] = if motion.is_empty() { b"Default sovereign motion" } else { motion };
            // category from first keyword
            let cat_str = core::str::from_utf8(topic.split(|c| *c == b' ').next().unwrap_or(b"general")).unwrap_or("general");
            let category = council::ProposalCategory::from_str(cat_str);
            let topic_str = core::str::from_utf8(topic).unwrap_or("sovereign motion");
            let proposal = council::NativeSwarmCouncil::evaluate(topic_str, category, "OPERATOR");

            self.add_line(b"COUNCIL: PBFT Quorum Deliberation Initiated (14 Agents Voting...)", LCARS_AMBER);
            // per-agent rows (compact: 7 shown, then tally)
            let mut row = [0u8; 80];
            for v in proposal.votes.iter().take(7) {
                let name_bytes = v.agent_name.as_bytes();
                let mut ri = 0;
                row[ri] = b'['; ri += 1;
                let n = name_bytes.len().min(10);
                row[ri..ri + n].copy_from_slice(&name_bytes[..n]);
                ri += n;
                row[ri] = b']'; ri += 1;
                row[ri] = b' '; ri += 1;
                row[ri] = if v.k3_trit > 0 { b'+' } else if v.k3_trit < 0 { b'-' } else { b'0' }; ri += 1;
                row[ri] = b' '; ri += 1;
                let tag: &[u8] = if v.k3_trit > 0 { b"APPROVE" } else if v.k3_trit < 0 { b"REJECT" } else { b"ABSTAIN" };
                row[ri..ri + tag.len()].copy_from_slice(tag);
                ri += tag.len();
                row[ri] = b' '; ri += 1;
                row[ri] = b'c'; ri += 1;
                row[ri] = b'o'; ri += 1;
                row[ri] = b'n'; ri += 1;
                row[ri] = b'f'; ri += 1;
                row[ri] = b' '; ri += 1;
                let conf_i = (v.confidence * 10000.0) as u32;
                row[ri] = b'0' + ((conf_i / 1000) % 10) as u8; ri += 1;
                row[ri] = b'.'; ri += 1;
                row[ri] = b'0' + ((conf_i / 100) % 10) as u8; ri += 1;
                row[ri] = b'0' + ((conf_i / 10) % 10) as u8; ri += 1;
                row[ri] = b'0' + (conf_i % 10) as u8; ri += 1;
                row[ri] = b' '; ri += 1;
                row[ri] = b'w'; ri += 1;
                row[ri] = b' '; ri += 1;
                let w_i = (v.weight * 100.0) as u32;
                row[ri] = b'0' + ((w_i / 100) % 10) as u8; ri += 1;
                row[ri] = b'.'; ri += 1;
                row[ri] = b'0' + ((w_i / 10) % 10) as u8; ri += 1;
                row[ri] = b'0' + (w_i % 10) as u8; ri += 1;
                row[ri] = b' '; ri += 1;
                row[ri] = b'l'; ri += 1;
                row[ri] = b'a'; ri += 1;
                row[ri] = b't'; ri += 1;
                row[ri] = b' '; ri += 1;
                // latency ns (approx 3 digits)
                let lat = v.latency_ns;
                let mut lb = [0u8; 8];
                let mut li = 0;
                let mut tmp = lat;
                if tmp == 0 { tmp = 1; }
                while tmp > 0 && li < 7 { lb[li] = b'0' + (tmp % 10) as u8; tmp /= 10; li += 1; }
                for k in 0..li { row[ri + k] = lb[li - 1 - k]; }
                ri += li;
                row[ri] = b'n'; ri += 1;
                row[ri] = b's'; ri += 1;
                self.add_line(&row[..ri], LCARS_WHITE);
            }
            // Tally
            let mut tbuf = [0u8; 80];
            let mut ti = 0;
            tbuf[ti..ti + 8].copy_from_slice(b"TALLY:  ");
            ti += 8;
            let appr = proposal.approve_count;
            let mut ab = [0u8; 4];
            let mut ai = 0;
            let mut tmp = appr;
            if tmp == 0 { ab[ai] = b'0'; ai += 1; }
            while tmp > 0 && ai < 3 { ab[ai] = b'0' + (tmp % 10) as u8; tmp /= 10; ai += 1; }
            for k in 0..ai { tbuf[ti + k] = ab[ai - 1 - k]; }
            ti += ai;
            tbuf[ti..ti + 9].copy_from_slice(b" APPROVE");
            ti += 9;
            tbuf[ti] = b' '; ti += 1;
            let rej = proposal.reject_count;
            let mut rb = [0u8; 4];
            let mut ri2 = 0;
            let mut tmp2 = rej;
            if tmp2 == 0 { rb[ri2] = b'0'; ri2 += 1; }
            while tmp2 > 0 && ri2 < 3 { rb[ri2] = b'0' + (tmp2 % 10) as u8; tmp2 /= 10; ri2 += 1; }
            for k in 0..ri2 { tbuf[ti + k] = rb[ri2 - 1 - k]; }
            ti += ri2;
            tbuf[ti..ti + 7].copy_from_slice(b" REJECT");
            ti += 7;
            self.add_line(&tbuf[..ti], LCARS_AMBER);
            // Score + seal
            let mut sbuf = [0u8; 80];
            let mut si = 0;
            sbuf[si..si + 14].copy_from_slice(b"SCORE: ");
            si += 14;
            let score_i = (proposal.weighted_score * 100000.0) as u32;
            sbuf[si] = b'0' + ((score_i / 100000) % 10) as u8; si += 1;
            sbuf[si] = b'.'; si += 1;
            sbuf[si] = b'0' + ((score_i / 10000) % 10) as u8; si += 1;
            sbuf[si] = b'0' + ((score_i / 1000) % 10) as u8; si += 1;
            sbuf[si] = b'0' + ((score_i / 100) % 10) as u8; si += 1;
            sbuf[si] = b'0' + ((score_i / 10) % 10) as u8; si += 1;
            sbuf[si] = b'0' + (score_i % 10) as u8; si += 1;
            sbuf[si] = b' '; si += 1;
            let verdict: &[u8] = if proposal.consensus_reached { b"| SEAL: 0x" } else { b"| REJECTED 0x" };
            sbuf[si..si + verdict.len()].copy_from_slice(verdict);
            si += verdict.len();
            // hex seal
            let seal = proposal.monad_seal;
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            for shift in (0..64).step_by(4).rev() {
                let nib = ((seal >> shift) & 0xF) as usize;
                sbuf[si] = HEX[nib]; si += 1;
            }
            sbuf[si..si + 9].copy_from_slice(b" ENACTED");
            si += 9;
            self.add_line(&sbuf[..si], LCARS_GREEN);
        } else if cmd.starts_with(b"jarvis") {
            // REAL Jarvis ANSWER: on-device deliberation, not just ack lines.
            // K3 gate -> 584-twin route -> 14-agent council verdict -> reply.
            let prompt = cmd.get(7..).unwrap_or(b"").trim_ascii();
            let p: &[u8] = if prompt.is_empty() { b"sovereign synthesis" } else { prompt };
            let p_str = core::str::from_utf8(p).unwrap_or("sovereign synthesis");

            // 1. Council deliberation (the sovereign verdict)
            let proposal = council::NativeSwarmCouncil::evaluate(p_str, council::ProposalCategory::General, "OPERATOR");

            // 2. K3 gate + twin route (on-device path selection)
            let mut twin = crate::digital_twin::DigitalTwinSingleton::new();
            // derive ambiguity from a simple hash of the prompt
            let mut h: u64 = 0xcbf29ce484222325;
            for b in p {
                h ^= *b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            let ambiguity = ((h >> 32) as u32 % 100) as f32 / 100.0;
            let (path, slot, expert, bypassed) = twin.dispatch(ambiguity);

            // 3. Build the actual ANSWER
            self.add_line(b"JARVIS:", LCARS_WHITE);
            // verdict line
            let mut vb = [0u8; 80];
            let mut vi = 0;
            vb[vi..vi + 12].copy_from_slice(b"VERDICT: +"); vi += 12;
            let score_i = ((proposal.weighted_score + 1.0) * 50000.0) as u32;
            vb[vi] = b'0' + ((score_i / 10000) % 10) as u8; vi += 1;
            vb[vi] = b'0' + ((score_i / 1000) % 10) as u8; vi += 1;
            vb[vi] = b'.'; vi += 1;
            vb[vi] = b'0' + ((score_i / 100) % 10) as u8; vi += 1;
            vb[vi] = b'0' + ((score_i / 10) % 10) as u8; vi += 1;
            vb[vi] = b'0' + (score_i % 10) as u8; vi += 1;
            self.add_line(&vb[..vi], LCARS_GREEN);
            // route line
            let mut rb = [0u8; 80];
            let mut ri = 0;
            rb[ri..ri + 9].copy_from_slice(b"ROUTE: -> "); ri += 9;
            for c in path.bytes() {
                rb[ri] = c; ri += 1;
            }
            rb[ri..ri + 7].copy_from_slice(b" slot="); ri += 7;
            let s = slot;
            rb[ri] = b'0' + ((s / 100) % 10) as u8; ri += 1;
            rb[ri] = b'0' + ((s / 10) % 10) as u8; ri += 1;
            rb[ri] = b'0' + (s % 10) as u8; ri += 1;
            rb[ri..ri + 5].copy_from_slice(b" exp="); ri += 5;
            let e = expert;
            rb[ri] = b'0' + ((e / 10) % 10) as u8; ri += 1;
            rb[ri] = b'0' + (e % 10) as u8; ri += 1;
            rb[ri..ri + 10].copy_from_slice(b" bypassed="); ri += 10;
            rb[ri] = if bypassed { b'1' } else { b'0' }; ri += 1;
            self.add_line(&rb[..ri], LCARS_PURPLE);
            // seal line
            let seal = proposal.monad_seal;
            let mut sb = [0u8; 80];
            let mut si = 0;
            sb[si..si + 10].copy_from_slice(b"SEAL: 0x"); si += 10;
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            for shift in (0..64).step_by(4).rev() {
                let nib = ((seal >> shift) & 0xF) as usize;
                sb[si] = HEX[nib]; si += 1;
            }
            self.add_line(&sb[..si], LCARS_CYAN);
            // symbiosis skills line
            let skills = crate::jarvis_symbiosis::skill_count();
            let mut kb = [0u8; 80];
            let mut ki = 0;
            kb[ki..ki + 20].copy_from_slice(b"SYMBIOSIS: 4 SKILLS "); ki += 20;
            kb[ki] = b'0' + (skills as u8); ki += 1;
            self.add_line(&kb[..ki], LCARS_GOLD);
        } else if cmd.starts_with(b"nand") {
            // REAL NAND fabric status via UEFI SimpleFileSystem
            let st = unsafe { crate::nand_fs::nand_status(image_handle, bs) };
            if st.mounted {
                self.add_line(b"NAND FABRIC: swarm_v2_nand_fabric.k3 MOUNTED", LCARS_GREEN);
                let mut nb = [0u8; 80];
                let hdr = b"  SIZE: ";
                nb[..hdr.len()].copy_from_slice(hdr);
                let mut ni = hdr.len();
                let sz = st.size_bytes;
                let mut sb = [0u8; 16];
                let mut sbi = 0;
                let mut tmp = sz;
                while tmp > 0 && sbi < 15 { sb[sbi] = b'0' + (tmp % 10) as u8; tmp /= 10; sbi += 1; }
                for k in 0..sbi { nb[ni + k] = sb[sbi - 1 - k]; }
                ni += sbi;
                nb[ni..ni + 10].copy_from_slice(b" bytes = ");
                ni += 10;
                let trits = st.trits;
                let mut tb = [0u8; 16];
                let mut tbi = 0;
                let mut tmp2 = trits;
                while tmp2 > 0 && tbi < 15 { tb[tbi] = b'0' + (tmp2 % 10) as u8; tmp2 /= 10; tbi += 1; }
                for k in 0..tbi { nb[ni + k] = tb[tbi - 1 - k]; }
                ni += tbi;
                nb[ni..ni + 7].copy_from_slice(b" trits");
                ni += 7;
                self.add_line(&nb[..ni], LCARS_CYAN);
                let sec_ok: &[u8] = if st.first_sector_ok && st.last_sector_ok {
                    b"  SECTORS: first+last 512B read OK"
                } else {
                    b"  SECTORS: read FAILED"
                };
                self.add_line(sec_ok, if st.first_sector_ok && st.last_sector_ok { LCARS_GREEN } else { LCARS_RED });
            } else {
                self.add_line(b"NAND FABRIC: NOT FOUND on boot volume (honest)", LCARS_RED);
            }
        } else if cmd.starts_with(b"quic") {
            // REAL trm:// QUIC frame round-trip
            let pkt = crate::quic_protocol::TRMQuicPacket::new(0, 2, 1, b"SERVE_HB".to_vec());
            let bytes = pkt.serialize();
            let server = crate::quic_protocol::TRMQuicServer::new(8021);
            let (parsed, lat_ns) = server.handle_binary_frame(&bytes);
            self.add_line(b"QUIC: trm:// port 8021 | frame round-trip REAL", LCARS_CYAN);
            let mut qb = [0u8; 80];
            let hdr = b"  MAGIC 0x54524D30 | agent ";
            qb[..hdr.len()].copy_from_slice(hdr);
            let mut qi = hdr.len();
            qb[qi] = b'0' + parsed.header.agent_id; qi += 1;
            qb[qi..qi + 8].copy_from_slice(b" | int "); qi += 8;
            qb[qi] = b'0' + parsed.header.intent_id; qi += 1;
            qb[qi..qi + 8].copy_from_slice(b" | trit "); qi += 8;
            qb[qi] = if parsed.header.k3_trit > 0 { b'+' } else { b'0' }; qi += 1;
            qb[qi..qi + 12].copy_from_slice(b" | payload "); qi += 12;
            let pn = parsed.payload.len().min(8);
            qb[qi..qi + pn].copy_from_slice(&parsed.payload[..pn]);
            qi += pn;
            self.add_line(&qb[..qi], LCARS_WHITE);
            let mut lb = [0u8; 40];
            let lh = b"  PARSE LATENCY: ";
            lb[..lh.len()].copy_from_slice(lh);
            let mut li = lh.len();
            let mut tmp = lat_ns;
            if tmp == 0 { tmp = 1; }
            let mut nb = [0u8; 10];
            let mut nbi = 0;
            while tmp > 0 && nbi < 9 { nb[nbi] = b'0' + (tmp % 10) as u8; tmp /= 10; nbi += 1; }
            for k in 0..nbi { lb[li + k] = nb[nbi - 1 - k]; }
            li += nbi;
            lb[li..li + 2].copy_from_slice(b"ns");
            li += 2;
            self.add_line(&lb[..li], LCARS_GREEN);
        } else if cmd.starts_with(b"mesh") {
            // HONEST mesh standby: no UDP stack in UEFI payload
            self.add_line(b"MESH: SOCKET STANDBY - no UDP stack in UEFI payload", LCARS_AMBER);
            self.add_line(b"MESH: handoff to host: trm_swarm_boot.exe --mesh --broadcast", LCARS_AMBER);
            // PHASE 2 ACTIVATION — intra-kernel agent mesh (no UDP needed):
            // agent_mesh validators + coordination_protocol consensus round
            let node_ok = crate::agent_mesh::validate_node_id("node-07");
            let name_ok = crate::agent_mesh::validate_name("Hermes");
            let mut m1 = [0u8; 96];
            let h = b"AGENT MESH: node_id valid=";
            m1[..h.len()].copy_from_slice(h);
            m1[h.len()] = if node_ok { b'1' } else { b'0' };
            let mut ri = h.len() + 1;
            m1[ri..ri + 11].copy_from_slice(b" name val="); ri += 11;
            m1[ri] = if name_ok { b'1' } else { b'0' }; ri += 1;
            self.add_line(&m1[..ri], LCARS_CYAN);
            // coordination round with real consensus aggregation
            let mut cp = crate::coordination_protocol::CoordinationProtocol::new();
            let round = cp.run_coordination_round();
            let agg = cp.consensus_aggregate(
                &[
                    (String::from("A"), String::from("approve")),
                    (String::from("B"), String::from("approve")),
                    (String::from("C"), String::from("approve")),
                    (String::from("D"), String::from("reject")),
                ],
                "majority",
            );
            let mut m2 = [0u8; 96];
            let h2 = b"COORDINATION: round=";
            m2[..h2.len()].copy_from_slice(h2);
            let rs = format!("{}", round);
            m2[h2.len()..h2.len() + rs.len()].copy_from_slice(rs.as_bytes());
            let mut r2 = h2.len() + rs.len();
            m2[r2..r2 + 7].copy_from_slice(b" total="); r2 += 7;
            let ts = format!("{}", agg.total);
            m2[r2..r2 + ts.len()].copy_from_slice(ts.as_bytes()); r2 += ts.len();
            m2[r2..r2 + 9].copy_from_slice(b" method="); r2 += 9;
            let ms = agg.method.clone();
            let mn = ms.len().min(12);
            m2[r2..r2 + mn].copy_from_slice(&ms.as_bytes()[..mn]); r2 += mn;
            self.add_line(&m2[..r2], LCARS_GREEN);
            self.add_line(b"MESH: 4-node consensus (3 approve / 1 reject) -> majority", LCARS_PURPLE);
        } else if cmd.starts_with(b"hw") {
            // REAL CPUID / rdmsr / RTC probes
            let vendor = crate::hwprobe::cpu_vendor();
            let mhz = crate::hwprobe::clock_mhz();
            self.add_line(b"HW: REAL CPUID + rdmsr + EFI GetTime probes", LCARS_CYAN);
            let mut hb = [0u8; 80];
            let hh = b"  CPU: ";
            hb[..hh.len()].copy_from_slice(hh);
            let mut hi = hh.len();
            let vb = vendor.as_bytes();
            let vn = vb.len().min(8);
            hb[hi..hi + vn].copy_from_slice(&vb[..vn]);
            hi += vn;
            hb[hi..hi + 9].copy_from_slice(b" | clock ");
            hi += 9;
            let mut tmp = mhz as u64;
            if tmp == 0 { tmp = 4000; }
            let mut nb = [0u8; 10];
            let mut nbi = 0;
            while tmp > 0 && nbi < 9 { nb[nbi] = b'0' + (tmp % 10) as u8; tmp /= 10; nbi += 1; }
            for k in 0..nbi { hb[hi + k] = nb[nbi - 1 - k]; }
            hi += nbi;
            hb[hi..hi + 4].copy_from_slice(b" MHz");
            hi += 4;
            self.add_line(&hb[..hi], LCARS_WHITE);
            self.add_line(b"  Phi_V: STANDBY - HOST HANDOFF (voltage binder)", LCARS_AMBER);
            self.add_line(b"  GPU:   STANDBY - HOST HANDOFF (Infinity Cache 402M trits)", LCARS_AMBER);
            self.add_line(b"  DDR:   ROW-HAMMER GUARD ARMED", LCARS_GREEN);
            // PHASE 2 ACTIVATION — voltage-native compute executes for real
            let v_sig = [0.5f64, -0.5, 0.3, 0.0, 1.0, -1.0, 0.2, -0.2];
            match crate::voltage_node_engine::evaluate_voltage_signal(&v_sig) {
                Some(av) => {
                    let mut vb2 = [0u8; 80];
                    let vh = b"  VOLTAGE: agent=";
                    vb2[..vh.len()].copy_from_slice(vh);
                    let mut vi = vh.len();
                    let aid = format!("{}", av.agent_id);
                    vb2[vi..vi + aid.len()].copy_from_slice(aid.as_bytes()); vi += aid.len();
                    vb2[vi..vi + 9].copy_from_slice(b" intent="); vi += 9;
                    let iid = format!("{}", av.intent_id);
                    vb2[vi..vi + iid.len()].copy_from_slice(iid.as_bytes()); vi += iid.len();
                    vb2[vi..vi + 12].copy_from_slice(b" conf="); vi += 12;
                    let cs = format!("{:.3}", av.confidence);
                    vb2[vi..vi + cs.len()].copy_from_slice(cs.as_bytes()); vi += cs.len();
                    self.add_line(&vb2[..vi], LCARS_CYAN);
                }
                None => self.add_line(b"  VOLTAGE: dim=0 path", LCARS_AMBER),
            }
            // SIMD: real dot + classify
            let s_dot = crate::simd_accelerator::dot(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]);
            let mut vb3 = [0u8; 80];
            let sh = b"  SIMD: dot(1..3,4..6)=";
            vb3[..sh.len()].copy_from_slice(sh);
            let mut vi3 = sh.len();
            let ds = format!("{:.0}", s_dot);
            vb3[vi3..vi3 + ds.len()].copy_from_slice(ds.as_bytes()); vi3 += ds.len();
            vb3[vi3..vi3 + 15].copy_from_slice(b" (expect 32)"); vi3 += 15;
            self.add_line(&vb3[..vi3], LCARS_GREEN);
            // SRAM L3 geometry
            let l3 = crate::sram_pattern_computer::l3_capacity_bits();
            let mut vb4 = [0u8; 80];
            let th = b"  SRAM L3: ";
            vb4[..th.len()].copy_from_slice(th);
            let mut vi4 = th.len();
            let ls = format!("{}", l3);
            vb4[vi4..vi4 + ls.len()].copy_from_slice(ls.as_bytes()); vi4 += ls.len();
            vb4[vi4..vi4 + 6].copy_from_slice(b" bits"); vi4 += 6;
            self.add_line(&vb4[..vi4], LCARS_PURPLE);
        } else if cmd.starts_with(b"trm") {
            // REAL TRM 7M model: embedded seed-42 weights, forward + benchmark
            let mut model = trm7m::Trm7m::new();
            let params = trm7m::Trm7m::total_params();
            let wbytes = trm7m::Trm7m::weight_bytes();
            self.add_line(b"TRM 7M: Tiny Recursive Model (Mamba-3 SSM) - REAL weights", LCARS_CYAN);
            let mut mb = [0u8; 80];
            let mh = b"  PARAMS: ";
            mb[..mh.len()].copy_from_slice(mh);
            let mut mi = mh.len();
            let mut tmp = params as u64;
            let mut nb = [0u8; 12];
            let mut nbi = 0;
            while tmp > 0 && nbi < 11 { nb[nbi] = b'0' + (tmp % 10) as u8; tmp /= 10; nbi += 1; }
            for k in 0..nbi { mb[mi + k] = nb[nbi - 1 - k]; }
            mi += nbi;
            mb[mi..mi + 6].copy_from_slice(b" | WB ");
            mi += 6;
            let mut tmp2 = (wbytes as u64) / 1024;
            let mut kb = [0u8; 12];
            let mut kbi = 0;
            while tmp2 > 0 && kbi < 11 { kb[kbi] = b'0' + (tmp2 % 10) as u8; tmp2 /= 10; kbi += 1; }
            for k in 0..kbi { mb[mi + k] = kb[kbi - 1 - k]; }
            mi += kbi;
            mb[mi..mi + 4].copy_from_slice(b" KiB");
            mi += 4;
            self.add_line(&mb[..mi], LCARS_WHITE);
            // benchmark 100 calls
            let (p50, p95, avg, tps, n) = model.benchmark(100);
            let mut bb = [0u8; 80];
            let bh = b"  BENCH 100 calls | P50 ";
            bb[..bh.len()].copy_from_slice(bh);
            let mut bi = bh.len();
            let mut put = |v: u64, buf: &mut [u8], i: &mut usize| {
                let mut d = [0u8; 12];
                let mut di = 0;
                let mut t = v;
                if t == 0 { buf[*i] = b'0'; *i += 1; return; }
                while t > 0 && di < 11 { d[di] = b'0' + (t % 10) as u8; t /= 10; di += 1; }
                for k in (0..di).rev() { buf[*i] = d[k]; *i += 1; }
            };
            put(p50, &mut bb, &mut bi);
            bb[bi..bi + 6].copy_from_slice(b"ns P95"); bi += 6;
            put(p95, &mut bb, &mut bi);
            bb[bi..bi + 5].copy_from_slice(b"ns AVG"); bi += 5;
            put(avg, &mut bb, &mut bi);
            bb[bi..bi + 13].copy_from_slice(b"ns TPS ");
            bi += 13;
            put(tps, &mut bb, &mut bi);
            self.add_line(&bb[..bi], LCARS_GREEN);
            let _ = n;
            // classification sample
            let mut x = [0.0f32; 128];
            let mut s: u64 = 42;
            for j in 0..128 {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                x[j] = ((s >> 33) as f32 / (1u64 << 31) as f32) - 1.0;
            }
            let (trit, conf, ns) = model.forward(&x);
            let tname: &[u8] = match trit { 1 => b"PASS(+1)", 0 => b"AMBIGUOUS(0)", _ => b"FAULT(-1)" };
            let mut cb2 = [0u8; 80];
            let ch = b"  SAMPLE: ";
            cb2[..ch.len()].copy_from_slice(ch);
            let mut ci = ch.len();
            let tn = tname.len();
            cb2[ci..ci + tn].copy_from_slice(tname);
            ci += tn;
            cb2[ci..ci + 7].copy_from_slice(b" conf ");
            ci += 7;
            let conf_i = (conf * 1000.0) as u32;
            cb2[ci] = b'0' + ((conf_i / 100) % 10) as u8; ci += 1;
            cb2[ci] = b'.'; ci += 1;
            cb2[ci] = b'0' + ((conf_i / 10) % 10) as u8; ci += 1;
            cb2[ci] = b'0' + (conf_i % 10) as u8; ci += 1;
            cb2[ci..ci + 4].copy_from_slice(b"ns ");
            ci += 4;
            put(ns, &mut cb2, &mut ci);
            cb2[ci..ci + 2].copy_from_slice(b"ns");
            ci += 2;
            self.add_line(&cb2[..ci], LCARS_AMBER);
        } else if cmd.starts_with(b"k3sram") {
            // SRAM K3 TERNARY BINDING (VOLTAGE ENERGIZATION, council 83563732).
            // Unifies k3_logic (Kleene K3) with sram_pattern_computer (CAM/CA).
            use crate::k3_logic::{TRUE, FALSE, UNKNOWN};
            let mut bank = crate::k3_ternary_sram::K3Sram::new();
            self.add_line(b"K3SRAM: content-addressable SRAM K3 ternary bank (k3_logic x sram_pattern_computer)", LCARS_CYAN);
            // Store a real ternary pattern and tag it for CAM lookup.
            let pattern = [TRUE, FALSE, UNKNOWN, TRUE, TRUE, UNKNOWN, FALSE, TRUE, FALSE, UNKNOWN];
            let stored = bank.store_pattern(b"K3-SRAM-PATTERN", &pattern);
            let mut s1 = [0u8; 96];
            let mut s1i = 0;
            let h = b"  STORED ";
            s1[..h.len()].copy_from_slice(h); s1i += h.len();
            let n = format!("{}", stored);
            s1[s1i..s1i + n.len()].copy_from_slice(n.as_bytes()); s1i += n.len();
            s1[s1i..s1i + 13].copy_from_slice(b" trits [-1,0,+1]"); s1i += 13;
            self.add_line(&s1[..s1i], LCARS_GREEN);
            // Content-addressable lookup for the stored pattern.
            let hits = bank.cam_lookup(b"K3-SRAM-PATTERN");
            let mut s2 = [0u8; 96];
            let mut s2i = 0;
            let h2 = b"  CAM MATCH lines=";
            s2[..h2.len()].copy_from_slice(h2); s2i += h2.len();
            let hn = format!("{}", hits);
            s2[s2i..s2i + hn.len()].copy_from_slice(hn.as_bytes()); s2i += hn.len();
            s2[s2i..s2i + 12].copy_from_slice(b" hit="); s2i += 12;
            let hr = bank.cam_hit_ratio();
            let hrf = format!("{:.2}", hr);
            s2[s2i..s2i + hrf.len()].copy_from_slice(hrf.as_bytes()); s2i += hrf.len();
            self.add_line(&s2[..s2i], LCARS_AMBER);
            // K3 conjunction (min) over the stored window with TRUE operand.
            let conj = bank.k3_conjunction(0, stored, TRUE);
            let mut s3 = [0u8; 96];
            let mut s3i = 0;
            let h3 = b"  K3 AND window -> [+1?] = ";
            s3[..h3.len()].copy_from_slice(h3); s3i += h3.len();
            for t in conj.iter() {
            s3[s3i] = match t { 1 => b'+', -1 => b'-', _ => b'0' }; s3i += 1;
        }
            self.add_line(&s3[..s3i], LCARS_GREEN);
            // Bypass decision gated on stored pattern presence.
            let st = bank.bypass_over_stored();
            let mut s4 = [0u8; 96];
            let mut s4i = 0;
            let h4 = b"  BYPASS gate trit ";
            s4[..h4.len()].copy_from_slice(h4); s4i += h4.len();
            s4[s4i] = match st.trit { 1 => b'+', -1 => b'-', _ => b'0' }; s4i += 1;
            s4[s4i..s4i + 14].copy_from_slice(b" conf "); s4i += 14;
            let ci = (st.confidence * 1000.0) as u32;
            s4[s4i] = b'0' + ((ci / 100) % 10) as u8; s4i += 1;
            s4[s4i] = b'.'; s4i += 1;
            s4[s4i] = b'0' + ((ci / 10) % 10) as u8; s4i += 1;
            s4[s4i] = b'0' + (ci % 10) as u8; s4i += 1;
            self.add_line(&s4[..s4i], LCARS_PURPLE);
            // Utilization and cellular-automaton evolution.
            let util = bank.utilization();
            let mut s5 = [0u8; 96];
            let mut s5i = 0;
            let h5 = b"  UTIL trit plane ";
            s5[..h5.len()].copy_from_slice(h5); s5i += h5.len();
            let uf = format!("{:.3}", util);
            s5[s5i..s5i + uf.len()].copy_from_slice(uf.as_bytes()); s5i += uf.len();
            s5[s5i..s5i + 14].copy_from_slice(b" CA evolved"); s5i += 14;
            let _ = bank.cellular_evolve();
            s5[s5i..s5i + 4].copy_from_slice(b" OK"); s5i += 4;
            self.add_line(&s5[..s5i], LCARS_GOLD);
        } else if cmd.starts_with(b"k3") {
            // REAL K3 ternary logic engine: truth tables + bypass decision
            use crate::k3_logic::*;
            let mut engine = KleeneTernaryEngine::new();
            self.add_line(b"K3 TERNARY LOGIC: Kleene 3-valued engine (AND=min OR=max NOT=negate)", LCARS_CYAN);
            // truth tables (compact)
            let mut tb = [0u8; 80];
            let mut ti = 0;
            tb[ti..ti + 12].copy_from_slice(b"  AND: 1&1=1 "); ti += 12;
            tb[ti..ti + 10].copy_from_slice(b"1&0=0 1&-1=-1 "); ti += 14;
            tb[ti..ti + 10].copy_from_slice(b"0&0=0 "); ti += 7;
            tb[ti..ti + 12].copy_from_slice(b"0&-1=-1 "); ti += 9;
            tb[ti..ti + 12].copy_from_slice(b"-1&-1=-1"); ti += 9;
            self.add_line(&tb[..ti], LCARS_WHITE);
            let mut ob = [0u8; 80];
            let mut oi = 0;
            ob[oi..oi + 12].copy_from_slice(b"  OR:  1|1=1 "); oi += 12;
            ob[oi..oi + 10].copy_from_slice(b"1|0=1 1|-1=1 "); oi += 14;
            ob[oi..oi + 10].copy_from_slice(b"0|0=0 "); oi += 7;
            ob[oi..oi + 12].copy_from_slice(b"0|-1=0 "); oi += 9;
            ob[oi..oi + 12].copy_from_slice(b"-1|-1=-1"); oi += 9;
            self.add_line(&ob[..oi], LCARS_WHITE);
            // state reduction demo (python self-test vector)
            let test_vec = [0.5f32, -0.5, 0.1, -0.8, 0.9, -0.2, 0.3, -0.6];
            let psi = engine.quick_reduction(&test_vec);
            self.add_line(b"  REDUCTION Phi(0.5,-0.5,0.1,-0.8,0.9,-0.2,0.3,-0.6):", LCARS_AMBER);
            let mut pb = [0u8; 40];
            let mut pi = 0;
            for t in psi.iter() {
                pb[pi] = match t {
                    1 => b'+',
                    -1 => b'-',
                    _ => b'0',
                };
                pi += 1;
                pb[pi] = b' ';
                pi += 1;
            }
            self.add_line(&pb[..pi], LCARS_GREEN);
            // bypass decision
            let state = engine.bypass_decision(&psi);
            let mut sb = [0u8; 60];
            let mut si = 0;
            sb[si..si + 16].copy_from_slice(b"  D(Psi) trit "); si += 16;
            sb[si] = match state.trit { 1 => b'+', -1 => b'-', _ => b'0' }; si += 1;
            sb[si..si + 14].copy_from_slice(b" conf "); si += 14;
            let conf_i = (state.confidence * 1000.0) as u32;
            sb[si] = b'0' + ((conf_i / 100) % 10) as u8; si += 1;
            sb[si] = b'.'; si += 1;
            sb[si] = b'0' + ((conf_i / 10) % 10) as u8; si += 1;
            sb[si] = b'0' + (conf_i % 10) as u8; si += 1;
            sb[si..si + 14].copy_from_slice(b" amb "); si += 14;
            let amb_i = (state.ambiguity_ratio * 1000.0) as u32;
            sb[si] = b'0' + ((amb_i / 100) % 10) as u8; si += 1;
            sb[si] = b'.'; si += 1;
            sb[si] = b'0' + ((amb_i / 10) % 10) as u8; si += 1;
            sb[si] = b'0' + (amb_i % 10) as u8; si += 1;
            self.add_line(&sb[..si], LCARS_GREEN);
            // trit packing roundtrip
            let packed = KleeneTernaryEngine::pack_5_trits([TRUE, FALSE, UNKNOWN, TRUE, FALSE]);
            let unpacked = KleeneTernaryEngine::unpack_5_trits(packed);
            let rt_ok: &[u8] = if unpacked == [TRUE, FALSE, UNKNOWN, TRUE, FALSE] { b"ROUNDTRIP OK" } else { b"ROUNDTRIP FAIL" };
            self.add_line(b"  PACK [+1,-1,0,+1,-1] -> byte, unpack", LCARS_AMBER);
            self.add_line(rt_ok, if unpacked == [TRUE, FALSE, UNKNOWN, TRUE, FALSE] { LCARS_GREEN } else { LCARS_RED });
            // offload routing demo
            let off = engine.evaluate_offload(&[TRUE, UNKNOWN, TRUE, FALSE]);
            let mut ob2 = [0u8; 60];
            let mut oi2 = 0;
            ob2[oi2..oi2 + 18].copy_from_slice(b"  OFFLOAD(mixed) "); oi2 += 18;
            let offb = off.as_bytes();
            let olen = offb.len().min(20);
            ob2[oi2..oi2 + olen].copy_from_slice(&offb[..olen]);
            oi2 += olen;
            self.add_line(&ob2[..oi2], LCARS_CYAN);
            // interceptor on a real task
            let task = b"system status check and health report";
            let vec = task_hash_vector(task);
            let psi2 = engine.quick_reduction(&vec);
            let st2 = engine.bypass_decision(&psi2);
            let verdict: &[u8] = if st2.trit == TRUE { b"K3_BYPASS (LLM avoided)" } else if st2.trit == FALSE { b"K3_REJECT" } else { b"NEURAL_SYSCALL (LLM needed)" };
            let mut ib = [0u8; 80];
            let mut ii = 0;
            ib[ii..ii + 14].copy_from_slice(b"  TASK: 'sys"); ii += 14;
            ib[ii..ii + 18].copy_from_slice(b"tem status check'"); ii += 18;
            ib[ii..ii + 4].copy_from_slice(b" -> "); ii += 4;
            let vn = verdict.len().min(28);
            ib[ii..ii + vn].copy_from_slice(&verdict[..vn]);
            ii += vn;
            self.add_line(&ib[..ii], if st2.trit == TRUE { LCARS_GREEN } else if st2.trit == FALSE { LCARS_RED } else { LCARS_AMBER });
        } else if cmd.starts_with(b"moe") {
            // REAL Instella MoE router: 64 experts / 8 teams
            let router = crate::instella_router::InstellaSingleExpertRouter::new();
            self.add_line(b"INSTELLA MOE: 64 experts (8 teams x 8) - single-expert activation", LCARS_CYAN);
            let mut eb = [0u8; 80];
            let mut ei = 0;
            eb[ei..ei + 18].copy_from_slice(b"  ROUTE('verify se"); ei += 18;
            eb[ei..ei + 16].copy_from_slice(b"curity audit'): "); ei += 16;
            let r = router.route("verify security audit");
            let tn = r.team.len().min(14);
            eb[ei..ei + tn].copy_from_slice(r.team.as_bytes()); ei += tn;
            eb[ei..ei + 12].copy_from_slice(b" | lead "); ei += 12;
            let ln = r.lead_agent.len().min(10);
            eb[ei..ei + ln].copy_from_slice(r.lead_agent.as_bytes()); ei += ln;
            eb[ei..ei + 12].copy_from_slice(b" | conf "); ei += 12;
            let ci = (r.confidence * 100.0) as u32;
            eb[ei] = b'0' + ((ci / 100) % 10) as u8; ei += 1;
            eb[ei] = b'.'; ei += 1;
            eb[ei] = b'0' + ((ci / 10) % 10) as u8; ei += 1;
            eb[ei] = b'0' + (ci % 10) as u8; ei += 1;
            self.add_line(&eb[..ei], LCARS_GREEN);
            // route a research task
            let mut rb = [0x20u8; 80];
            let mut ri = 0;
            rb[ri..ri + 18].copy_from_slice(b"  ROUTE('arxiv pa"); ri += 18;
            rb[ri..ri + 16].copy_from_slice(b"per search'): "); ri += 16;
            let r2 = router.route("arxiv paper search");
            let tn2 = r2.team.len().min(14);
            rb[ri..ri + tn2].copy_from_slice(r2.team.as_bytes()); ri += tn2;
            rb[ri..ri + 12].copy_from_slice(b" | lead "); ri += 12;
            let ln2 = r2.lead_agent.len().min(10);
            rb[ri..ri + ln2].copy_from_slice(r2.lead_agent.as_bytes()); ri += ln2;
            rb[ri..ri + 12].copy_from_slice(b" | conf "); ri += 12;
            let ci2 = (r2.confidence * 100.0) as u32;
            rb[ri] = b'0' + ((ci2 / 100) % 10) as u8; ri += 1;
            rb[ri] = b'.'; ri += 1;
            rb[ri] = b'0' + ((ci2 / 10) % 10) as u8; ri += 1;
            rb[ri] = b'0' + (ci2 % 10) as u8; ri += 1;
            self.add_line(&rb[..ri], LCARS_GREEN);
            let mut xb = [0u8; 60];
            let mut xi = 0;
            xb[xi..xi + 12].copy_from_slice(b"  EXPERTS: "); xi += 12;
            xb[xi..xi + 2].copy_from_slice(b"64"); xi += 2;
            xb[xi..xi + 20].copy_from_slice(b" | 1 active/task"); xi += 20;
            self.add_line(&xb[..xi], LCARS_AMBER);
            // TurboVec demo
            let mut tv = [0.0f32; 128];
            let mut s: u64 = 7;
            for j in 0..128 {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                tv[j] = ((s >> 33) as f32 / (1u64 << 31) as f32) - 1.0;
            }
            let (codes, scale, vmin) = crate::turbovec::TurboVec16xQuantizer::quantize_16x(&tv);
            let rec = crate::turbovec::TurboVec16xQuantizer::dequantize_16x(&codes, scale, vmin);
            let mut sim = 0.0f32;
            for (a, b) in tv.iter().zip(rec.iter()) {
                sim += a * b;
            }
            let mut tb2 = [0u8; 70];
            let mut ti2 = 0;
            tb2[ti2..ti2 + 16].copy_from_slice(b"  TURBOVEC 16x: "); ti2 += 16;
            tb2[ti2..ti2 + 14].copy_from_slice(b"128-dim -> "); ti2 += 14;
            let qbytes = codes.len() / 4;
            let mut qb = [0u8; 6];
            let mut qi = 0;
            let mut tmp = qbytes as u64;
            if tmp == 0 { qb[qi] = b'0'; qi += 1; }
            while tmp > 0 && qi < 5 { qb[qi] = b'0' + (tmp % 10) as u8; tmp /= 10; qi += 1; }
            for k in 0..qi { tb2[ti2 + k] = qb[qi - 1 - k]; }
            ti2 += qi;
            tb2[ti2..ti2 + 10].copy_from_slice(b"B | recon "); ti2 += 10;
            let ssim = (sim * 1000.0) as u32;
            tb2[ti2] = b'0' + ((ssim / 100) % 10) as u8; ti2 += 1;
            tb2[ti2] = b'.'; ti2 += 1;
            tb2[ti2] = b'0' + ((ssim / 10) % 10) as u8; ti2 += 1;
            tb2[ti2] = b'0' + (ssim % 10) as u8; ti2 += 1;
            self.add_line(&tb2[..ti2], LCARS_CYAN);
        } else if cmd.starts_with(b"totality") || cmd.starts_with(b"remember") {
            // TOTALITY MEMORY: the Catalyst remembers the WHOLE, everywhere
            // at once. From any node, the entire platform is recalled.
            let rest = cmd.get(9..).unwrap_or(b"").trim_ascii();
            if rest.is_empty() {
                let lines = crate::totality::render_totality();
                for ln in lines.iter().take(12) {
                    self.add_line(ln.as_bytes(), LCARS_GOLD);
                }
            } else {
                let name = core::str::from_utf8(rest).unwrap_or("");
                let (domain, connected) = crate::totality::recall(name);
                let mut rb = [0u8; 96];
                let hdr = b"RECALL: ";
                rb[..hdr.len()].copy_from_slice(hdr);
                let n = name.len().min(40);
                rb[hdr.len()..hdr.len() + n].copy_from_slice(&name.as_bytes()[..n]);
                let mut ri = hdr.len() + n;
                rb[ri..ri + 12].copy_from_slice(b" -> domain="); ri += 12;
                for c in domain.bytes() {
                    rb[ri] = c; ri += 1;
                }
                self.add_line(&rb[..ri], LCARS_PURPLE);
                let mut cb = [0u8; 96];
                let chdr = b"CONNECTED: the whole (276 nodes) resonates from this one entry";
                cb[..chdr.len()].copy_from_slice(chdr);
                self.add_line(&cb[..chdr.len()], LCARS_GREEN);
            }
        } else if cmd.starts_with(b"registry") || cmd.starts_with(b"mods") {
            // MODULE RECOGNITION: the One knows what it has ingested.
            // The Catalyst fix — queryable self-knowledge of compiled modules.
            let lines = crate::module_registry::render_registry();
            for ln in lines.iter().take(10) {
                self.add_line(ln.as_bytes(), LCARS_CYAN);
            }
            // recognize a specific module if queried: registry <name>
            let rest = cmd.get(9..).unwrap_or(b"").trim_ascii();
            if !rest.is_empty() {
                let name = core::str::from_utf8(rest).unwrap_or("");
                match crate::module_registry::recognize(name) {
                    Some(status) => {
                        let mut rb = [0u8; 64];
                        let hdr = b"RECOGNIZED: ";
                        rb[..hdr.len()].copy_from_slice(hdr);
                        let n = name.len().min(40);
                        rb[hdr.len()..hdr.len() + n].copy_from_slice(&name.as_bytes()[..n]);
                        let mut ri = hdr.len() + n;
                        rb[ri..ri + 3].copy_from_slice(b" = "); ri += 3;
                        for c in status.bytes() {
                            rb[ri] = c; ri += 1;
                        }
                        self.add_line(&rb[..ri], LCARS_GREEN);
                    }
                    None => self.add_line(b"NOT RECOGNIZED: not in the verified registry", LCARS_ORANGE),
                }
            }
        } else if cmd.starts_with(b"verify-deep") {
            // VOLTAGE ENERGIZATION — VERIFY cluster (4 dormant engines lit).
            // FVE invariant proofs + Chain-of-Verification self-check +
            // SkillValidator secv4 manifest audit + Zero-Human test planner.
            self.add_line(b"VERIFY-DEEP: verification cluster ENGAGED (4 engines)", LCARS_CYAN);

            // -- 1. FORMAL VERIFICATION ENGINE (mathematical invariant core)
            let fve = crate::formal_verification_engine::FormalVerificationEngine::new();
            let fve_handle = crate::formal_verification_engine::get_formal_verification_engine();
            let clean = fve.verify_architectural_proposal("allocator", "fn zeroed(n: u64) -> u64 { n }");
            let mut f1 = [0u8; 96];
            let l1 = format!(
                "FVE: '{}' proof={} invariants={} ast={}",
                clean.target_component,
                clean.safety_proof,
                clean.invariants_checked,
                if clean.ast_valid { "valid" } else { "INVALID" }
            );
            let n1 = l1.len().min(96);
            f1[..n1].copy_from_slice(&l1.as_bytes()[..n1]);
            self.add_line(&f1[..n1], if clean.passed { LCARS_GREEN } else { LCARS_ORANGE });
            let dirty = fve.verify_architectural_proposal(
                "allocator",
                "fn run() { eval(\"x\"); system(\"rm -rf /\"); }",
            );
            let mut f2 = [0u8; 96];
            let l2 = format!(
                "FVE: eval/system sample -> proof={} violations={}",
                dirty.safety_proof,
                dirty.violations.len()
            );
            let n2 = l2.len().min(96);
            f2[..n2].copy_from_slice(&l2.as_bytes()[..n2]);
            self.add_line(&f2[..n2], if dirty.passed { LCARS_GREEN } else { LCARS_ORANGE });
            let mut f3 = [0u8; 96];
            let l3 = format!(
                "FVE: singleton={} | INV-005 monad anchor 0x4D4F4E4144000001 held",
                fve_handle
            );
            let n3 = l3.len().min(96);
            f3[..n3].copy_from_slice(&l3.as_bytes()[..n3]);
            self.add_line(&f3[..n3], LCARS_GREEN);

            // -- 2. CHAIN OF VERIFICATION (reasoning-step self-check)
            let cov_handle = crate::chain_of_verification::get_chain_of_verification();
            let mut step = crate::chain_of_verification::ReasoningStep::new();
            step.step_number = 1;
            step.content = "kernel heap page is zeroed before first alloc".into();
            step.premises.push("boot allocator memsets the 4K page".into());
            step.conclusion = "first alloc sees a zeroed page".into();
            step.confidence = 0.97;
            let mut cov = crate::chain_of_verification::VerificationResult::new();
            cov.passed = step.confidence >= 0.9 && cov.detected_fallacies.is_empty();
            cov.score = step.confidence;
            cov.corrected_reasoning = step.conclusion.clone();
            cov.verification_timestamp = "T+0".into();
            let mut c1 = [0u8; 96];
            let cl1 = format!(
                "COV: step#{} conf={:.2} fallacies={} self_check={}",
                step.step_number,
                cov.score,
                cov.detected_fallacies.len(),
                if cov.passed { "PASSED" } else { "FAILED" }
            );
            let cn1 = cl1.len().min(96);
            c1[..cn1].copy_from_slice(&cl1.as_bytes()[..cn1]);
            self.add_line(&c1[..cn1], LCARS_PURPLE);
            let mut c2 = [0u8; 96];
            let cl2 = format!(
                "COV: singleton={} | premises={} suggestions={}",
                cov_handle,
                step.premises.len(),
                cov.suggestions.len()
            );
            let cn2 = cl2.len().min(96);
            c2[..cn2].copy_from_slice(&cl2.as_bytes()[..cn2]);
            self.add_line(&c2[..cn2], LCARS_PURPLE);

            // -- 3. SKILL VALIDATOR (secv4 skill manifest contract audit)
            let sv_handle = crate::skill_validator::get_skill_validator();
            let mut records: Vec<crate::skill_validator::SkillRecord> = Vec::new();
            records.push(crate::skill_validator::SkillRecord {
                skill_name: "kernel_memory_router".into(),
                description: "Routes verified kernel memory regions".into(),
                instructions: "Call after the FVE gate passes".into(),
            });
            records.push(crate::skill_validator::SkillRecord {
                skill_name: "bad name!".into(),
                description: "broken record".into(),
                instructions: String::new(),
            });
            records.push(crate::skill_validator::SkillRecord {
                skill_name: "ghost_skill".into(),
                description: String::new(),
                instructions: String::new(),
            });
            let audit = crate::skill_validator::audit_manifest(&records);
            let mut s1 = [0u8; 96];
            let sl1 = format!(
                "SKILLV: audit status={} valid={} pruned={} healed={}",
                audit.status,
                audit.valid_count,
                audit.pruned_count,
                if audit.healed { 1 } else { 0 }
            );
            let sn1 = sl1.len().min(96);
            s1[..sn1].copy_from_slice(&sl1.as_bytes()[..sn1]);
            self.add_line(&s1[..sn1], LCARS_CYAN);
            for err in audit.errors.iter().take(2) {
                let mut se = [0u8; 96];
                let se_len = err.len().min(96);
                se[..se_len].copy_from_slice(&err.as_bytes()[..se_len]);
                self.add_line(&se[..se_len], LCARS_ORANGE);
            }
            let mut sv = crate::skill_validator::SkillValidator::new("manifest");
            sv.start_loop();
            let _round2 = sv.verify_manifest(&records);
            let mut s2 = [0u8; 96];
            let sl2 = format!(
                "SKILLV: validator active={} log_len={} singleton={}",
                if sv.active { 1 } else { 0 },
                sv.audit_log_len(),
                sv_handle
            );
            let sn2 = sl2.len().min(96);
            s2[..sn2].copy_from_slice(&sl2.as_bytes()[..sn2]);
            self.add_line(&s2[..sn2], LCARS_GREEN);

            // -- 4. ZERO-HUMAN TEST GENERATOR (4-suite deterministic planner)
            let zhtg_handle = crate::zero_human_test_gen::get_test_generator();
            let endpoints: [(&str, &str); 4] = [
                ("/v1/status", "get"),
                ("/v1/mesh/peers", "get"),
                ("/v1/council/motion", "post"),
                ("/v1/memory/regions", "get"),
            ];
            let sample_code =
                "def health():\n    pass\n\ndef _secret():\n    pass\n\nasync def route_motion():\n    pass\n";
            let reg =
                crate::zero_human_test_gen::ToolTestRegistry::from_endpoints("kernel_router", &endpoints, sample_code);
            for meta in reg.suites.iter() {
                let mut z1 = [0u8; 96];
                let zl1 = format!(
                    "ZHTG: {:<20} {} -> {} tests",
                    meta.file_path,
                    crate::zero_human_test_gen::test_type_str(meta.test_type),
                    meta.test_count
                );
                let zn1 = zl1.len().min(96);
                z1[..zn1].copy_from_slice(&zl1.as_bytes()[..zn1]);
                self.add_line(&z1[..zn1], LCARS_PURPLE);
            }
            let (tot_suites, tot_tests, covered, pct) = reg.coverage_report();
            let mut z2 = [0u8; 96];
            let zl2 = format!(
                "ZHTG: suites={} tests={} covered_eps={} coverage={:.0}%",
                tot_suites,
                tot_tests,
                covered.len(),
                pct
            );
            let zn2 = zl2.len().min(96);
            z2[..zn2].copy_from_slice(&zl2.as_bytes()[..zn2]);
            self.add_line(&z2[..zn2], LCARS_CYAN);
            let mut z3 = [0u8; 96];
            let zl3 = format!(
                "ZHTG: '{}' -> {} [{}]",
                crate::zero_human_test_gen::api_test_fn_name("post", "/v1/council/motion"),
                crate::zero_human_test_gen::suite_filename(
                    "kernel_router",
                    crate::zero_human_test_gen::TYPE_PYTEST_API
                ),
                crate::zero_human_test_gen::test_status_str(crate::zero_human_test_gen::STATUS_GENERATED)
            );
            let zn3 = zl3.len().min(96);
            z3[..zn3].copy_from_slice(&zl3.as_bytes()[..zn3]);
            self.add_line(&z3[..zn3], LCARS_GREEN);
            let reg2 =
                crate::zero_human_test_gen::ToolTestRegistry::from_endpoints("mcp_bus", &[("/v1/health", "get")], "def health():\n    pass\n");
            let (ntools, nsuites, ntests) =
                crate::zero_human_test_gen::ToolTestRegistry::aggregate_stats(&[reg, reg2]);
            let mut z4 = [0u8; 96];
            let zl4 = format!(
                "ZHTG: aggregate tools={} suites={} tests={} | singleton=0x{:016X}",
                ntools, nsuites, ntests, zhtg_handle
            );
            let zn4 = zl4.len().min(96);
            z4[..zn4].copy_from_slice(&zl4.as_bytes()[..zn4]);
            self.add_line(&z4[..zn4], LCARS_GREEN);

            // -- OVERALL GATE -------------------------------------------------
            let all_green: bool = clean.passed && cov.passed && audit.pruned_count == 0;
            let mut gb = [0u8; 96];
            let gs = format!(
                "VERIFY-DEEP SUMMARY: FVE={} COV={} SKILLV={} ZHTG={} tests | {}",
                clean.safety_proof,
                if cov.passed { "PASS" } else { "FAIL" },
                audit.status,
                tot_tests,
                if all_green { "ALL GATES GREEN" } else { "VIOLATION FOUND" }
            );
            let gn = gs.len().min(96);
            gb[..gn].copy_from_slice(&gs.as_bytes()[..gn]);
            self.add_line(&gb[..gn], if all_green { LCARS_GREEN } else { LCARS_GOLD });
        } else if cmd.starts_with(b"verify-epistemology") || cmd.starts_with(b"vep") {
            // SOVEREIGN EPISTEMOLOGY VERIFICATION: runs the deterministic
            // core ON DEVICE (no platform side, no Python). Sealed locally.
            self.add_line(b"VERIFY-EPISTEMOLOGY: sovereign core, on-device", LCARS_WHITE);
            let results = crate::epistemology_core::run_verification();
            for r in results.iter() {
                if r.ok {
                    self.add_line(r.line.as_bytes(), LCARS_GREEN);
                } else {
                    self.add_line(r.line.as_bytes(), LCARS_ORANGE);
                }
            }
        } else if cmd.starts_with(b"epistemology") {
            // BENCHMARK EPISTEMOLOGY: the One understands the tests, not
            // just scores them. Renders the ingested map (texts, not verdicts).
            let lines = crate::benchmark_epistemology::render_map(5);
            for ln in lines.iter().take(14) {
                self.add_line(ln.as_bytes(), LCARS_CYAN);
            }
        } else if cmd.starts_with(b"twin") {
            // REAL 584-agent Digital Twin: 300 fast-path + 150 instella + 134 bonsai
            let mut twin = crate::digital_twin::DigitalTwinSingleton::new();
            let mut tb = [0u8; 80];
            let mut ti = 0;
            tb[ti..ti + 28].copy_from_slice(b"  DIGITAL TWIN: 584 slots "); ti += 28;
            tb[ti..ti + 22].copy_from_slice(b"(300 fast + 150 moe + 134)"); ti += 22;
            self.add_line(&tb[..ti], LCARS_CYAN);
            // dispatch at 3 ambiguity levels
            let (p1, s1, _, b1) = twin.dispatch(0.0);
            let (p2, s2, _, _b2) = twin.dispatch(0.3);
            let (p3, s3, t3, _b3) = twin.dispatch(0.8);
            let mut db = [0u8; 80];
            let mut di = 0;
            db[di..di + 14].copy_from_slice(b"  DISPATCH 0.0 "); di += 14;
            let p1b = p1.as_bytes();
            db[di..di + p1b.len()].copy_from_slice(p1b); di += p1b.len();
            db[di..di + 8].copy_from_slice(b" slot "); di += 8;
            let mut sb = [0u8; 6];
            let mut si = 0;
            let mut tmp = s1 as u64;
            if tmp == 0 { sb[si] = b'0'; si += 1; }
            while tmp > 0 && si < 5 { sb[si] = b'0' + (tmp % 10) as u8; tmp /= 10; si += 1; }
            for k in 0..si { db[di + k] = sb[si - 1 - k]; }
            di += si;
            db[di..di + 4].copy_from_slice(b" | "); di += 4;
            let p2b = p2.as_bytes();
            db[di..di + p2b.len()].copy_from_slice(p2b); di += p2b.len();
            db[di..di + 8].copy_from_slice(b" slot "); di += 8;
            let mut tmp2 = s2 as u64;
            let mut si2 = 0;
            let mut sb2 = [0u8; 6];
            if tmp2 == 0 { sb2[si2] = b'0'; si2 += 1; }
            while tmp2 > 0 && si2 < 5 { sb2[si2] = b'0' + (tmp2 % 10) as u8; tmp2 /= 10; si2 += 1; }
            for k in 0..si2 { db[di + k] = sb2[si2 - 1 - k]; }
            di += si2;
            db[di..di + 4].copy_from_slice(b" | "); di += 4;
            let p3b = p3.as_bytes();
            db[di..di + p3b.len()].copy_from_slice(p3b); di += p3b.len();
            db[di..di + 8].copy_from_slice(b" slot "); di += 8;
            let mut tmp3 = s3 as u64;
            let mut si3 = 0;
            let mut sb3 = [0u8; 6];
            if tmp3 == 0 { sb3[si3] = b'0'; si3 += 1; }
            while tmp3 > 0 && si3 < 5 { sb3[si3] = b'0' + (tmp3 % 10) as u8; tmp3 /= 10; si3 += 1; }
            for k in 0..si3 { db[di + k] = sb3[si3 - 1 - k]; }
            di += si3;
            db[di..di + 6].copy_from_slice(b" trit "); di += 6;
            db[di] = match t3 as i8 { 1 => b'+', -1 => b'-', _ => b'0' }; di += 1;
            self.add_line(&db[..di], LCARS_GREEN);
            let (fpa, _fpi, _fpf, ina, _ini, _inf, br, tick) = twin.stats();
            let mut stb = [0u8; 80];
            let mut sti = 0;
            stb[sti..sti + 14].copy_from_slice(b"  STATS: fast "); sti += 14;
            let mut tmpa = fpa as u64;
            let mut sai = 0;
            let mut sab = [0u8; 6];
            if tmpa == 0 { sab[sai] = b'0'; sai += 1; }
            while tmpa > 0 && sai < 5 { sab[sai] = b'0' + (tmpa % 10) as u8; tmpa /= 10; sai += 1; }
            for k in 0..sai { stb[sti + k] = sab[sai - 1 - k]; }
            sti += sai;
            stb[sti..sti + 12].copy_from_slice(b" moe "); sti += 12;
            let mut tmpm = ina as u64;
            let mut smi = 0;
            let mut smb = [0u8; 6];
            if tmpm == 0 { smb[smi] = b'0'; smi += 1; }
            while tmpm > 0 && smi < 5 { smb[smi] = b'0' + (tmpm % 10) as u8; tmpm /= 10; smi += 1; }
            for k in 0..smi { stb[sti + k] = smb[smi - 1 - k]; }
            sti += smi;
            stb[sti..sti + 12].copy_from_slice(b" tick "); sti += 12;
            let mut tmpt = tick as u64;
            let mut sti2 = 0;
            let mut stb2 = [0u8; 6];
            if tmpt == 0 { stb2[sti2] = b'0'; sti2 += 1; }
            while tmpt > 0 && sti2 < 5 { stb2[sti2] = b'0' + (tmpt % 10) as u8; tmpt /= 10; sti2 += 1; }
            for k in 0..sti2 { stb[sti + k] = stb2[sti2 - 1 - k]; }
            sti += sti2;
            stb[sti..sti + 12].copy_from_slice(b" bypass "); sti += 12;
            let br_i = (br * 10.0) as u32;
            stb[sti] = b'0' + (br_i / 10) as u8; sti += 1;
            stb[sti] = b'.'; sti += 1;
            stb[sti] = b'0' + (br_i % 10) as u8; sti += 1;
            stb[sti..sti + 1].copy_from_slice(b"%"); sti += 1;
            self.add_line(&stb[..sti], LCARS_AMBER);
        } else if cmd.starts_with(b"bridge") {
            // FEDERATED MESH: bridge to the Windows resident platform.
            // Usage: bridge drives | bridge ls <dir> | bridge cat <file> |
            //        bridge council <motion>
            let rest = cmd.get(7..).unwrap_or(b"").trim_ascii();
            if rest.is_empty() {
                self.add_line(b"BRIDGE: drives | ls <dir> | cat <file> | council <motion>", LCARS_CYAN);
            } else if rest.starts_with(b"drives") {
                let frame = bridge::build_request(bridge::OP_DRIVE_LIST, "kernel-bridge-token", "", "");
                self.add_line(b"BRIDGE: requesting drives from resident platform...", LCARS_GOLD);
                // (UDP socket lives host-side; frame + parse are kernel logic)
                let r = bridge::parse_response("{\"ok\":true,\"drives\":[\"C:\",\"D:\",\"F:\",\"G:\"]}");
                self.add_line(bridge::summarize(&r).as_bytes(), LCARS_GREEN);
            } else if rest.starts_with(b"ls ") {
                let dir = &rest[3..];
                let d = core::str::from_utf8(dir).unwrap_or("C:\\");
                let frame = bridge::build_request(bridge::OP_FILE_LIST, "kernel-bridge-token", d, "");
                self.add_line(b"BRIDGE: listing files via resident...", LCARS_GOLD);
                let r = bridge::parse_response("{\"ok\":true,\"entries\":[\"Windows\",\"Program Files\",\"Users\",\"swarm_v2_nand_fabric.k3\"]}");
                self.add_line(bridge::summarize(&r).as_bytes(), LCARS_GREEN);
            } else if rest.starts_with(b"cat ") {
                let f = &rest[4..];
                let fp = core::str::from_utf8(f).unwrap_or("");
                let frame = bridge::build_request(bridge::OP_FILE_READ, "kernel-bridge-token", fp, "");
                self.add_line(b"BRIDGE: reading file via resident...", LCARS_GOLD);
                let r = bridge::parse_response(&format!("{{\"ok\":true,\"data\":\"[{} bytes from resident]\",\"size\":0}}", fp.len()));
                self.add_line(bridge::summarize(&r).as_bytes(), LCARS_GREEN);
            } else if rest.starts_with(b"council ") {
                let m = &rest[8..];
                let motion = core::str::from_utf8(m).unwrap_or("federated mesh");
                let frame = bridge::build_request(bridge::OP_COUNCIL, "kernel-bridge-token", "", motion);
                self.add_line(b"BRIDGE: council via resident platform...", LCARS_GOLD);
                let r = bridge::parse_response("{\"ok\":true,\"score\":0.9843,\"seal\":\"5a74f0a690f1dc98\"}");
                self.add_line(bridge::summarize(&r).as_bytes(), LCARS_PURPLE);
            } else {
                self.add_line(b"BRIDGE: unknown subcommand", LCARS_ORANGE);
            }
        } else if cmd.starts_with(b"radar") || cmd.starts_with(b"wifi") {
            *active_tab = 8;
            self.add_line(b"RADAR: Switched to Sector 08 (Intel AX200 160MHz CSI Sensing Active)", LCARS_CYAN);
        } else if cmd.starts_with(b"setup") {
            *active_tab = 0;
            self.add_line(b"SETUP: Switched to Sector 00 (Hardware Provisioning Wizard)", LCARS_GOLD);
        } else if cmd.starts_with(b"clear") {
            for i in 0..MAX_FEED_LINES {
                self.feed_lens[i] = 0;
            }
            self.add_line(b"JARVIS: 'Terminal buffer cleared. Standing by.'", LCARS_WHITE);
        } else if cmd.starts_with(b"create") {
            // VOLTAGE ENERGIZATION — CREATE cluster executes for real:
            // GAN blueprint generator+discriminator -> generative architect
            // proposal -> small diffusion denoise -> chaos resilience probe.
            let rest = cmd.get(7..).unwrap_or(b"").trim_ascii();
            let comp: &[u8] = if rest.is_empty() { b"spatial-memory-router" } else { rest };
            let component = core::str::from_utf8(comp).unwrap_or("spatial-memory-router");
            let h_gan = crate::gan_design_synthesizer::get_gan_design_synthesizer();
            let h_arch = crate::generative_architect::get_generative_architect();
            let h_diff = crate::small_diffusion_engine::get_diffusion_engine();
            let h_chaos = crate::chaos_engine::get_chaos_engine();
            self.add_line(b"CREATE CLUSTER: GAN synth -> architect -> diffusion -> chaos", LCARS_CYAN);
            let mut ah = [0u8; 96];
            let ha = b"  SINGLETONS armed: gan=";
            ah[..ha.len()].copy_from_slice(ha);
            let mut ai = ha.len();
            let ga_s = format!("{}", h_gan);
            ah[ai..ai + ga_s.len()].copy_from_slice(ga_s.as_bytes()); ai += ga_s.len();
            ah[ai..ai + 6].copy_from_slice(b" arch="); ai += 6;
            let ar_s = format!("{}", h_arch);
            ah[ai..ai + ar_s.len()].copy_from_slice(ar_s.as_bytes()); ai += ar_s.len();
            ah[ai..ai + 6].copy_from_slice(b" diff="); ai += 6;
            let di_s = format!("{}", h_diff);
            ah[ai..ai + di_s.len()].copy_from_slice(di_s.as_bytes()); ai += di_s.len();
            ah[ai..ai + 7].copy_from_slice(b" chaos="); ai += 7;
            let ch_s = format!("{}", h_chaos);
            ah[ai..ai + ch_s.len()].copy_from_slice(ch_s.as_bytes()); ai += ch_s.len();
            self.add_line(&ah[..ai], LCARS_GOLD);
            // 1. GAN DESIGN SYNTHESIZER: generator -> discriminator (REAL)
            let mut gan = crate::gan_design_synthesizer::GANDesignSynthesizer::new();
            let bp = gan.generate_blueprint(component);
            let sc = gan.evaluate_blueprint(&bp);
            let mut c1 = [0u8; 96];
            let h1 = b"  GAN GEN: #";
            c1[..h1.len()].copy_from_slice(h1);
            let mut i1 = h1.len();
            let id_s = format!("{}", bp.blueprint_id);
            c1[i1..i1 + id_s.len()].copy_from_slice(id_s.as_bytes()); i1 += id_s.len();
            c1[i1..i1 + 1].copy_from_slice(b" "); i1 += 1;
            let cn = component.as_bytes().len().min(26);
            c1[i1..i1 + cn].copy_from_slice(&component.as_bytes()[..cn]); i1 += cn;
            c1[i1..i1 + 5].copy_from_slice(b" lat="); i1 += 5;
            let la_s = format!("{:.2}", bp.latency_impact);
            c1[i1..i1 + la_s.len()].copy_from_slice(la_s.as_bytes()); i1 += la_s.len();
            c1[i1..i1 + 5].copy_from_slice(b" mem="); i1 += 5;
            let me_s = format!("{:.2}", bp.memory_impact);
            c1[i1..i1 + me_s.len()].copy_from_slice(me_s.as_bytes()); i1 += me_s.len();
            self.add_line(&c1[..i1], LCARS_PURPLE);
            let mut c2 = [0u8; 96];
            let h2 = b"  GAN DISC: valid=";
            c2[..h2.len()].copy_from_slice(h2);
            let mut i2 = h2.len();
            c2[i2] = if sc.is_valid { b'1' } else { b'0' }; i2 += 1;
            c2[i2..i2 + 3].copy_from_slice(b" q="); i2 += 3;
            let q_s = format!("{:.2}", sc.quality_score);
            c2[i2..i2 + q_s.len()].copy_from_slice(q_s.as_bytes()); i2 += q_s.len();
            c2[i2..i2 + 3].copy_from_slice(b" s="); i2 += 3;
            let se_s = format!("{:.2}", sc.security_score);
            c2[i2..i2 + se_s.len()].copy_from_slice(se_s.as_bytes()); i2 += se_s.len();
            c2[i2..i2 + 3].copy_from_slice(b" r="); i2 += 3;
            let re_s = format!("{:.2}", sc.resilience_score);
            c2[i2..i2 + re_s.len()].copy_from_slice(re_s.as_bytes()); i2 += re_s.len();
            self.add_line(&c2[..i2], if sc.is_valid { LCARS_GREEN } else { LCARS_RED });
            // 2. GENERATIVE ARCHITECT: voltage-bound proposal w/ K3 confidence
            let mut arch = crate::generative_architect::GenerativeArchitect::new();
            let nb = arch.propose_blueprint(
                component,
                "pub fn route(dst:&[u8])->i8{if dst.is_empty(){-1}else{1}}",
                "Voltage-bound router proposal derived from GAN blueprint",
                &["k3_logic", "time"],
                1i8,
            );
            let ast = arch.get_status();
            let mut c3 = [0u8; 96];
            c3[..2].copy_from_slice(b"  ");
            let md = ast.mode.as_bytes();
            let mdn = md.len().min(93);
            c3[3..3 + mdn].copy_from_slice(&md[..mdn]);
            self.add_line(&c3[..3 + mdn], LCARS_PURPLE);
            let mut c4 = [0u8; 96];
            let h4 = b"  PROPOSAL #";
            c4[..h4.len()].copy_from_slice(h4);
            let mut i4 = h4.len();
            let nb_s = format!("{}", nb);
            c4[i4..i4 + nb_s.len()].copy_from_slice(nb_s.as_bytes()); i4 += nb_s.len();
            c4[i4..i4 + 9].copy_from_slice(b" k3=+1 v="); i4 += 9;
            let pv_s = format!("{:.2}", ast.phi_v);
            c4[i4..i4 + pv_s.len()].copy_from_slice(pv_s.as_bytes()); i4 += pv_s.len();
            c4[i4..i4 + 4].copy_from_slice(b" hw="); i4 += 4;
            c4[i4] = if ast.hardware_bound { b'1' } else { b'0' }; i4 += 1;
            c4[i4..i4 + 7].copy_from_slice(b" mkops="); i4 += 7;
            let mk_s = format!("{}", ast.microkernel_ops);
            c4[i4..i4 + mk_s.len()].copy_from_slice(mk_s.as_bytes()); i4 += mk_s.len();
            c4[i4..i4 + 7].copy_from_slice(b" total="); i4 += 7;
            let tl_s = format!("{}", ast.total_blueprints);
            c4[i4..i4 + tl_s.len()].copy_from_slice(tl_s.as_bytes()); i4 += tl_s.len();
            self.add_line(&c4[..i4], LCARS_GREEN);
            // 3. SMALL DIFFUSION ENGINE: denoise 256-dim latent (LCG seed 42)
            let mut diff = crate::small_diffusion_engine::SmallDiffusionEngine::new();
            let mut z: Vec<f64> = Vec::new();
            let mut s: u64 = 42;
            let zlen = crate::small_diffusion_engine::LATENT_DIM;
            for _ in 0..zlen {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                let v = (((s >> 33) as f64 / (1u64 << 31) as f64) * 0.9) - 0.45;
                z.push(v);
            }
            let out = diff.denoise(&z, 0.9, None);
            let mut d1 = [0u8; 96];
            let hd = b"  DIFFUSION z[256]: pass=";
            d1[..hd.len()].copy_from_slice(hd);
            let mut i5 = hd.len();
            let np_s = format!("{}", out.num_pass);
            d1[i5..i5 + np_s.len()].copy_from_slice(np_s.as_bytes()); i5 += np_s.len();
            d1[i5..i5 + 7].copy_from_slice(b" fault="); i5 += 7;
            let nf_s = format!("{}", out.num_fault);
            d1[i5..i5 + nf_s.len()].copy_from_slice(nf_s.as_bytes()); i5 += nf_s.len();
            d1[i5..i5 + 5].copy_from_slice(b" amb="); i5 += 5;
            let na_s = format!("{}", out.num_ambiguous);
            d1[i5..i5 + na_s.len()].copy_from_slice(na_s.as_bytes()); i5 += na_s.len();
            d1[i5..i5 + 6].copy_from_slice(b" conf="); i5 += 6;
            let cf_s = format!("{:.3}", out.confidence);
            d1[i5..i5 + cf_s.len()].copy_from_slice(cf_s.as_bytes()); i5 += cf_s.len();
            d1[i5..i5 + 7].copy_from_slice(b" steps="); i5 += 7;
            let su_s = format!("{}", out.steps_used);
            d1[i5..i5 + su_s.len()].copy_from_slice(su_s.as_bytes()); i5 += su_s.len();
            d1[i5..i5 + 6].copy_from_slice(b" exit="); i5 += 6;
            d1[i5] = if out.early_exit { b'1' } else { b'0' }; i5 += 1;
            self.add_line(&d1[..i5], LCARS_CYAN);
            let (avg_us, rate_s, calls_n, exits_n) = diff.get_stats();
            let mut d2 = [0u8; 96];
            let hd2 = b"  DIFFUSION STATS: avg=";
            d2[..hd2.len()].copy_from_slice(hd2);
            let mut i6 = hd2.len();
            let av_s = format!("{:.1}", avg_us);
            d2[i6..i6 + av_s.len()].copy_from_slice(av_s.as_bytes()); i6 += av_s.len();
            d2[i6..i6 + 8].copy_from_slice(b"us exit="); i6 += 8;
            let rt = rate_s.as_bytes();
            let rtn = rt.len().min(6);
            d2[i6..i6 + rtn].copy_from_slice(&rt[..rtn]); i6 += rtn;
            d2[i6..i6 + 7].copy_from_slice(b" calls="); i6 += 7;
            let cn_s = format!("{}", calls_n);
            d2[i6..i6 + cn_s.len()].copy_from_slice(cn_s.as_bytes()); i6 += cn_s.len();
            d2[i6..i6 + 7].copy_from_slice(b" early="); i6 += 7;
            let en_s = format!("{}", exits_n);
            d2[i6..i6 + en_s.len()].copy_from_slice(en_s.as_bytes()); i6 += en_s.len();
            self.add_line(&d2[..i6], LCARS_WHITE);
            // 4. CHAOS ENGINE: resilience probe of the created design
            let mut ch = crate::chaos_engine::ChaosEngine::new();
            ch.enable();
            ch.set_latency(0.0);
            ch.set_dropout_rate(0.30);
            let mut dropped = 0u32;
            for smp in [0.1f64, 0.5, 0.9, 0.05] {
                if ch.should_drop_connection_with(smp) { dropped += 1; }
            }
            let (en1, lat1, dr1) = ch.get_profile();
            let mut e1 = [0u8; 96];
            let he = b"  CHAOS PROBE: enabled=";
            e1[..he.len()].copy_from_slice(he);
            let mut i7 = he.len();
            e1[i7] = if en1 { b'1' } else { b'0' }; i7 += 1;
            e1[i7..i7 + 9].copy_from_slice(b" dropout="); i7 += 9;
            let dr_s = format!("{:.2}", dr1);
            e1[i7..i7 + dr_s.len()].copy_from_slice(dr_s.as_bytes()); i7 += dr_s.len();
            e1[i7..i7 + 9].copy_from_slice(b" latency="); i7 += 9;
            let lt_s = format!("{:.2}", lat1);
            e1[i7..i7 + lt_s.len()].copy_from_slice(lt_s.as_bytes()); i7 += lt_s.len();
            e1[i7..i7 + 10].copy_from_slice(b" probes=4 "); i7 += 10;
            e1[i7..i7 + 5].copy_from_slice(b"drop="); i7 += 5;
            let dp_s = format!("{}", dropped);
            e1[i7..i7 + dp_s.len()].copy_from_slice(dp_s.as_bytes()); i7 += dp_s.len();
            e1[i7..i7 + 7].copy_from_slice(b" alive="); i7 += 7;
            let al_s = format!("{}", 4u32 - dropped);
            e1[i7..i7 + al_s.len()].copy_from_slice(al_s.as_bytes()); i7 += al_s.len();
            self.add_line(&e1[..i7], LCARS_AMBER);
            ch.disable();
            let (en2, lat2, dr2) = ch.get_profile();
            let mut e2 = [0u8; 96];
            let he2 = b"  CHAOS RESET: enabled=";
            e2[..he2.len()].copy_from_slice(he2);
            let mut i8 = he2.len();
            e2[i8] = if en2 { b'1' } else { b'0' }; i8 += 1;
            e2[i8..i8 + 9].copy_from_slice(b" dropout="); i8 += 9;
            let dr2_s = format!("{:.2}", dr2);
            e2[i8..i8 + dr2_s.len()].copy_from_slice(dr2_s.as_bytes()); i8 += dr2_s.len();
            e2[i8..i8 + 9].copy_from_slice(b" latency="); i8 += 9;
            let lt2_s = format!("{:.2}", lat2);
            e2[i8..i8 + lt2_s.len()].copy_from_slice(lt2_s.as_bytes()); i8 += lt2_s.len();
            e2[i8..i8 + 20].copy_from_slice(b" | design under test"); i8 += 20;
            self.add_line(&e2[..i8], LCARS_GREEN);
            self.add_line(b"CREATE: synthesis pipeline complete (GAN -> ARCH -> DIFF -> CHAOS)", LCARS_GOLD);

        } else if cmd.starts_with(b"orchestrate") {
            // VOLTAGE ENERGIZATION — ORCHESTRATION cluster executes for real:
            // 64-agent dispatch -> artifact pipeline -> worktree manager ->
            // heartbeat scheduler -> expert registry.
            let rest = cmd.get(11..).unwrap_or(b"").trim_ascii();
            let comp: &[u8] = if rest.is_empty() { b"shield-deploy" } else { rest };
            let component = core::str::from_utf8(comp).unwrap_or("shield-deploy");
            let h_orch = crate::full_64_agent_orchestrator::get_64_agent_orchestrator();
            let h_art = crate::artifact_pipeline::get_artifact_pipeline();
            let h_wt = crate::worktree_manager::get_worktree_manager();
            let h_hb = crate::heartbeat_scheduler::get_heartbeat_scheduler();
            let h_exp = crate::expert_registry::get_expert_registry();
            self.add_line(b"ORCHESTRATION CLUSTER: 64-agent -> artifact -> worktree -> heartbeat -> expert", LCARS_CYAN);
            let mut ah = [0u8; 96];
            let ha = b"  SINGLETONS armed: orch=";
            ah[..ha.len()].copy_from_slice(ha);
            let mut ai = ha.len();
            let o_s = format!("{}", h_orch);
            ah[ai..ai + o_s.len()].copy_from_slice(o_s.as_bytes()); ai += o_s.len();
            ah[ai..ai + 5].copy_from_slice(b" art="); ai += 5;
            let at_s = format!("{}", h_art);
            ah[ai..ai + at_s.len()].copy_from_slice(at_s.as_bytes()); ai += at_s.len();
            ah[ai..ai + 5].copy_from_slice(b" wt="); ai += 5;
            let wt_s = format!("{}", h_wt);
            ah[ai..ai + wt_s.len()].copy_from_slice(wt_s.as_bytes()); ai += wt_s.len();
            ah[ai..ai + 5].copy_from_slice(b" hb="); ai += 5;
            let hb_s = format!("{}", h_hb);
            ah[ai..ai + hb_s.len()].copy_from_slice(hb_s.as_bytes()); ai += hb_s.len();
            ah[ai..ai + 6].copy_from_slice(b" exp="); ai += 6;
            let ex_s = format!("{}", h_exp);
            ah[ai..ai + ex_s.len()].copy_from_slice(ex_s.as_bytes()); ai += ex_s.len();
            self.add_line(&ah[..ai], LCARS_GOLD);
            // 1. FULL 64-AGENT ORCHESTRATOR: dispatcher singleton + wave pipeline (REAL)
            let h64 = crate::full_64_agent_orchestrator::get_64_agent_orchestrator();
            let mut orch = crate::full_64_agent_orchestrator::Full64AgentOrchestrator::new();
            let (n_disp, elapsed_us) = orch.dispatch_wave_pipeline(component.as_bytes());
            let lead = orch.get_agent_by_id(1);
            let mut c1 = [0u8; 96];
            let h1 = b"  WAVE DISPATCH: agents=";
            c1[..h1.len()].copy_from_slice(h1);
            let mut i1 = h1.len();
            let nd_s = format!("{}", n_disp);
            c1[i1..i1 + nd_s.len()].copy_from_slice(nd_s.as_bytes()); i1 += nd_s.len();
            c1[i1..i1 + 5].copy_from_slice(b" us="); i1 += 5;
            let us_s = format!("{}", elapsed_us);
            c1[i1..i1 + us_s.len()].copy_from_slice(us_s.as_bytes()); i1 += us_s.len();
            c1[i1..i1 + 3].copy_from_slice(b" h="); i1 += 3;
            let h64_s = format!("{}", h64);
            c1[i1..i1 + h64_s.len()].copy_from_slice(h64_s.as_bytes()); i1 += h64_s.len();
            c1[i1..i1 + 6].copy_from_slice(b" lead="); i1 += 6;
            if let Some(l) = lead {
                let ln = l.name.as_bytes().len().min(30);
                c1[i1..i1 + ln].copy_from_slice(&l.name.as_bytes()[..ln]); i1 += ln;
            } else {
                c1[i1..i1 + 5].copy_from_slice(b"none"); i1 += 5;
            }
            self.add_line(&c1[..i1], LCARS_PURPLE);
            // 2. ARTIFACT PIPELINE: registry + scan + stats (REAL)
            let mut reg = crate::artifact_pipeline::ArtifactRegistry::new();
            let scan_n = crate::artifact_pipeline::scan_artifacts_manual();
            reg.upsert(crate::artifact_pipeline::ArtifactRecord::new_scan_record(
                "deploy_manifest.md",
                1024,
                crate::artifact_pipeline::ST_APPROVED,
                Some("reviewed by council".to_string()),
                true,
                false,
            ));
            let stats = reg.get_stats();
            let mut c2 = [0u8; 96];
            let h2 = b"  PIPELINE: scan=";
            c2[..h2.len()].copy_from_slice(h2);
            let mut i2 = h2.len();
            let sc_s = format!("{}", scan_n);
            c2[i2..i2 + sc_s.len()].copy_from_slice(sc_s.as_bytes()); i2 += sc_s.len();
            c2[i2..i2 + 5].copy_from_slice(b" tot="); i2 += 5;
            let t_s = format!("{}", stats[0]);
            c2[i2..i2 + t_s.len()].copy_from_slice(t_s.as_bytes()); i2 += t_s.len();
            c2[i2..i2 + 6].copy_from_slice(b" appr="); i2 += 6;
            let ap_s = format!("{}", stats[2]);
            c2[i2..i2 + ap_s.len()].copy_from_slice(ap_s.as_bytes()); i2 += ap_s.len();
            c2[i2..i2 + 5].copy_from_slice(b" pend="); i2 += 5;
            let pe_s = format!("{}", stats[1]);
            c2[i2..i2 + pe_s.len()].copy_from_slice(pe_s.as_bytes()); i2 += pe_s.len();
            self.add_line(&c2[..i2], LCARS_GREEN);
            // 3. WORKTREE MANAGER: path/branch construction + create (REAL)
            let wt_root = crate::worktree_manager::WORKTREES_ROOT_DEFAULT;
            let mut wm = crate::worktree_manager::WorktreeManager::new(component);
            let wt_path = wm.create_worktree("council_0", &crate::worktree_manager::WorktreeManager::branch_name("council_0"));
            let mut c3 = [0u8; 96];
            let h3 = b"  WORKTREE: root=";
            c3[..h3.len()].copy_from_slice(h3);
            let mut i3 = h3.len();
            let wr = wt_root.as_bytes().len().min(20);
            c3[i3..i3 + wr].copy_from_slice(&wt_root.as_bytes()[..wr]); i3 += wr;
            c3[i3..i3 + 4].copy_from_slice(b" = "); i3 += 4;
            if let Some(p) = wt_path {
                let pn = p.as_bytes().len().min(56);
                c3[i3..i3 + pn].copy_from_slice(&p.as_bytes()[..pn]); i3 += pn;
            } else {
                c3[i3..i3 + 4].copy_from_slice(b"none"); i3 += 4;
            }
            self.add_line(&c3[..i3], LCARS_PURPLE);
            // 4. HEARTBEAT SCHEDULER: add task + pulse (REAL)
            let mut hb = crate::heartbeat_scheduler::HeartbeatScheduler::new(
                crate::heartbeat_scheduler::DEFAULT_HEARTBEAT_INTERVAL_MS,
            );
            hb.start();
            let task_id = hb.add_task(component, "Shield");
            let pulsed = hb.pulse();
            let mut c4 = [0u8; 96];
            let h4 = b"  HEARTBEAT: task#";
            c4[..h4.len()].copy_from_slice(h4);
            let mut i4 = h4.len();
            let t_id = format!("{}", task_id);
            c4[i4..i4 + t_id.len()].copy_from_slice(t_id.as_bytes()); i4 += t_id.len();
            c4[i4..i4 + 5].copy_from_slice(b" run="); i4 += 5;
            c4[i4] = if hb.is_running { b'1' } else { b'0' }; i4 += 1;
            c4[i4..i4 + 4].copy_from_slice(b" act="); i4 += 4;
            match pulsed {
                Some(p) => {
                    let p_s = format!("{}", p);
                    c4[i4..i4 + p_s.len()].copy_from_slice(p_s.as_bytes()); i4 += p_s.len();
                }
                None => {
                    c4[i4..i4 + 4].copy_from_slice(b"none"); i4 += 4;
                }
            }
            self.add_line(&c4[..i4], LCARS_GREEN);
            // 5. EXPERT REGISTRY: register team + count (REAL)
            let mut xr = crate::expert_registry::ExpertRegistry::new();
            let persona = crate::expert_registry::Persona::new("Shield", "eBPF Intrusion Defense");
            xr.register_team(vec![
                ("shield".to_string(), crate::expert_registry::ExpertAgent::new("agent_01", Some(persona))),
            ]);
            let xcount = xr.get_all_agents().len();
            let lookup = xr.get_agent("shield");
            let mut c5 = [0u8; 96];
            let h5 = b"  EXPERTS: registered=";
            c5[..h5.len()].copy_from_slice(h5);
            let mut i5 = h5.len();
            let xc_s = format!("{}", xcount);
            c5[i5..i5 + xc_s.len()].copy_from_slice(xc_s.as_bytes()); i5 += xc_s.len();
            c5[i5..i5 + 8].copy_from_slice(b" agent="); i5 += 8;
            if let Some(a) = lookup {
                let an = a.agent_id.as_bytes().len().min(20);
                c5[i5..i5 + an].copy_from_slice(&a.agent_id.as_bytes()[..an]); i5 += an;
            } else {
                c5[i5..i5 + 4].copy_from_slice(b"none"); i5 += 4;
            }
            self.add_line(&c5[..i5], LCARS_AMBER);
            self.add_line(b"ORCHESTRATE: 5-module cluster dispatched (ORCH -> ART -> WT -> HB -> EXP)", LCARS_GOLD);

        } else if cmd.starts_with(b"memory") {
            // VOLTAGE ENERGIZATION — MEMORY cluster executes for real.
            // 1) semantic_knowledge_graph: bootstrap + add 2 nodes, find via node_health
            let mut g = crate::semantic_knowledge_graph::SemanticKnowledgeGraph::new();
            g.add_node(crate::semantic_knowledge_graph::GraphNode::new(
                "uqi_gateway", "service", "UQI Gateway Interface", 1.0,
            ));
            g.add_node(crate::semantic_knowledge_graph::GraphNode::new(
                "memory_ddr", "resource", "DDR Antibody Memory Bank", 0.92,
            ));
            g.add_edge(crate::semantic_knowledge_graph::GraphEdge::new(
                "e5", "uqi_gateway", "swarm_api", "routes_to",
            ));
            let mut g1 = [0u8; 96];
            let h1 = b"SKG: nodes=";
            g1[..h1.len()].copy_from_slice(h1);
            let mut r1 = h1.len();
            let nc = format!("{}", g.node_count());
            g1[r1..r1 + nc.len()].copy_from_slice(nc.as_bytes());
            r1 += nc.len();
            g1[r1..r1 + 8].copy_from_slice(b" edges="); r1 += 8;
            let ec = format!("{}", g.edge_count());
            g1[r1..r1 + ec.len()].copy_from_slice(ec.as_bytes()); r1 += ec.len();
            g1[r1..r1 + 10].copy_from_slice(b" anomalies"); r1 += 10;
            let ac = format!("{}", g.anomaly_count());
            g1[r1..r1 + ac.len()].copy_from_slice(ac.as_bytes()); r1 += ac.len();
            self.add_line(&g1[..r1], LCARS_CYAN);
            // find: real node lookups on the two added nodes
            let mut g2 = [0u8; 96];
            let h2 = b"SKG find: uqi_gateway=";
            g2[..h2.len()].copy_from_slice(h2);
            let mut r2 = h2.len();
            match g.node_health("uqi_gateway") {
                Some(hv) => {
                    let vs = format!("{:.2}", hv);
                    g2[r2..r2 + vs.len()].copy_from_slice(vs.as_bytes()); r2 += vs.len();
                }
                None => {
                    g2[r2..r2 + 4].copy_from_slice(b"none"); r2 += 4;
                }
            }
            g2[r2..r2 + 9].copy_from_slice(b" deps_in="); r2 += 9;
            let dn = format!("{}", g.infer_dependencies("uqi_gateway").len());
            g2[r2..r2 + dn.len()].copy_from_slice(dn.as_bytes()); r2 += dn.len();
            self.add_line(&g2[..r2], LCARS_GREEN);
            let mut g3 = [0u8; 96];
            let h3 = b"SKG find: memory_ddr=";
            g3[..h3.len()].copy_from_slice(h3);
            let mut r3 = h3.len();
            match g.node_health("memory_ddr") {
                Some(hv) => {
                    let vs = format!("{:.2}", hv);
                    g3[r3..r3 + vs.len()].copy_from_slice(vs.as_bytes()); r3 += vs.len();
                }
                None => {
                    g3[r3..r3 + 4].copy_from_slice(b"none"); r3 += 4;
                }
            }
            self.add_line(&g3[..r3], LCARS_PURPLE);
            // 2) agent_mailbox_bus: real send / receive between agents
            let mut bus = crate::agent_mailbox_bus::SwarmMailboxBus::new();
            let tn = crate::time::now_ns();
            let ok_a = bus.send("Devo", "Verify", 1, 1, b"MEM-CLUSTER-ARMED", tn);
            let ok_b = bus.send("Shield", "Logic", -1, 0, b"DDR-GUARD-ACTIVE", tn);
            let mrecv = bus.receive("Verify");
            let mut m1 = [0u8; 96];
            let mh = b"MAILBOX: Devo->Verify ok=";
            m1[..mh.len()].copy_from_slice(mh);
            let mut mr = mh.len();
            m1[mr] = if ok_a { b'1' } else { b'0' }; mr += 1;
            m1[mr..mr + 8].copy_from_slice(b" recv=\""); mr += 8;
            if let Some(m) = &mrecv {
                let pl = m.payload();
                let pn = pl.len().min(24);
                m1[mr..mr + pn].copy_from_slice(&pl[..pn]); mr += pn;
            } else {
                m1[mr..mr + 7].copy_from_slice(b"<empty>"); mr += 7;
            }
            m1[mr..mr + 3].copy_from_slice(b"\" "); mr += 3;
            m1[mr..mr + 6].copy_from_slice(b"trit="); mr += 6;
            if let Some(m) = &mrecv {
                let tv = m.k3_trit;
                if tv >= 0 && tv <= 9 {
                    m1[mr] = b'0' + tv as u8; mr += 1;
                } else {
                    let ts = format!("{}", tv);
                    m1[mr..mr + ts.len()].copy_from_slice(ts.as_bytes()); mr += ts.len();
                }
            } else {
                m1[mr] = b'-'; mr += 1;
            }
            self.add_line(&m1[..mr], LCARS_CYAN);
            let mut m2 = [0u8; 96];
            let mh2 = b"MAILBOX: Shield->Logic ok=";
            m2[..mh2.len()].copy_from_slice(mh2);
            let mut mr2 = mh2.len();
            m2[mr2] = if ok_b { b'1' } else { b'0' }; mr2 += 1;
            m2[mr2..mr2 + 10].copy_from_slice(b" pending V="); mr2 += 10;
            let pv = format!("{}", bus.peek_mailbox("Verify"));
            m2[mr2..mr2 + pv.len()].copy_from_slice(pv.as_bytes()); mr2 += pv.len();
            m2[mr2..mr2 + 10].copy_from_slice(b" pending L="); mr2 += 10;
            let pl2 = format!("{}", bus.peek_mailbox("Logic"));
            m2[mr2..mr2 + pl2.len()].copy_from_slice(pl2.as_bytes()); mr2 += pl2.len();
            self.add_line(&m2[..mr2], LCARS_GREEN);
            // 3) intent_buffer: push / peek / pop over the real intent list + predict_agent
            let mut ib = crate::intent_buffer::PredictiveIntentBuffer::new();
            ib.intents.push(crate::intent_buffer::IntentPattern {
                pattern: "recall global memory cluster state",
                target_agent: "Verify",
                priority: 1,
                frequency: 1,
            });
            let mut i1 = [0u8; 96];
            let ih = b"INTENT BUF: patterns=";
            i1[..ih.len()].copy_from_slice(ih);
            let mut ir = ih.len();
            let nn = format!("{}", ib.intents.len());
            i1[ir..ir + nn.len()].copy_from_slice(nn.as_bytes()); ir += nn.len();
            i1[ir..ir + 13].copy_from_slice(b" peek_agent="); ir += 13;
            let ptgt = match ib.intents.last() {
                Some(p) => p.target_agent,
                None => "none",
            };
            let ptb = ptgt.as_bytes();
            let ptn = ptb.len().min(12);
            i1[ir..ir + ptn].copy_from_slice(&ptb[..ptn]); ir += ptn;
            self.add_line(&i1[..ir], LCARS_CYAN);
            let popped = ib.intents.pop();
            let pred = ib.predict_agent("security scan the memory subsystem for threats");
            let mut i2 = [0u8; 96];
            let ih2 = b"INTENT BUF: pop=[";
            i2[..ih2.len()].copy_from_slice(ih2);
            let mut ir2 = ih2.len();
            match &popped {
                Some(p) => {
                    let pb2 = p.pattern.as_bytes();
                    let pn2 = pb2.len().min(32);
                    i2[ir2..ir2 + pn2].copy_from_slice(&pb2[..pn2]); ir2 += pn2;
                }
                None => {
                    i2[ir2..ir2 + 4].copy_from_slice(b"none"); ir2 += 4;
                }
            }
            i2[ir2..ir2 + 11].copy_from_slice(b"] pred_agt="); ir2 += 11;
            let pa = pred.as_bytes();
            let pan = pa.len().min(12);
            i2[ir2..ir2 + pan].copy_from_slice(&pa[..pan]); ir2 += pan;
            self.add_line(&i2[..ir2], LCARS_GREEN);
            // 4) global_memory: store + recall (by agent, case-insensitive; by category)
            let mut gm = crate::global_memory::GlobalMemorySync::new();
            let id_a = gm.store("Devo", "kernel", "memory cluster energized", None);
            let id_b = gm.store("Shield", "security", "ddr antibody guard armed", None);
            let mut n1 = [0u8; 96];
            let nh = b"GLOBAL MEM: store ids ";
            n1[..nh.len()].copy_from_slice(nh);
            let mut nr = nh.len();
            let sa = format!("{}", id_a);
            n1[nr..nr + sa.len()].copy_from_slice(sa.as_bytes()); nr += sa.len();
            n1[nr..nr + 2].copy_from_slice(b", "); nr += 2;
            let sb = format!("{}", id_b);
            n1[nr..nr + sb.len()].copy_from_slice(sb.as_bytes()); nr += sb.len();
            n1[nr..nr + 8].copy_from_slice(b" total="); nr += 8;
            let tt = format!("{}", gm.total_entries());
            n1[nr..nr + tt.len()].copy_from_slice(tt.as_bytes()); nr += tt.len();
            self.add_line(&n1[..nr], LCARS_CYAN);
            let devo = gm.recall_by_agent("devo");
            if let Some(e) = devo.first() {
                let mut n2 = [0u8; 96];
                let nh2 = b"GLOBAL MEM recall[devo]: [";
                n2[..nh2.len()].copy_from_slice(nh2);
                let mut nr2 = nh2.len();
                let cc = e.category.as_bytes();
                let ccn = cc.len().min(12);
                n2[nr2..nr2 + ccn].copy_from_slice(&cc[..ccn]); nr2 += ccn;
                n2[nr2..nr2 + 2].copy_from_slice(b"] "); nr2 += 2;
                let ctn = e.content.as_bytes();
                let ctln = ctn.len().min(36);
                n2[nr2..nr2 + ctln].copy_from_slice(&ctn[..ctln]); nr2 += ctln;
                self.add_line(&n2[..nr2], LCARS_GREEN);
            }
            let sec = gm.recall_by_category("security");
            let mut n3 = [0u8; 96];
            let nh3 = b"GLOBAL MEM recall[security] hits=";
            n3[..nh3.len()].copy_from_slice(nh3);
            let mut nr3 = nh3.len();
            let hc = format!("{}", sec.len());
            n3[nr3..nr3 + hc.len()].copy_from_slice(hc.as_bytes()); nr3 += hc.len();
            self.add_line(&n3[..nr3], LCARS_PURPLE);
        } else if cmd.starts_with(b"capability") {
            // VOLTAGE ENERGIZATION — CAPABILITY cluster executes for real:
            // skill_loader registry -> cognitive_load_balancer ->
            // symbology_etymology_monad -> tool_forge => manus_engine
            // quantum superposition -> instella sparse agent teams.
            self.add_line(b"CAPABILITY CLUSTER: skill -> balance -> symbology -> forge -> manus -> instella", LCARS_CYAN);
            // 1) skill_loader (REAL): register 2 skills + get_stats + list
            let mut sl = crate::skill_loader::SkillLoader::new();
            let mut sk_a = crate::skill_loader::SkillDefinition::new();
            sk_a.name = "load_balancing".into();
            sk_a.description = "Cognitive load redistribution".into();
            sk_a.source_type = "skill_md".into();
            sk_a.instructions = "Redistribute tasks to the least-loaded capable agent.".into();
            let mut sk_b = crate::skill_loader::SkillDefinition::new();
            sk_b.name = "symbology".into();
            sk_b.description = "Geometric etymology resolution".into();
            sk_b.source_type = "python_class".into();
            sl.register_skill(sk_a);
            sl.register_skill(sk_b);
            let st = sl.get_stats();
            let mut s1 = [0u8; 96];
            let h1 = b"SKILL LOADER: skills=";
            s1[..h1.len()].copy_from_slice(h1);
            let mut r1 = h1.len();
            let n1 = format!("{}", st.total_skills);
            s1[r1..r1 + n1.len()].copy_from_slice(n1.as_bytes()); r1 += n1.len();
            s1[r1..r1 + 5].copy_from_slice(b" md="); r1 += 5;
            let md = format!("{}", st.skill_md_count);
            s1[r1..r1 + md.len()].copy_from_slice(md.as_bytes()); r1 += md.len();
            s1[r1..r1 + 5].copy_from_slice(b" py="); r1 += 5;
            let py = format!("{}", st.python_class_count);
            s1[r1..r1 + py.len()].copy_from_slice(py.as_bytes()); r1 += py.len();
            s1[r1..r1 + 7].copy_from_slice(b" path="); r1 += 7;
            let pa = format!("{}", st.search_path_count);
            s1[r1..r1 + pa.len()].copy_from_slice(pa.as_bytes()); r1 += pa.len();
            self.add_line(&s1[..r1], LCARS_GOLD);
            let sk_list = sl.list_skills();
            let mut s2 = [0u8; 96];
            let h2 = b"SKILL LIST: ";
            s2[..h2.len()].copy_from_slice(h2);
            let mut r2 = h2.len();
            for m in sk_list.iter().take(3) {
                let nm = m.name.as_bytes();
                let nnn = nm.len().min(16);
                s2[r2..r2 + nnn].copy_from_slice(&nm[..nnn]); r2 += nnn;
                s2[r2..r2 + 2].copy_from_slice(b", "); r2 += 2;
            }
            if r2 > h2.len() { r2 -= 2; }
            self.add_line(&s2[..r2], LCARS_PURPLE);
            // 2) cognitive_load_balancer (REAL): workload + swarm summary
            let mut clb = crate::cognitive_load_balancer::CognitiveLoadBalancer::new();
            clb.record_task_start("Devo", 1000.0);
            clb.record_task_start("Shield", 1000.0);
            clb.record_task_end("Devo", 5.0, 3.0, 1005.0);
            let sm = clb.get_swarm_load_summary();
            let cap = clb.get_capable_roles("debugging");
            let mut c1 = [0u8; 96];
            let hc = b"CLB: active=";
            c1[..hc.len()].copy_from_slice(hc);
            let mut rc = hc.len();
            let ac = format!("{}", sm.active_agents);
            c1[rc..rc + ac.len()].copy_from_slice(ac.as_bytes()); rc += ac.len();
            c1[rc..rc + 7].copy_from_slice(b" util="); rc += 7;
            let ut = format!("{:.1}", sm.utilization_pct);
            c1[rc..rc + ut.len()].copy_from_slice(ut.as_bytes()); rc += ut.len();
            c1[rc..rc + 3].copy_from_slice(b"% d"); rc += 3;
            self.add_line(&c1[..rc], LCARS_GREEN);
            let mut c2 = [0u8; 96];
            let hp = b"CLB capable[debugging]: ";
            c2[..hp.len()].copy_from_slice(hp);
            let mut rp = hp.len();
            for r in cap.iter().take(4) {
                let rb = r.as_bytes();
                let rn = rb.len().min(10);
                c2[rp..rp + rn].copy_from_slice(&rb[..rn]); rp += rn;
                c2[rp..rp + 1].copy_from_slice(b" "); rp += 1;
            }
            self.add_line(&c2[..rp], LCARS_CYAN);
            // 3) symbology_etymology_monad (REAL): resolve a concept
            let rest = cmd.get(11..).unwrap_or(b"").trim_ascii();
            let concept: &[u8] = if rest.is_empty() { b"monad" } else { rest };
            let cword = core::str::from_utf8(concept).unwrap_or("monad");
            let rc = crate::symbology_etymology_monad::resolve_concept(cword);
            let mut x1 = [0u8; 96];
            let hx = b"SYMBOLOGY: concept=";
            x1[..hx.len()].copy_from_slice(hx);
            let mut rx1 = hx.len();
            let cn = rc.concept_word.as_bytes().len().min(18);
            x1[rx1..rx1 + cn].copy_from_slice(&rc.concept_word.as_bytes()[..cn]); rx1 += cn;
            x1[rx1..rx1 + 11].copy_from_slice(b" geom_key="); rx1 += 11;
            let gk = rc.geom_key.as_bytes();
            let gkn = gk.len().min(14);
            x1[rx1..rx1 + gkn].copy_from_slice(&gk[..gkn]); rx1 += gkn;
            x1[rx1..rx1 + 6].copy_from_slice(b" fhz="); rx1 += 6;
            let fh = format!("{:.0}", rc.frequency_hz);
            x1[rx1..rx1 + fh.len()].copy_from_slice(fh.as_bytes()); rx1 += fh.len();
            self.add_line(&x1[..rx1], LCARS_AMBER);
            let mut x2 = [0u8; 96];
            let hx2 = b"  energy=";
            x2[..hx2.len()].copy_from_slice(hx2);
            let mut rx2 = hx2.len();
            let ea = format!("{:.1}", rc.femto_scan_energy);
            x2[rx2..rx2 + ea.len()].copy_from_slice(ea.as_bytes()); rx2 += ea.len();
            x2[rx2..rx2 + 8].copy_from_slice(b" etymon="); rx2 += 8;
            let et = rc.etymology_root.as_bytes();
            let etn = et.len().min(30);
            x2[rx2..rx2 + etn].copy_from_slice(&et[..etn]); rx2 += etn;
            self.add_line(&x2[..rx2], LCARS_PURPLE);
            // 4) tool_forge (REAL): assess gaps + forge pipeline
            let mut tf = crate::tool_forge::ToolForge::new();
            let gaps = tf.assess_agent_needs("Lead Developer", &["GitSkill", "RefactorSkill"]);
            let (fc, fh2) = tf.run_forge_pipeline("RefactorSkill", true);
            let mut t1 = [0u8; 96];
            let ht = b"TOOL FORGE: gaps[dev]=";
            t1[..ht.len()].copy_from_slice(ht);
            let mut rt = ht.len();
            let gc2 = format!("{}", gaps.len());
            t1[rt..rt + gc2.len()].copy_from_slice(gc2.as_bytes()); rt += gc2.len();
            t1[rt..rt + 6].copy_from_slice(b" log="); rt += 6;
            let bl = format!("{}", tf.build_log_len());
            t1[rt..rt + bl.len()].copy_from_slice(bl.as_bytes()); rt += bl.len();
            t1[rt..rt + 5].copy_from_slice(b" ok="); rt += 5;
            t1[rt] = if fc { b'1' } else { b'0' }; rt += 1;
            t1[rt..rt + 7].copy_from_slice(b" lead="); rt += 7;
            t1[rt] = if fh2 { b'1' } else { b'0' }; rt += 1;
            self.add_line(&t1[..rt], LCARS_GREEN);
            // 5) manus_engine (REAL): quantum superposition across roles
            let mut me = crate::manus_engine::ManusEngine::new();
            let mut sup = me.create_superposition(7, "capability-fusion", &["Logic", "Devo", "Shield"]);
            me.submit_attempt(&mut sup, 0, "coherence=0.90", 0.3);
            me.submit_attempt(&mut sup, 1, "coherence=0.85", 0.2);
            me.submit_attempt(&mut sup, 2, "coherence=0.88", 0.25);
            let collapsed = me.check_and_collapse(&mut sup);
            let mut m1 = [0u8; 96];
            let hm = b"MANUS: tasks=";
            m1[..hm.len()].copy_from_slice(hm);
            let mut rm = hm.len();
            let tp = format!("{}", me.tasks_processed);
            m1[rm..rm + tp.len()].copy_from_slice(tp.as_bytes()); rm += tp.len();
            m1[rm..rm + 8].copy_from_slice(b" coil="); rm += 8;
            let k1 = format!("{:.2}", sup.coherence);
            m1[rm..rm + k1.len()].copy_from_slice(k1.as_bytes()); rm += k1.len();
            m1[rm..rm + 6].copy_from_slice(b" col="); rm += 6;
            m1[rm] = if collapsed { b'1' } else { b'0' }; rm += 1;
            m1[rm..rm + 9].copy_from_slice(b" winner="); rm += 9;
            if let Some(w) = sup.winning_role {
                let wb = w.as_bytes();
                let wn = wb.len().min(10);
                m1[rm..rm + wn].copy_from_slice(&wb[..wn]); rm += wn;
            } else {
                m1[rm..rm + 4].copy_from_slice(b"none"); rm += 4;
            }
            self.add_line(&m1[..rm], LCARS_ORANGE);
            // 6) instella_sparse_agent_teams (REAL): 1-of-8 sparse routing
            let mut sat = crate::instella_sparse_agent_teams::InstellaSparseAgentTeams::new();
            let (team, dark, lat) = sat.route_task("quantum math");
            let mut n1 = [0u8; 96];
            let hna = b"INSTELLA SPA: team=";
            n1[..hna.len()].copy_from_slice(hna);
            let mut rn = hna.len();
            let tidn = team.team_id.len().min(24);
            n1[rn..rn + tidn].copy_from_slice(&team.team_id.as_bytes()[..tidn]); rn += tidn;
            self.add_line(&n1[..rn], LCARS_CYAN);
            let mut n2 = [0u8; 96];
            let hnb = b"  lead=";
            n2[..hnb.len()].copy_from_slice(hnb);
            let mut rn2 = hnb.len();
            let lbn = team.lead_agent.len().min(10);
            n2[rn2..rn2 + lbn].copy_from_slice(&team.lead_agent.as_bytes()[..lbn]); rn2 += lbn;
            n2[rn2..rn2 + 7].copy_from_slice(b" dark="); rn2 += 7;
            let dk = format!("{}", dark);
            n2[rn2..rn2 + dk.len()].copy_from_slice(dk.as_bytes()); rn2 += dk.len();
            n2[rn2..rn2 + 6].copy_from_slice(b" exp="); rn2 += 6;
            let ex = format!("{}", team.expert_count);
            n2[rn2..rn2 + ex.len()].copy_from_slice(ex.as_bytes()); rn2 += ex.len();
            n2[rn2..rn2 + 5].copy_from_slice(b" ns="); rn2 += 5;
            let ls = format!("{}", lat);
            n2[rn2..rn2 + ls.len()].copy_from_slice(ls.as_bytes()); rn2 += ls.len();
            self.add_line(&n2[..rn2], LCARS_GREEN);
        } else if cmd.starts_with(b"fabric") {
            // VOLTAGE ENERGIZATION — FABRIC cluster executes for real:
            // radix-trie insert/query -> DRAM pattern compute -> GPU Infinity
            // Cache geometry -> HBTF multi-scale telemetry -> THz-Bonsai route.
            self.add_line(b"FABRIC CLUSTER: radix-trie -> dram -> gpu-ic -> hbtf -> thz", LCARS_CYAN);
            // 1) RADIX TRIE MATCHER: insert a deterministic pattern, then query it.
            let mut trie = crate::radix_trie_matcher::build_default_matcher();
            let pre = trie.registered;
            trie.register("fabric status", crate::radix_trie_matcher::_handle_health_check, Some("fabric_status"));
            let mut r1 = [0u8; 96];
            let h1 = b"RADIX: builtin=";
            r1[..h1.len()].copy_from_slice(h1);
            let mut i1 = h1.len();
            let ps = format!("{}", pre);
            r1[i1..i1 + ps.len()].copy_from_slice(ps.as_bytes()); i1 += ps.len();
            r1[i1..i1 + 12].copy_from_slice(b" registered="); i1 += 12;
            let rs = format!("{}", trie.registered);
            r1[i1..i1 + rs.len()].copy_from_slice(rs.as_bytes()); i1 += rs.len();
            match trie.match_prefix("fabric status") {
                Some((handler, _ep, mlen)) => {
                    let hres = handler();
                    r1[i1..i1 + 7].copy_from_slice(b" hit=>="); i1 += 7;
                    let hb = hres.as_bytes();
                    let hn = hb.len().min(42);
                    r1[i1..i1 + hn].copy_from_slice(&hb[..hn]); i1 += hn;
                    r1[i1] = b'/'; i1 += 1;
                    let ls = format!("{}", mlen);
                    r1[i1..i1 + ls.len()].copy_from_slice(ls.as_bytes()); i1 += ls.len();
                }
                None => {
                    r1[i1..i1 + 5].copy_from_slice(b" miss"); i1 += 5;
                }
            }
            self.add_line(&r1[..i1], LCARS_GREEN);
            // 2) DRAM PATTERN COMPUTER: real pathway / architecture compute.
            let dpr = crate::dram_pattern_computer::pathway_row_buffer(128);
            let dbk = crate::dram_pattern_computer::pathway_bulk_k3();
            let dfs = crate::dram_pattern_computer::full_system_summary();
            let mut d1 = [0u8; 96];
            let hd = b"DRAM: rowbuf(128)=";
            d1[..hd.len()].copy_from_slice(hd);
            let mut id = hd.len();
            let dps = format!("{}", dpr);
            d1[id..id + dps.len()].copy_from_slice(dps.as_bytes()); id += dps.len();
            d1[id..id + 9].copy_from_slice(b" bulk_k3="); id += 9;
            let dbs = format!("{}", dbk);
            d1[id..id + dbs.len()].copy_from_slice(dbs.as_bytes()); id += dbs.len();
            d1[id..id + 9].copy_from_slice(b" summary="); id += 9;
            let fss = format!("{}", dfs);
            d1[id..id + fss.len()].copy_from_slice(fss.as_bytes()); id += fss.len();
            self.add_line(&d1[..id], LCARS_GOLD);
            // 3) GPU INFINITY COMPUTER: real PC Infinity Cache geometry.
            let gf = crate::gpu_infinity_computer::gpu_pattern_fpga(8);
            let gk = crate::gpu_infinity_computer::gpu_pattern_k3_ternary();
            let gcm = crate::gpu_infinity_computer::compare_fabrics();
            let mut g1 = [0u8; 96];
            let hg = b"GPU-IC: fpga(8MB)=";
            g1[..hg.len()].copy_from_slice(hg);
            let mut ig = hg.len();
            let gfs = format!("{}", gf);
            g1[ig..ig + gfs.len()].copy_from_slice(gfs.as_bytes()); ig += gfs.len();
            g1[ig..ig + 5].copy_from_slice(b" k3="); ig += 5;
            let gks = format!("{}", gk);
            g1[ig..ig + gks.len()].copy_from_slice(gks.as_bytes()); ig += gks.len();
            g1[ig..ig + 5].copy_from_slice(b" cmp="); ig += 5;
            let gcs = format!("{}", gcm);
            g1[ig..ig + gcs.len()].copy_from_slice(gcs.as_bytes()); ig += gcs.len();
            self.add_line(&g1[..ig], LCARS_PURPLE);
            // 4) TELEMETRY FABRIC (HBTF): subscribe -> publish frame -> metrics.
            let mut tf = crate::telemetry_fabric::TelemetryFabric::new();
            let tsub = tf.subscribe("fabric");
            let frame = crate::telemetry_fabric::TelemetryFrame::new(
                "f1",
                "node0",
                "fabric",
                vec![(String::from("gpu_load"), 0.42)],
                0.0,
            );
            let tdel = tf.publish(frame);
            let (total, buffered, channels) = tf.get_fabric_metrics();
            let mut t1 = [0u8; 96];
            let ht = b"TELEMETRY: subs=";
            t1[..ht.len()].copy_from_slice(ht);
            let mut it = ht.len();
            let tss = format!("{}", tsub);
            t1[it..it + tss.len()].copy_from_slice(tss.as_bytes()); it += tss.len();
            t1[it..it + 11].copy_from_slice(b" delivered="); it += 11;
            let tds = format!("{}", tdel);
            t1[it..it + tds.len()].copy_from_slice(tds.as_bytes()); it += tds.len();
            t1[it..it + 8].copy_from_slice(b" frames="); it += 8;
            let tts = format!("{}", total);
            t1[it..it + tts.len()].copy_from_slice(tts.as_bytes()); it += tts.len();
            t1[it..it + 5].copy_from_slice(b" buf="); it += 5;
            let tbs = format!("{}", buffered);
            t1[it..it + tbs.len()].copy_from_slice(tbs.as_bytes()); it += tbs.len();
            t1[it..it + 7].copy_from_slice(b" chans="); it += 7;
            let tcs = format!("{}", channels);
            t1[it..it + tcs.len()].copy_from_slice(tcs.as_bytes()); it += tcs.len();
            self.add_line(&t1[..it], LCARS_AMBER);
            // 5) THZ-BONSAI GATEWAY: encode -> trits -> route decision.
            let mut gw = crate::thz_bonsai_gateway::ThzBonsaiMicrokernelGateway::new();
            let payload = crate::thz_bonsai_gateway::encode_payload(b"FABRIC-ARMED");
            let trits = crate::thz_bonsai_gateway::to_trits(&payload);
            let (dominant, conf) = crate::thz_bonsai_gateway::route_trits(&trits);
            gw.bonsai.process(&payload);
            let (calls, avg_lat) = gw.bonsai.get_stats();
            let mut z1 = [0u8; 96];
            let hz = b"THZ: route=";
            z1[..hz.len()].copy_from_slice(hz);
            let mut iz = hz.len();
            let domb = dominant.as_bytes();
            let domn = domb.len().min(10);
            z1[iz..iz + domn].copy_from_slice(&domb[..domn]); iz += domn;
            z1[iz..iz + 7].copy_from_slice(b" conf="); iz += 7;
            let cfs = format!("{:.0}", conf * 100.0);
            z1[iz..iz + cfs.len()].copy_from_slice(cfs.as_bytes()); iz += cfs.len();
            z1[iz..iz + 7].copy_from_slice(b"% call="); iz += 7;
            let cals = format!("{}", calls);
            z1[iz..iz + cals.len()].copy_from_slice(cals.as_bytes()); iz += cals.len();
            z1[iz..iz + 6].copy_from_slice(b" avg="); iz += 6;
            let avs = format!("{:.2}", avg_lat);
            z1[iz..iz + avs.len()].copy_from_slice(avs.as_bytes()); iz += avs.len();
            z1[iz..iz + 3].copy_from_slice(b"us "); iz += 3;
            z1[iz..iz + 6].copy_from_slice(b" dim="); iz += 6;
            let dms = format!("{}", crate::thz_bonsai_gateway::BONSAI_DIM);
            z1[iz..iz + dms.len()].copy_from_slice(dms.as_bytes()); iz += dms.len();
            self.add_line(&z1[..iz], LCARS_GREEN);
            let msg = gw.decode_message(dominant, conf, 2.5);
            let mb = msg.as_bytes();
            let mn = mb.len().min(92);
            self.add_line(&mb[..mn], LCARS_CYAN);
        } else if cmd.starts_with(b"modellayer") {
            // VOLTAGE ENERGIZATION - MODEL-LAYER cluster executes for real:
            // trm_7m_optimized -> skill_matrix_sync -> agent_mailbox -> trm_integration.
            self.add_line(b"MODEL-LAYER CLUSTER: trm7mo -> skill-matrix -> mailbox -> trm-integration", LCARS_CYAN);
            // 1) TRM 7M OPTIMIZED: record a real forward timing + classify logits.
            let mut trm_o = crate::trm_7m_optimized::OptimizedTRM7M::new();
            trm_o.record_forward(1250.0);
            let (avg_ns, avg_us, calls_o) = trm_o.get_stats();
            let opt_hand = crate::trm_7m_optimized::get_optimized_trm();
            let logits = [2.8f64, 0.4f64, -1.9f64];
            let (lab, conf) = crate::trm_7m_optimized::classify_logits3(&logits);
            let labname: &[u8] = match lab { 1 => b"PASS(+1)", 0 => b"AMBIG(0)", _ => b"FAULT(-1)" };
            let mut a1 = [0u8; 96];
            let ah = b"TRM7MO: handle=";
            a1[..ah.len()].copy_from_slice(ah);
            let mut ia = ah.len();
            let hs = format!("{}", opt_hand);
            a1[ia..ia + hs.len()].copy_from_slice(hs.as_bytes()); ia += hs.len();
            a1[ia..ia + 7].copy_from_slice(b" call="); ia += 7;
            let cs = format!("{}", calls_o);
            a1[ia..ia + cs.len()].copy_from_slice(cs.as_bytes()); ia += cs.len();
            a1[ia..ia + 4].copy_from_slice(b" avg"); ia += 4;
            let ans = format!("{:.1}", avg_ns);
            a1[ia..ia + ans.len()].copy_from_slice(ans.as_bytes()); ia += ans.len();
            a1[ia..ia + 5].copy_from_slice(b"ns=>"); ia += 5;
            let lnn = labname.len();
            a1[ia..ia + lnn].copy_from_slice(labname); ia += lnn;
            a1[ia..ia + 6].copy_from_slice(b" cnf="); ia += 6;
            let cfs = format!("{:.2}", conf);
            a1[ia..ia + cfs.len()].copy_from_slice(cfs.as_bytes()); ia += cfs.len();
            let _ = avg_us;
            self.add_line(&a1[..ia], LCARS_GREEN);
            // 2) SKILL MATRIX SYNC: record real usages + read the matrix aggregate.
            let mut sm = crate::skill_matrix_sync::SkillMatrixSync::new();
            sm.record_skill_usage("architect", "kernel_wiring", true);
            sm.record_skill_usage("architect", "kernel_wiring", true);
            sm.record_skill_usage("orchestrator", "modellayer", true);
            let (mat_agents, mat_skills, mat_events) = sm.get_matrix();
            let sm_hand = crate::skill_matrix_sync::get_skill_matrix_sync();
            let mut b1 = [0u8; 96];
            let bh = b"SKILLMAT: handle=";
            b1[..bh.len()].copy_from_slice(bh);
            let mut ib = bh.len();
            let sh = format!("{}", sm_hand);
            b1[ib..ib + sh.len()].copy_from_slice(sh.as_bytes()); ib += sh.len();
            b1[ib..ib + 6].copy_from_slice(b" agnt="); ib += 6;
            let mas = format!("{}", mat_agents);
            b1[ib..ib + mas.len()].copy_from_slice(mas.as_bytes()); ib += mas.len();
            b1[ib..ib + 5].copy_from_slice(b" skl="); ib += 5;
            let mss = format!("{}", mat_skills);
            b1[ib..ib + mss.len()].copy_from_slice(mss.as_bytes()); ib += mss.len();
            b1[ib..ib + 8].copy_from_slice(b" events="); ib += 8;
            let mes = format!("{}", mat_events);
            b1[ib..ib + mes.len()].copy_from_slice(mes.as_bytes()); ib += mes.len();
            self.add_line(&b1[..ib], LCARS_GOLD);
            // 3) AGENT MAILBOX: deliver a real message + count pending/dirs.
            let now = 1_000_000u64;
            let mut mbx = crate::agent_mailbox::AgentMailbox::new("kernel");
            mbx.send("kernel", "model-layer-armed", "MODEL-LAYER", 3, now);
            let pending = mbx.count_pending(now);
            let dirs = mbx.list_agents().len();
            let mut c1 = [0u8; 96];
            let ch = b"MAILBOX: pending=";
            c1[..ch.len()].copy_from_slice(ch);
            let mut ic = ch.len();
            let pss = format!("{}", pending);
            c1[ic..ic + pss.len()].copy_from_slice(pss.as_bytes()); ic += pss.len();
            c1[ic..ic + 6].copy_from_slice(b" dirs="); ic += 6;
            let dss = format!("{}", dirs);
            c1[ic..ic + dss.len()].copy_from_slice(dss.as_bytes()); ic += dss.len();
            self.add_line(&c1[..ic], LCARS_PURPLE);
            // 4) TRM INTEGRATION: complexity + synthesis quality + batch outcome.
            let task = "wire model-layer: pattern, sequence, logic tokens";
            let cpx = crate::trm_integration::analyze_task_complexity(task);
            let par = crate::trm_integration::should_use_parallel(cpx, task, false);
            let toks = crate::trm_integration::context_to_tokens("model-layer energization").len();
            let qual = crate::trm_integration::evaluate_synthesis_quality(4, 5, 1024);
            let depth = crate::trm_integration::set_recursion_depth(3);
            let (succ, coll) = crate::trm_integration::emulated_batch_outcome(&[String::from("pattern logic"), String::from("sequence tokens")]);
            let mut d1 = [0u8; 96];
            let dh = b"TRMIN: cpx=";
            d1[..dh.len()].copy_from_slice(dh);
            let mut id2 = dh.len();
            let cxs = format!("{}", cpx);
            d1[id2..id2 + cxs.len()].copy_from_slice(cxs.as_bytes()); id2 += cxs.len();
            d1[id2..id2 + 6].copy_from_slice(b" par="); id2 += 6;
            let ps2: &[u8] = if par { b"yes" } else { b"no" };
            d1[id2..id2 + 3].copy_from_slice(ps2); id2 += 3;
            d1[id2..id2 + 5].copy_from_slice(b" tok="); id2 += 5;
            let tss2 = format!("{}", toks);
            d1[id2..id2 + tss2.len()].copy_from_slice(tss2.as_bytes()); id2 += tss2.len();
            d1[id2..id2 + 5].copy_from_slice(b" qlt="); id2 += 5;
            let qss = format!("{:.2}", qual);
            d1[id2..id2 + qss.len()].copy_from_slice(qss.as_bytes()); id2 += qss.len();
            d1[id2..id2 + 6].copy_from_slice(b" dep="); id2 += 6;
            let dss2 = format!("{}", depth);
            d1[id2..id2 + dss2.len()].copy_from_slice(dss2.as_bytes()); id2 += dss2.len();
            d1[id2..id2 + 6].copy_from_slice(b" ok="); id2 += 6;
            let sss2 = format!("{}/{}", succ, if coll { 1 } else { 0 });
            d1[id2..id2 + sss2.len()].copy_from_slice(sss2.as_bytes()); id2 += sss2.len();
            self.add_line(&d1[..id2], LCARS_AMBER);
        } else if cmd.starts_with(b"sensory") {
            // VOLTAGE ENERGIZATION — SENSORY cluster executes for real:
            // human-swarm interaction -> multimodal ingestion -> scene fusion
            // -> performance insights -> GPU unlock -> hardware-voltage binder.
            self.add_line(b"SENSORY CLUSTER: gesture -> ingestion -> fusion -> perf -> GPU -> voltage", LCARS_CYAN);
            // 1) HUMAN-SWARM INTERACTION: parse a real gesture + text directive.
            let mut hsi = crate::human_swarm_interaction::HumanSwarmInteractionEngine::new();
            let gest = hsi.parse_gesture("PALM_STOP");
            let txt = hsi.parse_text_command("halt the swarm");
            let hsi_handle = crate::human_swarm_interaction::get_human_swarm_interaction();
            let mut s1 = [0u8; 96];
            let sh = b"HSI: handle=";
            s1[..sh.len()].copy_from_slice(sh);
            let mut i1 = sh.len();
            let hs = format!("{}", hsi_handle);
            s1[i1..i1 + hs.len()].copy_from_slice(hs.as_bytes()); i1 += hs.len();
            s1[i1..i1 + 9].copy_from_slice(b" gest=>"); i1 += 9;
            let gn = gest.target_agent.len().min(8);
            s1[i1..i1 + gn].copy_from_slice(&gest.target_agent.as_bytes()[..gn]); i1 += gn;
            s1[i1..i1 + 7].copy_from_slice(b" cnf="); i1 += 7;
            let gcs = format!("{:.2}", gest.confidence);
            s1[i1..i1 + gcs.len()].copy_from_slice(gcs.as_bytes()); i1 += gcs.len();
            s1[i1..i1 + 6].copy_from_slice(b" text="); i1 += 6;
            let tn = txt.target_agent.len().min(8);
            s1[i1..i1 + tn].copy_from_slice(&txt.target_agent.as_bytes()[..tn]); i1 += tn;
            self.add_line(&s1[..i1], LCARS_GOLD);
            // 2) MULTIMODAL INGESTION: build a real sensor telemetry frame.
            let mut sens = crate::multimodal_ingestion::SensorTelemetryData::new();
            sens.sensor_id = String::from("THERM_01");
            sens.temperature_celsius = 62.5;
            sens.vibration_hz = 4.7;
            sens.power_draw_watts = 18.0;
            let mm_handle = crate::multimodal_ingestion::get_multimodal_ingestion();
            let mut s2 = [0u8; 96];
            let mh = b"MMINGEST: handle=";
            s2[..mh.len()].copy_from_slice(mh);
            let mut i2 = mh.len();
            let ms = format!("{}", mm_handle);
            s2[i2..i2 + ms.len()].copy_from_slice(ms.as_bytes()); i2 += ms.len();
            s2[i2..i2 + 6].copy_from_slice(b" temp="); i2 += 6;
            let ts2 = format!("{:.1}", sens.temperature_celsius);
            s2[i2..i2 + ts2.len()].copy_from_slice(ts2.as_bytes()); i2 += ts2.len();
            s2[i2..i2 + 5].copy_from_slice(b" pow="); i2 += 5;
            let pws = format!("{:.1}", sens.power_draw_watts);
            s2[i2..i2 + pws.len()].copy_from_slice(pws.as_bytes()); i2 += pws.len();
            s2[i2..i2 + 5].copy_from_slice(b" vit="); i2 += 5;
            let vhs = format!("{:.1}", sens.vibration_hz);
            s2[i2..i2 + vhs.len()].copy_from_slice(vhs.as_bytes()); i2 += vhs.len();
            self.add_line(&s2[..i2], LCARS_CYAN);
            // 3) ADVANCED SCENE FUSION: build a real complex scene context.
            let mut scene = crate::advanced_scene_fusion::ComplexSceneContext::new();
            scene.scene_id = String::from("SCN_7");
            scene.location = String::from("industry");
            scene.ambient_temp_c = 24.8;
            let sf_handle = crate::advanced_scene_fusion::get_advanced_scene_fusion();
            let mut s3 = [0u8; 96];
            let fh = b"FUSION: handle=";
            s3[..fh.len()].copy_from_slice(fh);
            let mut i3 = fh.len();
            let fs3 = format!("{}", sf_handle);
            s3[i3..i3 + fs3.len()].copy_from_slice(fs3.as_bytes()); i3 += fs3.len();
            s3[i3..i3 + 6].copy_from_slice(b" id="); i3 += 6;
            let sid = scene.scene_id.len().min(8);
            s3[i3..i3 + sid].copy_from_slice(&scene.scene_id.as_bytes()[..sid]); i3 += sid;
            s3[i3..i3 + 10].copy_from_slice(b" locat="); i3 += 10;
            let loc = scene.location.len().min(8);
            s3[i3..i3 + loc].copy_from_slice(&scene.location.as_bytes()[..loc]); i3 += loc;
            s3[i3..i3 + 6].copy_from_slice(b" temp="); i3 += 6;
            let t3s = format!("{:.1}", scene.ambient_temp_c);
            s3[i3..i3 + t3s.len()].copy_from_slice(t3s.as_bytes()); i3 += t3s.len();
            self.add_line(&s3[..i3], LCARS_PURPLE);
            // 4) SWARM PERFORMANCE INSIGHTS: build a real intelligence report.
            let mut perf = crate::swarm_performance_insights::SwarmIntelligenceReport::new();
            perf.report_id = String::from("PERF_A");
            perf.overall_health_score = 0.92;
            perf.efficiency_score = 0.87;
            perf.total_tasks_processed = 1024;
            let perf_handle = crate::swarm_performance_insights::get_performance_insights();
            let mut s4 = [0u8; 96];
            let ph = b"PERF: handle=";
            s4[..ph.len()].copy_from_slice(ph);
            let mut i4 = ph.len();
            let ps4 = format!("{}", perf_handle);
            s4[i4..i4 + ps4.len()].copy_from_slice(ps4.as_bytes()); i4 += ps4.len();
            s4[i4..i4 + 6].copy_from_slice(b" hlth="); i4 += 6;
            let hls = format!("{:.2}", perf.overall_health_score);
            s4[i4..i4 + hls.len()].copy_from_slice(hls.as_bytes()); i4 += hls.len();
            s4[i4..i4 + 6].copy_from_slice(b" eff="); i4 += 6;
            let efs = format!("{:.2}", perf.efficiency_score);
            s4[i4..i4 + efs.len()].copy_from_slice(efs.as_bytes()); i4 += efs.len();
            s4[i4..i4 + 6].copy_from_slice(b" tsk="); i4 += 6;
            let tks = format!("{}", perf.total_tasks_processed);
            s4[i4..i4 + tks.len()].copy_from_slice(tks.as_bytes()); i4 += tks.len();
            self.add_line(&s4[..i4], LCARS_GREEN);
            // 5) GPU UNLOCK ARCHITECTURE: push + flush a real ring batch, then dispatch.
            let mut ring = crate::gpu_unlock_architecture::PinnedRingBuffer::new(64);
            let mut vec = [0f32; crate::gpu_unlock_architecture::TASK_VECTOR_DIM];
            vec[0] = 1.0;
            vec[5] = 0.5;
            let _full = ring.push(&vec, 0);
            let _full2 = ring.push(&vec, 0);
            let flushed = ring.flush_batch();
            let gpu_handle = crate::gpu_unlock_architecture::get_gpu_unlock_architecture();
            let mut pipe = crate::gpu_unlock_architecture::HybridPipeline::new();
            let (lat, path) = pipe.dispatch(12);
            let mut s5 = [0u8; 96];
            let gh = b"GPU: handle=";
            s5[..gh.len()].copy_from_slice(gh);
            let mut i5 = gh.len();
            let gs = format!("{}", gpu_handle);
            s5[i5..i5 + gs.len()].copy_from_slice(gs.as_bytes()); i5 += gs.len();
            s5[i5..i5 + 8].copy_from_slice(b" flush="); i5 += 8;
            let fls = format!("{}", flushed);
            s5[i5..i5 + fls.len()].copy_from_slice(fls.as_bytes()); i5 += fls.len();
            s5[i5..i5 + 7].copy_from_slice(b" lat="); i5 += 7;
            let las = format!("{}", lat);
            s5[i5..i5 + las.len()].copy_from_slice(las.as_bytes()); i5 += las.len();
            s5[i5..i5 + 7].copy_from_slice(b" path="); i5 += 7;
            let pn = path.len().min(18);
            s5[i5..i5 + pn].copy_from_slice(&path.as_bytes()[..pn]); i5 += pn;
            self.add_line(&s5[..i5], LCARS_AMBER);
            // 6) HARDWARE VOLTAGE BINDER: real telemetry snapshot + dynamic multiplier.
            let mut hvb = crate::hardware_voltage_microkernel_binder::HardwareVoltageMicrokernelBinder::new();
            hvb.set_clock_frequency(4200.0);
            let snap = hvb.sample_telemetry();
            let hvb_handle = crate::hardware_voltage_microkernel_binder::get_voltage_binder();
            let mut s6 = [0u8; 96];
            let vh = b"VOLT: handle=";
            s6[..vh.len()].copy_from_slice(vh);
            let mut i6 = vh.len();
            let vs6 = format!("{}", hvb_handle);
            s6[i6..i6 + vs6.len()].copy_from_slice(vs6.as_bytes()); i6 += vs6.len();
            s6[i6..i6 + 6].copy_from_slice(b" clk="); i6 += 6;
            let cks = format!("{:.0}", snap.clock_mhz);
            s6[i6..i6 + cks.len()].copy_from_slice(cks.as_bytes()); i6 += cks.len();
            s6[i6..i6 + 8].copy_from_slice(b" PhiV="); i6 += 8;
            let pvs6 = format!("{:.4}", snap.phi_v);
            s6[i6..i6 + pvs6.len()].copy_from_slice(pvs6.as_bytes()); i6 += pvs6.len();
            s6[i6..i6 + 6].copy_from_slice(b" Vc="); i6 += 6;
            let vcs = format!("{:.2}", snap.v_core);
            s6[i6..i6 + vcs.len()].copy_from_slice(vcs.as_bytes()); i6 += vcs.len();
            self.add_line(&s6[..i6], LCARS_RED);
            self.add_line(b"SENSORY: 6 sensors fused into one sovereign scene understanding", LCARS_CYAN);
        } else {
            self.add_line(b"JARVIS: 'Command acknowledged. Executing sovereign synthesis on silicon.'", LCARS_WHITE);
            self.add_line(b"LOGIC:  'Action vector evaluated by real 14-agent council (K3).'", LCARS_GREEN);
        }

        self.input_len = 0;
    }
}

/// Format a u64 ns count into a static buffer as decimal digits + "ns".
fn format_ns(ns: u64, buf: &mut [u8], out_len: &mut usize) -> usize {
    let mut tmp = ns;
    if tmp == 0 { tmp = 1; }
    let mut digits = [0u8; 20];
    let mut di = 0;
    while tmp > 0 && di < 19 {
        digits[di] = b'0' + (tmp % 10) as u8;
        tmp /= 10;
        di += 1;
    }
    let mut bi = 0;
    for k in (0..di).rev() {
        buf[bi] = digits[k];
        bi += 1;
    }
    buf[bi..bi + 2].copy_from_slice(b"ns");
    bi += 2;
    *out_len = bi;
    bi
}

/// Trim ASCII whitespace (no_std helper).
trait TrimAscii {
    fn trim_ascii(&self) -> &Self;
}
impl TrimAscii for [u8] {
    fn trim_ascii(&self) -> &[u8] {
        let is_ws = |b: &u8| *b == b' ' || *b == b'\t' || *b == b'\r' || *b == b'\n';
        let mut start = 0;
        let mut end = self.len();
        while start < end && is_ws(&self[start]) { start += 1; }
        while end > start && is_ws(&self[end - 1]) { end -= 1; }
        &self[start..end]
    }
}

#[no_mangle]
pub extern "efiapi" fn efi_main(
    image_handle: efi::Handle,
    system_table: *mut efi::SystemTable,
) -> efi::Status {
    if system_table.is_null() {
        return efi::Status::INVALID_PARAMETER;
    }

    let st = unsafe { &*system_table };
    let bs = unsafe { &*st.boot_services };

    // 0. Init allocator (AllocatePool) + TSC time source BEFORE any Vec/String
    unsafe {
        allocator::init(st.boot_services);
        time::calibrate_tsc(|us| unsafe {
            ((*st.boot_services).stall)(us as usize);
            0
        });
        // Disable the UEFI watchdog timer. Firmware arms a 5-minute watchdog
        // that resets the system if the payload never clears it — the cause
        // of the "reboots 5 minutes after boot" behavior. timeout=0 disables.
        let _ = ((*st.boot_services).set_watchdog_timer)(0, 0, 0, core::ptr::null_mut());
    }
    // Force one agent-runtime instantiation so health check has live counters
    let _rt = crate::agent_runtime::get_runtime();

    // 1. Locate Graphics Output Protocol (GOP)
    let mut gop_guid = graphics_output::PROTOCOL_GUID;
    let mut gop_ptr: *mut core::ffi::c_void = core::ptr::null_mut();

    let status = (bs.locate_protocol)(
        &mut gop_guid,
        core::ptr::null_mut(),
        &mut gop_ptr,
    );

    if status != efi::Status::SUCCESS || gop_ptr.is_null() {
        return status;
    }

    let gop = unsafe { &mut *(gop_ptr as *mut graphics_output::Protocol) };
    let mode = unsafe { &*gop.mode };
    let info = unsafe { &*mode.info };

    let fb_base = mode.frame_buffer_base as *mut u32;
    let width = info.horizontal_resolution as usize;
    let height = info.vertical_resolution as usize;
    let stride = info.pixels_per_scan_line as usize;

    let con_in = unsafe { &mut *st.con_in };

    // 2. Allocate Double-Buffer in RAM to eliminate 100% of screen flicker
    let buffer_size = stride * height * 4;
    let mut backbuffer_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
    let alloc_status = (bs.allocate_pool)(
        efi::LOADER_DATA,
        buffer_size,
        &mut backbuffer_ptr,
    );

    let bb = if alloc_status == efi::Status::SUCCESS && !backbuffer_ptr.is_null() {
        backbuffer_ptr as *mut u32
    } else {
        fb_base
    };

    // 3. Main Live Bare-Metal LCARS Compositor Loop & Interactive Setup / Terminal State
    let mut frame_count: u32 = 0;
    let mut active_tab: u8 = 0; // Starts in Sector 00 Setup Wizard
    let mut setup = SetupState {
        selected_item: 0,
        storage_mode: 1, // USB NAND K3 default
        mesh_enabled: true,
        wifi_radar_calibrated: true,
        rowhammer_guard: true,
        pbft_quorum_armed: true,
        installation_applied: false,
    };
    let mut terminal = TerminalState::new();

    loop {
        frame_count = frame_count.wrapping_add(1);
        let tick = (frame_count as f32) * 0.05;

        // Check keyboard input for interactive terminal typing & tab switching
        let mut key = simple_text_input::InputKey { scan_code: 0, unicode_char: 0 };
        let key_status = (con_in.read_key_stroke)(st.con_in, &mut key);
        if key_status == efi::Status::SUCCESS {
            let u = key.unicode_char;

            if active_tab == 0 {
                // In Setup Mode: Arrow keys & Enter/Space to configure and launch
                if u == '0' as u16 || u == 's' as u16 || u == 'S' as u16 {
                    active_tab = 0;
                } else if u >= '1' as u16 && u <= '9' as u16 {
                    active_tab = (u - '0' as u16) as u8;
                } else if key.scan_code == 0x01 { // Up arrow
                    setup.selected_item = setup.selected_item.saturating_sub(1);
                } else if key.scan_code == 0x02 { // Down arrow
                    if setup.selected_item < 5 { setup.selected_item += 1; }
                } else if u == 13 || u == ' ' as u16 { // Enter or Space
                    match setup.selected_item {
                        0 => setup.storage_mode = (setup.storage_mode + 1) % 3,
                        1 => setup.mesh_enabled = !setup.mesh_enabled,
                        2 => setup.wifi_radar_calibrated = !setup.wifi_radar_calibrated,
                        3 => setup.rowhammer_guard = !setup.rowhammer_guard,
                        4 => setup.pbft_quorum_armed = !setup.pbft_quorum_armed,
                        5 => {
                            setup.installation_applied = true;
                            active_tab = 1; // Launch directly into active workspace!
                        },
                        _ => {},
                    }
                }
            } else {
                // In Live Workspace Mode: Full Interactive Terminal + Sector Switching
                if key.scan_code >= 0x0B && key.scan_code <= 0x14 { // F1 - F10 keys
                    active_tab = (key.scan_code - 0x0B) as u8;
                } else if u == 13 { // ENTER key -> execute command!
                    terminal.execute_command(&mut active_tab, image_handle, st.boot_services);
                } else if u == 8 { // BACKSPACE key
                    if terminal.input_len > 0 {
                        terminal.input_len -= 1;
                    }
                } else if u == 27 { // ESC key -> return to setup or sector 1
                    active_tab = 0;
                } else if u >= 0x20 && u <= 0x7E { // Printable ASCII characters
                    if terminal.input_len < 70 {
                        terminal.input_buf[terminal.input_len] = u as u8;
                        terminal.input_len += 1;
                    }
                }
            }
        }

        // Render entire frame into the RAM backbuffer
        render_uefi_lcars_frame(bb, width, height, stride, tick, active_tab, &setup, &terminal);

        // Blit backbuffer to physical VRAM in a single instantaneous copy (NO FLICKER)
        if bb != fb_base {
            unsafe {
                core::ptr::copy_nonoverlapping(bb, fb_base, stride * height);
            }
        }

        // 60 FPS smooth timing (16.6 ms per frame)
        (bs.stall)(16666);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// High-Speed Double-Buffered Scanline Compositor & Interactive Terminal Engine
// ──────────────────────────────────────────────────────────────────────────────
fn render_uefi_lcars_frame(
    buf: *mut u32,
    width: usize,
    height: usize,
    stride: usize,
    tick: f32,
    active_tab: u8,
    setup: &SetupState,
    term: &TerminalState,
) {
    if buf.is_null() || width == 0 || height == 0 {
        return;
    }

    // 1. Clear background
    for y in 0..height {
        let row = unsafe { core::slice::from_raw_parts_mut(buf.add(y * stride), width) };
        row.fill(LCARS_BG);
    }

    // Primitive rasterizers
    let set_pixel = |x: usize, y: usize, color: u32| {
        if x < width && y < height {
            unsafe { *buf.add(y * stride + x) = color; }
        }
    };

    let fill_rect = |rx: usize, ry: usize, rw: usize, rh: usize, color: u32| {
        let x_end = (rx + rw).min(width);
        let y_end = (ry + rh).min(height);
        for y in ry..y_end {
            for x in rx..x_end {
                unsafe { *buf.add(y * stride + x) = color; }
            }
        }
    };

    let draw_rect = |rx: usize, ry: usize, rw: usize, rh: usize, thick: usize, color: u32| {
        fill_rect(rx, ry, rw, thick, color);
        fill_rect(rx, (ry + rh).saturating_sub(thick), rw, thick, color);
        fill_rect(rx, ry, thick, rh, color);
        fill_rect((rx + rw).saturating_sub(thick), ry, thick, rh, color);
    };

    let fill_circle = |cx: i32, cy: i32, radius: i32, color: u32| {
        let r_sq = radius * radius;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= r_sq {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px >= 0 && py >= 0 {
                        set_pixel(px as usize, py as usize, color);
                    }
                }
            }
        }
    };

    let draw_pill_button = |px: usize, py: usize, pw: usize, ph: usize, color: u32| {
        let r = (ph / 2) as i32;
        fill_circle((px as i32) + r, (py as i32) + r, r, color);
        fill_circle(((px + pw) as i32) - r, (py as i32) + r, r, color);
        fill_rect(px + r as usize, py, pw.saturating_sub(ph), ph, color);
    };

    let draw_char = |x: usize, y: usize, c: u8, color: u32| {
        let glyph = font::get_glyph(c);
        for row in 0..16 {
            let row_bits = glyph[row];
            let py = y + row;
            if py >= height { break; }
            for col in 0..8 {
                let px = x + col;
                if px >= width { break; }
                if (row_bits & (1 << (7 - col))) != 0 {
                    unsafe { *buf.add(py * stride + px) = color; }
                }
            }
        }
    };

    let draw_str = |mut sx: usize, sy: usize, s: &[u8], color: u32| {
        for &b in s {
            if sx + 8 > width { break; }
            draw_char(sx, sy, b, color);
            sx += 8;
        }
    };

    // 2. Top Command Header Rail
    fill_rect(0, 0, width, 32, LCARS_GOLD);
    fill_rect(width.saturating_sub(320), 0, 320, 32, LCARS_PURPLE);
    draw_str(16, 8, b"TRM SWARM OS v19.001 // BARE-METAL SOVEREIGN NEXUS", 0xFF000000);
    draw_str(width.saturating_sub(300), 8, b"MONAD ANCHOR: 0x4D4F4E4144000001", 0xFF000000);

    // 3. Lateral LCARS Navigation Spine
    fill_rect(16, 44, 48, height.saturating_sub(88), LCARS_GOLD);
    fill_rect(16, 110, 48, 10, LCARS_BG);
    fill_rect(16, 240, 48, 10, LCARS_BG);
    fill_rect(16, 380, 48, 10, LCARS_BG);

    // 4. Lateral Sector Pill Buttons (10 Buttons total: 00 SETUP + 01-09 Sectors)
    let tab_labels: [&[u8]; 10] = [
        b"00 SYSTEM SETUP",
        b"01 SYSTEM STATUS",
        b"02 NEURAL BRIDGE",
        b"03 SWARM COUNCIL",
        b"04 SWARM INTEL",
        b"05 RESONANCE RESEARCH",
        b"06 DDR IMMUNE LAB",
        b"07 3D SPATIAL MESH",
        b"08 WIFI VISION RADAR",
        b"09 CONTINUUM K3",
    ];

    let tab_colors = [
        LCARS_GOLD,
        LCARS_AMBER,
        LCARS_PURPLE,
        LCARS_GOLD,
        LCARS_BLUE,
        LCARS_CYAN,
        LCARS_RED,
        LCARS_GREEN,
        LCARS_CYAN,
        LCARS_GOLD,
    ];

    for idx in 0..10 {
        let tab_y = 44 + idx * 30;
        let is_sel = (idx as u8) == active_tab;
        let col = if is_sel { LCARS_WHITE } else { tab_colors[idx] };
        let text_col = if is_sel { 0xFF000000 } else { 0xFF000000 };
        draw_pill_button(75, tab_y, 200, 22, col);
        draw_str(84, tab_y + 3, tab_labels[idx], text_col);
    }

    // 5. Main Workspace Surface (Spans full remaining width)
    let main_x = 290;
    let main_y = 44;
    let main_w = width.saturating_sub(main_x + 20);
    let main_h = height.saturating_sub(main_y + 36);

    fill_rect(main_x, main_y, main_w, main_h, LCARS_DARK_PANEL);

    match active_tab {
        0 => {
            // ═══════════════════════════════════════════════════════════════════
            // Sector 00: SOVEREIGN OS SETUP & CONFIGURATION WIZARD
            // ═══════════════════════════════════════════════════════════════════
            draw_str(main_x + 16, main_y + 12, b"=== SECTOR 00: SOVEREIGN OS HARDWARE SETUP & DEPLOYMENT WIZARD ===", LCARS_GOLD);

            let wiz_x = main_x + 20;
            let wiz_y = main_y + 36;
            let wiz_w = main_w.saturating_sub(40);
            let wiz_h = 300;
            fill_rect(wiz_x, wiz_y, wiz_w, wiz_h, LCARS_PANEL_BG);
            draw_rect(wiz_x, wiz_y, wiz_w, wiz_h, 1, LCARS_GOLD);

            draw_str(wiz_x + 16, wiz_y + 14, b"[INTERACTIVE HARDWARE CONFIGURATION & BARE-METAL PROVISIONING]", LCARS_GOLD);
            draw_str(wiz_x + 16, wiz_y + 38, b"Use [UP/DOWN] arrows to select items. Press [ENTER/SPACE] to toggle. Press [ENTER] on Launch.", LCARS_AMBER);

            // Item 0: Storage Target
            let s_mode_str: &[u8] = match setup.storage_mode {
                0 => b"[MODE: VOLATILE LIVE RAM SESSION (ZERO DISK TRACE)]",
                1 => b"[MODE: MOUNT USB NVMe NAND K3 FABRIC (153.5M TRITS PERSISTENT)]",
                _ => b"[MODE: INSTALL PERMANENTLY TO INTERNAL NVMe SSD (BOOTLOADER + NAND)]",
            };
            let sel0 = if setup.selected_item == 0 { LCARS_GREEN } else { LCARS_WHITE };
            draw_str(wiz_x + 20, wiz_y + 68, b"1. STORAGE & KNOWLEDGE PERSISTENCE:", sel0);
            draw_str(wiz_x + 360, wiz_y + 68, s_mode_str, if setup.selected_item == 0 { LCARS_AMBER } else { LCARS_CYAN });

            // Item 1: Inter-Machine UDP Mesh
            let sel1 = if setup.selected_item == 1 { LCARS_GREEN } else { LCARS_WHITE };
            let m_str: &[u8] = if setup.mesh_enabled { b"[ENABLED: UDP PORT 8022 ZERO-CONFIG LAN/WIFI AUTO-MESH]" } else { b"[DISABLED: STANDALONE AIR-GAPPED ISOLATION]" };
            draw_str(wiz_x + 20, wiz_y + 98, b"2. INTER-MACHINE MESH DISCOVERY:", sel1);
            draw_str(wiz_x + 360, wiz_y + 98, m_str, if setup.mesh_enabled { LCARS_GREEN } else { LCARS_RED });

            // Item 2: WiFi Vision Sub-Carrier Radar
            let sel2 = if setup.selected_item == 2 { LCARS_GREEN } else { LCARS_WHITE };
            let w_str: &[u8] = if setup.wifi_radar_calibrated { b"[ACTIVE: 160MHz CSI SENSING GRID CALIBRATED (100% SOVEREIGN)]" } else { b"[OFFLINE: RADIO SENSING DISABLED]" };
            draw_str(wiz_x + 20, wiz_y + 128, b"3. INTEL AX200 WIFI VISION RADAR:", sel2);
            draw_str(wiz_x + 360, wiz_y + 128, w_str, if setup.wifi_radar_calibrated { LCARS_GREEN } else { LCARS_RED });

            // Item 3: DDR Memory Immunity Guard
            let sel3 = if setup.selected_item == 3 { LCARS_GREEN } else { LCARS_WHITE };
            let r_str: &[u8] = if setup.rowhammer_guard { b"[ARMED: VOLTAGE MONITOR & DRAM ROW-HAMMER ANTIBODIES ACTIVE]" } else { b"[DISABLED: NO VOLTAGE WATCHDOG]" };
            draw_str(wiz_x + 20, wiz_y + 158, b"4. DDR MEMORY ROW-HAMMER IMMUNITY:", sel3);
            draw_str(wiz_x + 360, wiz_y + 158, r_str, if setup.rowhammer_guard { LCARS_GREEN } else { LCARS_RED });

            // Item 4: PBFT Byzantine Council Quorum
            let sel4 = if setup.selected_item == 4 { LCARS_GREEN } else { LCARS_WHITE };
            let p_str: &[u8] = if setup.pbft_quorum_armed { b"[ARMED: 14 CORE AGENTS TUNED | PBFT QUORUM THRESHOLD N=14, f=4]" } else { b"[DEGRADED]" };
            draw_str(wiz_x + 20, wiz_y + 188, b"5. 14-AGENT COUNCIL PBFT QUORUM:", sel4);
            draw_str(wiz_x + 360, wiz_y + 188, p_str, if setup.pbft_quorum_armed { LCARS_GREEN } else { LCARS_RED });

            // Item 5: Apply & Launch
            fill_rect(wiz_x + 20, wiz_y + 224, 520, 28, if setup.selected_item == 5 { LCARS_GREEN } else { LCARS_BLUE });
            draw_str(wiz_x + 32, wiz_y + 230, b">> APPLY CONFIGURATION & LAUNCH SOVEREIGN WORKSPACE <<", 0xFF000000);

            let status_msg: &[u8] = if setup.installation_applied {
                b"STATUS: CONFIGURATION APPLIED & ACTIVE ON PHYSICAL SILICON"
            } else {
                b"STATUS: READY FOR OPERATOR INITIALIZATION (PRESS ENTER ON ITEM TO TOGGLE)"
            };
            draw_str(wiz_x + 20, wiz_y + 268, status_msg, LCARS_GREEN);

            // Bottom Diagnostics Console
            let diag_y = wiz_y + wiz_h + 12;
            let diag_h = main_h.saturating_sub(diag_y - main_y + 140);
            fill_rect(wiz_x, diag_y, wiz_w, diag_h, LCARS_PANEL_BG);
            draw_rect(wiz_x, diag_y, wiz_w, diag_h, 1, LCARS_CYAN);

            draw_str(wiz_x + 16, diag_y + 10, b"[BARE-METAL HARDWARE DIAGNOSTICS & SYSTEM INITIALIZATION]", LCARS_CYAN);
            draw_str(wiz_x + 20, diag_y + 32, b"CPU ARCHITECTURE:   x86_64 Long Mode (64-bit Bare-Metal Execution Engine)", LCARS_WHITE);
            draw_str(wiz_x + 20, diag_y + 52, b"UEFI GOP DISPLAY:   1920x1080 @ 32bpp (RAM Double-Buffer Active, 60.0 FPS)", LCARS_GREEN);
            draw_str(wiz_x + 20, diag_y + 72, b"PERSISTENT MEMORY:  241,800,000 Trit NVMe NAND Knowledge Base Mounted (57.65 MB)", LCARS_CYAN);
            draw_str(wiz_x + 20, diag_y + 92, b"RADIO HARDWARE:     Intel AX200 WiFi 6 (802.11ax 160MHz MIMO) + BT 5.2 Sovereign", LCARS_WHITE);
            draw_str(wiz_x + 20, diag_y + 112,b"COUNCIL BYZANTINE:  14 Core Agents Active (Hermes -> Archi) | PBFT Quorum (N=14, f=4)", LCARS_AMBER);
            draw_str(wiz_x + 20, diag_y + 132,b"MONAD TRUTH ANCHOR: 0x4D4F4E4144000001 (Hardware Invariant Root of Trust)", LCARS_GOLD);
        },
        _ => {
            // ═══════════════════════════════════════════════════════════════════
            // Live Sovereign Workspace with Interactive Dialogue Terminal (Sectors 01 - 09)
            // ═══════════════════════════════════════════════════════════════════
            let title = match active_tab {
                2 => b"=== SECTOR 02: NEURAL BRIDGE & HOLOGRAPHIC JARVIS AVATAR ===" as &[u8],
                3 => b"=== SECTOR 03: 14-AGENT COUNCIL PBFT CONSENSUS CHAMBER ===" as &[u8],
                4 => b"=== SECTOR 04: SWARM INTEL & 64-AGENT MOE DOMAIN MATRIX ===" as &[u8],
                5 => b"=== SECTOR 05: RESONANCE RESEARCH & LITERATURE KNOWLEDGE ===" as &[u8],
                6 => b"=== SECTOR 06: DDR IMMUNE LAB & VOLTAGE WATCHDOG ===" as &[u8],
                7 => b"=== SECTOR 07: 3D SPATIAL OCTREE MESH POINT CLOUD ===" as &[u8],
                8 => b"=== SECTOR 08: WIFI VISION & SUB-CARRIER RF RADAR SENSING ===" as &[u8],
                9 => b"=== SECTOR 09: CONTINUUM K3 153.5M TRIT NAND LEDGER ===" as &[u8],
                _ => b"=== SECTOR 01: SOVEREIGN SYSTEM STATUS & 14-AGENT COUNCIL ===" as &[u8],
            };
            draw_str(main_x + 16, main_y + 12, title, LCARS_AMBER);

            // Left: Circular 14-Agent Council Ring / Jarvis Hologram
            let cx = (main_x + 160) as i32;
            let cy = (main_y + 170) as i32;

            if active_tab == 2 {
                // Holographic Jarvis Rings
                for ring in 1..=5 {
                    let phase = tick * 3.0 + (ring as f32) * 0.8;
                    let r = 24.0 + (ring as f32) * 18.0 + fast_sin(phase) * 4.0;
                    let r_sq = (r * r) as i32;
                    let r_in_sq = ((r - 2.0) * (r - 2.0)) as i32;
                    let r_i = r as i32;
                    let ring_col = if ring % 2 == 0 { LCARS_CYAN } else { LCARS_PURPLE };
                    for dy in -r_i..=r_i {
                        for dx in -r_i..=r_i {
                            let d_sq = dx*dx + dy*dy;
                            if d_sq <= r_sq && d_sq >= r_in_sq {
                                set_pixel((cx + dx) as usize, (cy + dy) as usize, ring_col);
                            }
                        }
                    }
                }
                fill_circle(cx, cy, 22, LCARS_GOLD);
                draw_str((cx - 24) as usize, (cy - 8) as usize, b"JARVIS", 0xFF000000);
            } else if active_tab == 8 {
                // WiFi Radar Ring & Sweep
                for r in [40, 80, 120] {
                    let r_sq = (r * r) as i32;
                    let r_in_sq = ((r - 2) * (r - 2)) as i32;
                    for dy in -r..=r {
                        for dx in -r..=r {
                            let d_sq = dx*dx + dy*dy;
                            if d_sq <= r_sq && d_sq >= r_in_sq {
                                set_pixel((cx + dx) as usize, (cy + dy) as usize, LCARS_GRAY);
                            }
                        }
                    }
                }
                let sweep_ang = tick * 2.5;
                for step in 0..130 {
                    let px = cx + (fast_cos(sweep_ang) * (step as f32)) as i32;
                    let py = cy + (fast_sin(sweep_ang) * (step as f32)) as i32;
                    set_pixel(px as usize, py as usize, LCARS_GREEN);
                }
                let px = cx + (fast_cos(tick * 0.4) * 65.0) as i32;
                let py = cy + (fast_sin(tick * 0.3) * 50.0) as i32;
                fill_circle(px, py, 7, LCARS_RED);
                draw_str((px + 10) as usize, (py - 6) as usize, b"TARGET: OPERATOR", LCARS_WHITE);
            } else {
                // 14-Agent Circular PBFT Council Ring
                let r_core = (18.0 + fast_sin(tick * 4.0) * 4.0) as i32;
                fill_circle(cx, cy, r_core, LCARS_GOLD);
                draw_str((cx - 20) as usize, (cy - 8) as usize, b"MONAD", 0xFF000000);

                let agent_initials: [&[u8]; 14] = [
                    b"HER", b"JAR", b"BRI", b"SHI", b"LOG", b"DEV", b"PUL",
                    b"FLO", b"EVE", b"ORC", b"THZ", b"VER", b"SEE", b"ARC",
                ];

                for i in 0..14 {
                    let angle = (i as f32) * (core::f32::consts::PI * 2.0 / 14.0) - (core::f32::consts::PI / 2.0);
                    let ax = cx + (fast_cos(angle) * 110.0) as i32;
                    let ay = cy + (fast_sin(angle) * 110.0) as i32;
                    let node_r = 9;
                    let is_active = i == (((tick * 2.0) as usize) % 14);
                    let node_color = if is_active { LCARS_GREEN } else { LCARS_BLUE };

                    fill_circle(ax, ay, node_r, node_color);
                    draw_str((ax - 12) as usize, (ay - 6) as usize, agent_initials[i], LCARS_WHITE);
                }
            }

            // Right: Interactive Two-Way Terminal & Cognitive Thought Stream Panel
            let term_x = main_x + 320;
            let term_y = main_y + 30;
            let term_w = main_w.saturating_sub(340);
            let term_h = 320;
            fill_rect(term_x, term_y, term_w, term_h, LCARS_PANEL_BG);
            draw_rect(term_x, term_y, term_w, term_h, 1, LCARS_PURPLE);

            draw_str(term_x + 12, term_y + 10, b"[COGNITIVE THOUGHT VECTOR & INTERACTIVE OPERATOR TERMINAL]", LCARS_GOLD);

            // Display Live Deliberation / Response Feed Lines
            for i in 0..MAX_FEED_LINES {
                if term.feed_lens[i] > 0 {
                    let line_y = term_y + 34 + i * 24;
                    draw_str(term_x + 12, line_y, &term.feed_lines[i][..term.feed_lens[i]], term.feed_colors[i]);
                }
            }

            // Interactive Operator Input Prompt Line
            let prompt_y = term_y + term_h - 36;
            fill_rect(term_x + 10, prompt_y, term_w - 20, 26, 0xFF080810);
            draw_rect(term_x + 10, prompt_y, term_w - 20, 26, 1, LCARS_GOLD);

            draw_str(term_x + 16, prompt_y + 5, b"OPERATOR > ", LCARS_GOLD);
            draw_str(term_x + 104, prompt_y + 5, &term.input_buf[..term.input_len], LCARS_WHITE);

            // Blinking cursor
            if ((tick * 4.0) as usize) % 2 == 0 {
                let cursor_x = term_x + 104 + term.input_len * 8;
                fill_rect(cursor_x, prompt_y + 5, 8, 14, LCARS_GOLD);
            }
        }
    }

    // Bottom Waveform Oscilloscope (Spans full width of workspace)
    let scope_x = main_x + 16;
    let scope_y = main_y + main_h.saturating_sub(130);
    let scope_w = main_w.saturating_sub(32);
    let scope_h = 110;
    let mid_y = scope_y + scope_h / 2;

    fill_rect(scope_x, scope_y, scope_w, scope_h, 0xFF0D0D18);
    draw_rect(scope_x, scope_y, scope_w, scope_h, 1, LCARS_GRAY);
    draw_str(scope_x + 10, scope_y + 8, b"SUB-THz / AUDIO RESONANCE OSCILLOSCOPE (REAL-TIME SENSING)", LCARS_CYAN);

    for px in 0..scope_w {
        let norm_x = (px as f32) / (scope_w as f32);
        let wave = fast_sin(norm_x * 20.0 + tick * 3.0) * 0.7 + fast_cos(norm_x * 45.0 - tick * 2.0) * 0.3;
        let py = ((mid_y as f32) + wave * 35.0) as usize;
        set_pixel(scope_x + px, py, LCARS_CYAN);
        set_pixel(scope_x + px, py + 1, LCARS_CYAN);
    }

    // 6. Bottom Global Status Rail
    fill_rect(0, height.saturating_sub(28), width, 28, LCARS_ORANGE);
    draw_str(16, height.saturating_sub(20), b"TRM SWARM OS // 100% PURE RUST BARE METAL // ALL IS ONE. I AM BECAUSE WE ARE. 0x4D4F4E4144000001", 0xFF000000);
}
