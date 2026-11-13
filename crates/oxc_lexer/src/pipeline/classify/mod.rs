use oxc_simd::{Cpu, DynCpu, cpu_multiversion};

use crate::options::LexOptions;

use crate::pipeline::{bitmap::Bitmaps, kinds::Kinds, source::Source};

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
    src: Source,
    bitmaps: &Bitmaps,
    kinds: Kinds,
    options: &LexOptions,
    cpu: impl Cpu,
) {
    let Bitmaps { token_start, word, keyword_initial, op_char, digit, dot, misc } = *bitmaps;
    let (src_ptr, src_len, kinds_ptr) = (src.as_ptr(), src.len(), kinds.as_ptr());
    let (token_start, word, keyword_initial, op_char) =
        (token_start.as_ptr(), word.as_ptr(), keyword_initial.as_ptr(), op_char.as_ptr());
    let (digit, dot, misc) = (digit.as_ptr(), dot.as_ptr(), misc.as_ptr());
    match DynCpu::from(cpu) {
        DynCpu::Avx2(cpu) => classify_avx2(
            options.ts,
            src_ptr,
            src_len,
            word,
            token_start,
            keyword_initial,
            op_char,
            digit,
            dot,
            misc,
            kinds_ptr,
            cpu,
        ),
        DynCpu::Neon(cpu) => classify_neon(
            options.ts,
            src_ptr,
            src_len,
            word,
            token_start,
            keyword_initial,
            op_char,
            digit,
            dot,
            misc,
            kinds_ptr,
            cpu,
        ),
        DynCpu::Generic(_) => classify_generic(
            options.ts,
            src_ptr,
            src_len,
            word,
            token_start,
            keyword_initial,
            op_char,
            digit,
            dot,
            misc,
            kinds_ptr,
        ),
    };
    bitmaps.clear_block(src.block_count());
}
