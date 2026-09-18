//! Is a `<` in a type, or in an expression?
//!
//! This is the recursive core of `type_context`.
//!
//! [`ctx_for_lt`] answers with [`Ctx::Type`] or [`Ctx::Expr`]. It steps back from the `<` over its head,
//! e.g. `Foo` in `Foo<T>`, and asks what the token before the head means. For example:
//!
//! - After `:`, the `<` may be in a type annotation (`let x: Foo<T>`) or a ternary (`c ? a : b<d`).
//! - After `=`, it may be in a type alias (`type A = Foo<T>`) or an assignment (`x = a<b`).
//! - After `(` or `,`, it depends on the enclosing construct,
//!   e.g. a call's arguments (`f(a, b<c)`) or a type argument list (`Map<K, V<T>>`).
//!
//! Answering often means asking the same question about an earlier token or an enclosing construct,
//! so the functions here call each other in a cycle, bounded by [`CTX_HOPS`].
//!
//! The file also holds a few small helpers which don't recurse. One of them, [`member_start_before`],
//! checks whether a position is the start of a class, interface or object member. [`tsx`] uses it too.
//!
//! [`tsx`]: super::tsx

use crate::{
    opmap::OP_KIND_BASE,
    tables::{Tables, is_ws},
    token::tk,
};

use crate::pipeline::bitmap::{bm_get, bm_next1};

use crate::pipeline::disambiguate::common::{
    AngleMatch, LT_OPERAND_WORDS, angle_match_back, annotation_colon_is_declaration,
    bang_is_postfix, bm_prev_sig, brace_opens_value, chain_head, class_like_walk,
    conditional_type_question, declarator_without_init, extends_precedes_question, ident_is,
    incdec_is_postfix, is_binder_keyword, kind_at, lt_in_range, match_delim_back, prop_name,
    type_alias_head, word_is_any,
};

use super::{
    bytes::{ENCLOSING_SCAN_CAP, Encl, GT_SCAN_CAP, angle_close_fwd_capped, enclosing_opener},
    type_list::{Follow, gt_follower, type_list_legal},
};

pub(super) const CTX_HOPS: u32 = 32;

pub(super) const MEMBER_OPEN_WORDS: &[&[u8]] = &[
    b"class",
    b"interface",
    b"enum",
    b"return",
    b"case",
    b"in",
    b"of",
    b"typeof",
    b"await",
    b"yield",
    b"delete",
    b"void",
    b"instanceof",
    b"new",
    b"export",
    b"default",
    b"throw",
    b"as",
    b"satisfies",
    b"extends",
    b"implements",
    b"keyof",
];

pub(super) const BLOCK_OPEN_WORDS: &[&[u8]] =
    &[b"else", b"do", b"try", b"finally", b"declare", b"global"];

const EXPR_WORDS: &[&[u8]] = &[
    b"new",
    b"return",
    b"throw",
    b"yield",
    b"await",
    b"typeof",
    b"delete",
    b"void",
    b"in",
    b"instanceof",
    b"case",
    b"else",
    b"do",
    b"of",
    b"if",
    b"while",
    b"for",
    b"switch",
    b"catch",
    b"with",
    b"default",
    b"let",
    b"const",
    b"var",
];

const TYPE_WORDS: &[&[u8]] = &[
    b"class",
    b"interface",
    b"type",
    b"function",
    b"extends",
    b"implements",
    b"as",
    b"satisfies",
    b"is",
    b"asserts",
    b"keyof",
    b"infer",
    b"readonly",
    b"unique",
    b"abstract",
    b"declare",
    b"enum",
    b"namespace",
    b"module",
    b"get",
    b"set",
    b"static",
    b"public",
    b"private",
    b"protected",
    b"override",
    b"accessor",
];

const GENERATOR_STAR_WORDS: &[&[u8]] =
    &[b"static", b"async", b"public", b"private", b"protected", b"override", b"abstract"];

const JSX_BLOCK_WORDS: &[&[u8]] = &[b"else", b"do", b"try", b"finally", b"static", b"global"];

const TYPE_LITERAL_WORDS: &[&[u8]] = &[
    b"as",
    b"satisfies",
    b"extends",
    b"implements",
    b"keyof",
    b"readonly",
    b"is",
    b"asserts",
    b"in",
];

const SIGNATURE_HEAD_WORDS: &[&[u8]] = &[
    b"function",
    b"async",
    b"get",
    b"set",
    b"static",
    b"public",
    b"private",
    b"protected",
    b"readonly",
    b"abstract",
    b"override",
    b"accessor",
    b"declare",
    b"constructor",
];

