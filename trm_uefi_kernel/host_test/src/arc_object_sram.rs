// ARC: segment -> K3 sign -> multi-entry SRAM store -> bit-sliced associative match
//
// Measures, honestly and per task:
//   * how many test objects find an identical K3 signature among THAT task's
//     train objects (no cross-task leakage: a fresh SRAM per task),
//   * the CAM utilization actually used,
//   * the CAM hit ratio,
//   * mean associative lookup cost in nanoseconds,
//   * equivalence between the bit-sliced whole-CAM path and the scalar
//     line-scan reference path (0 disagreements required).
#[path = "../../host_council_server/src/k3_logic.rs"]
mod k3_logic;
#[path = "../../host_council_server/src/sram_pattern_computer.rs"]
mod sram_pattern_computer;
#[path = "../../host_council_server/src/k3_ternary_sram.rs"]
mod k3_ternary_sram;

use k3_ternary_sram::{K3Sram, CAM_LINES};

type Grid = Vec<Vec<u8>>;

struct Task {
    id: String,
    train: Vec<(Grid, Grid)>,
    test: Vec<Grid>,
    test_out: Vec<Grid>,
}

fn parse(path: &str) -> Vec<Task> {
    let txt = std::fs::read_to_string(path).unwrap();
    let mut it = txt.lines();
    let mut tasks: Vec<Task> = Vec::new();
    while let Some(raw) = it.next() {
        let l = raw.trim();
        if l.is_empty() {
            continue;
        }
        if let Some(rest) = l.strip_prefix("TASK ") {
            tasks.push(Task {
                id: rest.to_string(),
                train: Vec::new(),
                test: Vec::new(),
                test_out: Vec::new(),
            });
            continue;
        }
        let parts: Vec<&str> = l.split_whitespace().collect();
        let h: usize = parts[1].parse().unwrap();
        let mut g: Grid = Vec::new();
        for _ in 0..h {
            let row = it.next().unwrap().trim();
            g.push(row.bytes().map(|b| b - b'0').collect());
        }
        let t = tasks.last_mut().unwrap();
        match parts[0] {
            "IN" => t.train.push((g, Vec::new())),
            "OUT" => {
                if let Some(last) = t.train.last_mut() {
                    last.1 = g;
                }
            }
            "TEST" => t.test.push(g),

            "TOUT" => t.test_out.push(g),

            _ => {}
        }
    }
    tasks
}

/// 4-connectivity, same-colour, non-background object segmentation.
fn segment(g: &Grid) -> Vec<Vec<(usize, usize)>> {
    let h = g.len();
    let w = g[0].len();
    let mut seen = vec![vec![false; w]; h];
    let mut out = Vec::new();
    for r in 0..h {
        for c in 0..w {
            if seen[r][c] || g[r][c] == 0 {
                continue;
            }
            let col = g[r][c];
            let mut stack = vec![(r, c)];
            let mut cells = Vec::new();
            seen[r][c] = true;
            while let Some((y, x)) = stack.pop() {
                cells.push((y, x));
                for (dy, dx) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                    let ny = y as i32 + dy;
                    let nx = x as i32 + dx;
                    if ny < 0 || nx < 0 || ny >= h as i32 || nx >= w as i32 {
                        continue;
                    }
                    let (ny, nx) = (ny as usize, nx as usize);
                    if !seen[ny][nx] && g[ny][nx] == col {
                        seen[ny][nx] = true;
                        stack.push((ny, nx));
                    }
                }
            }
            out.push(cells);
        }
    }
    out
}

const NSIG: usize = 19;

