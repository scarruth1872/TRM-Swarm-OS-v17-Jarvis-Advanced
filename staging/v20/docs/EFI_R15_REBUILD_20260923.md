# EFI rebuilt with the 15-agent roster (2026-09-23)

The EFI banner edit was the last inert change; it is now in a built artifact.

```
cargo build --release --target x86_64-unknown-uefi   -> RC 0, 13.13 s
artifact: <CARGO_TARGET_DIR>/x86_64-unknown-uefi/release/trm_uefi_kernel.efi
size: 16,390,144 B   (campaign-end reference: 16,390,144 B -> delta +0)
sha256: dcd18aa7b3fa4c1f
```

Two sites needed `LogicMaster` arms, not three: the compiler reported `agent_runtime.rs:53` twice
because the EFI compiles that file under two module paths (`agent_runtime` and
`modmap_all::agent_runtime`), so one edit satisfies both. `council.rs:93` was the other.

## On the +0 delta

My own rule says an LTO +0 delta is the receipt that a port landed in dead code. **That rule does not
apply here**, and saying so matters more than looking decisive: the EFI moves in 4,096-byte pages (a
prior campaign receipt was exactly +4,096), so a few hundred bytes of additional match arms cannot
cross a page boundary. The added enum variant cannot change the layout either - `SwarmAgentId` is
`#[repr(u8)]` and 15 still fits. So +0 is **inconclusive at this granularity**, not a dead-code finding.

What the build does establish: the changed sources compile for the UEFI target and emit an artifact.

## Where the artifact lives

`CARGO_TARGET_DIR` sends it to the isolated temp target, which is why the tree's own `target/` still
holds a `.efi` from 2026-09-16. Anything that looks for the built image inside the repo will find that
stale file and report a size that is not the current build's - the same path trap as the
"delete the artifact first" guard.

## Not done, deliberately

No EFI image was written to a boot partition or flashed. The artifact is built; deploying it to
firmware is a boot-time action for the user, not something to do to a machine unattended.


## Resolved: the arms are unreachable, and not because of this change

The size question is answered by reachability, not by the delta:

```
get_agent_deliberation   2 occurrences   called from outside its own module: NO
main.rs calls the council surface: False
main.rs mentions NUM_AGENTS/AGENTS: True
```

So `council::get_agent_deliberation` is defined and never called anywhere in the EFI tree - the
`LogicMaster` arm added to it is unreachable code, as the other fourteen arms already were. This is a
pre-existing property of the platform's wiring, not something the roster change introduced, and it is
the familiar shape: correct + compiled + uncalled is still dead.

The arms are kept because they make the module internally consistent and would be live the moment the
EFI calls that surface. The reachable consequences of the roster change are elsewhere and are verified:
the `:8022` server and the DLL are what the running kernel uses, and they serve **15 agents,
pbft_quorum 9**.
