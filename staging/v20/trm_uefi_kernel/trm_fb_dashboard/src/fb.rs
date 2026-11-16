//! The framebuffer the firmware handed the kernel: open /dev/fb0, read the geometry it actually
//! has, map it, and write pixels through the bitfield offsets the mode reports -- no assumed pixel
//! layout, because the one assumption this pathway keeps paying for is the assumption.
use crate::sys;

const FBIOGET_VSCREENINFO: usize = 0x4600;
const FBIOGET_FSCREENINFO: usize = 0x4602;
const PROT_READ: i32 = 1;
const PROT_WRITE: i32 = 2;
const MAP_SHARED: i32 = 1;

#[repr(C)]
#[derive(Default)]
struct Bitfield {
    offset: u32,
    length: u32,
    msb_right: u32,
}

#[repr(C)]
#[derive(Default)]
struct Var {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: Bitfield,
    green: Bitfield,
    blue: Bitfield,
    transp: Bitfield,
    nonstd: u32,
    activate: u32,
    height: u32,
    width: u32,
    accel_flags: u32,
    pixclock: u32,
    left_margin: u32,
    right_margin: u32,
    upper_margin: u32,
    lower_margin: u32,
    hsync_len: u32,
    vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    reserved: [u32; 4],
}

#[repr(C)]
#[derive(Default)]
struct Fix {
    id: [u8; 16],
    smem_start: u64,
    smem_len: u32,
    type_: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
    mmio_start: u64,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

pub struct Framebuffer {
    map: *mut u8,
    map_len: usize,
    pub width: usize,
    pub height: usize,
    stride: usize,
    bpp: u32,
    r_off: u32,
    g_off: u32,
    b_off: u32,
    r_len: u32,
    g_len: u32,
    b_len: u32,
}

impl Framebuffer {
    pub fn open() -> Option<Framebuffer> {
        let fd = sys::open(b"/dev/fb0\0", sys::O_RDWR).ok()?;
        let mut var: Var = Default::default();
        if sys::ioctl(fd, FBIOGET_VSCREENINFO, &mut var as *mut Var as *mut u8).is_err() {
            sys::close(fd);
            return None;
        }
        let mut fix: Fix = Default::default();
        if sys::ioctl(fd, FBIOGET_FSCREENINFO, &mut fix as *mut Fix as *mut u8).is_err() {
            sys::close(fd);
            return None;
        }
        let map = sys::mmap(
            std::ptr::null_mut(),
            fix.smem_len as usize,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd,
            0,
        );
        sys::close(fd); // the mapping outlives the descriptor
        if map as usize == usize::MAX {
            return None;
        }
        Some(Framebuffer {
            map,
            map_len: fix.smem_len as usize,
            width: var.xres as usize,
            height: var.yres as usize,
            stride: fix.line_length as usize,
            bpp: var.bits_per_pixel,
            r_off: var.red.offset,
            g_off: var.green.offset,
            b_off: var.blue.offset,
            r_len: var.red.length,
            g_len: var.green.length,
            b_len: var.blue.length,
        })
    }

    pub fn describe(&self) -> String {
        format!("{}x{} {}bpp line {}", self.width, self.height, self.bpp, self.stride)
    }

    #[inline]
    pub fn pixel(&mut self, x: usize, y: usize, r: u32, g: u32, b: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let val = (r << self.r_off) | (g << self.g_off) | (b << self.b_off);
        let off = y * self.stride + x * (self.bpp as usize / 8);
        unsafe {
            let p = self.map.add(off);
            match self.bpp {
                32 => *(p as *mut u32) = val,
                16 => *(p as *mut u16) = (val & 0xffff) as u16,
                _ => {}
            }
        }
    }

    #[inline]
    fn read_pixel(&self, x: usize, y: usize) -> (u32, u32, u32) {
        if x >= self.width || y >= self.height {
            return (0, 0, 0);
        }
        let off = y * self.stride + x * (self.bpp as usize / 8);
        unsafe {
            let val = match self.bpp {
                32 => *(self.map.add(off) as *const u32),
                16 => (*(self.map.add(off) as *const u16)) as u32,
                _ => return (0, 0, 0),
            };
            let mask = |len: u32| {
                if len >= 32 {
                    u32::MAX
                } else {
                    (1u32 << len).wrapping_sub(1)
                }
            };
            (
                (val >> self.r_off) & mask(self.r_len),
                (val >> self.g_off) & mask(self.g_len),
                (val >> self.b_off) & mask(self.b_len),
            )
        }
    }

    /// Alpha-blend a glyph sample of coverage `cov` (0-255) over the pixel already there.
    #[inline]
    pub fn blend_pixel(&mut self, x: usize, y: usize, cov: u8, r: u32, g: u32, b: u32) {
        if cov == 0 {
            return;
        }
        let (dr, dg, db) = self.read_pixel(x, y);
        let a = cov as u32;
        let nr = (r * a + dr * (255 - a)) / 255;
        let ng = (g * a + dg * (255 - a)) / 255;
        let nb = (b * a + db * (255 - a)) / 255;
        self.pixel(x, y, nr, ng, nb);
    }

    pub fn rect(&mut self, x0: usize, y0: usize, w: usize, h: usize, r: u32, g: u32, b: u32) {
        for y in y0..y0.saturating_add(h) {
            for x in x0..x0.saturating_add(w) {
                self.pixel(x, y, r, g, b);
            }
        }
    }

    pub fn clear(&mut self, r: u32, g: u32, b: u32) {
        if self.bpp == 32 {
            let val = (r << self.r_off) | (g << self.g_off) | (b << self.b_off);
            let total = self.stride * self.height;
            let mut o = 0usize;
            while o + 4 <= total {
                unsafe { *(self.map.add(o) as *mut u32) = val; }
                o += 4;
            }
        } else {
            for y in 0..self.height {
                for x in 0..self.width {
                    self.pixel(x, y, r, g, b);
                }
            }
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        sys::munmap(self.map, self.map_len);
    }
}
