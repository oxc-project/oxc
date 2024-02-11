use crate::lexer::source::Source;

pub(crate) const ALIGNMENT: usize = 16;

pub struct LookupTable {}

impl LookupTable {
    pub fn new(delimiters: &[u8]) -> Self {
        todo!()
    }

    #[inline]
    pub fn match_vectored(&self, source: &Source) -> (Option<usize>, usize) {
        todo!()
    }
}
