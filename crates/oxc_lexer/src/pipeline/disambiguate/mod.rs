//! Deciding what a token is, when that depends on the tokens before or after it.
//!
//! Some characters mean different things in different places:
//!
//! - `/` can start a regex, or be division.
//! - `<` can start a JSX element, open a list of type arguments or parameters, or be less-than.
//! - `>>` can close two type argument lists, or be a shift.
//!
//! A parser knows which from its context. The lexer has no parser, so it works the context out
//! from the surrounding tokens. The token before a site settles most cases; the rest are questions
//! about the parser's state, which [`context`] computes by walking forward from a nearby point
//! whose state is certain.
//!
//! There are 2 questions:
//!
//! - [`operator`]: Is a position directly after a complete value?
//!   `carve` asks this for every `/` which doesn't start a comment, and for most `<` in JSX files.
//! - [`type_context`]: Is an angle bracket part of a TypeScript type?
//!   `coalesce` asks this for runs like `>>` and `<<`, and `carve` for some `<` in `.tsx` files.
//!
//! Every question reads the lex through a [`TokensView`], which the asking stage builds over its
//! buffers, and keeps what it learns across the lex in a [`DisambiguateState`] the lexer owns.
//! [`common`] holds the parts both questions need.

use oxc_simd::{Cpu, cpu_multiversion, cpu_version};

use crate::pipeline::lex::Lex;

mod common;
mod context;
mod operator;
mod type_context;

#[cfg(test)]
mod tests;

use common::{Brackets, Closers, TokensView};
use context::Walks;

// Every scan is exact past its budget; the budget only bounds the work one site can spend.

/// Step cap for the scan back to an anchor (tokens plus the groups it jumps over);
/// past it the walk from the start of the source answers.
const WALK_SCAN_CAP: u32 = 2048;

/// Scan steps bounded walks may take, plus one per 32 bytes, before the full walk answers the rest.
const LOCAL_WALK_BUDGET: u32 = 16384;

/// Step cap for the rules which answer without a walk: finding the JSX tag around a keyword,
/// and settling a `>` run from the tokens around it.
const RULE_SCAN_CAP: u32 = 256;

/// Bracket steps a backward match takes before the per-lex closer-to-opener table answers, so a
/// group matched again and again (queries after a huge wrapper function) costs a lookup, not a
/// pass over its brackets each time.
const BRACKET_STEP_CAP: u32 = 1024;

/// Byte cap on the forward scans. Past it one pass resolves the opener exactly, and records what
/// it crosses in [`Closers`].
const FORWARD_SCAN_CAP: usize = 4096;

/// What the questions keep across a lex: the context walks, the bracket bitmap and the closers
/// the forward scans resolved. Owned by the lexer, begun at each lex, restarted when `coalesce`
/// starts asking.
pub(super) struct DisambiguateState {
    walks: Walks,
    shared: Shared,
}

/// What every view of the lex reads besides the stage's buffers.
#[derive(Default)]
pub struct Shared {
    brackets: Brackets,
    closers: Closers,
    is_ts: bool,
    is_module: bool,
}

impl Default for DisambiguateState {
    #[cpu_version(Generic)]
    fn default() -> Self {
        Self { walks: Walks::new(), shared: Shared::default() }
    }
}

impl DisambiguateState {
    /// A new lex over `n` bytes of a script or module.
    #[cpu_multiversion]
    pub fn begin(&mut self, n: usize, is_ts: bool, is_module: bool, cpu: impl Cpu) {
        self.shared.is_ts = is_ts;
        self.shared.is_module = is_module;
        self.walks.restart(is_module, cpu);
        self.shared.brackets.begin(n, cpu);
        self.shared.closers.clear(cpu);
    }

    /// `coalesce` starts asking: it sees keyword kinds `carve` did not, so the walks start over.
    #[cpu_multiversion]
    pub fn restart(&mut self, cpu: impl Cpu) {
        self.walks.restart(self.shared.is_module, cpu);
        // carve has hidden the JSX brackets these were computed over.
        self.shared.closers.clear(cpu);
    }

    #[cpu_multiversion]
    #[inline(always)]
    unsafe fn view<'a>(&'a self, lex: Lex<'a>, kw_final: usize, cpu: impl Cpu) -> TokensView<'a> {
        TokensView::new(lex, kw_final, &self.shared, cpu)
    }

    #[cpu_multiversion]
    #[inline(always)]
    unsafe fn view_and_walks<'a>(
        &'a mut self,
        lex: Lex<'a>,
        kw_final: usize,
        cpu: impl Cpu,
    ) -> (TokensView<'a>, &'a mut Walks) {
        (TokensView::new(lex, kw_final, &self.shared, cpu), &mut self.walks)
    }

    /// Safety: as for TokensView::new.
    #[cpu_multiversion]
    #[inline(always)]
    pub unsafe fn prev_sig(&self, lex: Lex, pos: usize, cpu: impl Cpu) -> Option<usize> {
        self.view(lex, 0, cpu).prev_sig(pos, cpu)
    }

    /// Safety: as for TokensView::new.
    #[cpu_multiversion]
    #[inline(always)]
    pub unsafe fn not_operator_position(&mut self, lex: Lex, p: usize, cpu: impl Cpu) -> bool {
        let (tokens, walks) = self.view_and_walks(lex, 0, cpu);
        operator::not_operator_position(&tokens, walks, p, cpu)
    }

    /// Safety: as for TokensView::new.
    #[cpu_multiversion]
    #[inline(always)]
    pub unsafe fn gt_run_split(
        &mut self,
        lex: Lex,
        kw_final: usize,
        p: usize,
        run: usize,
        cpu: impl Cpu,
    ) -> usize {
        let (tokens, walks) = self.view_and_walks(lex, kw_final, cpu);
        type_context::gt_run_split(&tokens, walks, p, run, cpu)
    }

    /// Safety: as for TokensView::new.
    #[cpu_multiversion]
    #[inline(always)]
    pub unsafe fn lt_run_split(&self, lex: Lex, lt: usize, cpu: impl Cpu) -> bool {
        type_context::lt_run_split(&self.view(lex, 0, cpu), lt, cpu)
    }

    /// Safety: as for TokensView::new.
    #[cpu_multiversion]
    #[inline(always)]
    pub unsafe fn jsx_over_generic(
        &mut self,
        lex: Lex,
        lt: usize,
        lp: usize,
        cpu: impl Cpu,
    ) -> (bool, bool) {
        let (tokens, walks) = self.view_and_walks(lex, 0, cpu);
        type_context::jsx_over_generic(&tokens, walks, lt, lp, cpu)
    }
}
