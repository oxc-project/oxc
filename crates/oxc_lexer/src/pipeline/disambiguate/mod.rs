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
//! It keeps a cache of bracket pairs for each thread, reset by calling [`memo_new_lex`].

mod common;
mod operator;
mod type_context;

pub(super) use common::{bm_prev_sig, memo_new_lex};
pub(super) use operator::not_operator_position;
pub(super) use type_context::{
    gt_run_split, jsx_site_is_expression, lt_run_split, ts_type_region_open,
    type_parameter_list_head,
};

#[cfg(test)]
mod tests;
