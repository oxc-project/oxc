use oxc_simd::{Cpu, cpu_multiversion};

use crate::options::LexOptions;

use crate::pipeline::{bitmap::Bitmaps, kinds::Kinds, lanes::Lanes, source::Source};

mod comments;
mod common;
mod js;
mod jsx;

use js::carve_js;
pub(super) use jsx::JsxFrame;
use jsx::carve_jsx;

#[cpu_multiversion]
#[inline(never)]
pub unsafe fn carve(
    src: Source,
    bitmaps: &Bitmaps,
    kinds: Kinds,
    options: &LexOptions,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) {
    if options.jsx {
        carve_jsx(src, bitmaps, kinds, options, lanes, cpu);
    } else {
        carve_js(src, bitmaps, kinds, options, lanes, cpu);
    }
}
