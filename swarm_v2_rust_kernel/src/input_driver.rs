//! TRM Swarm OS v19.001 "Sovereign" — Native Bare-Metal Input & Mouse Subsystem
//! Decodes PS/2 & USB HID scan-codes, tracks mouse cursor coordinates,
//! and dispatches click/keyboard events into the LCARS GUI Compositor.

use crate::lcars_render::{LcarsSurface, LCARS_WHITE, LCARS_GOLD};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    Up,
    Down,
    Left,
    Right,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

pub struct InputManager {
    pub mouse: MouseState,
    pub key_buffer: Vec<KeyCode>,
    pub width: usize,
    pub height: usize,
}

impl InputManager {
    pub fn new(width: usize, height: usize) -> Self {
        InputManager {
            mouse: MouseState {
                x: (width / 2) as i32,
                y: (height / 2) as i32,
                left_button: false,
                right_button: false,
                middle_button: false,
            },
            key_buffer: Vec::with_capacity(64),
            width,
            height,
        }
    }

    /// Process relative mouse movement delta (from PS/2 packet or USB HID report)
    pub fn update_mouse_delta(&mut self, dx: i32, dy: i32, left: bool, right: bool, middle: bool) {
        self.mouse.x = (self.mouse.x + dx).clamp(0, self.width as i32 - 1);
        self.mouse.y = (self.mouse.y - dy).clamp(0, self.height as i32 - 1);
        self.mouse.left_button = left;
        self.mouse.right_button = right;
        self.mouse.middle_button = middle;
    }

    /// Decode PS/2 Set 1 or Set 2 Scan Code
    pub fn push_scan_code(&mut self, scan_code: u8) {
        let key = match scan_code {
            0x1C => KeyCode::Enter,
            0x0E => KeyCode::Backspace,
            0x0F => KeyCode::Tab,
            0x01 => KeyCode::Escape,
            0x48 => KeyCode::Up,
            0x50 => KeyCode::Down,
            0x4B => KeyCode::Left,
            0x4D => KeyCode::Right,
            0x3B => KeyCode::F1,
            0x3C => KeyCode::F2,
            0x3D => KeyCode::F3,
            0x3E => KeyCode::F4,
            0x3F => KeyCode::F5,
            0x40 => KeyCode::F6,
            0x41 => KeyCode::F7,
            0x42 => KeyCode::F8,
            0x43 => KeyCode::F9,
            0x44 => KeyCode::F10,
            0x57 => KeyCode::F11,
            0x58 => KeyCode::F12,
            0x02..=0x0B => {
                let digit = b"1234567890"[(scan_code - 0x02) as usize];
                KeyCode::Char(digit as char)
            },
            0x10..=0x19 => {
                let letter = b"qwertyuiop"[(scan_code - 0x10) as usize];
                KeyCode::Char(letter as char)
            },
            0x1E..=0x26 => {
                let letter = b"asdfghjkl"[(scan_code - 0x1E) as usize];
                KeyCode::Char(letter as char)
            },
            0x2C..=0x32 => {
                let letter = b"zxcvbnm"[(scan_code - 0x2C) as usize];
                KeyCode::Char(letter as char)
            },
            0x39 => KeyCode::Char(' '),
            _ => KeyCode::Unknown(scan_code),
        };
        self.key_buffer.push(key);
    }

    /// Render Star Trek LCARS Precision Crosshair / Pointer Cursor onto surface
    pub fn render_cursor(&self, surface: &mut LcarsSurface) {
        let cx = self.mouse.x as usize;
        let cy = self.mouse.y as usize;
        let cursor_color = if self.mouse.left_button { LCARS_GOLD } else { LCARS_WHITE };

        // Draw crosshair cursor (12x12)
        for i in 0..10 {
            if cx + i < surface.width { surface.set_pixel(cx + i, cy, cursor_color); }
            if cx >= i { surface.set_pixel(cx - i, cy, cursor_color); }
            if cy + i < surface.height { surface.set_pixel(cx, cy + i, cursor_color); }
            if cy >= i { surface.set_pixel(cx, cy - i, cursor_color); }
        }

        // Center dot
        surface.set_pixel(cx, cy, LCARS_GOLD);
    }
}
