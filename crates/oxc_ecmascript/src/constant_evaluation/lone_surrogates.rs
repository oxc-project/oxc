//! Detection helpers for bailing out of folds that would drop the `lone_surrogates` flag.
//!
//! `oxc_ast` stores strings with unpaired UTF-16 surrogates in a special encoding: each surrogate
//! code point is spelled `\u{FFFD}XXXX` (U+FFFD followed by four lowercase hex digits), with
//! `\u{FFFD}fffd` reserved as the self-escape for a real U+FFFD. The `StringLiteral` /
//! `TemplateElement` `lone_surrogates` flag tells codegen to decode those escapes back into
//! `\uXXXX`; when the flag is clear, codegen emits the bytes as-is.
//!
//! The encoding is produced by `oxc_parser::lexer::unicode` (string literals) and
//! `oxc_parser::lexer::template` (template elements) at the point they first see a lone
//! surrogate; both write `\u{FFFD}{code_point:04x}` and set `lone_surrogates: true` on the token.
//!
//! Two strings with the same bytes but different flags are different runtime values. Folds that
//! produce a new `ConstantValue::String` drop the flag, and `value_to_expr` then builds a literal
//! defaulting to `lone_surrogates: false` — so any fold consuming a `lone_surrogates: true` input
//! would silently corrupt the value. The helpers here let callers detect that and bail.
//!
//! Detection is conservative: false positives only skip a fold that could have been performed,
//! never produce wrong output.
//!
//! Invariant (load-bearing for the `_ => false` arm of [`expr_may_have_lone_surrogates`]): every
//! fold that produces a `ConstantValue::String` from possibly-flagged bytes either rewrites the
//! result into a `StringLiteral` / `TemplateLiteral` in place (which the typed arms above catch)
//! or bails itself. Under bottom-up evaluation, a parent that inspects a `CallExpression` /
//! `NewExpression` / `TaggedTemplateExpression` subexpression therefore sees either an
//! already-rewritten literal or a still-unfolded call — never a raw flagless string value. When
//! adding a new string-producing fold, either add a typed arm here for the kind it produces, or
//! gate the fold site with [`expr_may_have_lone_surrogates`] before emitting a literal.

use std::borrow::Cow;

use oxc_ast::ast::*;
use oxc_syntax::operator::BinaryOperator;

use crate::{GlobalContext, ToJsString, side_effects::MayHaveSideEffects};

use super::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue};

/// Returns `true` if `s` contains the lone-surrogate encoding pattern `\u{FFFD}XXXX` — surrogate
/// range `d800`..=`dfff`, or the self-escape `fffd`.
///
/// Scans raw bytes without consulting any AST flag, so a genuine U+FFFD followed by four matching
/// hex characters also matches. That false positive only skips a fold.
pub fn str_has_lone_surrogate_encoding(s: &str) -> bool {
    let bytes = s.as_bytes();
    // U+FFFD is `EF BF BD` in UTF-8; short-circuit when absent (the common case).
    if !bytes.contains(&0xEF) {
        return false;
    }
    // 3 bytes for U+FFFD + 4 bytes for the hex suffix.
    bytes.windows(7).any(|w| w[..3] == [0xEF, 0xBF, 0xBD] && is_lone_surrogate_suffix(&w[3..]))
}

