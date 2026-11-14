// Raw kernels: the unsafe fn boundary is the reviewed surface, not each operation inside.
#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]

use oxc_simd::{Cpu, DynCpu, cpu_multiversion};

use crate::pipeline::{bitmap::Bitmaps, lex::Lex};

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
pub(super) fn classify(lex: Lex, cpu: impl Cpu) {
    let Lex { src, bitmaps, kinds, options } = lex;
    let blocks = src.block_count();
    let Bitmaps { token_start, word, keyword_initial, op_char, digit, dot, misc } =
        bitmaps.first(blocks);
    let (src_ptr, src_len, kinds_ptr) =
        (src.as_ptr(), src.len(), kinds.first(blocks * 64).as_ptr());
    let (token_start, word, keyword_initial, op_char) =
        (token_start.as_ptr(), word.as_ptr(), keyword_initial.as_ptr(), op_char.as_ptr());
    let (digit, dot, misc) = (digit.as_ptr(), dot.as_ptr(), misc.as_ptr());
    // SAFETY: the kernels write only the blocks just sliced, and read src within its padding.
    unsafe {
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
        }
    };
    bitmaps.clear_block(blocks);
}
