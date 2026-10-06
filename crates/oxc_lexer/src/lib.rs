#![cfg(target_endian = "little")]

use std::{cell::RefCell, mem, ptr, slice};

use oxc_ast::ast::RegExpFlags;
use oxc_span::Span;
use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};

pub mod arena;
mod comment_meta;
#[cfg(feature = "oxc_diagnostics")]
pub mod diagnostics;
pub mod error;
mod lanes;
pub mod options;
mod pipeline;
pub mod token;

pub use arena::{Arena, LexResult, LineEntry};
pub use error::{DiagCode, DiagSeverity, Diagnostic};
pub use lanes::Lanes;
pub use options::LexOptions;
pub use pipeline::Lexer;
pub use token::{KW_KIND_BASE, TRIVIA_MAX, TRIVIA_MIN, TokenKind, token_flags};

pub const PAD: usize = 64;

/// `true` if the SIMD core is compiled in, `false` if the scalar fallback is.
/// Used by CI to ensure it's testing the implementation it thinks it is.
pub const IS_SIMD: bool = cfg!(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
));

thread_local! {
    static SCRATCH: RefCell<Lexer> = RefCell::new(Lexer::new());
}

const _: () = assert!(size_of::<RegExpFlags>() == 1);

/// # Panics
/// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `len`,
/// or if the arena's token buffers are smaller than `len + PAD`.
pub fn lex_utf8_arena(src: &[u8], len: u32, options: LexOptions, arena: &mut Arena) -> LexResult {
    lex_into_arena(src, len, options, arena)
}

/// # Panics
/// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `len`.
pub fn lex_utf8(src: &[u8], len: u32, options: LexOptions) -> (LexResult, Arena) {
    #[expect(clippy::cast_possible_truncation, reason = "PAD is a small constant")]
    let tok_cap = len + PAD as u32;
    let diag_cap =
        if options.max_diagnostic_count > 0 { options.max_diagnostic_count } else { 1024 };
    let line_cap = (len / 32) + 64;
    let mut arena = Arena::new(tok_cap, diag_cap, line_cap);
    let r = lex_utf8_arena(src, len, options, &mut arena);
    (r, arena)
}

fn lex_into_arena(src: &[u8], len: u32, options: LexOptions, arena: &mut Arena) -> LexResult {
    arena.ensure_token_capacity();
    let n = len as usize;

    if arena.tok_kinds.is_null() {
        return empty_result(arena);
    }
    assert!(
        arena.tok_kinds_capacity as usize >= n + PAD
            && arena.tok_spans_capacity as usize >= n + PAD,
        "lexer: arena token capacity too small for source len {n} (tok_kinds={}, tok_spans={}); need >= n + {PAD} \
         - build_spans writes a full 4-lane group past the last token and the pipeline appends EOF sentinels",
        arena.tok_kinds_capacity,
        arena.tok_spans_capacity
    );

    assert!(
        src.len() >= n + PAD,
        "lexer: src must have >= {PAD} bytes of padding past len {n} (got {})",
        src.len()
    );

    SCRATCH.with(|cell| {
        let lx = &mut *cell.borrow_mut();
        // SAFETY: pad and arena capacities asserted above.
        let k = unsafe {
            lx.lex_raw(
                src,
                n,
                arena.tok_kinds,
                arena.tok_spans,
                options.jsx,
                options.ts,
                options.source_type_module,
                options.validate_utf8,
            )
        };

        if !lx.lanes.unicode_leads.is_empty() && k > 0 {
            // SAFETY: `lex_raw` wrote `k` kinds and `k` spans.
            let (kind_bytes, spans_all) = unsafe {
                (
                    slice::from_raw_parts(arena.tok_kinds, k),
                    slice::from_raw_parts(arena.tok_spans, k),
                )
            };
            token::debug_assert_kind_bytes(kind_bytes);
            // SAFETY: every kind the pipeline writes is a declared discriminant.
            let kinds_all = unsafe { token::kinds_from_bytes(kind_bytes) };
            resolve_unicode_leads(&mut lx.lanes, &src[..n], kinds_all, spans_all);
        }

        if !lx.lanes.diag_suppress.is_empty() {
            let (diags, sup) = (&mut lx.lanes.diags, &lx.lanes.diag_suppress);
            diags.retain(|d| !sup.iter().any(|&(a, b)| d.off >= a && d.off < b));
        }

        let l = &lx.lanes;
        let n_num = copy_lane(&l.numbers, arena.numbers, arena.numbers_capacity);
        let n_cb = copy_lane(&l.cooked, arena.cooked_bytes, arena.cooked_bytes_capacity);
        let n_str = copy_lane(&l.strings, arena.strings, arena.strings_capacity);
        let n_tpl = copy_lane(&l.templates, arena.templates, arena.templates_capacity);
        let n_atm = copy_lane(&l.atoms, arena.atoms, arena.atoms_capacity);
        let n_rxf =
            copy_lane(&l.regex_flags, arena.regex_flags.cast::<u8>(), arena.regex_flags_capacity);
        let n_cm = copy_lane(&l.comment_meta, arena.comment_meta, arena.comment_meta_capacity);
        let n_cr = copy_lane(&l.comments, arena.comments, arena.comments_capacity);
        let n_diag = copy_lane(&l.diags, arena.diags, arena.diags_capacity);

        #[expect(
            clippy::cast_possible_truncation,
            reason = "token counts are bounded by MAX_SOURCE_LEN"
        )]
        let token_count = k as u32;

        LexResult {
            diagnostics: arena.diags,
            diagnostic_count: n_diag,
            lines: arena.lines,
            line_count: 0,
            hit_resource_limit: false,
            token_count,
            numbers_count: n_num,
            atoms_count: n_atm,
            strings_count: n_str,
            templates_count: n_tpl,
            regex_flags_count: n_rxf,
            comment_meta_count: n_cm,
            comments_count: n_cr,
            cooked_bytes_count: n_cb,
        }
    })
}

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
        let Some(ch) = lanes::decode_char_at(src, off as usize) else { continue };
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
        lanes.diags.push(Diagnostic { off, len, code, severity: DiagSeverity::Error });
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

fn empty_result(arena: &Arena) -> LexResult {
    LexResult {
        diagnostics: arena.diags,
        diagnostic_count: 0,
        lines: arena.lines,
        line_count: 0,
        hit_resource_limit: true,
        token_count: 0,
        numbers_count: 0,
        atoms_count: 0,
        strings_count: 0,
        templates_count: 0,
        regex_flags_count: 0,
        comment_meta_count: 0,
        comments_count: 0,
        cooked_bytes_count: 0,
    }
}

#[inline]
fn copy_lane<T: Copy>(srcv: &[T], dst: *mut T, cap: u32) -> u32 {
    if dst.is_null() {
        return 0;
    }

    assert!(
        srcv.len() <= cap as usize,
        "lexer: lane overflow ({} entries, capacity {cap}) - arena lane sizing out of date",
        srcv.len()
    );

    // SAFETY: `dst` is non-null with capacity >= srcv.len(), asserted above.
    unsafe {
        ptr::copy_nonoverlapping(srcv.as_ptr(), dst, srcv.len());
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "lane lengths are bounded by u32 capacities"
    )]
    let len = srcv.len() as u32;
    len
}
