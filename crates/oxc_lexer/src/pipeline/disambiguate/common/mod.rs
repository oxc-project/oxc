use core::cell::{Cell, RefCell};

use crate::{
    opmap::OP_KIND_BASE,
    tables::{Tables, is_word},
    token::{KW_BASE, KW_MAX, TokenKind},
};

use super::super::{
    BCOM, BIGINT, HASHBANG, IDENT, IDENT_ESC, JEND, JTEXT, LCOM, NUM, PRIV_IDENT, PRIV_IDENT_ESC,
    REGEX, STR, TMPL_HEAD, TMPL_MIDDLE, TMPL_NOSUB, TMPL_TAIL, WS,
    bitmap::{bm_next1, bm_prev1},
};

const ANGLE_MATCH_CAP: u32 = 4096;

/// Distance cap (in token starts) for the backward delimiter matches below;
/// past it we fall back to the safe legacy "`}` means regex" answer. Only
/// pathological input gets near it.
const BRACE_MATCH_CAP: u32 = 1024;

pub(super) const KW_ABSTRACT: u8 = TokenKind::KwAbstract as u8;
pub(super) const KW_AS: u8 = TokenKind::KwAs as u8;
pub(super) const KW_ASSERTS: u8 = TokenKind::KwAsserts as u8;
pub(super) const KW_AWAIT: u8 = TokenKind::KwAwait as u8;
pub(super) const KW_BREAK: u8 = TokenKind::KwBreak as u8;
pub(super) const KW_CASE: u8 = TokenKind::KwCase as u8;
pub(super) const KW_CATCH: u8 = TokenKind::KwCatch as u8;
pub(super) const KW_CLASS: u8 = TokenKind::KwClass as u8;
pub(super) const KW_CONST: u8 = TokenKind::KwConst as u8;
pub(super) const KW_CONTINUE: u8 = TokenKind::KwContinue as u8;
pub(super) const KW_DEBUGGER: u8 = TokenKind::KwDebugger as u8;
pub(super) const KW_DEFAULT: u8 = TokenKind::KwDefault as u8;
pub(super) const KW_DELETE: u8 = TokenKind::KwDelete as u8;
pub(super) const KW_DO: u8 = TokenKind::KwDo as u8;
pub(super) const KW_ELSE: u8 = TokenKind::KwElse as u8;
pub(super) const KW_ENUM: u8 = TokenKind::KwEnum as u8;
pub(super) const KW_EXPORT: u8 = TokenKind::KwExport as u8;
pub(super) const KW_EXTENDS: u8 = TokenKind::KwExtends as u8;
pub(super) const KW_FINALLY: u8 = TokenKind::KwFinally as u8;
pub(super) const KW_FOR: u8 = TokenKind::KwFor as u8;
pub(super) const KW_FUNCTION: u8 = TokenKind::KwFunction as u8;
pub(super) const KW_IF: u8 = TokenKind::KwIf as u8;
pub(super) const KW_IMPORT: u8 = TokenKind::KwImport as u8;
pub(super) const KW_IN: u8 = TokenKind::KwIn as u8;
pub(super) const KW_INFER: u8 = TokenKind::KwInfer as u8;
pub(super) const KW_INSTANCEOF: u8 = TokenKind::KwInstanceof as u8;
pub(super) const KW_IS: u8 = TokenKind::KwIs as u8;
pub(super) const KW_KEYOF: u8 = TokenKind::KwKeyof as u8;
pub(super) const KW_LET: u8 = TokenKind::KwLet as u8;
pub(super) const KW_NEW: u8 = TokenKind::KwNew as u8;
pub(super) const KW_READONLY: u8 = TokenKind::KwReadonly as u8;
pub(super) const KW_RETURN: u8 = TokenKind::KwReturn as u8;
pub(super) const KW_SUPER: u8 = TokenKind::KwSuper as u8;
pub(super) const KW_SWITCH: u8 = TokenKind::KwSwitch as u8;
pub(super) const KW_THIS: u8 = TokenKind::KwThis as u8;
pub(super) const KW_THROW: u8 = TokenKind::KwThrow as u8;
pub(super) const KW_TRY: u8 = TokenKind::KwTry as u8;
pub(super) const KW_TYPEOF: u8 = TokenKind::KwTypeof as u8;
pub(super) const KW_UNIQUE: u8 = TokenKind::KwUnique as u8;
pub(super) const KW_VAR: u8 = TokenKind::KwVar as u8;
pub(super) const KW_WHILE: u8 = TokenKind::KwWhile as u8;
pub(super) const KW_WITH: u8 = TokenKind::KwWith as u8;
pub(super) const KW_YIELD: u8 = TokenKind::KwYield as u8;

