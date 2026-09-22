//! Is a `<` in a `.tsx` file TypeScript syntax, rather than the start of a JSX element?
//!
//! In `.tsx` files `<T>` can open either a JSX element or a type parameter list,
//! as in `let f: <T>(x: T) => T` or `function <T>(x: T) {}`.
//! `carve` settles most cases by looking ahead.
//! When that isn't enough, it asks these questions, which look back at what comes before the `<`:
//!
//! - [`ts_type_region_open`]: Does a type start here, as in after the `:` in `let f: <T>(x: T) => T`?
//! - [`type_parameter_list_head`]: Does this `<` open a type parameter list,
//!   as in `function <T>` or a method `m<T>()`?
//! - [`jsx_site_is_expression`]: Is this `<` in expression position?
//!   There TypeScript reads `<T>(x: T) => x` as a JSX element, which never closes.
//!   So this decides whether that error is reported.

use crate::pipeline::disambiguate::{Tokens, Walks, context};

pub fn ts_type_region_open(tokens: &Tokens, walks: &mut Walks, lt: usize) -> bool {
    context::before(tokens, walks, lt).in_type
}

pub fn type_parameter_list_head(tokens: &Tokens, walks: &mut Walks, lt: usize) -> bool {
    context::before(tokens, walks, lt).type_params
}

pub fn jsx_site_is_expression(tokens: &Tokens, walks: &mut Walks, lt: usize) -> bool {
    context::before(tokens, walks, lt).operand
}
