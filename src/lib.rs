//! Serialize to and deserialize from LEB128.
//!
//! # Examples
//! The library writes to any type implementing `std::io::Write` and reads from any type implementing `std::io::Read`.
//! The serialization variants accept references to avoid having to clone values for serialization.
//!
//! ## With a buffer
//! ```rust
//! use leb128rs::{leb128_write_u64, leb128_read_u64};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let myu64 = 20260107;
//!
//!     let mut buf: Vec<u8> = vec![];
//!     leb128_write_u64(&myu64, &mut buf)?;
//!
//!     // Prints [8B, CA, D4, 09]
//!     println!("{:02X?}", buf);
//!
//!     let read_val = leb128_read_u64(&mut buf.as_slice())?;
//!
//!     // Prints 20260107
//!     println!("{}", read_val);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## With a file
//! ```rust
//! use std::fs::File;
//! use std::io::{Read, BufReader, BufWriter};
//! use leb128rs::{leb128_write_u64, leb128_read_u64};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let myu64 = 20260107;
//!
//!     {
//!         let file = File::create("leb128.bin")?;
//!         let mut buf_writer = BufWriter::new(&file);
//!         leb128_write_u64(&myu64, &mut buf_writer)?;
//!     }
//!
//!     {
//!         let file = File::open("leb128.bin")?;
//!         let mut buf_reader = BufReader::new(&file);
//!         let mut raw = Vec::new();
//!         buf_reader.read_to_end(&mut raw)?;
//!
//!         // Prints [8B, CA, D4, 09]
//!         println!("{:02X?}", raw);
//!     }
//!
//!     {
//!         let file = File::open("leb128.bin")?;
//!         let mut buf_reader = BufReader::new(&file);
//!         let read_val = leb128_read_u64(&mut buf_reader)?;
//!
//!         // Prints 20260107
//!         println!("{}", read_val);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!

use std::io::{Read, Write};

const fn leb128_max_bytes(len: usize) -> usize {
    let info_bits = len * 8;
    let indicator_bits = (info_bits + 6) / 7;
    (info_bits + indicator_bits + 7) / 8
}

macro_rules! makefn_leb128_write_unsigned_type {
    ($fname:ident, $uty:ty) => {
        pub fn $fname<W: Write>(value: &$uty, writer: &mut W) -> Result<(), std::io::Error> {
            let mut val = *value;
            loop {
                let byte = (val as u8) & 0x7F;
                val >>= 7;

                if val == 0 {
                    writer.write_all(&[byte])?;
                    break Ok(());
                } else {
                    let byte = byte | 0x80;
                    writer.write_all(&[byte])?;
                }
            }
        }
    };
}

makefn_leb128_write_unsigned_type!(leb128_write_u16, u16);
makefn_leb128_write_unsigned_type!(leb128_write_u32, u32);
makefn_leb128_write_unsigned_type!(leb128_write_u64, u64);
makefn_leb128_write_unsigned_type!(leb128_write_u128, u128);
makefn_leb128_write_unsigned_type!(leb128_write_usize, usize);

macro_rules! makefn_leb128_read_unsigned_type {
    ($fname:ident, $uty:ty) => {
        pub fn $fname<R: Read>(reader: &mut R) -> Result<$uty, std::io::Error> {
            let mut val: $uty = 0;
            let mut shift: usize = 0;
            let max_shift: usize = leb128_max_bytes(std::mem::size_of::<$uty>()) * 7;
            loop {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                let byte = buf[0];

                val = val | (<$uty>::from(byte & 0x7F) << shift);

                let have_next = byte & 0x80 == 0x80;

                // slightly faster, will never wrap
                shift = shift.wrapping_add(7);
                let may_read_more = shift < max_shift;

                if !have_next || !may_read_more {
                    break Ok(val);
                }
            }
        }
    };
}

makefn_leb128_read_unsigned_type!(leb128_read_u16, u16);
makefn_leb128_read_unsigned_type!(leb128_read_u32, u32);
makefn_leb128_read_unsigned_type!(leb128_read_u64, u64);
makefn_leb128_read_unsigned_type!(leb128_read_u128, u128);
makefn_leb128_read_unsigned_type!(leb128_read_usize, usize);

