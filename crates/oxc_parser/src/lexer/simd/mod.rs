#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
mod avx2;
#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(all(not(target_feature = "avx2"), not(target_arch = "aarch64")))]
mod swar;

use crate::lexer::source::Source;
use once_cell::sync::Lazy;

pub(crate) struct Position {
    // the offset of the first found delimiter
    pub(crate) offset: usize,
    // the maximum length of each segment, in avx2, it's 32 bytes
    pub(crate) alignment: usize,
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
static AVX2_STRING_LITERAL_LOOKUP_TABLE: Lazy<avx2::LookupTable> =
    Lazy::new(|| avx2::LookupTable::new(&[b'\r', b'\n', b'"', b'\'', b'\\']));

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))]
pub(crate) fn string_literal_lookup(source: &Source) -> Position {
    Position {
        offset: AVX2_STRING_LITERAL_LOOKUP_TABLE.match_vectored(source),
        alignment: avx2::ALIGNMENT,
    }
}

#[cfg(target_arch = "aarch64")]
static NEON_STRING_LITERAL_LOOKUP_TABLE: Lazy<swar::LookupTable> =
    Lazy::new(|| swar::LookupTable::new(&[b'\r', b'\n', b'"', b'\'', b'\\']));

#[cfg(target_arch = "aarch64")]
pub(crate) fn string_literal_lookup(source: &Source) -> Position {
    Position {
        offset: NEON_STRING_LITERAL_LOOKUP_TABLE.match_vectored(source),
        alignment: swar::ALIGNMENT,
    }
}

#[cfg(all(not(target_feature = "avx2"), not(target_arch = "aarch64")))]
static SWAR_STRING_LITERAL_LOOKUP_TABLE: Lazy<swar::LookupTable> =
    Lazy::new(|| swar::LookupTable::new(&[b'\r', b'\n', b'"', b'\'', b'\\']));

#[cfg(all(not(target_feature = "avx2"), not(target_arch = "aarch64")))]
pub(crate) fn string_literal_lookup(source: &Source) -> Position {
    Position {
        offset: SWAR_STRING_LITERAL_LOOKUP_TABLE.match_vectored(source),
        alignment: swar::ALIGNMENT,
    }
}
