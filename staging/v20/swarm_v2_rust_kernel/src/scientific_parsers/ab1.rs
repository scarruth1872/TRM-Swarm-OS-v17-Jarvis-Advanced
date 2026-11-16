use super::error::ParserError;
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct SangerTrace {
    pub dye_order: String,
    pub channel_g: Vec<i16>,
    pub channel_a: Vec<i16>,
    pub channel_t: Vec<i16>,
    pub channel_c: Vec<i16>,
    pub base_calls: String,
    pub peak_locations: Vec<i16>,
    pub quality_scores: Vec<u8>,
    pub trace_length: usize,
}

#[derive(Debug, Clone)]
struct DirEntry {
    name: [u8; 4],
    number: i32,
    element_type: u16,
    num_elements: i32,
    data_size: i32,
    data_offset: i32,
}

impl DirEntry {
    fn is_inline(&self) -> bool {
        self.data_size <= 4
    }

    fn inline_bytes(&self) -> Vec<u8> {
        let bytes = self.data_offset.to_be_bytes();
        let size = self.data_size.max(0) as usize;
        if size <= 4 {
            bytes[..size].to_vec()
        } else {
            vec![]
        }
    }
}

pub struct Ab1Parser;

impl Ab1Parser {
    pub fn parse(bytes: &[u8]) -> Result<SangerTrace, ParserError> {
        let mut cursor = Cursor::new(bytes);

        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        if &magic != b"ABIF" {
            return Err(ParserError::InvalidFormat("Invalid ABIF magic bytes".to_string()));
        }

        // Root entry is at byte offset 6
        cursor.seek(SeekFrom::Start(6))?;
        let root = Self::read_dir_entry(&mut cursor)?;

        cursor.seek(SeekFrom::Start(root.data_offset as u64))?;
        let mut entries: HashMap<(String, i32), DirEntry> = HashMap::new();

        for _ in 0..root.num_elements {
            let entry = Self::read_dir_entry(&mut cursor)?;
            let name = String::from_utf8_lossy(&entry.name).to_string();
            entries.insert((name, entry.number), entry);
        }

        let dye_order = Self::extract_string(&mut cursor, entries.get(&("FWO_".to_string(), 1)))
            .unwrap_or_else(|_| "GATC".to_string());

        let d9 = Self::extract_i16(&mut cursor, entries.get(&("DATA".to_string(), 9)))?;
        let d10 = Self::extract_i16(&mut cursor, entries.get(&("DATA".to_string(), 10)))?;
        let d11 = Self::extract_i16(&mut cursor, entries.get(&("DATA".to_string(), 11)))?;
        let d12 = Self::extract_i16(&mut cursor, entries.get(&("DATA".to_string(), 12)))?;

        let raw_traces = [d9, d10, d11, d12];
        let mut channel_g = Vec::new();
        let mut channel_a = Vec::new();
        let mut channel_t = Vec::new();
        let mut channel_c = Vec::new();

        for (idx, ch) in dye_order.chars().take(4).enumerate() {
            let trace = if idx < raw_traces.len() { raw_traces[idx].clone() } else { vec![] };
            match ch {
                'G' | 'g' => channel_g = trace,
                'A' | 'a' => channel_a = trace,
                'T' | 't' => channel_t = trace,
                'C' | 'c' => channel_c = trace,
                _ => {}
            }
        }

        let base_calls = Self::extract_string(&mut cursor, entries.get(&("PBAS".to_string(), 2)))
            .or_else(|_| Self::extract_string(&mut cursor, entries.get(&("PBAS".to_string(), 1))))?;

        let peak_locations = Self::extract_i16(&mut cursor, entries.get(&("PLOC".to_string(), 2)))
            .or_else(|_| Self::extract_i16(&mut cursor, entries.get(&("PLOC".to_string(), 1))))
            .unwrap_or_default();

        let quality_scores = Self::extract_u8(&mut cursor, entries.get(&("PCON".to_string(), 2)))
            .or_else(|_| Self::extract_u8(&mut cursor, entries.get(&("PCON".to_string(), 1))))
            .unwrap_or_default();

        let trace_length = channel_g.len().max(channel_a.len()).max(channel_t.len()).max(channel_c.len());

        Ok(SangerTrace {
            dye_order,
            channel_g,
            channel_a,
            channel_t,
            channel_c,
            base_calls,
            peak_locations,
            quality_scores,
            trace_length,
        })
    }