const MEMBER_MODIFIER_WORDS: &[&[u8]] = &[
    b"static",
    b"async",
    b"get",
    b"set",
    b"public",
    b"private",
    b"protected",
    b"override",
    b"abstract",
    b"readonly",
    b"declare",
    b"accessor",
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Ctx {
    Type,
    Expr,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum After {
    Head,
    Paren,
    Bracket,
}

pub(super) unsafe fn ctx_for_lt(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
    hops: u32,
) -> Ctx {
    if hops > CTX_HOPS {
        return Ctx::Type;
    }
    let q = bm_prev_sig(st, kind, lt);
    if q < 0 {
        return Ctx::Type;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if matches!(k, tk!(Ident) | tk!(IdentEscaped) | tk!(PrivateIdent) | tk!(PrivateIdentEscaped)) {
        if !prop_name(src, w) && ident_is(src, w, b"this") {
            let p = bm_prev_sig(st, kind, w);
            if p >= 0 {
                let pw = p as usize;
                let pk = kind_at(kind, pw);
                if pk == tk!(Ident)
                    && !prop_name(src, pw)
                    && word_is_any(src, pw, &[b"extends", b"implements"])
                {
                    return Ctx::Type;
                }
                if pk >= OP_KIND_BASE && *src.add(pw) == b',' && class_like_walk(src, st, kind, pw)
                {
                    return Ctx::Type;
                }
            }
            return Ctx::Expr;
        }
        if !prop_name(src, w) && word_is_any(src, w, LT_OPERAND_WORDS) {
            return Ctx::Type;
        }
        let Some(head) = chain_head(src, st, kind, w) else {
            return Ctx::Expr;
        };
        let hp = bm_prev_sig(st, kind, head);
        if hp >= 0
            && kind_at(kind, hp as usize) >= OP_KIND_BASE
            && *src.add(hp as usize) == b')'
            && member_start_before(src, st, kind, head, false)
        {
            return enclosing_context(t, src, st, opch, kind, n, head, false, hops + 1);
        }
        return ctx_after_token(t, src, st, opch, kind, n, hp, After::Head, hops + 1);
    }
    if k >= OP_KIND_BASE {
        let c = *src.add(w);
        if c == b']' {
            if let Some(o) = match_delim_back(src, st, kind, w, b'[', b']') {
                if computed_key_at_member_start(src, st, kind, o) {
                    return enclosing_context(t, src, st, opch, kind, n, w, false, hops + 1);
                }
                if lt_in_range(src, w + 1, lt) {
                    return ctx_after_token(
                        t,
                        src,
                        st,
                        opch,
                        kind,
                        n,
                        bm_prev_sig(st, kind, o),
                        After::Bracket,
                        hops + 1,
                    );
                }
            }
            return Ctx::Expr;
        }
        if c == b'}' {
            return match match_delim_back(src, st, kind, w, b'{', b'}') {
                Some(o) if brace_opens_value(t, src, st, kind, n, o, true, 0) => Ctx::Expr,
                _ => Ctx::Type,
            };
        }
        if c == b'.' {
            let spread = (*src.add(w + 1) == b'.' && *src.add(w + 2) == b'.')
                || (w >= 2 && *src.add(w - 1) == b'.' && *src.add(w - 2) == b'.');
            return if spread { Ctx::Type } else { Ctx::Expr };
        }
        if c == b')' || (c == b'?' && *src.add(w + 1) == b'.') {
            return Ctx::Expr;
        }
        if (c == b'+' || c == b'-') && (*src.add(w + 1) == c || (w > 0 && *src.add(w - 1) == c)) {
            let first = if *src.add(w + 1) == c { w } else { w - 1 };
            return if incdec_is_postfix(src, st, kind, n, first) { Ctx::Expr } else { Ctx::Type };
        }
        if c == b'!' && *src.add(w + 1) != b'=' {
            return if bang_is_postfix(src, st, kind, n, w) { Ctx::Expr } else { Ctx::Type };
        }
        return Ctx::Type;
    }
    if matches!(k, tk!(String) | tk!(Number) | tk!(BigInt))
        && member_start_before(src, st, kind, w, true)
    {
        return enclosing_context(t, src, st, opch, kind, n, w, false, hops + 1);
    }
    Ctx::Expr
}

unsafe fn enclosing_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    from: usize,
    stop_semi: bool,
    hops: u32,
) -> Ctx {
    if stop_semi && class_like_walk(src, st, kind, from) {
        return Ctx::Type;
    }
    let cap = if stop_semi { GT_SCAN_CAP } else { ENCLOSING_SCAN_CAP };
    match enclosing_opener(src, st, opch, kind, n, from, stop_semi, cap) {
        Encl::Open(o, b'<') => enclosing_list_context(t, src, st, opch, kind, n, o, hops + 1),
        Encl::Open(o, b'(') => ctx_after_token(
            t,
            src,
            st,
            opch,
            kind,
            n,
            bm_prev_sig(st, kind, o),
            After::Paren,
            hops + 1,
        ),
        Encl::Open(o, b'[') => ctx_after_token(
            t,
            src,
            st,
            opch,
            kind,
            n,
            bm_prev_sig(st, kind, o),
            After::Bracket,
            hops + 1,
        ),
        Encl::Open(o, _) => {
            if brace_is_member_container(src, st, kind, o) {
                Ctx::Type
            } else {
                Ctx::Expr
            }
        }
        Encl::Top => Ctx::Expr,
        Encl::Capped => Ctx::Type,
    }
}

unsafe fn enclosing_list_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
    hops: u32,
) -> Ctx {
    if hops > CTX_HOPS {
        return Ctx::Type;
    }
    if ctx_for_lt(t, src, st, opch, kind, n, lt, hops + 1) == Ctx::Type {
        return Ctx::Type;
    }
    let lim = (lt + 2 * GT_SCAN_CAP).min(n);
    let (close, capped) = angle_close_fwd_capped(src, st, opch, kind, lt + 1, lim, 1);
    match close {
        Some(gt) => match gt_follower(src, n, gt + 1) {
            Follow::Split => {
                if type_list_legal(t, src, st, kind, lt + 1, gt) {
                    Ctx::Type
                } else {
                    Ctx::Expr
                }
            }
            Follow::Fuse => {
                if lt_head_is_operand(t, src, st, opch, kind, n, lt, hops + 1) {
                    Ctx::Type
                } else {
                    Ctx::Expr
                }
            }
            Follow::Ctx => Ctx::Expr,
        },
        None => {
            if capped {
                Ctx::Type
            } else {
                Ctx::Expr
            }
        }
    }
}

pub(super) unsafe fn lt_head_is_operand(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
    hops: u32,
) -> bool {
    let q = bm_prev_sig(st, kind, lt);
    if q < 0 {
        return true;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    let broke = lt_in_range(src, bm_next1(st, w + 1, n), lt);
    let asi_head = |t: &Tables| {
        broke && operand_head_context(t, src, st, opch, kind, n, lt, true, hops + 1) == Ctx::Type
    };
    if k == tk!(Ident) || k == tk!(IdentEscaped) {
        if !prop_name(src, w) && word_is_any(src, w, LT_OPERAND_WORDS) {
            return true;
        }
        if broke {
            return declarator_without_init(src, st, kind, w) || asi_head(t);
        }
        return ctx_for_lt(t, src, st, opch, kind, n, lt, hops + 1) == Ctx::Type;
    }
    if matches!(k, tk!(Number) | tk!(BigInt) | tk!(String) | tk!(TemplateNoSub) | tk!(TemplateTail))
    {
        return asi_head(t);
    }
    if k >= OP_KIND_BASE {
        let c = *src.add(w);
        if c == b')' {
            if let Some(lp) = match_delim_back(src, st, kind, w, b'(', b')')
                && paren_is_statement_head(src, st, kind, lp)
            {
                return true;
            }
            return asi_head(t);
        }
        if c == b']' {
            return asi_head(t);
        }
        if c == b'>' && !(w > 0 && *src.add(w - 1) == b'=') && broke {
            if let AngleMatch::Found(lt2) = angle_match_back(src, st, kind, w) {
                let p = bm_prev_sig(st, kind, lt2);
                if p >= 0 && matches!(kind_at(kind, p as usize), tk!(Ident) | tk!(IdentEscaped)) {
                    return asi_head(t);
                }
            }
            return true;
        }
        if c == b'.' {
            return *src.add(w + 1) == b'.' && *src.add(w + 2) == b'.';
        }
        if c == b'}' {
            return match match_delim_back(src, st, kind, w, b'{', b'}') {
                Some(o) => !brace_opens_value(t, src, st, kind, n, o, true, 0),
                None => true,
            };
        }
        if (c == b'+' || c == b'-') && (*src.add(w + 1) == c || (w > 0 && *src.add(w - 1) == c)) {
            let first = if *src.add(w + 1) == c { w } else { w - 1 };
            return !incdec_is_postfix(src, st, kind, n, first);
        }
        if c == b'!' && *src.add(w + 1) != b'=' {
            return !bang_is_postfix(src, st, kind, n, w);
        }
        return !(c == b'?' && *src.add(w + 1) == b'.');
    }
    false
}

unsafe fn operand_head_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    op: usize,
    after_break: bool,
    hops: u32,
) -> Ctx {
    let mut q = bm_prev_sig(st, kind, op);
    let mut start: i64 = -1;
    while q >= 0 {
        let v = q as usize;
        let vk = kind_at(kind, v);
        if vk >= OP_KIND_BASE {
            let c = *src.add(v);
            match c {
                b')' | b']' | b'}' => {
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    match match_delim_back(src, st, kind, v, open, c) {
                        Some(o) => {
                            if after_break && c == b')' && paren_is_statement_head(src, st, kind, o)
                            {
                                return Ctx::Type;
                            }
                            start = o as i64;
                            q = bm_prev_sig(st, kind, o);
                            continue;
                        }
                        None => return Ctx::Type,
                    }
                }
                b'>' => {
                    if v > 0 && *src.add(v - 1) == b'=' {
                        break;
                    }
                    match angle_match_back(src, st, kind, v) {
                        AngleMatch::Found(lt) => {
                            start = lt as i64;
                            q = bm_prev_sig(st, kind, lt);
                            continue;
                        }
                        _ => return Ctx::Expr,
                    }
                }
                b'.' => {
                    start = v as i64;
                    q = bm_prev_sig(st, kind, v);
                    continue;
                }
                b'-' if start >= 0
                    && matches!(kind_at(kind, start as usize), tk!(Number) | tk!(BigInt))
                    && !matches!(*src.add(v + 1), b'-' | b'=') =>
                {
                    start = v as i64;
                    q = bm_prev_sig(st, kind, v);
                    continue;
                }
                _ => break,
            }
        } else if matches!(
            vk,
            tk!(Ident)
                | tk!(IdentEscaped)
                | tk!(Number)
                | tk!(BigInt)
                | tk!(String)
                | tk!(TemplateNoSub)
        ) {
            if (vk == tk!(Ident) || vk == tk!(IdentEscaped))
                && !prop_name(src, v)
                && word_is_any(src, v, &[b"as", b"satisfies"])
            {
                return if after_break { Ctx::Expr } else { Ctx::Type };
            }
            start = v as i64;
            q = bm_prev_sig(st, kind, v);
            continue;
        } else if vk == tk!(TemplateTail) {
            let mut depth = 1u32;
            let mut h = bm_prev_sig(st, kind, v);
            while h >= 0 {
                let hk = kind_at(kind, h as usize);
                if hk == tk!(TemplateTail) {
                    depth += 1;
                } else if hk == tk!(TemplateHead) {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                h = bm_prev_sig(st, kind, h as usize);
            }
            if h < 0 {
                return Ctx::Expr;
            }
            start = h;
            q = bm_prev_sig(st, kind, h as usize);
            continue;
        } else {
            break;
        }
    }
    if start < 0 {
        return ctx_after_token(t, src, st, opch, kind, n, q, After::Head, hops + 1);
    }
    let p = bm_prev_sig(st, kind, start as usize);
    ctx_after_token(t, src, st, opch, kind, n, p, After::Head, hops + 1)
}

pub(super) unsafe fn ctx_after_token(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    p: i64,
    mode: After,
    hops: u32,
) -> Ctx {
    if hops > CTX_HOPS {
        return Ctx::Type;
    }
    if p < 0 {
        return Ctx::Expr;
    }
    let w = p as usize;
    let k = kind_at(kind, w);
    if k == tk!(Ident) || k == tk!(IdentEscaped) {
        if prop_name(src, w) {
            let Some(h) = chain_head(src, st, kind, w) else {
                return Ctx::Expr;
            };
            return ctx_after_token(
                t,
                src,
                st,
                opch,
                kind,
                n,
                bm_prev_sig(st, kind, h),
                mode,
                hops + 1,
            );
        }
        if word_is_any(src, w, EXPR_WORDS) {
            return Ctx::Expr;
        }
        if ident_is(src, w, b"async") {
            return if mode == After::Paren { Ctx::Expr } else { Ctx::Type };
        }
        if word_is_any(src, w, TYPE_WORDS) {
            return Ctx::Type;
        }
        return match mode {
            After::Head => Ctx::Type,
            After::Paren => Ctx::Expr,
            After::Bracket => {
                let Some(h) = chain_head(src, st, kind, w) else {
                    return Ctx::Expr;
                };
                ctx_after_token(
                    t,
                    src,
                    st,
                    opch,
                    kind,
                    n,
                    bm_prev_sig(st, kind, h),
                    After::Head,
                    hops + 1,
                )
            }
        };
    }
    if k >= OP_KIND_BASE {
        let c = *src.add(w);
        return match c {
            b')' => Ctx::Expr,
            b'.' => {
                if *src.add(w + 1) != b'.' || *src.add(w + 2) != b'.' {
                    return Ctx::Expr;
                }
                match enclosing_opener(src, st, opch, kind, n, w, true, GT_SCAN_CAP) {
                    Encl::Open(o, b'[') => ctx_after_token(
                        t,
                        src,
                        st,
                        opch,
                        kind,
                        n,
                        bm_prev_sig(st, kind, o),
                        After::Bracket,
                        hops + 1,
                    ),
                    Encl::Open(o, b'(') => ctx_after_token(
                        t,
                        src,
                        st,
                        opch,
                        kind,
                        n,
                        bm_prev_sig(st, kind, o),
                        After::Paren,
                        hops + 1,
                    ),
                    _ => Ctx::Expr,
                }
            }
            b'*' => {
                if mode != After::Head || *src.add(w + 1) == b'*' {
                    return Ctx::Expr;
                }
                let pv = bm_prev_sig(st, kind, w);
                if pv < 0 {
                    return Ctx::Expr;
                }
                let pw = pv as usize;
                let pk = kind_at(kind, pw);
                if pk >= OP_KIND_BASE {
                    return if matches!(*src.add(pw), b'{' | b';' | b'}') {
                        enclosing_context(t, src, st, opch, kind, n, w, false, hops)
                    } else {
                        Ctx::Expr
                    };
                }
                if (pk == tk!(Ident) || pk == tk!(IdentEscaped)) && !prop_name(src, pw) {
                    if ident_is(src, pw, b"function") {
                        return Ctx::Type;
                    }
                    if word_is_any(src, pw, GENERATOR_STAR_WORDS) {
                        return enclosing_context(t, src, st, opch, kind, n, w, false, hops);
                    }
                }
                Ctx::Expr
            }
            b']' => {
                if mode != After::Bracket {
                    return Ctx::Expr;
                }
                match match_delim_back(src, st, kind, w, b'[', b']') {
                    Some(o) => ctx_after_token(
                        t,
                        src,
                        st,
                        opch,
                        kind,
                        n,
                        bm_prev_sig(st, kind, o),
                        After::Bracket,
                        hops + 1,
                    ),
                    None => Ctx::Expr,
                }
            }
            b':' => colon_context(t, src, st, opch, kind, n, w, hops),
            b'=' => eq_context(t, src, st, opch, kind, n, w, hops),
            b'(' => ctx_after_token(
                t,
                src,
                st,
                opch,
                kind,
                n,
                bm_prev_sig(st, kind, w),
                After::Paren,
                hops + 1,
            ),
            b'[' => ctx_after_token(
                t,
                src,
                st,
                opch,
                kind,
                n,
                bm_prev_sig(st, kind, w),
                After::Bracket,
                hops + 1,
            ),
            b'{' => {
                if mode == After::Head && brace_is_member_container(src, st, kind, w) {
                    Ctx::Type
                } else {
                    Ctx::Expr
                }
            }
            b';' | b'}' => {
                if mode == After::Head {
                    enclosing_context(t, src, st, opch, kind, n, w, false, hops)
                } else {
                    Ctx::Expr
                }
            }
            b',' => enclosing_context(t, src, st, opch, kind, n, w, true, hops),
            b'<' => {
                let nx = *src.add(w + 1);
                if nx == b'=' || (nx == b'<' && !bm_get(st, w + 1)) {
                    Ctx::Expr
                } else {
                    enclosing_list_context(t, src, st, opch, kind, n, w, hops)
                }
            }
            b'>' => {
                let nx = *src.add(w + 1);
                if ((nx == b'>' || nx == b'=') && !bm_get(st, w + 1)) || mode == After::Paren {
                    Ctx::Expr
                } else {
                    Ctx::Type
                }
            }
            b'|' | b'&' => {
                if *src.add(w + 1) == c {
                    Ctx::Expr
                } else {
                    operand_head_context(t, src, st, opch, kind, n, w, false, hops)
                }
            }
            b'?' => {
                let nx = *src.add(w + 1);
                if nx == b'?' || nx == b'.' {
                    Ctx::Expr
                } else if extends_precedes_question(src, st, kind, w) {
                    Ctx::Type
                } else {
                    Ctx::Expr
                }
            }
            _ => Ctx::Expr,
        };
    }
    Ctx::Expr
}

unsafe fn colon_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
    hops: u32,
) -> Ctx {
    let q = bm_prev_sig(st, kind, colon);
    if q < 0 {
        return Ctx::Expr;
    }
    let v = q as usize;
    let vk = kind_at(kind, v);
    if (vk == tk!(Ident) || vk == tk!(IdentEscaped))
        && !prop_name(src, v)
        && ident_is(src, v, b"default")
    {
        return Ctx::Expr;
    }
    let q2 = bm_prev_sig(st, kind, v);
    if q2 >= 0 {
        let u = q2 as usize;
        if kind_at(kind, u) == tk!(Ident) && !prop_name(src, u) && ident_is(src, u, b"case") {
            return Ctx::Expr;
        }
        if (vk == tk!(Ident) || vk == tk!(IdentEscaped))
            && !prop_name(src, v)
            && is_binder_keyword(src, kind, u)
        {
            return Ctx::Type;
        }
    }
    if vk >= OP_KIND_BASE {
        let c = *src.add(v);
        if c == b'?' && !matches!(*src.add(v + 1), b'?' | b'.') {
            return Ctx::Type;
        }
        if c == b')'
            && let Some(ctx) = signature_colon(src, st, kind, v)
        {
            return ctx;
        }
        if c == b']' || c == b'}' {
            let open = if c == b']' { b'[' } else { b'{' };
            if let Some(op) = match_delim_back(src, st, kind, v, open, c) {
                let bp = bm_prev_sig(st, kind, op);
                if bp >= 0 && is_binder_keyword(src, kind, bp as usize) {
                    return Ctx::Type;
                }
            }
        }
    }
    if conditional_type_question(src, st, kind, colon).is_some() {
        return Ctx::Type;
    }
    if ternary_colon(t, src, st, opch, kind, n, colon) {
        return Ctx::Expr;
    }
    if annotation_colon_is_declaration(t, src, st, kind, n, colon, 0) {
        return Ctx::Type;
    }
    if vk >= OP_KIND_BASE && *src.add(v) == b')' {
        return Ctx::Type;
    }
    match enclosing_opener(src, st, opch, kind, n, colon, false, ENCLOSING_SCAN_CAP) {
        Encl::Open(o, b'{') => {
            let bp = bm_prev_sig(st, kind, o);
            if bp >= 0 {
                let bw = bp as usize;
                if kind_at(kind, bw) >= OP_KIND_BASE
                    && matches!(*src.add(bw), b')' | b'>')
                    && class_like_walk(src, st, kind, bw)
                {
                    return Ctx::Type;
                }
            }
            ctx_after_token(t, src, st, opch, kind, n, bp, After::Head, hops + 1)
        }
        Encl::Open(..) => Ctx::Type,
        Encl::Top => Ctx::Expr,
        Encl::Capped => Ctx::Type,
    }
}

pub(super) unsafe fn ternary_colon(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
) -> bool {
    let mut debt: u32 = 0;
    let mut q = bm_prev_sig(st, kind, colon);
    while q >= 0 {
        let w = q as usize;
        if kind_at(kind, w) >= OP_KIND_BASE {
            match *src.add(w) {
                c @ (b')' | b']' | b'}') => {
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    match match_delim_back(src, st, kind, w, open, c) {
                        Some(op) => {
                            q = bm_prev_sig(st, kind, op);
                            continue;
                        }
                        None => return false,
                    }
                }
                b'{' | b'(' | b'[' | b';' => return false,
                b'>' if !(w > 0 && *src.add(w - 1) == b'=') => {
                    if let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) {
                        q = bm_prev_sig(st, kind, lt);
                        continue;
                    }
                }
                b':' => debt += 1,
                b'?' => {
                    if *src.add(w + 1) != b'?'
                        && *src.add(w + 1) != b'.'
                        && (w == 0 || *src.add(w - 1) != b'?')
                    {
                        let mut j = w + 1;
                        while is_ws(*src.add(j)) {
                            j += 1;
                        }
                        let optional = matches!(*src.add(j), b',' | b']' | b')' | b':')
                            || (*src.add(j) == b'('
                                && optional_method_marker(t, src, st, opch, kind, n, w));
                        if !optional {
                            if debt == 0 {
                                return !extends_precedes_question(src, st, kind, w);
                            }
                            debt -= 1;
                        }
                    }
                }
                _ => {}
            }
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}

unsafe fn optional_method_marker(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    question: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, question);
    if q < 0 {
        return false;
    }
    let mut name = q as usize;
    let nk = kind_at(kind, name);
    if nk >= OP_KIND_BASE {
        if *src.add(name) != b']' {
            return false;
        }
        let Some(lb) = match_delim_back(src, st, kind, name, b'[', b']') else {
            return false;
        };
        name = lb;
    } else if !matches!(
        nk,
        tk!(Ident) | tk!(String) | tk!(Number) | tk!(BigInt) | tk!(PrivateIdent)
    ) {
        return false;
    }
    if !member_start_before(src, st, kind, name, true) {
        return false;
    }
    match enclosing_opener(src, st, opch, kind, n, q as usize, false, ENCLOSING_SCAN_CAP) {
        Encl::Open(o, b'{') => brace_is_type_literal(t, src, st, opch, kind, n, o),
        Encl::Capped => true,
        _ => false,
    }
}

pub(super) unsafe fn brace_is_type_literal(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    brace: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if k >= OP_KIND_BASE {
        return match *src.add(w) {
            b'=' => {
                if *src.add(w + 1) == b'>' {
                    arrow_context(t, src, st, opch, kind, n, w, 0) == Ctx::Type
                } else {
                    type_alias_head(src, st, kind, w)
                        || matches!(
                            enclosing_opener(src, st, opch, kind, n, w, true, GT_SCAN_CAP),
                            Encl::Open(_, b'<')
                        )
                }
            }
            b'>' => {
                if w > 0 && *src.add(w - 1) == b'=' {
                    arrow_context(t, src, st, opch, kind, n, w - 1, 0) == Ctx::Type
                } else {
                    class_like_walk(src, st, kind, w)
                }
            }
            b':' => colon_context(t, src, st, opch, kind, n, w, 0) == Ctx::Type,
            b')' => class_like_walk(src, st, kind, w),
            b'{' | b';' | b'}' | b'.' => false,
            _ => true,
        };
    }
    if k == tk!(Ident) {
        if prop_name(src, w) || word_is_any(src, w, JSX_BLOCK_WORDS) {
            return false;
        }
        if word_is_any(src, w, TYPE_LITERAL_WORDS) {
            return true;
        }
        return class_like_walk(src, st, kind, w);
    }
    false
}

pub(super) unsafe fn arrow_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    eq: usize,
    hops: u32,
) -> Ctx {
    let q = bm_prev_sig(st, kind, eq);
    if q < 0 {
        return Ctx::Expr;
    }
    let v = q as usize;
    if kind_at(kind, v) < OP_KIND_BASE || *src.add(v) != b')' {
        return Ctx::Expr;
    }
    let Some(lp) = match_delim_back(src, st, kind, v, b'(', b')') else {
        return Ctx::Expr;
    };
    let mut b = bm_prev_sig(st, kind, lp);
    if b >= 0 {
        let bw = b as usize;
        if kind_at(kind, bw) >= OP_KIND_BASE
            && *src.add(bw) == b'>'
            && !(bw > 0 && *src.add(bw - 1) == b'=')
        {
            match angle_match_back(src, st, kind, bw) {
                AngleMatch::Found(lt) => b = bm_prev_sig(st, kind, lt),
                _ => return Ctx::Expr,
            }
        }
    }
    if b >= 0 {
        let bw = b as usize;
        let bk = kind_at(kind, bw);
        if (bk == tk!(Ident) || bk == tk!(IdentEscaped)) && !prop_name(src, bw) {
            if ident_is(src, bw, b"async") {
                return Ctx::Expr;
            }
            if ident_is(src, bw, b"new") {
                b = bm_prev_sig(st, kind, bw);
                if b >= 0
                    && kind_at(kind, b as usize) == tk!(Ident)
                    && ident_is(src, b as usize, b"abstract")
                {
                    b = bm_prev_sig(st, kind, b as usize);
                }
            }
        }
    }
    ctx_after_token(t, src, st, opch, kind, n, b, After::Head, hops + 1)
}

