#![expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::float_cmp
)]

use std::borrow::Cow;

use cow_utils::CowUtils;
use smallvec::SmallVec;

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_str::{JSStr, JSStrBuilder};
use oxc_syntax::number::ToJsString;

use crate::{
    StringCharCodeAt, StringIndexOf, StringLastIndexOf, ToInt32, ToJsString as ToJsStringTrait,
    ToUint32,
    constant_evaluation::url_encoding::{
        decode_uri_chars, encode_uri_chars, is_uri_always_unescaped,
    },
    side_effects::MayHaveSideEffects,
};

use super::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue, js_str_from_cow};

fn try_fold_global_functions<'a>(
    ident: &IdentifierReference<'a>,
    arguments: &ArenaVec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    match ident.name.as_str() {
        "encodeURI" if ctx.is_global_reference(ident) => try_fold_encode_uri(arguments, ctx),
        "encodeURIComponent" if ctx.is_global_reference(ident) => {
            try_fold_encode_uri_component(arguments, ctx)
        }
        "decodeURI" if ctx.is_global_reference(ident) => try_fold_decode_uri(arguments, ctx),
        "decodeURIComponent" if ctx.is_global_reference(ident) => {
            try_fold_decode_uri_component(arguments, ctx)
        }
        "isNaN" if ctx.is_global_reference(ident) => try_fold_global_is_nan(arguments, ctx),
        "isFinite" if ctx.is_global_reference(ident) => try_fold_global_is_finite(arguments, ctx),
        "parseFloat" if ctx.is_global_reference(ident) => {
            try_fold_global_parse_float(arguments, ctx)
        }
        "parseInt" if ctx.is_global_reference(ident) => try_fold_global_parse_int(arguments, ctx),
        _ => None,
    }
}

pub fn try_fold_known_global_methods<'a>(
    callee: &Expression<'a>,
    arguments: &ArenaVec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if let Expression::Identifier(ident) = callee {
        if let Some(result) = try_fold_global_functions(ident, arguments, ctx) {
            return Some(result);
        }
        return None;
    }

    let (name, object) = match callee {
        Expression::StaticMemberExpression(member) if !member.optional => {
            (member.property.name.as_str(), &member.object)
        }
        Expression::ComputedMemberExpression(member) if !member.optional => {
            match &member.expression {
                Expression::StringLiteral(s) => (s.value.as_str()?, &member.object),
                _ => return None,
            }
        }
        _ => return None,
    };
    match name {
        "toLowerCase" | "toUpperCase" | "trim" | "trimStart" | "trimEnd" => {
            try_fold_string_casing(arguments, name, object, ctx)
        }
        "substring" | "slice" => try_fold_string_substring_or_slice(arguments, object, ctx),
        "indexOf" | "lastIndexOf" => try_fold_string_index_of(arguments, name, object, ctx),
        "charAt" => try_fold_string_char_at(arguments, object, ctx),
        "charCodeAt" => try_fold_string_char_code_at(arguments, object, ctx),
        "startsWith" => try_fold_starts_with(arguments, object, ctx),
        "replace" | "replaceAll" => try_fold_string_replace(arguments, name, object, ctx),
        "fromCharCode" => try_fold_string_from_char_code(arguments, object, ctx),
        "toString" => try_fold_to_string(arguments, object, ctx),
        "isFinite" | "isNaN" | "isInteger" | "isSafeInteger" => {
            try_fold_number_methods(arguments, object, name, ctx)
        }
        "sqrt" | "cbrt" => try_fold_roots(arguments, name, object, ctx),
        "abs" | "ceil" | "floor" | "round" | "fround" | "trunc" | "sign" | "clz32" => {
            try_fold_math_unary(arguments, name, object, ctx)
        }
        "imul" | "min" | "max" => try_fold_math_variadic(arguments, name, object, ctx),
        _ => None,
    }
}