/// 19 K3 trits per object:
///   0-9  colour one-hot (+1 the object's colour, -1 others)
///   10-14 size bucket 1 / 2 / 3 / 4-6 / 7+
///   15   square      +1/-1
///   16   touches edge +1/-1
///   17   fills its bbox +1/-1
///   18   single cell   +1/-1
fn sign(g: &Grid, cells: &[(usize, usize)]) -> Vec<i8> {
    let mut v = vec![-1i8; NSIG];
    let col = g[cells[0].0][cells[0].1] as usize;
    v[col.min(9)] = 1;
    let n = cells.len();
    let b = match n {
        1 => 0,
        2 => 1,
        3 => 2,
        4..=6 => 3,
        _ => 4,
    };
    v[10 + b] = 1;
    let (mut y0, mut y1, mut x0, mut x1) = (usize::MAX, 0usize, usize::MAX, 0usize);
    for &(y, x) in cells {
        y0 = y0.min(y);
        y1 = y1.max(y);
        x0 = x0.min(x);
        x1 = x1.max(x);
    }
    let sq = (y1 - y0) == (x1 - x0);
    v[15] = if sq { 1 } else { -1 };
    let edge = y0 == 0 || x0 == 0 || y1 == g.len() - 1 || x1 == g[0].len() - 1;
    v[16] = if edge { 1 } else { -1 };
    let fill = n == (y1 - y0 + 1) * (x1 - x0 + 1);
    v[17] = if fill { 1 } else { -1 };
    v[18] = if n == 1 { 1 } else { -1 };
    v
}

/// Signature -> CAM tag bytes (ternary {-1,0,+1} mapped to {0,1,2}).
fn sig_key(s: &[i8]) -> Vec<u8> {
    s.iter().map(|&t| (t + 1) as u8).collect()
}