unsafe fn eq_context(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    eq: usize,
    hops: u32,
) -> Ctx {
    let nx = *src.add(eq + 1);
    if nx == b'>' {
        return arrow_context(t, src, st, opch, kind, n, eq, hops);
    }
    if nx == b'=' || (eq > 0 && matches!(*src.add(eq - 1), b'=' | b'!' | b'<' | b'>')) {
        return Ctx::Expr;
    }
    if type_alias_head(src, st, kind, eq) {
        return Ctx::Type;
    }
    match enclosing_opener(src, st, opch, kind, n, eq, true, GT_SCAN_CAP) {
        Encl::Open(_, b'<') => Ctx::Type,
        _ => Ctx::Expr,
    }
}

unsafe fn computed_key_at_member_start(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    lb: usize,
) -> bool {
    member_start_before(src, st, kind, lb, true)
}

unsafe fn brace_is_member_container(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    brace: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if k == tk!(Ident) || k == tk!(IdentEscaped) {
        if prop_name(src, w) {
            return chain_head(src, st, kind, w).is_some_and(|h| class_like_walk(src, st, kind, h));
        }
        if word_is_any(src, w, MEMBER_OPEN_WORDS) {
            return true;
        }
        if word_is_any(src, w, BLOCK_OPEN_WORDS) {
            return false;
        }
        return class_like_walk(src, st, kind, w);
    }
    if k >= OP_KIND_BASE {
        let c = *src.add(w);
        return match c {
            b')' | b'>' => class_like_walk(src, st, kind, w),
            b'=' => *src.add(w + 1) != b'>',
            b'{' | b';' | b'}' => false,
            _ => true,
        };
    }
    false
}

