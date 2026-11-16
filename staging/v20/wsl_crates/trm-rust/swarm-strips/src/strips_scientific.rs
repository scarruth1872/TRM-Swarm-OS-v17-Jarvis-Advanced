//! Scientific Intelligence strips: AlphaGenome (variant effect), SequenceAligner (Needleman-Wunsch).

/// Variant effect = normalized Hamming distance between reference and variant. Input: ref:variant.
pub fn alpha_genome(input: &str) -> String {
    let parts: Vec<&str> = input.split(':').collect();
    if parts.len() != 2 || parts[0].len() != parts[1].len() {
        return r#"{"agent":"AlphaGenome","status":"error","error":"input = ref:variant (equal length)"}"#.to_string();
    }
    let mismatches = parts[0].bytes().zip(parts[1].bytes()).filter(|(a, b)| a != b).count();
    let effect = mismatches as f64 / parts[0].len() as f64;
    format!(r#"{{"agent":"AlphaGenome","status":"ok","mismatches":{mismatches},"effect_score":{effect:.4},"kind":"derived"}}"#)
}

/// Needleman-Wunsch global alignment score (+1 match, -1 mismatch/gap). Input: seq1|seq2.
pub fn sequence_aligner(input: &str) -> String {
    let parts: Vec<&str> = input.split('|').collect();
    if parts.len() != 2 {
        return r#"{"agent":"SequenceAligner","status":"error","error":"input = seq1|seq2"}"#.to_string();
    }
    let (a, b) = (parts[0].as_bytes(), parts[1].as_bytes());
    let (n, m) = (a.len(), b.len());
    let (gap, mismatch, mat) = (-1i32, -1i32, 1i32);
    let mut d = vec![vec![0i32; m + 1]; n + 1];
    for i in 0..=n { d[i][0] = i as i32 * gap; }
    for j in 0..=m { d[0][j] = j as i32 * gap; }
    for i in 1..=n {
        for j in 1..=m {
            let s = if a[i - 1] == b[j - 1] { mat } else { mismatch };
            d[i][j] = (d[i - 1][j - 1] + s).max(d[i - 1][j] + gap).max(d[i][j - 1] + gap);
        }
    }
    format!(r#"{{"agent":"SequenceAligner","status":"ok","algorithm":"Needleman-Wunsch","score":{},"kind":"derived"}}"#, d[n][m])
}
