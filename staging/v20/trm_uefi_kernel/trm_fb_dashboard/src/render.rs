//! Draw the dashboard onto the framebuffer. LCARS-shaped: a black ground, an amber shoulder, a
//! cyan title rail, and panels whose values come from the platform's own routes -- the same JSON
//! the React frontend fetches, drawn natively so no browser is needed.
use crate::fb::Framebuffer;
use crate::font;

// Palette, from the dashboard's NativeSurface theme.
const BG: (u32, u32, u32) = (10, 14, 20);
const PANEL: (u32, u32, u32) = (17, 24, 35);
const LINE: (u32, u32, u32) = (31, 43, 58);
const CYAN: (u32, u32, u32) = (0, 212, 255);
const GREEN: (u32, u32, u32) = (0, 255, 136);
const AMBER: (u32, u32, u32) = (255, 176, 0);
const VIOLET: (u32, u32, u32) = (123, 97, 255);
const DIM: (u32, u32, u32) = (74, 85, 104);
const TEXT: (u32, u32, u32) = (200, 214, 229);
const DARK: (u32, u32, u32) = (6, 10, 16);

/// What one frame shows, parsed from the platform's routes by main.
pub struct Data {
    pub experts: usize,
    pub engine: [u8; 40],
    pub engine_len: usize,
    pub fabric: u64,
    pub net: [u8; 48],
    pub net_len: usize,
    pub tick: usize,
    pub sel: usize, // 0..=3 selected panel, 4 = none
    pub cx: usize,  // cursor, from the input device
    pub cy: usize,
    pub input: [u8; 64], // the typed line, echoed into the input field
    pub input_len: usize,
    pub answer: [u8; 2048], // the last trm:// answer, shown in the answer area
    pub answer_len: usize,
}

impl Data {
    pub fn new() -> Data {
        Data {
            experts: 0,
            engine: [0u8; 40],
            engine_len: 0,
            fabric: 0,
            net: [0u8; 48],
            net_len: 0,
            tick: 0,
            sel: 4,
            cx: 0,
            cy: 0,
            input: [0u8; 64],
            input_len: 0,
            answer: [0u8; 2048],
            answer_len: 0,
        }
    }
}

/// Draw `s` with its baseline at (x, y) -- fontdue's coordinate, where a glyph's ymin is negative
/// and it rises above the baseline. Returns the x coordinate just past the drawn text.
pub fn text(fb: &mut Framebuffer, x: usize, y: usize, px: f32, r: u32, g: u32, b: u32, s: &str) -> usize {
    let f = font::get();
    let mut cx = x as i32;
    for ch in s.chars() {
        let (m, bitmap) = f.rasterize(ch, px);
        for row in 0..m.height {
            let src = row * m.width;
            for col in 0..m.width {
                let cov = bitmap[src + col];
                if cov > 0 {
                    let gx = cx + m.xmin + col as i32;
                    // fontdue's ymin is the bottom edge of the glyph (negative = below baseline),
                    // and the bitmap starts at the top-left -- so the top of the bitmap sits at
                    // (baseline - ymin - height), and row r is that plus r.
                    let gy = y as i32 - m.ymin - m.height as i32 + row as i32;
                    if gx >= 0 && gy >= 0 {
                        fb.blend_pixel(gx as usize, gy as usize, cov, r, g, b);
                    }
                }
            }
        }
        cx += m.advance_width as i32;
    }
    cx as usize
}

fn width_of(px: f32, s: &str) -> usize {
    let f = font::get();
    let mut w = 0.0f32;
    for ch in s.chars() {
        w += f.metrics(ch, px).advance_width;
    }
    w as usize
}

pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

/// The four panel rectangles, one layout function shared by draw and the click hit-test so the two
/// can never disagree about where a panel is.
pub fn panel_rects(w: usize, h: usize) -> [Rect; 4] {
    let m = 16usize;
    let top = 52usize;
    let gap = 12usize;
    let col_w = (w - 2 * m - gap) / 2;
    let panel_h = (h - top - m - 36) / 2;
    let x2 = m + col_w + gap;
    let y3 = top + panel_h + gap;
    [
        Rect { x: m, y: top, w: col_w, h: panel_h },
        Rect { x: x2, y: top, w: col_w, h: panel_h },
        Rect { x: m, y: y3, w: col_w, h: panel_h },
        Rect { x: x2, y: y3, w: col_w, h: panel_h },
    ]
}

/// The pointer: a small crosshair that reads on any panel.
fn cursor(fb: &mut Framebuffer, cx: usize, cy: usize) {
    for i in 0..9 {
        let x = cx.saturating_sub(4).saturating_add(i);
        let y = cy.saturating_sub(4).saturating_add(i);
        fb.pixel(x, cy, TEXT.0, TEXT.1, TEXT.2);
        fb.pixel(cx, y, TEXT.0, TEXT.1, TEXT.2);
    }
}

fn panel(
    fb: &mut Framebuffer,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    px: f32,
    head: &str,
    head_c: (u32, u32, u32),
) {
    fb.rect(x, y, w, h, PANEL.0, PANEL.1, PANEL.2);
    fb.rect(x, y, w, 2, head_c.0, head_c.1, head_c.2);
    let base = y + 8 + font::ascent(px) as usize;
    let _ = text(fb, x + 8, base, px, head_c.0, head_c.1, head_c.2, head);
}

