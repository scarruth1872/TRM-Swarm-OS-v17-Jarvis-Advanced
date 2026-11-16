use super::error::ParserError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MicroarrayProbe {
    pub id_ref: String,
    pub value: f64,
    pub call: Option<String>,
    pub p_value: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct GeoSample {
    pub sample_id: String,
    pub title: String,
    pub organism: String,
    pub platform_id: String,
    pub source_name: String,
    pub attributes: HashMap<String, Vec<String>>,
    pub probes: Vec<MicroarrayProbe>,
}

impl GeoSample {
    /// Calculate summary statistics (min, max, mean, present_calls)
    pub fn statistics(&self) -> (f64, f64, f64, usize) {
        if self.probes.is_empty() {
            return (0.0, 0.0, 0.0, 0);
        }
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut sum = 0.0;
        let mut present_count = 0;

        for p in &self.probes {
            min = min.min(p.value);
            max = max.max(p.value);
            sum += p.value;
            if let Some(call) = &p.call {
                if call.eq_ignore_ascii_case("P") {
                    present_count += 1;
                }
            }
        }

        let mean = sum / self.probes.len() as f64;
        (min, max, mean, present_count)
    }

    /// Retrieve probe by ID_REF
    pub fn get_probe(&self, probe_id: &str) -> Option<&MicroarrayProbe> {
        self.probes.iter().find(|p| p.id_ref.eq_ignore_ascii_case(probe_id))
    }
}

pub struct GeoSoftParser;

impl GeoSoftParser {
    pub fn parse(text: &str) -> Result<Vec<GeoSample>, ParserError> {
        let mut samples = Vec::new();
        let mut current_sample: Option<GeoSample> = None;
        let mut in_table = false;
        let mut col_indices: HashMap<String, usize> = HashMap::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check for entity definition
            if trimmed.starts_with('^') {
                if let Some(s) = current_sample.take() {
                    samples.push(s);
                }
                in_table = false;
                col_indices.clear();

                if let Some(eq_pos) = trimmed.find('=') {
                    let entity_type = trimmed[1..eq_pos].trim().to_uppercase();
                    let entity_id = trimmed[eq_pos + 1..].trim().to_string();

                    if entity_type == "SAMPLE" {
                        current_sample = Some(GeoSample {
                            sample_id: entity_id,
                            title: String::new(),
                            organism: String::new(),
                            platform_id: String::new(),
                            source_name: String::new(),
                            attributes: HashMap::new(),
                            probes: Vec::new(),
                        });
                    }
                }
                continue;
            }

            // Table control tags
            if trimmed.eq_ignore_ascii_case("!sample_table_begin") {
                in_table = true;
                col_indices.clear();
                continue;
            } else if trimmed.eq_ignore_ascii_case("!sample_table_end") {
                in_table = false;
                continue;
            }

            // Metadata attribute tags
            if trimmed.starts_with('!') {
                if let Some(eq_pos) = trimmed.find('=') {
                    let key = trimmed[1..eq_pos].trim().to_lowercase();
                    let val = trimmed[eq_pos + 1..].trim().to_string();

                    if let Some(sample) = current_sample.as_mut() {
                        if key == "sample_title" {
                            sample.title = val.clone();
                        } else if key == "sample_organism_ch1" {
                            sample.organism = val.clone();
                        } else if key == "sample_platform_id" {
                            sample.platform_id = val.clone();
                        } else if key == "sample_source_name_ch1" {
                            sample.source_name = val.clone();
                        }
                        sample.attributes.entry(key).or_default().push(val);
                    }
                }
                continue;
            }

            // Skip comment/header lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Table rows
            if in_table {
                if let Some(sample) = current_sample.as_mut() {
                    let cols: Vec<&str> = trimmed.split('\t').map(|s| s.trim()).collect();
                    if cols.is_empty() {
                        continue;
                    }

                    if col_indices.is_empty() {
                        // First line in table is the header row
                        for (idx, name) in cols.iter().enumerate() {
                            col_indices.insert(name.to_uppercase(), idx);
                        }
                    } else {
                        // Data row
                        let id_idx = col_indices.get("ID_REF").copied().unwrap_or(0);
                        let val_idx = col_indices.get("VALUE").copied().unwrap_or(1);
                        let call_idx = col_indices.get("ABS_CALL").copied();
                        let pval_idx = col_indices.get("DETECTION P-VALUE").copied();

                        let id_ref = cols.get(id_idx).unwrap_or(&"").to_string();
                        let value = cols.get(val_idx).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                        let call = call_idx.and_then(|i| cols.get(i)).map(|s| s.to_string());
                        let p_value = pval_idx.and_then(|i| cols.get(i)).and_then(|s| s.parse::<f64>().ok());

                        sample.probes.push(MicroarrayProbe {
                            id_ref,
                            value,
                            call,
                            p_value,
                        });
                    }
                }
            }
        }

        if let Some(s) = current_sample {
            samples.push(s);
        }

        if samples.is_empty() {
            return Err(ParserError::InvalidFormat("No ^SAMPLE entities found in GEO SOFT file".to_string()));
        }

        Ok(samples)
    }
}
