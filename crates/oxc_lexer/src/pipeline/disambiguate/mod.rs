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
//! Every question reads the lex through a [`Tokens`] view, which the asking stage builds over its
//! buffers, and keeps what it learns across the lex in a [`State`] the lexer owns. [`common`]
//! holds the parts both questions need.

mod common;
mod context;
mod operator;
mod type_context;

pub(super) use common::{Brackets, Closers, Tokens, prev_sig};
pub(super) use context::Walks;
pub(super) use operator::not_operator_position;
pub(super) use type_context::{
    arrow_after_params, gt_run_split, jsx_site_is_expression, lt_run_split, ts_type_region_open,
    type_parameter_list_head,
};

// Every scan is exact past its budget; the budget only bounds the work one site can spend.

/// Step cap for the scan back to an anchor (tokens plus the groups it jumps over);
/// past it the walk from the start of the source answers.
pub(crate) const WALK_SCAN_CAP: u32 = 2048;

/// Step cap for the rules which answer without a walk: finding the JSX tag around a keyword,
/// and settling a `>` run from the tokens around it.
pub(crate) const RULE_SCAN_CAP: u32 = 256;

/// Bracket steps a backward match takes before the per-lex closer-to-opener table answers, so a
/// group matched again and again (queries after a huge wrapper function) costs a lookup, not a
/// pass over its brackets each time.
pub(crate) const BRACKET_STEP_CAP: u32 = 1024;

/// Byte cap on the forward scans. Past it one pass resolves the opener exactly, and records what
/// it crosses in [`Closers`].
pub(crate) const FORWARD_SCAN_CAP: usize = 4096;

#[cfg(test)]
mod tests;

/// What the questions keep across a lex: the context walks, the bracket bitmap and the closers
/// the forward scans resolved. Owned by the lexer, begun at each lex, restarted when `coalesce`
/// starts asking.
pub(crate) struct State {
    pub(crate) walks: Walks,
    pub(crate) brackets: Brackets,
    pub(crate) closers: Closers,
}

impl Default for State {
    fn default() -> Self {
        State { walks: Walks::new(), brackets: Brackets::default(), closers: Closers::default() }
    }
}

impl State {
    /// A new lex over `n` bytes of a script or module.
    pub(crate) fn begin(&mut self, n: usize, module: bool) {
        self.walks.restart(module);
        self.brackets.begin(n);
        self.closers.clear();
    }

    /// `coalesce` starts asking: it sees keyword kinds `carve` did not, so the walks start over.
    pub(crate) fn restart(&mut self, module: bool) {
        self.walks.restart(module);
        // carve has hidden the JSX brackets these were computed over.
        self.closers.clear();
    }
}
