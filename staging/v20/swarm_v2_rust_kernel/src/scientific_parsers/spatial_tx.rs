use super::error::ParserError;

#[derive(Debug, Clone)]
pub struct SpatialSpot {
    pub barcode: String,
    pub in_tissue: bool,
    pub array_row: i32,
    pub array_col: i32,
    pub pxl_row_in_fullres: i32,
    pub pxl_col_in_fullres: i32,
}

#[derive(Debug, Clone)]
pub struct SpatialTranscriptomicsData {
    pub total_spots: usize,
    pub in_tissue_spots: usize,
    pub spots: Vec<SpatialSpot>,
    pub spot_diameter_fullres: f64,
}

impl SpatialTranscriptomicsData {
    pub fn bounding_box(&self) -> (i32, i32, i32, i32) {
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;

        for s in &self.spots {
            if s.in_tissue {
                min_x = min_x.min(s.pxl_col_in_fullres);
                max_x = max_x.max(s.pxl_col_in_fullres);
                min_y = min_y.min(s.pxl_row_in_fullres);
                max_y = max_y.max(s.pxl_row_in_fullres);
            }
        }
        if min_x == i32::MAX { (0, 0, 0, 0) } else { (min_x, max_x, min_y, max_y) }
    }
}

pub struct SpatialParser;

impl SpatialParser {
    /// Parses 10x Genomics Visium tissue_positions_list (.csv)
    /// Format: barcode,in_tissue,array_row,array_col,pxl_row_in_fullres,pxl_col_in_fullres
    pub fn parse_visium_csv(content: &str, spot_diameter: f64) -> Result<SpatialTranscriptomicsData, ParserError> {
        let mut spots = Vec::new();
        let mut in_tissue_count = 0;

        for (line_idx, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() { continue; }

            // Skip header if present
            if line_idx == 0 && (line.starts_with("barcode") || line.starts_with("spot_id")) {
                continue;
            }

            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() < 6 {
                continue;
            }

            let barcode = parts[0].to_string();
            let in_tissue = parts[1] == "1" || parts[1].eq_ignore_ascii_case("true");
            let array_row = parts[2].parse::<i32>().unwrap_or(0);
            let array_col = parts[3].parse::<i32>().unwrap_or(0);
            let pxl_row_in_fullres = parts[4].parse::<i32>().unwrap_or(0);
            let pxl_col_in_fullres = parts[5].parse::<i32>().unwrap_or(0);

            if in_tissue {
                in_tissue_count += 1;
            }

            spots.push(SpatialSpot {
                barcode,
                in_tissue,
                array_row,
                array_col,
                pxl_row_in_fullres,
                pxl_col_in_fullres,
            });
        }

        if spots.is_empty() {
            return Err(ParserError::InvalidFormat("No spatial spots found in CSV".to_string()));
        }

        Ok(SpatialTranscriptomicsData {
            total_spots: spots.len(),
            in_tissue_spots: in_tissue_count,
            spots,
            spot_diameter_fullres: spot_diameter,
        })
    }
}
