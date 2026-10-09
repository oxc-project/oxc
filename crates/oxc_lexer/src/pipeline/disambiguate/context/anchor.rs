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
    /// An operand starts at the paren here, whose group holds the query.
    Expr(usize),
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
                    let Some(o) = tokens.match_delim_back(w) else {
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
    if prev.is_member_dot(tokens) {
        return None;
    }
    let f = tokens.peek(e);
    if f.kind == tk!(Eof) {
        return None;
    }
    let f_kw = if f.kind == tk!(Ident) { tokens.ident_kw(f.pos) } else { 0 };
    let same_line = !tokens.line_break_between(e, f.pos);
    match kw {
        // A named function or class where only a declaration can start.
        tk!(KwFunction | KwClass | KwAsync) => {
            let head = match kw {
                tk!(KwAsync) if f_kw == tk!(KwFunction) && same_line => tokens.peek(f.pos + 1),
                tk!(KwAsync) => return None,
                _ => f,
            };
            let named = head.kind == tk!(Ident)
                || (kw != tk!(KwClass) && head.kind >= OP_KIND_BASE && head.byte == b'*');
            let start = match prev {
                Prev::None | Prev::Op(_, b';' | b'{') => true,
                Prev::Op(_, b'}') => !in_jsx_tag(tokens, p),
                Prev::Word(_, w) => {
                    matches_tk!(w, KwElse | KwDo | KwExport | KwDefault | KwDeclare | KwAbstract)
                }
                _ => false,
            };
            (named && start).then_some(Anchor::Stmt(p))
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
