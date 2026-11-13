use oxc_simd::{Cpu, DynCpu, cpu_multiversion};

use crate::pipeline::bitmap::Bitmap;

mod avx2;
mod generic;
mod luts;
mod neon;
mod punct;

use avx2::classify_avx2;
use generic::classify_generic;
use neon::classify_neon;

#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn classify(
    ts: bool,
    src: *const u8,
    n: usize,
    nb: usize,
    word: Bitmap,
    st: Bitmap,
    kwinit: Bitmap,
    opch: Bitmap,
    digit: Bitmap,
    dot: Bitmap,
    misc: Bitmap,
    kind: *mut u8,
    cpu: impl Cpu,
) {
    let (word, st, kwinit, opch) = (word.as_ptr(), st.as_ptr(), kwinit.as_ptr(), opch.as_ptr());
    let (digit, dot, misc) = (digit.as_ptr(), dot.as_ptr(), misc.as_ptr());
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => {
            classify_avx2(ts, src, n, word, st, kwinit, opch, digit, dot, misc, kind, cpu)
        }
        DynCpu::Neon(cpu) => {
            classify_neon(ts, src, n, word, st, kwinit, opch, digit, dot, misc, kind, cpu)
        }
        DynCpu::Generic(_) => {
            classify_generic(ts, src, n, word, st, kwinit, opch, digit, dot, misc, kind)
        }
    };

    *word.add(nb) = 0;
    *st.add(nb) = 0;
    *kwinit.add(nb) = 0;
    *opch.add(nb) = 0;
    *digit.add(nb) = 0;
    *dot.add(nb) = 0;
    *misc.add(nb) = 0;
}
