//! Is this an expression position?
//!
//! This is the recursive core of `common`. It answers two questions:
//!
//! - [`operand_position`]: Can only an expression start here?
//!   After `=` or `(` it can, so a `{` there opens an object literal.
//!   At the start of a statement it can't, so a `{` there opens a block.
//! - [`brace_opens_value`]: Does this `{` open a value or a block?
//!   A value is an object literal, or the body of a function or class expression.
//!   For example, this decides whether a `/` after the matching `}` is division or starts a regex.
//!
//! Answering these needs recursion. Whether `class C {}` is a value depends on whether `class` itself
//! is in expression position, which depends on what comes before it, and so on.
//! So the functions here call each other in a cycle, bounded by a depth limit.
//! Whether a line break after a TypeScript type annotation ends the statement ([`type_annotation_asi`])
//! is decided in the same cycle.
//!
//! [`LT_OPERAND_WORDS`] also lives here. It lists the words after which a `<` starts an operand,
//! for [`operator`] and [`type_context`].
//!
//! [`operator`]: super::super::operator
//! [`type_context`]: super::super::type_context

use crate::{opmap::OP_KIND_BASE, tables::Tables};

use super::super::super::{
    BIGINT, IDENT, IDENT_ESC, JEND, JTEXT, NUM, REGEX, STR, TMPL_HEAD, TMPL_MIDDLE, TMPL_NOSUB,
    TMPL_TAIL, bitmap::bm_next1,
};

use super::{
    KW_AS, KW_EXTENDS, KW_IN, KW_IS,
    constructs::{
        CLASS_WALK_STOP_WORDS, bang_is_postfix, binder_outside_paren_head, class_like_walk,
        conditional_type_question, declarator_without_init, function_keyword_before_params,
        incdec_is_postfix, is_binder_keyword, of_is_forof_keyword, return_type_signature_paren,
        signature_return_type, tail_or_brace_before, type_alias_head, type_literal_brace_opener,
        type_prefix_kind,
    },
    walk::{
        AngleMatch, angle_match_back, bm_prev_sig, chain_head, ident_is, kind_at, lt_in_range,
        match_delim_back, prop_name, word_is_any, word_len,
    },
};

pub const LT_OPERAND_WORDS: &[&[u8]] = &[
    b"return",
    b"debugger",
    b"break",
    b"continue",
    b"default",
    b"class",
    b"function",
    b"throw",
    b"yield",
    b"await",
    b"typeof",
    b"void",
    b"delete",
    b"in",
    b"instanceof",
    b"case",
    b"else",
    b"do",
    b"of",
    b"extends",
    b"implements",
    b"as",
    b"satisfies",
    b"keyof",
    b"infer",
    b"readonly",
    b"is",
    b"asserts",
    b"unique",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum GtBrace {
    Value,
    Body,
    No,
}

#[inline]
pub unsafe fn brace_opens_value(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    brace: usize,
    ts: bool,
    depth: u32,
) -> bool {
    if depth > 64 {
        return false;
    }
    if brace_opens_object_literal(t, src, st, kind, n, brace, ts, depth) {
        return true;
    }
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false; // start of input => block
    }
    let p = q as usize;
    if kind_at(kind, p) >= OP_KIND_BASE {
        let ch = *src.add(p);
        if ts
            && ch == b'>'
            && !(p > 0 && *src.add(p - 1) == b'=')
            && ts_gt_brace(t, src, st, kind, n, p, depth) == GtBrace::Body
        {
            return true;
        }
        if ch == b')' {
            if let Some(lp) = match_delim_back(src, st, kind, p, b'(', b')') {
                if let Some(fk) = function_keyword_before_params(src, st, kind, lp) {
                    if operand_position(t, src, st, kind, n, fk, ts, depth) {
                        return true;
                    }
                }
            }
        }
    }
    if ts {
        if let Some(lp) = return_type_signature_paren(src, st, kind, brace) {
            if let Some(fk) = function_keyword_before_params(src, st, kind, lp) {
                return operand_position(t, src, st, kind, n, fk, ts, depth);
            }
        }
    }
    class_brace_is_value(t, src, st, kind, n, brace, ts, depth)
}

