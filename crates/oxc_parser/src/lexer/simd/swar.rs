use crate::lexer::source::Source;

pub(crate) const ALIGNMENT: usize = core::mem::size_of::<usize>();

// The capacity of each segment being processed
type Segment = [u8; ALIGNMENT];

pub struct LookupTable<const N: usize> {
    delimiters: [usize; N],
}

impl<const N: usize> LookupTable<N> {
    pub fn new(delimiters: [u8; N]) -> Self {
        Self { delimiters: delimiters.map(uniform_segment) }
    }

    /// Returns[0]: the number of bytes to the next delimiter
    /// Returns[1]: the number of actual remaining bytes in the source
    #[inline]
    pub fn match_vectored(&self, source: &Source) -> (Option<usize>, usize) {
        if let Some((seg, actual_len)) = source.peek_n_with_padding::<ALIGNMENT>() {
            (self.match_delimiters(seg), actual_len)
        } else {
            (None, 0)
        }
    }

    #[inline]
    fn match_delimiters(&self, seg: Segment) -> Option<usize> {
        for d in self.delimiters {
            let x = usize::from_ne_bytes(seg);
            let y = d ^ x;
            let found = y.to_ne_bytes().into_iter().position(|b| b == 0);
            if let Some(i) = found {
                return Some(i);
            }
        }
        None
    }
}

// creates a u64 whose bytes are each equal to b
#[allow(overflowing_literals)]
const fn uniform_segment(b: u8) -> usize {
    usize::from_ne_bytes([b; ALIGNMENT])
}
