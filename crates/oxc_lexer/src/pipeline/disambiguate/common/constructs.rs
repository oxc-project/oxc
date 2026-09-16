//! Recognizers for individual JS and TS constructs.
//!
//! Each function answers one question about the construct just before a position,
//! using a short backward walk built from [`walk`]. For example:
//!
//! - [`bang_is_postfix`]: Is this `!` a non-null assertion, as in `x!`?
//! - [`type_alias_head`]: Is this `=` the one in `type X<T> =`?
//! - [`declarator_without_init`]: Is this identifier a variable declared without an initializer, as in `let x`?
//! - [`conditional_type_question`]: Which `?` does this `:` pair with, in `A extends B ? C : D`?
//!
//! None of them asks whether a position is an expression position, so none of them recurses.
//! That is what separates this file from [`operand`], which combines these recognizers to answer that question.
//!
//! [`walk`]: super::walk
//! [`operand`]: super::operand

use crate::{opmap::OP_KIND_BASE, tables::Tables, token::tk};

use super::super::super::bitmap::bm_next1;

use super::walk::{
    AngleMatch, angle_match_back, bm_prev_sig, chain_head, ident_is, kind_at, lt_in_range,
    match_delim_back, prop_name, word_is_any,
};

pub(super) const CLASS_WALK_STOP_WORDS: &[&[u8]] = &[
    b"function",
    b"return",
    b"if",
    b"while",
    b"for",
    b"switch",
    b"catch",
    b"with",
    b"new",
    b"typeof",
    b"await",
    b"yield",
    b"else",
    b"do",
    b"try",
    b"finally",
    b"namespace",
    b"module",
];

const RETURN_TYPE_STOP_WORDS: &[&[u8]] = &[
    b"return",
    b"throw",
    b"yield",
    b"await",
    b"delete",
    b"case",
    b"else",
    b"do",
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
    b"class",
    b"function",
    b"interface",
    b"enum",
    b"try",
    b"finally",
    b"export",
];

const TYPE_LITERAL_HEAD_WORDS: &[&[u8]] =
    &[b"extends", b"keyof", b"is", b"in", b"readonly", b"asserts"];

pub unsafe fn return_type_signature_paren(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    body: usize,
) -> Option<usize> {
    let mut q = bm_prev_sig(st, kind, body);
    while q >= 0 {
        let w = q as usize;
        let k = kind_at(kind, w);
        if k >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b')' | b']' | b'}' => {
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    let op = match_delim_back(src, st, kind, w, open, c)?;
                    if c == b'}' && !type_literal_brace_opener(src, st, kind, op) {
                        return None;
                    }
                    q = bm_prev_sig(st, kind, op);
                    continue;
                }
                b'>' => {
                    if w > 0 && *src.add(w - 1) == b'=' {
                        q = bm_prev_sig(st, kind, w);
                        continue;
                    }
                    let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) else {
                        return None;
                    };
                    q = bm_prev_sig(st, kind, lt);
                    continue;
                }
                b'=' => {
                    if *src.add(w + 1) != b'>' {
                        return None;
                    }
                }
                b':' => {
                    let p = bm_prev_sig(st, kind, w);
                    if p >= 0
                        && kind_at(kind, p as usize) >= OP_KIND_BASE
                        && *src.add(p as usize) == b')'
                    {
                        return match_delim_back(src, st, kind, p as usize, b'(', b')');
                    }
                    let qm = conditional_type_question(src, st, kind, w)?;
                    q = bm_prev_sig(st, kind, qm);
                    continue;
                }
                b'.' | b'|' | b'&' | b'?' | b',' => {}
                _ => return None,
            }
        } else if k == tk!(Ident) {
            if !prop_name(src, w) && word_is_any(src, w, RETURN_TYPE_STOP_WORDS) {
                return None;
            }
        } else if !matches!(
            k,
            tk!(Number)
                | tk!(BigInt)
                | tk!(String)
                | tk!(TemplateNoSub)
                | tk!(TemplateHead)
                | tk!(TemplateMiddle)
                | tk!(TemplateTail)
        ) {
            return None;
        }
        q = bm_prev_sig(st, kind, w);
    }
    None
}

pub(super) unsafe fn type_literal_brace_opener(
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
    if k >= OP_KIND_BASE {
        return match *src.add(w) {
            b':' | b'|' | b'&' | b'=' | b',' | b'<' | b'(' | b'[' | b'?' => true,
            b'>' => w > 0 && *src.add(w - 1) == b'=',
            _ => false,
        };
    }
    k == tk!(Ident) && !prop_name(src, w) && word_is_any(src, w, TYPE_LITERAL_HEAD_WORDS)
}

