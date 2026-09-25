//! Would TypeScript accept `<...>` as type arguments in an expression?
//!
//! In an expression, TypeScript reads `f<T>(x)` as a call with type arguments,
//! but `a < b > c` as comparisons.
//! It decides by trying to parse type arguments, and keeps the result only if both checks here pass:
//!
//! - [`type_list_legal`]: Do the tokens between `<` and `>` form a list of types?
//!   `a<b + 1, c>` doesn't, because `+` can't appear in a type.
//! - [`gt_follower`]: Can the token after the `>` follow type arguments?
//!   A `(` can, as in `f<T>(x)`. An identifier can't, as in `a < b > c`.
//!   When the token alone doesn't settle it, the answer is [`Follow::Ctx`], and the caller decides.

use crate::{
    pipeline::bytes::{
        block_comment_end, is_digit, is_id_start, line_break_in, line_terminator_after,
        unicode_ws_len_at,
    },
    token::{OP_KIND_BASE, is_trivia_byte, matches_tk, tk},
};

use crate::pipeline::disambiguate::common::{Tokens, bits, kind_at, word_is_any, word_len};

use super::bytes::{raw_template_end, scan_list_closer, skip_raw_literal, skip_ws_fwd};

const FOLLOW_SPLIT_WORDS: &[&[u8]] =
    &[b"in", b"instanceof", b"as", b"satisfies", b"extends", b"implements"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Follow {
    Split,
    Fuse,
    Ctx,
}

fn gt_follower(src: &[u8], n: usize, mut i: usize) -> Follow {
    let mut broke = false;
    loop {
        if i >= n {
            return Follow::Split;
        }
        let c = src[i];
        match c {
            b' ' | b'\t' | 0x0b | 0x0c => {
                i += 1;
                continue;
            }
            b'\n' | b'\r' => {
                broke = true;
                i += 1;
                continue;
            }
            b'/' => {
                let d = src[i + 1];
                if d == b'/' {
                    broke = true;
                    i = line_terminator_after(src, n, i + 2);
                    continue;
                }
                if d != b'*' {
                    return Follow::Split;
                }
                let e = block_comment_end(src, n, i + 2);
                if e >= n {
                    return Follow::Split;
                }
                if line_break_in(src, i + 2, e) {
                    broke = true;
                }
                i = e + 1;
                continue;
            }
            _ => {}
        }
        if c >= 0x80 {
            if c == 0xe2 && src[i + 1] == 0x80 && (src[i + 2] == 0xa8 || src[i + 2] == 0xa9) {
                broke = true;
                i += 3;
                continue;
            }
            let wl = unicode_ws_len_at(src, i);
            if wl != 0 {
                i += wl;
                continue;
            }
            return if broke { Follow::Split } else { Follow::Fuse };
        }
        let nx = src[i + 1];
        if broke {
            return match c {
                b'<' if nx != b'<' && nx != b'=' => Follow::Ctx,
                b'+' | b'-' if nx != b'=' && nx != c => Follow::Ctx,
                b'>' => Follow::Ctx,
                _ => Follow::Split,
            };
        }
        return match c {
            b'(' | b'`' | b'=' | b')' | b']' | b'}' | b',' | b';' | b':' | b'?' | b'|' | b'&'
            | b'*' | b'%' | b'^' => Follow::Split,
            b'!' => {
                if nx == b'=' {
                    Follow::Split
                } else {
                    Follow::Fuse
                }
            }
            b'.' => {
                if is_digit(nx) {
                    Follow::Fuse
                } else {
                    Follow::Split
                }
            }
            b'{' | b'[' | b'>' => Follow::Ctx,
            b'+' | b'-' => {
                if nx == b'=' {
                    Follow::Split
                } else {
                    Follow::Fuse
                }
            }
            b'<' => {
                if nx == b'<' || nx == b'=' {
                    Follow::Split
                } else {
                    Follow::Fuse
                }
            }
            b'~' | b'@' | b'#' | b'"' | b'\'' => Follow::Fuse,
            // An identifier written with a leading Unicode escape starts an expression like any
            // other name.
            b'\\' => {
                if nx == b'u' {
                    Follow::Fuse
                } else {
                    Follow::Split
                }
            }
            _ => {
                if is_digit(c) {
                    Follow::Fuse
                } else if is_id_start(c) {
                    if word_is_any(src, i, FOLLOW_SPLIT_WORDS) {
                        Follow::Split
                    } else {
                        Follow::Fuse
                    }
                } else {
                    Follow::Split
                }
            }
        };
    }
}

#[rustfmt::skip::macros(matches_tk)]
fn type_list_legal(tokens: &Tokens, lo: usize, hi: usize) -> bool {
    let Tokens { tables: t, src, st, kind, .. } = *tokens;
    let mut start = true;
    let mut braces: i32 = 0;
    let mut brackets: i32 = 0;
    let mut angle_bits: u64 = 0;
    let mut angle_depth: u32 = 0;
    let mut paren_ok = false;
    let mut cond_ok = false;
    let mut parens: i32 = 0;
    let mut this_head = false;
    // Keyword kind of the previous token (0 when it was not a keyword) and whether that keyword
    // is a whole type that no `.` may follow.
    let mut prev_kw: u8 = 0;
    let mut no_dot = false;
    // A list element starts here: after the `<` or a `,` of a type-argument list.
    let mut elem_start = true;
    let mut skip = usize::MAX;
    // Start of the previous significant token: a type reference takes no arguments across a
    // line break, so a `<` after one is checked against it.
    let mut prev = usize::MAX;
    let mut w = bits::next1(st, lo, hi);
    while w < hi {
        let mut k = kind_at(kind, w);
        if w == skip || is_trivia_byte(k) {
            w = bits::next1(st, w + 1, hi);
            continue;
        }
        // Text carve has not reached yet: a raw comment is trivia, a raw string or template is a
        // literal type.
        let j = skip_raw_literal(src, kind, hi, w);
        if j != w {
            let c0 = src[w];
            if c0 == b'/' {
                w = bits::next1(st, j, hi);
                continue;
            }
            k = if c0 == b'`' { tk!(TemplateNoSub) } else { tk!(String) };
            if !start && braces == 0 {
                return false;
            }
            start = false;
            paren_ok = false;
            prev = w;
            prev_kw = 0;
            no_dot = false;
            w = bits::next1(st, j, hi);
            continue;
        }
        let c = src[w];
        let mut ok_paren = false;
        let was_this = this_head;
        this_head = false;
        let was_no_dot = no_dot;
        no_dot = false;
        let last_kw = prev_kw;
        prev_kw = 0;
        let was_elem_start = elem_start;
        elem_start = false;
        if matches_tk!(k, Ident | IdentEscaped) {
            let kk = t.keywords.kwts.lookup_at(src, w, word_len(src, w)) as u8;
            // A keyword type (`this`, `any`, `null`, ...) takes no type arguments; in a type
            // query it names a value, which may (`typeof this<A>`).
            this_head = matches_tk!(kk,
                            KwThis | KwAny | KwUnknown | KwString | KwNumber | KwBoolean | KwSymbol
                            | KwObject | KwNever | KwUndefined | KwNull | KwVoid | KwTrue | KwFalse
                            | KwBigInt
                        )
                && last_kw != tk!(KwTypeof);
            if !start && braces == 0 && !matches_tk!(kk, KwExtends | KwIs | KwIn) {
                return false;
            }
            // tsc checks `isStartOfType` only where a list element starts (after `<` or a
            // `,` of the list): a reserved word there is no type. Elsewhere a keyword is read
            // as a name: a property (`{ return: T }`), a reference after `=>`, `:`, `|`. `super`
            // names a value only in a type query (`typeof super.x`).
            if was_elem_start
                && type_illegal_kind(kk)
                && !(kk == tk!(KwSuper) && last_kw == tk!(KwTypeof))
            {
                return false;
            }
            if kk == tk!(KwExtends) {
                cond_ok = true;
            }
            // `this`, `null`, `true`, `false` and `void` are whole types: a `.` after one is
            // a member access, not a qualified name (`any.x` and `string.x` are references).
            no_dot = last_kw != tk!(KwTypeof)
                && matches_tk!(kk, KwThis | KwNull | KwTrue | KwFalse | KwVoid);
            prev_kw = kk;
            start = type_prefix_kind(kk);
        } else if matches_tk!(k, Number | BigInt | String | TemplateNoSub | TemplateTail) {
            if !start && braces == 0 && k != tk!(TemplateTail) {
                return false;
            }
            start = false;
        } else if matches_tk!(k, TemplateHead | TemplateMiddle) {
            if k == tk!(TemplateHead) && !start && braces == 0 {
                return false;
            }
            // Asked from the JSX carve, the head is carved but the rest of the template is raw
            // text; its substitution `}` is still a punctuator. Read the literal as a whole then
            // (a template type), instead of its tail as tokens.
            if k == tk!(TemplateHead) {
                let (close, end) = raw_template_end(src, hi, w);
                if close < hi && kind_at(kind, close) >= OP_KIND_BASE {
                    if end > hi {
                        return false;
                    }
                    start = false;
                    paren_ok = false;
                    prev = w;
                    w = bits::next1(st, end, hi);
                    continue;
                }
            }
            start = true;
        } else if k >= OP_KIND_BASE {
            match c {
                b'(' => {
                    if !start && braces == 0 && !paren_ok {
                        return false;
                    }
                    parens += 1;
                    start = true;
                }
                b')' => {
                    parens -= 1;
                    start = false;
                }
                b']' => {
                    brackets -= 1;
                    start = false;
                }
                b'[' => {
                    brackets += 1;
                    start = true;
                }
                b'{' => {
                    if !start && braces == 0 {
                        return false;
                    }
                    braces += 1;
                    start = true;
                }
                b'}' => {
                    braces -= 1;
                    start = false;
                }
                b'<' => {
                    let nx = src[w + 1];
                    if was_this || nx == b'=' || (nx == b'<' && !bits::get(st, w + 1)) {
                        return false;
                    }
                    if !start && prev != usize::MAX && line_break_in(src, prev, w) {
                        return false;
                    }
                    angle_bits = (angle_bits << 1) | u64::from(start);
                    angle_depth += 1;
                    start = true;
                    elem_start = true;
                }
                b'>' => {
                    let nx = src[w + 1];
                    if (nx == b'=' || nx == b'>') && !bits::get(st, w + 1) {
                        return false;
                    }
                    if angle_depth > 0 {
                        ok_paren = angle_bits & 1 != 0;
                        angle_bits >>= 1;
                        angle_depth -= 1;
                    }
                    start = false;
                }
                b'=' => {
                    // `=>` of a function type, or a type-parameter default (the walk also reads
                    // member type-parameter lists here).
                    if src[w + 1] == b'>' && bits::get(st, w + 1) {
                        skip = w + 1;
                    }
                    start = true;
                }
                b':' => start = true,
                b'.' => {
                    if was_no_dot {
                        return false;
                    }
                    start = true;
                }
                b',' => {
                    if angle_depth == 0 && braces == 0 && brackets == 0 && parens == 0 {
                        cond_ok = false;
                    }
                    if braces == 0 && brackets == 0 && parens == 0 {
                        elem_start = true;
                    }
                    start = true;
                }
                b'|' | b'&' => {
                    if src[w + 1] == c {
                        return false;
                    }
                    start = true;
                }
                b'?' => {
                    let nx = src[w + 1];
                    if nx == b'.' || nx == b'?' {
                        return false;
                    }
                    if braces == 0 && brackets == 0 {
                        let optional = parens > 0
                            && matches!(src[skip_ws_fwd(src, w + 1, hi)], b':' | b',' | b')');
                        if !optional {
                            if !cond_ok {
                                return false;
                            }
                            cond_ok = false;
                        }
                    }
                    start = true;
                }
                b';' => {
                    if braces == 0 {
                        return false;
                    }
                    start = true;
                }
                b'-' => {
                    if !start && braces == 0 {
                        return false;
                    }
                    start = true;
                }
                b'+' => {
                    if braces == 0 {
                        return false;
                    }
                    start = true;
                }
                _ => return false,
            }
        } else {
            return false;
        }
        paren_ok = ok_paren;
        prev = w;
        w = bits::next1(st, w + 1, hi);
    }
    true
}

#[inline(always)]
#[rustfmt::skip::macros(matches_tk)]
fn type_illegal_kind(k: u8) -> bool {
    matches_tk!(
        k,
        KwAwait | KwYield | KwDelete | KwFunction | KwClass | KwInstanceof | KwSuper | KwSwitch
        | KwCase | KwReturn | KwThrow | KwVar | KwConst | KwIf | KwElse | KwFor | KwWhile | KwDo
        | KwBreak | KwContinue | KwWith | KwTry | KwCatch | KwFinally | KwDebugger | KwDefault
        | KwExport | KwEnum
    )
}

/// TypeScript's speculative parse of a type-argument list at the `<` at `lt` in expression
/// position: a balanced list whose contents are types and whose closer is followed by a token that
/// cannot start an expression. In a type, every `<` after a name opens a list, so a `<` this
/// accepts opens one in any context.
pub(crate) fn type_args_at(tokens: &Tokens, lt: usize) -> bool {
    let closers = tokens.closers;
    if let Some(r) = closers.get(lt) {
        return r.args.unwrap_or_else(|| {
            let yes = r.closer().is_some_and(|gt| list_is_type_args(tokens, lt, gt));
            closers.set_args(lt, yes);
            yes
        });
    }
    let yes = scan_list_closer(tokens, lt).is_some_and(|gt| list_is_type_args(tokens, lt, gt));
    closers.set_args(lt, yes);
    yes
}

fn list_is_type_args(tokens: &Tokens, lt: usize, gt: usize) -> bool {
    if matches!(tokens.src[gt + 1], b'=' | b'>') {
        return false;
    }
    match gt_follower(tokens.src, tokens.n, gt + 1) {
        Follow::Split => type_list_legal(tokens, lt + 1, gt),
        Follow::Fuse | Follow::Ctx => false,
    }
}

#[inline(always)]
#[rustfmt::skip::macros(matches_tk)]
fn type_prefix_kind(k: u8) -> bool {
    matches_tk!(
        k,
        KwKeyof | KwTypeof | KwReadonly | KwUnique | KwInfer | KwAbstract | KwNew | KwAsserts
        | KwImport | KwExtends | KwIs | KwIn | KwAs
    )
}
