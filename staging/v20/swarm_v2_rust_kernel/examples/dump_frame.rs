//! dump_frame.rs — Render one LCARS frame through the real Rust compositor
//! and dump the framebuffer to a BMP for verification against Antigravity's renders.
use swarm_rust_kernel::lcars_render::{LcarsSurface, LcarsTab, render_lcars_desktop};
use swarm_rust_kernel::council::{NativeSwarmCouncil, ProposalCategory};
use std::fs::File;
use std::io::{BufWriter, Write};

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

fn write_bmp(path: &str, pixels: &[u32], w: usize, h: usize) -> std::io::Result<()> {
    let row_size = (w * 3 + 3) & !3; // 24bpp, 4-byte row alignment
    let data_size = row_size * h;
    let file_size = 54 + data_size;
    let mut f = BufWriter::new(File::create(path)?);

    // BMP header
    f.write_all(b"BM")?;
    f.write_all(&(file_size as u32).to_le_bytes())?;
    f.write_all(&0u16.to_le_bytes())?;
    f.write_all(&0u16.to_le_bytes())?;
    f.write_all(&54u32.to_le_bytes())?;
    // DIB header (BITMAPINFOHEADER, 40 bytes)
    f.write_all(&40u32.to_le_bytes())?;
    f.write_all(&(w as i32).to_le_bytes())?;
    f.write_all(&(h as i32).to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&24u16.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&(data_size as u32).to_le_bytes())?;
    f.write_all(&2835i32.to_le_bytes())?;
    f.write_all(&2835i32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;

    // Pixel data — bottom-up, BGRA -> BGR
    let mut row = vec![0u8; row_size];
    for y in (0..h).rev() {
        for x in 0..w {
            let c = pixels[y * w + x];
            row[x * 3] = (c & 0xFF) as u8;         // B
            row[x * 3 + 1] = ((c >> 8) & 0xFF) as u8; // G
            row[x * 3 + 2] = ((c >> 16) & 0xFF) as u8; // R
        }
        f.write_all(&row)?;
    }
    f.flush()?;
    Ok(())
}

fn main() {
    let tab = LcarsTab::NeuralBridge; // Sector 02 — matches Antigravity's uefi_sector02_jarvis.bmp
    let mut surface = LcarsSurface::new(WIDTH, HEIGHT);

    let proposal = NativeSwarmCouncil::evaluate(
        "Sovereign Sub-THz Sensing Grid Activation",
        ProposalCategory::Communication,
        "Shawn (Operator)",
    );

    render_lcars_desktop(&mut surface, 0.0, tab, Some(&proposal));

    write_bmp("rust_compositor_sector02.bmp", &surface.pixels, WIDTH, HEIGHT)
        .expect("failed to write BMP");

    // Stats for the comparison
    use std::collections::HashMap;
    let mut counts: HashMap<u32, usize> = HashMap::new();
    for &p in &surface.pixels {
        *counts.entry(p).or_insert(0) += 1;
    }
    let mut sorted: Vec<(u32, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    println!("rust_compositor_sector02.bmp written ({}x{})", WIDTH, HEIGHT);
    println!("top colors (BGRA as u32):");
    for (c, n) in sorted.iter().take(8) {
        let b = c & 0xFF;
        let g = (c >> 8) & 0xFF;
        let r = (c >> 16) & 0xFF;
        println!("  #{:02X}{:02X}{:02X} x{}", r, g, b, n);
    }
    println!("total distinct colors: {}", sorted.len());
}