unsafe fn class_brace_is_value(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    brace: usize,
    ts: bool,
    depth: u32,
) -> bool {
    class_walk_from(t, src, st, kind, n, bm_prev_sig(st, kind, brace), ts, depth)
}

/// Is the token at `pos` in operand (expression-only) position? Strict
/// whitelist; anything else returns false. Deliberately not `not_operator_position`:
/// a regex may follow `;`/`:`/`else`, but a `{` there is a block, so reusing
/// it would be unsound. Complement of acorn's `braceIsBlock`.
#[inline]
pub unsafe fn operand_position(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    pos: usize,
    ts: bool,
    depth: u32,
) -> bool {
    operand_position_at(t, src, st, kind, n, pos, ts, depth, 0)
}

unsafe fn operand_position_at(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    pos: usize,
    ts: bool,
    depth: u32,
    hops: u32,
) -> bool {
    if hops > 64 || depth > 64 {
        return false;
    }
    let q = bm_prev_sig(st, kind, pos);
    if q < 0 {
        return false; // start of input => statement position
    }
    let p = q as usize;
    if kind_at(kind, p) < OP_KIND_BASE {
        let k = kind_at(kind, p);
        if k == TMPL_HEAD || k == TMPL_MIDDLE {
            return true;
        }
        if k == IDENT && !prop_name(src, p) {
            if ident_is(src, p, b"void") {
                return !(ts && void_is_type_position(t, src, st, kind, n, p, depth));
            }
            if ident_is(src, p, b"new")
                || ident_is(src, p, b"typeof")
                || ident_is(src, p, b"delete")
                || ident_is(src, p, b"in")
                || ident_is(src, p, b"instanceof")
                || ident_is(src, p, b"case")
            {
                return true;
            }
            if ident_is(src, p, b"return")
                || ident_is(src, p, b"throw")
                || ident_is(src, p, b"yield")
                || ident_is(src, p, b"await")
            {
                let e = bm_next1(st, p + 1, n);
                return !lt_in_range(src, e, pos);
            }
            if ident_is(src, p, b"async") && ident_is(src, pos, b"function") {
                let e = bm_next1(st, p + 1, n);
                if !lt_in_range(src, e, pos) {
                    return operand_position_at(t, src, st, kind, n, p, ts, depth, hops + 1);
                }
                return false;
            }
            if ident_is(src, p, b"of") && of_is_forof_keyword(t, src, st, kind, n, p) {
                return true;
            }
            if ident_is(src, p, b"default") && *src.add(pos) == b'{' {
                let e = bm_prev_sig(st, kind, p);
                return e >= 0
                    && kind_at(kind, e as usize) == IDENT
                    && !prop_name(src, e as usize)
                    && ident_is(src, e as usize, b"export");
            }
        }
        if let Some(at) = decorator_start(src, st, kind, p) {
            return operand_position_at(t, src, st, kind, n, at, ts, depth, hops + 1);
        }
        return false;
    }
    let ch = *src.add(p);
    if (ch == b'+' || ch == b'-') && p > 0 && *src.add(p - 1) == ch {
        return !(incdec_is_postfix(src, st, kind, n, p - 1) && lt_in_range(src, p + 1, pos));
    }
    if (ch == b'+' || ch == b'-') && *src.add(p + 1) == ch {
        return !(incdec_is_postfix(src, st, kind, n, p) && lt_in_range(src, p + 2, pos));
    }
    if (ch == b'>' && p > 0 && *src.add(p - 1) == b'=') || (ch == b'=' && *src.add(p + 1) == b'>') {
        return *src.add(pos) != b'{';
    }
    if ch == b'>' {
        if *src.add(pos) == b'{' {
            return false;
        }
        return !(ts
            && lt_in_range(src, p + 1, pos)
            && type_annotation_asi(t, src, st, kind, n, p, depth + 1));
    }
    if ch == b'.' {
        return (*src.add(p + 1) == b'.' && *src.add(p + 2) == b'.')
            || (p >= 2 && *src.add(p - 1) == b'.' && *src.add(p - 2) == b'.');
    }
    if ch == b'{' {
        return jsx_container_or_object_brace(t, src, st, kind, n, p, ts, depth + 1, false);
    }
    if ts && ch == b'!' && *src.add(p + 1) != b'=' && bang_is_postfix(src, st, kind, n, p) {
        return false;
    }
    if is_operand_punct(ch) {
        return true;
    }
    if ch == b':' {
        return colon_marks_value(t, src, st, kind, n, p, ts, depth);
    }
    if ch == b')' {
        if let Some(at) = decorator_start(src, st, kind, p) {
            return operand_position_at(t, src, st, kind, n, at, ts, depth, hops + 1);
        }
    }
    false
}