fn main() {
    let path = if std::path::Path::new("arc_tasks.txt").exists() {
        "arc_tasks.txt"
    } else {
        "../../arc_tasks.txt"
    };
    let tasks = parse(path);

    let mut train_obj = 0usize;
    let mut test_obj = 0usize;
    let mut matched = 0usize;
    let mut tasks_match = 0usize;
    let mut tasks_total = 0usize;

    let mut eq_fail = 0u64;
    let mut lookup_count = 0u64;
    let mut lookup_ns_total = 0u128;
    let mut scalar_ns_total = 0u128;
    let mut cam_util_sum = 0f64;
    let mut hit_ratio_sum = 0f64;
    let mut multi_entry_tasks = 0usize;
    let mut max_resident = 0usize;

    for t in &tasks {
        tasks_total += 1;
        let mut sram = K3Sram::new();
        let mut train_sigs: Vec<Vec<i8>> = Vec::new();

        // ---- STORE: every train object of THIS task into the multi-entry CAM
        for (gin, _gout) in &t.train {
            for cells in segment(gin) {
                train_obj += 1;
                let s = sign(gin, &cells);
                let key = sig_key(&s);
                sram.store_pattern_multi(&key, &s);
                train_sigs.push(s);
            }
        }

        let resident = sram.patterns_resident();
        max_resident = max_resident.max(resident);
        if resident > 1 {
            multi_entry_tasks += 1;
        }

        // ---- MATCH: every test object, associative lookup over all lines
        let mut task_hit = false;
        for gtest in &t.test {
            for cells in segment(gtest) {
                test_obj += 1;
                let q = sign(gtest, &cells);
                let mask = vec![1u8; q.len()];

                let t0 = std::time::Instant::now();
                let hit = sram.tcam_match_any(&q, &mask);
                lookup_ns_total += t0.elapsed().as_nanos();
                lookup_count += 1;

                // Reference path equivalence (same CAM, same query/mask).
                let ts = std::time::Instant::now();
                let reference = sram.tcam_match_any_scalar(&q, &mask);
                scalar_ns_total += ts.elapsed().as_nanos();
                if hit.is_some() != reference.is_some() {
                    eq_fail += 1;
                }

                if hit.is_some() {
                    matched += 1;
                    task_hit = true;
                }
                let _ = &train_sigs;
            }
        }
        if task_hit {
            tasks_match += 1;
        }
        cam_util_sum += sram.cam_utilization() as f64;
        hit_ratio_sum += sram.cam_hit_ratio() as f64;

        // Bit-slice plane must agree with the row-major plane.
        let mut rebuilt = K3Sram::new();
        rebuilt.planes = sram.planes.clone();
        rebuilt.lens = sram.lens;
        rebuilt.entries = sram.entries;
        rebuilt.rebuild_bits();
        if rebuilt.cam_bits != sram.cam_bits {
            eq_fail += 1;
        }
        // Tag index integrity: one chain node per valid entry.
        let valid = sram.entries.iter().filter(|e| e.valid).count();
        if sram.idx_len() != valid {
            eq_fail += 1;
        }
    }

    let coverage = if test_obj == 0 {
        0.0
    } else {
        matched as f64 / test_obj as f64
    };
    let mean_lookup_ns = if lookup_count == 0 {
        0.0
    } else {
        lookup_ns_total as f64 / lookup_count as f64
    };

    println!("ARC OBJECT SIGNATURE SRAM (multi-entry + bit-sliced associative lookup)");
    println!("  tasks                 {}", tasks_total);
    println!("  train objects         {}", train_obj);
    println!("  test objects          {}", test_obj);
    println!("  matched objects       {}", matched);
    println!("  coverage              {:.4}   (test objects with a train signature)", coverage);
    println!("  tasks w/ a match      {}/{}", tasks_match, tasks_total);
    println!("  multi-entry tasks     {}", multi_entry_tasks);
    println!("  max resident lines    {}", max_resident);
    println!("  cam utilization       {:.4}   (mean per task)", cam_util_sum / tasks_total as f64);
    println!("  cam hit ratio         {:.4}   (mean per task)", hit_ratio_sum / tasks_total as f64);
    println!("  lookup mean           {:.1} ns   ({} lookups)", mean_lookup_ns, lookup_count);
    let scalar_ns = if lookup_count == 0 { 0.0 } else { scalar_ns_total as f64 / lookup_count as f64 };
    let speedup = if mean_lookup_ns > 0.0 { scalar_ns / mean_lookup_ns } else { 0.0 };
    println!("  scalar scan mean      {:.1} ns", scalar_ns);
    println!("  bit-slice speedup     {:.2}x", speedup);
    println!("  bitslice vs scalar    {} disagreements", eq_fail);

    // ---- colour-blind coverage: why the earlier 83% claim was inflated -----
    // Wildcard trits 0..9 (colour one-hot). Matching then only checks size
    // bucket / square / edge / fill / single-cell, i.e. object SHAPE CLASS.
    let mut cb_matched = 0usize;
    let mut cb_total = 0usize;
    for t in &tasks {
        let mut sram = K3Sram::new();
        for (gin, _) in &t.train {
            for cells in segment(gin) {
                let s = sign(gin, &cells);
                sram.store_pattern_multi(&sig_key(&s), &s);
            }
        }
        for gtest in &t.test {
            for cells in segment(gtest) {
                cb_total += 1;
                let q = sign(gtest, &cells);
                let mut mask = vec![1u8; q.len()];
                for m in mask.iter_mut().take(10) {
                    *m = 0;
                }
                if sram.tcam_match_any(&q, &mask).is_some() {
                    cb_matched += 1;
                }
            }
        }
    }
    println!(
        "  colour-blind cover    {:.4}   (mask wildcards 0..9; SHAPE ONLY)",
        cb_matched as f64 / cb_total.max(1) as f64
    );

    // ---- saturation: full 256-line CAM, bit-slice vs scalar ----------------
    let mut sat = K3Sram::new();
    for i in 0..CAM_LINES {
        let mut pat = vec![0i8; NSIG];
        for (j, p) in pat.iter_mut().enumerate() {
            *p = (((i * 7 + j * 13) % 3) as i8) - 1;
        }
        // Unique key per line so all CAM_LINES become resident.
        let mut key = [0u8; 16];
        key[..8].copy_from_slice(&(i as u64).to_le_bytes());
        sat.store_pattern_multi(&key, &pat);
    }
    let resident = sat.patterns_resident();
    let probe_q = vec![1i8; NSIG];
    let probe_m = vec![1u8; NSIG];
    let reps = 20000usize;

    let t0 = std::time::Instant::now();
    let mut hits_bit = 0usize;
    for _ in 0..reps {
        if sat.tcam_match_any(&probe_q, &probe_m).is_some() {
            hits_bit += 1;
        }
    }
    let bit_ns = t0.elapsed().as_nanos() as f64 / reps as f64;

    let t1 = std::time::Instant::now();
    let mut hits_sc = 0usize;
    for _ in 0..reps {
        if sat.tcam_match_any_scalar(&probe_q, &probe_m).is_some() {
            hits_sc += 1;
        }
    }
    let sc_ns = t1.elapsed().as_nanos() as f64 / reps as f64;

    println!("  --- saturation ({} resident lines) ---", resident);
    println!("  bit-slice             {:.1} ns/query  ({} hits)", bit_ns, hits_bit);
    println!("  scalar scan           {:.1} ns/query  ({} hits)", sc_ns, hits_sc);
    println!("  saturation speedup    {:.2}x", sc_ns / bit_ns.max(0.001));
    phase2(&tasks);
    sweep(&tasks);
    score(&tasks);
    score2(&tasks);
    score3(&tasks);

}

