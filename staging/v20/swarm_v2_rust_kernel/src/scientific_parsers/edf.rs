use super::error::ParserError;

#[derive(Debug, Clone)]
pub struct EdfSignalHeader {
    pub label: String,
    pub transducer: String,
    pub physical_dimension: String,
    pub physical_min: f64,
    pub physical_max: f64,
    pub digital_min: i32,
    pub digital_max: i32,
    pub prefiltering: String,
    pub num_samples_per_record: usize,
}

#[derive(Debug, Clone)]
pub struct EdfHeader {
    pub version: String,
    pub patient_id: String,
    pub record_id: String,
    pub start_date: String,
    pub start_time: String,
    pub header_bytes: usize,
    pub num_records: usize,
    pub record_duration_sec: f64,
    pub num_signals: usize,
    pub signals: Vec<EdfSignalHeader>,
}

#[derive(Debug)]
pub struct EdfRecording {
    pub header: EdfHeader,
    /// Physical calibrated signals: channel index -> vector of physical floats (e.g. uV)
    pub channels: Vec<Vec<f64>>,
    pub records_parsed: usize,
}

impl EdfRecording {
    /// Compute statistics (min, max, mean, RMS amplitude) for a given channel
    pub fn channel_stats(&self, channel_idx: usize) -> Option<(f64, f64, f64, f64)> {
        let sig = self.channels.get(channel_idx)?;
        if sig.is_empty() {
            return None;
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;

        for &val in sig {
            min = min.min(val);
            max = max.max(val);
            sum += val;
            sum_sq += val * val;
        }

        let n = sig.len() as f64;
        let mean = sum / n;
        let rms = (sum_sq / n).sqrt();

        Some((min, max, mean, rms))
    }
}

pub struct EdfParser;

impl EdfParser {
    pub fn parse(bytes: &[u8]) -> Result<EdfRecording, ParserError> {
        if bytes.len() < 256 {
            return Err(ParserError::InvalidFormat("File too small for EDF header (< 256 bytes)".to_string()));
        }

        let read_ascii = |start: usize, len: usize| -> String {
            if start + len <= bytes.len() {
                String::from_utf8_lossy(&bytes[start..start + len]).trim().to_string()
            } else {
                String::new()
            }
        };

        let version = read_ascii(0, 8);
        let patient_id = read_ascii(8, 80);
        let record_id = read_ascii(88, 80);
        let start_date = read_ascii(168, 8);
        let start_time = read_ascii(176, 8);

        let header_bytes = read_ascii(184, 8).parse::<usize>()
            .map_err(|e| ParserError::InvalidFormat(format!("Invalid header bytes: {}", e)))?;
        let num_records = read_ascii(236, 8).parse::<usize>()
            .map_err(|e| ParserError::InvalidFormat(format!("Invalid record count: {}", e)))?;
        let record_duration_sec = read_ascii(244, 8).parse::<f64>()
            .map_err(|e| ParserError::InvalidFormat(format!("Invalid record duration: {}", e)))?;
        let num_signals = read_ascii(252, 4).parse::<usize>()
            .map_err(|e| ParserError::InvalidFormat(format!("Invalid signals count: {}", e)))?;

        if bytes.len() < header_bytes {
            return Err(ParserError::InvalidFormat(format!("File smaller than declared header: {} < {}", bytes.len(), header_bytes)));
        }

        // Parse variable signal headers
        let mut curr = 256;
        let read_signal_field = |len: usize, curr: &mut usize| -> Vec<String> {
            let mut list = Vec::with_capacity(num_signals);
            for _ in 0..num_signals {
                list.push(read_ascii(*curr, len));
                *curr += len;
            }
            list
        };

        let labels = read_signal_field(16, &mut curr);
        let transducers = read_signal_field(80, &mut curr);
        let physical_dimensions = read_signal_field(8, &mut curr);
        let physical_mins = read_signal_field(8, &mut curr);
        let physical_maxs = read_signal_field(8, &mut curr);
        let digital_mins = read_signal_field(8, &mut curr);
        let digital_maxs = read_signal_field(8, &mut curr);
        let prefilterings = read_signal_field(80, &mut curr);
        let samples_per_rec_strs = read_signal_field(8, &mut curr);
        // curr += num_signals * 32; // reserved field

        let mut signal_headers = Vec::with_capacity(num_signals);
        for i in 0..num_signals {
            let p_min = physical_mins[i].parse::<f64>().unwrap_or(-100.0);
            let p_max = physical_maxs[i].parse::<f64>().unwrap_or(100.0);
            let d_min = digital_mins[i].parse::<i32>().unwrap_or(-32768);
            let d_max = digital_maxs[i].parse::<i32>().unwrap_or(32767);
            let n_samples = samples_per_rec_strs[i].parse::<usize>().unwrap_or(256);

            signal_headers.push(EdfSignalHeader {
                label: labels[i].clone(),
                transducer: transducers[i].clone(),
                physical_dimension: physical_dimensions[i].clone(),
                physical_min: p_min,
                physical_max: p_max,
                digital_min: d_min,
                digital_max: d_max,
                prefiltering: prefilterings[i].clone(),
                num_samples_per_record: n_samples,
            });
        }

        let header = EdfHeader {
            version,
            patient_id,
            record_id,
            start_date,
            start_time,
            header_bytes,
            num_records,
            record_duration_sec,
            num_signals,
            signals: signal_headers.clone(),
        };

        // Parse Data Records
        let bytes_per_record: usize = signal_headers.iter().map(|s| s.num_samples_per_record * 2).sum();
        if bytes_per_record == 0 {
            return Err(ParserError::InvalidFormat("Invalid EDF: zero samples per record".to_string()));
        }

        let data_payload = &bytes[header_bytes..];
        let records_available = data_payload.len() / bytes_per_record;
        let records_to_read = records_available.min(num_records);

        let mut channels: Vec<Vec<f64>> = (0..num_signals)
            .map(|i| Vec::with_capacity(signal_headers[i].num_samples_per_record * records_to_read))
            .collect();

        let mut data_cursor = 0;
        for _ in 0..records_to_read {
            for sig_idx in 0..num_signals {
                let sig = &signal_headers[sig_idx];
                let p_min = sig.physical_min;
                let p_max = sig.physical_max;
                let d_min = sig.digital_min as f64;
                let d_max = sig.digital_max as f64;
                let scale = if (d_max - d_min).abs() > 1e-6 {
                    (p_max - p_min) / (d_max - d_min)
                } else {
                    1.0
                };

                for _ in 0..sig.num_samples_per_record {
                    if data_cursor + 2 <= data_payload.len() {
                        let raw_int = i16::from_le_bytes([data_payload[data_cursor], data_payload[data_cursor + 1]]) as f64;
                        data_cursor += 2;
                        let physical_val = (raw_int - d_min) * scale + p_min;
                        channels[sig_idx].push(physical_val);
                    }
                }
            }
        }

        Ok(EdfRecording {
            header,
            channels,
            records_parsed: records_to_read,
        })
    }
}