macro_rules! makefn_leb128_write_signed_type {
    ($fname:ident, $ity:ty) => {
        pub fn $fname<W: Write>(value: &$ity, writer: &mut W) -> Result<(), std::io::Error> {
            let mut val = *value;
            loop {
                let byte = (val as u8) & 0x7F;
                val >>= 7;

                let encodes_negative = byte & 0x40 == 0x40;
                if (val == 0 && !encodes_negative) || (val == -1 && encodes_negative) {
                    writer.write_all(&[byte])?;
                    break Ok(());
                } else {
                    let byte = byte | 0x80;
                    writer.write_all(&[byte])?;
                }
            }
        }
    };
}

makefn_leb128_write_signed_type!(leb128_write_i16, i16);
makefn_leb128_write_signed_type!(leb128_write_i32, i32);
makefn_leb128_write_signed_type!(leb128_write_i64, i64);
makefn_leb128_write_signed_type!(leb128_write_i128, i128);
makefn_leb128_write_signed_type!(leb128_write_isize, isize);

// This is a relatively weak assertion to cover the '<$ty> as usize' use below.
const _: () = assert!(
    128 < usize::MAX,
    "We require usize to be able to hold the value 128."
);

macro_rules! makefn_leb128_read_signed_type {
    ($fname:ident, $ity:ty) => {
        pub fn $fname<R: Read>(reader: &mut R) -> Result<$ity, std::io::Error> {
            let mut val: $ity = 0;
            let mut shift: usize = 0;
            let max_shift: usize = leb128_max_bytes(std::mem::size_of::<$ity>()) * 7;
            let last_byte = loop {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                let byte = buf[0];

                val = val | (<$ity>::from(byte & 0x7F) << shift);

                let have_next = byte & 0x80 == 0x80;

                // slightly faster, will never wrap
                shift = shift.wrapping_add(7);
                let may_read_more = shift < max_shift;

                if !have_next || !may_read_more {
                    break byte;
                }
            };

            // Static assertion above ensures usize can hold 128 as value.
            if (shift < (<$ity>::BITS as usize)) && (last_byte & 0x40 == 0x40) {
                val = val | (!0 << shift);
            }
            Ok(val)
        }
    };
}

makefn_leb128_read_signed_type!(leb128_read_i16, i16);
makefn_leb128_read_signed_type!(leb128_read_i32, i32);
makefn_leb128_read_signed_type!(leb128_read_i64, i64);
makefn_leb128_read_signed_type!(leb128_read_i128, i128);
makefn_leb128_read_signed_type!(leb128_read_isize, isize);

pub mod short {
    //! # Short names
    //! This module provides shorter function names.
    //!
    //! Example:
    //! ```rust
    //! use leb128rs::short::{write_u64, read_u64};
    //!
    //! fn main() -> Result<(), Box<dyn std::error::Error>> {
    //!     let myu64 = 20260107;
    //!
    //!     let mut buf: Vec<u8> = vec![];
    //!     write_u64(&myu64, &mut buf)?;
    //!
    //!     // Prints [8B, CA, D4, 09]
    //!     println!("{:02X?}", buf);
    //!
    //!     let read_val = read_u64(&mut buf.as_slice())?;
    //!
    //!     // Prints 20260107
    //!     println!("{}", read_val);
    //!
    //!     Ok(())
    //! }
    //! ```
    pub use super::leb128_read_i16 as read_i16;
    pub use super::leb128_read_i32 as read_i32;
    pub use super::leb128_read_i64 as read_i64;
    pub use super::leb128_read_i128 as read_i128;
    pub use super::leb128_read_isize as read_isize;

    pub use super::leb128_read_u16 as read_u16;
    pub use super::leb128_read_u32 as read_u32;
    pub use super::leb128_read_u64 as read_u64;
    pub use super::leb128_read_u128 as read_u128;
    pub use super::leb128_read_usize as read_usize;

