//! Read the input devices the boot enumerated into /dev/input/event* -- the evdev stream -- and
//! turn them into a cursor, a click, and a typed line. This uses the `evdev` crate instead of
//! hand-parsing the 24-byte struct: it is the mature reimplementation of libevdev (no C .so), and
//! it handles what a hand-rolled reader misses -- ABS_MT multi-touch, SYN_DROPPED recovery, and
//! device names via EVIOCGNAME (so the boot log now says WHICH devices it opened, not just how many).
use evdev::{AbsoluteAxisCode, Device, EventSummary, KeyCode, RelativeAxisCode};

/// How far an arrow key moves the cursor when there is no mouse.
const STEP: i32 = 40;

pub struct Input {
    devs: Vec<Device>,
    names: Vec<String>,
    pub cx: i32,
    pub cy: i32,
    /// A left-click (or empty-Enter) landed this frame, at the cursor position.
    pub click: bool,
    /// Enter landed this frame with a non-empty line -- the typed text is ready to submit.
    pub submit: bool,
    /// The typed line, one keystroke at a time, echoed to the input field.
    pub text: String,
    shift: bool,
    w: i32,
    h: i32,
}

/// A US-layout keymap: a keycode plus the shift flag becomes a character, or None when the key is
/// not text (arrows, Enter, Escape, ...). This is the one place the renderer maps a raw keycode to
/// a glyph -- the renderer's whole job is drawing, so the map stays here beside the reader.
fn key_to_char(code: KeyCode, shift: bool) -> Option<char> {
    let c = match code {
        KeyCode::KEY_A => if shift { 'A' } else { 'a' },
        KeyCode::KEY_B => if shift { 'B' } else { 'b' },
        KeyCode::KEY_C => if shift { 'C' } else { 'c' },
        KeyCode::KEY_D => if shift { 'D' } else { 'd' },
        KeyCode::KEY_E => if shift { 'E' } else { 'e' },
        KeyCode::KEY_F => if shift { 'F' } else { 'f' },
        KeyCode::KEY_G => if shift { 'G' } else { 'g' },
        KeyCode::KEY_H => if shift { 'H' } else { 'h' },
        KeyCode::KEY_I => if shift { 'I' } else { 'i' },
        KeyCode::KEY_J => if shift { 'J' } else { 'j' },
        KeyCode::KEY_K => if shift { 'K' } else { 'k' },
        KeyCode::KEY_L => if shift { 'L' } else { 'l' },
        KeyCode::KEY_M => if shift { 'M' } else { 'm' },
        KeyCode::KEY_N => if shift { 'N' } else { 'n' },
        KeyCode::KEY_O => if shift { 'O' } else { 'o' },
        KeyCode::KEY_P => if shift { 'P' } else { 'p' },
        KeyCode::KEY_Q => if shift { 'Q' } else { 'q' },
        KeyCode::KEY_R => if shift { 'R' } else { 'r' },
        KeyCode::KEY_S => if shift { 'S' } else { 's' },
        KeyCode::KEY_T => if shift { 'T' } else { 't' },
        KeyCode::KEY_U => if shift { 'U' } else { 'u' },
        KeyCode::KEY_V => if shift { 'V' } else { 'v' },
        KeyCode::KEY_W => if shift { 'W' } else { 'w' },
        KeyCode::KEY_X => if shift { 'X' } else { 'x' },
        KeyCode::KEY_Y => if shift { 'Y' } else { 'y' },
        KeyCode::KEY_Z => if shift { 'Z' } else { 'z' },
        KeyCode::KEY_0 => if shift { ')' } else { '0' },
        KeyCode::KEY_1 => if shift { '!' } else { '1' },
        KeyCode::KEY_2 => if shift { '@' } else { '2' },
        KeyCode::KEY_3 => if shift { '#' } else { '3' },
        KeyCode::KEY_4 => if shift { '$' } else { '4' },
        KeyCode::KEY_5 => if shift { '%' } else { '5' },
        KeyCode::KEY_6 => if shift { '^' } else { '6' },
        KeyCode::KEY_7 => if shift { '&' } else { '7' },
        KeyCode::KEY_8 => if shift { '*' } else { '8' },
        KeyCode::KEY_9 => if shift { '(' } else { '9' },
        KeyCode::KEY_SPACE => ' ',
        KeyCode::KEY_MINUS => if shift { '_' } else { '-' },
        KeyCode::KEY_EQUAL => if shift { '+' } else { '=' },
        KeyCode::KEY_SLASH => if shift { '?' } else { '/' },
        KeyCode::KEY_DOT => if shift { '>' } else { '.' },
        KeyCode::KEY_COMMA => if shift { '<' } else { ',' },
        KeyCode::KEY_SEMICOLON => if shift { ':' } else { ';' },
        KeyCode::KEY_APOSTROPHE => if shift { '"' } else { '\'' },
        KeyCode::KEY_LEFTBRACE => if shift { '{' } else { '[' },
        KeyCode::KEY_RIGHTBRACE => if shift { '}' } else { ']' },
        KeyCode::KEY_BACKSLASH => if shift { '|' } else { '\\' },
        KeyCode::KEY_GRAVE => if shift { '~' } else { '`' },
        _ => return None,
    };
    Some(c)
}