unsafe fn paren_is_statement_head(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    lp: usize,
) -> bool {
    let h = bm_prev_sig(st, kind, lp);
    h >= 0
        && kind_at(kind, h as usize) == tk!(Ident)
        && !prop_name(src, h as usize)
        && word_is_any(src, h as usize, &[b"if", b"while", b"for", b"with"])
}

unsafe fn signature_colon(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    rparen: usize,
) -> Option<Ctx> {
    let lp = match_delim_back(src, st, kind, rparen, b'(', b')')?;
    let mut q = bm_prev_sig(st, kind, lp);
    if q < 0 {
        return None;
    }
    let mut w = q as usize;
    if kind_at(kind, w) >= OP_KIND_BASE {
        let c = *src.add(w);
        if c == b'>' && !(w > 0 && *src.add(w - 1) == b'=') {
            let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) else {
                return None;
            };
            q = bm_prev_sig(st, kind, lt);
            if q < 0 {
                return None;
            }
            w = q as usize;
        } else if c == b'*' {
            q = bm_prev_sig(st, kind, w);
            if q < 0 {
                return None;
            }
            w = q as usize;
        } else {
            if c != b'?' || *src.add(w + 1) == b'?' || *src.add(w + 1) == b'.' {
                return None;
            }
            let nm = bm_prev_sig(st, kind, w);
            if nm < 0 {
                return Some(Ctx::Expr);
            }
            let mut name = nm as usize;
            let nk = kind_at(kind, name);
            if nk >= OP_KIND_BASE {
                if *src.add(name) != b']' {
                    return Some(Ctx::Expr);
                }
                let Some(lb) = match_delim_back(src, st, kind, name, b'[', b']') else {
                    return Some(Ctx::Expr);
                };
                name = lb;
            } else if !matches!(
                nk,
                tk!(Ident)
                    | tk!(IdentEscaped)
                    | tk!(String)
                    | tk!(Number)
                    | tk!(BigInt)
                    | tk!(PrivateIdent)
                    | tk!(PrivateIdentEscaped)
            ) {
                return Some(Ctx::Expr);
            }
            return if member_start_before(src, st, kind, name, false) {
                Some(Ctx::Type)
            } else {
                Some(Ctx::Expr)
            };
        }
    }
    let k = kind_at(kind, w);
    if k != tk!(Ident) && k != tk!(IdentEscaped) {
        return None;
    }
    if !prop_name(src, w) && word_is_any(src, w, SIGNATURE_HEAD_WORDS) {
        return Some(Ctx::Type);
    }
    let head = chain_head(src, st, kind, w)?;
    let pq = bm_prev_sig(st, kind, head);
    if pq < 0 {
        return Some(Ctx::Type);
    }
    let pw = pq as usize;
    let pk = kind_at(kind, pw);
    if pk >= OP_KIND_BASE {
        let c = *src.add(pw);
        if c == b'?' && *src.add(pw + 1) != b'?' && *src.add(pw + 1) != b'.' {
            return if extends_precedes_question(src, st, kind, pw) {
                Some(Ctx::Type)
            } else {
                Some(Ctx::Expr)
            };
        }
        if matches!(c, b'{' | b';' | b'}' | b',' | b'*' | b'>' | b']') {
            return Some(Ctx::Type);
        }
        return None;
    }
    if (pk == tk!(Ident) || pk == tk!(IdentEscaped))
        && !prop_name(src, pw)
        && word_is_any(src, pw, SIGNATURE_HEAD_WORDS)
    {
        return Some(Ctx::Type);
    }
    None
}

