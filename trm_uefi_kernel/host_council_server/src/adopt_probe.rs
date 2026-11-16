//! Adoption probe (G194): the three modules the base called "rotten", tested on this machine.
//!
//! The base's copies fail to build because they were written against OLDER versions of
//! `sram_pattern_computer`, `microkernel_spawner` and `nand_fs`. Here those three dependencies still carry the
//! functions they name (`pattern_tcam_match`, `make_subagent_id`, `continuum_function_for`, 2-arg
//! `nand_status`), so the modules are not rotten - the base is stale. This probe proves that by running them.

fn s(v: &str) -> serde_json::Value { serde_json::json!(v) }

pub fn run() -> serde_json::Value {
    use trm_uefi_kernel::k3_ternary_sram::K3Sram;
    use trm_uefi_kernel::spawn_subagent::SpawnPlanner;

    let mut out = serde_json::Map::new();

    // ── K3 SRAM: store/lookup, CAM, ternary conjunction ──────────────────────────────────────────
    let mut sram = K3Sram::new();
    out.insert("sram.empty_stored".into(), serde_json::json!(sram.stored()));
    out.insert("sram.empty_utilization".into(), serde_json::json!(sram.utilization()));
    let idx = sram.store_entry(b"alpha", 1, 0.9);
    sram.store_entry(b"beta", -1, 0.4);
    sram.store_entry(b"gamma", 0, 0.6);
    out.insert("sram.store_index".into(), serde_json::json!(idx));
    out.insert("sram.stored".into(), serde_json::json!(sram.stored()));
    out.insert("sram.lookup_hit".into(), serde_json::json!(format!("{:?}", sram.lookup_entry(b"alpha"))));
    out.insert("sram.lookup_neg".into(), serde_json::json!(format!("{:?}", sram.lookup_entry(b"beta"))));
    out.insert("sram.lookup_miss".into(), serde_json::json!(format!("{:?}", sram.lookup_entry(b"nope"))));
    out.insert("sram.utilization".into(), serde_json::json!(sram.utilization()));

    // pattern storage + content-addressable lookup: a stored pattern must be findable, an unlike one not
    sram.store_pattern(b"pat-a", &[1, -1, 1, 0]);
    out.insert("sram.cam_hit".into(), serde_json::json!(sram.cam_lookup(b"pat-a")));
    out.insert("sram.cam_miss".into(), serde_json::json!(sram.cam_lookup(b"pat-z")));
    out.insert("sram.cam_hit_ratio".into(), serde_json::json!(sram.cam_hit_ratio()));
    // exact query matches; a query that differs only where the mask is 0 still matches
    out.insert("sram.tcam_exact".into(), serde_json::json!(sram.tcam_match(&[1, -1, 1, 0], &[1, 1, 1, 1])));
    out.insert("sram.tcam_masked".into(), serde_json::json!(sram.tcam_match(&[1, 0, 0, 0], &[1, 0, 0, 0])));
    out.insert("sram.tcam_reject".into(), serde_json::json!(sram.tcam_match(&[-1, 1, -1, 1], &[1, 1, 1, 1])));

    // Kleene conjunction is IN PLACE, so each case gets a fresh bank. The first version reused one SRAM
    // and the FALSE pass overwrote the plane before the UNKNOWN pass read it - the code was right and the
    // probe was measuring its own mutation. Full 3x3 truth table, one fresh bank per cell.
    for a in [-1i8, 0, 1] {
        for b in [-1i8, 0, 1] {
            let mut fresh = K3Sram::new();
            fresh.store_pattern(b"cell", &[a]);
            let r = fresh.k3_conjunction(0, 1, b);
            out.insert(format!("sram.k3_table.{}_and_{}", a, b),
                serde_json::json!(format!("{:?}", r)));
        }
    }
    // window behaviour on a fresh bank: [1,-1,0] conjoined with TRUE leaves it unchanged
    let mut win = K3Sram::new();
    win.store_pattern(b"win", &[1, -1, 0]);
    out.insert("sram.k3_window_true".into(), serde_json::json!(win.k3_conjunction(0, 3, 1)));
    out.insert("sram.bypass_after_pattern".into(), serde_json::json!(format!("{:?}", win.bypass_over_stored())));
    out.insert("sram.stored_after_pattern".into(), serde_json::json!(win.stored()));
    out.insert("sram.utilization_after_pattern".into(), serde_json::json!(win.utilization()));
    out.insert("sram.bypass_state".into(), serde_json::json!(format!("{:?}", sram.bypass_over_stored())));

    // ── spawn planner: deterministic plans, ids, lookup ─────────────────────────────────────────
    let mut planner = SpawnPlanner::new();
    out.insert("spawn.empty_len".into(), serde_json::json!(planner.len()));
    out.insert("spawn.empty_is_empty".into(), serde_json::json!(planner.is_empty()));
    let p1 = planner.plan_spawn("Devo", "devo-1", "analyze the ledger", None, 7);
    let p2 = planner.plan_spawn("Devo", "devo-1", "analyze the ledger", None, 7);
    let p3 = planner.plan_spawn("Shield", "shield-1", "audit security posture", Some("audit"), 9);
    out.insert("spawn.p1".into(), s(&p1.render()));
    out.insert("spawn.deterministic".into(), serde_json::json!(
        p1.subagent_id == p2.subagent_id && p1.render() == p2.render()));
    out.insert("spawn.distinct_ids".into(), serde_json::json!(p1.subagent_id != p3.subagent_id));
    out.insert("spawn.note".into(), s(p1.note()));
    out.insert("spawn.len".into(), serde_json::json!(planner.len()));
    out.insert("spawn.find_hit".into(), serde_json::json!(planner.find(&p1.subagent_id).is_some()));
    out.insert("spawn.find_miss".into(), serde_json::json!(planner.find("no-such-id").is_some()));
    let demo = trm_uefi_kernel::spawn_subagent::demo();
    out.insert("spawn.demo_lines".into(), serde_json::json!(demo.0.lines().count()));
    out.insert("spawn.demo_note".into(), s(demo.1));
    out.insert("spawn.demo_len".into(), serde_json::json!(demo.2));

    serde_json::Value::Object(out)
}
