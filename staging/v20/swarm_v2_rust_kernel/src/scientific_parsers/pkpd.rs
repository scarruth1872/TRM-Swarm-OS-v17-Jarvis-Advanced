use super::error::ParserError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PkObservation {
    pub time: f64,
    pub conc: f64,
}

#[derive(Debug, Clone)]
pub struct PkSubject {
    pub id: String,
    pub weight_kg: Option<f64>,
    pub dose_mg_kg: Option<f64>,
    pub observations: Vec<PkObservation>,
}

#[derive(Debug, Clone)]
pub struct NcaMetrics {
    pub c_max: f64,
    pub t_max: f64,
    pub auc_last: f64,
    pub lambda_z: Option<f64>,
    pub half_life: Option<f64>,
    pub auc_inf: Option<f64>,
    pub cl_f: Option<f64>,
}

impl PkSubject {
    /// Perform Non-Compartmental Analysis (NCA)
    pub fn compute_nca(&self) -> NcaMetrics {
        if self.observations.is_empty() {
            return NcaMetrics {
                c_max: 0.0,
                t_max: 0.0,
                auc_last: 0.0,
                lambda_z: None,
                half_life: None,
                auc_inf: None,
                cl_f: None,
            };
        }

        // 1. C_max and T_max
        let mut c_max = f64::MIN;
        let mut t_max = 0.0;

        for obs in &self.observations {
            if obs.conc > c_max {
                c_max = obs.conc;
                t_max = obs.time;
            }
        }

        // 2. AUC_last using linear trapezoidal method
        let mut auc_last = 0.0;
        for i in 1..self.observations.len() {
            let t0 = self.observations[i - 1].time;
            let c0 = self.observations[i - 1].conc;
            let t1 = self.observations[i].time;
            let c1 = self.observations[i].conc;

            let dt = t1 - t0;
            if dt > 0.0 {
                auc_last += ((c0 + c1) / 2.0) * dt;
            }
        }

        // 3. Lambda_z (terminal elimination rate) via log-linear regression
        // Use the last 3-4 positive concentration points after T_max
        let terminal_points: Vec<&PkObservation> = self.observations.iter()
            .filter(|obs| obs.time >= t_max && obs.conc > 0.0)
            .collect();

        let (lambda_z, half_life, auc_inf, cl_f) = if terminal_points.len() >= 3 {
            let n = terminal_points.len().min(4);
            let subset = &terminal_points[terminal_points.len() - n..];

            // Linear regression of ln(conc) vs time
            let mut sum_t = 0.0;
            let mut sum_y = 0.0;
            let mut sum_tt = 0.0;
            let mut sum_ty = 0.0;
            let num = subset.len() as f64;

            for p in subset {
                let t = p.time;
                let y = p.conc.ln();
                sum_t += t;
                sum_y += y;
                sum_tt += t * t;
                sum_ty += t * y;
            }

            let denom = num * sum_tt - sum_t * sum_t;
            let slope = if denom.abs() > 1e-12 {
                (num * sum_ty - sum_t * sum_y) / denom
            } else {
                0.0
            };

            let lz = -slope;
            if lz > 0.001 {
                let thalf = (2.0f64).ln() / lz;
                let c_last = self.observations.last().map(|o| o.conc).unwrap_or(0.0);
                let a_inf = auc_last + (c_last / lz);

                // CL/F = Dose / AUC_inf (in (mg/kg)/(mg*h/L) = L/(kg*h))
                let cl = if let Some(dose) = self.dose_mg_kg {
                    if a_inf > 0.0 { Some(dose / a_inf) } else { None }
                } else {
                    None
                };

                (Some(lz), Some(thalf), Some(a_inf), cl)
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

        NcaMetrics {
            c_max,
            t_max,
            auc_last,
            lambda_z,
            half_life,
            auc_inf,
            cl_f,
        }
    }
}

pub struct PkParser;

impl PkParser {
    pub fn parse_csv(content: &str) -> Result<Vec<PkSubject>, ParserError> {
        let mut lines = content.lines();
        let header_line = lines.next().ok_or_else(|| ParserError::InvalidFormat("Empty PK CSV".to_string()))?;
        let headers: Vec<&str> = header_line.split(',').map(|s| s.trim()).collect();

        let mut col_map = HashMap::new();
        for (i, h) in headers.iter().enumerate() {
            col_map.insert(h.to_uppercase(), i);
        }

        let id_col = col_map.get("SUBJECT").or_else(|| col_map.get("ID")).copied()
            .ok_or_else(|| ParserError::TagNotFound("Subject/ID column".to_string()))?;
        let time_col = col_map.get("TIME").copied()
            .ok_or_else(|| ParserError::TagNotFound("Time column".to_string()))?;
        let conc_col = col_map.get("CONC").or_else(|| col_map.get("DV")).copied()
            .ok_or_else(|| ParserError::TagNotFound("Conc/DV column".to_string()))?;

        let wt_col = col_map.get("WT").or_else(|| col_map.get("WEIGHT")).copied();
        let dose_col = col_map.get("DOSE").or_else(|| col_map.get("AMT")).copied();

        let mut subjects: Vec<PkSubject> = Vec::new();
        let mut current_subject: Option<PkSubject> = None;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let cols: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
            if cols.len() <= id_col || cols.len() <= time_col || cols.len() <= conc_col {
                continue;
            }

            let id = cols[id_col].to_string();
            let time = cols[time_col].parse::<f64>().unwrap_or(0.0);
            let conc = cols[conc_col].parse::<f64>().unwrap_or(0.0);
            let wt = wt_col.and_then(|c| cols.get(c)).and_then(|s| s.parse::<f64>().ok());
            let dose = dose_col.and_then(|c| cols.get(c)).and_then(|s| s.parse::<f64>().ok());

            if let Some(cur) = &mut current_subject {
                if cur.id != id {
                    subjects.push(current_subject.take().unwrap());
                    current_subject = Some(PkSubject {
                        id: id.clone(),
                        weight_kg: wt,
                        dose_mg_kg: dose,
                        observations: Vec::new(),
                    });
                }
            } else {
                current_subject = Some(PkSubject {
                    id: id.clone(),
                    weight_kg: wt,
                    dose_mg_kg: dose,
                    observations: Vec::new(),
                });
            }

            if let Some(cur) = &mut current_subject {
                cur.observations.push(PkObservation { time, conc });
            }
        }

        if let Some(cur) = current_subject {
            subjects.push(cur);
        }

        if subjects.is_empty() {
            return Err(ParserError::InvalidFormat("No PK records extracted from CSV".to_string()));
        }

        Ok(subjects)
    }
}
