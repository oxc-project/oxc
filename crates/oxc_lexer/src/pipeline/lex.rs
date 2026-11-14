use crate::options::LexOptions;

use crate::pipeline::{bitmap::Bitmaps, kinds::Kinds, source::Source};

/// One lex as the stages see it: the source, the options and the per-block buffers.
#[derive(Clone, Copy)]
pub(super) struct Lex<'a> {
    pub src: Source<'a>,
    pub bitmaps: &'a Bitmaps,
    pub kinds: Kinds,
    pub options: &'a LexOptions,
}