fn try_fold_string_casing<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !args.is_empty() {
        return None;
    }

    let value = match object {
        Expression::StringLiteral(s) => s.value,
        Expression::Identifier(ident) => ident
            .reference_id
            .get()
            .and_then(|reference_id| ctx.get_constant_value_for_reference_id(reference_id))
            .and_then(ConstantValue::into_string)?,
        _ => return None,
    };
    // Casing and trimming operate on UTF-8; a value containing a lone
    // surrogate stays unfolded.
    let value = value.as_str()?;

    let result = match name {
        "toLowerCase" => Str::from_str_in(&value.cow_to_lowercase(), ctx),
        "toUpperCase" => Str::from_str_in(&value.cow_to_uppercase(), ctx),
        "trim" => Str::from_str_in(value.trim(), ctx),
        "trimStart" => Str::from_str_in(value.trim_start(), ctx),
        "trimEnd" => Str::from_str_in(value.trim_end(), ctx),
        _ => return None,
    };
    Some(ConstantValue::String(result.into()))
}

fn try_fold_string_index_of<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() >= 3 {
        return None;
    }
    let Expression::StringLiteral(s) = object else { return None };
    let search_value = match args.first() {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_string_value(ctx)?)
        }
        None => None,
    };
    let search_start_index = match args.get(1) {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };

    let result = match name {
        "indexOf" => s.value.index_of(search_value, search_start_index),
        "lastIndexOf" => s.value.last_index_of(search_value, search_start_index),
        _ => unreachable!(),
    };
    Some(ConstantValue::Number(result as f64))
}

fn try_fold_string_substring_or_slice<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() > 2 {
        return None;
    }
    let Expression::StringLiteral(s) = object else { return None };
    let start_idx = match args.first() {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };
    let end_idx = match args.get(1) {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };
    // A NaN end cannot be folded: the number alone no longer says whether the
    // argument was `undefined` (end of string) or NaN (index zero), and for
    // `slice` a zero end must not swap with the start. A NaN start folds the
    // same as zero through the clamp below either way.
    if end_idx.is_some_and(f64::is_nan) {
        return None;
    }
    if start_idx.is_some_and(|start| start > s.value.len() as f64 || start < 0.0)
        || end_idx.is_some_and(|end| end > s.value.len() as f64 || end < 0.0)
    {
        return None;
    }
    if let (Some(start), Some(end)) = (start_idx, end_idx)
        && start > end
    {
        return None;
    }
    let value = s.value;
    // The guards above leave ordered, in-range, non-NaN positions.
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let to_unit = |position: Option<f64>, default: usize| {
        position.map_or(default, |p| if p.is_nan() { 0 } else { p.trunc().max(0.0) as usize })
    };
    let start = to_unit(start_idx, 0);
    let end = to_unit(end_idx, usize::MAX).max(start);
    if let Some(value) = value.as_str() {
        // Resolve both UTF-16 positions to byte offsets in one pass; a
        // position past the end clamps to the end of the string. When both
        // land on character boundaries the result borrows the arena without
        // allocating; a boundary inside a surrogate pair drops to the code
        // unit path below, which splits the pair.
        let mut units = 0;
        let mut from = None;
        let mut to = None;
        let mut splits_pair = false;
        for (offset, c) in value.char_indices() {
            if units == start && from.is_none() {
                from = Some(offset);
            }
            if units == end {
                to = Some(offset);
                break;
            }
            units += c.len_utf16();
            if (from.is_none() && units > start) || units > end {
                splits_pair = true;
                break;
            }
        }
        if !splits_pair {
            let from = from.unwrap_or(value.len());
            let to = to.unwrap_or(value.len());
            return Some(ConstantValue::String(JSStr::from(&value[from..to])));
        }
    }
    // Collect the code units of the window. A window boundary may split a
    // surrogate pair; the lone half is representable, so the result is exact.
    let mut result = JSStrBuilder::new_in(ctx.allocator());
    for unit in value.encode_utf16().skip(start).take(end - start) {
        result.push_code_unit(unit);
    }
    Some(ConstantValue::String(result.into_js_str()))
}

fn try_fold_string_char_at<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() > 1 {
        return None;
    }
    let Expression::StringLiteral(s) = object else { return None };
    let char_at_index = match args.first() {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };

    // The result is the single code unit at the position, which may be one
    // half of a surrogate pair; a position out of range gives the empty
    // string.
    let result = match s.value.char_code_at(char_at_index) {
        Some(unit) => {
            let mut result = JSStrBuilder::with_capacity_in(3, ctx.allocator());
            result.push_code_unit(unit as u16);
            result.into_js_str()
        }
        None => JSStr::empty(),
    };
    Some(ConstantValue::String(result))
}

