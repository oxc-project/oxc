// Kernel lint policy (this module tree and `lanes`):
// the `unsafe fn` boundary is the reviewed surface, and the pedantic/nursery
// style lints fight the SIMD idiom. API modules keep the full workspace bar.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(clippy::needless_range_loop, clippy::manual_range_contains)]

use std::slice;

use oxc_simd::{Cpu, cpu_multiversion};

use crate::PAD;

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
mod unicode_leads;

pub(crate) use lanes::TokenLanes;
pub use lexer::Lexer;

use disambiguate::{Brackets, Closers, TokensView};

/// The lex in progress as `disambiguate` reads it, over the buffers [`Lexer::ensure`] sized.
///
/// # SAFETY
///
/// - src must be valid for n + PAD bytes, st and opch for n / 64 + 1 words and
///   `kind` for `(n / 64 + 1) * 64` bytes.
/// - Nothing may write to them while the view is alive: a question reads, answers and returns
///   before the stage writes again.
///
/// `brackets` is the lex's bracket cache, `lanes.disambiguate.brackets`.
#[cpu_multiversion]
#[inline(always)]
unsafe fn token_view<'a>(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    ts: bool,
    kw_final: usize,
    module: bool,
    brackets: &'a Brackets,
    closers: &'a Closers,
    _cpu: impl Cpu,
) -> TokensView<'a> {
    let nb = n.div_ceil(64) + 1;
    TokensView {
        src: slice::from_raw_parts(src, n + PAD),
        st: slice::from_raw_parts(st, nb),
        opch: slice::from_raw_parts(opch, nb),
        kind: slice::from_raw_parts(kind, nb * 64),
        n,
        ts,
        module,
        kw_final,
        brackets,
        closers,
    }
}