/// Returns the runtime UTF-16 length of a flagged string.
///
/// A single byte-level walk: each `\u{FFFD}XXXX` encoded run (7 bytes / 5 stored UTF-16 code
/// units) contributes 1 runtime code unit — either a lone surrogate, or U+FFFD itself via the
/// `fffd` self-escape — and any other codepoint contributes its normal UTF-16 length. Fuses
/// what used to be a two-pass `encode_utf16().count() - 4 * run_count`.
///
/// Only meaningful when the caller has already established `lone_surrogates: true`: the scan
/// can't distinguish an encoded run from a coincidentally-matching U+FFFD followed by four hex
/// characters, so on an unflagged string this would under-count.
pub fn flagged_str_runtime_utf16_length(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut len = 0;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        // Encoded run: U+FFFD (EF BF BD) + 4 ASCII hex digits = 7 bytes, 1 runtime UTF-16 unit.
        if b == 0xEF
            && i + 7 <= bytes.len()
            && bytes[i + 1] == 0xBF
            && bytes[i + 2] == 0xBD
            && is_lone_surrogate_suffix(&bytes[i + 3..i + 7])
        {
            len += 1;
            i += 7;
            continue;
        }
        // Otherwise advance by one UTF-8 codepoint and add its UTF-16 length. `&str` guarantees
        // valid UTF-8, so `b` is always a lead byte here.
        let (step, units) = match b {
            0x00..=0x7F => (1, 1), // ASCII
            0xC2..=0xDF => (2, 1), // 2-byte sequence (U+0080..=U+07FF)
            0xE0..=0xEF => (3, 1), // 3-byte sequence (U+0800..=U+FFFF, BMP)
            0xF0..=0xF4 => (4, 2), // 4-byte sequence (supplementary → UTF-16 surrogate pair)
            _ => unreachable!("invalid UTF-8 lead byte {b:#x}"),
        };
        len += units;
        i += step;
    }
    len
}

fn is_lone_surrogate_suffix(b: &[u8]) -> bool {
    debug_assert_eq!(b.len(), 4);
    // Surrogate range d800–dfff, lowercase hex.
    (b[0] == b'd'
        && matches!(b[1], b'8'..=b'9' | b'a'..=b'f')
        && matches!(b[2], b'0'..=b'9' | b'a'..=b'f')
        && matches!(b[3], b'0'..=b'9' | b'a'..=b'f'))
        // Self-escape for a real U+FFFD inside a lone-surrogate string.
        || b == b"fffd"
}

/// Returns `true` if any quasi or interpolation in `t` may carry the lone-surrogate encoding.
///
/// Split out from [`expr_may_have_lone_surrogates`]'s `TemplateLiteral` arm so sites that hold a
/// `&TemplateLiteral` directly can reuse the check.
pub fn template_may_have_lone_surrogates<'a>(
    t: &TemplateLiteral<'a>,
    ctx: &impl GlobalContext<'a>,
) -> bool {
    t.quasis.iter().any(|q| q.lone_surrogates)
        || t.expressions.iter().any(|e| expr_may_have_lone_surrogates(e, ctx))
}

/// Returns `true` if any element in `arr` may carry the lone-surrogate encoding.
///
/// `as_expression` skips `SpreadElement` and `Elision`. Sound for every current caller because
/// each one establishes, by its own precondition, that spreads can't reach a string-producing
/// fold:
///
/// - [`expr_may_have_lone_surrogates`]'s `ArrayExpression` arm reflects `ArrayJoin::array_join`,
///   which bails on any element whose `to_js_string` fails; `SpreadElement::to_js_string` always
///   fails, so an array with a spread never produces a `ConstantValue::String` — we can't
///   mislead a parent that wouldn't itself be misled.
/// - The minifier's `.split(',')` rewrite in `substitute_array_expression` gates on
///   `is_all_string` (every element is a `StringLiteral`), ruling out spreads before this check.
///
/// Elisions stringify to `""`, so skipping them is also sound. If a future caller doesn't
/// satisfy one of these preconditions, it has to justify spread-skip on its own terms.
///
/// Split out from [`expr_may_have_lone_surrogates`]'s `ArrayExpression` arm so sites that hold an
/// `&ArrayExpression` directly can reuse the check.
pub fn array_may_have_lone_surrogates<'a>(
    arr: &ArrayExpression<'a>,
    ctx: &impl GlobalContext<'a>,
) -> bool {
    arr.elements
        .iter()
        .any(|el| el.as_expression().is_some_and(|e| expr_may_have_lone_surrogates(e, ctx)))
}