fn try_fold_string_char_code_at<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args
        .iter()
        .skip(1)
        .any(|arg| arg.as_expression().is_none_or(|e| e.may_have_side_effects(ctx)))
    {
        return None;
    }
    let Expression::StringLiteral(s) = object else { return None };
    let char_at_index = match args.first() {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };

    let value = s.value.char_code_at(char_at_index).map_or(f64::NAN, |n| n as f64);
    Some(ConstantValue::Number(value))
}

fn try_fold_starts_with<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    object: &Expression<'a>,
    _ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() != 1 {
        return None;
    }
    let Argument::StringLiteral(arg) = args.first().unwrap() else { return None };
    let Expression::StringLiteral(s) = object else { return None };
    // A UTF-8 search value prefix-matches over WTF-8 bytes even when the
    // receiver holds lone surrogates. A search value with a lone surrogate
    // could still match the same half of a formed pair, which the byte
    // comparison misses, so it stays unfolded.
    Some(ConstantValue::Boolean(s.value.starts_with(arg.value.as_str()?)))
}

fn try_fold_string_replace<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() != 2 {
        return None;
    }
    let Expression::StringLiteral(s) = object else { return None };
    // The replace machinery is UTF-8; a receiver, search value, or
    // replacement containing a lone surrogate stays unfolded.
    let value = s.value.as_str()?;
    let search_value = args.first().unwrap();
    let search_value = match search_value {
        Argument::SpreadElement(_) => return None,
        match_expression!(Argument) => {
            let value = search_value.to_expression();
            if value.may_have_side_effects(ctx) {
                return None;
            }
            value.evaluate_value(ctx)?.into_string()?.as_str()?
        }
    };
    let replace_value = args.get(1).unwrap();
    let replace_value = match replace_value {
        Argument::SpreadElement(_) => return None,
        match_expression!(Argument) => {
            replace_value.to_expression().get_side_free_string_value(ctx)?.as_str()?
        }
    };
    if replace_value.contains('$') {
        return None;
    }
    let result = match name {
        "replace" => value.cow_replacen(search_value, replace_value, 1),
        "replaceAll" => value.cow_replace(search_value, replace_value),
        _ => unreachable!(),
    };
    Some(ConstantValue::String(js_str_from_cow(result, ctx)))
}

fn try_fold_string_from_char_code<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !ctx.is_global_expr("String", object) {
        return None;
    }
    // Each argument contributes its ToUint16 code unit; like the runtime
    // string, a lead surrogate followed by a trail surrogate forms the
    // supplementary character, and any other surrogate stays a lone unit.
    let mut result = JSStrBuilder::with_capacity_in(args.len(), ctx.allocator());
    for arg in args {
        let expr = arg.as_expression()?;
        let v = expr.get_side_free_number_value(ctx)?;
        result.push_code_unit(v.to_int_32() as u16);
    }
    Some(ConstantValue::String(result.into_js_str()))
}

fn try_fold_to_string<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    match object {
        // Number.prototype.toString()
        // Number.prototype.toString(radix)
        Expression::NumericLiteral(lit) if args.len() <= 1 => {
            let mut radix: u32 = 0;
            if args.is_empty() {
                radix = 10;
            }
            if let Some(Argument::NumericLiteral(n)) = args.first()
                && n.value >= 2.0
                && n.value <= 36.0
                && n.value.fract() == 0.0
            {
                radix = n.value as u32;
            }
            if radix == 0 {
                return None;
            }
            if radix == 10 {
                let s = lit.value.to_js_string();
                return Some(ConstantValue::String(JSStr::from_str_in(&s, ctx)));
            }
            // Only convert integers for other radix values.
            let value = lit.value;
            if value.is_infinite() {
                let s = if value.is_sign_negative() { "-Infinity" } else { "Infinity" };
                return Some(ConstantValue::String(JSStr::from(s)));
            }
            if value.is_nan() {
                return Some(ConstantValue::String(JSStr::from("NaN")));
            }
            if value >= 0.0 && value.fract() != 0.0 {
                return None;
            }
            let i = value as u32;
            if i as f64 != value {
                return None;
            }
            let result = format_radix(i, radix);
            Some(ConstantValue::String(JSStr::from_str_in(&result, ctx)))
        }
        Expression::RegExpLiteral(lit) if args.is_empty() => {
            lit.to_js_string(ctx).map(|s| ConstantValue::String(js_str_from_cow(s, ctx)))
        }
        e if args.is_empty() && !e.may_have_side_effects(ctx) => e
            .evaluate_value(ctx)
            // `null` and `undefined` returns type errors
            .filter(|v| !v.is_undefined() && !v.is_null())
            .and_then(|v| match v {
                ConstantValue::String(s) => Some(ConstantValue::String(s)),
                v => v.to_js_string(ctx).map(|s| ConstantValue::String(js_str_from_cow(s, ctx))),
            }),
        _ => None,
    }
}

