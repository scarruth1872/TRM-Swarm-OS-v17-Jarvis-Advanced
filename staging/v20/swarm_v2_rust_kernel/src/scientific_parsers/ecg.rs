use super::error::ParserError;

#[derive(Debug, Clone)]
pub struct WfdbSignalSpec {
    pub filename: String,
    pub format: u16,
    pub gain: f64,
    pub baseline: i32,
    pub units: String,
    pub lead_name: String,
}

#[derive(Debug, Clone)]
pub struct WfdbHeader {
    pub record_name: String,
    pub num_leads: usize,
    pub sampling_rate_hz: f64,
    pub total_samples: usize,
    pub signals: Vec<WfdbSignalSpec>,
    pub comments: Vec<String>,
}

#[derive(Debug)]
pub struct EcgRecord {
    pub header: WfdbHeader,
    pub lead_voltages_mv: Vec<Vec<f64>>,
    pub num_samples: usize,
}

pub struct EcgParser;

impl EcgParser {
    pub fn parse_header(content: &str) -> Result<WfdbHeader, ParserError> {
        let mut lines = content.lines();
        let first_line = lines.next().ok_or_else(|| ParserError::InvalidFormat("Empty header file".to_string()))?;
        let first_tokens: Vec<&str> = first_line.split_whitespace().collect();

        if first_tokens.len() < 4 {
            return Err(ParserError::InvalidFormat("Invalid WFDB record line".to_string()));
        }

        let record_name = first_tokens[0].to_string();
        let num_leads: usize = first_tokens[1].parse().map_err(|e: std::num::ParseIntError| ParserError::InvalidFormat(e.to_string()))?;
        let sampling_rate_hz: f64 = first_tokens[2].parse().map_err(|e: std::num::ParseFloatError| ParserError::InvalidFormat(e.to_string()))?;
        let total_samples: usize = first_tokens[3].parse().map_err(|e: std::num::ParseIntError| ParserError::InvalidFormat(e.to_string()))?;

        let mut signals = Vec::new();
        let mut comments = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                comments.push(trimmed[1..].trim().to_string());
                continue;
            }
            if trimmed.is_empty() { continue; }

            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            if tokens.len() >= 9 {
                let filename = tokens[0].to_string();
                let format: u16 = tokens[1].parse().unwrap_or(212);
                let gain: f64 = tokens[2].parse().unwrap_or(200.0);
                let baseline: i32 = tokens[4].parse().unwrap_or(1024);
                let lead_name = tokens[8].to_string();

                signals.push(WfdbSignalSpec {
                    filename,
                    format,
                    gain,
                    baseline,
                    units: "mV".to_string(),
                    lead_name,
                });
            }
        }

        Ok(WfdbHeader {
            record_name,
            num_leads,
            sampling_rate_hz,
            total_samples,
            signals,
            comments,
        })
    }

    pub fn parse_format_212(raw_bytes: &[u8], header: &WfdbHeader) -> Result<EcgRecord, ParserError> {
        let num_triplets = raw_bytes.len() / 3;
        let mut ch0 = Vec::with_capacity(num_triplets);
        let mut ch1 = Vec::with_capacity(num_triplets);

        let gain0 = header.signals.first().map(|s| s.gain).unwrap_or(200.0);
        let base0 = header.signals.first().map(|s| s.baseline).unwrap_or(1024);

        let gain1 = header.signals.get(1).map(|s| s.gain).unwrap_or(200.0);
        let base1 = header.signals.get(1).map(|s| s.baseline).unwrap_or(1024);

        for i in 0..num_triplets {
            let b0 = raw_bytes[i * 3] as i32;
            let b1 = raw_bytes[i * 3 + 1] as i32;
            let b2 = raw_bytes[i * 3 + 2] as i32;

            let mut val0 = b0 | ((b1 & 0x0F) << 8);
            if val0 > 2047 { val0 -= 4096; }

            let mut val1 = b2 | ((b1 & 0xF0) << 4);
            if val1 > 2047 { val1 -= 4096; }

            let mv0 = (val0 - base0) as f64 / gain0;
            let mv1 = (val1 - base1) as f64 / gain1;

            ch0.push(mv0);
            ch1.push(mv1);
        }

        let num_samples = ch0.len();

        Ok(EcgRecord {
            header: header.clone(),
            lead_voltages_mv: vec![ch0, ch1],
            num_samples,
        })
    }

    pub fn calculate_heart_rate(lead_mv: &[f64], sampling_rate: f64) -> (f64, Vec<usize>) {
        if lead_mv.is_empty() || sampling_rate <= 0.0 {
            return (0.0, vec![]);
        }

        let mut r_peaks = Vec::new();
        let refractory = (0.25 * sampling_rate) as usize;
        let mut last_peak = 0usize;

        for i in 1..lead_mv.len().saturating_sub(1) {
            if lead_mv[i] > 0.4 && lead_mv[i] > lead_mv[i - 1] && lead_mv[i] > lead_mv[i + 1] {
                if r_peaks.is_empty() || (i - last_peak) >= refractory {
                    r_peaks.push(i);
                    last_peak = i;
                }
            }
        }

        if r_peaks.len() < 2 {
            return (0.0, r_peaks);
        }

        let mut total_rr = 0.0;
        for j in 1..r_peaks.len() {
            total_rr += (r_peaks[j] - r_peaks[j - 1]) as f64 / sampling_rate;
        }

        let avg_rr = total_rr / (r_peaks.len() - 1) as f64;
        let bpm = 60.0 / avg_rr;

        (bpm, r_peaks)
    }
}