pub unsafe fn conditional_type_question(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    colon: usize,
) -> Option<usize> {
    let mut debt: u32 = 0;
    let mut q = bm_prev_sig(st, kind, colon);
    while q >= 0 {
        let w = q as usize;
        if kind_at(kind, w) >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b')' | b']' | b'}' => {
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
                        None => return None,
                    }
                }
                b'(' | b'[' | b'{' | b';' => return None,
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
                        if debt == 0 {
                            return if extends_precedes_question(src, st, kind, w) {
                                Some(w)
                            } else {
                                None
                            };
                        }
                        debt -= 1;
                    }
                }
                _ => {}
            }
        }
        q = bm_prev_sig(st, kind, w);
    }
    None
}

pub unsafe fn extends_precedes_question(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    question: usize,
) -> bool {
    let mut q = bm_prev_sig(st, kind, question);
    while q >= 0 {
        let w = q as usize;
        if kind_at(kind, w) >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b')' | b']' | b'}' => {
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
                b'(' | b'[' | b'{' | b';' | b':' | b'?' => return false,
                b'>' if !(w > 0 && *src.add(w - 1) == b'=') => {
                    if let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) {
                        q = bm_prev_sig(st, kind, lt);
                        continue;
                    }
                }
                _ => {}
            }
        } else if kind_at(kind, w) == tk!(Ident)
            && !prop_name(src, w)
            && ident_is(src, w, b"extends")
        {
            return true;
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}

pub unsafe fn of_is_forof_keyword(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    qi: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, qi);
    if q < 0 {
        return false;
    }
    let tp = q as usize;
    let tk = *kind.add(tp);
    if tk == tk!(Ident) || tk == tk!(IdentEscaped) {
        let e = bm_next1(st, tp + 1, n);
        if !prop_name(src, tp) && t.is_regex_keyword(src.add(tp), e - tp) {
            return false;
        }
    } else if tk >= OP_KIND_BASE {
        let c = *src.add(tp);
        if c != b']' && c != b'}' && c != b')' {
            return false;
        }
    } else {
        return false;
    }
    let mut par: i32 = 0;
    let mut brk: i32 = 0;
    let mut brc: i32 = 0;
    let mut w = q;
    while w >= 0 {
        let p = w as usize;
        let kk = *kind.add(p);
        if kk >= OP_KIND_BASE {
            match *src.add(p) {
                b')' => par += 1,
                b']' => brk += 1,
                b'}' => brc += 1,
                b'(' => {
                    if par == 0 && brk == 0 && brc == 0 {
                        let h = bm_prev_sig(st, kind, p);
                        if h < 0 || *kind.add(h as usize) != tk!(Ident) {
                            return false;
                        }
                        let mut hp = h as usize;
                        if ident_is(src, hp, b"await") {
                            let h2 = bm_prev_sig(st, kind, hp);
                            if h2 < 0 || *kind.add(h2 as usize) != tk!(Ident) {
                                return false;
                            }
                            hp = h2 as usize;
                        }
                        return !prop_name(src, hp) && ident_is(src, hp, b"for");
                    }
                    par -= 1;
                }
                b'[' => {
                    if brk == 0 && par == 0 && brc == 0 {
                        return false;
                    }
                    brk -= 1;
                }
                b'{' => {
                    if brc == 0 && par == 0 && brk == 0 {
                        return false;
                    }
                    brc -= 1;
                }
                _ => {}
            }
        } else if kk == tk!(Ident)
            && par == 0
            && brk == 0
            && brc == 0
            && !prop_name(src, p)
            && ident_is(src, p, b"of")
        {
            let b = bm_prev_sig(st, kind, p);
            if b >= 0 {
                let bp = b as usize;
                let bk = *kind.add(bp);
                let tail = if bk == tk!(Ident) || bk == tk!(IdentEscaped) {
                    let be = bm_next1(st, bp + 1, n);
                    prop_name(src, bp) || !t.is_regex_keyword(src.add(bp), be - bp)
                } else {
                    bk >= OP_KIND_BASE && matches!(*src.add(bp), b']' | b'}' | b')')
                };
                if tail {
                    return false;
                }
            }
        }
        w = bm_prev_sig(st, kind, p);
    }
    false
}

pub unsafe fn incdec_is_postfix(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    first: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, first);
    if q < 0 {
        return false;
    }
    let v = q as usize;
    let vk = kind_at(kind, v);
    let value = matches!(
        vk,
        tk!(Ident)
            | tk!(IdentEscaped)
            | tk!(Number)
            | tk!(BigInt)
            | tk!(String)
            | tk!(TemplateNoSub)
            | tk!(TemplateTail)
    ) || (vk >= OP_KIND_BASE && matches!(*src.add(v), b')' | b']'));
    value && !lt_in_range(src, bm_next1(st, v + 1, n), first)
}

