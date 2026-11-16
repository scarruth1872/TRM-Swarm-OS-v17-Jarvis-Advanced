//! TRM Swarm OS — Performance & Hardware Acceleration Campaign (Papers 128-137)
//!
//! Benchmark suite covering Tasks 81-90.
//!
//! HONESTY CONTRACT (see MEASUREMENT_INTEGRITY_SWEEP.md):
//! Every number here carries a `MeasurementBasis`. A figure derived from a timer that
//! actually elapsed on this host is `Measured`; a figure derived from a hardware
//! specification or a scenario constant is `Specified`/`Projected`. Only `Measured`
//! figures count toward `tasks_passed` or K3 consensus.
//!
//! What this module used to do, and no longer does:
//!   - Task 86 reported a *hardcoded* 42.5 / 54.8 MOps pair as a "NUMA throughput boost".
//!   - Task 88 reported a *hardcoded* 6.45 / 13.21 TFLOPS pair as "2x GPU throughput".
//!   - Task 89 reported a *hardcoded* 230 W / 168 W pair as "26.4% energy saved".
//!   - Task 87 derived ">= 10 Gbps network throughput" from the time taken to run
//!     `ring_head = (ring_head + 1) & 0xFFFF` a million times. No socket was opened.
//!   - Task 90 asserted success by counting its own results vector.
//!   - `k3_consensus` was the literal `K3Trit::True`.
//! Projected rows now say PROJECTED, never pass, and K3 consensus is derived from
//! evidence. No divisor is floored to manufacture a ratio.

use std::time::Instant;
use rayon::prelude::*;
use crate::MONAD_TRUTH_ANCHOR;
use crate::unified_resonance_stream::K3Trit;

/// How a reported number was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementBasis {
    /// Derived from a timer that actually elapsed on this host.
    Measured,
    /// Taken from a vendor/hardware specification — not a run on this host.
    Specified,
    /// Derived from constants chosen to represent a scenario. Not a measurement.
    Projected,
}

impl MeasurementBasis {
    pub fn tag(&self) -> &'static str {
        match self {
            MeasurementBasis::Measured => "MEASURED",
            MeasurementBasis::Specified => "SPECIFIED",
            MeasurementBasis::Projected => "PROJECTED",
        }
    }

    /// Only `Measured` figures may be counted as demonstrated capability.
    pub fn is_evidence(&self) -> bool {
        *self == MeasurementBasis::Measured
    }
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: usize,
    pub task_code: &'static str,
    pub title: &'static str,
    pub measured_metric: &'static str,
    pub numerical_value: f64,
    pub target_threshold: &'static str,
    pub passed: bool,
    pub basis: MeasurementBasis,
    pub latency_us: f64,
}

#[derive(Debug, Clone)]
pub struct SupremacyCampaignReport {
    pub total_tasks_executed: usize,
    pub tasks_passed: usize,
    pub tasks_measured: usize,
    pub tasks_projected: usize,
    pub overall_execution_time_ms: f64,
    pub results: Vec<TaskResult>,
    pub k3_consensus: K3Trit,
    pub monad_anchor: u64,
}

/// Ratio of a baseline duration to a measured duration.
///
/// Returns `None` when the measured duration did not resolve (0 s), so the caller
/// reports "unresolved" instead of dividing by a floored epsilon. Flooring the
/// divisor is how a 0 s measurement becomes a 25,000,000x headline.
fn speedup(baseline_s: f64, measured_s: f64) -> Option<f64> {
    if measured_s > 0.0 && baseline_s > 0.0 {
        Some(baseline_s / measured_s)
    } else {
        None
    }
}

/// Physical ceiling for a simple integer op rate. A core sustains only a few ops
/// per cycle, so a higher figure is an artifact (LLVM folding the loop into a
/// closed form), not a measurement.
fn plausible_op_ceiling(cores: usize) -> f64 {
    const MAX_OPS_PER_CYCLE_PER_CORE: f64 = 16.0;
    const ASSUMED_CLOCK_HZ: f64 = 3_300_000_000.0;
    (cores.max(1) as f64) * ASSUMED_CLOCK_HZ * MAX_OPS_PER_CYCLE_PER_CORE
}

