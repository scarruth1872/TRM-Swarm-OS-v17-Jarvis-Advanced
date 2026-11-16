//! real_act.rs — the REAL WriteFile act-arm: live fabric state → real file.
use swarm_agent::realfab::fabric_report;

fn main() {
    let out = "/home/shawn/trm-rust/fabric_report.txt";
    match fabric_report(out) {
        Ok(content) => {
            let len = content.len();
            println!("ACT: WriteFile → {out} ({len} bytes, REAL fabric data)");
            println!("--- content ---");
            println!("{content}");
        }
        Err(e) => println!("ACT FAILED: {e}"),
    }
}
