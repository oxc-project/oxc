use crate::lexer::source::Source;

const ALIGNMENT: usize = core::mem::size_of::<usize>();

// The capacity of each segment being processed
type Segment = [u8; ALIGNMENT];

pub struct LookupTable {}

impl LookupTable {
    pub fn new(delimiters: &[u8]) -> Self {
        todo!()
    }

    #[inline]
    pub fn match_vectored(&self, source: &Source) -> usize {
        todo!()
    }

    #[inline]
    fn match_delimiters_swar(&self, seg: Segment) -> usize {
        todo!()
    }
}
