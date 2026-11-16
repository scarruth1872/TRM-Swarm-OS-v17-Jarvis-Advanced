//! Real text: a license-clean TTF (DejaVu Sans Mono, Bitstream Vera license) rasterized on demand
//! with fontdue. fontdue is the ONE crate this renderer carries; it resolves to a two-node chain
//! (fontdue -> ttf-parser), both pure Rust no_std with no system deps, so the framebuffer still
//! needs no libc beyond musl's own statics and no transitive crate graph to rebuild.
use std::sync::OnceLock;

use fontdue::{Font, FontSettings};

pub const FONT_DATA: &[u8] = include_bytes!("../assets/DejaVuSansMono.ttf");

static FONT: OnceLock<Font> = OnceLock::new();

pub fn get() -> &'static Font {
    FONT.get_or_init(|| match Font::from_bytes(FONT_DATA, FontSettings::default()) {
        Ok(f) => f,
        Err(_) => panic!("embedded font failed to parse"),
    })
}

pub fn ascent(px: f32) -> f32 {
    get()
        .horizontal_line_metrics(px)
        .map(|m| m.ascent)
        .unwrap_or(px * 0.8)
}

pub fn new_line_size(px: f32) -> f32 {
    get()
        .horizontal_line_metrics(px)
        .map(|m| m.new_line_size)
        .unwrap_or(px * 1.2)
}
