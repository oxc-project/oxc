use std::mem;

use oxc_simd::{Cpu, cpu_multiversion, cpu_version};
use oxc_span::Span;

use crate::{error::DiagCode, options::LexOptions, token::TokenKind};

use crate::pipeline::{
    bitmap::Bitmaps, carve::JsxFrame, disambiguate::DisambiguateState, kinds::Kinds, lanes::Lanes,
    source::Source, unicode_leads::resolve_unicode_leads,
};

/// One lex as the stages see it: the source, the options and the per-block buffers.
#[derive(Clone, Copy)]
pub(super) struct Lex<'a> {
    pub src: Source<'a>,
    pub bitmaps: &'a Bitmaps,
    pub kinds: Kinds,
    pub options: &'a LexOptions,
}

/// What the stages keep across a lex besides the per-block buffers.
#[derive(Default)]
pub(super) struct LexState {
    /// The lanes of the Tokens being filled, lent for the lex.
    pub lanes: Lanes,
    /// What the disambiguation questions learn across the lex.
    pub disambiguate: DisambiguateState,
    /// Non-ASCII lead bytes, checked as identifier chars once the tokens are known.
    pub unicode_leads: Vec<u32>,
    /// Ranges lexed content-blind (the tsx type-argument skip), whose diagnostics are dropped.
    pub diag_suppress: Vec<(u32, u32)>,
    /// Carve's nesting stacks.
    pub template_depth: Vec<u32>,
    pub jsx_stack: Vec<JsxFrame>,
}

impl LexState {
    /// Start a lex of n bytes that fills the lanes lent by Tokens.
    #[cpu_multiversion]
    pub fn begin(&mut self, lanes: Lanes, n: usize, options: &LexOptions, cpu: impl Cpu) {
        self.lanes = lanes;
        self.lanes.clear(cpu);
        self.disambiguate.begin(n, options.ts, options.source_type_module, cpu);
        self.unicode_leads.clear();
        self.diag_suppress.clear();
    }

    /// Resolve the unicode leads, drop diagnostics in suppressed ranges, and return the lanes.
    #[cpu_version(Generic)]
    pub fn finish(&mut self, src: &[u8], kinds: &[TokenKind], spans: &[Span]) -> Lanes {
        if !self.unicode_leads.is_empty() && !kinds.is_empty() {
            resolve_unicode_leads(&mut self.unicode_leads, &mut self.lanes, src, kinds, spans);
        }
        if !self.diag_suppress.is_empty() {
            // InvalidUtf8 survives a dropped range, or invalid UTF-8 could lex with no diagnostic.
            let ranges = &self.diag_suppress;
            self.lanes.diags.retain(|d| {
                d.code == DiagCode::InvalidUtf8
                    || !ranges.iter().any(|&(a, b)| d.off >= a && d.off < b)
            });
        }
        mem::take(&mut self.lanes)
    }
}