/// Returns `true` if the expression, when stringified, may carry the lone-surrogate encoding.
///
/// Fold sites call this before consuming an operand's string value; when it returns `true`, the
/// caller must skip the fold or the result would be a new string literal with `lone_surrogates:
/// false`, silently corrupting the value. Conservatively over-approximates — false positives only
/// cost a missed fold.
///
/// Identifiers are resolved through `ctx.get_constant_value_for_reference_id` and the resulting
/// `ConstantValue::String` bytes are byte-scanned. That loses the AST flag but is sound for
/// bail-out: a lone-surrogate literal's bytes always contain the encoding, and a byte-identical
/// non-surrogate string only causes a missed fold. One source of such byte-identical constants
/// is a prior concat like `'�' + 'dc00'` — neither operand is flagged, the concat folds
/// legitimately, and the resulting `ConstantValue::String` bytes match the encoding pattern.
pub fn expr_may_have_lone_surrogates<'a>(
    expr: &Expression<'a>,
    ctx: &impl GlobalContext<'a>,
) -> bool {
    match expr {
        Expression::StringLiteral(s) => s.lone_surrogates,
        Expression::TemplateLiteral(t) => template_may_have_lone_surrogates(t, ctx),
        Expression::BinaryExpression(e) if e.operator == BinaryOperator::Addition => {
            expr_may_have_lone_surrogates(&e.left, ctx)
                || expr_may_have_lone_surrogates(&e.right, ctx)
        }
        Expression::ArrayExpression(arr) => array_may_have_lone_surrogates(arr, ctx),
        // The Identifier arm pays a `get_constant_value_for_reference_id` lookup; fold sites that
        // pair a gate check with `get_side_free_string_value` / `evaluate_value_to_string` on the
        // same identifier then pay a second one. `get_side_free_string_value_without_lone_surrogates`
        // collapses the two into one lookup for its callers. `try_fold_string_casing` inlines its
        // own resolution; the remaining standalone call sites in `equality_comparison`,
        // `is_less_than`, binary-`+` in `mod.rs`, the `a + 'b' + 'c'` reshape, and `String()` in
        // `substitute_alternate_syntax` still pay the double lookup — each uses a different
        // string-fetching method, so a single shared helper doesn't fit them all.
        Expression::Identifier(ident) => ident
            .reference_id
            .get()
            .and_then(|rid| ctx.get_constant_value_for_reference_id(rid))
            .is_some_and(|cv| match cv {
                ConstantValue::String(s) => str_has_lone_surrogate_encoding(&s),
                _ => false,
            }),
        Expression::LogicalExpression(e) => {
            expr_may_have_lone_surrogates(&e.left, ctx)
                || expr_may_have_lone_surrogates(&e.right, ctx)
        }
        Expression::ConditionalExpression(e) => {
            expr_may_have_lone_surrogates(&e.consequent, ctx)
                || expr_may_have_lone_surrogates(&e.alternate, ctx)
        }
        Expression::SequenceExpression(e) => {
            e.expressions.last().is_some_and(|e| expr_may_have_lone_surrogates(e, ctx))
        }
        Expression::ParenthesizedExpression(e) => expr_may_have_lone_surrogates(&e.expression, ctx),

        // All remaining kinds return `false`. Two groups, same result:
        //
        // (1) `Call` / `New` / `TaggedTemplate` *can* fold to a string (via
        //     `try_fold_known_global_methods` and the `.concat` path in `replace_known_methods`).
        //     Load-bearing invariant: every string-producing fold self-bails on flagged inputs, so
        //     under bottom-up evaluation a parent sees either an already-rewritten `StringLiteral`
        //     (caught by the first arm above) or a still-unfolded call whose `evaluate_value` will
        //     return `None` at the same bail — flagged bytes never reach the parent through this
        //     arm. When adding a new string-producing fold, either add a dedicated arm above for
        //     its `Expression` kind, or guard the fold site with `expr_may_have_lone_surrogates`
        //     before emitting a literal.
        //
        // (2) Kinds that can't surface a flagged string here at all: numeric/boolean/null/bigint
        //     /regexp literals; object/function/class expressions; `MetaProperty`, `Super`,
        //     `ThisExpression`, `PrivateInExpression`; non-Addition `BinaryExpression` (number/
        //     boolean/bigint); `UnaryExpression` (`typeof`/`void`/`!`/…, fixed ASCII or number);
        //     member/chain expressions (only fold `.length` → number); side-effecting kinds
        //     (`Assignment`, `Update`, `Await`, `Yield`, `Import`) gated out upstream by
        //     `may_have_side_effects`; JSX / TS / V8 intrinsics (not folded to strings).
        //
        // Listed exhaustively (rather than `_ => false`) so a future `Expression` variant yields a
        // compile error here, forcing an explicit decision.
        Expression::CallExpression(_)
        | Expression::NewExpression(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::MetaProperty(_)
        | Expression::Super(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::ChainExpression(_)
        | Expression::ClassExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ImportExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::ThisExpression(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::YieldExpression(_)
        | Expression::PrivateInExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_)
        | Expression::TSAsExpression(_)
        | Expression::TSSatisfiesExpression(_)
        | Expression::TSTypeAssertion(_)
        | Expression::TSNonNullExpression(_)
        | Expression::TSInstantiationExpression(_)
        | Expression::V8IntrinsicExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => false,
    }
}

/// Extract `expr`'s side-free string value, unless [`expr_may_have_lone_surrogates`] says the
/// bytes might carry the encoding.
///
/// The bail pattern — check the flag/byte-scan, then pull `get_side_free_string_value` — recurs
/// at every fold site that reads an operand's string and emits a new `StringLiteral` from the
/// bytes. Collapsing it into one call keeps the invariant ("consume only flag-safe bytes") in
/// one place and avoids drift between sites.
///
/// For a constant-bound `Identifier`, resolves the binding once rather than looking it up twice
/// (in `expr_may_have_lone_surrogates` and again in `get_side_free_string_value`). Globals like
/// `undefined`/`Infinity`/`NaN` — which `get_side_free_string_value` stringifies via
/// `ToJsString` without a constant lookup — go through the generic fall-through below.
pub fn get_side_free_string_value_without_lone_surrogates<'a>(
    expr: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<Cow<'a, str>> {
    if let Expression::Identifier(ident) = expr
        && let Some(rid) = ident.reference_id.get()
        && let Some(cv) = ctx.get_constant_value_for_reference_id(rid)
    {
        if expr.may_have_side_effects(ctx) {
            return None;
        }
        // `to_js_string` — not `into_string` — so non-`String` constants (`Number`, `Boolean`,
        // `BigInt`, `Null`, `Undefined`) stringify through `ToJsString` the same way the generic
        // fall-through would via `evaluate_value_to_string`. `into_string` would drop the fold.
        let s = cv.to_js_string(ctx)?;
        return (!str_has_lone_surrogate_encoding(&s)).then_some(s);
    }
    if expr_may_have_lone_surrogates(expr, ctx) {
        return None;
    }
    expr.get_side_free_string_value(ctx)
}

#[cfg(test)]
mod tests {
    use super::{flagged_str_runtime_utf16_length, str_has_lone_surrogate_encoding};

    #[test]
    fn empty_and_short_inputs() {
        assert!(!str_has_lone_surrogate_encoding(""));
        assert!(!str_has_lone_surrogate_encoding("abc"));
        // 6 bytes: one short of the 7-byte window.
        assert!(!str_has_lone_surrogate_encoding("\u{FFFD}dc0"));
    }

    #[test]
    fn no_u_fffd_short_circuits() {
        assert!(!str_has_lone_surrogate_encoding("plain ascii text"));
        assert!(!str_has_lone_surrogate_encoding(&"a".repeat(1024)));
    }

    #[test]
    fn surrogate_range_boundaries() {
        // Low and high surrogate boundaries match.
        assert!(str_has_lone_surrogate_encoding("\u{FFFD}d800"));
        assert!(str_has_lone_surrogate_encoding("\u{FFFD}dbff"));
        assert!(str_has_lone_surrogate_encoding("\u{FFFD}dc00"));
        assert!(str_has_lone_surrogate_encoding("\u{FFFD}dfff"));
        // Just outside the surrogate range.
        assert!(!str_has_lone_surrogate_encoding("\u{FFFD}d7ff"));
        assert!(!str_has_lone_surrogate_encoding("\u{FFFD}e000"));
    }

    #[test]
    fn self_escape_for_literal_u_fffd() {
        assert!(str_has_lone_surrogate_encoding("\u{FFFD}fffd"));
    }

    #[test]
    fn uppercase_hex_is_not_the_encoding() {
        // The encoding uses lowercase hex; `�D800` is real U+FFFD
        // followed by the ASCII text "D800".
        assert!(!str_has_lone_surrogate_encoding("\u{FFFD}D800"));
        assert!(!str_has_lone_surrogate_encoding("\u{FFFD}FFFD"));
    }

    #[test]
    fn non_hex_suffix_is_rejected() {
        // `�dz00` — "dz00" isn't a valid hex run, and isn't
        // "fffd", so no match.
        assert!(!str_has_lone_surrogate_encoding("\u{FFFD}dz00"));
        // `�d80g` — trailing non-hex.
        assert!(!str_has_lone_surrogate_encoding("\u{FFFD}d80g"));
    }

    #[test]
    fn matches_anywhere_in_string() {
        assert!(str_has_lone_surrogate_encoding("prefix\u{FFFD}d800suffix"));
        assert!(str_has_lone_surrogate_encoding("a\u{FFFD}dc00b\u{FFFD}dfffc"));
    }

    #[test]
    fn lone_u_fffd_alone_is_not_the_encoding() {
        assert!(!str_has_lone_surrogate_encoding("\u{FFFD}"));
        assert!(!str_has_lone_surrogate_encoding("hello \u{FFFD} world"));
    }

    #[test]
    fn runtime_length_empty_and_ascii() {
        assert_eq!(flagged_str_runtime_utf16_length(""), 0);
        assert_eq!(flagged_str_runtime_utf16_length("plain"), 5);
    }

    #[test]
    fn runtime_length_counts_encoded_runs_as_one() {
        // One encoded lone surrogate: 5 stored UTF-16 units → 1 runtime unit.
        assert_eq!(flagged_str_runtime_utf16_length("\u{FFFD}d800"), 1);
        // Two back-to-back runs.
        assert_eq!(flagged_str_runtime_utf16_length("\u{FFFD}d800\u{FFFD}dc00"), 2);
        // Self-escape counts as a run (one U+FFFD at runtime).
        assert_eq!(flagged_str_runtime_utf16_length("\u{FFFD}fffd\u{FFFD}d800"), 2);
    }

    #[test]
    fn runtime_length_mixes_encoded_and_plain_codepoints() {
        // ASCII around a run: 'a' + run + 'b' + run + 'c' = 5 runtime units.
        assert_eq!(flagged_str_runtime_utf16_length("a\u{FFFD}dc00b\u{FFFD}dfffc"), 5);
        // Non-matching U+FFFD followed by a matching run: 1 (plain U+FFFD) + 1 (run) = 2.
        assert_eq!(flagged_str_runtime_utf16_length("\u{FFFD}\u{FFFD}d800"), 2);
        // A non-hex suffix defeats the run match, so each of the 5 stored chars counts normally
        // (U+FFFD + 4 ASCII = 5 UTF-16 units).
        assert_eq!(flagged_str_runtime_utf16_length("\u{FFFD}d7ff"), 5);
        // Supplementary-plane codepoint (U+1F600 😀) encodes as a UTF-16 surrogate pair → 2 units.
        assert_eq!(flagged_str_runtime_utf16_length("a\u{1F600}b"), 4);
    }
}