/// One full frame.
pub fn draw(fb: &mut Framebuffer, d: &Data) {
    fb.clear(BG.0, BG.1, BG.2);
    let w = fb.width;
    let h = fb.height;
    // Font sizes per resolution (pixels). 1280x800 is QEMU, 1920x1080 is the metal panel.
    let (px_title, px_head, px_body, px_small) = if w >= 1280 {
        (24.0f32, 20.0f32, 18.0f32, 15.0f32)
    } else {
        (18.0f32, 15.0f32, 14.0f32, 12.0f32)
    };
    let rects = panel_rects(w, h);

    // Amber shoulder + cyan title rail (36px tall at every resolution).
    fb.rect(0, 0, 40, h, AMBER.0, AMBER.1, AMBER.2);
    fb.rect(40, 0, w - 40, 36, CYAN.0, CYAN.1, CYAN.2);
    let title_base = ((36.0 - font::new_line_size(px_title)) / 2.0 + font::ascent(px_title)) as usize;
    let _ = text(fb, 56, title_base, px_title, DARK.0, DARK.1, DARK.2, "TRM SWARM OS");

    let title_right = "NATIVE / RUST";
    let tw = width_of(px_title, title_right);
    let _ = text(fb, w - tw - 16, title_base, px_title, DARK.0, DARK.1, DARK.2, title_right);

    // The four panels, one layout shared with the click hit-test in main.
    let heads = ["PLATFORM", "EXPERTS", "K3 ENGINE", "FABRIC"];
    let colors = [CYAN, GREEN, VIOLET, AMBER];
    for i in 0..4 {
        let r = &rects[i];
        panel(fb, r.x, r.y, r.w, r.h, px_head, heads[i], colors[i]);
    }
    let va = font::ascent(px_body) as usize;
    let _ = text(fb, rects[0].x + 12, rects[0].y + 48 + va, px_body, GREEN.0, GREEN.1, GREEN.2, "state: UP");
    let line = format!("{} ONLINE", d.experts);
    let _ = text(fb, rects[1].x + 12, rects[1].y + 48 + va, px_body, TEXT.0, TEXT.1, TEXT.2, &line);
    let engine = core::str::from_utf8(&d.engine[..d.engine_len]).unwrap_or("?");
    let _ = text(fb, rects[2].x + 12, rects[2].y + 48 + va, px_body, TEXT.0, TEXT.1, TEXT.2, engine);
    // The FABRIC panel doubles as the answer surface: the query's trm:// result is what the
    // operator came to read, so it leads; the fabric byte count follows in the dim colour.
    let fabric_line = format!("{} B fabric", d.fabric);
    let _ = text(fb, rects[3].x + 12, rects[3].y + 48 + va, px_body, DIM.0, DIM.1, DIM.2, &fabric_line);
    let answer = core::str::from_utf8(&d.answer[..d.answer_len]).unwrap_or("");
    if !answer.is_empty() {
        // Wrap the reply across the panel: split on the answer's own newlines, clip each line to
        // the panel width on a character boundary, and stop at the panel's bottom edge.
        let max_chars = (rects[3].w - 24) / 10;
        let mut y = rects[3].y + 72 + va;
        for line in answer.lines().take(6) {
            if y >= rects[3].y + rects[3].h - 8 {
                break;
            }
            let end = line
                .char_indices()
                .nth(max_chars)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
            let _ = text(fb, rects[3].x + 12, y, px_body, GREEN.0, GREEN.1, GREEN.2, &line[..end]);
            y += font::new_line_size(px_body) as usize;
        }
    }

    // Selection highlight: a bright border on the chosen panel.
    if d.sel < 4 {
        let r = &rects[d.sel];
        fb.rect(r.x, r.y, r.w, 2, CYAN.0, CYAN.1, CYAN.2);
        fb.rect(r.x, r.y + r.h - 2, r.w, 2, CYAN.0, CYAN.1, CYAN.2);
        fb.rect(r.x, r.y, 2, r.h, CYAN.0, CYAN.1, CYAN.2);
        fb.rect(r.x + r.w - 2, r.y, 2, r.h, CYAN.0, CYAN.1, CYAN.2);
    }

    // Net line + selection hint + live tick across the bottom.
    let by = h - 32;
    fb.rect(40, by - 8, w - 40, 2, LINE.0, LINE.1, LINE.2);
    let sa = font::ascent(px_small) as usize;
    let net = core::str::from_utf8(&d.net[..d.net_len]).unwrap_or("no interface");
    let _ = text(fb, 56, by + sa, px_small, DIM.0, DIM.1, DIM.2, net);
    // The input field: a prompt + the typed line + a blinking cursor, on the status bar's second
    // line. Typing lands here as the keyboard works -- the text field the dashboard was missing.
    let input = core::str::from_utf8(&d.input[..d.input_len]).unwrap_or("");
    let _ = text(fb, 56, by + 16 + sa, px_small, CYAN.0, CYAN.1, CYAN.2, "QUERY> ");
    let tx = 56 + width_of(px_small, "QUERY> ");
    let _ = text(fb, tx, by + 16 + sa, px_small, TEXT.0, TEXT.1, TEXT.2, input);
    if d.tick & 1 == 0 {
        let _ = text(fb, tx + width_of(px_small, input), by + 16 + sa, px_small, CYAN.0, CYAN.1, CYAN.2, "_");
    }
    let tick = format!("FRAME {}", d.tick);
    let tkw = width_of(px_small, &tick);
    let _ = text(fb, w - tkw - 16, by + sa, px_small, CYAN.0, CYAN.1, CYAN.2, &tick);

    // The cursor, last so it sits on top of everything.
    cursor(fb, d.cx, d.cy);
}
