use oxc_ast::ast::{Comment, RegExpFlags};
use oxc_simd::{Cpu, cpu_multiversion, cpu_version};
use oxc_span::Span;

use crate::{
    error::{DiagCode, DiagSeverity, Diagnostic},
    token::StringSpan,
};

use crate::pipeline::{carve::JsxFrame, disambiguate::DisambiguateState};

mod numbers;
mod strings;

#[derive(Debug, Default)]
pub(crate) struct TokenLanes {
    pub numbers: Vec<f64>,
    pub strings: Vec<StringSpan>,
    pub templates: Vec<StringSpan>,
    pub atoms: Vec<StringSpan>,
    pub regex_flags: Vec<RegExpFlags>,
    pub cooked: Vec<u8>,
    pub comments: Vec<Comment>,
    /// Lexer diagnostics, pushed only on cold error paths; empty for valid input.
    pub diags: Vec<Diagnostic>,
    /// Code-level whitespace chars other than space, tab, LF and CR, one span per char.
    pub irregular_whitespaces: Vec<Span>,
}

#[derive(Default)]
pub(crate) struct Lanes {
    pub out: TokenLanes,
    /// Ranges lexed content-blind (the tsx type-argument skip), whose diagnostics are dropped.
    pub diag_suppress: Vec<(u32, u32)>,
    /// Non-ASCII lead bytes, checked as identifier chars once the tokens are known.
    pub unicode_leads: Vec<u32>,
    /// Carve's nesting stacks, kept between lexes to reuse their allocations.
    pub(super) template_depth: Vec<u32>,
    pub(super) jsx_stack: Vec<JsxFrame>,
    /// What the disambiguation questions keep across the lex.
    pub(super) disambiguate: DisambiguateState,
}

impl Lanes {
    #[cpu_multiversion]
    pub(crate) fn clear(&mut self, _cpu: impl Cpu) {
        self.out.numbers.clear();
        self.out.strings.clear();
        self.out.templates.clear();
        self.out.atoms.clear();
        self.out.regex_flags.clear();
        self.out.cooked.clear();
        self.out.comments.clear();
        self.out.diags.clear();
        self.diag_suppress.clear();
        self.unicode_leads.clear();
        self.out.irregular_whitespaces.clear();
    }

    #[cpu_multiversion]
    #[inline]
    pub(crate) fn push_regex_flags(&mut self, src: &[u8], fs: usize, fe: usize, _cpu: impl Cpu) {
        let mut f: u8 = 0;
        for (k, &c) in src[fs..fe].iter().enumerate() {
            let bit: u8 = match c {
                b'g' => RegExpFlags::G.bits(),
                b'i' => RegExpFlags::I.bits(),
                b'm' => RegExpFlags::M.bits(),
                b's' => RegExpFlags::S.bits(),
                b'u' => RegExpFlags::U.bits(),
                b'y' => RegExpFlags::Y.bits(),
                b'd' => RegExpFlags::D.bits(),
                b'v' => RegExpFlags::V.bits(),
                _ => 0,
            };
            // Unknown or repeated flag: diagnostic. At most 8 flag chars.
            if bit == 0 {
                self.push_diag((fs + k) as u32, 1, DiagCode::InvalidRegexpFlag);
            } else if f & bit != 0 {
                self.push_diag((fs + k) as u32, 1, DiagCode::DuplicateRegexpFlag);
            }
            f |= bit;
        }
        self.out.regex_flags.push(RegExpFlags::from_bits_retain(f));
    }

    /// Cold and compiled once, so the push stays out of the hot loops of every CPU version.
    #[cpu_version(Generic)]
    #[cold]
    #[inline(never)]
    pub(crate) fn push_diag(&mut self, off: u32, len: u32, code: DiagCode) {
        self.out.diags.push(Diagnostic { off, len, code, severity: DiagSeverity::Error });
    }
}
