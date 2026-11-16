use super::error::ParserError;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use flate2::read::ZlibDecoder;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct Peak {
    pub mz: f64,
    pub intensity: f64,
}

#[derive(Debug, Clone)]
pub struct Spectrum {
    pub id: String,
    pub index: usize,
    pub ms_level: u8,
    pub retention_time: Option<f64>,
    pub peaks: Vec<Peak>,
}

#[derive(Debug, PartialEq)]
enum ArrayKind {
    Mz,
    Intensity,
    Unknown,
}

#[derive(Debug, PartialEq)]
enum Precision {
    Float32,
    Float64,
}

#[derive(Debug, PartialEq)]
enum Compression {
    None,
    Zlib,
}

pub struct MzmlParser;

impl MzmlParser {
    pub fn parse(xml_content: &str) -> Result<Vec<Spectrum>, ParserError> {
        let mut reader = Reader::from_str(xml_content);
        reader.config_mut().trim_text(true);

        let mut spectra = Vec::new();
        let mut in_spectrum = false;
        let mut in_binary_data_array = false;
        let mut in_binary = false;

        let mut current_id = String::new();
        let mut current_index = 0usize;
        let mut current_ms_level = 1u8;
        let mut current_rt: Option<f64> = None;

        let mut current_array_kind = ArrayKind::Unknown;
        let mut current_precision = Precision::Float64;
        let mut current_compression = Compression::None;

        let mut mz_array: Vec<f64> = Vec::new();
        let mut intensity_array: Vec<f64> = Vec::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = e.name();
                    if name.as_ref() == b"spectrum" {
                        in_spectrum = true;
                        mz_array.clear();
                        intensity_array.clear();
                        current_id = String::new();
                        current_index = 0;
                        current_ms_level = 1;
                        current_rt = None;

                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                current_id = String::from_utf8_lossy(&attr.value).to_string();
                            } else if attr.key.as_ref() == b"index" {
                                current_index = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0);
                            }
                        }
                    } else if name.as_ref() == b"binaryDataArray" {
                        in_binary_data_array = true;
                        current_array_kind = ArrayKind::Unknown;
                        current_precision = Precision::Float64;
                        current_compression = Compression::None;
                    } else if name.as_ref() == b"binary" {
                        in_binary = true;
                    }
                }
                Ok(Event::Empty(e)) => {
                    let name = e.name();
                    if name.as_ref() == b"cvParam" {
                        let mut param_name = String::new();
                        let mut param_value = String::new();

                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                param_name = String::from_utf8_lossy(&attr.value).to_string();
                            } else if attr.key.as_ref() == b"value" {
                                param_value = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }

                        if in_binary_data_array {
                            if param_name == "m/z array" {
                                current_array_kind = ArrayKind::Mz;
                            } else if param_name == "intensity array" {
                                current_array_kind = ArrayKind::Intensity;
                            } else if param_name == "64-bit float" {
                                current_precision = Precision::Float64;
                            } else if param_name == "32-bit float" {
                                current_precision = Precision::Float32;
                            } else if param_name == "zlib compression" {
                                current_compression = Compression::Zlib;
                            } else if param_name == "no compression" {
                                current_compression = Compression::None;
                            }
                        } else if in_spectrum {
                            if param_name == "ms level" {
                                current_ms_level = param_value.parse().unwrap_or(1);
                            } else if param_name == "scan start time" {
                                current_rt = param_value.parse().ok();
                            }
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_binary {
                        let raw_base64 = e.unescape().map_err(|err| ParserError::XmlError(err.to_string()))?;
                        let cleaned = raw_base64.trim().replace('\n', "").replace('\r', "");

                        if let Ok(decoded) = BASE64.decode(cleaned.as_bytes()) {
                            let uncompressed = if current_compression == Compression::Zlib {
                                let mut decoder = ZlibDecoder::new(&decoded[..]);
                                let mut decompressed = Vec::new();
                                decoder.read_to_end(&mut decompressed).map_err(|err| ParserError::DecompressionError(err.to_string()))?;
                                decompressed
                            } else {
                                decoded
                            };

                            let floats = Self::decode_floats(&uncompressed, &current_precision);
                            match current_array_kind {
                                ArrayKind::Mz => mz_array = floats,
                                ArrayKind::Intensity => intensity_array = floats,
                                ArrayKind::Unknown => {}
                            }
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    let name = e.name();
                    if name.as_ref() == b"binary" {
                        in_binary = false;
                    } else if name.as_ref() == b"binaryDataArray" {
                        in_binary_data_array = false;
                    } else if name.as_ref() == b"spectrum" {
                        in_spectrum = false;
                        let count = mz_array.len().min(intensity_array.len());
                        let mut peaks = Vec::with_capacity(count);
                        for i in 0..count {
                            peaks.push(Peak { mz: mz_array[i], intensity: intensity_array[i] });
                        }

                        spectra.push(Spectrum {
                            id: current_id.clone(),
                            index: current_index,
                            ms_level: current_ms_level,
                            retention_time: current_rt,
                            peaks,
                        });
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ParserError::XmlError(format!("XML error at pos {}: {:?}", reader.buffer_position(), e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(spectra)
    }

    fn decode_floats(bytes: &[u8], precision: &Precision) -> Vec<f64> {
        match precision {
            Precision::Float64 => {
                let chunks = bytes.len() / 8;
                let mut res = Vec::with_capacity(chunks);
                for i in 0..chunks {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
                    res.push(f64::from_le_bytes(b));
                }
                res
            }
            Precision::Float32 => {
                let chunks = bytes.len() / 4;
                let mut res = Vec::with_capacity(chunks);
                for i in 0..chunks {
                    let mut b = [0u8; 4];
                    b.copy_from_slice(&bytes[i * 4..(i + 1) * 4]);
                    res.push(f32::from_le_bytes(b) as f64);
                }
                res
            }
        }
    }
}
