#![cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]
#![allow(unsafe_code)]

pub mod arena;
mod comment_meta;
#[cfg(feature = "oxc_diagnostics")]
pub mod diagnostics;
pub mod error;
mod lanes;
mod opmap;
pub mod options;
mod pipeline;
mod tables;
pub mod token;

pub use arena::{Arena, LexResult, LineEntry};
pub use error::{Diagnostic, diag_code, diag_severity};
pub use lanes::Lanes;
pub use options::{LexOptions, default_options};
pub use pipeline::Lexer;
pub use token::{
    TRIVIA_MAX, TRIVIA_MIN, is_string_kind, is_trivia, token_flags, token_kind, token_kind_name,
};

use core::cell::RefCell;

/// Zero-byte padding required past the lexed length `n`: the SIMD scanners over-read (but never act on) up to this many bytes.
pub const PAD: usize = 64;

thread_local! {
    static SCRATCH: RefCell<Lexer> = RefCell::new(Lexer::new());
}

const _: () = assert!(core::mem::size_of::<oxc_ast::ast::RegExpFlags>() == 1);

/// # Panics
/// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `len`,
/// or if the arena's token buffers are smaller than `len + PAD`.
pub fn lex_utf8_arena(src: &[u8], len: u32, options: LexOptions, arena: &mut Arena) -> LexResult {
    lex_into_arena(src, len, options, arena)
}

/// # Panics
/// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `len`.
#[expect(clippy::cast_possible_truncation, reason = "PAD is a small constant")]
pub fn lex_utf8(src: &[u8], len: u32, options: LexOptions) -> (LexResult, Arena) {
    let tok_cap = len + PAD as u32;
    let diag_cap =
        if options.max_diagnostic_count > 0 { options.max_diagnostic_count } else { 1024 };
    let line_cap = (len / 32) + 64;
    let mut arena = Arena::new(tok_cap, diag_cap, line_cap);
    let r = lex_utf8_arena(src, len, options, &mut arena);
    (r, arena)
}

#[expect(clippy::cast_possible_truncation, reason = "token counts are bounded by MAX_SOURCE_LEN")]
fn lex_into_arena(src: &[u8], len: u32, options: LexOptions, arena: &mut Arena) -> LexResult {
    arena.ensure_token_capacity();
    let n = len as usize;

    if arena.tok_kinds.is_null() {
        return empty_result(arena);
    }
    assert!(
        arena.tok_kinds_capacity as usize >= n + PAD
            && arena.tok_starts_capacity as usize >= n + PAD,
        "lexer: arena token capacity too small for source len {n} (tok_kinds={}, tok_starts={}); need >= n + {PAD} \
         — compress writes a full 8-lane group past the last token and the pipeline appends an EOF sentinel",
        arena.tok_kinds_capacity,
        arena.tok_starts_capacity
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
                arena.tok_starts,
                options.jsx,
                options.ts,
                options.validate_utf8,
            )
        };

        if !lx.lanes.unicode_leads.is_empty() && k > 0 {
            // SAFETY: `lex_raw` wrote `k` kinds and `k + 1` starts.
            let (kinds_all, starts_all) = unsafe {
                (
                    core::slice::from_raw_parts(arena.tok_kinds, k),
                    core::slice::from_raw_parts(arena.tok_starts, k + 1),
                )
            };
            resolve_unicode_leads(&mut lx.lanes, &src[..n], kinds_all, starts_all);
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

        LexResult {
            diagnostics: arena.diags,
            diagnostic_count: n_diag,
            lines: arena.lines,
            line_count: 0,
            hit_resource_limit: false,
            token_count: k as u32,
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

#[expect(clippy::cast_possible_truncation, reason = "char lengths are 1..=4")]
fn resolve_unicode_leads(lanes: &mut Lanes, src: &[u8], kinds_all: &[u8], starts_all: &[u32]) {
    let k = kinds_all.len();
    let mut leads = core::mem::take(&mut lanes.unicode_leads);
    let mut ti = 0usize;
    for &off in &leads {
        while ti + 1 < k && token::offset(starts_all[ti + 1]) <= off {
            ti += 1;
        }
        if !candidate_is_code_level(kinds_all[ti]) {
            continue;
        }
        let Some(ch) = lanes::decode_char_at(src, off as usize) else { continue };
        if oxc_syntax::identifier::is_identifier_part(ch) {
            continue;
        }
        let code = if ch == '\u{FFFD}' {
            // oxc_parser treats a code-level replacement char as a binary file.
            error::diag_code::INVALID_UTF8
        } else {
            error::diag_code::UNEXPECTED_CHARACTER
        };
        lanes.diags.push(error::Diagnostic {
            off,
            len: ch.len_utf8() as u32,
            code,
            severity: error::diag_severity::ERROR,
        });
    }
    leads.clear();
    lanes.unicode_leads = leads;
}

/// Literal interiors, trivia, and JSX text may legally contain any char; only candidates landing in code-level tokens are worth checking.
fn candidate_is_code_level(kind: u8) -> bool {
    use token::token_kind as T;
    !(is_trivia(kind)
        || is_string_kind(kind)
        || kind == T::REGEXP
        || (T::TEMPLATE_NO_SUB..=T::TEMPLATE_TAIL).contains(&kind)
        || (T::TEMPLATE_NO_SUB_COOKED..=T::TEMPLATE_TAIL_COOKED).contains(&kind)
        || kind == T::JSX_TEXT)
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
#[expect(clippy::cast_possible_truncation, reason = "lane lengths are bounded by u32 capacities")]
fn copy_lane<T: Copy>(srcv: &[T], dst: *mut T, cap: u32) -> u32 {
    if dst.is_null() {
        return 0;
    }

    assert!(
        srcv.len() <= cap as usize,
        "lexer: lane overflow ({} entries, capacity {cap}) — arena lane sizing out of date",
        srcv.len()
    );
    // SAFETY: `dst` is non-null with capacity >= srcv.len(), asserted above.
    unsafe {
        core::ptr::copy_nonoverlapping(srcv.as_ptr(), dst, srcv.len());
    }
    srcv.len() as u32
}
