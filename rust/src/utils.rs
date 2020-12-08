use std::hint::unreachable_unchecked;
use std::slice;

use memchr::memchr;

pub trait ByteSliceExt {
    fn get_u16(&self) -> u16;
    fn skip_past(&self, c: u8, i: usize) -> &Self;
    fn get_digit(&self) -> u8;
    fn check_first(&self, c: u8) -> bool;
    fn get_first(&self) -> u8;
    fn advance(&self, n: usize) -> &Self;
}

impl ByteSliceExt for [u8] {
    #[inline]
    fn get_u16(&self) -> u16 {
        let mut a = [0; 2];
        a.copy_from_slice(&self[..2]);
        u16::from_ne_bytes(a)
    }

    #[inline]
    fn skip_past(&self, c: u8, i: usize) -> &Self {
        &self[1 + i + memchr(c, self).unwrap_or_else(|| unsafe { unreachable_unchecked() })..]
    }

    #[inline]
    fn get_digit(&self) -> u8 {
        self.get_first().wrapping_sub(b'0')
    }

    #[inline]
    fn check_first(&self, c: u8) -> bool {
        self.get_first() == c
    }

    #[inline]
    fn get_first(&self) -> u8 {
        unsafe { *self.as_ptr() }
    }

    #[inline]
    fn advance(&self, n: usize) -> &Self {
        unsafe { slice::from_raw_parts(self.as_ptr().add(n), self.len() - n) }
    }
}