// Phase 2: is input-object signature -> output-object signature stable?
fn phase2(tasks: &[Task]) {
    let mut eligible = 0usize;
    let mut pairs = 0usize;
    let mut determined = 0usize;
    let mut ambiguous = 0usize;
    let mut test_obj = 0usize;
    let mut test_det = 0usize;
    let mut consistent_tasks = 0usize;
    for t in tasks {
        let mut map: Vec<(Vec<i8>, Vec<i8>)> = Vec::new();
        let mut ok = true;
        for (gin, gout) in &t.train {
            if gin.len() != gout.len() || gin[0].len() != gout[0].len() { ok = false; break; }
            let og = segment(gout);
            for cells in segment(gin) {
                let mut hit: Option<Vec<i8>> = None;
                for oc in &og {
                    if oc.len() == cells.len() && oc.iter().all(|c| cells.contains(c)) {
                        hit = Some(sign(gout, oc));
                        break;
                    }
                }
                if let Some(os) = hit {
                    map.push((sign(gin, &cells), os));
                    pairs += 1;
                }
            }
        }
        if !ok { continue; }
        eligible += 1;
        let mut task_ok = true;
        let mut det_keys: Vec<Vec<i8>> = Vec::new();
        for (k, v) in map.iter() {
            if det_keys.contains(k) { continue; }
            let uniq = map.iter().filter(|(k2, _)| k2 == k).all(|(_, v2)| v2 == v);
            if uniq { det_keys.push(k.clone()); determined += 1; }
            else { ambiguous += 1; task_ok = false; }
        }
        if task_ok { consistent_tasks += 1; }
        for gt in &t.test {
            for cells in segment(gt) {
                test_obj += 1;
                if det_keys.contains(&sign(gt, &cells)) { test_det += 1; }
            }
        }
    }
    println!("--- phase 2: object rule consistency ---");
    println!("  eligible tasks (same dims) {}", eligible);
    println!("  object pairs collected     {}", pairs);
    println!("  determined input sigs      {}", determined);
    println!("  ambiguous input sigs       {}", ambiguous);
    println!("  fully consistent tasks     {} / {}", consistent_tasks, eligible);
    println!("  test objs w/ determ. sig   {} / {} ({:.4})",
        test_det, test_obj, test_det as f64 / (test_obj.max(1) as f64));
}

fn mkey(sig: &[i8], mask: &[u8]) -> Vec<i8> {
    sig.iter().zip(mask.iter()).filter(|(_, m)| **m != 0).map(|(v, _)| *v).collect()
}

