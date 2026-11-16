use super::error::ParserError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FcsParameter {
    pub index: usize,
    pub name: String,
    pub stain: String,
    pub bits: usize,
    pub range: f64,
}

#[derive(Debug, Clone)]
pub struct FcsDataset {
    pub version: String,
    pub total_events: usize,
    pub num_parameters: usize,
    pub data_type: String,
    pub byte_order: String,
    pub parameters: Vec<FcsParameter>,
    pub metadata: HashMap<String, String>,
    /// Flattened event matrix: total_events rows x num_parameters columns
    pub events: Vec<f32>,
}

impl FcsDataset {
    /// Retrieve value for event index and parameter column
    pub fn get_value(&self, event_idx: usize, param_idx: usize) -> Option<f32> {
        if event_idx < self.total_events && param_idx < self.num_parameters {
            Some(self.events[event_idx * self.num_parameters + param_idx])
        } else {
            None
        }
    }

    /// Extract an entire column/channel as a vector across all single cells
    pub fn get_channel_by_name(&self, name: &str) -> Option<Vec<f32>> {
        let p_idx = self.parameters.iter().position(|p| p.name.eq_ignore_ascii_case(name))?;
        let mut col = Vec::with_capacity(self.total_events);
        for ev in 0..self.total_events {
            col.push(self.events[ev * self.num_parameters + p_idx]);
        }
        Some(col)
    }

    /// Compute statistics (min, max, mean) for a given channel
    pub fn channel_stats(&self, param_idx: usize) -> Option<(f32, f32, f32)> {
        if param_idx >= self.num_parameters || self.total_events == 0 {
            return None;
        }
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        let mut sum = 0.0f64;

        for ev in 0..self.total_events {
            let val = self.events[ev * self.num_parameters + param_idx];
            min = min.min(val);
            max = max.max(val);
            sum += val as f64;
        }

        let mean = (sum / self.total_events as f64) as f32;
        Some((min, max, mean))
    }
}

pub struct FcsParser;

