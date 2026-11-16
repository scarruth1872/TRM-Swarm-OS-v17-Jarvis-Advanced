//! Framebuffer-only console for output AFTER ExitBootServices.
//!
//! Once boot services are gone there is no ConOut, no stdout, no way to print.
//! The framebuffer is ordinary memory though, and stays mapped, so writing glyphs
//! straight into it keeps the platform able to report what it is doing.
//!
//! Capture the framebuffer geometry BEFORE exiting; after exiting we must not
//! call any GOP protocol function again.

use crate::font::get_glyph;

/// Captured GOP framebuffer geometry.
#[derive(Clone, Copy)]
pub struct FbInfo {
    pub base: *mut u8,
    pub width: usize,
    pub height: usize,
    pub stride_px: usize,
    pub px_bytes: usize,
}

/// Minimal console that writes pixels directly, no boot services involved.
pub struct FbConsole {
    pub info: FbInfo,
    pub cx: usize,
    pub cy: usize,
    pub fg: u32,
    pub scale: usize,
}

impl FbConsole {
    pub fn new(info: FbInfo, fg: u32) -> Self {
        FbConsole { info, cx: 0, cy: 0, fg, scale: 1 }
    }

    /// Write one pixel as 0x00RRGGBB. Bounds-checked: a bad stride or a cursor
    /// past the edge must not scribble outside the framebuffer.
    #[inline]
    pub unsafe fn put_pixel(&mut self, x: usize, y: usize, rgb: u32) {
        if x >= self.info.width || y >= self.info.height { return; }
        let off = (y * self.info.stride_px + x) * self.info.px_bytes;
        let p = self.info.base.add(off);
        // GOP BGRx: byte order B,G,R,X
        *p = (rgb & 0xFF) as u8;
        *p.add(1) = ((rgb >> 8) & 0xFF) as u8;
        *p.add(2) = ((rgb >> 16) & 0xFF) as u8;
        if self.info.px_bytes >= 4 { *p.add(3) = 0xFF; }
    }

    /// Draw one glyph at the cursor, advancing it.
    pub unsafe fn draw_char(&mut self, c: u8) {
        if c == b'\n' { self.cx = 0; self.cy += 16 * self.scale; return; }
        let glyph = get_glyph(c);
        let s = self.scale;
        let mut row = 0;
        while row < 16 {
            let bits = glyph[row];
            let mut col = 0;
            while col < 8 {
                if (bits >> (7 - col)) & 1 == 1 {
                    let mut dy = 0;
                    while dy < s {
                        let mut dx = 0;
                        while dx < s {
                            self.put_pixel(self.cx + col * s + dx, self.cy + row * s + dy, self.fg);
                            dx += 1;
                        }
                        dy += 1;
                    }
                }
                col += 1;
            }
            row += 1;
        }
        self.cx += 8 * s;
        if self.cx + 8 * s >= self.info.width { self.cx = 0; self.cy += 16 * s; }
    }

    /// Write a string. Soft-wraps and stops at the bottom edge.
    pub unsafe fn write_str(&mut self, s: &str) {
        for b in s.as_bytes() {
            if self.cy + 16 * self.scale >= self.info.height { break; }
            self.draw_char(*b);
        }
    }
}