    pub use super::leb128_write_i16 as write_i16;
    pub use super::leb128_write_i32 as write_i32;
    pub use super::leb128_write_i64 as write_i64;
    pub use super::leb128_write_i128 as write_i128;
    pub use super::leb128_write_isize as write_isize;

    pub use super::leb128_write_u16 as write_u16;
    pub use super::leb128_write_u32 as write_u32;
    pub use super::leb128_write_u64 as write_u64;
    pub use super::leb128_write_u128 as write_u128;
    pub use super::leb128_write_usize as write_usize;
}

#[cfg(test)]
mod test {
    use crate::*;

    macro_rules! make_test_624485 {
        ($fname:ident, $rf:ident, $wf:ident) => {
            #[test]
            fn $fname() -> Result<(), std::io::Error> {
                let val = 624485;
                let expected: Vec<u8> = vec![0xE5, 0x8E, 0x26];

                let mut buf: Vec<u8> = vec![];
                $wf(&val, &mut buf)?;

                assert_eq!(buf, expected);

                let read_val = $rf(&mut buf.as_slice())?;
                assert_eq!(val, read_val);

                Ok(())
            }
        };
    }

    make_test_624485!(i32_624485, leb128_read_i32, leb128_write_i32);
    make_test_624485!(i64_624485, leb128_read_i64, leb128_write_i64);
    make_test_624485!(i128_624485, leb128_read_i128, leb128_write_i128);
    make_test_624485!(isize_624485, leb128_read_isize, leb128_write_isize);

    make_test_624485!(u32_624485, leb128_read_u32, leb128_write_u32);
    make_test_624485!(u64_624485, leb128_read_u64, leb128_write_u64);
    make_test_624485!(u128_624485, leb128_read_u128, leb128_write_u128);
    make_test_624485!(usize_624485, leb128_read_usize, leb128_write_usize);

    macro_rules! make_test_m123456 {
        ($fname:ident, $rf:ident, $wf:ident) => {
            #[test]
            fn $fname() -> Result<(), std::io::Error> {
                let val = -123456;
                let expected: Vec<u8> = vec![0xC0, 0xBB, 0x78];

                let mut buf: Vec<u8> = vec![];
                $wf(&val, &mut buf)?;

                assert_eq!(buf, expected);

                let read_val = $rf(&mut buf.as_slice())?;
                assert_eq!(val, read_val);

                Ok(())
            }
        };
    }

    make_test_m123456!(i32_m123456, leb128_read_i32, leb128_write_i32);
    make_test_m123456!(i64_m123456, leb128_read_i64, leb128_write_i64);
    make_test_m123456!(i128_m123456, leb128_read_i128, leb128_write_i128);
    make_test_m123456!(isize_m123456, leb128_read_isize, leb128_write_isize);

    macro_rules! make_DRAWRF_tests_unsigned {
        ($fname:ident, $rf:ident, $wf:ident) => {
            #[test]
            fn $fname() -> Result<(), std::io::Error> {
                let vals: Vec<(_, &[u8])> = vec![
                    (2, &[0x02]),
                    (127, &[0x7F]),
                    (128, &[0x80, 0x01]),
                    (129, &[0x81, 0x01]),
                    (130, &[0x82, 0x01]),
                    (12857, &[0xB9, 0x64]),
                ];

                for (val, expected) in vals {
                    let mut buf: Vec<u8> = vec![];
                    $wf(&val, &mut buf)?;
                    assert_eq!(buf, expected);
                    let read_val = $rf(&mut buf.as_slice())?;
                    assert_eq!(val, read_val);
                }
                Ok(())
            }
        };
    }

    make_DRAWRF_tests_unsigned!(u16_dwarf, leb128_read_u16, leb128_write_u16);
    make_DRAWRF_tests_unsigned!(u32_dwarf, leb128_read_u32, leb128_write_u32);
    make_DRAWRF_tests_unsigned!(u64_dwarf, leb128_read_u64, leb128_write_u64);
    make_DRAWRF_tests_unsigned!(u128_dwarf, leb128_read_u128, leb128_write_u128);
    make_DRAWRF_tests_unsigned!(usize_dwarf, leb128_read_usize, leb128_write_usize);