fn format_radix(mut x: u32, radix: u32) -> String {
    debug_assert!((2..=36).contains(&radix));
    let mut result = vec![];
    loop {
        let m = x % radix;
        x /= radix;
        result.push(std::char::from_digit(m, radix).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}

fn validate_arguments(args: &ArenaVec<'_, Argument<'_>>, expected_len: usize) -> bool {
    (args.len() == expected_len) && args.iter().all(Argument::is_expression)
}

fn try_fold_number_methods<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    object: &Expression<'a>,
    name: &str,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !ctx.is_global_expr("Number", object) {
        return None;
    }
    if args.len() != 1 {
        return None;
    }
    let extracted_expr = args.first()?.as_expression()?;
    if !extracted_expr.is_number_literal() {
        return None;
    }
    let extracted = extracted_expr.get_side_free_number_value(ctx)?;
    let result = match name {
        "isFinite" => Some(extracted.is_finite()),
        "isInteger" => Some(extracted.fract().abs() < f64::EPSILON),
        "isNaN" => Some(extracted.is_nan()),
        "isSafeInteger" => {
            let integer = extracted.fract().abs() < f64::EPSILON;
            let safe = extracted.abs() <= 2f64.powi(53) - 1.0;
            Some(safe && integer)
        }
        _ => None,
    };
    result.map(ConstantValue::Boolean)
}

fn try_fold_roots<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !ctx.is_global_expr("Math", object) || !validate_arguments(args, 1) {
        return None;
    }
    let arg_val = args[0].to_expression().get_side_free_number_value(ctx)?;
    if arg_val == f64::INFINITY || arg_val.is_nan() || arg_val == 0.0 {
        return Some(ConstantValue::Number(arg_val));
    }
    if arg_val < 0.0 {
        return Some(ConstantValue::Number(f64::NAN));
    }
    let calculated_val = match name {
        "sqrt" => arg_val.sqrt(),
        "cbrt" => arg_val.cbrt(),
        _ => unreachable!(),
    };
    (calculated_val.fract() == 0.0).then_some(ConstantValue::Number(calculated_val))
}

fn try_fold_math_unary<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !ctx.is_global_expr("Math", object) || !validate_arguments(args, 1) {
        return None;
    }
    let arg_val = args[0].to_expression().get_side_free_number_value(ctx)?;
    let result = match name {
        "abs" => arg_val.abs(),
        "ceil" => arg_val.ceil(),
        "floor" => arg_val.floor(),
        "round" => {
            // We should be aware that the behavior in JavaScript and Rust towards `round` is different.
            // In Rust, when facing `.5`, it may follow `half-away-from-zero` instead of round to upper bound.
            // So we need to handle it manually.
            let frac_part = arg_val.fract();
            if frac_part.abs() == 0.5 {
                // We should ceil it.
                arg_val.ceil()
            } else {
                arg_val.round()
            }
        }
        "fround" if arg_val.fract() == 0f64 || arg_val.is_nan() || arg_val.is_infinite() => {
            f64::from(arg_val as f32)
        }
        "fround" => return None,
        "trunc" => arg_val.trunc(),
        "sign" if arg_val.to_bits() == 0f64.to_bits() => 0f64,
        "sign" if arg_val.to_bits() == (-0f64).to_bits() => -0f64,
        "sign" => arg_val.signum(),
        "clz32" => f64::from(arg_val.to_uint_32().leading_zeros()),
        _ => unreachable!(),
    };
    // These results are always shorter to return as a number, so we can just return them as NumericLiteral.
    Some(ConstantValue::Number(result))
}

