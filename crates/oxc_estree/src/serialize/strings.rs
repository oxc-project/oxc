use std::{num::NonZeroU64, slice};

use oxc_data_structures::{code_buffer::CodeBuffer, slice_iter::SliceIter};

use super::{ESTree, Serializer};

/// Convert `char` to UTF-8 bytes array.
const fn char_to_bytes<const N: usize>(ch: char) -> [u8; N] {
    let mut bytes = [0u8; N];
    ch.encode_utf8(&mut bytes);
    bytes
}

/// Lossy replacement character (U+FFFD) as UTF-8 bytes.
const LOSSY_REPLACEMENT_CHAR_BYTES: [u8; 3] = char_to_bytes('\u{FFFD}');
const LOSSY_REPLACEMENT_CHAR_FIRST_BYTE: u8 = LOSSY_REPLACEMENT_CHAR_BYTES[0]; // 0xEF

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
        let buffer = serializer.buffer_mut();
        buffer.print_ascii_byte(b'"');
        buffer.print_str(self.0);
        buffer.print_ascii_byte(b'"');
    }
}

/// A string which contains lone surrogates escaped with lossy replacement character (U+FFFD).
///
/// Lone surrogates are encoded in the string as `\uFFFD1234` where `1234` is the code point in hex.
/// These are converted to `\u1234` in JSON.
/// An actual lossy replacement character is encoded in the string as `\uFFFDfffd`, and is converted
/// to the actual character.
pub struct LoneSurrogatesString<'s>(pub &'s str);

impl ESTree for LoneSurrogatesString<'_> {
    #[inline(always)]
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        write_str::<LoneSurrogatesEscapeTable>(self.0, serializer.buffer_mut());
    }
}

/// [`ESTree`] implementation for string slice.
impl ESTree for str {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        write_str::<StandardEscapeTable>(self, serializer.buffer_mut());
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
    LO = b'X',  // Lossy escape character first byte
    UU = b'u',  // \x00...\x1F except the ones above
}

/// Lookup table of escape sequences. A value of `b'x'` at index `i` means that byte `i`
/// is escaped as "\x" in JSON. A value of 0 means that byte `i` is not escaped.
///
/// A value of `UU` means that byte is escaped as `\u00xx`, where `xx` is the hex code of the byte.
/// e.g. `0x1F` is output as `\u001F`.
static ESCAPE: [Escape; 256] = create_table(Escape::__);

/// Same as `ESCAPE` but with `Escape::LO` for byte 0xEF.
static ESCAPE_LONE_SURROGATES: [Escape; 256] = create_table(Escape::LO);

const fn create_table(lo: Escape) -> [Escape; 256] {
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
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, lo, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
}

/// Trait for tables determining whether bytes need escaping.
trait EscapeTable {
    /// `true` if need to escape lone surrogate escape sequences.
    const LONE_SURROGATES: bool;

    /// Get `Escape` for a byte.
    /// If byte does not require escaping, returns `Escape::__`.
    fn get_escape_for_byte(b: u8) -> Escape;

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
    fn get_escapes_mask(bytes: [u8; 8]) -> u64;
}

/// Escape table for standard strings (`impl ESTree for str`).
struct StandardEscapeTable;

impl EscapeTable for StandardEscapeTable {
    const LONE_SURROGATES: bool = false;

    #[inline]
    fn get_escape_for_byte(b: u8) -> Escape {
        ESCAPE[b as usize]
    }

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
}

/// Escape table for strings containing lone surrogates (`impl ESTree for LoneSurrogatesString`).
struct LoneSurrogatesEscapeTable;

impl EscapeTable for LoneSurrogatesEscapeTable {
    const LONE_SURROGATES: bool = true;

    #[inline]
    fn get_escape_for_byte(b: u8) -> Escape {
        ESCAPE_LONE_SURROGATES[b as usize]
    }

    #[inline]
    fn get_escapes_mask(bytes: [u8; 8]) -> u64 {
        const LOSSYS: u64 = splat_u64(LOSSY_REPLACEMENT_CHAR_FIRST_BYTE);
        const ONES: u64 = splat_u64(1);
        const TOP_BITS: u64 = splat_u64(0x80);

        // Get mask for ASCII escapes
        let ascii_escapes = StandardEscapeTable::get_escapes_mask(bytes);

        // Convert bytes to a `u64` in native byte order
        let n = u64::from_ne_bytes(bytes);

        // 0xEF -> 0xFF (top bit set).
        // All other non-ASCII bytes -> values with top bit unset.
        let lossys = (n ^ LOSSYS).wrapping_sub(ONES);
        // ASCII bytes -> 0x00 (top bit unset).
        // Non-ASCII bytes -> 0x80 (top bit set).
        let non_ascii = n & TOP_BITS;
        // Mask `lossys` to only top bits, and zero any ASCII bytes
        let lossys = lossys & non_ascii;

        // Combine masks for ASCII escapes and lossy replacement characters.
        // Now any bytes needing escape = 0x80.
        // Any bytes not needing escape = 0.
        ascii_escapes | lossys
    }
}

/// Create `u64` with all bytes set to `n`.
/// e.g. `0x20` -> `0x2020202020202020`.
const fn splat_u64(n: u8) -> u64 {
    (u64::MAX / 0xFF) * (n as u64)
}