    macro_rules! make_DRAWRF_tests_signed {
        ($fname:ident, $rf:ident, $wf:ident) => {
            #[test]
            fn $fname() -> Result<(), std::io::Error> {
                let vals: Vec<(_, &[u8])> = vec![
                    (2, &[0x02]),
                    (-2, &[0x7E]),
                    (127, &[0xFF, 0x00]),
                    (-127, &[0x81, 0x7F]),
                    (128, &[0x80, 0x01]),
                    (-128, &[0x80, 0x7F]),
                    (129, &[0x81, 0x01]),
                    (-129, &[0xFF, 0x7E]),
                ];

                for (val, expected) in vals {
                    let mut buf: Vec<u8> = vec![];
                    $wf(&val, &mut buf)?;
                    assert_eq!(buf, expected);
                    let read_val = $rf(&mut buf.as_slice())?;
                    assert_eq!(val, read_val);
                }
                Ok(())
            }
        };
    }

    make_DRAWRF_tests_signed!(i16_dwarf, leb128_read_i16, leb128_write_i16);
    make_DRAWRF_tests_signed!(i32_dwarf, leb128_read_i32, leb128_write_i32);
    make_DRAWRF_tests_signed!(i64_dwarf, leb128_read_i64, leb128_write_i64);
    make_DRAWRF_tests_signed!(i128_dwarf, leb128_read_i128, leb128_write_i128);
    make_DRAWRF_tests_signed!(isize_dwarf, leb128_read_isize, leb128_write_isize);

    macro_rules! make_test_pattern_max_to_zero {
        ($fname:ident, $tp:ty, $rf:ident, $wf:ident) => {
            #[test]
            fn $fname() -> Result<(), std::io::Error> {
                let mut val = <$tp>::MAX;
                loop {
                    let mut buf: Vec<u8> = vec![];
                    $wf(&val, &mut buf)?;
                    let read_val = $rf(&mut buf.as_slice())?;
                    assert_eq!(val, read_val);

                    if val == 0 {
                        break Ok(());
                    }

                    val >>= 1;
                }
            }
        };
    }

    make_test_pattern_max_to_zero!(u16_max_to_zero, u16, leb128_read_u16, leb128_write_u16);
    make_test_pattern_max_to_zero!(u32_max_to_zero, u32, leb128_read_u32, leb128_write_u32);
    make_test_pattern_max_to_zero!(u64_max_to_zero, u64, leb128_read_u64, leb128_write_u64);
    make_test_pattern_max_to_zero!(u128_max_to_zero, u128, leb128_read_u128, leb128_write_u128);
    make_test_pattern_max_to_zero!(
        usize_max_to_zero,
        usize,
        leb128_read_usize,
        leb128_write_usize
    );

    make_test_pattern_max_to_zero!(i16_max_to_zero, i16, leb128_read_i16, leb128_write_i16);
    make_test_pattern_max_to_zero!(i32_max_to_zero, i32, leb128_read_i32, leb128_write_i32);
    make_test_pattern_max_to_zero!(i64_max_to_zero, i64, leb128_read_i64, leb128_write_i64);
    make_test_pattern_max_to_zero!(i128_max_to_zero, i128, leb128_read_i128, leb128_write_i128);
    make_test_pattern_max_to_zero!(
        isize_max_to_zero,
        isize,
        leb128_read_isize,
        leb128_write_isize
    );

    trait IsSigned {
        const VALUE: bool;
    }

    macro_rules! impl_signed {
    ($($t:ty),*) => {
        $(impl IsSigned for $t { const VALUE: bool = true; })*
    };
}

    macro_rules! impl_unsigned {
    ($($t:ty),*) => {
        $(impl IsSigned for $t { const VALUE: bool = false; })*
    };
}

    impl_signed!(i16, i32, i64, i128, isize);
    impl_unsigned!(u16, u32, u64, u128, usize);

