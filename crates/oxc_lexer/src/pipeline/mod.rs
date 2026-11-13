// Kernel lint policy (this module tree and `lanes`):
// the `unsafe fn` boundary is the reviewed surface, and the pedantic/nursery
// style lints fight the SIMD idiom. API modules keep the full workspace bar.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(clippy::needless_range_loop, clippy::manual_range_contains)]

mod bitmap;
mod bytes;
mod carve;
mod chunk;
mod classify;
mod coalesce;
mod compress;
mod disambiguate;
mod find;
mod keywords;
mod lanes;
mod lexer;
mod misc;
mod operators;
mod scan;
mod source;
mod unicode_leads;

pub(crate) use lanes::TokenLanes;
pub use lexer::Lexer;
