//! Anchors: tokens whose context is certain from their neighbours alone, where a bounded walk
//! may start. Statement keywords that are not property names, members or JSX attributes; a
//! `function` or `class` whose token before says declaration or expression; the `(` whose group
//! holds the query when the token before makes it an expression's.

use crate::token::{OP_KIND_BASE, matches_tk, tk};

use super::*;
use crate::pipeline::disambiguate::{
    RULE_SCAN_CAP,
    common::{Peek, Prev},
};

/// Where a bounded walk starts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Anchor {
    /// A statement starts at the token here.
    Stmt(usize),
    /// An operand starts at the token here: an expression-position `function` / `class`, or the
    /// `(` whose group holds the query.
    Expr(usize),
    /// The bounded walk already covers this point and continues from where it stopped.
    Continue { semi: u32, brace: u32 },
}

/// Is the word at `p` an attribute inside a JSX opening tag? Scans back over attribute names,
/// values and `=` to the tag's `<`.
fn in_jsx_tag(tokens: &Tokens, p: usize) -> bool {
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
        } else if !matches_tk!(k, Ident | String | Number | BigInt | TemplateNoSub) {
            return false;
        }
        q = tokens.prev_sig(w);
    }
    false
}

/// Can the token at `f` follow a statement keyword but not a property, member or attribute name?
#[rustfmt::skip::macros(matches_tk)]
fn keyword_follower(f: Peek) -> bool {
    if f.kind >= OP_KIND_BASE {
        return matches!(f.byte, b'{' | b'-' | b'+' | b'~' | b'*' | b'@');
    }
    matches_tk!(f.kind,
        Ident | String | Number | BigInt | TemplateNoSub | TemplateHead | RegExp
        | PrivateIdent | JsxLt
    )
}

/// Does a statement start at `p` as far as the token before it can tell? `}` needs the JSX check
/// (an attribute after a `{...}` value); a line break allows a statement after anything else.
fn stmt_boundary(tokens: &Tokens, p: usize, prev: Prev) -> bool {
    match prev {
        Prev::None => true,
        Prev::Op(_, b';' | b'{' | b')' | b':') => true,
        Prev::Op(_, b'}') => !in_jsx_tag(tokens, p),
        Prev::Word(_, tk!(KwElse | KwDo | KwExport | KwDefault | KwDeclare | KwAbstract)) => true,
        Prev::Op(q, _) | Prev::Word(q, _) | Prev::Other(q) => {
            let e = tokens.next_start(q + 1);
            tokens.line_break_between(e, p) && !in_jsx_tag(tokens, p)
        }
    }
}

/// `function` / `class` at `at` (`async` for `async function`): a declaration or an expression,
/// read off the token before. `named` says a name follows (a label's `:` then precedes a
/// declaration, a property's an expression; only a name allows ASI to start a declaration).
fn fn_class_anchor(
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
            // `if (x) function f() {}`; a decorator's `)` before a class expression.
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
        Prev::Word(_, tk!(KwElse | KwDo | KwExport | KwDefault | KwDeclare | KwAbstract)) => stmt,
        // Restricted productions: a line break ends the statement.
        Prev::Word(q, tk!(KwReturn | KwYield)) => {
            if named && broken(q) {
                stmt
            } else {
                expr
            }
        }
        #[rustfmt::skip]
        Prev::Word(
            _,
            tk!(KwTypeof | KwThrow | KwAwait | KwVoid | KwDelete | KwNew | KwIn | KwOf | KwInstanceof | KwCase)
        ) => expr,
        // A heritage expression: what follows it is the enclosing class's body.
        Prev::Word(_, tk!(KwExtends)) => None,
        Prev::Word(q, _) | Prev::Other(q) => {
            if named && broken(q) {
                stmt
            } else {
                None
            }
        }
    }
}

