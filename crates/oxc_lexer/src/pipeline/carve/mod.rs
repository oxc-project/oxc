use crate::pipeline::{
    cpu::{Cpu, apply, cpu_versions},
    lanes::Lanes,
};

mod comments;
mod common;
mod js;
mod jsx;

use js::carve_js;
use jsx::carve_jsx;

#[apply(cpu_versions)]
#[inline(never)]
pub unsafe fn carve(
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
    cpu: impl Cpu,
) {
    if jsx {
        carve_jsx(srcs, n, st, kind, opch, word, digit, dot, kwinit, ts, lanes, cpu);
    } else {
        carve_js(srcs, n, st, kind, opch, word, ts, lanes, cpu);
    }
}