    macro_rules! make_test_pattern_high_one_to_zero {
        ($fname:ident, $tp:ty, $rf:ident, $wf:ident) => {
            #[test]
            fn $fname() -> Result<(), std::io::Error> {
                let shift = if <$tp as IsSigned>::VALUE {
                    (<$tp>::BITS - 2)
                } else {
                    (<$tp>::BITS - 1)
                };

                let mut val = 1 << shift;
                loop {
                    let mut buf: Vec<u8> = vec![];
                    $wf(&val, &mut buf)?;
                    let read_val = $rf(&mut buf.as_slice())?;
                    assert_eq!(val, read_val);

                    if val == 0 {
                        break Ok(());
                    }

                    val >>= 1;
                }
            }
        };
    }

    make_test_pattern_high_one_to_zero!(
        u16_high_one_to_zero,
        u16,
        leb128_read_u16,
        leb128_write_u16
    );
    make_test_pattern_high_one_to_zero!(
        u32_high_one_to_zero,
        u32,
        leb128_read_u32,
        leb128_write_u32
    );
    make_test_pattern_high_one_to_zero!(
        u64_high_one_to_zero,
        u64,
        leb128_read_u64,
        leb128_write_u64
    );
    make_test_pattern_high_one_to_zero!(
        u128_high_one_to_zero,
        u128,
        leb128_read_u128,
        leb128_write_u128
    );
    make_test_pattern_high_one_to_zero!(
        usize_high_one_to_zero,
        usize,
        leb128_read_usize,
        leb128_write_usize
    );

    make_test_pattern_high_one_to_zero!(
        i16_high_one_to_zero,
        i16,
        leb128_read_i16,
        leb128_write_i16
    );
    make_test_pattern_high_one_to_zero!(
        i32_high_one_to_zero,
        i32,
        leb128_read_i32,
        leb128_write_i32
    );
    make_test_pattern_high_one_to_zero!(
        i64_high_one_to_zero,
        i64,
        leb128_read_i64,
        leb128_write_i64
    );
    make_test_pattern_high_one_to_zero!(
        i128_high_one_to_zero,
        i128,
        leb128_read_i128,
        leb128_write_i128
    );
    make_test_pattern_high_one_to_zero!(
        isize_high_one_to_zero,
        isize,
        leb128_read_isize,
        leb128_write_isize
    );

    #[test]
    fn full_range_u16() -> Result<(), std::io::Error> {
        for val in 0..u16::MAX {
            let mut buf: Vec<u8> = vec![];
            leb128_write_u16(&val, &mut buf)?;
            let read_val = leb128_read_u16(&mut buf.as_slice())?;
            assert_eq!(val, read_val)
        }
        Ok(())
    }

    #[test]
    fn full_range_i16() -> Result<(), std::io::Error> {
        for val in i16::MIN..i16::MAX {
            let mut buf: Vec<u8> = vec![];
            leb128_write_i16(&val, &mut buf)?;
            let read_val = leb128_read_i16(&mut buf.as_slice())?;
            assert_eq!(val, read_val)
        }
        Ok(())
    }

    macro_rules! make_step_test {
        ($fname:ident, $tp:ty, $step:expr, $rf:ident, $wf:ident) => {
            #[test]
            fn $fname() -> Result<(), std::io::Error> {
                let step = $step;
                let mut val = <$tp>::MIN;
                let max = <$tp>::MAX - step;
                loop {
                    let mut buf: Vec<u8> = vec![];
                    $wf(&val, &mut buf)?;
                    let read_val = $rf(&mut buf.as_slice())?;
                    assert_eq!(val, read_val);

                    if val > max {
                        break;
                    }

                    // never wraps, slightly faster
                    val = val.wrapping_add(step);
                }
                Ok(())
            }
        };
    }

    make_step_test!(u32_step_137, u32, 137u32, leb128_read_u32, leb128_write_u32);
    make_step_test!(
        u64_step_702942986507,
        u64,
        702942986507u64,
        leb128_read_u64,
        leb128_write_u64
    );
    make_step_test!(
        u128_step_1298074214633706907131921139318517,
        u128,
        1298074214633706907131921139318517u128,
        leb128_read_u128,
        leb128_write_u128
    );

