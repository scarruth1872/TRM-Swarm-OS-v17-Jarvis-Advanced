//! swarm-deepdta — native Rust DeepDTA affinity prediction.
//! Tokenizes (SMILES, protein sequence) -> char indices, then walks the sha256-gated CNN.
//! Honest scope: a real forward pass over the fine-tuned Mpro weights; the numeric output is
//! labeled retired (leave-one-out r = 0.39 < 0.5), so callers must carry that verdict.

pub mod model;

use model::{SMILEN, SEQLEN};

/// ligand char -> index (ligand_dict.json). 'dummy' = 19 for unknown/pad.
pub fn lig_char(c: char) -> usize {
    match c {
        '5' => 0, 'B' => 1, ')' => 2, '-' => 3, 'S' => 4, 'C' => 5, '4' => 6, '3' => 7,
        'F' => 8, '#' => 9, '6' => 10, '~' => 11, 'l' => 12, '\\' => 13, 'r' => 14, 'H' => 15,
        'n' => 16, 'o' => 17, '7' => 18, 'N' => 20, '2' => 21, '@' => 22, '=' => 23, 's' => 24,
        '/' => 25, 'I' => 26, '[' => 27, 'O' => 28, '1' => 29, 'P' => 30, '.' => 31, ']' => 32,
        '(' => 33, '+' => 34, 'c' => 35,
        _ => 19, // dummy
    }
}

/// protein char -> index (protein_dict.json). 'dummy' = 14.
pub fn pro_char(c: char) -> usize {
    match c {
        ':' => 0, 'S' => 1, 'E' => 2, 'C' => 3, 'R' => 4, 'A' => 5, 'F' => 6, 'G' => 7,
        'Y' => 8, 'V' => 9, 'H' => 10, 'L' => 11, 'Q' => 12, 'T' => 13, 'N' => 15, 'X' => 16,
        'W' => 17, 'M' => 18, 'K' => 19, 'D' => 20, 'I' => 21, 'P' => 22,
        _ => 14, // dummy
    }
}

pub fn tokenize_ligand(smiles: &str) -> Vec<usize> {
    let mut v: Vec<usize> = smiles.chars().map(lig_char).take(SMILEN).collect();
    v.resize(SMILEN, 19);
    v
}

pub fn tokenize_protein(seq: &str) -> Vec<usize> {
    let mut v: Vec<usize> = seq.chars().map(pro_char).take(SEQLEN).collect();
    v.resize(SEQLEN, 14);
    v
}

/// Load + predict one (SMILES, sequence) pair. Returns pKd (-log10 Kd/M).
pub fn predict_pkd(smiles: &str, sequence: &str) -> Result<f32, String> {
    let m = model::DeepDtaModel::load(swarm_paths::path(model::RECORDED_MODEL).to_str().unwrap_or(""))?;
    Ok(m.predict(&tokenize_protein(sequence), &tokenize_ligand(smiles)))
}
