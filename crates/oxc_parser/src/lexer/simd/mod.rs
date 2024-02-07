#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
#[cfg(target_arch = "aarch64")]
mod neon;
mod swar;

use crate::lexer::source::Source;
use once_cell::sync::Lazy;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static AVX2_STRING_LITERAL_LOOKUP_TABLE: Lazy<avx2::LookupTable> =
    Lazy::new(|| avx2::LookupTable::new(&[b'\r', b'\n', b'"', b'\'', b'\\']));

pub(crate) struct Position {
    // the offset of the first found delimiter
    pub(crate) offset: usize,
    // the maximum length of each segment, in avx2, it's 32 bytes
    pub(crate) alignment: usize,
}

pub(crate) fn string_literal_lookup(source: &Source) -> Position {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if cfg!(target_feature = "avx2") {
            return Position {
                offset: AVX2_STRING_LITERAL_LOOKUP_TABLE.match_vectored(source),
                alignment: avx2::ALIGNMENT,
            };
        }
    }
    todo!()
}
