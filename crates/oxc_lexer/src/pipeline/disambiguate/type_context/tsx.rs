//! Entry points for `carve`: a `<` in a `.tsx` file.
//!
//! `<T>` at operand position may open a JSX element, or the type parameters of a generic arrow
//! function (`<T,>(x: T) => x`). Elsewhere it may sit in a type (`let f: <T>(x: T) => T`) or open
//! the type parameters of a declaration or member. Which one is a question about the parser's
//! state at the `<`, so all three answers read the forward context walk ([`context`]).
//!
//! [`context`]: crate::pipeline::disambiguate::context

use crate::pipeline::disambiguate::{Tokens, Walks, context};

/// Is the `<` at `lt` inside a type: an annotation, an alias, a type literal or a type argument
/// list? A `<` there opens a list, never a JSX element.
pub(crate) fn ts_type_region_open(tokens: &Tokens, walks: &mut Walks, lt: usize) -> bool {
    context::before(tokens, walks, lt).in_type
}

/// Does the `<` at `lt` open the type parameters of a declaration head or member
/// (`function f<`, `class C<`, `m<T>() {}`)?
pub(crate) fn type_parameter_list_head(tokens: &Tokens, walks: &mut Walks, lt: usize) -> bool {
    context::before(tokens, walks, lt).type_params
}

/// Can an operand start at `lt`? A generic arrow there is an expression, so a `<T>(...)` that
/// turns out not to be one is an unterminated JSX element.
pub(crate) fn jsx_site_is_expression(tokens: &Tokens, walks: &mut Walks, lt: usize) -> bool {
    context::before(tokens, walks, lt).operand
}
