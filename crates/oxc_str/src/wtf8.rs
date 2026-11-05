//! Code point ranges and byte values of [WTF-8](https://wtf-8.codeberg.page/):
//! [surrogates](https://wtf-8.codeberg.page/#surrogates-code-points),
//! [generalized UTF-8](https://wtf-8.codeberg.page/#generalized-utf-8), and
//! [surrogate byte sequences](https://wtf-8.codeberg.page/#surrogates-byte-sequences).

/// The largest [code point](https://wtf-8.codeberg.page/#code-point).
pub const CODE_POINT_MAX: u32 = char::MAX as u32;

/// The first [lead surrogate](https://wtf-8.codeberg.page/#surrogates-code-points).
pub const LEAD_SURROGATE_MIN: u32 = 0xD800;
pub const LEAD_SURROGATE_MAX: u32 = 0xDBFF;
pub const TRAIL_SURROGATE_MIN: u32 = 0xDC00;
pub const TRAIL_SURROGATE_MAX: u32 = 0xDFFF;
/// The first code point that UTF-16 encodes as a surrogate pair.
pub const SUPPLEMENTARY_MIN: u32 = 0x1_0000;
/// Payload bits of each half of a surrogate pair.
pub const SURROGATE_PAYLOAD_MASK: u32 = 0x3FF;

/// The largest code point that encodes as one byte, per
/// [Table 2](https://wtf-8.codeberg.page/#generalized-utf-8).
pub const ONE_BYTE_CODE_POINT_MAX: u32 = 0x7F;
/// The code points that encode as two bytes.
pub const TWO_BYTE_CODE_POINT_MIN: u32 = ONE_BYTE_CODE_POINT_MAX + 1;
pub const TWO_BYTE_CODE_POINT_MAX: u32 = 0x7FF;
/// The code points that encode as three bytes.
pub const THREE_BYTE_CODE_POINT_MIN: u32 = TWO_BYTE_CODE_POINT_MAX + 1;
pub const THREE_BYTE_CODE_POINT_MAX: u32 = SUPPLEMENTARY_MIN - 1;

/// Tag bits of a continuation byte, `10xxxxxx`.
pub const CONT_TAG: u8 = 0x80;
/// Payload bits of a continuation byte.
pub const CONT_MASK: u8 = 0x3F;

/// Tag bits of the first byte of a two-byte sequence, `110xxxxx`.
pub const TWO_BYTE_TAG: u8 = 0xC0;
/// Payload bits of the first byte of a two-byte sequence.
pub const TWO_BYTE_MASK: u8 = 0x1F;
/// Tag bits of the first byte of a three-byte sequence, `1110xxxx`.
pub const THREE_BYTE_TAG: u8 = 0xE0;
/// Payload bits of the first byte of a three-byte sequence.
pub const THREE_BYTE_MASK: u8 = 0x0F;
/// Tag bits of the first byte of a four-byte sequence, `11110xxx`.
pub const FOUR_BYTE_TAG: u8 = 0xF0;
/// Payload bits of the first byte of a four-byte sequence.
pub const FOUR_BYTE_MASK: u8 = 0x07;

/// The first byte of every surrogate's three-byte encoding, per
/// [Table 1](https://wtf-8.codeberg.page/#surrogates-byte-sequences).
pub const SURROGATE_FIRST_BYTE: u8 = 0xED;
/// The lowest second byte of any surrogate's encoding.
pub const SURROGATE_SECOND_BYTE_MIN: u8 = 0xA0;
/// The second byte of a surrogate's encoding tells lead from trail.
pub const LEAD_SURROGATE_SECOND_BYTE_MIN: u8 = 0xA0;
pub const LEAD_SURROGATE_SECOND_BYTE_MAX: u8 = 0xAF;
pub const TRAIL_SURROGATE_SECOND_BYTE_MIN: u8 = 0xB0;
/// The byte length of a surrogate's encoding.
pub const SURROGATE_BYTE_LEN: usize = 3;