pub unsafe fn bang_is_postfix(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    bang: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, bang);
    if q < 0 {
        return false;
    }
    let v = q as usize;
    let vk = kind_at(kind, v);
    let value = matches!(
        vk,
        tk!(Ident)
            | tk!(PrivateIdent)
            | tk!(Number)
            | tk!(BigInt)
            | tk!(String)
            | tk!(TemplateNoSub)
            | tk!(TemplateTail)
    ) || (vk >= OP_KIND_BASE && matches!(*src.add(v), b')' | b']' | b'}'));
    value && !lt_in_range(src, bm_next1(st, v + 1, n), bang)
}

pub unsafe fn class_like_walk(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    from: usize,
) -> bool {
    let mut q = from as i64;
    while q >= 0 {
        let w = q as usize;
        let k = kind_at(kind, w);
        if k == tk!(Ident) || k == tk!(IdentEscaped) {
            if !prop_name(src, w) {
                if word_is_any(src, w, &[b"class", b"interface", b"enum"]) {
                    return true;
                }
                if word_is_any(src, w, CLASS_WALK_STOP_WORDS) {
                    return false;
                }
            }
        } else if k >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b'.' | b',' => {}
                b')' => match match_delim_back(src, st, kind, w, b'(', b')') {
                    Some(o) => {
                        q = bm_prev_sig(st, kind, o);
                        continue;
                    }
                    None => return false,
                },
                b'>' => {
                    if w > 0 && *src.add(w - 1) == b'=' {
                        return false;
                    }
                    match angle_match_back(src, st, kind, w) {
                        AngleMatch::Found(lt) => {
                            q = bm_prev_sig(st, kind, lt);
                            continue;
                        }
                        _ => return false,
                    }
                }
                _ => return false,
            }
        } else if !matches!(k, tk!(String) | tk!(Number) | tk!(TemplateNoSub)) {
            return false;
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}

pub unsafe fn type_alias_head(src: *const u8, st: *const u64, kind: *const u8, eq: usize) -> bool {
    let q = bm_prev_sig(st, kind, eq);
    if q < 0 {
        return false;
    }
    let mut w = q as usize;
    if kind_at(kind, w) >= OP_KIND_BASE && *src.add(w) == b'>' {
        match angle_match_back(src, st, kind, w) {
            AngleMatch::Found(lt) => {
                let p = bm_prev_sig(st, kind, lt);
                if p < 0 {
                    return false;
                }
                w = p as usize;
            }
            _ => return false,
        }
    }
    if kind_at(kind, w) != tk!(Ident) || prop_name(src, w) {
        return false;
    }
    let p = bm_prev_sig(st, kind, w);
    p >= 0
        && kind_at(kind, p as usize) == tk!(Ident)
        && !prop_name(src, p as usize)
        && ident_is(src, p as usize, b"type")
}

#[inline(always)]
pub fn type_prefix_kind(k: u8) -> bool {
    matches!(
        k,
        tk!(KwKeyof)
            | tk!(KwTypeof)
            | tk!(KwReadonly)
            | tk!(KwUnique)
            | tk!(KwInfer)
            | tk!(KwAbstract)
            | tk!(KwNew)
            | tk!(KwAsserts)
            | tk!(KwImport)
            | tk!(KwExtends)
            | tk!(KwIs)
            | tk!(KwIn)
            | tk!(KwAs)
    )
}

pub unsafe fn declarator_without_init(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    id: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, id);
    if q < 0 {
        return false;
    }
    let mut w = q as usize;
    if is_binder_keyword(src, kind, w) {
        return binder_outside_paren_head(src, st, kind, w);
    }
    if !(kind_at(kind, w) >= OP_KIND_BASE && *src.add(w) == b',') {
        return false;
    }
    loop {
        if kind_at(kind, w) >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b')' | b']' | b'}' => {
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    match match_delim_back(src, st, kind, w, open, c) {
                        Some(op) => {
                            let q2 = bm_prev_sig(st, kind, op);
                            if q2 < 0 {
                                return false;
                            }
                            w = q2 as usize;
                            continue;
                        }
                        None => return false,
                    }
                }
                b'(' | b'[' | b'{' | b';' => return false,
                _ => {}
            }
        } else if is_binder_keyword(src, kind, w) {
            return binder_outside_paren_head(src, st, kind, w);
        }
        let q2 = bm_prev_sig(st, kind, w);
        if q2 < 0 {
            return false;
        }
        w = q2 as usize;
    }
}

