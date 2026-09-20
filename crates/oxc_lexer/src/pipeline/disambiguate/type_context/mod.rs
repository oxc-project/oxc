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
//! There are 5 files, each depending only on the ones listed before it:
//!
//! - [`bytes`]: Bracket matching by scanning raw source bytes.
//! - [`type_list`]: TypeScript's checks for accepting `<...>` as type arguments in an expression.
//! - [`context`]: Whether a `<` is in a type or an expression. This is the recursive core.
//! - [`runs`]: Entry points for `coalesce`.
//! - [`tsx`]: Entry points for `carve`.

mod bytes;
mod context;
mod runs;
mod tsx;
mod type_list;

pub use runs::{gt_run_split, lt_run_split};
pub use tsx::{jsx_site_is_expression, ts_type_region_open, type_parameter_list_head};

#[cfg(test)]
mod tests;
