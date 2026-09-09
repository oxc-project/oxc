use crate::{lanes::Lanes, tables::Tables};

mod common;
mod js;
mod jsx;

use js::carve_js;
use jsx::carve_jsx;

#[inline]
pub unsafe fn carve(
    t: &Tables,
    srcs: &[u8],
    n: usize,
    st: *mut u64,
    kind: *mut u8,
    opch: *mut u64,
    word: *const u64,
    digit: *mut u64,
    dot: *mut u64,
    kwinit: *mut u64,
    jsx: bool,
    ts: bool,
    lanes: &mut Lanes,
) {
    if jsx {
        carve_jsx(t, srcs, n, st, kind, opch, word, digit, dot, kwinit, ts, lanes);
    } else {
        carve_js(t, srcs, n, st, kind, opch, word, digit, ts, lanes);
    }
}
