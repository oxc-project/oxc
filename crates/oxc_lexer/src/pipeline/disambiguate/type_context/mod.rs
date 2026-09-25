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
//! There are 4 files:
//!
//! - [`bytes`]: Bracket matching by scanning raw source bytes.
//! - [`type_list`]: TypeScript's checks for accepting `<...>` as type arguments in an expression.
//! - [`runs`]: Entry points for `coalesce`.
//! - [`tsx`]: Entry points for `carve`.
//!
//! What the bytes around a site cannot settle, the forward context walk answers ([`context`]).
//!
//! [`context`]: super::context

mod bytes;
mod runs;
mod tsx;
mod type_list;

pub(crate) use bytes::arrow_after_params;
pub(crate) use runs::{gt_run_split, lt_run_split};
pub(crate) use tsx::{jsx_site_is_expression, ts_type_region_open, type_parameter_list_head};
pub(super) use type_list::type_args_at;

#[cfg(test)]
mod tests;
