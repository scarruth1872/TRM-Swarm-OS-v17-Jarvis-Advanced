//! TRM Swarm OS v19.001 — CLI Argument Parser & Subcommand Dispatcher
//! Zero-dependency, high-performance command-line interface parser for trm_swarm_boot.exe.
//! Supports production bare-metal boot, 14-agent Council debate/voting, LCARS GUI rendering, Mesh Discovery, and verification.

use std::env;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Boot,
    BareMetal { fast: bool },
    Council { topic: String, category: String, proposer: String },
    CouncilBench { iterations: usize },
    Gui { frames: usize, headless: bool, tab: String },
    Serve,
    Mesh { broadcast: bool },
    Bench { tasks: usize },
    Spatial { points: usize },
    Verify,
    Help,
}

pub struct CLIConfig {
    pub command: Command,
}

impl CLIConfig {
    pub fn parse_args() -> Self {
        let args: Vec<String> = env::args().collect();
        if args.len() <= 1 {
            // Boot handoff default: PERSISTENT serve mode (no verify-and-exit).
            return CLIConfig { command: Command::Serve };
        }

        let cmd_str = args[1].to_lowercase();
        let command = match cmd_str.as_str() {
            "boot" => Command::Boot,
            "serve" | "resident" | "daemon" | "platform" => Command::Serve,
            "baremetal" | "bare-metal" | "production" => {
                let fast = args.iter().any(|a| {
                    let lower = a.to_lowercase();
                    lower == "--fast" || lower == "--production" || lower == "-f"
                });
                Command::BareMetal { fast }
            }
            "gui" | "desktop" | "lcars" | "compositor" | "jarvis" => {
                let mut frames = 60;
                let mut tab = "jarvis".to_string();
                let headless = args.iter().any(|a| a == "--headless" || a == "-h");
                for i in 2..args.len() {
                    if (args[i] == "--frames" || args[i] == "-n") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<usize>() {
                            frames = parsed;
                        }
                    } else if (args[i] == "--tab" || args[i] == "-t") && i + 1 < args.len() {
                        tab = args[i + 1].to_lowercase();
                    }
                }
                Command::Gui { frames, headless, tab }
            }
            "mesh" | "discovery" | "peers" => {
                let broadcast = !args.iter().any(|a| a == "--listen-only");
                Command::Mesh { broadcast }
            }
            "council" | "debate" | "vote" => {
                let mut topic = "Sovereign System State Mutation Protocol".to_string();
                let mut category = "general".to_string();
                let mut proposer = "Shawn (Operator)".to_string();

                let mut i = 2;
                while i < args.len() {
                    let arg = &args[i];
                    if (arg == "--topic" || arg == "-t") && i + 1 < args.len() {
                        topic = args[i + 1].clone();
                        i += 2;
                    } else if (arg == "--category" || arg == "-c") && i + 1 < args.len() {
                        category = args[i + 1].clone();
                        i += 2;
                    } else if (arg == "--proposer" || arg == "-p") && i + 1 < args.len() {
                        proposer = args[i + 1].clone();
                        i += 2;
                    } else if arg == "--bench" || arg == "bench" {
                        let iterations = if i + 1 < args.len() {
                            args[i + 1].parse::<usize>().unwrap_or(10_000)
                        } else {
                            10_000
                        };
                        return CLIConfig { command: Command::CouncilBench { iterations } };
                    } else {
                        if !arg.starts_with('-') && topic == "Sovereign System State Mutation Protocol" {
                            topic = arg.clone();
                        }
                        i += 1;
                    }
                }
                Command::Council { topic, category, proposer }
            }
            "bench" | "benchmark" => {
                let mut tasks = 1_000_000;
                for i in 2..args.len() {
                    if (args[i] == "--tasks" || args[i] == "-n") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<usize>() {
                            tasks = parsed;
                        }
                    }
                }
                Command::Bench { tasks }
            }
            "spatial" | "octree" => {
                let mut points = 100_000;
                for i in 2..args.len() {
                    if (args[i] == "--points" || args[i] == "-p") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<usize>() {
                            points = parsed;
                        }
                    }
                }
                Command::Spatial { points }
            }
            "verify" | "anchor" => Command::Verify,
            "help" | "--help" | "-h" => Command::Help,
            _ => Command::Boot,
        };

        CLIConfig { command }
    }

    pub fn print_help() {
        println!("===========================================================================");
        println!("  TRM SWARM OS v19.001 \"Sovereign\" — CLI TOOL & KERNEL BOOT");
        println!("===========================================================================");
        println!("Usage: trm_swarm_boot [SUBCOMMAND] [OPTIONS]\n");
        println!("SUBCOMMANDS:");
        println!("  boot                           Default cold-boot sequence & 1M diagnostic sweep");
        println!("  baremetal [--fast]             Production bare-metal boot (fast 14-agent health check)");
        println!("  gui [--frames N] [--headless]  Execute native Rust LCARS Framebuffer Compositor (120+ FPS)");
        println!("  mesh [--listen-only]           Broadcast & probe sovereign UDP peer mesh discovery");
        println!("  council [--topic T] [-c CAT]   Execute pure Rust 14-agent Council deliberation & K3 vote");
        println!("  council --bench [N]            Run high-throughput Council consensus benchmark");
        println!("  bench [--tasks N]              Execute Rayon multi-threaded task benchmark (default N=1,000,000)");
        println!("  spatial [--points N]           Run 3D Octree spatial indexing benchmark (default N=100,000)");
        println!("  verify                         Verify Monad Truth Anchor (0x4D4F4E4144000001)");
        println!("  help                           Print this operational CLI guide\n");
        println!("OPTIONS:");
        println!("  --topic, -t <STRING>           Proposal topic for Council debate");
        println!("  --category, -c <CAT>           Category: security, architecture, code, data, comms, health, infra, protocol");
        println!("  --proposer, -p <STRING>        Entity initiating the proposal");
        println!("  --frames, -n <NUM>             Number of frames to render in GUI test (default 60)");
        println!("  --headless, -h                 Run GUI compositor in headless test mode");
        println!("  --fast, -f                     Skip diagnostic sweep (production mode, ~1ms boot)");
        println!("  --production                   Alias for --fast\n");
        println!("EXAMPLES:");
        println!("  trm_swarm_boot gui --frames 120");
        println!("  trm_swarm_boot mesh");
        println!("  trm_swarm_boot council --topic \"Enable Zero-RTT WiFi Mesh\" -c infra");
        println!("  trm_swarm_boot baremetal --fast");
        println!("===========================================================================");
    }
}