impl Input {
    pub fn open(w: i32, h: i32) -> Input {
        let mut devs = Vec::new();
        let mut names = Vec::new();
        for i in 0..8 {
            let path = format!("/dev/input/event{}", i);
            if let Ok(d) = Device::open(&path) {
                let name = d.name().unwrap_or("(unnamed)").to_string();
                // Non-blocking so a device with no pending events cannot stall the dashboard the
                // way a stuck platform can; every receive is bounded.
                let _ = d.set_nonblocking(true);
                names.push(name);
                devs.push(d);
            }
        }
        Input {
            devs,
            names,
            cx: w / 2,
            cy: h / 2,
            click: false,
            submit: false,
            text: String::new(),
            shift: false,
            w,
            h,
        }
    }

    pub fn count(&self) -> usize {
        self.devs.len()
    }

    /// The device names, in open order -- logged once at boot so a later read of the log proves
    /// which of the enumerated devices actually showed up (PS/2 vs I2C-HID, keyboard vs touchpad).
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Drain every pending event from every device. The one-shot `click` and `submit` flags are
    /// cleared at the start of each poll, so a held button or key does not re-fire across frames.
    pub fn poll(&mut self) {
        self.click = false;
        self.submit = false;
        for dev in &mut self.devs {
            if let Ok(events) = dev.fetch_events() {
                for ev in events {
                    match ev.destructure() {
                        EventSummary::RelativeAxis(_, RelativeAxisCode::REL_X, v) => self.cx += v,
                        EventSummary::RelativeAxis(_, RelativeAxisCode::REL_Y, v) => self.cy += v,
                        // Single-touch absolute (touchscreen) and multi-touch (trackpad) both
                        // land here: the latest reported position is the cursor.
                        EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_X, v) => self.cx = v,
                        EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_Y, v) => self.cy = v,
                        EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_MT_POSITION_X, v) => {
                            self.cx = v
                        }
                        EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_MT_POSITION_Y, v) => {
                            self.cy = v
                        }
                        EventSummary::Key(_, KeyCode::BTN_LEFT | KeyCode::BTN_TOUCH, 1) => {
                            self.click = true
                        }
                        EventSummary::Key(_, KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT, v) => {
                            self.shift = v != 0;
                        }
                        EventSummary::Key(_, KeyCode::KEY_BACKSPACE, 1) => {
                            self.text.pop();
                        }
                        EventSummary::Key(_, KeyCode::KEY_ENTER, 1) => {
                            if self.text.is_empty() {
                                // Empty Enter keeps the old behaviour: select the panel under the
                                // cursor. A non-empty line is a submission instead.
                                self.click = true;
                            } else {
                                self.submit = true;
                            }
                        }
                        EventSummary::Key(_, KeyCode::KEY_UP, 1) => self.cy -= STEP,
                        EventSummary::Key(_, KeyCode::KEY_DOWN, 1) => self.cy += STEP,
                        EventSummary::Key(_, KeyCode::KEY_LEFT, 1) => self.cx -= STEP,
                        EventSummary::Key(_, KeyCode::KEY_RIGHT, 1) => self.cx += STEP,
                        EventSummary::Key(_, code, 1) => {
                            if let Some(c) = key_to_char(code, self.shift) {
                                if self.text.len() < 63 {
                                    self.text.push(c);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        self.cx = self.cx.clamp(0, self.w - 1);
        self.cy = self.cy.clamp(0, self.h - 1);
    }
}
