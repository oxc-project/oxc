use oxc_ast::ast::{Comment, RegExpFlags};
use oxc_simd::{Cpu, cpu_multiversion, cpu_version};
use oxc_span::Span;

use crate::{
    error::{DiagCode, DiagSeverity, Diagnostic},
    token::StringSpan,
};

mod numbers;
mod strings;

/// The per-token values besides kinds and spans, owned by Tokens and lent to the lexer.
#[derive(Debug, Default)]
pub(crate) struct Lanes {
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

impl Lanes {
    #[cpu_multiversion]
    pub(crate) fn clear(&mut self, _cpu: impl Cpu) {
        self.numbers.clear();
        self.strings.clear();
        self.templates.clear();
        self.atoms.clear();
        self.regex_flags.clear();
        self.cooked.clear();
        self.comments.clear();
        self.diags.clear();
        self.irregular_whitespaces.clear();
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
        self.regex_flags.push(RegExpFlags::from_bits_retain(f));
    }

    /// Cold and compiled once, so the push stays out of the hot loops of every CPU version.
    #[cpu_version(Generic)]
    #[cold]
    #[inline(never)]
    pub(crate) fn push_diag(&mut self, off: u32, len: u32, code: DiagCode) {
        self.diags.push(Diagnostic { off, len, code, severity: DiagSeverity::Error });
    }
}
