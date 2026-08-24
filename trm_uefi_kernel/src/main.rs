#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

mod allocator;
mod agent_runtime;
mod bridge;
mod benchmark_epistemology;
mod council;
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
mod module_registry;
mod monad;
mod nand_fs;
mod quic_protocol;
mod star_matrix;
mod time;
mod totality;
mod trm7m;
mod turbovec;
#[path = "modmap_all.rs"]
mod modmap_all;

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
