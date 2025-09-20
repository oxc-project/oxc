mod global_functions;
mod math_functions;
mod string_methods;

use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::constant_evaluation::{ConstantEvaluationCtx, ConstantValue};

use global_functions::try_fold_global_functions;

pub fn try_fold_known_global_methods<'a>(
    callee: &Expression<'a>,
    arguments: &Vec<'a, Argument<'a>>,
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
                Expression::StringLiteral(s) => (s.value.as_str(), &member.object),
                _ => return None,
            }
        }
        _ => return None,
    };
    match name {
        "toLowerCase" | "toUpperCase" | "trim" | "trimStart" | "trimEnd" => {
            string_methods::try_fold_string_casing(arguments, name, object, ctx)
        }
        "substring" | "slice" => {
            string_methods::try_fold_string_substring_or_slice(arguments, object, ctx)
        }
        "indexOf" | "lastIndexOf" => {
            string_methods::try_fold_string_index_of(arguments, name, object, ctx)
        }
        "charAt" => string_methods::try_fold_string_char_at(arguments, object, ctx),
        "charCodeAt" => string_methods::try_fold_string_char_code_at(arguments, object, ctx),
        "startsWith" => string_methods::try_fold_starts_with(arguments, object, ctx),
        "replace" | "replaceAll" => {
            string_methods::try_fold_string_replace(arguments, name, object, ctx)
        }
        "fromCharCode" => string_methods::try_fold_string_from_char_code(arguments, object, ctx),
        "toString" => string_methods::try_fold_to_string(arguments, object, ctx),
        "isFinite" | "isNaN" | "isInteger" | "isSafeInteger" => {
            math_functions::try_fold_number_methods(arguments, object, name, ctx)
        }
        "sqrt" | "cbrt" => math_functions::try_fold_roots(arguments, name, object, ctx),
        "abs" | "ceil" | "floor" | "round" | "fround" | "trunc" | "sign" | "clz32" => {
            math_functions::try_fold_math_unary(arguments, name, object, ctx)
        }
        "imul" | "min" | "max" => {
            math_functions::try_fold_math_variadic(arguments, name, object, ctx)
        }
        _ => None,
    }
}
