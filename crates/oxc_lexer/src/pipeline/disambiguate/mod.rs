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

pub(super) use common::{Brackets, Tokens, prev_sig};
pub(super) use context::Walks;
pub(super) use operator::not_operator_position;
pub(super) use type_context::{
    gt_run_split, jsx_site_is_expression, lt_run_split, ts_type_region_open,
    type_parameter_list_head,
};

#[cfg(test)]
mod tests;

/// What the questions keep across a lex: the context walks and the bracket bitmap. Owned by the
/// lexer, begun at each lex, restarted when `coalesce` starts asking.
pub(crate) struct State {
    pub(crate) walks: Walks,
    pub(crate) brackets: Brackets,
}

impl Default for State {
    fn default() -> Self {
        State { walks: Walks::new(), brackets: Brackets::default() }
    }
}

impl State {
    /// A new lex over `n` bytes of a script or module.
    pub(crate) fn begin(&mut self, n: usize, module: bool) {
        self.walks.restart(module);
        self.brackets.begin(n);
    }

    /// `coalesce` starts asking: it sees keyword kinds `carve` did not, so the walks start over.
    pub(crate) fn restart(&mut self, module: bool) {
        self.walks.restart(module);
    }
}