    make_step_test!(i32_step_137, i32, 137i32, leb128_read_i32, leb128_write_i32);
    make_step_test!(
        i64_step_702942986507,
        i64,
        702942986507i64,
        leb128_read_i64,
        leb128_write_i64
    );
    make_step_test!(
        i128_step_1298074214633706907131921139318517,
        i128,
        1298074214633706907131921139318517i128,
        leb128_read_i128,
        leb128_write_i128
    );

    struct CountingReader<'a, R: std::io::Read> {
        inner: &'a mut R,
        pub bytes_read: usize,
    }

    impl<'a, R: std::io::Read> std::io::Read for CountingReader<'a, R> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let n = self.inner.read(buf)?;
            self.bytes_read += n;
            Ok(n)
        }
    }

    macro_rules! make_test_read_from_invalid {
        ($fname:ident, $tp:ty, $rf:ident, $pattern:literal, $expected:expr) => {
            #[test]
            fn $fname() -> Result<(), std::io::Error> {
                const MAX_BYTES: usize = leb128_max_bytes(std::mem::size_of::<$tp>());
                let mut buf: &[u8] = &[$pattern; MAX_BYTES + 1];
                let mut counting_reader = CountingReader {
                    inner: &mut buf,
                    bytes_read: 0,
                };
                let read_val = $rf(&mut counting_reader)?;
                assert_eq!(read_val, $expected);
                assert_eq!(counting_reader.bytes_read, MAX_BYTES);
                Ok(())
            }
        };
    }

    make_test_read_from_invalid!(u16_from_invalid_0x80, u16, leb128_read_u16, 0x80, 0u16);
    make_test_read_from_invalid!(u32_from_invalid_0x80, u32, leb128_read_u32, 0x80, 0u32);
    make_test_read_from_invalid!(u64_from_invalid_0x80, u64, leb128_read_u64, 0x80, 0u64);
    make_test_read_from_invalid!(u128_from_invalid_0x80, u128, leb128_read_u128, 0x80, 0u128);
    make_test_read_from_invalid!(
        usize_from_invalid_0x80,
        usize,
        leb128_read_usize,
        0x80,
        0usize
    );

    make_test_read_from_invalid!(i16_from_invalid_0x80, i16, leb128_read_i16, 0x80, 0i16);
    make_test_read_from_invalid!(i32_from_invalid_0x80, i32, leb128_read_i32, 0x80, 0i32);
    make_test_read_from_invalid!(i64_from_invalid_0x80, i64, leb128_read_i64, 0x80, 0i64);
    make_test_read_from_invalid!(i128_from_invalid_0x80, i128, leb128_read_i128, 0x80, 0i128);
    make_test_read_from_invalid!(
        isize_from_invalid_0x80,
        isize,
        leb128_read_isize,
        0x80,
        0isize
    );

    make_test_read_from_invalid!(u16_from_invalid_0xff, u16, leb128_read_u16, 0xFF, 0xFFFFu16);
    make_test_read_from_invalid!(
        u32_from_invalid_0xff,
        u32,
        leb128_read_u32,
        0xFF,
        0xFFFFFFFFu32
    );
    make_test_read_from_invalid!(
        u64_from_invalid_0xff,
        u64,
        leb128_read_u64,
        0xFF,
        0xFFFFFFFFFFFFFFFFu64
    );
    make_test_read_from_invalid!(
        u128_from_invalid_0xff,
        u128,
        leb128_read_u128,
        0xFF,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128
    );

    make_test_read_from_invalid!(i16_from_invalid_0xff, i16, leb128_read_i16, 0xFF, -1i16);
    make_test_read_from_invalid!(i32_from_invalid_0xff, i32, leb128_read_i32, 0xFF, -1i32);
    make_test_read_from_invalid!(i64_from_invalid_0xff, i64, leb128_read_i64, 0xFF, -1i64);
    make_test_read_from_invalid!(i128_from_invalid_0xff, i128, leb128_read_i128, 0xFF, -1i128);
}
