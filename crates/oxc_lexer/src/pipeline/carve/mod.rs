use oxc_simd::{Cpu, cpu_multiversion};

use crate::pipeline::{bitmap::Bitmap, lanes::Lanes};

mod comments;
mod common;
mod js;
mod jsx;

use js::carve_js;
pub(super) use jsx::JFrame;
use jsx::carve_jsx;

#[cpu_multiversion]
#[inline(never)]
pub unsafe fn carve(
    srcs: &[u8],
    n: usize,
    st: Bitmap,
    kind: *mut u8,
    opch: Bitmap,
    word: Bitmap,
    digit: Bitmap,
    dot: Bitmap,
    kwinit: Bitmap,
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
