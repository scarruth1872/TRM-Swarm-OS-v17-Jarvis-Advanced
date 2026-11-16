use super::error::ParserError;
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DicomTag(pub u16, pub u16);

#[derive(Debug, Clone)]
pub struct DicomElement {
    pub tag: DicomTag,
    pub vr: String,
    pub length: u32,
    pub value_bytes: Vec<u8>,
}

impl DicomElement {
    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.value_bytes).trim_end_matches('\0').trim().to_string()
    }

    pub fn to_u16(&self) -> Option<u16> {
        if self.value_bytes.len() >= 2 {
            Some(u16::from_le_bytes([self.value_bytes[0], self.value_bytes[1]]))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct DicomImage {
    pub patient_name: String,
    pub modality: String,
    pub study_date: String,
    pub rows: u16,
    pub columns: u16,
    pub bits_allocated: u16,
    pub bits_stored: u16,
    pub pixel_data_bytes: usize,
    pub elements_count: usize,
}

pub struct DicomParser;

impl DicomParser {
    pub fn parse(bytes: &[u8]) -> Result<DicomImage, ParserError> {
        let mut cursor = Cursor::new(bytes);

        if bytes.len() < 132 {
            return Err(ParserError::InvalidFormat("DICOM file < 132 bytes".to_string()));
        }

        cursor.seek(SeekFrom::Start(128))?;
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        if &magic != b"DICM" {
            return Err(ParserError::InvalidFormat(format!("Invalid DICM magic bytes: {:?}", magic)));
        }

        let mut elements: HashMap<DicomTag, DicomElement> = HashMap::new();
        let total_len = bytes.len() as u64;

        while cursor.position() < total_len {
            let start_pos = cursor.position();
            if start_pos + 4 > total_len { break; }

            let mut tag_buf = [0u8; 4];
            if cursor.read_exact(&mut tag_buf).is_err() { break; }

            let group = u16::from_le_bytes([tag_buf[0], tag_buf[1]]);
            let element = u16::from_le_bytes([tag_buf[2], tag_buf[3]]);
            let tag = DicomTag(group, element);

            let mut vr_buf = [0u8; 2];
            if cursor.read_exact(&mut vr_buf).is_err() { break; }

            let vr_str = String::from_utf8_lossy(&vr_buf).to_string();
            let is_explicit = Self::is_standard_vr(&vr_str);

            let (length, vr_final) = if is_explicit {
                if vr_str == "OB" || vr_str == "OW" || vr_str == "OF" || vr_str == "SQ" || vr_str == "UT" || vr_str == "UN" {
                    let mut reserved = [0u8; 2];
                    cursor.read_exact(&mut reserved)?;
                    let mut len_buf = [0u8; 4];
                    cursor.read_exact(&mut len_buf)?;
                    (u32::from_le_bytes(len_buf), vr_str)
                } else {
                    let mut len_buf = [0u8; 2];
                    cursor.read_exact(&mut len_buf)?;
                    (u16::from_le_bytes(len_buf) as u32, vr_str)
                }
            } else {
                cursor.seek(SeekFrom::Current(-2))?;
                let mut len_buf = [0u8; 4];
                cursor.read_exact(&mut len_buf)?;
                (u32::from_le_bytes(len_buf), "UN".to_string())
            };

            if length == 0xFFFF_FFFF {
                let remaining = (total_len - cursor.position()) as usize;
                let mut val = vec![0u8; remaining];
                cursor.read_exact(&mut val)?;
                elements.insert(tag, DicomElement { tag, vr: vr_final, length: remaining as u32, value_bytes: val });
                break;
            }

            let read_len = (length as usize).min((total_len - cursor.position()) as usize);
            let mut value_bytes = vec![0u8; read_len];
            cursor.read_exact(&mut value_bytes)?;

            elements.insert(tag, DicomElement { tag, vr: vr_final, length, value_bytes });

            if tag == DicomTag(0x7FE0, 0x0010) {
                break;
            }
        }

        let patient_name = elements.get(&DicomTag(0x0010, 0x0010)).map(|e| e.to_string_lossy()).unwrap_or_else(|| "Anonymous".to_string());
        let modality = elements.get(&DicomTag(0x0008, 0x0060)).map(|e| e.to_string_lossy()).unwrap_or_else(|| "Unknown".to_string());
        let study_date = elements.get(&DicomTag(0x0008, 0x0020)).map(|e| e.to_string_lossy()).unwrap_or_else(|| "Unknown Date".to_string());
        let rows = elements.get(&DicomTag(0x0028, 0x0010)).and_then(|e| e.to_u16()).unwrap_or(0);
        let columns = elements.get(&DicomTag(0x0028, 0x0011)).and_then(|e| e.to_u16()).unwrap_or(0);
        let bits_allocated = elements.get(&DicomTag(0x0028, 0x0100)).and_then(|e| e.to_u16()).unwrap_or(16);
        let bits_stored = elements.get(&DicomTag(0x0028, 0x0101)).and_then(|e| e.to_u16()).unwrap_or(12);
        let pixel_data_bytes = elements.get(&DicomTag(0x7FE0, 0x0010)).map(|e| e.value_bytes.len()).unwrap_or(0);
        let elements_count = elements.len();

        Ok(DicomImage {
            patient_name,
            modality,
            study_date,
            rows,
            columns,
            bits_allocated,
            bits_stored,
            pixel_data_bytes,
            elements_count,
        })
    }

    fn is_standard_vr(vr: &str) -> bool {
        matches!(
            vr,
            "AE" | "AS" | "AT" | "CS" | "DA" | "DS" | "DT" | "FL" | "FD" | "IS"
                | "LO" | "LT" | "OB" | "OD" | "OF" | "OL" | "OW" | "PN" | "SH" | "SL"
                | "SQ" | "SS" | "ST" | "TM" | "UC" | "UI" | "UL" | "UN" | "UR" | "US" | "UT"
        )
    }
}
