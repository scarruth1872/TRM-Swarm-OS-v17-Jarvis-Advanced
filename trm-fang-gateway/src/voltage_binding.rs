// voltage_binding.rs — TRM Swarm OS v19.001 "Sovereign"
// Physical Hardware Voltage & Monad Anchor Binding

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;

pub struct VoltageBinding {
    pub monad_anchor: u64,
    pub v_core_mv: u32,
    pub v_soc_mv: u32,
    pub is_bound: bool,
}

impl VoltageBinding {
    pub fn new() -> Self {
        VoltageBinding {
            monad_anchor: MONAD_ANCHOR,
            v_core_mv: 1250,
            v_soc_mv: 1100,
            is_bound: true,
        }
    }

    pub fn verify_anchor(&self) -> bool {
        self.monad_anchor == MONAD_ANCHOR && self.is_bound
    }
}
