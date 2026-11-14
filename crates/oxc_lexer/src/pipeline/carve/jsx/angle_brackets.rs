use oxc_simd::{Cpu, cpu_multiversion};

use crate::{error::DiagCode, token::tk};

use crate::pipeline::{
    bitmap::{Bitmap, Bitmaps},
    bytes::{is_id_start, is_word, line_break_in},
    disambiguate::DisambiguateState,
    find::find,
    lanes::Lanes,
    lex::Lex,
    source::Source,
};

use super::{
    super::{
        comments::lex_comment_at,
        common::{OPENER_BYTES, OPENER8_BYTES, Substitutions, lex_string},
    },
    jsx_punct,
    names::jsx_skip_trivia_fast,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum AngleVerdict {
    TypeParams,
    Jsx,
    Ambiguous { gt: usize, lp: usize },
}

#[cpu_multiversion]
#[inline]
pub(super) unsafe fn jsx_over_type_params(
    lex: Lex,
    lt: usize,
    name_pos: usize,
    lanes: &mut Lanes,
    disambiguate: &mut DisambiguateState,
    cpu: impl Cpu,
) -> bool {
    if !lex.options.ts {
        return true;
    }
    match ts_angle_verdict(lex.src, name_pos, lex.bitmaps.word, cpu) {
        AngleVerdict::TypeParams => false,
        AngleVerdict::Jsx => true,
        AngleVerdict::Ambiguous { gt, lp } => {
            let (jsx, unterminated) = disambiguate.jsx_over_generic(lex, lt, lp, cpu);
            if unterminated {
                lanes.push_diag(lt as u32, (gt + 1 - lt) as u32, DiagCode::UnterminatedJsxElement);
            }
            jsx
        }
    }
}

/// Lex the type-argument list at s, carving literals so a < or > inside one leaves the depth alone.
#[cpu_multiversion]
#[cold]
#[inline(never)]
pub(super) unsafe fn lex_type_arguments(
    lex: Lex,
    s: usize,
    lanes: &mut Lanes,
    cpu: impl Cpu,
) -> usize {
    let Lex { src, bitmaps, kinds, .. } = lex;
    let Bitmaps { token_start, op_char, .. } = *bitmaps;
    jsx_punct(kinds, op_char, s, tk!(Lt), cpu);
    let mut depth = 1i32;
    let mut subs = Substitutions::default();
    let mut p = s + 1;
    while p < src.len() && depth != 0 {
        // Braces only matter inside a template substitution.
        let q = if subs.is_empty() {
            find(src, p, OPENER_BYTES, cpu)
        } else {
            find(src, p, OPENER8_BYTES, cpu)
        };
        if q >= src.len() {
            return src.len();
        }
        p = match src[q] {
            b'<' => {
                depth += 1;
                jsx_punct(kinds, op_char, q, tk!(Lt), cpu);
                q + 1
            }
            b'>' => {
                // The > of an arrow in a function type closes nothing.
                if !(q > 0 && src[q - 1] == b'=') {
                    depth -= 1;
                    jsx_punct(kinds, op_char, q, tk!(Gt), cpu);
                }
                q + 1
            }
            b'"' | b'\'' => lex_string(src, token_start, kinds, q, src[q], lanes, cpu),
            b'`' => subs.lex_template(src, token_start, kinds, q, lanes, cpu),
            b'{' if !subs.is_empty() => {
                subs.open_brace();
                q + 1
            }
            b'}' if !subs.is_empty() => subs.close_brace(src, token_start, kinds, q, lanes, cpu),
            // A comment is trivia, and a lone slash cannot start a type.
            b'/' => lex_comment_at(src, token_start, kinds, q, lanes, cpu),
            _ => q + 1,
        };
    }
    p
}

/// `.tsx` disambiguation: at an operand-position `<IDENT...`, is this a TS
/// type-parameter list rather than a JSX element? In `.tsx` a bare `<T>` is
/// JSX (a generic arrow must be written `<T,>`), so the signals are a
/// trailing `,`, a default `=`, or an `extends` constraint. Bounded forward
/// peek; the source pad makes the look-aheads safe past the end.
///
/// Identifier runs come off the `word` bitmap, not `is_word` on the raw
/// byte. `is_word` accepts every byte >= 0x80, so non-ASCII whitespace in
/// the head glued into the first identifier and hid the signal; `misc_pre`
/// has already cleared `word` across Unicode whitespace by the time
/// `carve_jsx` runs, so the corrected run is free to read and no byte has to
/// be re-scanned for a leading 0x80.
#[cpu_multiversion]
#[inline]
unsafe fn ts_angle_verdict(src: Source, t: usize, word: Bitmap, cpu: impl Cpu) -> AngleVerdict {
    let mut p = t;
    // optional `const` type-parameter modifier: `<const T,>`. Like any modifier it must stay
    // on the line of its parameter; a comment between them is fine.
    if src.len() - p >= 6 && &src[p..p + 5] == b"const" && !is_word(src[p + 5], cpu) {
        let qq = jsx_skip_trivia_fast(src, p + 5, cpu);
        // A modifier must be on the same line as its parameter, so `const` followed by a line break
        // can only be a JSX tag name, e.g. `<const\nextends U>` is JSX.
        if line_break_in(src.padded(), p + 5, qq, cpu) {
            return AngleVerdict::Jsx;
        }
        if qq < src.len() && is_id_start(src[qq], cpu) {
            p = qq; // `const` was a modifier; advance to the real param
        }
    }
    while p < src.len() && word.get(p, cpu) {
        p += 1; // first type-parameter identifier
    }
    // The signal may sit behind whitespace (Unicode too) or a comment: `<T /*c*/ extends U>`.
    p = jsx_skip_trivia_fast(src, p, cpu);
    if p >= src.len() {
        return AngleVerdict::Jsx;
    }
    let c = src[p];
    if c == b',' || c == b'=' {
        return AngleVerdict::TypeParams; // `<T,>`  `<T,U>`  `<T = D>`
    }
    if c == b'>' {
        let q = jsx_skip_trivia_fast(src, p + 1, cpu);
        let v = if q < src.len() && src[q] == b'(' {
            AngleVerdict::Ambiguous { gt: p, lp: q }
        } else {
            AngleVerdict::Jsx
        };
        return v;
    }
    // `extends` is also a legal JSX attribute name; it signals a generic only
    // as a full word not followed by `=` (attr value) or `>` (boolean attr).
    if src.len() - p >= 7 && &src[p..p + 7] == b"extends" && !word.get(p + 7, cpu) {
        let qq = jsx_skip_trivia_fast(src, p + 7, cpu);
        let d = if qq < src.len() { src[qq] } else { 0 };
        if d == b'/' && !matches!(if qq + 1 < src.len() { src[qq + 1] } else { 0 }, b'*' | b'/') {
            return AngleVerdict::Jsx;
        }
        let v = if d == b'=' || d == b'>' { AngleVerdict::Jsx } else { AngleVerdict::TypeParams };
        return v;
    }
    AngleVerdict::Jsx
}
