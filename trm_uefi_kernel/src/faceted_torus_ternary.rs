//! faceted_torus_ternary.rs — TRM Swarm OS v19.002 "Sovereign"
//! ==========================================================
//! The 108-Faceted Ternary Expansion & 4D Toroidal Latent Manifold
//! 
//! Unfolds the discrete ternary K3 line {-1, 0, +1} into a continuous
//! 217-state helical manifold with 108 facets per pole and 0 as the
//! central Monad invariant axis.
//!
//! Monad Truth Anchor: 0x4D4F4E4144000001

extern crate alloc;
use alloc::vec::Vec;

pub const MONAD_TRUTH_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;
pub const MAX_FACET: i16 = 108;
pub const TOTAL_STATES: usize = 217;

// no_std has no f32::round/sin/cos - those come from std's libm. This module
// was never compiled before being declared, which is why the missing methods
// went unnoticed. Local versions keep it dependency-free (~1e-4 accurate).
fn round_f32(x: f32) -> f32 {
    if x >= 0.0 { (x + 0.5) as i32 as f32 } else { -((-x + 0.5) as i32 as f32) }
}

fn sin_f32(x: f32) -> f32 {
    const PI: f32 = 3.14159265;
    const TWO_PI: f32 = 6.28318531;
    let mut t = x % TWO_PI;
    if t > PI { t -= TWO_PI; }
    if t < -PI { t += TWO_PI; }
    let neg = t < 0.0;
    let a = if neg { -t } else { t };
    let a = if a > PI * 0.5 { PI - a } else { a };
    let a2 = a * a;
    let v = a * (1.0 - a2 / 6.0 + a2 * a2 / 120.0 - a2 * a2 * a2 / 5040.0);
    if neg { -v } else { v }
}

fn cos_f32(x: f32) -> f32 { sin_f32(x + 1.57079633) }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FacetedTrit {
    pub facet: i16, // In [-108, +108]
}

impl FacetedTrit {
    pub const CENTER_ZERO: Self = Self { facet: 0 };
    pub const MAX_POSITIVE: Self = Self { facet: 108 };
    pub const MAX_NEGATIVE: Self = Self { facet: -108 };

    #[inline(always)]
    pub fn from_f32(val: f32) -> Self {
        let clamped = val.clamp(-1.0, 1.0);
        let facet = round_f32(clamped * 108.0) as i16;
        Self { facet }
    }

    #[inline(always)]
    pub fn to_f32(&self) -> f32 {
        (self.facet as f32) / 108.0
    }

    #[inline(always)]
    pub fn to_k3_trit(&self) -> i8 {
        if self.facet > 0 {
            1
        } else if self.facet < 0 {
            -1
        } else {
            0
        }
    }

    #[inline(always)]
    pub fn phase_angle_rad(&self) -> f32 {
        (self.facet as f32 / 108.0) * core::f32::consts::PI
    }

    /// Index of this facet on the 217-state axis: -108 -> 0, 0 -> 108, +108 -> 216.
    #[inline(always)]
    pub fn axis_index(&self) -> usize {
        (self.facet as i32 + MAX_FACET as i32) as usize
    }

    /// Embeds into the 4D Clifford torus (S^3 unit sphere in R^4), INJECTIVELY.
    ///
    /// DEFECT FIXED IN G85. The previous embedding used `theta = facet/108 * PI`, so facet +108 and -108
    /// both produced theta = +/-PI and `cos(PI) == cos(-PI)`: the two poles of the ternary axis were the
    /// SAME point in space. Measured: 217 facet states -> 216 distinct (x,y) points, i.e. the densest
    /// packing lost its two extremes. With 217 equally spaced angles over the full circle there is no such
    /// symmetry: k and k' differ by less than 217 steps, so theta differs mod 2*pi by definition.
    ///
    /// The angle is offset by -PI so that facet 0 still lands near theta = 0 (it now sits 0.83 degrees off,
    /// 2*pi/217/2, which is the cost of separating the poles).
    pub fn to_4d_torus(&self, recurrence_turn: f32) -> (f32, f32, f32, f32) {
        const N: f32 = TOTAL_STATES as f32; // 217
        const PI: f32 = 3.14159265;
        let k = self.axis_index() as f32;
        let theta = (2.0 * PI * k / N) - PI;
        let phi = recurrence_turn * 2.0 * PI;
        let r = 0.70710678; // 1 / sqrt(2)
        (
            r * cos_f32(theta),
            r * sin_f32(theta),
            r * cos_f32(phi),
            r * sin_f32(phi),
        )
    }

    /// The superseded embedding, kept so the G84 measurement stays reproducible and the change is auditable
    /// rather than erased. Do not use for new work: it is not injective (facet +108 == facet -108).
    #[deprecated(note = "not injective: facet +108 and -108 collide; use to_4d_torus")]
    pub fn to_4d_torus_legacy(&self, recurrence_turn: f32) -> (f32, f32, f32, f32) {
        let theta = self.phase_angle_rad();
        let phi = recurrence_turn * 2.0 * core::f32::consts::PI;
        let r = 0.70710678;
        (
            r * cos_f32(theta),
            r * sin_f32(theta),
            r * cos_f32(phi),
            r * sin_f32(phi),
        )
    }
}

pub struct TorusFacetQuantizer {
    pub resolution: usize,
    pub monad_anchor: u64,
}

impl TorusFacetQuantizer {
    pub fn new() -> Self {
        Self {
            resolution: TOTAL_STATES,
            monad_anchor: MONAD_TRUTH_ANCHOR,
        }
    }

    pub fn quantize_stream(&self, input: &[f32], out: &mut [FacetedTrit]) {
        for (i, &val) in input.iter().enumerate() {
            if i < out.len() {
                out[i] = FacetedTrit::from_f32(val);
            }
        }
    }
}