    fn read_dir_entry<R: Read>(reader: &mut R) -> Result<DirEntry, ParserError> {
        let mut name = [0u8; 4];
        reader.read_exact(&mut name)?;

        let mut buf4 = [0u8; 4];
        let mut buf2 = [0u8; 2];

        reader.read_exact(&mut buf4)?;
        let number = i32::from_be_bytes(buf4);

        reader.read_exact(&mut buf2)?;
        let element_type = u16::from_be_bytes(buf2);

        reader.read_exact(&mut buf2)?; // skip element_size
        reader.read_exact(&mut buf4)?;
        let num_elements = i32::from_be_bytes(buf4);

        reader.read_exact(&mut buf4)?;
        let data_size = i32::from_be_bytes(buf4);

        reader.read_exact(&mut buf4)?;
        let data_offset = i32::from_be_bytes(buf4);

        let mut reserved = [0u8; 4];
        reader.read_exact(&mut reserved)?;

        Ok(DirEntry {
            name,
            number,
            element_type,
            num_elements,
            data_size,
            data_offset,
        })
    }

    fn extract_i16(cursor: &mut Cursor<&[u8]>, entry: Option<&DirEntry>) -> Result<Vec<i16>, ParserError> {
        let e = entry.ok_or_else(|| ParserError::TagNotFound("DATA/PLOC i16 tag".to_string()))?;
        if e.is_inline() {
            let b = e.inline_bytes();
            if b.len() >= 2 {
                return Ok(vec![i16::from_be_bytes([b[0], b[1]])]);
            }
            return Ok(vec![]);
        }
        cursor.seek(SeekFrom::Start(e.data_offset as u64))?;
        let mut res = Vec::with_capacity(e.num_elements as usize);
        let mut b2 = [0u8; 2];
        for _ in 0..e.num_elements {
            cursor.read_exact(&mut b2)?;
            res.push(i16::from_be_bytes(b2));
        }
        Ok(res)
    }

    fn extract_string(cursor: &mut Cursor<&[u8]>, entry: Option<&DirEntry>) -> Result<String, ParserError> {
        let e = entry.ok_or_else(|| ParserError::TagNotFound("PBAS/FWO_ string tag".to_string()))?;
        let raw = if e.is_inline() {
            e.inline_bytes()
        } else {
            cursor.seek(SeekFrom::Start(e.data_offset as u64))?;
            let mut buf = vec![0u8; e.data_size as usize];
            cursor.read_exact(&mut buf)?;
            buf
        };
        if e.element_type == 18 && !raw.is_empty() {
            let len = raw[0] as usize;
            let actual = len.min(raw.len().saturating_sub(1));
            return Ok(String::from_utf8_lossy(&raw[1..=actual]).to_string());
        }
        Ok(String::from_utf8_lossy(&raw).to_string())
    }

    fn extract_u8(cursor: &mut Cursor<&[u8]>, entry: Option<&DirEntry>) -> Result<Vec<u8>, ParserError> {
        let e = entry.ok_or_else(|| ParserError::TagNotFound("PCON u8 tag".to_string()))?;
        if e.is_inline() {
            return Ok(e.inline_bytes());
        }
        cursor.seek(SeekFrom::Start(e.data_offset as u64))?;
        let mut buf = vec![0u8; e.data_size as usize];
        cursor.read_exact(&mut buf)?;
        Ok(buf)
    }
}
