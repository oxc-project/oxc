// The pedantic and nursery style lints fight the SIMD idiom.
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
mod kinds;
mod lanes;
mod lex;
mod lexer;
mod misc;
mod operators;
mod scan;
mod source;
mod unicode_leads;

pub(crate) use lanes::Lanes;
pub use lexer::Lexer;
