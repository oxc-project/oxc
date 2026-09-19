//! Splitting runs of angle brackets which open or close type argument lists.
//!
//! `coalesce` joins a `>` with the characters after it into operators like `>>`, `>>>` and `>=`.
//! That is wrong when the `>`s close type argument lists, as in `Foo<Bar<T>>`.
//! [`gt_run_split`] decides how many of the `>`s must stay separate tokens.
//! [`lt_run_split`] does the same for a `<<` which opens two lists, as in `Array<<T>(x: T) => T>`.
//!
//! The decision draws on the other files:
//! - [`bytes`] finds the `<` which a run of `>`s would close.
//! - [`type_list`] applies TypeScript's rules for type arguments in an expression.
//! - [`context`] says whether the `<` is in a type.
//!
//! [`bytes`]: super::bytes
//! [`type_list`]: super::type_list
//! [`context`]: super::context

use crate::token::tk;

use crate::pipeline::{
    bitmap::bm_next1,
    tables::{OP_KIND_BASE, Tables},
};

use crate::pipeline::disambiguate::common::{
    AngleMatch, angle_match_back, as_gated_type_ref, as_type_operand, bm_prev_sig, class_like_walk,
    ident_is, kind_at, lt_in_range, prop_name, word_is_any,
};

use super::{
    bytes::{gt_run_closes_type_args, lt_run_opens_type_args},
    context::{After, Ctx, ctx_after_token, ctx_for_lt, lt_head_is_operand},
    type_list::{Follow, gt_follower, type_list_legal},
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
pub unsafe fn gt_run_split(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    p: usize,
    run: usize,
) -> usize {
    let mut g = 1usize;
    while g < run && *src.add(p + g) == b'>' {
        g += 1;
    }
    let mut closes = g;
    let head = loop {
        if let Some(h) = gt_run_closes_type_args(src, st, opch, kind, p, closes) {
            if closes == g {
                break h;
            }
            if ctx_for_lt(t, src, st, opch, kind, n, h, 0) == Ctx::Type {
                return closes;
            }
        }
        if closes == 1 {
            return 0;
        }
        closes -= 1;
    };
    if g < run && *src.add(p + g) == b'=' {
        return if ctx_for_lt(t, src, st, opch, kind, n, head, 0) == Ctx::Type { g } else { 0 };
    }
    match gt_follower(src, n, p + g) {
        Follow::Split => {
            if type_list_legal(t, src, st, kind, head + 1, p) {
                if type_list_head_is_relational(t, src, st, opch, kind, n, head) {
                    return 0;
                }
                return g;
            }
        }
        Follow::Fuse => {
            return if lt_head_is_operand(t, src, st, opch, kind, n, head, 0) { g } else { 0 };
        }
        Follow::Ctx => {}
    }
    if ctx_for_lt(t, src, st, opch, kind, n, head, 0) == Ctx::Type { g } else { 0 }
}

unsafe fn type_list_head_is_relational(
    t: &Tables,
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    head: usize,
) -> bool {
    let hq = bm_prev_sig(st, kind, head);
    if hq < 0 {
        return false;
    }
    let hw = hq as usize;
    let hk = kind_at(kind, hw);
    if hk == tk!(Ident) && !prop_name(src, hw) && ident_is(src, hw, b"this") {
        let p = bm_prev_sig(st, kind, hw);
        if p < 0 {
            return false;
        }
        let pw = p as usize;
        let pk = kind_at(kind, pw);
        if pk == tk!(Ident)
            && !prop_name(src, pw)
            && word_is_any(src, pw, &[b"extends", b"implements"])
        {
            return false;
        }
        if pk >= OP_KIND_BASE && *src.add(pw) == b',' && class_like_walk(src, st, kind, pw) {
            return false;
        }
        return ctx_after_token(t, src, st, opch, kind, n, p, After::Head, 0) == Ctx::Type;
    }
    if hk >= OP_KIND_BASE
        && *src.add(hw) == b'>'
        && !(hw > 0 && *src.add(hw - 1) == b'=')
        && let AngleMatch::Found(lt2) = angle_match_back(src, st, kind, hw)
    {
        return as_gated_type_ref(t, src, st, kind, n, lt2);
    }
    hk == tk!(Ident)
        && !prop_name(src, hw)
        && as_type_operand(src, st, kind, hw)
        && lt_in_range(src, bm_next1(st, hw + 1, n), head)
}

/// `coalesce` entry for a `<<` run: true when the two `<` must stay separate
/// tokens. Cold - `<<` is shift-left everywhere except this one shape.
#[inline(never)]
pub unsafe fn lt_run_split(
    src: *const u8,
    st: *const u64,
    opch: *const u64,
    kind: *const u8,
    n: usize,
    p: usize,
) -> bool {
    lt_run_opens_type_args(src, st, opch, kind, n, p)
}