impl FcsParser {
    pub fn parse(bytes: &[u8]) -> Result<FcsDataset, ParserError> {
        if bytes.len() < 58 {
            return Err(ParserError::InvalidFormat("File too small to contain FCS header (< 58 bytes)".to_string()));
        }

        // 1. Parse 58-byte Header
        let version = String::from_utf8_lossy(&bytes[0..6]).trim().to_string();
        if !version.starts_with("FCS") {
            return Err(ParserError::InvalidFormat(format!("Invalid FCS version magic: '{}'", version)));
        }

        let parse_offset = |start: usize, end: usize| -> Result<usize, ParserError> {
            let s = String::from_utf8_lossy(&bytes[start..end]).trim().to_string();
            s.parse::<usize>().map_err(|_| ParserError::InvalidFormat(format!("Invalid offset at bytes {}..{}: '{}'", start, end, s)))
        };

        let text_start = parse_offset(10, 18)?;
        let text_end = parse_offset(18, 26)?;
        let mut data_start = parse_offset(26, 34)?;
        let mut data_end = parse_offset(34, 42)?;

        if text_start >= bytes.len() || text_end >= bytes.len() || text_start >= text_end {
            return Err(ParserError::InvalidFormat(format!("Invalid TEXT offsets: {}..{}", text_start, text_end)));
        }

        // 2. Parse TEXT Segment
        let text_bytes = &bytes[text_start..=text_end];
        let delimiter = text_bytes[0] as char;
        let text_str = String::from_utf8_lossy(text_bytes);
        let tokens: Vec<&str> = text_str.split(delimiter).collect();

        let mut metadata: HashMap<String, String> = HashMap::new();
        for i in (1..tokens.len().saturating_sub(1)).step_by(2) {
            let key = tokens[i].trim().to_uppercase();
            let val = tokens[i + 1].trim().to_string();
            metadata.insert(key, val);
        }

        // Handle FCS3.0+ extended offsets when stored in TEXT segment
        if data_start == 0 {
            if let Some(ds) = metadata.get("$DATASTART").and_then(|v| v.parse::<usize>().ok()) {
                data_start = ds;
            }
        }
        if data_end == 0 {
            if let Some(de) = metadata.get("$DATAEND").and_then(|v| v.parse::<usize>().ok()) {
                data_end = de;
            }
        }

        let num_parameters = metadata.get("$PAR")
            .and_then(|v| v.parse::<usize>().ok())
            .ok_or_else(|| ParserError::TagNotFound("$PAR (number of parameters)".to_string()))?;

        let total_events = metadata.get("$TOT")
            .and_then(|v| v.parse::<usize>().ok())
            .ok_or_else(|| ParserError::TagNotFound("$TOT (total events)".to_string()))?;

        let data_type = metadata.get("$DATATYPE").cloned().unwrap_or_else(|| "F".to_string());
        let byte_order = metadata.get("$BYTEORD").cloned().unwrap_or_else(|| "1,2,3,4".to_string());

        let is_little_endian = byte_order.starts_with('1');

        let mut parameters = Vec::with_capacity(num_parameters);
        for p in 1..=num_parameters {
            let name = metadata.get(&format!("$P{}N", p)).cloned().unwrap_or_else(|| format!("P{}", p));
            let stain = metadata.get(&format!("$P{}S", p)).cloned().unwrap_or_else(|| name.clone());
            let bits = metadata.get(&format!("$P{}B", p)).and_then(|v| v.parse::<usize>().ok()).unwrap_or(32);
            let range = metadata.get(&format!("$P{}R", p)).and_then(|v| v.parse::<f64>().ok()).unwrap_or(262144.0);

            parameters.push(FcsParameter {
                index: p,
                name,
                stain,
                bits,
                range,
            });
        }

        // 3. Parse DATA Segment
        if data_start > bytes.len() || data_end > bytes.len() || data_start > data_end {
            return Err(ParserError::InvalidFormat(format!("Invalid DATA offsets: {}..{}", data_start, data_end)));
        }

        let data_slice = &bytes[data_start..=data_end];
        let total_cells = total_events * num_parameters;
        let mut events = Vec::with_capacity(total_cells);

        match data_type.to_uppercase().as_str() {
            "F" => {
                // 32-bit IEEE float
                let chunk_size = 4;
                let num_floats = data_slice.len() / chunk_size;
                let read_count = num_floats.min(total_cells);

                for i in 0..read_count {
                    let mut b = [0u8; 4];
                    b.copy_from_slice(&data_slice[i * 4..(i + 1) * 4]);
                    let val = if is_little_endian {
                        f32::from_le_bytes(b)
                    } else {
                        f32::from_be_bytes(b)
                    };
                    events.push(val);
                }
            }
            "D" => {
                // 64-bit IEEE double
                let chunk_size = 8;
                let num_doubles = data_slice.len() / chunk_size;
                let read_count = num_doubles.min(total_cells);

                for i in 0..read_count {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&data_slice[i * 8..(i + 1) * 8]);
                    let val = if is_little_endian {
                        f64::from_le_bytes(b) as f32
                    } else {
                        f64::from_be_bytes(b) as f32
                    };
                    events.push(val);
                }
            }
            "I" => {
                // 16-bit or 32-bit unsigned integer
                let bits = parameters.first().map(|p| p.bits).unwrap_or(16);
                if bits <= 16 {
                    let num_u16 = data_slice.len() / 2;
                    let read_count = num_u16.min(total_cells);
                    for i in 0..read_count {
                        let mut b = [0u8; 2];
                        b.copy_from_slice(&data_slice[i * 2..(i + 1) * 2]);
                        let val = if is_little_endian {
                            u16::from_le_bytes(b) as f32
                        } else {
                            u16::from_be_bytes(b) as f32
                        };
                        events.push(val);
                    }
                } else {
                    let num_u32 = data_slice.len() / 4;
                    let read_count = num_u32.min(total_cells);
                    for i in 0..read_count {
                        let mut b = [0u8; 4];
                        b.copy_from_slice(&data_slice[i * 4..(i + 1) * 4]);
                        let val = if is_little_endian {
                            u32::from_le_bytes(b) as f32
                        } else {
                            u32::from_be_bytes(b) as f32
                        };
                        events.push(val);
                    }
                }
            }
            other => {
                return Err(ParserError::InvalidFormat(format!("Unsupported FCS $DATATYPE: '{}'", other)));
            }
        }

        Ok(FcsDataset {
            version,
            total_events,
            num_parameters,
            data_type,
            byte_order,
            parameters,
            metadata,
            events,
        })
    }
}