/// `ops/elapsed` only when the timer resolved AND the rate is physically
/// reachable on this host; otherwise the measurement did not resolve (NaN).
fn honest_rate(total_ops: f64, elapsed_s: f64, cores: usize) -> f64 {
    if elapsed_s <= 0.0 {
        return f64::NAN;
    }
    let rate = total_ops / elapsed_s;
    if rate > plausible_op_ceiling(cores) {
        f64::NAN
    } else {
        rate
    }
}

pub struct PerformanceSupremacyCampaign;

impl PerformanceSupremacyCampaign {
    pub fn execute_all_tasks_81_to_90() -> SupremacyCampaignReport {
        let t0 = Instant::now();
        let mut results = Vec::with_capacity(10);
        // Real thread count on this host, used to bound derived op rates.
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        // -------------------------------------------------------------
        // Task 81: Real kernel benchmark — a timer around real work
        // -------------------------------------------------------------
        let t_81 = Instant::now();
        let iters_81 = 10_000_000u64;
        let mut accum_81 = 0u64;
        for i in 0..iters_81 {
            // black_box stops LLVM replacing this loop with a closed form.
            accum_81 = std::hint::black_box(
                accum_81.wrapping_add(std::hint::black_box(i) ^ MONAD_TRUTH_ANCHOR),
            );
        }
        let _ = std::hint::black_box(accum_81);
        let elapsed_81 = t_81.elapsed().as_secs_f64();
        let tasks_per_sec_81 = honest_rate(iters_81 as f64, elapsed_81, cores);
        results.push(TaskResult {
            task_id: 81,
            task_code: "Task 81 🔴",
            title: "Real Kernel Benchmark (Unsimulated)",
            measured_metric: "Actual Kernel Ops/sec",
            numerical_value: tasks_per_sec_81,
            target_threshold: "> 100M ops/sec",
            passed: tasks_per_sec_81 > 100_000_000.0,
            basis: MeasurementBasis::Measured,
            latency_us: elapsed_81 * 1e6,
        });

        // -------------------------------------------------------------
        // Task 82: Rayon parallel dispatch over 100 work items
        // (was labelled "100 nodes" — these are work items on this host's
        //  cores, not remote nodes; no fabric traffic is involved)
        // -------------------------------------------------------------
        let t_82 = Instant::now();
        let batch_per_item = 5_000_000u64;
        let total_ops_82 = 100 * batch_per_item;
        let _: u64 = (0..100u64).into_par_iter().map(|item_id| {
            let mut local_sum = 0u64;
            let seed = item_id ^ MONAD_TRUTH_ANCHOR;
            for k in 0..batch_per_item {
                // black_box prevents the arithmetic-series closed form.
                local_sum = std::hint::black_box(
                    local_sum.wrapping_add(std::hint::black_box(k).wrapping_mul(seed)),
                );
            }
            local_sum
        }).reduce(|| 0u64, |a, b| a.wrapping_add(b));
        let elapsed_82 = t_82.elapsed().as_secs_f64();
        let throughput_82 = honest_rate(total_ops_82 as f64, elapsed_82, cores);
        results.push(TaskResult {
            task_id: 82,
            task_code: "Task 82 🟡",
            title: "Rayon Parallel Dispatch (100 work items)",
            measured_metric: "Sustained Parallel Ops/sec",
            numerical_value: throughput_82,
            target_threshold: ">= 1,000,000,000 ops/sec",
            passed: throughput_82 >= 1_000_000_000.0,
            basis: MeasurementBasis::Measured,
            latency_us: elapsed_82 * 1e6,
        });

        // -------------------------------------------------------------
        // Task 83: serial vs thread-parallel docking kernel
        // NOTE: both arms run the SAME arithmetic on this CPU. The parallel arm is
        // Rayon over CPU cores — there is no GPU dispatch anywhere in this file.
        // -------------------------------------------------------------
        let ligands = 30_000;
        let conformations_per_ligand = 32;
        let t_83_cpu = Instant::now();
        let mut cpu_energy_sum = 0.0f64;
        for i in 0..ligands {
            for c in 0..conformations_per_ligand {
                let dist = (((i * 17 + c * 31) % 100) as f64) * 0.08 + 0.1;
                let r6 = dist.powi(6);
                let vdw = (1.0 / (r6 * r6 + 0.001)) - (2.0 / (r6 + 0.001));
                let coulomb = -1.5 / (dist + 0.05);
                cpu_energy_sum += std::hint::black_box(vdw + coulomb);
            }
        }
        let cpu_elapsed = t_83_cpu.elapsed().as_secs_f64();

        let t_83_parallel = Instant::now();
        let mut parallel_results = vec![0.0f64; ligands];
        parallel_results.par_chunks_mut(64).enumerate().for_each(|(chunk_idx, chunk)| {
            for k in 0..chunk.len() {
                let ligand_id = chunk_idx * 64 + k;
                let mut sum = 0.0;
                for c in 0..conformations_per_ligand {
                    let dist = (((ligand_id * 17 + c * 31) % 100) as f64) * 0.08 + 0.1;
                    let r6 = dist.powi(6);
                    let vdw = (1.0 / (r6 * r6 + 0.001)) - (2.0 / (r6 + 0.001));
                    let coulomb = -1.5 / (dist + 0.05);
                    sum += vdw + coulomb;
                }
                chunk[k] = std::hint::black_box(sum);
            }
        });
        let parallel_elapsed = t_83_parallel.elapsed().as_secs_f64();
        let speedup_83 = speedup(cpu_elapsed, parallel_elapsed);
        results.push(TaskResult {
            task_id: 83,
            task_code: "Task 83 🟡",
            title: "Parallel Docking Kernel (Rayon vs serial)",
            measured_metric: "Parallel vs Serial Wall-clock Ratio",
            numerical_value: speedup_83.unwrap_or(f64::NAN),
            target_threshold: ">= 10.0x",
            passed: speedup_83.map(|s| s >= 10.0).unwrap_or(false),
            basis: MeasurementBasis::Measured,
            latency_us: parallel_elapsed * 1e6,
        });
        let _ = cpu_energy_sum; // consumed by the serial arm's side effect

        // -------------------------------------------------------------
        // Task 84: scalar vs Rayon chunked ternary K3 combine
        // NOTE: this is thread-level parallelism, NOT AVX/SIMD intrinsics.
        // -------------------------------------------------------------
        let t_84_scalar = Instant::now();
        let num_trits = 2_000_000;
        let mut trits_a: Vec<i8> = (0..num_trits).map(|i| ((i % 3) as i8) - 1).collect();
        let trits_b: Vec<i8> = (0..num_trits).map(|i| (((i + 1) % 3) as i8) - 1).collect();
        let mut scalar_res = vec![0i8; num_trits];
        for i in 0..num_trits {
            // black_box keeps the scalar arm doing the same work as the parallel arm.
            scalar_res[i] = std::hint::black_box(match (trits_a[i], trits_b[i]) {
                (1, 1) => 1,
                (-1, _) | (_, -1) => -1,
                _ => 0,
            });
        }
        let scalar_time = t_84_scalar.elapsed().as_secs_f64();

        let t_84_parallel = Instant::now();
        trits_a.par_chunks_mut(64).zip(trits_b.par_chunks(64)).for_each(|(chunk_a, chunk_b)| {
            for k in 0..chunk_a.len() {
                chunk_a[k] = if chunk_a[k] == 1 && chunk_b[k] == 1 {
                    1
                } else if chunk_a[k] == -1 || chunk_b[k] == -1 {
                    -1
                } else {
                    0
                };
            }
        });
        let parallel_time_84 = t_84_parallel.elapsed().as_secs_f64();
        let parallel_speedup_84 = speedup(scalar_time, parallel_time_84);
        let _ = scalar_res;
        results.push(TaskResult {
            task_id: 84,
            task_code: "Task 84 🟢",
            title: "Rayon Ternary K3 Combine (parallel vs scalar)",
            measured_metric: "Thread-parallel Speedup (not SIMD)",
            numerical_value: parallel_speedup_84.unwrap_or(f64::NAN),
            target_threshold: ">= 10.0x",
            passed: parallel_speedup_84.map(|s| s >= 10.0).unwrap_or(false),
            basis: MeasurementBasis::Measured,
            latency_us: parallel_time_84 * 1e6,
        });

        // -------------------------------------------------------------
        // Task 85: in-process store/load latency over a STACK buffer.
        // NOTE: no memory mapping and no second process are involved, so this is
        // NOT inter-process IPC and NOT zero-copy. It measures a volatile-ish
        // round trip through a shared array in the current address space.
        // -------------------------------------------------------------
        let t_85 = Instant::now();
        let ipc_roundtrips = 100_000;
        let mut shared_buffer = [0u64; 8];
        for i in 0..ipc_roundtrips {
            std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
            shared_buffer[0] = (i as u64) ^ MONAD_TRUTH_ANCHOR;
            std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
            let _val = std::hint::black_box(shared_buffer[0]);
        }
        let ipc_time = t_85.elapsed().as_secs_f64();
        let ipc_latency_ns = if ipc_time > 0.0 {
            (ipc_time * 1e9) / (ipc_roundtrips as f64)
        } else {
            f64::NAN
        };
        results.push(TaskResult {
            task_id: 85,
            task_code: "Task 85 🟢",
            title: "In-process Store/Load Latency (stack buffer)",
            measured_metric: "Round-trip Latency (not IPC)",
            numerical_value: ipc_latency_ns,
            target_threshold: "< 50.0 ns",
            passed: ipc_latency_ns < 50.0,
            basis: MeasurementBasis::Measured,
            latency_us: ipc_time * 1e6,
        });

        // -------------------------------------------------------------
        // Task 86: NUMA-aware core affinity — PROJECTED, NOT MEASURED.
        // Pinning threads to NUMA nodes is not performed here, and no throughput
        // is sampled. The two figures below are a scenario, so this row can never
        // be evidence of a speedup.
        // -------------------------------------------------------------
        let throughput_baseline_86 = 42.5; // scenario constant — no run backs this
        let throughput_numa_86 = 54.8;     // scenario constant — no run backs this
        let projected_boost_pct = ((throughput_numa_86 - throughput_baseline_86)
            / throughput_baseline_86) * 100.0;
        results.push(TaskResult {
            task_id: 86,
            task_code: "Task 86 🔵",
            title: "NUMA Core Affinity (PROJECTED — not run)",
            measured_metric: "Projected Throughput Boost",
            numerical_value: projected_boost_pct,
            target_threshold: ">= 20.0% (projection only)",
            passed: false,
            basis: MeasurementBasis::Projected,
            latency_us: 0.0,
        });

        // -------------------------------------------------------------
        // Task 87: local counter advance rate.
        // NOTE: this loop touches no socket, no ring buffer and no packet. It was
        // reported as ">= 10 Gbps network throughput"; that claim is withdrawn.
        // -------------------------------------------------------------
        let t_87 = Instant::now();
        let counter_steps = 10_000_000u64;
        let mut ring_head = 0usize;
        for _ in 0..counter_steps {
            // black_box stops LLVM solving the modular counter in closed form.
            ring_head = std::hint::black_box((std::hint::black_box(ring_head) + 1) & 0xFFFF);
        }
        let net_time = t_87.elapsed().as_secs_f64();
        // Single-threaded loop, so the plausibility ceiling uses one core.
        let counter_ops_per_sec = honest_rate(counter_steps as f64, net_time, 1);
        results.push(TaskResult {
            task_id: 87,
            task_code: "Task 87 🔵",
            title: "Local Counter Advance (no network path)",
            measured_metric: "Counter Ops/sec (original 'Gbps' claim withdrawn)",
            numerical_value: counter_ops_per_sec,
            target_threshold: "n/a — not a network measurement",
            passed: false,
            basis: MeasurementBasis::Measured,
            latency_us: net_time * 1e6,
        });
        let _ = ring_head;

        // -------------------------------------------------------------
        // Task 88: RX 6700 XT Wave32 throughput — SPECIFIED, NOT MEASURED.
        // Both figures are vendor datasheet numbers for a GPU this binary never
        // dispatches to. This row cannot be evidence of a speedup.
        // -------------------------------------------------------------
        let baseline_wave64_tflops = 6.45; // datasheet — not sampled
        let optimized_wave32_dual_cu_tflops = 13.21; // datasheet — not sampled
        let rx6700xt_ratio = optimized_wave32_dual_cu_tflops / baseline_wave64_tflops;
        results.push(TaskResult {
            task_id: 88,
            task_code: "Task 88 🔵",
            title: "RX 6700 XT Wave32 (SPECIFIED — datasheet)",
            measured_metric: "Datasheet Throughput Ratio",
            numerical_value: rx6700xt_ratio,
            target_threshold: ">= 2.0x (datasheet only)",
            passed: false,
            basis: MeasurementBasis::Specified,
            latency_us: 0.0,
        });

        // -------------------------------------------------------------
        // Task 89: DVFS energy — PROJECTED, NOT MEASURED.
        // No power is sampled on this host; 230 W / 168 W are assumptions.
        // -------------------------------------------------------------
        let full_power_watts = 230.0; // assumed baseline TDP — not sampled
        let dvfs_power_watts = 168.0; // assumed throttled TDP — not sampled
        let projected_energy_saved_pct =
            ((full_power_watts - dvfs_power_watts) / full_power_watts) * 100.0;
        results.push(TaskResult {
            task_id: 89,
            task_code: "Task 89 🔵",
            title: "DVFS Energy Saving (PROJECTED — not sampled)",
            measured_metric: "Projected Energy Saved",
            numerical_value: projected_energy_saved_pct,
            target_threshold: ">= 26.4% (projection only)",
            passed: false,
            basis: MeasurementBasis::Projected,
            latency_us: 0.0,
        });

        // -------------------------------------------------------------
        // Task 90: measurement-coverage audit
        // (was: `passed = (tracked_count + 1) == 10`, which passed by counting
        //  its own results vector — a tautology, not a benchmark. It now counts
        //  how many rows are backed by a real measurement, and can fail.)
        // -------------------------------------------------------------
        let measured_count = results.iter().filter(|r| r.basis.is_evidence()).count();
        let evaluated = results.len();
        results.push(TaskResult {
            task_id: 90,
            task_code: "Task 90 🔵",
            title: "Measurement Coverage Audit",
            measured_metric: "Tasks Backed by a Real Measurement",
            numerical_value: measured_count as f64,
            target_threshold: "all tasks measured",
            passed: measured_count == evaluated,
            basis: MeasurementBasis::Measured,
            latency_us: 0.0,
        });

        let total_passed = results.iter().filter(|r| r.passed).count();
        let tasks_measured = results.iter().filter(|r| r.basis.is_evidence()).count();
        let tasks_projected = results.len() - tasks_measured;
        let total_wall_ms = t0.elapsed().as_secs_f64() * 1000.0;

        // Consensus is DERIVED from evidence, never asserted. Ternary: True only
        // when every task is measured and passed; False when a measured task
        // failed; Unknown when any claim rests on a projection or datasheet.
        let k3_consensus = if tasks_projected > 0 {
            K3Trit::Unknown
        } else if results.iter().all(|r| r.passed) {
            K3Trit::True
        } else {
            K3Trit::False
        };

        SupremacyCampaignReport {
            total_tasks_executed: results.len(),
            tasks_passed: total_passed,
            tasks_measured,
            tasks_projected,
            overall_execution_time_ms: total_wall_ms,
            results,
            k3_consensus,
            monad_anchor: MONAD_TRUTH_ANCHOR,
        }
    }
}
