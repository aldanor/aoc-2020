use std::fmt::{Debug, Display};
use std::slice::from_raw_parts;

/// Fast equivalent to `&s[offset..]`.
#[inline]
pub fn unsafe_slice<T>(s: &[T], offset: usize) -> &[T] {
    unsafe { from_raw_parts(s.as_ptr().add(offset), s.len() - offset) }
}

/// Fast equivalent to `&s[..count]`.
#[inline]
pub fn unsafe_truncate<T>(s: &[T], count: usize) -> &[T] {
    unsafe { from_raw_parts(s.as_ptr(), count) }
}

// Most trait bounds are here really only to make testing simpler and avoid macros.
pub trait Integer: Sized + Copy + Default + ToString + PartialEq + Eq + Debug + Display {
    const MAX_DIGITS: usize;

    fn add_digit(self, digit: u8) -> Self;
    fn checked_add_digit(self, digit: u8) -> Option<Self>;
    fn from_digit(digit: u8) -> Self;
    fn is_negative(self) -> bool;
}

macro_rules! impl_integer {
    ($t:ty => $d:expr, $($tt:tt)*) => {
        impl_integer!($t => $d);
        impl_integer!($($tt)*);
    };
    ($t:ty => $d:expr) => {
        #[allow(clippy::use_self)]
        impl Integer for $t {
            const MAX_DIGITS: usize = $d;

            #[inline(always)]
            fn add_digit(self, digit: u8) -> Self {
                self * 10 + (digit as Self)
            }

            #[inline(always)]
            fn checked_add_digit(self, digit: u8) -> Option<Self> {
                self.checked_mul(10).and_then(|v| v.checked_add(digit as Self))
            }

            #[inline(always)]
            fn from_digit(digit: u8) -> Self {
                digit as Self
            }

            #[inline(always)]
            #[allow(unused_comparisons)]
            fn is_negative(self) -> bool {
                self < 0
            }
        }
    };
    () => {};
}

impl_integer!(
    i8 => 3, i16 => 5, i32 => 10, i64 => 19, isize => 19,
    u8 => 3, u16 => 5, u32 => 10, u64 => 20, usize => 20
);

/// Check if the byte is a digit.
#[inline]
const fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Parse unsigned integer.
pub fn parse_uint<T: Integer>(s: &[u8]) -> Option<(&[u8], T)> {
    if let Some((&c, s)) = s.split_first() {
        if is_digit(c) {
            let mut num = T::from_digit(c - b'0');
            for (i, &c) in s.iter().enumerate() {
                if !is_digit(c) {
                    return Some((unsafe_slice(s, i), num));
                }
                num = if i < T::MAX_DIGITS - 2 {
                    num.add_digit(c - b'0')
                } else {
                    num.checked_add_digit(c - b'0')?
                };
            }
            return Some((unsafe_slice(s, s.len()), num));
        }
    }
    None
}

/// Skip an unsigned integer, regardless of its type.
pub fn skip_uint(s: &[u8]) -> Option<(&[u8], ())> {
    let s = match s.split_first() {
        Some((&c, s)) if is_digit(c) => s,
        _ => return None,
    };
    for (i, &c) in s.iter().enumerate() {
        if !is_digit(c) {
            return Some((unsafe_slice(s, i), ()));
        }
    }
    Some((&[], ()))
}

#[inline]
pub fn parse_until_eol(s: &[u8]) -> Option<(&[u8], &[u8])> {
    for (i, &c) in s.iter().enumerate() {
        if c == b'\n' {
            return Some((unsafe_slice(s, i + 1), unsafe_truncate(s, i)));
        }
    }
    Some((&[], s))
}

/// Expect end of string.
#[inline]
pub const fn parse_eof(s: &[u8]) -> Option<(&[u8], ())> {
    if s.is_empty() {
        Some((&[], ()))
    } else {
        None
    }
}

#[inline]
pub fn expect_char(ch: u8) -> impl Fn(&[u8]) -> Option<(&[u8], ())> {
    #[inline]
    move |s: &[u8]| match s.split_first() {
        Some((&c, s)) if c == ch => Some((s, ())),
        _ => None,
    }
}

#[inline]
pub fn parse_char(s: &[u8]) -> Option<(&[u8], u8)> {
    match s.split_first() {
        Some((&ch, s)) => Some((s, ch)),
        _ => None,
    }
}

#[inline]
pub fn expect_str(prefix: &'static [u8]) -> impl Fn(&[u8]) -> Option<(&[u8], ())> {
    move |s: &[u8]| {
        if s.starts_with(prefix) {
            Some((unsafe_slice(s, prefix.len()), ()))
        } else {
            None
        }
    }
}
