use std::{num::NonZeroU64, slice};

use oxc_data_structures::{code_buffer::CodeBuffer, slice_iter::SliceIter};

use super::{ESTree, Serializer};

/// A string which does not need any escaping in JSON.
///
/// This provides better performance when you know that the string definitely contains no characters
/// that require escaping, as it avoids the cost of checking that.
///
/// If the string does in fact contain characters that did need escaping, it will result in invalid JSON.
pub struct JsonSafeString<'s>(pub &'s str);

impl ESTree for JsonSafeString<'_> {
    #[inline(always)]
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        serializer.buffer_mut().print_strs_array(["\"", self.0, "\""]);
    }
}

/// [`ESTree`] implementation for string slice.
impl ESTree for str {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        write_str(self, serializer.buffer_mut());
    }
}

/// [`ESTree`] implementation for `String`.
impl ESTree for String {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.as_str().serialize(serializer);
    }
}

/// Escapes
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Escape {
    __ = 0,
    BB = b'b',  // \x08
    TT = b't',  // \x09
    NN = b'n',  // \x0A
    FF = b'f',  // \x0C
    RR = b'r',  // \x0D
    QU = b'"',  // \x22
    BS = b'\\', // \x5C
    UU = b'u',  // \x00...\x1F except the ones above
}

/// Lookup table of escape sequences. A value of `b'x'` at index `i` means that byte `i`
/// is escaped as "\x" in JSON. A value of 0 means that byte `i` is not escaped.
///
/// A value of `UU` means that byte is escaped as `\u00xx`, where `xx` is the hex code of the byte.
/// e.g. `0x1F` is output as `\u001F`.
static ESCAPE: [Escape; 256] = create_table();

const fn create_table() -> [Escape; 256] {
    #[allow(clippy::enum_glob_use, clippy::allow_attributes)]
    use Escape::*;

    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
        UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
        __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
}

/// Get `Escape` for a byte.
/// If byte does not require escaping, returns `Escape::__`.
#[inline]
fn get_escape_for_byte(b: u8) -> Escape {
    ESCAPE[b as usize]
}

/// Check a block of 8 bytes for whether any byte needs escaping.
///
/// Returns a `u64` mask, with each byte representing whether the corresponding byte in `bytes`
/// needs escaping or not.
///
/// * Bytes which need escaping are represented by `0x80`.
/// * Bytes which don't need escaping are represented by `0`.
///
/// If no bytes in `bytes` require escaping, the returned `u64` is 0.
///
/// The returned `u64` is in native byte order.
/// i.e. 1st byte in `bytes` is lowest byte in returned `u64`.
///
/// For `_"_____"`, returns:
/// * `0x0080_0000_0000_0080` on little endian.
/// * `0x8000_0000_0000_8000` on big endian.
///
/// Adapted from:
/// <https://chromium.googlesource.com/v8/v8/+/1645281bbd1b183a252835d376166bd210135bbe/src/json/json-stringifier.cc#521>
///
/// An efficient way to check 8 bytes in one go, without any branches.
/// <https://godbolt.org/z/TEKqxcnP4>
///
/// I (@overlookmotel) tried to expand this search to 16 bytes using loops which I'd hoped would be
/// auto-vectorized to SIMD ops, but compiler does not do a good job of auto-vectorizing this.
/// It is SIMD-able, but would require explicit SIMD ops.
#[inline]
fn get_escapes_mask(bytes: [u8; 8]) -> u64 {
    const SPACES: u64 = splat_u64(b' ');
    const QUOTES: u64 = splat_u64(b'"');
    const SLASHES: u64 = splat_u64(b'\\');
    const ONES: u64 = splat_u64(1);
    const TOP_BITS: u64 = splat_u64(0x80);

    // Convert bytes to a `u64` in native byte order
    let n = u64::from_ne_bytes(bytes);

    // 0x00..=0x1F -> 0xE0..=0xFF (top bit set).
    // All other ASCII bytes -> 0x00..=0x5F (top bit unset).
    // Some non-ASCII bytes also have top bit set.
    let less_than_spaces = n.wrapping_sub(SPACES);
    // `"` -> 0xFF (top bit set).
    // All other ASCII bytes -> values with top bit unset.
    let quotes = (n ^ QUOTES).wrapping_sub(ONES);
    // `\` -> 0xFF (top bit set).
    // All other ASCII bytes -> values with top bit unset.
    let slashes = (n ^ SLASHES).wrapping_sub(ONES);
    // Any bytes requiring escape -> top bit set.
    // Any ASCII bytes not requiring escape -> top bit unset.
    // Non-ASCII bytes -> may or may not have top bit set.
    let escapes = less_than_spaces | quotes | slashes;
    // ASCII bytes -> 0x80 (top bit set).
    // Non-ASCII bytes -> 0x00 (top bit unset).
    let asciis = (!n) & TOP_BITS;
    // Mask `escapes` to only top bits, and zero any non-ASCII bytes.
    // Now any bytes needing escape = 0x80.
    // Any bytes not needing escape = 0.
    escapes & asciis
}

