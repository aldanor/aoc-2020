use std::hint::unreachable_unchecked;
use std::slice;

use memchr::memchr;

pub trait SliceExt<T>
where
    T: Copy,
{
    fn get_at(&self, i: usize) -> T;
    fn advance(&self, n: usize) -> &Self;

    #[inline]
    fn get_first(&self) -> T {
        self.get_at(0)
    }
}

impl<T: Copy + PartialEq> SliceExt<T> for [T] {
    #[inline]
    fn get_at(&self, i: usize) -> T {
        unsafe { *self.get_unchecked(i) }
    }

    #[inline]
    fn advance(&self, n: usize) -> &Self {
        unsafe { slice::from_raw_parts(self.as_ptr().add(n), self.len() - n) }
    }
}

pub trait ByteSliceExt: SliceExt<u8> {
    fn memchr(&self, c: u8) -> usize;
    fn get_u16_ne(&self) -> u16;

    #[inline]
    fn get_digit(&self) -> u8 {
        self.get_first().wrapping_sub(b'0')
    }

    #[inline]
    fn get_digit_at(&self, i: usize) -> u8 {
        self.get_at(i).wrapping_sub(b'0')
    }

    #[inline]
    fn skip_past(&self, c: u8, i: usize) -> &Self {
        self.advance(1 + i + self.memchr(c))
    }
}

impl ByteSliceExt for [u8] {
    #[inline]
    fn memchr(&self, c: u8) -> usize {
        memchr(c, self).unwrap_or_else(|| unsafe { unreachable_unchecked() })
    }

    #[inline]
    fn get_u16_ne(&self) -> u16 {
        let mut a = [0; 2];
        a.copy_from_slice(&self[..2]);
        u16::from_ne_bytes(a)
    }
}