unsafe fn decorator_start(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    p: usize,
) -> Option<usize> {
    let mut q = p as i64;
    while q >= 0 {
        let w = q as usize;
        let kk = kind_at(kind, w);
        if kk >= OP_KIND_BASE {
            let c = *src.add(w);
            if c == b'@' {
                return Some(w);
            }
            if c == b')' {
                let lp = match_delim_back(src, st, kind, w, b'(', b')')?;
                q = bm_prev_sig(st, kind, lp);
                continue;
            }
            if c != b'.' {
                return None;
            }
        } else if kk != IDENT {
            return None;
        }
        q = bm_prev_sig(st, kind, w);
    }
    None
}

/// Punctuators after which only an expression can begin: a `{` here is an
/// object literal, a `function` a function expression. `>` is excluded
/// because `=>` ends in it (an arrow's `{}` is a block body); `=` is safe
/// because every operator ending in `=` lands on the `=`.
#[inline(always)]
fn is_operand_punct(ch: u8) -> bool {
    matches!(
        ch,
        b'(' | b'['
            | b','
            | b'='
            | b'?'
            | b'+'
            | b'-'
            | b'*'
            | b'/'
            | b'%'
            | b'&'
            | b'|'
            | b'^'
            | b'!'
            | b'~'
            | b'<'
    )
}

unsafe fn void_is_type_position(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    p: usize,
    depth: u32,
) -> bool {
    let q = bm_prev_sig(st, kind, p);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if k >= OP_KIND_BASE {
        return match *src.add(w) {
            b':' => !colon_marks_value(t, src, st, kind, n, w, true, depth + 1),
            b',' => class_like_walk(src, st, kind, w),
            _ => false,
        };
    }
    k == IDENT
        && !prop_name(src, w)
        && word_is_any(src, w, &[b"implements", b"extends"])
        && class_like_walk(src, st, kind, w)
}

unsafe fn colon_marks_value(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
    ts: bool,
    depth: u32,
) -> bool {
    if depth > 64 {
        return false;
    }
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
                b'{' => {
                    return brace_opens_object_literal(t, src, st, kind, n, w, ts, depth + 1);
                }
                b'(' | b'[' | b';' => return false,
                b':' => debt += 1,
                b'?' => {
                    if *src.add(w + 1) != b'?'
                        && *src.add(w + 1) != b'.'
                        && (w == 0 || *src.add(w - 1) != b'?')
                    {
                        if debt == 0 {
                            return true;
                        }
                        debt -= 1;
                    }
                }
                _ => {}
            }
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}