fn sweep(tasks: &[Task]) {
    let mut masks: Vec<(&str, Vec<u8>)> = Vec::new();
    masks.push(("exact (all 19)", vec![1u8; 19]));
    masks.push(("colour only", (0..19).map(|i| if i < 10 { 1 } else { 0 }).collect()));
    masks.push(("colour+topology", (0..19).map(|i| if i < 10 || i == 15 || i == 18 { 1 } else { 0 }).collect()));
    masks.push(("no colour", (0..19).map(|i| if i >= 10 { 1 } else { 0 }).collect()));
    masks.push(("colour+size", (0..19).map(|i| if i < 15 { 1 } else { 0 }).collect()));
    masks.push(("drop size+edge", (0..19).map(|i| if i < 10 || i == 15 || i == 17 || i == 18 { 1 } else { 0 }).collect()));
    masks.push(("topology only", (0..19).map(|i| if i >= 15 { 1 } else { 0 }).collect()));
    println!("{:<18} {:>9} {:>9} {:>6} {:>6} {:>9}", "mask", "consist", "cover", "det", "amb", "tasks");
    for (name, m) in masks.iter() {
        let (mut elig, mut det, mut amb, mut cons) = (0usize, 0usize, 0usize, 0usize);
        let (mut tdet, mut ttot) = (0usize, 0usize);
        for t in tasks {
            let mut map: Vec<(Vec<i8>, Vec<i8>)> = Vec::new();
            let mut ok = true;
            for (gin, gout) in &t.train {
                if gin.len() != gout.len() || gin[0].len() != gout[0].len() { ok = false; break; }
                let og = segment(gout);
                for cells in segment(gin) {
                    let mut hit: Option<Vec<i8>> = None;
                    for oc in &og {
                        if oc.len() == cells.len() && oc.iter().all(|c| cells.contains(c)) {
                            hit = Some(sign(gout, oc)); break;
                        }
                    }
                    if let Some(os) = hit {
                        map.push((mkey(&sign(gin, &cells), m), mkey(&os, m)));
                    }
                }
            }
            if !ok { continue; }
            elig += 1;
            let mut task_ok = true;
            let mut det_keys: Vec<Vec<i8>> = Vec::new();
            for (k, v) in map.iter() {
                let uniq = map.iter().filter(|(k2, _)| k2 == k).all(|(_, v2)| v2 == v);
                if uniq {
                    if !det_keys.contains(k) { det_keys.push(k.clone()); det += 1; }
                } else {
                    if !det_keys.contains(k) { amb += 1; }
                    task_ok = false;
                }
            }
            if task_ok { cons += 1; }
            for gt in &t.test {
                for cells in segment(gt) {
                    ttot += 1;
                    if det_keys.contains(&mkey(&sign(gt, &cells), m)) { tdet += 1; }
                }
            }
        }
        println!("{:<18} {:>9.3} {:>9.3} {:>6} {:>6} {:>5}/{}",
            name, cons as f64 / elig.max(1) as f64, tdet as f64 / ttot.max(1) as f64, det, amb, cons, elig);
    }
}

fn sig_colour(s: &[i8]) -> Option<u8> {
    (0..10).find(|&i| s[i] == 1).map(|i| i as u8)
}