/// `function` at `at` (or the `async` before it), with the token after `function` at `f`: a named
/// function is a declaration or an expression by its context; an anonymous one is an expression,
/// or a method named `function`, which is no anchor.
fn function_anchor(tokens: &Tokens, at: usize, prev: Prev, f: Peek) -> Option<Anchor> {
    if f.kind == tk!(Ident) || (f.kind >= OP_KIND_BASE && f.byte == b'*') {
        fn_class_anchor(tokens, at, prev, false, true)
    } else if f.kind >= OP_KIND_BASE && f.byte == b'(' {
        match fn_class_anchor(tokens, at, prev, false, false) {
            Some(Anchor::Expr(a)) => Some(Anchor::Expr(a)),
            _ => None,
        }
    } else {
        None
    }
}

/// Is the word at `p` an anchor? Also its keyword code (0: a plain name).
pub(super) fn anchor_at(tokens: &Tokens, p: usize) -> (u8, Option<Anchor>) {
    let e = tokens.next_start(p + 1);
    let kw = tokens.word_kw(p, e - p);
    (kw, anchor_of(tokens, p, e, kw))
}

#[rustfmt::skip::macros(tk)]
fn anchor_of(tokens: &Tokens, p: usize, e: usize, kw: u8) -> Option<Anchor> {
    if kw == 0 {
        return None;
    }
    let prev = tokens.prev_token(p);
    // A property name.
    if prev.is_member_dot(tokens.src) {
        return None;
    }
    let f = tokens.peek(e);
    if f.kind == tk!(Eof) {
        return None;
    }
    let f_kw = if f.kind == tk!(Ident) { tokens.ident_kw(f.pos) } else { 0 };
    let same_line = !tokens.line_break_between(e, f.pos);
    match kw {
        tk!(KwFunction) => function_anchor(tokens, p, prev, f),
        tk!(KwClass) => {
            if f.kind == tk!(Ident) {
                fn_class_anchor(tokens, p, prev, true, true)
            } else if f.kind >= OP_KIND_BASE && matches!(f.byte, b'{' | b'<') {
                fn_class_anchor(tokens, p, prev, true, false)
            } else {
                None
            }
        }
        tk!(KwAsync) => {
            if f_kw == tk!(KwFunction) && same_line {
                function_anchor(tokens, p, prev, tokens.peek(f.pos + 1))
            } else {
                None
            }
        }
        tk!(
            KwVar | KwConst | KwReturn | KwThrow | KwCase | KwExport | KwImport | KwEnum | KwElse
            | KwDo | KwTry | KwFinally | KwBreak | KwContinue | KwDebugger | KwIf | KwFor | KwWhile
            | KwSwitch | KwWith | KwCatch
        ) => {
            if keyword_follower(f) && stmt_boundary(tokens, p, prev) {
                Some(Anchor::Stmt(p))
            } else {
                None
            }
        }
        tk!(KwLet | KwType | KwInterface | KwNamespace | KwModule | KwDeclare | KwAbstract) => {
            if !same_line {
                return None;
            }
            let ok = match kw {
                tk!(KwLet) => {
                    f.kind == tk!(Ident)
                        && f_kw == 0
                        && second_follower(tokens, f.pos, &[b'=', b';', b':', b','])
                }
                tk!(KwType) => {
                    f.kind == tk!(Ident)
                        && f_kw == 0
                        && second_follower(tokens, f.pos, &[b'=', b'<'])
                }
                tk!(KwInterface) => {
                    f.kind == tk!(Ident)
                        && f_kw == 0
                        && (second_follower(tokens, f.pos, &[b'{', b'<'])
                            || second_word(tokens, f.pos) == tk!(KwExtends))
                }
                tk!(KwNamespace) => {
                    f.kind == tk!(Ident)
                        && f_kw == 0
                        && second_follower(tokens, f.pos, &[b'{', b'.'])
                }
                tk!(KwModule) => {
                    ((f.kind == tk!(Ident) && f_kw == 0) || f.kind == tk!(String))
                        && second_follower(tokens, f.pos, &[b'{'])
                }
                tk!(KwDeclare) => matches_tk!(
                    f_kw,
                    KwConst | KwLet | KwVar | KwFunction | KwClass | KwModule | KwNamespace
                    | KwGlobal | KwEnum | KwInterface | KwType | KwAbstract | KwAsync
                ),
                _ => f_kw == tk!(KwClass),
            };
            if !ok {
                return None;
            }
            let boundary = match prev {
                Prev::None => true,
                Prev::Op(_, b';') => true,
                Prev::Op(_, b'{') => kw == tk!(KwLet),
                Prev::Op(_, b'}') => !in_jsx_tag(tokens, p),
                Prev::Word(_, tk!(KwExport | KwDeclare)) => true,
                _ => false,
            };
            if boundary { Some(Anchor::Stmt(p)) } else { None }
        }
        _ => None,
    }
}

