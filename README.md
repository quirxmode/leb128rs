# leb128rs
This crate provides LEB128 serialization and deserialization routines for the following types:
- Unsigned:
  - `u16`
  - `u32`
  - `u64`
  - `u128`
  - `usize`
- Signed:
  - `i16`
  - `i32`
  - `i64`
  - `i128`
  - `isize`

There are no functions for `u8` and `i8` as actual LEB128 encoding would never save space. If there is a need to encode these types, let me know.

The deserialization functions
- refuse to read more bytes than could be serialized for a type but
- do not check whether the last byte indicates more bytes (i.e., has its high bit set).

If there is demand for a pedantic (and thus, slower) deserialization variant, let me know.

## Tests
Commonly agreed-upon conversions have been inspired from
- https://en.wikipedia.org/wiki/LEB128, namely
  - `624485`: `0xE5 0x8E 0x26`
  - `-123456`: `0xC0 0xBB 0x78`
- DWARF Debugging Information Format, Version 4, SECTION 7-- DATA REPRESENTATION:
  - Unsigned:
    - `2`: `0x02`
    - `127`: `0x7f`
    - `128`: `0x80 0x01`
    - `129`: `0x81 0x01`
    - `130`: `0x82 0x01`
    - `12857`: `0xB9 0x64`
  - Signed:
    - `2`: `0x02`
    - `-2`: `0x7E`
    - `127`: `0xFF 0x00`
    - `-127`: `0x81 0x7F`
    - `128`: `0x80 0x01`
    - `-128`: `0x80 0x7F`
    - `129`: `0x81 0x01`
    - `-129`: `0xFF 0x7E`

The default unit tests also test the full range of
- `u16`,
- `i16`,

as well as
- patterns for all types and
- steps of regular intervals for types with more than 16 bits and
- reads from invalid encodings (i.e., values which indicate more bytes after the maximum number of bytes for a type).
