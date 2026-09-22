use crate::token::{OP_KIND_BASE, tk};

use super::*;
use crate::pipeline::disambiguate::{RULE_SCAN_CAP, common::Prev};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Anchor {
    Stmt(usize),
    Expr(usize),
    Continue,
}

pub(super) fn in_jsx_tag(tokens: &Tokens, p: usize) -> bool {
    let mut q = tokens.prev_sig(p);
    let mut steps = 0u32;
    while let Some(w) = q {
        steps += 1;
        if steps > RULE_SCAN_CAP {
            // Out of budget: assume a tag, which only costs an anchor.
            return true;
        }
        let k = tokens.base_kind(w);
        if k == tk!(JsxLt) {
            return true;
        }
        if k >= OP_KIND_BASE {
            match tokens.src[w] {
                b'=' => {}
                b'}' | b')' | b']' => {
                    let c = tokens.src[w];
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    let Some(o) = tokens.match_delim_back(w, open, c) else {
                        return false;
                    };
                    q = tokens.prev_sig(o);
                    continue;
                }
                _ => return false,
            }
        } else if !matches!(
            k,
            tk!(Ident) | tk!(String) | tk!(Number) | tk!(BigInt) | tk!(TemplateNoSub)
        ) {
            return false;
        }
        q = tokens.prev_sig(w);
    }
    false
}

pub(super) fn keyword_follower(tokens: &Tokens, f: usize) -> bool {
    let k = tokens.base_kind(f);
    if k >= OP_KIND_BASE {
        return matches!(tokens.src[f], b'{' | b'-' | b'+' | b'~' | b'*' | b'@');
    }
    matches!(
        k,
        tk!(Ident)
            | tk!(String)
            | tk!(Number)
            | tk!(BigInt)
            | tk!(TemplateNoSub)
            | tk!(TemplateHead)
            | tk!(RegExp)
            | tk!(PrivateIdent)
            | tk!(JsxLt)
    )
}

pub(super) fn stmt_boundary(tokens: &Tokens, p: usize, prev: Prev) -> bool {
    match prev {
        Prev::None => true,
        Prev::Op(_, b';' | b'{' | b')' | b':') => true,
        Prev::Op(_, b'}') => !in_jsx_tag(tokens, p),
        Prev::Word(_, K_ELSE | K_DO | K_EXPORT | K_DEFAULT | K_DECLARE | K_ABSTRACT) => true,
        Prev::Op(q, _) | Prev::Word(q, _) | Prev::Other(q) => {
            let e = tokens.next_start(q + 1);
            tokens.line_break_between(e, p) && !in_jsx_tag(tokens, p)
        }
    }
}

pub(super) fn fn_class_anchor(
    tokens: &Tokens,
    at: usize,
    prev: Prev,
    class: bool,
    named: bool,
) -> Option<Anchor> {
    let broken = |q: usize| {
        let e = tokens.next_start(q + 1);
        tokens.line_break_between(e, at)
    };
    let stmt = Some(Anchor::Stmt(at));
    let expr = Some(Anchor::Expr(at));
    match prev {
        Prev::None => stmt,
        Prev::Op(q, c) => match c {
            b';' | b'{' => stmt,
            b'}' => {
                if in_jsx_tag(tokens, at) {
                    None
                } else {
                    stmt
                }
            }
            // if (x) function f() {}; a decorator's ) before a class expression.
            b')' => {
                if !class || (named && broken(q)) {
                    stmt
                } else {
                    None
                }
            }
            b']' => {
                if named && broken(q) {
                    stmt
                } else {
                    None
                }
            }
            b'>' if q > 0 && tokens.src[q - 1] == b'=' => expr,
            // A value or type ended on the previous line.
            b'>' => {
                if named && broken(q) {
                    stmt
                } else {
                    expr
                }
            }
            b'+' | b'-' if tokens.src[q + 1] == c || (q > 0 && tokens.src[q - 1] == c) => {
                if named && broken(q) { stmt } else { None }
            }
            b':' => {
                if named {
                    None
                } else {
                    expr
                }
            }
            _ => expr,
        },
        Prev::Word(_, K_ELSE | K_DO | K_EXPORT | K_DEFAULT | K_DECLARE | K_ABSTRACT) => stmt,
        // Restricted productions: a line break ends the statement.
        Prev::Word(q, K_RETURN | K_YIELD) => {
            if named && broken(q) {
                stmt
            } else {
                expr
            }
        }
        Prev::Word(
            _,
            K_TYPEOF | K_THROW | K_AWAIT | K_VOID | K_DELETE | K_NEW | K_IN | K_OF | K_INSTANCEOF
            | K_CASE,
        ) => expr,
        // A heritage expression: what follows it is the enclosing class's body.
        Prev::Word(_, K_EXTENDS) => None,
        Prev::Word(q, _) | Prev::Other(q) => {
            if named && broken(q) {
                stmt
            } else {
                None
            }
        }
    }
}