#[inline]
pub unsafe fn is_binder_keyword(src: *const u8, kind: *const u8, w: usize) -> bool {
    kind_at(kind, w) == tk!(Ident)
        && !prop_name(src, w)
        && (ident_is(src, w, b"let")
            || ident_is(src, w, b"const")
            || ident_is(src, w, b"var")
            || ident_is(src, w, b"using"))
}

#[inline]
pub(super) unsafe fn binder_outside_paren_head(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    binder: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, binder);
    q < 0 || !(kind_at(kind, q as usize) >= OP_KIND_BASE && *src.add(q as usize) == b'(')
}

pub unsafe fn signature_return_type(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    rparen: usize,
) -> bool {
    let Some(lp) = match_delim_back(src, st, kind, rparen, b'(', b')') else {
        return false;
    };
    function_keyword_before_params(src, st, kind, lp).is_some()
}

pub(super) unsafe fn function_keyword_before_params(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    lp: usize,
) -> Option<usize> {
    let mut q = bm_prev_sig(st, kind, lp);
    if q < 0 {
        return None;
    }
    let mut w = q as usize;
    if kind_at(kind, w) >= OP_KIND_BASE
        && *src.add(w) == b'>'
        && !(w > 0 && *src.add(w - 1) == b'=')
    {
        let AngleMatch::Found(lt) = angle_match_back(src, st, kind, w) else {
            return None;
        };
        q = bm_prev_sig(st, kind, lt);
        if q < 0 {
            return None;
        }
        w = q as usize;
    }
    if kind_at(kind, w) < OP_KIND_BASE && !ident_is(src, w, b"function") {
        q = bm_prev_sig(st, kind, w);
        if q < 0 {
            return None;
        }
        w = q as usize;
    }
    if kind_at(kind, w) >= OP_KIND_BASE && *src.add(w) == b'*' {
        q = bm_prev_sig(st, kind, w);
        if q < 0 {
            return None;
        }
        w = q as usize;
    }
    if kind_at(kind, w) < OP_KIND_BASE && !prop_name(src, w) && ident_is(src, w, b"function") {
        Some(w)
    } else {
        None
    }
}

#[inline]
pub unsafe fn as_type_operand(src: *const u8, st: *const u64, kind: *const u8, pos: usize) -> bool {
    let q = bm_prev_sig(st, kind, pos);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    *kind.add(w) == tk!(Ident)
        && !prop_name(src, w)
        && (ident_is(src, w, b"as") || ident_is(src, w, b"satisfies"))
}

pub unsafe fn as_gated_type_ref(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
) -> bool {
    let b = bm_prev_sig(st, kind, lt);
    if b < 0 || *kind.add(b as usize) != tk!(Ident) {
        return false;
    }
    let Some(head) = chain_head(src, st, kind, b as usize) else {
        return false;
    };
    let a = bm_prev_sig(st, kind, head);
    if a < 0 {
        return false;
    }
    let ap = a as usize;
    if *kind.add(ap) != tk!(Ident)
        || prop_name(src, ap)
        || !(ident_is(src, ap, b"as") || ident_is(src, ap, b"satisfies"))
    {
        return false;
    }
    tail_or_brace_before(t, src, st, kind, n, ap)
}

pub(super) unsafe fn tail_or_brace_before(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    pos: usize,
) -> bool {
    if tail_before(t, src, st, kind, n, pos) {
        return true;
    }
    let s = bm_prev_sig(st, kind, pos);
    s >= 0 && kind_at(kind, s as usize) >= OP_KIND_BASE && *src.add(s as usize) == b'}'
}

pub unsafe fn tail_before(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    pos: usize,
) -> bool {
    let mut s = bm_prev_sig(st, kind, pos);
    loop {
        if s < 0 {
            return false;
        }
        let w = s as usize;
        if kind_at(kind, w) >= OP_KIND_BASE && *src.add(w) == b'!' && *src.add(w + 1) != b'=' {
            s = bm_prev_sig(st, kind, w);
            continue;
        }
        break;
    }
    if s < 0 {
        return false;
    }
    let sp = s as usize;
    let sk = kind_at(kind, sp);
    if sk >= OP_KIND_BASE {
        return matches!(*src.add(sp), b')' | b']');
    }
    if sk == tk!(Ident) {
        let e = bm_next1(st, sp + 1, n);
        return prop_name(src, sp) || !t.is_regex_keyword(src.add(sp), e - sp);
    }
    matches!(
        sk,
        tk!(Number)
            | tk!(BigInt)
            | tk!(String)
            | tk!(TemplateNoSub)
            | tk!(TemplateTail)
            | tk!(RegExp)
            | tk!(PrivateIdent)
    )
}
