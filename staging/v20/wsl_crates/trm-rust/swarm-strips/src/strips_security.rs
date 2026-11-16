//! eBPFProbe — kernel-space dynamic tracer. Honest boundary: this host runs Windows,
//! which has no eBPF subsystem; the real kernel facts are served by the kernel's own host probe.

pub fn ebpf_probe(input: &str) -> String {
    let target = if input.is_empty() { "kernel" } else { input };
    let probeable = matches!(target, "kernel" | "msr" | "pstate" | "cpu" | "smbios");
    let facts = crate::json_escape(&format!(
        "Windows host: no eBPF; real facts served by /infra/health + host_probe (MSR P-state, SMBIOS, core-pinning). target={target}"
    ));
    format!(
        r#"{{"agent":"eBPFProbe","status":"ok","probeable":{probeable},"boundary":"no-eBPF-on-windows","facts":"{facts}","kind":"derived"}}"#
    )
}
