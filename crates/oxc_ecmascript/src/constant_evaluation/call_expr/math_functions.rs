#![expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::ToUint32;

use crate::constant_evaluation::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue};

fn validate_arguments(args: &Vec<'_, Argument<'_>>, expected_len: usize) -> bool {
    (args.len() == expected_len) && args.iter().all(Argument::is_expression)
}

pub fn try_fold_number_methods<'a>(
    args: &Vec<'a, Argument<'a>>,
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

pub fn try_fold_roots<'a>(
    args: &Vec<'a, Argument<'a>>,
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

pub fn try_fold_math_unary<'a>(
    args: &Vec<'a, Argument<'a>>,
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
            let epsilon = 2f64.powi(-52);
            if (frac_part.abs() - 0.5).abs() < epsilon {
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

pub fn try_fold_math_variadic<'a>(
    args: &Vec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !ctx.is_global_expr("Math", object) {
        return None;
    }
    let mut numbers = std::vec::Vec::new();
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
                    // NOTE: See <https://github.com/rust-lang/rust/issues/83984>
                    // We can't use `f64::min` and `f64::max` here due to inconsistency with JavaScript semantics
                    // for -0.0 vs +0.0 handling. Custom implementation ensures JavaScript-compliant behavior.
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