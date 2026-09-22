//! Deciding what a token is, when that depends on the tokens before or after it.
//!
//! Some characters mean different things in different places:
//!
//! - `/` can start a regex, or be division.
//! - `<` can start a JSX element, open a list of type arguments or parameters, or be less-than.
//! - `>>` can close two type argument lists, or be a shift.
//!
//! A parser knows which from its context. The lexer has no parser, so it works the context out
//! from the surrounding tokens, mostly by walking backwards over the ones it has already lexed,
//! but sometimes also by looking forwards.
//!
//! There are 2 questions:
//!
//! - [`operator`]: Is a position directly after a complete value?
//!   `carve` asks this for every `/` which doesn't start a comment, and for most `<` in JSX files.
//! - [`type_context`]: Is an angle bracket part of a TypeScript type?
//!   `coalesce` asks this for runs like `>>` and `<<`, and `carve` for some `<` in `.tsx` files.
//!
//! [`common`] holds the parts of those walks which both questions need.

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
pub(crate) const WALK_SCAN_CAP: u32 = 2048;

pub(crate) const RULE_SCAN_CAP: u32 = 256;

pub(crate) const BRACKET_STEP_CAP: u32 = 1024;

pub(crate) const FORWARD_SCAN_CAP: usize = 4096;

#[cfg(test)]
mod tests;

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
    pub(crate) fn begin(&mut self, n: usize, module: bool) {
        self.walks.restart(module);
        self.brackets.begin(n);
        self.closers.clear();
    }

    pub(crate) fn restart(&mut self, module: bool) {
        self.walks.restart(module);
        // carve has hidden the JSX brackets these were computed over.
        self.closers.clear();
    }
}