/// Create `u64` with all bytes set to `n`.
/// e.g. `0x20` -> `0x2020202020202020`.
const fn splat_u64(n: u8) -> u64 {
    (u64::MAX / 0xFF) * (n as u64)
}

/// Write string to buffer.
/// String is wrapped in `"`s, and with any characters which are not valid in JSON escaped.
fn write_str(s: &str, buffer: &mut CodeBuffer) {
    buffer.print_ascii_byte(b'"');

    let bytes = s.as_bytes();
    let mut chunk_start_ptr = bytes.as_ptr();
    let mut iter = bytes.iter();

    'outer: loop {
        // Consume bytes which require no unescaping.
        // Search in batches of 8 bytes for speed with longer strings.
        // Use arithmetic operations to check 8 bytes in one go.
        let mut byte;
        let mut escape;
        'inner: loop {
            if let Some(chunk) = iter.as_slice().get(..8) {
                let chunk: &[u8; 8] = chunk.try_into().unwrap(); // Infallible

                let escapes_mask = get_escapes_mask(*chunk);
                // `NonZeroU64::trailing_zeros` is more efficient than `u64::trailing_zeros`
                // on some platforms. Ditto `leading_zeros`.
                if let Some(escapes_mask) = NonZeroU64::new(escapes_mask) {
                    // At least 1 byte in this chunk needs escaping. Get index of that byte.
                    let found_bit_index = if cfg!(target_endian = "little") {
                        escapes_mask.trailing_zeros()
                    } else {
                        escapes_mask.leading_zeros()
                    };
                    let found_byte_index = (found_bit_index as usize) / 8;

                    // SAFETY: `escapes_mask` is non-zero, so must have at least 1 bit set.
                    // So `found_bit_index <= 63`, therefore `found_byte_index <= 7`.
                    // Chunk is 8 bytes, so `found_byte_index` cannot be out of bounds.
                    byte = unsafe { *chunk.get_unchecked(found_byte_index) };
                    escape = get_escape_for_byte(byte);
                    // Consume bytes before this one.
                    // SAFETY: `found_byte_index < 8` and there are at least 8 bytes remaining in `iter`
                    unsafe { iter.advance_unchecked(found_byte_index) };
                    break 'inner;
                }

                // Consume the whole batch.
                // SAFETY: There are at least `BATCH_SIZE` bytes remaining in `iter`.
                unsafe { iter.advance_unchecked(8) };

                // Go round `'inner` loop again to continue searching
            } else {
                // Not enough bytes remaining for a batch. Search byte-by-byte.
                for (i, &next_byte) in iter.clone().enumerate() {
                    byte = next_byte;
                    escape = get_escape_for_byte(byte);
                    if escape != Escape::__ {
                        // Consume bytes before this one.
                        // SAFETY: `i` is an index of `iter`, so cannot be out of bounds.
                        unsafe { iter.advance_unchecked(i) };
                        break 'inner;
                    }
                }

                // Got to end of string with no further characters requiring escaping found.
                // No need to advance `iter`, as we don't use its current pointer again.
                break 'outer;
            }
        }

        // Found a character that needs escaping

        // Print the chunk up to before the character which requires escaping.
        let current_ptr = iter.ptr();
        // SAFETY: `escape` is only non-zero for ASCII bytes.
        // Therefore `current_ptr` must be on an ASCII byte.
        // `chunk_start_ptr` is start of string originally, and is only updated to be after
        // an ASCII character, so must also be on a UTF-8 character boundary, and in bounds.
        // `chunk_start_ptr` is after a previous byte so must be `<= current_ptr`.
        unsafe {
            let len = current_ptr.offset_from_unsigned(chunk_start_ptr);
            let chunk = slice::from_raw_parts(chunk_start_ptr, len);
            buffer.print_bytes_unchecked(chunk);
        }

        write_char_escape(escape, byte, buffer);

        // SAFETY: `'inner` loop above ensures `iter` is not at end of string
        unsafe { iter.advance_unchecked(1) };

        // Set `chunk_start_ptr` to be after this character.
        // `escape` is only non-zero for ASCII bytes.
        // We just consumed that ASCII byte, so `chunk_start_ptr` must be on a UTF-8 char boundary.
        chunk_start_ptr = iter.ptr();
    }

    // Print last chunk.
    // SAFETY: `chunk_start_ptr` is start of string originally, and is only updated to be after
    // an ASCII character, so must be on a UTF-8 character boundary, and in bounds.
    // `chunk_start_ptr` is after a previous byte so must be `<= iter.end_ptr()`.
    unsafe {
        let len = iter.end_ptr().offset_from_unsigned(chunk_start_ptr);
        let chunk = slice::from_raw_parts(chunk_start_ptr, len);
        buffer.print_bytes_unchecked(chunk);
    }

    buffer.print_ascii_byte(b'"');
}

