// Basis projection plugin (MOTION-041): a seam for transforming the recall's 768-dim vectors
// into a different basis. The orthonormal baseline is the identity; the icosahedral basis is a
// fixed 12x12 orthonormal matrix (the eigenvector matrix of the icosahedron's 12-vertex Gram
// matrix) applied blockwise to the 12 chunks of 64 dims.
//
// Verified (scratch/build_icosahedral_basis.py): the matrix is orthonormal to 6.7e-16 and preserves
// cosine to 1.4e-17, so projecting BOTH query and store through it leaves retrieval ranking
// unchanged in the continuous limit — the only source of any P@1 delta is 2-bit quantization
// re-rounding after the projection.

/// Transform a raw encoder vector (768-dim) into the target basis.
pub trait BasisProjector: Send + Sync {
    fn project(&self, vec: &[f32]) -> Vec<f32>;
    /// The output dimension. Must match the store's dim.
    fn output_dim(&self) -> usize;
    /// Stable identifier recorded in the ledger.
    fn name(&self) -> &'static str;
}

/// The no-op baseline: the store's native orthonormal basis.
pub struct OrthonormalBasis;
impl BasisProjector for OrthonormalBasis {
    fn project(&self, vec: &[f32]) -> Vec<f32> {
        vec.to_vec()
    }
    fn output_dim(&self) -> usize {
        768
    }
    fn name(&self) -> &'static str {
        "orthonormal"
    }
}

/// The 12-fold icosahedral basis: a fixed 12x12 orthonormal matrix applied to the 12 chunks.
pub struct IcosahedralBasis {
    matrix: [[f32; 12]; 12],
}

impl IcosahedralBasis {
    pub fn new() -> Self {
        Self {
            matrix: [
                [-0.318929448, -0.504778483, -0.041747723, -0.103323743, 0.034589711, -0.025365320, -0.011600502, -0.614206608, 0.042893415, 0.500000000, -0.000000000, -0.000000000],
                [0.114499131, 0.183347869, 0.168312022, -0.184503517, 0.367884753, -0.305934484, -0.609296498, 0.050003798, 0.195512970, 0.223606798, 0.415447879, -0.165538697],
                [0.358749579, 0.163112099, -0.125383894, 0.365814791, -0.137954915, -0.518017549, 0.344400484, -0.178188450, 0.086110653, 0.223606798, 0.285817111, 0.343960142],
                [0.351446949, -0.514753604, 0.531641996, 0.010031060, -0.149606504, -0.068057640, 0.210693062, 0.016982141, 0.084119123, -0.223606798, 0.238803190, -0.378117755],
                [-0.350083875, 0.267721371, 0.614197718, -0.235974731, -0.219101985, -0.094513592, 0.010588487, -0.221981300, 0.128519744, -0.223606798, 0.029056201, 0.446268683],
                [0.080161917, 0.093421580, 0.061378689, -0.031418136, 0.039992354, 0.030669882, 0.170175872, 0.120274244, 0.827119267, 0.223606798, -0.433405599, -0.110270517],
                [0.341181781, -0.110431123, 0.366720165, 0.450816131, 0.141088055, 0.375683682, -0.329887600, -0.014620492, -0.116608060, 0.223606798, -0.238803190, 0.378117755],
                [0.241851984, 0.500952268, 0.082185925, -0.042543766, -0.432084306, 0.253353628, -0.051345055, -0.397500861, -0.143044706, 0.223606798, -0.029056201, -0.446268683],
                [-0.105482939, -0.128468062, -0.285066957, 0.254878441, -0.411258189, 0.340479530, -0.313936444, -0.045995539, 0.436362037, -0.223606798, 0.433405599, 0.110270517],
                [0.226207666, 0.111140815, -0.126030745, 0.002346746, 0.533869059, 0.154066017, 0.100137557, -0.575945381, 0.141758363, -0.500000000, 0.000000000, 0.000000000],
                [0.375865947, -0.213965331, -0.209682702, -0.328800661, -0.333328293, -0.343209148, -0.406190737, -0.129631001, -0.013097155, -0.223606798, -0.415447879, 0.165538697],
                [-0.354265334, 0.042630897, 0.090585418, 0.621744997, -0.024201649, -0.400164656, -0.226961062, -0.124960768, -0.007882369, -0.223606798, -0.285817111, -0.343960142],
            ],
        }
    }
}

impl BasisProjector for IcosahedralBasis {
    fn project(&self, vec: &[f32]) -> Vec<f32> {
        let mut out = vec![0.0f32; 768];
        for chunk in 0..12usize {
            let base = chunk * 64;
            for off in 0..64usize {
                let mut acc = 0.0f32;
                for j in 0..12usize {
                    acc += self.matrix[chunk][j] * vec[j * 64 + off];
                }
                out[base + off] = acc;
            }
        }
        out
    }
    fn output_dim(&self) -> usize {
        768
    }
    fn name(&self) -> &'static str {
        "icosahedral-12x64"
    }
}

/// Resolve a basis projector by its stable name. Unknown names fall back to the orthonormal
/// baseline (the store's native basis) rather than erroring — the caller records which name it
/// asked for, so a mistyped basis is visible in the output, not silently hidden.
pub fn projector(name: &str) -> Box<dyn BasisProjector> {
    match name {
        "icosahedral-12x64" | "icosahedral" => Box::new(IcosahedralBasis::new()),
        _ => Box::new(OrthonormalBasis),
    }
}
