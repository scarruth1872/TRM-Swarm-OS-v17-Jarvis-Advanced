use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ParserError {
    Io(io::Error),
    InvalidFormat(String),
    TagNotFound(String),
    XmlError(String),
    JsonError(String),
    DecompressionError(String),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::Io(err) => write!(f, "I/O error: {}", err),
            ParserError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParserError::TagNotFound(tag) => write!(f, "Required tag/field not found: {}", tag),
            ParserError::XmlError(msg) => write!(f, "XML syntax/streaming error: {}", msg),
            ParserError::JsonError(msg) => write!(f, "JSON deserialization error: {}", msg),
            ParserError::DecompressionError(msg) => write!(f, "Binary decompression error: {}", msg),
        }
    }
}

impl std::error::Error for ParserError {}

impl From<io::Error> for ParserError {
    fn from(err: io::Error) -> Self {
        ParserError::Io(err)
    }
}

impl From<serde_json::Error> for ParserError {
    fn from(err: serde_json::Error) -> Self {
        ParserError::JsonError(err.to_string())
    }
}
