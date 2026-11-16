//! Logic & Mathematics strips: MathSolver (Gaussian elimination), AttemptSampler (superposition
//! sampling), SymbolSolver (Euclidean GCD). All real, deterministic derivations.

/// Gaussian elimination on an NxN system. Input: rows of N coefficients + RHS, pipe-separated,
/// space-separated within a row, e.g. "2 3 8 | 4 1 10".
pub fn math_solver(input: &str) -> String {
    let rows: Vec<Vec<f64>> = input
        .split('|')
        .map(|r| r.split_whitespace().filter_map(|t| t.parse::<f64>().ok()).collect())
        .collect();
    // The system is N rows of N coefficients PLUS the RHS, so every row must hold exactly
    // n + 1 numbers. The old check only required equal row lengths and len >= 2, which let a
    // square NxN matrix through; `n = rows.len()` then drove `for j in k..n + 1 { a[i][j] }`
    // one index past the end of every row and the strip panicked (index 2 into a len-2 row)
    // instead of returning a format error.
    if rows.is_empty() {
        return r#"{"agent":"MathSolver","status":"error","error":"input = N rows of N coeffs + RHS, pipe-separated"}"#.to_string();
    }
    let n = rows.len();
    if rows.iter().any(|r| r.len() != n + 1) {
        return r#"{"agent":"MathSolver","status":"error","error":"input = N rows of N coeffs + RHS, pipe-separated"}"#.to_string();
    }
    let mut a = rows.clone();
    for k in 0..n {
        if a[k][k].abs() < 1e-12 {
            return format!(r#"{{"agent":"MathSolver","status":"error","error":"singular at pivot {k}"}}"#);
        }
        for i in k + 1..n {
            let f = a[i][k] / a[k][k];
            for j in k..n + 1 {
                a[i][j] -= f * a[k][j];
            }
        }
    }
    let mut x = vec![0.0f64; n];
    for i in (0..n).rev() {
        let mut s = a[i][n];
        for j in i + 1..n {
            s -= a[i][j] * x[j];
        }
        x[i] = s / a[i][i];
    }
    let sol = x.iter().map(|v| format!("{v:.4}")).collect::<Vec<_>>().join(", ");
    format!(r#"{{"agent":"MathSolver","status":"ok","solution":"[{sol}]","kind":"derived"}}"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_a_full_system() {
        // 2x + 1y = 5, 1x + 3y = 7  ->  x = 1.6, y = 1.8
        let out = math_solver("2 1 5|1 3 7");
        assert!(out.contains("\"status\":\"ok\""), "{out}");
        assert!(out.contains("1.6000") && out.contains("1.8000"), "{out}");
    }

    #[test]
    fn rejects_malformed_systems_instead_of_panicking() {
        // Regression (measured live 2026-11-02): a square NxN matrix with no RHS column used to
        // index one past the end of every row and panic. A strip must answer, not abort.
        for bad in ["1 2|3 4", "2 1|1 3|5 7", "1 2 3", "", "|"] {
            let out = math_solver(bad);
            assert!(out.contains("\"status\":\"error\""), "{bad:?} -> {out}");
        }
    }
}

/// Sample a basis state from complex amplitudes (Born rule: |amp|^2). Input: comma-separated amplitudes.
pub fn attempt_sampler(input: &str) -> String {
    let amps: Vec<f64> = input.split(',').filter_map(|t| t.trim().parse::<f64>().ok()).collect();
    if amps.is_empty() {
        return r#"{"agent":"AttemptSampler","status":"error","error":"input = comma-separated amplitudes"}"#.to_string();
    }
    let probs: Vec<f64> = amps.iter().map(|a| a * a).collect();
    let total: f64 = probs.iter().sum();
    if total <= 0.0 {
        return r#"{"agent":"AttemptSampler","status":"error","error":"zero norm"}"#.to_string();
    }
    let norm: Vec<f64> = probs.iter().map(|p| p / total).collect();
    let mut r = input.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64)) | 1;
    r ^= r << 13; r ^= r >> 7; r ^= r << 17;
    let rnd = (r % 1_000_000) as f64 / 1_000_000.0;
    let mut cum = 0.0;
    let mut idx = 0;
    for (i, p) in norm.iter().enumerate() {
        cum += p;
        if rnd <= cum { idx = i; break; }
    }
    let ps = norm.iter().map(|p| format!("{p:.4}")).collect::<Vec<_>>().join(", ");
    format!(r#"{{"agent":"AttemptSampler","status":"ok","state":{idx},"probabilities":"[{ps}]","kind":"derived"}}"#)
}

/// Euclidean GCD of two integers (the core of a polynomial CAS reduced to its integer kernel).
pub fn symbol_solver(input: &str) -> String {
    let nums: Vec<i64> = input.split(',').filter_map(|t| t.trim().parse::<i64>().ok()).collect();
    if nums.len() < 2 {
        return r#"{"agent":"SymbolSolver","status":"error","error":"input = two integers 'a,b'"}"#.to_string();
    }
    let (mut a, mut b) = (nums[0].abs(), nums[1].abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    format!(r#"{{"agent":"SymbolSolver","status":"ok","gcd":{a},"kind":"derived"}}"#)
}
