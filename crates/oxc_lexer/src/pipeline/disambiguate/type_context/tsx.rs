//! Entry points for `carve`: a `<` in a `.tsx` file.
//!
//! `<T>` at operand position opens a JSX element, as in TypeScript, so a generic arrow function
//! there is written `<T,>(x: T) => x`. Elsewhere it may sit in a type (`let f: <T>(x: T) => T`)
//! or open the type parameters of a declaration or member. Which one is a question about the
//! parser's state at the `<`, so all three answers read the forward context walk ([`context`]).
//!
//! [`context`]: crate::pipeline::disambiguate::context

use oxc_simd::cpu_version;

use crate::pipeline::disambiguate::{TokensView, Walks, context};

/// Whether the ambiguous < at lt opens JSX, as it does outside types and type parameters.
#[cpu_version(Generic)]
#[inline(never)]
pub fn jsx_over_generic(tokens: &TokensView, walks: &mut Walks, lt: usize) -> bool {
    let site = context::before(tokens, walks, lt);
    !site.in_type && !site.type_params
}