#[inline]
pub unsafe fn brace_opens_object_literal(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    brace: usize,
    ts: bool,
    depth: u32,
) -> bool {
    if depth > 64 {
        return false;
    }
    if operand_position(t, src, st, kind, n, brace, ts, depth) {
        return true;
    }
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false;
    }
    let p = q as usize;
    if kind_at(kind, p) >= OP_KIND_BASE {
        let ch = *src.add(p);
        if ch == b'>' {
            if p > 0 && *src.add(p - 1) == b'=' {
                return false;
            }
            if !ts {
                return true;
            }
            return ts_gt_brace(t, src, st, kind, n, p, depth) == GtBrace::Value;
        }
        return false;
    }
    ts && kind_at(kind, p) == IDENT
        && !prop_name(src, p)
        && (ident_is(src, p, b"as") || ident_is(src, p, b"satisfies"))
        && tail_or_brace_before(t, src, st, kind, n, p)
}

unsafe fn ts_gt_brace(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    gt: usize,
    depth: u32,
) -> GtBrace {
    let body = |yes: bool| if yes { GtBrace::Body } else { GtBrace::No };
    let lt = match angle_match_back(src, st, kind, gt) {
        AngleMatch::Found(lt) => lt,
        AngleMatch::NotType => return GtBrace::Value,
        AngleMatch::Unknown => return GtBrace::No,
    };
    let b = bm_prev_sig(st, kind, lt);
    if b < 0 {
        return GtBrace::Value;
    }
    let bp = b as usize;
    if kind_at(kind, bp) != IDENT {
        return GtBrace::Value;
    }
    let Some(head) = chain_head(src, st, kind, bp) else {
        return GtBrace::No;
    };
    let tq = bm_prev_sig(st, kind, head);
    if tq < 0 {
        return GtBrace::Value;
    }
    let tp = tq as usize;
    let tk = kind_at(kind, tp);
    if tk == IDENT && !prop_name(src, tp) {
        if ident_is(src, tp, b"class") {
            return body(operand_position(t, src, st, kind, n, tp, true, depth));
        }
        if ident_is(src, tp, b"interface") {
            return GtBrace::No;
        }
        if ident_is(src, tp, b"extends") || ident_is(src, tp, b"implements") {
            return body(class_walk_from(t, src, st, kind, n, tq, true, depth));
        }
        let e = bm_next1(st, tp + 1, n);
        if t.is_regex_keyword(src.add(tp), e - tp) {
            return GtBrace::Value;
        }
        return GtBrace::No;
    }
    if tk >= OP_KIND_BASE && *src.add(tp) == b':' {
        return body(colon_return_type_value(t, src, st, kind, n, tp, true, depth));
    }
    if tk >= OP_KIND_BASE && *src.add(tp) == b',' && class_like_walk(src, st, kind, tp) {
        return body(class_walk_from(t, src, st, kind, n, tq, true, depth));
    }
    if tk >= OP_KIND_BASE
        && (matches!(*src.add(tp), b'|' | b'&')
            || (*src.add(tp) == b'>' && tp > 0 && *src.add(tp - 1) == b'=')
            || (*src.add(tp) == b'=' && *src.add(tp + 1) == b'>'))
    {
        return GtBrace::No;
    }
    GtBrace::Value
}

