//! Is an angle bracket part of a TypeScript type?
//!
//! Two stages ask this question:
//!
//! - `coalesce`, for runs of angle brackets ([`runs`]).
//!   A `>>` which closes two type argument lists, as in `Foo<Bar<T>>`, must be split into two `>` tokens.
//!   So must a `<<` which opens two lists, as in `Array<<T>(x: T) => T>`.
//! - `carve`, for a `<` in a `.tsx` file ([`tsx`]).
//!   `<T>` can open either a JSX element or a type parameter list, as in `let f: <T>(x: T) => T`.
//!
//! - [`bytes`]: Bracket matching by scanning raw source bytes.
//! - [`type_list`]: TypeScript's checks for accepting `<...>` as type arguments in an expression.
//! - [`runs`]: Entry points for `coalesce`.
//! - [`tsx`]: Entry points for `carve`.

mod bytes;
mod runs;
mod tsx;
mod type_list;

pub use bytes::arrow_after_params;
pub(super) use bytes::lt_run_opens_type_args;
pub use runs::{gt_run_split, lt_run_split};
pub use tsx::{jsx_site_is_expression, ts_type_region_open, type_parameter_list_head};
pub(super) use type_list::type_args_at;

#[cfg(test)]
mod tests;