fn try_fold_math_variadic<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !ctx.is_global_expr("Math", object) {
        return None;
    }
    let mut numbers = SmallVec::<[f64; 8]>::new();
    for arg in args {
        let expr = arg.as_expression()?;
        let value = expr.get_side_free_number_value(ctx)?;
        numbers.push(value);
    }
    let result = match name {
        "min" | "max" => {
            if numbers.iter().any(|n: &f64| n.is_nan()) {
                f64::NAN
            } else {
                match name {
                    // TODO
                    // see <https://github.com/rust-lang/rust/issues/83984>, we can't use `min` and `max` here due to inconsistency
                    "min" => numbers.iter().copied().fold(f64::INFINITY, |a, b| {
                        if a < b || ((a == 0f64) && (b == 0f64) && (a.to_bits() > b.to_bits())) {
                            a
                        } else {
                            b
                        }
                    }),
                    "max" => numbers.iter().copied().fold(f64::NEG_INFINITY, |a, b| {
                        if a > b || ((a == 0f64) && (b == 0f64) && (a.to_bits() < b.to_bits())) {
                            a
                        } else {
                            b
                        }
                    }),
                    _ => return None,
                }
            }
        }
        "imul" => {
            let a = numbers.first().copied().unwrap_or(f64::NAN).to_uint_32();
            let b = numbers.get(1).copied().unwrap_or(f64::NAN).to_uint_32();
            f64::from(a.wrapping_mul(b).cast_signed())
        }
        _ => return None,
    };
    Some(ConstantValue::Number(result))
}

fn try_fold_encode_uri<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::String(JSStr::from("undefined")));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first()?;
    let expr = arg.as_expression()?;
    let string_value = expr.get_side_free_string_value(ctx)?;
    // `encodeURI` throws a URIError on a lone surrogate at runtime, so such
    // an input must stay unfolded.
    let string_value = Cow::Borrowed(string_value.as_str()?);

    // SAFETY: should_encode only returns false for ascii chars
    let encoded = unsafe {
        encode_uri_chars(
            string_value,
            #[inline(always)]
            |c| match c {
                c if is_uri_always_unescaped(c) => false,
                b';' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b',' | b'#' => false,
                _ => true,
            },
        )
    };
    Some(ConstantValue::String(js_str_from_cow(encoded, ctx)))
}

fn try_fold_encode_uri_component<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::String(JSStr::from("undefined")));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first()?;
    let expr = arg.as_expression()?;
    let string_value = expr.get_side_free_string_value(ctx)?;
    // `encodeURIComponent` throws a URIError on a lone surrogate at runtime,
    // so such an input must stay unfolded.
    let string_value = Cow::Borrowed(string_value.as_str()?);

    // SAFETY: should_encode only returns false for ascii chars
    let encoded = unsafe {
        encode_uri_chars(
            string_value,
            #[inline(always)]
            |c| !is_uri_always_unescaped(c),
        )
    };
    Some(ConstantValue::String(js_str_from_cow(encoded, ctx)))
}

fn try_fold_decode_uri<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::String(JSStr::from("undefined")));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first()?;
    let expr = arg.as_expression()?;
    let string_value = expr.get_side_free_string_value(ctx)?;
    // The decode machinery is UTF-8; an input containing a lone surrogate
    // stays unfolded.
    let string_value = Cow::Borrowed(string_value.as_str()?);

    let decoded = decode_uri_chars(
        string_value,
        #[inline(always)]
        |c| matches!(c, b';' | b',' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b'#'),
    )?;
    Some(ConstantValue::String(js_str_from_cow(decoded, ctx)))
}

fn try_fold_decode_uri_component<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::String(JSStr::from("undefined")));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first()?;
    let expr = arg.as_expression()?;
    let string_value = expr.get_side_free_string_value(ctx)?;
    // The decode machinery is UTF-8; an input containing a lone surrogate
    // stays unfolded.
    let string_value = Cow::Borrowed(string_value.as_str()?);

    // decodeURIComponent decodes all percent-encoded sequences
    let decoded = decode_uri_chars(
        string_value,
        #[inline(always)]
        |_| false,
    )?;
    Some(ConstantValue::String(js_str_from_cow(decoded, ctx)))
}

fn try_fold_global_is_nan<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::Boolean(true));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first().unwrap();
    let expr = arg.as_expression()?;
    let num = expr.get_side_free_number_value(ctx)?;
    Some(ConstantValue::Boolean(num.is_nan()))
}

