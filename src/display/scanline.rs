use super::{WIDTH, rgb_to_16bpp};
use super::ili9486::command::PixelFormat;

pub struct ScanLine {
    buf: [u8; 2 * WIDTH],
}

impl ScanLine {
    #[inline(always)]
    pub fn new<F: Fn(usize) -> (u8, u8, u8)>(f: F) -> Self {
        let mut this = ScanLine {
            buf: unsafe { core::mem::uninitialized() }}
        ;
        let mut i = 0;
        let mut x = 0;
        while i < this.buf.len() {
            let (r, g, b) = f(x);
            this.buf[i..(i + 2)].copy_from_slice(&rgb_to_16bpp(r, g, b));
            i += 2;
            x += 1;
        }
        this
    }
}

impl AsRef<[u8]> for ScanLine {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..]
    }
}