/// Write escape sequence to `buffer`.
fn write_char_escape(escape: Escape, byte: u8, buffer: &mut CodeBuffer) {
    #[expect(clippy::if_not_else)]
    if escape != Escape::UU {
        // SAFETY: All values of `Escape` are ASCII
        unsafe { buffer.print_bytes_unchecked(&[b'\\', escape as u8]) };
    } else {
        static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
        let bytes = [
            b'\\',
            b'u',
            b'0',
            b'0',
            HEX_DIGITS[(byte >> 4) as usize],
            HEX_DIGITS[(byte & 0xF) as usize],
        ];
        // SAFETY: `bytes` contains only ASCII bytes
        unsafe { buffer.print_bytes_unchecked(&bytes) }
    }
}

#[cfg(test)]
mod tests {
    use super::super::CompactSerializer;
    use super::*;

    #[test]
    fn serialize_string_slice() {
        let cases = [
            ("", r#""""#),
            ("foobar", r#""foobar""#),
            ("\n", r#""\n""#),
            ("\nfoobar", r#""\nfoobar""#),
            ("foo\nbar", r#""foo\nbar""#),
            ("foobar\n", r#""foobar\n""#),
            (
                "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F",
                r#""\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007\b\t\n\u000b\f\r\u000e\u000f""#,
            ),
            (
                "\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F",
                r#""\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017\u0018\u0019\u001a\u001b\u001c\u001d\u001e\u001f""#,
            ),
            (r"\r\n\q", r#""\\r\\n\\q""#),
            (
                r#"They call me "Bob" but I prefer "Dennis", innit?"#,
                r#""They call me \"Bob\" but I prefer \"Dennis\", innit?""#,
            ),
        ];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_string() {
        let cases = [(String::new(), r#""""#), ("foobar".to_string(), r#""foobar""#)];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            input.clone().serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_json_safe_string() {
        let cases = [("", r#""""#), ("a", r#""a""#), ("abc", r#""abc""#)];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            JsonSafeString(input).serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }
}
