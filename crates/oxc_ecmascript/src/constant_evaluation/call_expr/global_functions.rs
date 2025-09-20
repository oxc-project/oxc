use std::borrow::Cow;

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    StringToNumber, ToInt32, ToNumber,
    constant_evaluation::url_encoding::{decode_uri_chars, encode_uri_chars, is_uri_always_unescaped},
};

use crate::constant_evaluation::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue};

pub fn try_fold_global_functions<'a>(
    ident: &IdentifierReference<'a>,
    arguments: &Vec<'a, Argument<'a>>,
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

fn try_fold_encode_uri<'a>(
    args: &Vec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::String(Cow::Borrowed("undefined")));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first()?;
    let expr = arg.as_expression()?;
    let string_value = expr.get_side_free_string_value(ctx)?;

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
    Some(ConstantValue::String(encoded))
}

fn try_fold_encode_uri_component<'a>(
    args: &Vec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::String(Cow::Borrowed("undefined")));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first()?;
    let expr = arg.as_expression()?;
    let string_value = expr.get_side_free_string_value(ctx)?;

    // SAFETY: should_encode only returns false for ascii chars
    let encoded = unsafe {
        encode_uri_chars(
            string_value,
            #[inline(always)]
            |c| !is_uri_always_unescaped(c),
        )
    };
    Some(ConstantValue::String(encoded))
}

fn try_fold_decode_uri<'a>(
    args: &Vec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::String(Cow::Borrowed("undefined")));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first()?;
    let expr = arg.as_expression()?;
    let string_value = expr.get_side_free_string_value(ctx)?;
    let decoded = decode_uri_chars(
        string_value,
        |c| match c {
            c if is_uri_always_unescaped(c) => false,
            b';' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b',' | b'#' => false,
            _ => true,
        },
    )?;
    Some(ConstantValue::String(decoded))
}

fn try_fold_decode_uri_component<'a>(
    args: &Vec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::String(Cow::Borrowed("undefined")));
    }
    if args.len() != 1 {
        return None;
    }
    let arg = args.first()?;
    let expr = arg.as_expression()?;
    let string_value = expr.get_side_free_string_value(ctx)?;
    let decoded = decode_uri_chars(
        string_value,
        |c| !is_uri_always_unescaped(c),
    )?;
    Some(ConstantValue::String(decoded))
}

fn try_fold_global_is_nan<'a>(
    args: &Vec<'a, Argument<'a>>,
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
    args: &Vec<'a, Argument<'a>>,
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
    args: &Vec<'a, Argument<'a>>,
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
    let string_value = expr.get_side_free_string_value(ctx)?;
    let value = string_value.as_ref().trim_start().string_to_number();
    Some(ConstantValue::Number(value))
}

fn try_fold_global_parse_int<'a>(
    args: &Vec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.is_empty() {
        return Some(ConstantValue::Number(f64::NAN));
    }
    if args.len() > 2 {
        return None;
    }
    let string_arg = args.first().unwrap();
    let string_expr = string_arg.as_expression()?;
    let string_value = string_expr.get_side_free_string_value(ctx)?;

    let radix = if args.len() > 1 {
        let radix_arg = args.get(1).unwrap();
        let radix_expr = radix_arg.as_expression()?;
        let radix_value = radix_expr.evaluate_value(ctx)?;
        match radix_value {
            ConstantValue::Undefined => 10,
            _ => {
                let r = radix_value.to_number(ctx)?.to_int_32();
                if r == 0 {
                    10
                } else if !(2..=36).contains(&r) {
                    return Some(ConstantValue::Number(f64::NAN));
                } else {
                    r
                }
            }
        }
    } else {
        10
    };

    // Parse as a BigInt since JS can represent arbitrarily large integers as BigInts.
    // For parsing, JS semantics are:
    // 1. Let inputString be ? ToString(string).
    // 2. Let S be ! TrimString(inputString, start).
    // 3. Let sign be 1.
    // 4. If S is not empty and the first code unit of S is the code unit 0x002D (HYPHEN-MINUS), set sign to -1.
    // 5. If S is not empty and the first code unit of S is either the code unit 0x002B (PLUS SIGN) or the code unit 0x002D (HYPHEN-MINUS), set S to the substring of S from index 1.
    // 6. Let R be ℝ(? ToInt32(radix)).
    // ...
    let string = string_value.as_ref().trim_start();
    let (string, is_negative) = if let Some(stripped) = string.strip_prefix('-') {
        (stripped, true)
    } else if let Some(stripped) = string.strip_prefix('+') {
        (stripped, false)
    } else {
        (string, false)
    };
    // 8. If R = 0, then
    //   a. If S is empty, return NaN.
    //   b. If S begins with "0x" or "0X", then
    //     i. Set S to the substring of S from index 2.
    //     ii. Set R to 16.
    // 9. Else if R < 2 or R > 36, return NaN.
    // 10. If S contains a code unit that is not a radix-R digit, let end be the index within S of the first such code unit; otherwise, let end be the length of S.
    // 11. Let Z be the substring of S from 0 to end.
    // 12. If Z is empty, return NaN.
    // 13. Let mathInt be the integer value that is represented by Z in radix-R notation, using the letters A to Z and a to z for digits with values 10 through 35. (However, if R = 10 and Z contains more than 20 significant digits, every significant digit after the 20th may be replaced by a 0 digit, at the option of the implementation; and if R is not one of 2, 4, 8, 10, 16, or 32, then mathInt may be an implementation-approximated integer representing the integer value denoted by Z in radix-R notation.)
    // 14. If mathInt = 0, then
    //   a. If sign = -1, return -0𝔽.
    //   b. Return +0𝔽.
    // 15. Return 𝔽(sign × mathInt).
    let radix_u32 = radix as u32;
    let parsed_int = if radix == 10 && string.strip_prefix("0x").is_some() {
        let string = &string[2..];
        let chars = string.chars().take_while(|c| c.is_ascii_hexdigit()).collect::<String>();
        if chars.is_empty() {
            return Some(ConstantValue::Number(f64::NAN));
        }
        BigInt::parse_bytes(chars.as_bytes(), 16)?
    } else if radix == 10 && string.strip_prefix("0X").is_some() {
        let string = &string[2..];
        let chars = string.chars().take_while(|c| c.is_ascii_hexdigit()).collect::<String>();
        if chars.is_empty() {
            return Some(ConstantValue::Number(f64::NAN));
        }
        BigInt::parse_bytes(chars.as_bytes(), 16)?
    } else {
        let chars = string
            .chars()
            .take_while(|c| c.is_digit(radix_u32) || c.is_ascii_alphabetic())
            .collect::<String>();
        if chars.is_empty() {
            return Some(ConstantValue::Number(f64::NAN));
        }
        BigInt::parse_bytes(chars.as_bytes(), radix_u32)?
    };
    let parsed_int = if is_negative { -parsed_int } else { parsed_int };
    let double = parsed_int.to_f64()?;
    Some(ConstantValue::Number(double))
}