pub(super) fn function_anchor(tokens: &Tokens, at: usize, prev: Prev, f: usize) -> Option<Anchor> {
    if f >= tokens.n {
        return None;
    }
    let fk = tokens.base_kind(f);
    let fc = tokens.src[f];
    if fk == tk!(Ident) || (fk >= OP_KIND_BASE && fc == b'*') {
        fn_class_anchor(tokens, at, prev, false, true)
    } else if fk >= OP_KIND_BASE && fc == b'(' {
        match fn_class_anchor(tokens, at, prev, false, false) {
            Some(Anchor::Expr(a)) => Some(Anchor::Expr(a)),
            _ => None,
        }
    } else {
        None
    }
}

pub(super) fn anchor_at(tokens: &Tokens, p: usize) -> (u8, Option<Anchor>) {
    let e = tokens.next_start(p + 1);
    // Keywords are lowercase words of up to ten letters: skip the hash for the rest.
    if !tokens.src[p].is_ascii_lowercase() || e - p > 10 {
        return (0, None);
    }
    let kw = tokens.word_kw(p, e - p);
    let anchor = anchor_of(tokens, p, e, kw);
    (kw, anchor)
}

pub(super) fn anchor_of(tokens: &Tokens, p: usize, e: usize, kw: u8) -> Option<Anchor> {
    if kw == 0 {
        return None;
    }
    let prev = tokens.prev_token(p);
    // A property name.
    if let Prev::Op(q, c) = prev
        && (c == b'.' || (c == b'?' && tokens.src[q + 1] == b'.'))
    {
        return None;
    }
    let f = tokens.next_sig(e);
    if f >= tokens.n {
        return None;
    }
    let fk = tokens.base_kind(f);
    let fc = tokens.src[f];
    let f_kw = if fk == tk!(Ident) { tokens.ident_kw(f) } else { 0 };
    let same_line = !tokens.line_break_between(e, f);
    match kw {
        K_FUNCTION => function_anchor(tokens, p, prev, f),
        K_CLASS => {
            if fk == tk!(Ident) {
                fn_class_anchor(tokens, p, prev, true, true)
            } else if fk >= OP_KIND_BASE && matches!(fc, b'{' | b'<') {
                fn_class_anchor(tokens, p, prev, true, false)
            } else {
                None
            }
        }
        K_ASYNC => {
            if f_kw == K_FUNCTION && same_line {
                let e2 = tokens.next_start(f + 1);
                function_anchor(tokens, p, prev, tokens.next_sig(e2))
            } else {
                None
            }
        }
        K_VAR | K_CONST | K_RETURN | K_THROW | K_CASE | K_EXPORT | K_IMPORT | K_ENUM | K_ELSE
        | K_DO | K_TRY | K_FINALLY | K_BREAK | K_CONTINUE | K_DEBUGGER | K_IF | K_FOR | K_WHILE
        | K_SWITCH | K_WITH | K_CATCH => {
            if keyword_follower(tokens, f) && stmt_boundary(tokens, p, prev) {
                Some(Anchor::Stmt(p))
            } else {
                None
            }
        }
        K_LET | K_TYPE | K_INTERFACE | K_NAMESPACE | K_MODULE | K_DECLARE | K_ABSTRACT => {
            if !same_line {
                return None;
            }
            let ok = match kw {
                K_LET => {
                    fk == tk!(Ident)
                        && f_kw == 0
                        && second_follower(tokens, f, &[b'=', b';', b':', b','])
                }
                K_TYPE => {
                    fk == tk!(Ident) && f_kw == 0 && second_follower(tokens, f, &[b'=', b'<'])
                }
                K_INTERFACE => {
                    fk == tk!(Ident)
                        && f_kw == 0
                        && (second_follower(tokens, f, &[b'{', b'<'])
                            || second_word(tokens, f) == K_EXTENDS)
                }
                K_NAMESPACE => {
                    fk == tk!(Ident) && f_kw == 0 && second_follower(tokens, f, &[b'{', b'.'])
                }
                K_MODULE => {
                    ((fk == tk!(Ident) && f_kw == 0) || fk == tk!(String))
                        && second_follower(tokens, f, &[b'{'])
                }
                K_DECLARE => matches!(
                    f_kw,
                    K_CONST
                        | K_LET
                        | K_VAR
                        | K_FUNCTION
                        | K_CLASS
                        | K_MODULE
                        | K_NAMESPACE
                        | K_GLOBAL
                        | K_ENUM
                        | K_INTERFACE
                        | K_TYPE
                        | K_ABSTRACT
                        | K_ASYNC
                ),
                _ => f_kw == K_CLASS,
            };
            if !ok {
                return None;
            }
            let boundary = match prev {
                Prev::None => true,
                Prev::Op(_, b';') => true,
                Prev::Op(_, b'{') => kw == K_LET,
                Prev::Op(_, b'}') => !in_jsx_tag(tokens, p),
                Prev::Word(_, K_EXPORT | K_DECLARE) => true,
                _ => false,
            };
            if boundary { Some(Anchor::Stmt(p)) } else { None }
        }
        _ => None,
    }
}

