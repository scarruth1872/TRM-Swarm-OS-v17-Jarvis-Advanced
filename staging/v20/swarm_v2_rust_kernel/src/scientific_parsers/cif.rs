use super::error::ParserError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UnitCell {
    pub length_a: f64,
    pub length_b: f64,
    pub length_c: f64,
    pub angle_alpha: f64,
    pub angle_beta: f64,
    pub angle_gamma: f64,
}

#[derive(Debug, Clone)]
pub struct AtomSite {
    pub id: usize,
    pub type_symbol: String,
    pub label_atom_id: String,
    pub label_comp_id: String,
    pub label_asym_id: String,
    pub label_seq_id: Option<i32>,
    pub cartn_x: f64,
    pub cartn_y: f64,
    pub cartn_z: f64,
    pub occupancy: f64,
    pub b_factor: f64,
}

#[derive(Debug)]
pub struct CifDataBlock {
    pub block_name: String,
    pub single_items: HashMap<String, String>,
    pub loops: Vec<CifLoop>,
}

#[derive(Debug, Clone)]
pub struct CifLoop {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl CifLoop {
    pub fn get_column(&self, header_name: &str) -> Option<usize> {
        self.headers.iter().position(|h| h.eq_ignore_ascii_case(header_name))
    }
}

pub struct CifParser;

impl CifParser {
    pub fn parse(text: &str) -> Result<Vec<CifDataBlock>, ParserError> {
        let tokens = Self::tokenize(text);
        let mut blocks = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            let tok = &tokens[i];
            if tok.starts_with("data_") {
                let block_name = tok[5..].to_string();
                i += 1;
                let (block, next_i) = Self::parse_data_block(&tokens, i, block_name)?;
                blocks.push(block);
                i = next_i;
            } else {
                i += 1;
            }
        }

        if blocks.is_empty() {
            return Err(ParserError::InvalidFormat("No data_ block found in CIF".to_string()));
        }

        Ok(blocks)
    }

    fn parse_data_block(tokens: &[String], mut i: usize, block_name: String) -> Result<(CifDataBlock, usize), ParserError> {
        let mut single_items = HashMap::new();
        let mut loops = Vec::new();

        while i < tokens.len() {
            let tok = &tokens[i];
            if tok.starts_with("data_") {
                break;
            } else if tok.eq_ignore_ascii_case("loop_") {
                i += 1;
                let mut headers = Vec::new();
                while i < tokens.len() && tokens[i].starts_with('_') {
                    headers.push(tokens[i].clone());
                    i += 1;
                }

                if headers.is_empty() {
                    return Err(ParserError::InvalidFormat(format!("Empty loop headers in block {}", block_name)));
                }

                let num_cols = headers.len();
                let mut rows = Vec::new();
                let mut current_row = Vec::with_capacity(num_cols);

                while i < tokens.len() {
                    let next_tok = &tokens[i];
                    if next_tok.starts_with('_') || next_tok.eq_ignore_ascii_case("loop_") || next_tok.starts_with("data_") {
                        break;
                    }
                    current_row.push(next_tok.clone());
                    if current_row.len() == num_cols {
                        rows.push(current_row);
                        current_row = Vec::with_capacity(num_cols);
                    }
                    i += 1;
                }

                loops.push(CifLoop { headers, rows });
            } else if tok.starts_with('_') {
                let tag = tok.clone();
                i += 1;
                if i < tokens.len() {
                    single_items.insert(tag, tokens[i].clone());
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        Ok((CifDataBlock { block_name, single_items, loops }, i))
    }

    fn tokenize(input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
                continue;
            }
            if c == '#' {
                while let Some(ch) = chars.next() {
                    if ch == '\n' { break; }
                }
                continue;
            }
            if c == ';' {
                chars.next();
                let mut content = String::new();
                let mut at_line_start = false;
                while let Some(ch) = chars.next() {
                    if at_line_start && ch == ';' { break; }
                    at_line_start = ch == '\n';
                    content.push(ch);
                }
                tokens.push(content.trim().to_string());
                continue;
            }
            if c == '\'' || c == '"' {
                let quote = chars.next().unwrap();
                let mut content = String::new();
                while let Some(ch) = chars.next() {
                    if ch == quote { break; }
                    content.push(ch);
                }
                tokens.push(content);
                continue;
            }

            let mut word = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() || ch == '#' { break; }
                word.push(chars.next().unwrap());
            }
            if !word.is_empty() {
                tokens.push(word);
            }
        }

        tokens
    }

    pub fn extract_unit_cell(block: &CifDataBlock) -> Option<UnitCell> {
        let get_f64 = |tag: &str| -> Option<f64> {
            block.single_items.get(tag).and_then(|v| v.parse::<f64>().ok())
        };

        Some(UnitCell {
            length_a: get_f64("_cell.length_a")?,
            length_b: get_f64("_cell.length_b")?,
            length_c: get_f64("_cell.length_c")?,
            angle_alpha: get_f64("_cell.angle_alpha")?,
            angle_beta: get_f64("_cell.angle_beta")?,
            angle_gamma: get_f64("_cell.angle_gamma")?,
        })
    }

    pub fn extract_atom_sites(block: &CifDataBlock) -> Vec<AtomSite> {
        for l in &block.loops {
            if let (Some(x_col), Some(y_col), Some(z_col)) = (
                l.get_column("_atom_site.cartn_x"),
                l.get_column("_atom_site.cartn_y"),
                l.get_column("_atom_site.cartn_z"),
            ) {
                let id_col = l.get_column("_atom_site.id");
                let sym_col = l.get_column("_atom_site.type_symbol");
                let atom_col = l.get_column("_atom_site.label_atom_id");
                let comp_col = l.get_column("_atom_site.label_comp_id");
                let asym_col = l.get_column("_atom_site.label_asym_id");
                let seq_col = l.get_column("_atom_site.label_seq_id");
                let occ_col = l.get_column("_atom_site.occupancy");
                let b_col = l.get_column("_atom_site.b_iso_or_equiv");

                let mut atoms = Vec::with_capacity(l.rows.len());

                for (idx, row) in l.rows.iter().enumerate() {
                    let cartn_x = row.get(x_col).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                    let cartn_y = row.get(y_col).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                    let cartn_z = row.get(z_col).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

                    let id = id_col.and_then(|c| row.get(c)).and_then(|s| s.parse::<usize>().ok()).unwrap_or(idx + 1);
                    let type_symbol = sym_col.and_then(|c| row.get(c)).cloned().unwrap_or_else(|| "C".to_string());
                    let label_atom_id = atom_col.and_then(|c| row.get(c)).cloned().unwrap_or_default();
                    let label_comp_id = comp_col.and_then(|c| row.get(c)).cloned().unwrap_or_default();
                    let label_asym_id = asym_col.and_then(|c| row.get(c)).cloned().unwrap_or_default();
                    let label_seq_id = seq_col.and_then(|c| row.get(c)).and_then(|s| s.parse::<i32>().ok());
                    let occupancy = occ_col.and_then(|c| row.get(c)).and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
                    let b_factor = b_col.and_then(|c| row.get(c)).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

                    atoms.push(AtomSite {
                        id,
                        type_symbol,
                        label_atom_id,
                        label_comp_id,
                        label_asym_id,
                        label_seq_id,
                        cartn_x,
                        cartn_y,
                        cartn_z,
                        occupancy,
                        b_factor,
                    });
                }
                return atoms;
            }
        }
        Vec::new()
    }
}