/// Write string to buffer.
/// String is wrapped in `"`s, and with any characters which are not valid in JSON escaped.
fn write_str<T: EscapeTable>(s: &str, buffer: &mut CodeBuffer) {
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

                let escapes_mask = T::get_escapes_mask(*chunk);
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
                    escape = T::get_escape_for_byte(byte);
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
                    escape = T::get_escape_for_byte(byte);
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

        // Handle lone surrogates.
        // Skip this block if not escaping lone surrogates.
        if T::LONE_SURROGATES && escape == Escape::LO {
            // SAFETY: `0xEF` is always 1st byte in a 3-byte UTF-8 character,
            // so reading next 2 bytes cannot be out of bounds
            let next_2_bytes = unsafe { iter.as_slice().get_unchecked(1..3) };
            if next_2_bytes == &LOSSY_REPLACEMENT_CHAR_BYTES[1..] {
                // Lossy replacement character (U+FFFD) is used as an escape before lone surrogates,
                // with the code point as 4 x hex characters after it e.g. `\u{FFFD}d800`.

                // Print the chunk up to before the lossy replacement character.
                // SAFETY: 0xEF is always the start of a 3-byte unicode character.
                // Therefore `current_ptr` must be on a UTF-8 character boundary.
                // `chunk_start_ptr` is start of string originally, and is only updated to be after
                // an ASCII character, so must also be on a UTF-8 character boundary, and in bounds.
                // `chunk_start_ptr` is after a previous byte so must be `<= current_ptr`.
                unsafe {
                    let current_ptr = iter.ptr();
                    let len = current_ptr.offset_from_unsigned(chunk_start_ptr);
                    let chunk = slice::from_raw_parts(chunk_start_ptr, len);
                    buffer.print_bytes_unchecked(chunk);
                }

                // Consume the lossy replacement character.
                // SAFETY: Lossy replacement character is 3 bytes.
                unsafe { iter.advance_unchecked(3) };

                let hex = iter.as_slice().get(..4).unwrap();
                if hex == b"fffd" {
                    // This is an actual lossy replacement character (not an escaped lone surrogate)
                    buffer.print_str("\u{FFFD}");

                    // Consume `fffd`.
                    // SAFETY: We know next 4 bytes are `fffd`.
                    unsafe { iter.advance_unchecked(4) };

                    // Set `chunk_start_ptr` to after `\u{FFFD}fffd`.
                    // That's a complete UTF-8 sequence, so `chunk_start_ptr` is definitely
                    // left on a UTF-8 character boundary.
                    chunk_start_ptr = iter.ptr();
                } else {
                    // This is an escaped lone surrogate.
                    // Next 4 bytes should be code point encoded as 4 x hex bytes.
                    #[cfg(debug_assertions)]
                    for &b in hex {
                        assert!(matches!(b, b'0'..=b'9' | b'a'..=b'f'));
                    }

                    // Print `\u`. Leave the hex bytes to be printed in next batch.
                    // After lossy replacement character is definitely a UTF-8 boundary.
                    buffer.print_str("\\u");
                    chunk_start_ptr = iter.ptr();

                    // SAFETY: `iter.as_slice().get(..4).unwrap()` above would have panicked
                    // if there weren't at least 4 bytes remaining in `iter`.
                    // We haven't checked that the 4 following bytes are ASCII, but it doesn't matter
                    // whether `iter` is left on a UTF-8 char boundary or not.
                    unsafe { iter.advance_unchecked(4) }
                }
            } else {
                // Some other unicode character starting with 0xEF.
                // Consume it and continue the loop.
                // SAFETY: `0xEF` is always 1st byte in a 3-byte UTF-8 character.
                unsafe { iter.advance_unchecked(3) };
            }
            continue;
        }

        // Print the chunk up to before the character which requires escaping.
        let current_ptr = iter.ptr();
        // SAFETY: `escape` is only non-zero for ASCII bytes, except `Escape::LO` which is handled above.
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
        // `escape` is only non-zero for ASCII bytes, except `Escape::LO` which is handled above.
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
    use super::super::CompactTSSerializer;
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
            let mut serializer = CompactTSSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_string() {
        let cases = [(String::new(), r#""""#), ("foobar".to_string(), r#""foobar""#)];

        for (input, output) in cases {
            let mut serializer = CompactTSSerializer::default();
            input.to_string().serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_json_safe_string() {
        let cases = [("", r#""""#), ("a", r#""a""#), ("abc", r#""abc""#)];

        for (input, output) in cases {
            let mut serializer = CompactTSSerializer::default();
            JsonSafeString(input).serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_lone_surrogates_string() {
        let cases = [
            ("\u{FFFD}fffd", "\"\u{FFFD}\""),
            ("_x_\u{FFFD}fffd_y_\u{FFFD}fffd_z_", "\"_x_\u{FFFD}_y_\u{FFFD}_z_\""),
            ("\u{FFFD}d834\u{FFFD}d835", r#""\ud834\ud835""#),
            ("_x_\u{FFFD}d834\u{FFFD}d835", r#""_x_\ud834\ud835""#),
            ("\u{FFFD}d834\u{FFFD}d835_y_", r#""\ud834\ud835_y_""#),
            ("_x_\u{FFFD}d834_y_\u{FFFD}d835_z_", r#""_x_\ud834_y_\ud835_z_""#),
            (
                "They call me \"Bob\" but I prefer \"Den\u{FFFD}d834\\qnis\", innit?",
                r#""They call me \"Bob\" but I prefer \"Den\ud834\\qnis\", innit?""#,
            ),
        ];

        for (input, output) in cases {
            let mut serializer = CompactTSSerializer::default();
            LoneSurrogatesString(input).serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }
}
