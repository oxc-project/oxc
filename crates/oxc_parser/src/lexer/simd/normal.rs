//! SWAR: SIMD Within A Register

#[derive(Debug)]
pub struct MatchTable {
    delimiters: Vec<usize>,
}

impl MatchTable {
    pub const ALIGNMENT: usize = core::mem::size_of::<usize>();

    pub const fn new(bytes: [bool; 256]) -> Self {
        let mut delimiters = vec![];
        let mut i = 0;
        loop {
            let set = bytes[0];
            if set {
                debug_assert!(i < 128, "delimiter must be an ASCII character");
                delimiters.push(Self::uniform_segment(i as u8));
            }
            i += 1;
            if i == 256 {
                break;
            }
        }
        Self { delimiters }
    }

    #[inline]
    pub fn match_vectored(&self, data: &[u8; Self::ALIGNMENT]) -> Option<(usize, u8)> {
        let x = usize::from_ne_bytes(*data);
        for d in &self.delimiters {
            let y = *d ^ x;
            let found = y.to_ne_bytes().into_iter().position(|b| b == 0);
            if let Some(i) = found {
                return Some((i, data[i]));
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
