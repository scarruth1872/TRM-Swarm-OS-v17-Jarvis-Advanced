# Sandbox Execution Specification

## Runtime: gVisor (runsc) or Firecracker MicroVM

### Isolation Parameters
- **Filesystem**: Read-only root; tmpfs for `/tmp` and `/workspace`
- **Network**: Disabled by default; opt-in via explicit proxy (localhost-only)
- **Syscalls**: Seccomp allowlist (only `read`, `write`, `exit`, `futex`, etc.)
- **Resources**:
  - CPU: 1 core max per test process
  - Memory: 512 MB hard limit (OOM-kill on breach)
  - Disk: 100 MB tmpfs quota
  - Time: 30-second wall-clock timeout (SIGKILL after 5s grace)

### Implementation (Docker-compatible)
```yaml
version: "3.8"
services:
  test-runner:
    image: swarm-test-runner:hardened
    runtime: runsc
    read_only: true
    tmpfs:
      - /tmp:size=100m,noexec,nosuid
      - /workspace:size=100m,noexec,nosuid
    network_mode: "none"
    cap_drop:
      - ALL
    security_opt:
      - no-new-privileges:true
      - seccomp=./seccomp-profile.json
    pids_limit: 128
    mem_limit: 512m
    cpus: "1.0"
    environment:
      - NODE_ENV=test
      - SWARM_SANDBOXED=true
```

### Seccomp Profile (minimal)
```json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "architectures": ["SCMP_ARCH_X86_64"],
  "syscalls": [
    {"names": ["read", "write", "exit", "exit_group", "futex", "mmap", "munmap", "brk", "openat", "close", "stat", "fstat", "lseek", "getpid", "gettid", "clock_gettime"], "action": "SCMP_ACT_ALLOW"}
  ]
}
```