fn score(tasks: &[Task]) {
    let (mut exact, mut cells_ok, mut cells_tot, mut eligible) = (0usize, 0usize, 0usize, 0usize);
    let (mut b_ok, mut b_tot, mut b_exact) = (0usize, 0usize, 0usize);
    for t in tasks {
        if t.test.is_empty() || t.test_out.is_empty() { continue; }
        let mut map: Vec<(Vec<i8>, Vec<i8>)> = Vec::new();
        let mut ok = true;
        for (gin, gout) in &t.train {
            if gin.len() != gout.len() || gin[0].len() != gout[0].len() { ok = false; break; }
            let og = segment(gout);
            for cells in segment(gin) {
                let mut hit: Option<Vec<i8>> = None;
                for oc in &og {
                    if oc.len() == cells.len() && oc.iter().all(|c| cells.contains(c)) {
                        hit = Some(sign(gout, oc));
                        break;
                    }
                }
                if let Some(os) = hit { map.push((sign(gin, &cells), os)); }
            }
        }
        if !ok { continue; }
        eligible += 1;
        let mut pred = t.test[0].clone();
        for cells in segment(&t.test[0]) {
            let k = sign(&t.test[0], &cells);
            let outs: Vec<&Vec<i8>> = map.iter().filter(|(k2, _)| k2 == &k).map(|(_, v)| v).collect();
            if outs.is_empty() { continue; }
            if !outs.iter().all(|x| *x == outs[0]) { continue; }
            if let Some(c) = sig_colour(outs[0]) {
                for &(y, x) in &cells { pred[y][x] = c; }
            }
        }
        let truth = &t.test_out[0];
        if pred.len() != truth.len() || pred[0].len() != truth[0].len() { continue; }
        for r in 0..truth.len() { for c in 0..truth[0].len() {
            cells_tot += 1;
            if pred[r][c] == truth[r][c] { cells_ok += 1; }
        }}
        for r in 0..truth.len() { for c in 0..truth[0].len() {
            b_tot += 1;
            if t.test[0][r][c] == truth[r][c] { b_ok += 1; }
        }}
        if t.test[0] == *truth { b_exact += 1; }
        if pred == *truth { exact += 1; }
    }
    println!("--- phase 3: scored vs real test outputs ---");
    println!("  scored tasks   {}", eligible);
    println!("  exact          {}", exact);
    println!("  exact rate     {:.4}", exact as f64 / eligible.max(1) as f64);
    println!("  cell accuracy  {:.4}", cells_ok as f64 / cells_tot.max(1) as f64);
    println!("  --- control: copy input unchanged ---");
    println!("  copy exact     {}", b_exact);
    println!("  copy cell acc  {:.4}", b_ok as f64 / b_tot.max(1) as f64);
    println!("  LIFT cell      {:+.4}", cells_ok as f64 / cells_tot.max(1) as f64 - b_ok as f64 / b_tot.max(1) as f64);
}

fn bbox(cells: &[(usize, usize)]) -> (usize, usize) {
    let y = cells.iter().map(|c| c.0).min().unwrap_or(0);
    let x = cells.iter().map(|c| c.1).min().unwrap_or(0);
    (y, x)
}

