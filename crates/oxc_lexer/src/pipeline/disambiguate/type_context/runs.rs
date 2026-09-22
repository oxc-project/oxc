//! Splitting runs of angle brackets which open or close type argument lists.
//!
//! `coalesce` joins a `>` with the characters after it into operators like `>>`, `>>>` and `>=`.
//! That is wrong when the `>`s close type argument lists, as in `Foo<Bar<T>>`.
//! [`gt_run_split`] decides how many of the `>`s must stay separate tokens.
//! [`lt_run_split`] does the same for a `<<` which opens two lists, as in `Array<<T>(x: T) => T>`.

use crate::token::{OP_KIND_BASE, tk};

use super::{bytes::lt_run_opens_type_args, type_list::type_args_at};
use crate::pipeline::disambiguate::{
    RULE_SCAN_CAP,
    common::{Prev, Tokens},
    context::{self, Walks},
};

/// `coalesce` entry for a `>`-run (`>>`, `>>>`, and a `>` glued to `=`): the
/// number of leading `>` bytes to leave unfused, or 0 to fuse as today. Cold
/// by construction - `>>` occurs once per ~110 KB of production TypeScript.
///
/// A balanced region is necessary but not sufficient: TypeScript's
/// speculative type-argument parse in expression position also needs the
/// region to scan as types (`foo(a<b + 1, c<d >> (e))` is a shift) and the
/// token after the list to be one that may follow type arguments
/// (`foo(a<b, c<d >> e)` is a shift). A glued `=` makes the rescan yield a
/// compound operator, so `a<b>=c` fuses, while type context never rescans
/// and `var v: Foo<T>= 1` splits.
#[inline(never)]
pub fn gt_run_split(tokens: &Tokens, walks: &mut Walks, p: usize, run: usize) -> usize {
    let mut g = 0usize;
    while g < run && tokens.src[p + g] == b'>' {
        g += 1;
    }
    run_shortcut(tokens, p, g).unwrap_or_else(|| context::angles_before(tokens, walks, p)).min(g)
}

#[inline(never)]
pub fn lt_run_split(tokens: &Tokens, p: usize) -> bool {
    lt_run_opens_type_args(tokens, p)
}

/// The lists the run closes when its context cannot matter; None when the walk must decide.
fn run_shortcut(tokens: &Tokens, pos: usize, run: usize) -> Option<usize> {
    let mut pending = 0i32;
    let mut found = 0usize;
    let mut lt = 0usize;
    let mut bounded = false;
    let mut steps = 0u32;
    let mut q = tokens.prev_sig(pos);
    while let Some(p) = q {
        steps += 1;
        if steps > RULE_SCAN_CAP {
            return None;
        }
        let k = tokens.base_kind(p);
        if k >= OP_KIND_BASE {
            let c = tokens.src[p];
            match c {
                b')' | b']' | b'}' => {
                    let open = match c {
                        b')' => b'(',
                        b']' => b'[',
                        _ => b'{',
                    };
                    let o = tokens.match_delim_back(p, open, c)?;
                    q = tokens.prev_sig(o);
                    continue;
                }
                b'(' | b'[' | b'{' | b';' => {
                    bounded = true;
                    break;
                }
                b'>' => {
                    if !(p > 0 && tokens.src[p - 1] == b'=') {
                        pending += tokens.run_len(p, b'>');
                    }
                }
                b'<' if tokens.src[p + 1] != b'=' => {
                    let n = tokens.run_len(p, b'<');
                    if n != 1 {
                        return None;
                    }
                    if pending > 0 {
                        pending -= 1;
                    } else {
                        found += 1;
                        lt = p;
                        if found == run {
                            break;
                        }
                    }
                }
                _ => {}
            }
        } else if !matches!(
            k,
            tk!(Ident)
                | tk!(String)
                | tk!(Number)
                | tk!(BigInt)
                | tk!(PrivateIdent)
                | tk!(RegExp)
                | tk!(TemplateNoSub)
        ) {
            return None;
        }
        q = tokens.prev_sig(p);
    }
    if found == 0 {
        return if bounded || q.is_none() { Some(0) } else { None };
    }
    if list_in_any_context(tokens, lt) { Some(found) } else { None }
}

/// No expression reading: <T extends, a speculated list after a name, a type reference head.
fn list_in_any_context(tokens: &Tokens, lt: usize) -> bool {
    let t = tokens.next_sig(lt + 1);
    if t < tokens.n && tokens.base_kind(t) == tk!(Ident) {
        let x = tokens.next_sig(tokens.next_start(t + 1));
        if x < tokens.n && tokens.ident_kw(x) == tk!(KwExtends) {
            let f = tokens.next_sig(tokens.next_start(x + 1));
            let tag = f < tokens.n
                && tokens.base_kind(f) >= OP_KIND_BASE
                && matches!(tokens.src[f], b'=' | b'>' | b'/');
            if !tag {
                return true;
            }
        }
    }
    match tokens.prev_token(lt) {
        Prev::Word(k, tk!(KwFunction) | tk!(KwClass)) => !tokens.property_name(k),
        Prev::Word(head, 0) => {
            type_args_at(tokens, lt)
                || (!tokens.line_break_between(tokens.next_start(head + 1), lt)
                    && type_reference_head(tokens, head))
        }
        _ => false,
    }
}

fn type_reference_head(tokens: &Tokens, head: usize) -> bool {
    let mut h = head;
    let mut prev = tokens.prev_token(h);
    while let Prev::Op(d, b'.') = prev {
        if tokens.src[d + 1] == b'.' {
            break;
        }
        let Prev::Word(w, _) = tokens.prev_token(d) else {
            break;
        };
        h = w;
        prev = tokens.prev_token(h);
    }
    match prev {
        // class, extends, function are reserved, so a name on the next line is still theirs.
        Prev::Word(k, tk!(KwExtends) | tk!(KwClass) | tk!(KwFunction)) => !tokens.property_name(k),
        Prev::Word(
            k,
            tk!(KwImplements)
            | tk!(KwAs)
            | tk!(KwSatisfies)
            | tk!(KwKeyof)
            | tk!(KwInterface)
            | tk!(KwType),
        ) => !tokens.property_name(k) && !tokens.line_break_between(tokens.next_start(k + 1), h),
        Prev::Op(c, b':') => return_type_colon(tokens, c),
        _ => false,
    }
}

/// Is the : at c a return type's: function (...):  or name(...):  with no room for a call?
fn return_type_colon(tokens: &Tokens, c: usize) -> bool {
    let Prev::Op(rp, b')') = tokens.prev_token(c) else {
        return false;
    };
    let Some(lp) = tokens.match_delim_back(rp, b'(', b')') else {
        return false;
    };
    match tokens.prev_token(lp) {
        Prev::Word(f, tk!(KwFunction)) => !tokens.property_name(f),
        Prev::Word(_, tk!(KwCase)) => false,
        Prev::Word(name, _) => matches!(
            tokens.prev_token(name),
            Prev::None
                | Prev::Op(_, b'{' | b'}' | b';')
                | Prev::Word(
                    _,
                    tk!(KwFunction)
                        | tk!(KwStatic)
                        | tk!(KwAsync)
                        | tk!(KwPublic)
                        | tk!(KwPrivate)
                        | tk!(KwProtected)
                        | tk!(KwReadonly)
                        | tk!(KwAbstract)
                        | tk!(KwOverride)
                        | tk!(KwDeclare)
                )
        ),
        _ => false,
    }
}