pub(super) unsafe fn member_start_before(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    pos: usize,
    allow_comma: bool,
) -> bool {
    let p = bm_prev_sig(st, kind, pos);
    if p < 0 {
        return false;
    }
    let pw = p as usize;
    let pk = kind_at(kind, pw);
    if pk >= OP_KIND_BASE {
        let c = *src.add(pw);
        if c == b')' {
            let Some(lp) = match_delim_back(src, st, kind, pw, b'(', b')') else {
                return false;
            };
            let d = bm_prev_sig(st, kind, lp);
            return d >= 0
                && matches!(kind_at(kind, d as usize), tk!(Ident) | tk!(IdentEscaped))
                && decorator_before(src, st, kind, d as usize);
        }
        return matches!(c, b'{' | b';' | b'}' | b'*') || (allow_comma && c == b',');
    }
    if (pk == tk!(Ident) || pk == tk!(IdentEscaped)) && !prop_name(src, pw) {
        return word_is_any(src, pw, MEMBER_MODIFIER_WORDS) || decorator_before(src, st, kind, pw);
    }
    false
}

unsafe fn decorator_before(src: *const u8, st: *const u64, kind: *const u8, w: usize) -> bool {
    let Some(h) = chain_head(src, st, kind, w) else {
        return false;
    };
    let at = bm_prev_sig(st, kind, h);
    at >= 0 && *src.add(at as usize) == b'@'
}