fn score2(tasks: &[Task]) {
    let (mut exact, mut cells_ok, mut cells_tot, mut elig) = (0usize, 0usize, 0usize, 0usize);
    let (mut c1ex, mut c1ok, mut c2ex, mut c2ok) = (0usize, 0usize, 0usize, 0usize);
    let (mut det_obj, mut tot_obj) = (0usize, 0usize);
    for t in tasks {
        if t.test.is_empty() || t.test_out.is_empty() { continue; }
        let mut map: Vec<(Vec<i8>, (Vec<i8>, i32, i32))> = Vec::new();
        let mut ok = true;
        for (gin, gout) in &t.train {
            if gin.len() != gout.len() || gin[0].len() != gout[0].len() { ok = false; break; }
            let og = segment(gout);
            for cells in segment(gin) {
                let (iy, ix) = bbox(&cells);
                for oc in &og {
                    if oc.len() == cells.len() && oc.iter().all(|c| cells.contains(c)) {
                        let (oy, ox) = bbox(oc);
                        let d = (sign(gout, oc), oy as i32 - iy as i32, ox as i32 - ix as i32);
                        map.push((sign(gin, &cells), d));
                        break;
                    }
                }
            }
        }
        if !ok { continue; }
        elig += 1;
        let h = t.test[0].len(); let w = t.test[0][0].len();
        let mut pred = t.test[0].clone();
        for cells in segment(&t.test[0]) {
            tot_obj += 1;
            let k = sign(&t.test[0], &cells);
            let vals: Vec<&(Vec<i8>, i32, i32)> = map.iter().filter(|(k2, _)| k2 == &k).map(|(_, v)| v).collect();
            if vals.is_empty() { continue; }
            if !vals.iter().all(|v| **v == *vals[0]) { continue; }
            det_obj += 1;
            let (osig, dy, dx) = vals[0];
            let c0 = t.test[0][cells[0].0][cells[0].1];
            let c = sig_colour(osig).unwrap_or(c0);
            for &(y, x) in &cells {
                let ny = y as i32 + dy; let nx = x as i32 + dx;
                if ny >= 0 && nx >= 0 && (ny as usize) < h && (nx as usize) < w {
                    pred[ny as usize][nx as usize] = c;
                }
            }
        }
        let truth = &t.test_out[0];
        if pred.len() != truth.len() || pred[0].len() != truth[0].len() { continue; }
        let c1 = &t.test[0];
        let c2 = &t.train.last().unwrap().1;
        let c2_ok = c2.len() == truth.len() && c2[0].len() == truth[0].len();
        for r in 0..truth.len() { for cc in 0..truth[0].len() {
            cells_tot += 1;
            if pred[r][cc] == truth[r][cc] { cells_ok += 1; }
            if c1[r][cc] == truth[r][cc] { c1ok += 1; }
            if c2_ok && c2[r][cc] == truth[r][cc] { c2ok += 1; }
        }}
        if pred == *truth { exact += 1; }
        if c1 == truth { c1ex += 1; }
        if c2_ok && c2 == truth { c2ex += 1; }
    }

    println!("--- phase 4: displacement rule + controls ---");
    println!("  scored tasks      {}", elig);
    println!("  objects determined {} / {}", det_obj, tot_obj);
    println!("  RULE   exact {}  cell {:.4}", exact, cells_ok as f64 / cells_tot.max(1) as f64);
    println!("  CTRL1  exact {}  cell {:.4}   (copy input)", c1ex, c1ok as f64 / cells_tot.max(1) as f64);
    println!("  CTRL2  exact {}  cell {:.4}   (last train output)", c2ex, c2ok as f64 / cells_tot.max(1) as f64);
}

fn sizesig(s: &[i8]) -> usize { (10..15).find(|&i| s[i] == 1).unwrap_or(0) }

fn pair_score(a: &[i8], b: &[i8]) -> i32 {
    let mut s = 0;
    if sig_colour(a) == sig_colour(b) { s += 4; }
    if sizesig(a) == sizesig(b) { s += 2; }
    if a[15] == b[15] { s += 1; }
    if a[17] == b[17] { s += 1; }
    s
}

fn pair_objs(gin: &Grid, gout: &Grid) -> Vec<(Vec<i8>, Vec<i8>, i32, i32)> {
    let ib = segment(gin);
    let ob = segment(gout);
    let ni = ib.len(); let no = ob.len();
    let mut used_i = vec![false; ni];
    let mut used_o = vec![false; no];
    let mut out = Vec::new();
    loop {
        let mut best: Option<(usize, usize, i32)> = None;
        for i in 0..ni {
            if used_i[i] { continue; }
            let si = sign(gin, &ib[i]);
            for j in 0..no {
                if used_o[j] { continue; }
                let so = sign(gout, &ob[j]);
                let sc = pair_score(&si, &so);
                if sc >= 4 && best.map_or(true, |(_, _, b)| sc > b) { best = Some((i, j, sc)); }
            }
        }
        match best {
            None => break,
            Some((i, j, _)) => {
                let (iy, ix) = bbox(&ib[i]);
                let (oy, ox) = bbox(&ob[j]);
                let dy = oy as i32 - iy as i32;
                let dx = ox as i32 - ix as i32;
                out.push((sign(gin, &ib[i]), sign(gout, &ob[j]), dy, dx));
                used_i[i] = true; used_o[j] = true;
            }
        }
    }
    out
}

