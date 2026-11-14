use oxc_simd::{Cpu, cpu_multiversion};

use crate::pipeline::lex::{Lex, LexState};

mod comments;
mod common;
mod js;
mod jsx;

pub(super) use common::Substitutions;
use js::carve_js;
pub(super) use jsx::JsxFrame;
use jsx::carve_jsx;

#[cpu_multiversion]
#[inline(never)]
pub fn carve(lex: Lex, state: &mut LexState, cpu: impl Cpu) {
    // The token markers write unchecked, so cut the handles to the source's blocks and a spare.
    let blocks = lex.src.block_count() + 1;
    let bitmaps = lex.bitmaps.first(blocks);
    let lex = Lex { bitmaps: &bitmaps, kinds: lex.kinds.first(blocks * 64), ..lex };
    if lex.options.jsx {
        carve_jsx(lex, state, cpu);
    } else {
        carve_js(lex, state, cpu);
    }
}