fn try_fold_global_is_finite<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::Boolean(false));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first().unwrap();
    let expr = arg.as_expression()?;
    let num = expr.get_side_free_number_value(ctx)?;
    Some(ConstantValue::Boolean(num.is_finite()))
}

fn try_fold_global_parse_float<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::Number(f64::NAN));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first().unwrap();
    let expr = arg.as_expression()?;
    let input_string = expr.get_side_free_string_value(ctx)?;
    // The prefix scan is UTF-8; an input containing a lone surrogate stays
    // unfolded.
    let input_string = input_string.as_str()?;
    let trimmed = input_string.trim_start();
    let Some(trimmed_prefix) = find_str_decimal_literal_prefix(trimmed) else {
        return Some(ConstantValue::Number(f64::NAN));
    };

    let parsed = trimmed_prefix.cow_replace('_', "").parse::<f64>().unwrap_or_else(|_| {
        unreachable!(
            "StrDecimalLiteral should be parse-able with Rust FromStr for f64: {trimmed_prefix}"
        )
    });
    Some(ConstantValue::Number(parsed))
}

/// Find the longest prefix of a string that satisfies the syntax of a `StrDecimalLiteral`.
/// Returns None when not found.
///
/// This function implements step 4 of `parseFloat`.
/// <https://tc39.es/ecma262/2025/multipage/global-object.html#sec-parsefloat-string>
fn find_str_decimal_literal_prefix(input: &str) -> Option<&str> {
    fn match_decimal_digits(s: &str) -> Option<usize> {
        let bytes = s.as_bytes();
        if bytes.first().is_none_or(|b| !b.is_ascii_digit()) {
            // must have at least one digit
            return None;
        }
        let mut iter = bytes.iter().enumerate().skip(1);
        while let Some((i, &b)) = iter.next() {
            match b {
                b'0'..=b'9' => {}
                b'_' => {
                    let Some((i, &b)) = iter.next() else {
                        // must have at least one digit after _
                        return Some(i); // without _
                    };
                    if !b.is_ascii_digit() {
                        // must have at least one digit after _
                        return Some(i); // without _
                    }
                }
                _ => return Some(i),
            }
        }
        Some(s.len())
    }
    fn match_exponent_part(mut s: &str) -> Option<usize> {
        if !s.starts_with(['e', 'E']) {
            return None;
        }
        let mut last_index = 1;
        s = &s[1..];
        if s.starts_with(['+', '-']) {
            last_index += 1;
            s = &s[1..];
        }
        let end_of_decimal_digits = match_decimal_digits(s)?;
        last_index += end_of_decimal_digits;
        Some(last_index)
    }

    let mut s = input;
    let mut last_index: usize = 0;
    if s.starts_with(['+', '-']) {
        s = &s[1..];
        last_index += 1;
    }
    if s.starts_with("Infinity") {
        last_index += "Infinity".len();
        return Some(&input[..last_index]);
    }
    // . DecimalDigits ExponentPart
    if s.starts_with('.') {
        last_index += 1;
        s = &s[1..];
        let end_of_decimal_digits = match_decimal_digits(s)?;
        last_index += end_of_decimal_digits;
        s = &s[end_of_decimal_digits..];
        let Some(end_of_exponent_part) = match_exponent_part(s) else {
            return Some(&input[..last_index]);
        };
        last_index += end_of_exponent_part;
        return Some(&input[..last_index]);
    }

    let end_of_decimal_digits = match_decimal_digits(s)?;
    last_index += end_of_decimal_digits;
    s = &s[end_of_decimal_digits..];

    // DecimalDigits . DecimalDigits ExponentPart
    if s.starts_with('.') {
        last_index += 1;
        s = &s[1..];
        let Some(end_of_decimal_digits) = match_decimal_digits(s) else {
            return Some(&input[..last_index - 1]); // without .
        };
        last_index += end_of_decimal_digits;
        s = &s[end_of_decimal_digits..];
        let Some(end_of_exponent_part) = match_exponent_part(s) else {
            return Some(&input[..last_index]);
        };
        last_index += end_of_exponent_part;
        return Some(&input[..last_index]);
    }

    // DecimalDigits ExponentPart
    let Some(end_of_exponent_part) = match_exponent_part(s) else {
        return Some(&input[..last_index]);
    };
    last_index += end_of_exponent_part;
    Some(&input[..last_index])
}