unsafe fn class_walk_from(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    start: i64,
    ts: bool,
    depth: u32,
) -> bool {
    let mut q = start;
    let mut crossed_obj = false;
    let mut pending_comma = false;
    let mut hopped = false;
    while q >= 0 {
        let w = q as usize;
        let kk = kind_at(kind, w);
        if kk >= OP_KIND_BASE {
            let c = *src.add(w);
            if c == b')' || c == b']' {
                let open = if c == b')' { b'(' } else { b'[' };
                match match_delim_back(src, st, kind, w, open, c) {
                    Some(op) => {
                        hopped = true;
                        q = bm_prev_sig(st, kind, op);
                        continue;
                    }
                    None => return false,
                }
            }
            if c == b'}' {
                if crossed_obj {
                    return false;
                }
                let Some(ob) = match_delim_back(src, st, kind, w, b'{', b'}') else {
                    return false;
                };
                if ts && type_literal_brace_opener(src, st, kind, ob) {
                    q = bm_prev_sig(st, kind, ob);
                    continue;
                }
                let nx = bm_prev_sig(st, kind, ob);
                if nx < 0 {
                    return false;
                }
                let np = nx as usize;
                if kind_at(kind, np) != IDENT
                    || prop_name(src, np)
                    || !ident_is(src, np, b"extends")
                {
                    return false;
                }
                crossed_obj = true;
                q = nx;
                continue;
            }
            if ts && c == b',' {
                pending_comma = true;
                q = bm_prev_sig(st, kind, w);
                continue;
            }
            if c == b':' {
                if hopped {
                    return false;
                }
                return colon_return_type_value(t, src, st, kind, n, w, ts, depth)
                    || colon_marks_value(t, src, st, kind, n, w, ts, depth);
            }
            if ts && c == b'>' && !(w > 0 && *src.add(w - 1) == b'=') {
                match angle_match_back(src, st, kind, w) {
                    AngleMatch::Found(lt) => {
                        q = bm_prev_sig(st, kind, lt);
                        continue;
                    }
                    _ => return false,
                }
            }
            if c != b'.' && c != b'?' {
                return false;
            }
        } else if kk == IDENT {
            if !prop_name(src, w) {
                if word_is_any(src, w, CLASS_WALK_STOP_WORDS) {
                    return false;
                }
                if ident_is(src, w, b"class") {
                    if pending_comma {
                        return false;
                    }
                    return operand_position(t, src, st, kind, n, w, ts, depth);
                }
                if ts && ident_is(src, w, b"implements") {
                    pending_comma = false;
                }
            }
        } else if !matches!(kk, NUM | BIGINT | STR | REGEX | TMPL_NOSUB) {
            return false;
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}

unsafe fn colon_return_type_value(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
    ts: bool,
    depth: u32,
) -> bool {
    let q = bm_prev_sig(st, kind, colon);
    if q < 0 || kind_at(kind, q as usize) < OP_KIND_BASE || *src.add(q as usize) != b')' {
        return false;
    }
    let Some(lp) = match_delim_back(src, st, kind, q as usize, b'(', b')') else {
        return false;
    };
    function_keyword_before_params(src, st, kind, lp)
        .is_some_and(|fk| operand_position(t, src, st, kind, n, fk, ts, depth))
}

pub unsafe fn type_annotation_asi(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    from: usize,
    depth: u32,
) -> bool {
    let mut q = from as i64;
    let mut prev_atom = false;
    let mut prev_params = false;
    while q >= 0 {
        let w = q as usize;
        let kk = *kind.add(w);
        let ends;
        let starts;
        if kk >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b')' | b']' | b'}' => {
                    if prev_atom {
                        return false;
                    }
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    match match_delim_back(src, st, kind, w, open, c) {
                        Some(op) => {
                            if c == b'}' && !type_literal_brace_opener(src, st, kind, op) {
                                return false;
                            }
                            prev_atom = c != b']';
                            prev_params = c == b')';
                            q = bm_prev_sig(st, kind, op);
                            continue;
                        }
                        None => return false,
                    }
                }
                b'(' | b'[' | b'{' | b';' => return false,
                b':' => {
                    if annotation_colon_is_declaration(t, src, st, kind, n, w, depth) {
                        return true;
                    }
                    match conditional_type_question(src, st, kind, w) {
                        Some(qm) => {
                            prev_atom = false;
                            q = bm_prev_sig(st, kind, qm);
                            continue;
                        }
                        None => return false,
                    }
                }
                b'=' => {
                    if *src.add(w + 1) != b'>' {
                        if *src.add(w + 1) == b'=' {
                            return false;
                        }
                        if w > 0
                            && matches!(*src.add(w - 1), b'=' | b'!' | b'<' | b'>' | b'+' | b'-')
                        {
                            return false;
                        }
                        return type_alias_head(src, st, kind, w);
                    }
                    ends = false;
                    starts = false;
                }
                b'>' => {
                    if !(w > 0 && *src.add(w - 1) == b'=') {
                        if prev_atom && !prev_params {
                            return false;
                        }
                        match angle_match_back(src, st, kind, w) {
                            AngleMatch::Found(lt) => {
                                prev_atom = false;
                                prev_params = false;
                                q = bm_prev_sig(st, kind, lt);
                                continue;
                            }
                            _ => return false,
                        }
                    }
                    ends = false;
                    starts = false;
                }
                b'.' | b'|' | b'&' | b'?' | b'-' => {
                    ends = false;
                    starts = false;
                }
                _ => return false,
            }
        } else if kk == IDENT || kk == IDENT_ESC {
            let kw = t.kwts.lookup(src.add(w), word_len(src, w)) as u8;
            if matches!(kw, KW_EXTENDS | KW_IS | KW_IN | KW_AS) {
                ends = false;
                starts = false;
            } else if type_prefix_kind(kw) {
                ends = false;
                starts = true;
            } else {
                ends = true;
                starts = true;
            }
        } else if matches!(kk, NUM | BIGINT | STR | TMPL_NOSUB) {
            ends = true;
            starts = true;
        } else if kk == TMPL_HEAD {
            ends = false;
            starts = true;
        } else if kk == TMPL_MIDDLE {
            ends = false;
            starts = false;
        } else if kk == TMPL_TAIL {
            ends = true;
            starts = false;
        } else {
            return false;
        }
        if ends && prev_atom {
            return false;
        }
        prev_atom = starts;
        prev_params = false;
        q = bm_prev_sig(st, kind, w);
    }
    false
}

