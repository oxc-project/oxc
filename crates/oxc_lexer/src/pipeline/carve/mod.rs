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
pub unsafe fn carve(lex: Lex, state: &mut LexState, cpu: impl Cpu) {
    if lex.options.jsx {
        carve_jsx(lex, state, cpu);
    } else {
        carve_js(lex, state, cpu);
    }
}