fn try_fold_global_parse_int<'a>(
    args: &ArenaVec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::Number(f64::NAN));
    }
    if args.len() > 2
        || args
            .iter()
            .any(|arg| arg.as_expression().is_none_or(|arg| arg.may_have_side_effects(ctx)))
    {
        return None;
    }
    let string_arg = args.first().unwrap();
    let string_expr = string_arg.as_expression()?;
    let string_value = string_expr.evaluate_value_to_string(ctx)?;
    // The digit scan is UTF-8; an input containing a lone surrogate stays
    // unfolded.
    let string_value = string_value.as_str()?;
    let mut string_value = string_value.trim_start();

    let mut sign = 1;
    if string_value.starts_with('-') {
        sign = -1;
    }
    if string_value.starts_with(['+', '-']) {
        string_value = &string_value[1..];
    }

    let mut strip_prefix = true;
    let mut radix = if let Some(arg) = args.get(1) {
        let expr = arg.as_expression()?;
        let mut radix = expr.evaluate_value_to_number(ctx)?.to_int_32();
        if radix == 0 {
            radix = 10;
        } else if !(2..=36).contains(&radix) {
            return Some(ConstantValue::Number(f64::NAN));
        } else if radix != 16 {
            strip_prefix = false;
        }
        radix as u32
    } else {
        10
    };

    if !matches!(radix, 2 | 4 | 8 | 10 | 16 | 32) {
        // implementation can approximate the values. bail out to be safe
        return None;
    }

    if strip_prefix && (string_value.starts_with("0x") || string_value.starts_with("0X")) {
        string_value = &string_value[2..];
        radix = 16;
    }

    if let Some(non_radix_digit_pos) = string_value.chars().position(|c| !c.is_digit(radix)) {
        string_value = &string_value[..non_radix_digit_pos];
    }

    if string_value.is_empty() {
        return Some(ConstantValue::Number(f64::NAN));
    }

    if radix == 10 && string_value.len() > 20 {
        // implementation can approximate the values. bail out to be safe
        return None;
    }

    let Ok(math_int) = i32::from_str_radix(string_value, radix) else {
        // ignore values that cannot be represented as i32 to avoid precision issues
        return None;
    };
    if math_int == 0 {
        return Some(ConstantValue::Number(if sign == -1 { -0.0 } else { 0.0 }));
    }
    Some(ConstantValue::Number((math_int as f64) * sign as f64))
}

#[test]
fn test_find_str_decimal_literal_prefix() {
    assert_eq!(find_str_decimal_literal_prefix("Infinitya"), Some("Infinity"));
    assert_eq!(find_str_decimal_literal_prefix("+Infinitya"), Some("+Infinity"));
    assert_eq!(find_str_decimal_literal_prefix("-Infinitya"), Some("-Infinity"));
    assert_eq!(find_str_decimal_literal_prefix("0a"), Some("0"));
    assert_eq!(find_str_decimal_literal_prefix("+0a"), Some("+0"));
    assert_eq!(find_str_decimal_literal_prefix("-0a"), Some("-0"));
    assert_eq!(find_str_decimal_literal_prefix("0."), Some("0"));
    assert_eq!(find_str_decimal_literal_prefix("0.e"), Some("0"));
    assert_eq!(find_str_decimal_literal_prefix("0.e1"), Some("0"));
    assert_eq!(find_str_decimal_literal_prefix("0.1"), Some("0.1"));
    assert_eq!(find_str_decimal_literal_prefix("0.1."), Some("0.1"));
    assert_eq!(find_str_decimal_literal_prefix("0.1e"), Some("0.1"));
    assert_eq!(find_str_decimal_literal_prefix("0.1e1"), Some("0.1e1"));
    assert_eq!(find_str_decimal_literal_prefix(".1"), Some(".1"));
    assert_eq!(find_str_decimal_literal_prefix(".1."), Some(".1"));
    assert_eq!(find_str_decimal_literal_prefix(".1e"), Some(".1"));
    assert_eq!(find_str_decimal_literal_prefix(".1e1"), Some(".1e1"));
    assert_eq!(find_str_decimal_literal_prefix("1_"), Some("1"));
    assert_eq!(find_str_decimal_literal_prefix("1_1"), Some("1_1"));
    assert_eq!(find_str_decimal_literal_prefix("1_1_"), Some("1_1"));
}
