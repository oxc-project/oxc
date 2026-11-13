use oxc_simd::{Cpu, cpu_multiversion};

use crate::pipeline::{lanes::Lanes, lex::Lex};

mod comments;
mod common;
mod js;
mod jsx;

use js::carve_js;
pub(super) use jsx::JsxFrame;
use jsx::carve_jsx;

#[cpu_multiversion]
#[inline(never)]
pub unsafe fn carve(lex: Lex, lanes: &mut Lanes, cpu: impl Cpu) {
    if lex.options.jsx {
        carve_jsx(lex, lanes, cpu);
    } else {
        carve_js(lex, lanes, cpu);
    }
}
