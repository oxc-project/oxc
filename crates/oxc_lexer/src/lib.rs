// `oxc_lexer` does not support big endian at present
#![cfg(target_endian = "little")]

use std::mem;

use oxc_span::Span;
use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};

pub mod error;
pub mod options;
mod pipeline;
mod token;
mod tokens;

#[cfg(feature = "oxc_diagnostics")]
pub mod diagnostics;
#[cfg(feature = "testing")]
pub mod testing;

pub use error::{DiagCode, DiagSeverity, Diagnostic};
pub use options::LexOptions;
pub use pipeline::Lexer;
pub use token::{KW_KIND_BASE, SPAN_SENTINELS, TRIVIA_MAX, TRIVIA_MIN, TokenKind, token_flags};
pub use tokens::Tokens;

use pipeline::{Lanes, decode_char_at};

/// Number of bytes of zeros that must follow source text in `src` param passed to [`Lexer::lex`].
pub const PAD: usize = 64;

fn resolve_unicode_leads(
    lanes: &mut Lanes,
    src: &[u8],
    kinds_all: &[TokenKind],
    spans_all: &[Span],
) {
    let k = kinds_all.len();
    let mut leads = mem::take(&mut lanes.unicode_leads);
    let mut ti = 0usize;
    for &off in &leads {
        while ti + 1 < k && spans_all[ti].end <= off {
            ti += 1;
        }
        if off < spans_all[ti].start
            || off >= spans_all[ti].end
            || !candidate_is_code_level(kinds_all[ti])
        {
            continue;
        }
        let Some(ch) = decode_char_at(src, off as usize) else { continue };
        let name_start = spans_all[ti].start
            + u32::from(matches!(
                kinds_all[ti],
                TokenKind::PrivateIdent | TokenKind::PrivateIdentEscaped
            ));
        let ok = if off == name_start { is_identifier_start(ch) } else { is_identifier_part(ch) };
        if ok {
            continue;
        }
        let code = if ch == '\u{FFFD}' {
            // oxc_parser treats a code-level replacement char as a binary file.
            DiagCode::InvalidUtf8
        } else {
            DiagCode::UnexpectedCharacter
        };
        #[expect(clippy::cast_possible_truncation, reason = "char lengths are 1..=4")]
        let len = ch.len_utf8() as u32;
        lanes.out.diags.push(Diagnostic { off, len, code, severity: DiagSeverity::Error });
    }
    leads.clear();
    lanes.unicode_leads = leads;
}

/// Literal interiors, trivia, and JSX text may legally contain any char; only candidates landing in code-level tokens are worth checking.
fn candidate_is_code_level(kind: TokenKind) -> bool {
    !(kind.is_trivia()
        || kind.is_string()
        || kind == TokenKind::RegExp
        || (TokenKind::TemplateNoSub..=TokenKind::TemplateTail).contains(&kind)
        || (TokenKind::TemplateNoSubCooked..=TokenKind::TemplateTailCooked).contains(&kind)
        || kind == TokenKind::JsxText)
}