const CLASS_WALK_STOP_WORDS: &[&[u8]] = &[
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

pub(super) const LT_OPERAND_WORDS: &[&[u8]] = &[
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

pub(super) enum AngleMatch {
    Found(usize),
    NotType,
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GtBrace {
    Value,
    Body,
    No,
}

struct DelimMemo {
    generation: u64,
    upto: usize,
    pairs: Vec<(u32, u32)>,
    open: [Vec<u32>; 3],
}

thread_local! {
    static MEMO_GEN: Cell<u64> = const { Cell::new(0) };
    static DELIM_MEMO: RefCell<DelimMemo> = const {
        RefCell::new(DelimMemo {
            generation: 0,
            upto: 0,
            pairs: Vec::new(),
            open: [Vec::new(), Vec::new(), Vec::new()],
        })
    };
}

pub fn memo_new_lex() {
    MEMO_GEN.with(|g| g.set(g.get().wrapping_add(1)));
}

#[inline]
pub(super) unsafe fn brace_opens_value(
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
pub(super) unsafe fn operand_position(
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
pub(super) unsafe fn brace_opens_object_literal(
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

pub(super) unsafe fn type_annotation_asi(
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

pub(super) unsafe fn annotation_colon_is_declaration(
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

pub(super) unsafe fn return_type_signature_paren(
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
        } else if k == IDENT {
            if !prop_name(src, w) && word_is_any(src, w, RETURN_TYPE_STOP_WORDS) {
                return None;
            }
        } else if !matches!(
            k,
            NUM | BIGINT | STR | TMPL_NOSUB | TMPL_HEAD | TMPL_MIDDLE | TMPL_TAIL
        ) {
            return None;
        }
        q = bm_prev_sig(st, kind, w);
    }
    None
}

unsafe fn type_literal_brace_opener(
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
    k == IDENT && !prop_name(src, w) && word_is_any(src, w, TYPE_LITERAL_HEAD_WORDS)
}

pub(super) unsafe fn conditional_type_question(
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

pub(super) unsafe fn extends_precedes_question(
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
        } else if kind_at(kind, w) == IDENT && !prop_name(src, w) && ident_is(src, w, b"extends") {
            return true;
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}

pub(super) unsafe fn of_is_forof_keyword(
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
    if tk == IDENT || tk == IDENT_ESC {
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
                        if h < 0 || *kind.add(h as usize) != IDENT {
                            return false;
                        }
                        let mut hp = h as usize;
                        if ident_is(src, hp, b"await") {
                            let h2 = bm_prev_sig(st, kind, hp);
                            if h2 < 0 || *kind.add(h2 as usize) != IDENT {
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
        } else if kk == IDENT
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
                let tail = if bk == IDENT || bk == IDENT_ESC {
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

pub(super) unsafe fn incdec_is_postfix(
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
    let value = matches!(vk, IDENT | IDENT_ESC | NUM | BIGINT | STR | TMPL_NOSUB | TMPL_TAIL)
        || (vk >= OP_KIND_BASE && matches!(*src.add(v), b')' | b']'));
    value && !lt_in_range(src, bm_next1(st, v + 1, n), first)
}

pub(super) unsafe fn bang_is_postfix(
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
    let value = matches!(vk, IDENT | PRIV_IDENT | NUM | BIGINT | STR | TMPL_NOSUB | TMPL_TAIL)
        || (vk >= OP_KIND_BASE && matches!(*src.add(v), b')' | b']' | b'}'));
    value && !lt_in_range(src, bm_next1(st, v + 1, n), bang)
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

pub(super) unsafe fn class_like_walk(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    from: usize,
) -> bool {
    let mut q = from as i64;
    while q >= 0 {
        let w = q as usize;
        let k = kind_at(kind, w);
        if k == IDENT || k == IDENT_ESC {
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
        } else if !matches!(k, STR | NUM | TMPL_NOSUB) {
            return false;
        }
        q = bm_prev_sig(st, kind, w);
    }
    false
}

pub(super) unsafe fn type_alias_head(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    eq: usize,
) -> bool {
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
    if kind_at(kind, w) != IDENT || prop_name(src, w) {
        return false;
    }
    let p = bm_prev_sig(st, kind, w);
    p >= 0
        && kind_at(kind, p as usize) == IDENT
        && !prop_name(src, p as usize)
        && ident_is(src, p as usize, b"type")
}

pub(super) unsafe fn declarator_without_init(
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
pub(super) unsafe fn is_binder_keyword(src: *const u8, kind: *const u8, w: usize) -> bool {
    kind_at(kind, w) == IDENT
        && !prop_name(src, w)
        && (ident_is(src, w, b"let")
            || ident_is(src, w, b"const")
            || ident_is(src, w, b"var")
            || ident_is(src, w, b"using"))
}

#[inline]
unsafe fn binder_outside_paren_head(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    binder: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, binder);
    q < 0 || !(kind_at(kind, q as usize) >= OP_KIND_BASE && *src.add(q as usize) == b'(')
}

pub(super) unsafe fn signature_return_type(
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

unsafe fn function_keyword_before_params(
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
pub(super) unsafe fn as_type_operand(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    pos: usize,
) -> bool {
    let q = bm_prev_sig(st, kind, pos);
    if q < 0 {
        return false;
    }
    let w = q as usize;
    *kind.add(w) == IDENT
        && !prop_name(src, w)
        && (ident_is(src, w, b"as") || ident_is(src, w, b"satisfies"))
}

pub(super) unsafe fn as_gated_type_ref(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    n: usize,
    lt: usize,
) -> bool {
    let b = bm_prev_sig(st, kind, lt);
    if b < 0 || *kind.add(b as usize) != IDENT {
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
    if *kind.add(ap) != IDENT
        || prop_name(src, ap)
        || !(ident_is(src, ap, b"as") || ident_is(src, ap, b"satisfies"))
    {
        return false;
    }
    tail_or_brace_before(t, src, st, kind, n, ap)
}

unsafe fn tail_or_brace_before(
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

pub(super) unsafe fn tail_before(
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
    if sk == IDENT {
        let e = bm_next1(st, sp + 1, n);
        return prop_name(src, sp) || !t.is_regex_keyword(src.add(sp), e - sp);
    }
    matches!(sk, NUM | BIGINT | STR | TMPL_NOSUB | TMPL_TAIL | REGEX | PRIV_IDENT)
}

#[inline(always)]
pub(super) fn type_prefix_kind(k: u8) -> bool {
    matches!(
        k,
        KW_KEYOF
            | KW_TYPEOF
            | KW_READONLY
            | KW_UNIQUE
            | KW_INFER
            | KW_ABSTRACT
            | KW_NEW
            | KW_ASSERTS
            | KW_IMPORT
            | KW_EXTENDS
            | KW_IS
            | KW_IN
            | KW_AS
    )
}

pub(super) unsafe fn angle_match_back(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    gt: usize,
) -> AngleMatch {
    let mut depth: i32 = 1;
    let mut tmpl: i32 = 0;
    let mut steps: u32 = 0;
    let mut q = bm_prev_sig(st, kind, gt);
    while q >= 0 {
        steps += 1;
        if steps > ANGLE_MATCH_CAP {
            return AngleMatch::Unknown;
        }
        let w = q as usize;
        let kk = kind_at(kind, w);
        if tmpl > 0 {
            if kk == TMPL_TAIL {
                tmpl += 1;
            } else if kk == TMPL_HEAD {
                tmpl -= 1;
            }
            q = bm_prev_sig(st, kind, w);
            continue;
        }
        if kk == TMPL_TAIL {
            tmpl = 1;
            q = bm_prev_sig(st, kind, w);
            continue;
        }
        if kk >= OP_KIND_BASE {
            let c = *src.add(w);
            match c {
                b'>' => {
                    if !(w > 0 && *src.add(w - 1) == b'=') {
                        depth += 1;
                    }
                }
                b'<' => {
                    if *src.add(w + 1) == b'=' {
                        return AngleMatch::NotType;
                    }
                    depth -= 1;
                    if depth == 0 {
                        return AngleMatch::Found(w);
                    }
                }
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
                        None => return AngleMatch::Unknown,
                    }
                }
                b'(' | b'[' | b'{' | b';' => return AngleMatch::NotType,
                b'.' | b',' | b'|' | b'&' | b'?' | b':' | b'=' | b'+' | b'-' => {}
                _ => return AngleMatch::NotType,
            }
        } else if !matches!(kk, IDENT | IDENT_ESC | NUM | BIGINT | STR | TMPL_NOSUB) {
            if kk == TMPL_HEAD || kk == TMPL_MIDDLE {
                return AngleMatch::Unknown;
            }
            return AngleMatch::NotType;
        }
        q = bm_prev_sig(st, kind, w);
    }
    AngleMatch::NotType
}

pub(super) unsafe fn chain_head(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    from: usize,
) -> Option<usize> {
    let mut head = from;
    while prop_name(src, head) {
        let d = bm_prev_sig(st, kind, head);
        if d < 0 {
            return None;
        }
        let o = bm_prev_sig(st, kind, d as usize);
        if o < 0 {
            return None;
        }
        let op = o as usize;
        if kind_at(kind, op) == IDENT {
            head = op;
            continue;
        }
        if kind_at(kind, op) >= OP_KIND_BASE && *src.add(op) == b')' {
            let lp = match_delim_back(src, st, kind, op, b'(', b')')?;
            let im = bm_prev_sig(st, kind, lp);
            if im >= 0
                && kind_at(kind, im as usize) == IDENT
                && ident_is(src, im as usize, b"import")
            {
                return Some(im as usize);
            }
        }
        return None;
    }
    Some(head)
}

/// Match the close punctuator at `from` back to its opener, counting only
/// punctuator delimiters — template-closing `}`s and cleared literal
/// interiors are invisible. Past the cap the answer comes from a per-file
/// closer-to-opener table built once, so it is exact; None if unbalanced.
#[inline]
pub(super) unsafe fn match_delim_back(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    from: usize,
    open: u8,
    close: u8,
) -> Option<usize> {
    let mut depth: i32 = 1;
    let mut steps: u32 = 0;
    let mut q = bm_prev1(st, from);
    while q >= 0 {
        steps += 1;
        if steps > BRACE_MATCH_CAP {
            return delim_memo_opener(src, st, kind, from);
        }
        let pos = q as usize;
        if *kind.add(pos) >= OP_KIND_BASE {
            let c = *src.add(pos);
            if c == close {
                depth += 1;
            } else if c == open {
                depth -= 1;
                if depth == 0 {
                    return Some(pos);
                }
            }
        }
        q = bm_prev1(st, pos);
    }
    None
}

#[inline(never)]
unsafe fn delim_memo_opener(
    src: *const u8,
    st: *const u64,
    kind: *const u8,
    from: usize,
) -> Option<usize> {
    let generation = MEMO_GEN.with(Cell::get);
    DELIM_MEMO.with(|cell| {
        let mut m = cell.borrow_mut();
        if m.generation != generation {
            m.generation = generation;
            m.upto = 0;
            m.pairs.clear();
            for stack in &mut m.open {
                stack.clear();
            }
        }
        let mut w = bm_next1(st, m.upto, from + 1);
        while w <= from {
            if *kind.add(w) >= OP_KIND_BASE {
                let c = *src.add(w);
                match c {
                    b'(' | b'[' | b'{' => m.open[delim_slot(c)].push(w as u32),
                    b')' | b']' | b'}' => {
                        if let Some(o) = m.open[delim_slot(c)].pop() {
                            m.pairs.push((w as u32, o));
                        }
                    }
                    _ => {}
                }
            }
            w = bm_next1(st, w + 1, from + 1);
        }
        m.upto = from + 1;
        match m.pairs.binary_search_by_key(&(from as u32), |pr| pr.0) {
            Ok(i) => Some(m.pairs[i].1 as usize),
            Err(_) => None,
        }
    })
}

fn delim_slot(c: u8) -> usize {
    match c {
        b'(' | b')' => 0,
        b'[' | b']' => 1,
        _ => 2,
    }
}

pub(super) unsafe fn word_is_any(src: *const u8, w: usize, words: &[&[u8]]) -> bool {
    let len = word_len(src, w);
    let first = *src.add(w);
    let mut i = 0;
    while i < words.len() {
        let kw = words[i];
        if kw.len() == len && kw[0] == first && ident_is(src, w, kw) {
            return true;
        }
        i += 1;
    }
    false
}

#[inline(always)]
pub(super) unsafe fn word_len(src: *const u8, w: usize) -> usize {
    let mut e = w + 1;
    while is_word(*src.add(e)) {
        e += 1;
    }
    e - w
}

/// Does the identifier at `pos` equal exactly `kw`? The following-byte check
/// rejects longer identifiers (the source pad makes it safe at EOF).
#[inline]
pub(super) unsafe fn ident_is(src: *const u8, pos: usize, kw: &[u8]) -> bool {
    let mut i = 0;
    while i < kw.len() {
        if *src.add(pos + i) != kw[i] {
            return false;
        }
        i += 1;
    }
    let after = pos + kw.len();
    !is_word(*src.add(after)) || trivia_at(src, after).is_some()
}

#[inline]
pub(super) unsafe fn trivia_at(src: *const u8, i: usize) -> Option<(bool, usize)> {
    let b = *src.add(i);
    if b < 0x80 {
        return None;
    }
    let b1 = *src.add(i + 1);
    let b2 = *src.add(i + 2);
    match b {
        0xc2 if b1 == 0xa0 || b1 == 0x85 => Some((false, 2)),
        0xe1 if b1 == 0x9a && b2 == 0x80 => Some((false, 3)),
        0xe2 if b1 == 0x80 && ((0x80..=0x8b).contains(&b2) || b2 == 0xaf) => Some((false, 3)),
        0xe2 if b1 == 0x80 && (b2 == 0xa8 || b2 == 0xa9) => Some((true, 3)),
        0xe2 if b1 == 0x81 && b2 == 0x9f => Some((false, 3)),
        0xe3 if b1 == 0x80 && b2 == 0x80 => Some((false, 3)),
        0xef if b1 == 0xbb && b2 == 0xbf => Some((false, 3)),
        _ => None,
    }
}

/// Previous significant token start before `pos` (skipping trivia), or -1 at
/// start of input.
#[inline]
pub unsafe fn bm_prev_sig(st: *const u64, kind: *const u8, pos: usize) -> i64 {
    let mut q = bm_prev1(st, pos);
    while q >= 0 {
        let k = *kind.add(q as usize);
        if k == WS || k == LCOM || k == BCOM || k == HASHBANG {
            q = bm_prev1(st, q as usize);
            continue;
        }
        break;
    }
    q
}

#[inline(always)]
pub(super) unsafe fn kind_at(kind: *const u8, w: usize) -> u8 {
    let k = *kind.add(w);
    if k >= KW_BASE && k <= KW_MAX {
        return IDENT;
    }
    if k == IDENT_ESC || k == PRIV_IDENT_ESC {
        return k & !(IDENT_ESC ^ IDENT);
    }
    k
}

#[inline]
pub(super) unsafe fn prop_name(src: *const u8, pos: usize) -> bool {
    pos > 0 && *src.add(pos - 1) == b'.' && (pos < 2 || *src.add(pos - 2) != b'.')
}

pub(super) unsafe fn lt_in_range(src: *const u8, a: usize, b: usize) -> bool {
    let mut i = a;
    while i < b {
        let c = *src.add(i);
        if c == b'\n' || c == b'\r' {
            return true;
        }
        if c == 0xe2
            && *src.add(i + 1) == 0x80
            && (*src.add(i + 2) == 0xa8 || *src.add(i + 2) == 0xa9)
        {
            return true;
        }
        i += 1;
    }
    false
}
