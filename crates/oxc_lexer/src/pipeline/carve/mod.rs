use oxc_simd::{Cpu, cpu_multiversion};

use crate::pipeline::{bitmap::Bitmaps, kinds::Kinds, lanes::Lanes, source::Source};

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
    src: Source,
    bitmaps: &Bitmaps,
    kinds: Kinds,
    jsx: bool,
    ts: bool,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) {
    if jsx {
        carve_jsx(src, bitmaps, kinds, ts, lanes, cpu);
    } else {
        carve_js(src, bitmaps, kinds, ts, lanes, cpu);
    }
}