fn score3(tasks: &[Task]) {
    let (mut exact, mut cells_ok, mut cells_tot, mut elig) = (0usize, 0usize, 0usize, 0usize);
    let (mut c1ex, mut c1ok, mut c2ex, mut c2ok) = (0usize, 0usize, 0usize, 0usize);
    let (mut det_obj, mut tot_obj, mut moved) = (0usize, 0usize, 0usize);
    for t in tasks {
        if t.test.is_empty() || t.test_out.is_empty() { continue; }
        let mut map: Vec<(Vec<i8>, (Vec<i8>, i32, i32))> = Vec::new();
        let mut ok = true;
        for (gin, gout) in &t.train {
            if gin.len() != gout.len() || gin[0].len() != gout[0].len() { ok = false; break; }
            for (si, so, dy, dx) in pair_objs(gin, gout) { map.push((si, (so, dy, dx))); }
        }
        if !ok { continue; }
        elig += 1;
        let h = t.test[0].len(); let w = t.test[0][0].len();
        let mut pred = t.test[0].clone();
        for cells in segment(&t.test[0]) {
            tot_obj += 1;
            let k = sign(&t.test[0], &cells);
            let vals: Vec<&(Vec<i8>, i32, i32)> = map.iter().filter(|(k2, _)| k2 == &k).map(|(_, v)| v).collect();
            if vals.is_empty() { continue; }
            if !vals.iter().all(|v| **v == *vals[0]) { continue; }
            det_obj += 1;
            let (osig, dy, dx) = vals[0];
            if *dy != 0 || *dx != 0 { moved += 1; }
            let c0 = t.test[0][cells[0].0][cells[0].1];
            let c = sig_colour(osig).unwrap_or(c0);
            for &(y, x) in &cells {
                let ny = y as i32 + dy; let nx = x as i32 + dx;
                if ny >= 0 && nx >= 0 && (ny as usize) < h && (nx as usize) < w {
                    pred[ny as usize][nx as usize] = c;
                }
            }
        }
        let truth = &t.test_out[0];
        if pred.len() != truth.len() || pred[0].len() != truth[0].len() { continue; }
        let c2 = t.train.last().map(|(_, o)| o.clone());
        let c1 = &t.test[0];
        let c2_ok = match &c2 {
            Some(g) => g.len() == truth.len() && g[0].len() == truth[0].len(),
            None => false,
        };
        for r in 0..truth.len() {
            for cc in 0..truth[0].len() {
                cells_tot += 1;
                if pred[r][cc] == truth[r][cc] { cells_ok += 1; }
                if c1[r][cc] == truth[r][cc] { c1ok += 1; }
                if c2_ok {
                    if let Some(g) = &c2 { if g[r][cc] == truth[r][cc] { c2ok += 1; } }
                }
            }
        }
        if pred == *truth { exact += 1; }
        if c1 == truth { c1ex += 1; }
        if c2_ok { if let Some(g) = &c2 { if g == truth { c2ex += 1; } } }
    }

    let cell = cells_ok as f64 / cells_tot.max(1) as f64;
    let c1cell = c1ok as f64 / cells_tot.max(1) as f64;
    let c2cell = c2ok as f64 / cells_tot.max(1) as f64;
    println!("--- phase 5: matching-based pairing (displacement enabled) ---");
    println!("  scored tasks       {}", elig);
    println!("  test objects       {}", tot_obj);
    println!("  determined objects {} / {}", det_obj, tot_obj);
    println!("  displaced objects  {}", moved);
    println!("  RULE   exact {}  cell {:.4}", exact, cell);
    println!("  CTRL1  exact {}  cell {:.4}   (copy input)", c1ex, c1cell);
    println!("  CTRL2  exact {}  cell {:.4}   (last train output)", c2ex, c2cell);
    println!("  LIFT cell vs CTRL1 {:+.4}", cell - c1cell);
}
