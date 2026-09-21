//! Splitting runs of angle brackets which open or close type argument lists.
//!
//! `coalesce` joins a `>` with the characters after it into operators like `>>`, `>>>` and `>=`.
//! That is wrong when the `>`s close type argument lists, as in `Foo<Bar<T>>`.
//! [`gt_run_split`] decides how many of the `>`s must stay separate tokens.
//! [`lt_run_split`] does the same for a `<<` which opens two lists, as in `Array<<T>(x: T) => T>`.
//!
//! A `>` run is settled by the tokens around it ([`run_shortcut`]) when the run closes no `<` on
//! its level, or when the list it closes opens in every context ([`list_in_any_context`]): after a
//! plain name TypeScript's speculative parse accepts ([`type_list`]), after a keyword only a type
//! follows, at a return type, or at `<T extends`. Otherwise the forward context walk counts the
//! lists open at the run ([`context`]).
//!
//! [`type_list`]: super::type_list
//! [`context`]: crate::pipeline::disambiguate::context

use crate::token::{OP_KIND_BASE, tk};

use super::{bytes::lt_run_opens_type_args, type_list::type_args_at};
use crate::pipeline::disambiguate::{
    common::{Prev, Tokens},
    context::{self, Walks},
};

/// `coalesce` entry for a `>` run (`>>`, `>>>`, and a `>` glued to `=`): how many leading `>` bytes
/// to leave unfused (each closes an open `<` list), or 0 to fuse.
#[inline(never)]
pub fn gt_run_split(tokens: &Tokens, walks: &mut Walks, p: usize, run: usize) -> usize {
    let mut g = 0usize;
    while g < run && tokens.src[p + g] == b'>' {
        g += 1;
    }
    run_shortcut(tokens, p, g).unwrap_or_else(|| context::angles_before(tokens, walks, p)).min(g)
}

/// `coalesce` entry for a `<<` run: true when the two `<` must stay separate
/// tokens. Cold - `<<` is shift-left everywhere except this one shape.
#[inline(never)]
pub fn lt_run_split(tokens: &Tokens, p: usize) -> bool {
    lt_run_opens_type_args(tokens.src, tokens.st, tokens.opch, tokens.kind, tokens.n, p)
}

/// Step cap for the short scan deciding a `>` run without context.
const RUN_SCAN_CAP: u32 = 256;

/// The lists a `>` run of `run` bytes at `pos` closes, when its context cannot matter: the run
/// closes no list on its level, or the list that would take the whole run opens in every context
/// ([`list_in_any_context`]). None when the context matters (the walk decides).
fn run_shortcut(tokens: &Tokens, pos: usize, run: usize) -> Option<usize> {
    let mut pending = 0i32;
    let mut found = 0usize;
    let mut lt = 0usize;
    let mut bounded = false;
    let mut steps = 0u32;
    let mut q = tokens.prev_sig(pos);
    while let Some(p) = q {
        steps += 1;
        if steps > RUN_SCAN_CAP {
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

/// Does the `<` at `lt` open a list whatever the context? Three shapes leave no expression
/// reading:
///
/// - `<T extends X`: nothing continues `T extends` in an expression (in JSX, `<T extends>`,
///   `<T extends=` and `<T extends/>` are tags).
/// - A list after a plain name that TypeScript's expression speculation accepts: in a type, a `<`
///   after a name always opens a list.
/// - A list after a name that must be a type reference ([`type_reference_head`]) on the same
///   line (a type reference takes no arguments across a line break), or after `function` or
///   `class`.
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

/// Is the name at `head` (the last part of a dotted name) a type reference by the tokens before
/// it: a keyword only a type follows on the same line, or the `:` of a return type?
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
        // `class`, `extends` and `function` are reserved words, so a name on the next line is
        // still theirs; the others can be identifiers that a line break ends (`x = as\nA<B>>c`).
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

/// Is the `:` at `c` the start of a return type: `function (...): ` or `name(...): ` where the
/// token before `name` leaves no room for a call? `case (x): ` is the one keyword before `(` an
/// expression follows.
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