/// Is the significant token after the word at `f` one of `ops`?
fn second_follower(tokens: &Tokens, f: usize, ops: &[u8]) -> bool {
    let s = tokens.peek(f + 1);
    s.kind >= OP_KIND_BASE && ops.contains(&s.byte)
}

/// Keyword code of the significant word after the word at `f` (0 if none).
fn second_word(tokens: &Tokens, f: usize) -> u8 {
    let s = tokens.peek(f + 1);
    if s.kind == tk!(Ident) { tokens.ident_kw(s.pos) } else { 0 }
}

/// After the `}` at `c`: the position of a token that must start a statement or member there
/// (a name, string, number, private name or decorator; not `as` / `satisfies` / `in` /
/// `instanceof`, which continue a value), or None.
pub(super) fn brace_boundary(tokens: &Tokens, c: usize) -> Option<usize> {
    let f = tokens.peek(c + 1);
    let ok = match f.kind {
        tk!(Ident) => !matches_tk!(
            tokens.ident_kw(f.pos),
            KwAs | KwSatisfies | KwIn | KwInstanceof | KwOf | KwImplements | KwExtends | KwFrom
        ),
        tk!(PrivateIdent | String | Number | BigInt) => true,
        _ => f.kind >= OP_KIND_BASE && f.byte == b'@',
    };
    ok.then_some(f.pos)
}

/// The `(` at `p` holds the query: where the walk starts when the token before makes the paren an
/// expression's (a call, a grouping, a statement head). None when it may be a type's, such as after
/// `:`, `=`, `,`, `<` or `=>`: the walk then reaches it from an anchor further back.
#[rustfmt::skip::macros(tk)]
pub(super) fn paren_anchor(tokens: &Tokens, p: usize) -> Option<Anchor> {
    match tokens.prev_token(p) {
        Prev::None => Some(Anchor::Expr(p)),
        Prev::Word(kp, tk!(KwIf | KwWhile | KwFor | KwWith | KwSwitch | KwCatch))
            if !tokens.property_name(kp) =>
        {
            Some(Anchor::Stmt(kp))
        }
        Prev::Word(kp, tk!(KwAwait)) => match tokens.prev_token(kp) {
            Prev::Word(fp, tk!(KwFor)) if !tokens.property_name(fp) => Some(Anchor::Stmt(fp)),
            _ => Some(Anchor::Expr(p)),
        },
        Prev::Word(
            _,
            0 | tk!(
                KwThis | KwSuper | KwReturn | KwThrow | KwTypeof | KwYield | KwVoid | KwDelete | KwIn
                | KwOf | KwInstanceof | KwCase | KwElse | KwDo | KwAsync | KwNull | KwTrue | KwFalse
            )
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
            tk!(
                String | Number | BigInt | TemplateNoSub | TemplateTail | RegExp | PrivateIdent
            ) => Some(Anchor::Expr(p)),
            _ => None,
        },
    }
}
