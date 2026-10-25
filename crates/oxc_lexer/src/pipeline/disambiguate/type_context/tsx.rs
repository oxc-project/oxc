//! Entry points for `carve`: a `<` in a `.tsx` file.
//!
//! `<T>` at operand position may open a JSX element, or the type parameters of a generic arrow
//! function (`<T,>(x: T) => x`). Elsewhere it may sit in a type (`let f: <T>(x: T) => T`) or open
//! the type parameters of a declaration or member. Which one is a question about the parser's
//! state at the `<`, so all three answers read the forward context walk ([`context`]).
//!
//! [`context`]: crate::pipeline::disambiguate::context

use crate::pipeline::disambiguate::{Tokens, Walks, context};

use super::bytes::arrow_after_params;

/// Whether the ambiguous < at lt is JSX, and whether it is an unterminated element to report.
#[inline(never)]
pub(crate) fn jsx_over_generic(
    tokens: &Tokens,
    walks: &mut Walks,
    lt: usize,
    lp: usize,
) -> (bool, bool) {
    let site = context::before(tokens, walks, lt);
    if site.in_type || site.type_params {
        return (false, false);
    }
    if arrow_after_params(tokens, lp) {
        return (false, site.operand);
    }
    (true, false)
}
