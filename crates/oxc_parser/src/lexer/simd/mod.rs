#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) mod avx2;
#[cfg(target_arch = "aarch64")]
pub(crate) mod neon;

use once_cell::sync::Lazy;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) static STRING_LITERAL_LOOKUP_TABLE: Lazy<avx2::LookupTable> =
    Lazy::new(|| avx2::LookupTable::new(&[b'\r', b'\n', b'"', b'\'', b'\\']));

pub(crate) struct Position {
    // the offset of the first found delimiter
    pub(crate) offset: usize,
    // the maximum length of each segment, in avx2, it's 32 bytes
    pub(crate) segment: usize,
}
