//! SWAR: SIMD Within A Register

#[derive(Debug)]
pub struct MatchTable {
    delimiters: Vec<usize>,
}

impl MatchTable {
    pub const ALIGNMENT: usize = core::mem::size_of::<usize>();

    pub fn new<const N: usize>(delimiters: [u8; N]) -> Self {
        Self { delimiters: delimiters.into_iter().map(Self::uniform_segment).collect() }
    }

    #[inline]
    pub fn match_vectored(&self, data: &[u8; Self::ALIGNMENT]) -> Option<usize> {
        let x = usize::from_ne_bytes(*data);
        for d in &self.delimiters {
            let y = *d ^ x;
            let found = y.to_ne_bytes().into_iter().position(|b| b == 0);
            if let Some(i) = found {
                return Some(i);
            }
        }
        None
    }

    // creates a u64 whose bytes are each equal to b
    #[inline]
    const fn uniform_segment(b: u8) -> usize {
        usize::from_ne_bytes([b; Self::ALIGNMENT])
    }
}
