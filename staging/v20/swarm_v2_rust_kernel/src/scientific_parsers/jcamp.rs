use super::error::ParserError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SpectralPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct JcampSpectrum {
    pub title: String,
    pub data_type: String,
    pub x_units: String,
    pub y_units: String,
    pub x_factor: f64,
    pub y_factor: f64,
    pub first_x: f64,
    pub last_x: f64,
    pub delta_x: f64,
    pub expected_npoints: usize,
    pub points: Vec<SpectralPoint>,
}

pub struct JcampParser;

impl JcampParser {
    pub fn parse(text: &str) -> Result<JcampSpectrum, ParserError> {
        let mut headers = HashMap::new();
        let mut data_lines = Vec::new();
        let mut in_data_block = false;

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            if trimmed.starts_with("##") {
                if let Some(eq_pos) = trimmed.find('=') {
                    let key = trimmed[2..eq_pos].trim().to_uppercase();
                    let val = trimmed[eq_pos + 1..].trim().to_string();

                    if key == "XYDATA" || key == "PEAK TABLE" {
                        in_data_block = true;
                        headers.insert(key, val);
                        continue;
                    } else if key == "END" {
                        break;
                    }
                    headers.insert(key, val);
                }
            } else if in_data_block {
                data_lines.push(trimmed);
            }
        }

        let title = headers.get("TITLE").cloned().unwrap_or_else(|| "Unknown".to_string());
        let data_type = headers.get("DATA TYPE").cloned().unwrap_or_else(|| "UNKNOWN".to_string());
        let x_units = headers.get("XUNITS").cloned().unwrap_or_else(|| "1/CM".to_string());
        let y_units = headers.get("YUNITS").cloned().unwrap_or_else(|| "ABSORBANCE".to_string());

        let x_factor = headers.get("XFACTOR").and_then(|v| v.parse::<f64>().ok()).unwrap_or(1.0);
        let y_factor = headers.get("YFACTOR").and_then(|v| v.parse::<f64>().ok()).unwrap_or(1.0);
        let first_x = headers.get("FIRSTX").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        let last_x = headers.get("LASTX").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        let delta_x = headers.get("DELTAX").and_then(|v| v.parse::<f64>().ok()).unwrap_or(1.0);
        let expected_npoints = headers.get("NPOINTS").and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);

        let mut points = Vec::new();

        for line in data_lines {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() { continue; }

            if let Ok(line_start_x) = tokens[0].parse::<f64>() {
                for (idx, y_tok) in tokens[1..].iter().enumerate() {
                    if let Ok(raw_y) = y_tok.parse::<f64>() {
                        let cur_x = (line_start_x + (idx as f64 * delta_x)) * x_factor;
                        let physical_y = raw_y * y_factor;
                        points.push(SpectralPoint { x: cur_x, y: physical_y });
                    }
                }
            }
        }

        Ok(JcampSpectrum {
            title,
            data_type,
            x_units,
            y_units,
            x_factor,
            y_factor,
            first_x,
            last_x,
            delta_x,
            expected_npoints,
            points,
        })
    }
}
