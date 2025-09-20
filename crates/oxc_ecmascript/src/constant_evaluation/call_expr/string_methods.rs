use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_syntax::number::ToJsString;

use crate::{
    StringCharAt, StringCharAtResult, StringCharCodeAt, StringIndexOf, StringLastIndexOf,
    StringSubstring, ToInt32, ToJsString as ToJsStringTrait,
    side_effects::MayHaveSideEffects,
};

use crate::constant_evaluation::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue};

pub fn try_fold_string_casing<'a>(
    args: &Vec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !args.is_empty() {
        return None;
    }

    let value = match object {
        Expression::StringLiteral(s) => Cow::Borrowed(s.value.as_str()),
        Expression::Identifier(ident) => ident
            .reference_id
            .get()
            .and_then(|reference_id| ctx.get_constant_value_for_reference_id(reference_id))
            .and_then(ConstantValue::into_string)?,
        _ => return None,
    };

    let result = match name {
        "toLowerCase" => ctx.ast().atom(&value.cow_to_lowercase()),
        "toUpperCase" => ctx.ast().atom(&value.cow_to_uppercase()),
        "trim" => ctx.ast().atom(value.trim()),
        "trimStart" => ctx.ast().atom(value.trim_start()),
        "trimEnd" => ctx.ast().atom(value.trim_end()),
        _ => return None,
    };
    Some(ConstantValue::String(Cow::Borrowed(result.as_str())))
}

pub fn try_fold_string_index_of<'a>(
    args: &Vec<'a, Argument<'a>>,
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
        "indexOf" => s.value.as_str().index_of(search_value.as_deref(), search_start_index),
        "lastIndexOf" => {
            s.value.as_str().last_index_of(search_value.as_deref(), search_start_index)
        }
        _ => unreachable!(),
    };
    Some(ConstantValue::Number(result as f64))
}

pub fn try_fold_string_substring_or_slice<'a>(
    args: &Vec<'a, Argument<'a>>,
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
    if start_idx.is_some_and(|start| start > s.value.len() as f64 || start < 0.0)
        || end_idx.is_some_and(|end| end > s.value.len() as f64 || end < 0.0)
    {
        return None;
    }
    if let (Some(start), Some(end)) = (start_idx, end_idx) {
        if start > end {
            return None;
        }
    }

    Some(ConstantValue::String(Cow::Owned(s.value.as_str().substring(start_idx, end_idx))))
}

pub fn try_fold_string_char_at<'a>(
    args: &Vec<'a, Argument<'a>>,
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

    let result = match s.value.as_str().char_at(char_at_index) {
        StringCharAtResult::Value(c) => c.to_string(),
        StringCharAtResult::InvalidChar(_) => return None,
        StringCharAtResult::OutOfRange => String::new(),
    };
    Some(ConstantValue::String(Cow::Owned(result)))
}

pub fn try_fold_string_char_code_at<'a>(
    args: &Vec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    let Expression::StringLiteral(s) = object else { return None };
    let char_at_index = match args.first() {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };

    let value = s.value.as_str().char_code_at(char_at_index).map_or(f64::NAN, |n| n as f64);
    Some(ConstantValue::Number(value))
}

pub fn try_fold_starts_with<'a>(
    args: &Vec<'a, Argument<'a>>,
    object: &Expression<'a>,
    _ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() != 1 {
        return None;
    }
    let Argument::StringLiteral(arg) = args.first().unwrap() else { return None };
    let Expression::StringLiteral(s) = object else { return None };
    Some(ConstantValue::Boolean(s.value.starts_with(arg.value.as_str())))
}

pub fn try_fold_string_replace<'a>(
    args: &Vec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() != 2 {
        return None;
    }
    let Expression::StringLiteral(s) = object else { return None };
    let search_value = args.first().unwrap();
    let search_value = match search_value {
        Argument::SpreadElement(_) => return None,
        match_expression!(Argument) => {
            let value = search_value.to_expression();
            if value.may_have_side_effects(ctx) {
                return None;
            }
            value.evaluate_value(ctx)?.into_string()?
        }
    };
    let replace_value = args.get(1).unwrap();
    let replace_value = match replace_value {
        Argument::SpreadElement(_) => return None,
        match_expression!(Argument) => {
            replace_value.to_expression().get_side_free_string_value(ctx)?
        }
    };
    if replace_value.contains('$') {
        return None;
    }
    let result = match name {
        "replace" => s.value.as_str().cow_replacen(search_value.as_ref(), &replace_value, 1),
        "replaceAll" => s.value.as_str().cow_replace(search_value.as_ref(), &replace_value),
        _ => unreachable!(),
    };
    Some(ConstantValue::String(result))
}

pub fn try_fold_string_from_char_code<'a>(
    args: &Vec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !ctx.is_global_expr("String", object) {
        return None;
    }
    let mut s = String::with_capacity(args.len());
    for arg in args {
        let expr = arg.as_expression()?;
        let v = expr.get_side_free_number_value(ctx)?;
        let v = v.to_int_32() as u16 as u32;
        let c = char::try_from(v).ok()?;
        s.push(c);
    }
    Some(ConstantValue::String(Cow::Owned(s)))
}

pub fn try_fold_to_string<'a>(
    args: &Vec<'a, Argument<'a>>,
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
            if let Some(Argument::NumericLiteral(n)) = args.first() {
                if n.value >= 2.0 && n.value <= 36.0 && n.value.fract() == 0.0 {
                    radix = n.value as u32;
                }
            }
            if radix == 0 {
                return None;
            }
            if radix == 10 {
                let s = lit.value.to_js_string();
                return Some(ConstantValue::String(Cow::Owned(s)));
            }
            // Only convert integers for other radix values.
            let value = lit.value;
            if value.is_infinite() {
                let s = if value.is_sign_negative() { "-Infinity" } else { "Infinity" };
                return Some(ConstantValue::String(Cow::Borrowed(s)));
            }
            if value.is_nan() {
                return Some(ConstantValue::String(Cow::Borrowed("NaN")));
            }
            if value >= 0.0 && value.fract() != 0.0 {
                return None;
            }
            let i = value as u32;
            if i as f64 != value {
                return None;
            }
            let result = format_radix(i, radix);
            Some(ConstantValue::String(Cow::Owned(result)))
        }
        Expression::RegExpLiteral(lit) if args.is_empty() => {
            lit.to_js_string(ctx).map(ConstantValue::String)
        }
        e if args.is_empty() => e
            .evaluate_value(ctx)
            // `null` and `undefined` returns type errors
            .filter(|v| !v.is_undefined() && !v.is_null())
            .and_then(|v| v.to_js_string(ctx).map(ConstantValue::String)),
        _ => None,
    }
}

pub fn format_radix(mut x: u32, radix: u32) -> String {
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