pub(super) fn second_follower(tokens: &Tokens, f: usize, ops: &[u8]) -> bool {
    let e = tokens.next_start(f + 1);
    let s = tokens.next_sig(e);
    s < tokens.n && tokens.base_kind(s) >= OP_KIND_BASE && ops.contains(&tokens.src[s])
}

pub(super) fn second_word(tokens: &Tokens, f: usize) -> u8 {
    let e = tokens.next_start(f + 1);
    let s = tokens.next_sig(e);
    if s < tokens.n && tokens.base_kind(s) == tk!(Ident) { tokens.ident_kw(s) } else { 0 }
}

/// The token after the } at c if it must start a statement or member, else None.
pub(super) fn brace_boundary(tokens: &Tokens, c: usize) -> Option<usize> {
    let f = tokens.next_sig(c + 1);
    if f >= tokens.n {
        return None;
    }
    let k = tokens.base_kind(f);
    let ok = match k {
        tk!(Ident) => !matches!(
            tokens.ident_kw(f),
            K_AS | K_SATISFIES | K_IN | K_INSTANCEOF | K_OF | K_IMPLEMENTS | K_EXTENDS | K_FROM
        ),
        tk!(PrivateIdent) | tk!(String) | tk!(Number) | tk!(BigInt) => true,
        _ => k >= OP_KIND_BASE && tokens.src[f] == b'@',
    };
    if ok { Some(f) } else { None }
}

/// Where the walk starts when the ( at p is an expression's; None when it may be a type's.
pub(super) fn paren_anchor(tokens: &Tokens, p: usize) -> Option<Anchor> {
    match tokens.prev_token(p) {
        Prev::None => Some(Anchor::Expr(p)),
        Prev::Word(kp, K_IF | K_WHILE | K_FOR | K_WITH | K_SWITCH | K_CATCH)
            if !tokens.property_name(kp) =>
        {
            Some(Anchor::Stmt(kp))
        }
        Prev::Word(kp, K_AWAIT) => match tokens.prev_token(kp) {
            Prev::Word(fp, K_FOR) if !tokens.property_name(fp) => Some(Anchor::Stmt(fp)),
            _ => Some(Anchor::Expr(p)),
        },
        Prev::Word(
            _,
            0 | K_THIS | K_SUPER | K_RETURN | K_THROW | K_TYPEOF | K_YIELD | K_VOID | K_DELETE
            | K_IN | K_OF | K_INSTANCEOF | K_CASE | K_ELSE | K_DO | K_ASYNC | K_NULL | K_TRUE
            | K_FALSE,
        ) => Some(Anchor::Expr(p)),
        Prev::Word(..) => None,
        Prev::Op(q, c) => match c {
            b')' | b']' | b'}' | b'!' | b'+' | b'-' | b'*' | b'/' | b'%' | b'^' | b'~' => {
                Some(Anchor::Expr(p))
            }
            b'>' if !(q > 0 && tokens.src[q - 1] == b'=') => Some(Anchor::Expr(p)),
            _ => None,
        },
        Prev::Other(q) => match tokens.base_kind(q) {
            tk!(String)
            | tk!(Number)
            | tk!(BigInt)
            | tk!(TemplateNoSub)
            | tk!(TemplateTail)
            | tk!(RegExp)
            | tk!(PrivateIdent) => Some(Anchor::Expr(p)),
            _ => None,
        },
    }
}
