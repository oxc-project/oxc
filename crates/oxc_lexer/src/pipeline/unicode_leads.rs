use std::mem;

use oxc_span::Span;
use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};

use crate::{
    error::{DiagCode, DiagSeverity, Diagnostic},
    token::TokenKind,
};

use crate::pipeline::lanes::{Lanes, decode_char_at};

/// Report non-ASCII chars in code which aren't valid identifier chars.
///
/// [`misc_pre`] records the position of each non-ASCII char which isn't whitespace
/// in [`Lanes::unicode_leads`], before it's known which token the char is in.
/// This runs once the tokens are known. It skips chars outside code-level tokens
/// (see [`candidate_is_code_level`]) and invalid UTF-8.
///
/// Each remaining char must pass [`is_identifier_start`] if it starts the token's name
/// (after `#` in a private name), or [`is_identifier_part`] if not.
/// A char which fails gets an [`UnexpectedCharacter`] diagnostic,
/// or [`InvalidUtf8`] if it's U+FFFD, because `oxc_parser` treats that as a binary file.
///
/// Positions are in ascending order, so one forward pass over the tokens
/// finds each char's token. Empties [`Lanes::unicode_leads`], keeping its allocation.
///
/// [`misc_pre`]: crate::pipeline::misc::misc_pre
/// [`UnexpectedCharacter`]: DiagCode::UnexpectedCharacter
/// [`InvalidUtf8`]: DiagCode::InvalidUtf8
pub(super) fn resolve_unicode_leads(
    lanes: &mut Lanes,
    src: &[u8],
    kinds_all: &[TokenKind],
    spans_all: &[Span],
) {
    let k = kinds_all.len();
    let mut leads = mem::take(&mut lanes.unicode_leads);
    let mut ti = 0usize;
    for &off in &leads {
        // Leads can be many tokens apart, so gallop then binary search past the current token
        if ti + 1 < k && spans_all[ti].end <= off {
            let rest = &spans_all[ti + 1..k - 1];
            let (mut lo, mut step) = (0, 1);
            while lo + step <= rest.len() && rest[lo + step - 1].end <= off {
                lo += step;
                step *= 2;
            }
            let hi = rest.len().min(lo + step);
            ti += 1 + lo + rest[lo..hi].partition_point(|span| span.end <= off);
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

/// Literal interiors, trivia, and JSX text may legally contain any char.
/// Only candidates landing in code-level tokens are worth checking.
fn candidate_is_code_level(kind: TokenKind) -> bool {
    !(kind.is_trivia()
        || kind.is_string()
        || kind == TokenKind::RegExp
        || (TokenKind::TemplateNoSub..=TokenKind::TemplateTail).contains(&kind)
        || (TokenKind::TemplateNoSubCooked..=TokenKind::TemplateTailCooked).contains(&kind)
        || kind == TokenKind::JsxText)
}
