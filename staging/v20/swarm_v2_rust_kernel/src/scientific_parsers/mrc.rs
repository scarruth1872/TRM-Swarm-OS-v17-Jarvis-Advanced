use super::error::ParserError;

#[derive(Debug, Clone)]
pub struct MrcHeader {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub mode: i32,
    pub mx: i32,
    pub my: i32,
    pub mz: i32,
    pub cell_a: f32,
    pub cell_b: f32,
    pub cell_c: f32,
    pub alpha: f32,
    pub beta: f32,
    pub gamma: f32,
    pub mapc: i32,
    pub mapr: i32,
    pub maps: i32,
    pub d_min: f32,
    pub d_max: f32,
    pub d_mean: f32,
    pub ispg: i32,
    pub extended_header_bytes: usize,
    pub rms: f32,
}

#[derive(Debug)]
pub struct MrcVolume {
    pub header: MrcHeader,
    /// 3D density grid stored in row-major order: total_voxels = nx * ny * nz
    pub data: Vec<f32>,
}

impl MrcVolume {
    /// Retrieve voxel density at 3D integer coordinate (x, y, z)
    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<f32> {
        if x >= self.header.nx || y >= self.header.ny || z >= self.header.nz {
            return None;
        }
        let idx = z * (self.header.nx * self.header.ny) + y * self.header.nx + x;
        self.data.get(idx).copied()
    }

    /// Extract a 2D orthoslice (XY plane) at a given z level
    pub fn slice_xy(&self, z: usize) -> Option<Vec<f32>> {
        if z >= self.header.nz {
            return None;
        }
        let slice_len = self.header.nx * self.header.ny;
        let start = z * slice_len;
        let end = start + slice_len;
        if end <= self.data.len() {
            Some(self.data[start..end].to_vec())
        } else {
            None
        }
    }

    /// Compute statistics of voxels above a given threshold contour level
    pub fn contour_stats(&self, threshold: f32) -> (usize, f32) {
        let mut count = 0;
        let mut sum = 0.0;
        for &val in &self.data {
            if val >= threshold {
                count += 1;
                sum += val;
            }
        }
        let mean = if count > 0 { sum / count as f32 } else { 0.0 };
        (count, mean)
    }
}

pub struct MrcParser;

impl MrcParser {
    pub fn parse(bytes: &[u8]) -> Result<MrcVolume, ParserError> {
        if bytes.len() < 1024 {
            return Err(ParserError::InvalidFormat("File too small for MRC header (< 1024 bytes)".to_string()));
        }

        // Validate MAP magic
        let magic = &bytes[208..212];
        if magic != b"MAP " && magic != b"MAP\0" {
            return Err(ParserError::InvalidFormat(format!("Invalid MRC magic letters: {:?}", String::from_utf8_lossy(magic))));
        }

        let read_i32 = |offset: usize| -> i32 {
            i32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]])
        };

        let read_f32 = |offset: usize| -> f32 {
            f32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]])
        };

        let nx = read_i32(0).max(0) as usize;
        let ny = read_i32(4).max(0) as usize;
        let nz = read_i32(8).max(0) as usize;
        let mode = read_i32(12);

        let mx = read_i32(24);
        let my = read_i32(28);
        let mz = read_i32(32);

        let cell_a = read_f32(40);
        let cell_b = read_f32(44);
        let cell_c = read_f32(48);
        let alpha = read_f32(52);
        let beta = read_f32(56);
        let gamma = read_f32(60);

        let mapc = read_i32(64);
        let mapr = read_i32(68);
        let maps = read_i32(72);

        let d_min = read_f32(76);
        let d_max = read_f32(80);
        let d_mean = read_f32(84);

        let ispg = read_i32(88);
        let extended_header_bytes = read_i32(92).max(0) as usize;
        let rms = read_f32(216);

        let total_voxels = nx * ny * nz;
        let data_start = 1024 + extended_header_bytes;

        if bytes.len() < data_start {
            return Err(ParserError::InvalidFormat("File truncated before data payload".to_string()));
        }

        let payload = &bytes[data_start..];
        let mut data = Vec::with_capacity(total_voxels);

        match mode {
            0 => {
                // 8-bit signed int
                for &b in payload.iter().take(total_voxels) {
                    data.push(b as i8 as f32);
                }
            }
            1 => {
                // 16-bit signed int
                let mut cursor = 0;
                while cursor + 2 <= payload.len() && data.len() < total_voxels {
                    let val = i16::from_le_bytes([payload[cursor], payload[cursor + 1]]) as f32;
                    data.push(val);
                    cursor += 2;
                }
            }
            2 => {
                // 32-bit float
                let mut cursor = 0;
                while cursor + 4 <= payload.len() && data.len() < total_voxels {
                    let val = f32::from_le_bytes([
                        payload[cursor],
                        payload[cursor + 1],
                        payload[cursor + 2],
                        payload[cursor + 3],
                    ]);
                    data.push(val);
                    cursor += 4;
                }
            }
            6 => {
                // 16-bit unsigned int
                let mut cursor = 0;
                while cursor + 2 <= payload.len() && data.len() < total_voxels {
                    let val = u16::from_le_bytes([payload[cursor], payload[cursor + 1]]) as f32;
                    data.push(val);
                    cursor += 2;
                }
            }
            _ => {
                return Err(ParserError::InvalidFormat(format!("Unsupported MRC density mode: {}", mode)));
            }
        }

        Ok(MrcVolume {
            header: MrcHeader {
                nx,
                ny,
                nz,
                mode,
                mx,
                my,
                mz,
                cell_a,
                cell_b,
                cell_c,
                alpha,
                beta,
                gamma,
                mapc,
                mapr,
                maps,
                d_min,
                d_max,
                d_mean,
                ispg,
                extended_header_bytes,
                rms,
            },
            data,
        })
    }
}