pub unsafe fn annotation_colon_is_declaration(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    colon: usize,
    depth: u32,
) -> bool {
    if colon_marks_value(t, src, st, kind, n, colon, true, depth) {
        return false;
    }
    let q = bm_prev_sig(st, kind, colon);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let kk = kind_at(kind, w);
    if kk == IDENT {
        return declarator_without_init(src, st, kind, w);
    }
    if kk >= OP_KIND_BASE {
        let c = *src.add(w);
        if c == b')' {
            return signature_return_type(src, st, kind, w);
        }
        if c == b'!' && *src.add(w + 1) != b'=' {
            let q2 = bm_prev_sig(st, kind, w);
            return q2 >= 0
                && kind_at(kind, q2 as usize) == IDENT
                && declarator_without_init(src, st, kind, q2 as usize);
        }
        if c == b']' || c == b'}' {
            let open = if c == b']' { b'[' } else { b'{' };
            if let Some(op) = match_delim_back(src, st, kind, w, open, c) {
                let p = bm_prev_sig(st, kind, op);
                if p >= 0 && is_binder_keyword(src, kind, p as usize) {
                    return binder_outside_paren_head(src, st, kind, p as usize);
                }
            }
        }
    }
    false
}

unsafe fn jsx_container_or_object_brace(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    brace: usize,
    ts: bool,
    depth: u32,
    strict: bool,
) -> bool {
    if depth > 64 {
        return false;
    }
    let q = bm_prev_sig(st, kind, brace);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    let k = kind_at(kind, w);
    if k == JTEXT || k == JEND {
        return true;
    }
    if k >= OP_KIND_BASE {
        match *src.add(w) {
            b'>' => {
                return !(w > 0 && *src.add(w - 1) == b'=')
                    && matches!(angle_match_back(src, st, kind, w), AngleMatch::NotType);
            }
            b'}' => {
                return match_delim_back(src, st, kind, w, b'{', b'}').is_some_and(|o| {
                    jsx_container_or_object_brace(t, src, st, kind, n, o, ts, depth + 1, true)
                });
            }
            _ => {}
        }
    }
    !strict && brace_opens_object_literal(t, src, st, kind, n, brace, ts, depth + 1)
}
