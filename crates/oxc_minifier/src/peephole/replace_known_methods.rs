use cow_utils::CowUtils;
use std::borrow::Cow;

use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ValueType},
    StringCharAt, StringCharCodeAt, StringIndexOf, StringLastIndexOf, StringSubstring, ToInt32,
};
use oxc_span::SPAN;
use oxc_syntax::es_target::ESTarget;
use oxc_traverse::Ancestor;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

type Arguments<'a> = oxc_allocator::Vec<'a, Argument<'a>>;

impl<'a> PeepholeOptimizations {
    /// Minimize With Known Methods
    /// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeReplaceKnownMethods.java>
    pub fn replace_known_methods_exit_expression(
        &mut self,
        node: &mut Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) {
        self.try_fold_concat_chain(node, ctx);
        self.try_fold_known_global_methods(node, ctx);
        self.try_fold_known_property_access(node, ctx);
    }

    fn try_fold_known_global_methods(&mut self, node: &mut Expression<'a>, ctx: Ctx<'a, '_>) {
        let Expression::CallExpression(ce) = node else { return };
        let CallExpression { span, callee, arguments, .. } = ce.as_mut();
        let (name, object) = match &callee {
            Expression::StaticMemberExpression(member) if !member.optional => {
                (member.property.name.as_str(), &member.object)
            }
            Expression::ComputedMemberExpression(member) if !member.optional => {
                match &member.expression {
                    Expression::StringLiteral(s) => (s.value.as_str(), &member.object),
                    _ => return,
                }
            }
            _ => return,
        };
        let replacement = match name {
            "toLowerCase" | "toUpperCase" | "trim" => {
                Self::try_fold_string_casing(*span, arguments, name, object, ctx)
            }
            "substring" | "slice" => {
                Self::try_fold_string_substring_or_slice(*span, arguments, object, ctx)
            }
            "indexOf" | "lastIndexOf" => {
                Self::try_fold_string_index_of(*span, arguments, name, object, ctx)
            }
            "charAt" => Self::try_fold_string_char_at(*span, arguments, object, ctx),
            "charCodeAt" => Self::try_fold_string_char_code_at(*span, arguments, object, ctx),
            "concat" => Self::try_fold_concat(*span, arguments, callee, ctx),
            "replace" | "replaceAll" => {
                Self::try_fold_string_replace(*span, arguments, name, object, ctx)
            }
            "fromCharCode" => Self::try_fold_string_from_char_code(*span, arguments, object, ctx),
            "toString" => Self::try_fold_to_string(*span, arguments, object, ctx),
            "pow" => self.try_fold_pow(*span, arguments, object, ctx),
            "sqrt" | "cbrt" => Self::try_fold_roots(*span, arguments, name, object, ctx),
            "abs" | "ceil" | "floor" | "round" | "fround" | "trunc" | "sign" => {
                Self::try_fold_math_unary(*span, arguments, name, object, ctx)
            }
            "min" | "max" => Self::try_fold_math_variadic(*span, arguments, name, object, ctx),
            "of" => Self::try_fold_array_of(*span, arguments, name, object, ctx),
            _ => None,
        };
        if let Some(replacement) = replacement {
            self.mark_current_function_as_changed();
            *node = replacement;
        }
    }

    fn try_fold_string_casing(
        span: Span,
        args: &Arguments,
        name: &str,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if args.len() >= 1 {
            return None;
        }
        let Expression::StringLiteral(s) = object else { return None };
        let value = match name {
            "toLowerCase" => s.value.cow_to_lowercase(),
            "toUpperCase" => s.value.cow_to_uppercase(),
            "trim" => Cow::Borrowed(s.value.trim()),
            _ => return None,
        };
        Some(ctx.ast.expression_string_literal(span, value, None))
    }

    fn try_fold_string_index_of(
        span: Span,
        args: &Arguments,
        name: &str,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if args.len() >= 3 {
            return None;
        }
        let Expression::StringLiteral(s) = object else { return None };
        let search_value = match args.first() {
            Some(Argument::StringLiteral(string_lit)) => Some(string_lit.value.as_str()),
            None => None,
            _ => return None,
        };
        let search_start_index = match args.get(1) {
            Some(Argument::NumericLiteral(numeric_lit)) => Some(numeric_lit.value),
            None => None,
            _ => return None,
        };
        let result = match name {
            "indexOf" => s.value.as_str().index_of(search_value, search_start_index),
            "lastIndexOf" => s.value.as_str().last_index_of(search_value, search_start_index),
            _ => unreachable!(),
        };
        #[expect(clippy::cast_precision_loss)]
        Some(ctx.ast.expression_numeric_literal(span, result as f64, None, NumberBase::Decimal))
    }

    fn try_fold_string_substring_or_slice(
        span: Span,
        args: &Arguments,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if args.len() > 2 {
            return None;
        }
        let Expression::StringLiteral(s) = object else { return None };
        let start_idx = args.first().and_then(|arg| match arg {
            Argument::SpreadElement(_) => None,
            _ => ctx.get_side_free_number_value(arg.to_expression()),
        });
        let end_idx = args.get(1).and_then(|arg| match arg {
            Argument::SpreadElement(_) => None,
            _ => ctx.get_side_free_number_value(arg.to_expression()),
        });
        #[expect(clippy::cast_precision_loss)]
        if start_idx.is_some_and(|start| start > s.value.len() as f64 || start < 0.0)
            || end_idx.is_some_and(|end| end > s.value.len() as f64 || end < 0.0)
        {
            return None;
        }
        if let (Some(start), Some(end)) = (start_idx, end_idx) {
            if start > end {
                return None;
            }
        };
        Some(ctx.ast.expression_string_literal(
            span,
            s.value.as_str().substring(start_idx, end_idx),
            None,
        ))
    }

    fn try_fold_string_char_at(
        span: Span,
        args: &Arguments,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if args.len() > 1 {
            return None;
        }
        let Expression::StringLiteral(s) = object else { return None };
        let char_at_index: Option<f64> = match args.first() {
            Some(Argument::NumericLiteral(numeric_lit)) => Some(numeric_lit.value),
            Some(Argument::UnaryExpression(unary_expr))
                if unary_expr.operator == UnaryOperator::UnaryNegation =>
            {
                let Expression::NumericLiteral(numeric_lit) = &unary_expr.argument else {
                    return None;
                };
                Some(-(numeric_lit.value))
            }
            None => None,
            _ => return None,
        };
        let result =
            &s.value.as_str().char_at(char_at_index).map_or(String::new(), |v| v.to_string());
        Some(ctx.ast.expression_string_literal(span, result, None))
    }

    #[expect(clippy::cast_lossless)]
    fn try_fold_string_char_code_at(
        span: Span,
        args: &Arguments,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let Expression::StringLiteral(s) = object else { return None };
        let char_at_index = match args.first() {
            None => Some(0.0),
            Some(Argument::SpreadElement(_)) => None,
            Some(e) => ctx.get_side_free_number_value(e.to_expression()),
        }?;
        let value = if (0.0..65536.0).contains(&char_at_index) {
            s.value.as_str().char_code_at(Some(char_at_index))? as f64
        } else if char_at_index.is_nan() || char_at_index.is_infinite() {
            return None;
        } else {
            f64::NAN
        };
        Some(ctx.ast.expression_numeric_literal(span, value, None, NumberBase::Decimal))
    }

    fn try_fold_string_replace(
        span: Span,
        args: &Arguments,
        name: &str,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if args.len() != 2 {
            return None;
        }
        let Expression::StringLiteral(s) = object else { return None };
        let search_value = args.first().unwrap();
        let search_value = match search_value {
            Argument::SpreadElement(_) => return None,
            match_expression!(Argument) => {
                ctx.get_side_free_string_value(search_value.to_expression())?
            }
        };
        let replace_value = args.get(1).unwrap();
        let replace_value = match replace_value {
            Argument::SpreadElement(_) => return None,
            match_expression!(Argument) => {
                ctx.get_side_free_string_value(replace_value.to_expression())?
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
        Some(ctx.ast.expression_string_literal(span, result, None))
    }

    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless)]
    fn try_fold_string_from_char_code(
        span: Span,
        args: &mut Arguments,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let Expression::Identifier(ident) = object else { return None };
        if ident.name != "String" || !ctx.is_global_reference(ident) {
            return None;
        }
        let mut s = String::with_capacity(args.len());
        for arg in args {
            let expr = arg.as_expression()?;
            let v = ctx.get_side_free_number_value(expr)?;
            let v = v.to_int_32() as u16 as u32;
            let c = char::try_from(v).ok()?;
            s.push(c);
        }
        Some(ctx.ast.expression_string_literal(span, s, None))
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_lossless,
        clippy::float_cmp
    )]
    fn try_fold_to_string(
        span: Span,
        args: &mut Arguments,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
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
                    use oxc_syntax::number::ToJsString;
                    let s = lit.value.to_js_string();
                    return Some(ctx.ast.expression_string_literal(span, s, None));
                }
                // Only convert integers for other radix values.
                let value = lit.value;
                if value.is_infinite() {
                    let s = if value.is_sign_negative() { "-Infinity" } else { "Infinity" };
                    return Some(ctx.ast.expression_string_literal(span, s, None));
                }
                if value.is_nan() {
                    return Some(ctx.ast.expression_string_literal(span, "NaN", None));
                }
                if value >= 0.0 && value.fract() != 0.0 {
                    return None;
                }
                let i = value as u32;
                if i as f64 != value {
                    return None;
                }
                Some(ctx.ast.expression_string_literal(span, Self::format_radix(i, radix), None))
            }
            // `null` returns type errors
            Expression::BooleanLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::StringLiteral(_)
                if args.is_empty() =>
            {
                use oxc_ecmascript::ToJsString;
                object.to_js_string().map(|s| ctx.ast.expression_string_literal(span, s, None))
            }
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

    fn validate_global_reference(expr: &Expression<'a>, target: &str, ctx: Ctx<'a, '_>) -> bool {
        let Expression::Identifier(ident) = expr else { return false };
        ctx.is_global_reference(ident) && ident.name == target
    }

    fn validate_arguments(args: &Arguments, expected_len: usize) -> bool {
        (args.len() == expected_len) && args.iter().all(Argument::is_expression)
    }

    /// `Math.pow(a, b)` -> `+(a) ** +b`
    fn try_fold_pow(
        &self,
        span: Span,
        arguments: &mut Arguments<'a>,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if self.target < ESTarget::ES2016 {
            return None;
        }
        if !Self::validate_global_reference(object, "Math", ctx)
            || !Self::validate_arguments(arguments, 2)
        {
            return None;
        }

        let mut second_arg = arguments.pop().expect("checked len above");
        let second_arg = second_arg.to_expression_mut(); // checked above
        let mut first_arg = arguments.pop().expect("checked len above");
        let first_arg = first_arg.to_expression_mut(); // checked above

        let wrap_with_unary_plus_if_needed = |expr: &mut Expression<'a>| {
            if ValueType::from(&*expr).is_number() {
                ctx.ast.move_expression(expr)
            } else {
                ctx.ast.expression_unary(
                    SPAN,
                    UnaryOperator::UnaryPlus,
                    ctx.ast.move_expression(expr),
                )
            }
        };

        Some(ctx.ast.expression_binary(
            span,
            wrap_with_unary_plus_if_needed(first_arg),
            BinaryOperator::Exponential,
            wrap_with_unary_plus_if_needed(second_arg),
        ))
    }

    /// `Math.sqrt(a)`, `Math.cbrt(a)`
    ///
    /// These cannot be replaced with `a ** .5`, `a ** (1/3)` because `Math.sqrt(-0)` returns `-0` where `(-0) ** .5` returns `0`.
    /// It can be replaced when the value is known to be not `-0`, but that makes the gzip output worse.
    fn try_fold_roots(
        span: Span,
        arguments: &Arguments,
        name: &str,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if !Self::validate_global_reference(object, "Math", ctx)
            || !Self::validate_arguments(arguments, 1)
        {
            return None;
        }
        let arg_val = ctx.get_side_free_number_value(arguments[0].to_expression())?;
        if arg_val == f64::INFINITY || arg_val.is_nan() || arg_val == 0.0 {
            return Some(ctx.ast.expression_numeric_literal(
                span,
                arg_val,
                None,
                NumberBase::Decimal,
            ));
        }
        if arg_val < 0.0 {
            return Some(ctx.ast.expression_numeric_literal(
                span,
                f64::NAN,
                None,
                NumberBase::Decimal,
            ));
        }
        let calculated_val = match name {
            "sqrt" => arg_val.sqrt(),
            "cbrt" => arg_val.cbrt(),
            _ => unreachable!(),
        };
        (calculated_val.fract() == 0.0).then(|| {
            ctx.ast.expression_numeric_literal(span, calculated_val, None, NumberBase::Decimal)
        })
    }

    fn try_fold_math_unary(
        span: Span,
        arguments: &Arguments,
        name: &str,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if !Self::validate_global_reference(object, "Math", ctx)
            || !Self::validate_arguments(arguments, 1)
        {
            return None;
        }
        let arg_val = ctx.get_side_free_number_value(arguments[0].to_expression())?;
        let result = match name {
            "abs" => arg_val.abs(),
            "ceil" => arg_val.ceil(),
            "floor" => arg_val.floor(),
            "round" => {
                // We should be aware that the behavior in JavaScript and Rust towards `round` is different.
                // In Rust, when facing `.5`, it may follow `half-away-from-zero` instead of round to upper bound.
                // So we need to handle it manually.
                let frac_part = arg_val.fract();
                let epsilon = 2f64.powf(-52f64);
                if (frac_part.abs() - 0.5).abs() < epsilon {
                    // We should ceil it.
                    arg_val.ceil()
                } else {
                    arg_val.round()
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            "fround" if arg_val.fract() == 0f64 || arg_val.is_nan() || arg_val.is_infinite() => {
                f64::from(arg_val as f32)
            }
            "fround" => return None,
            "trunc" => arg_val.trunc(),
            "sign" if arg_val.to_bits() == 0f64.to_bits() => 0f64,
            "sign" if arg_val.to_bits() == (-0f64).to_bits() => -0f64,
            "sign" => arg_val.signum(),
            _ => unreachable!(),
        };
        // These results are always shorter to return as a number, so we can just return them as NumericLiteral.
        Some(ctx.ast.expression_numeric_literal(span, result, None, NumberBase::Decimal))
    }

    fn try_fold_math_variadic(
        span: Span,
        arguments: &Arguments,
        name: &str,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if !Self::validate_global_reference(object, "Math", ctx) {
            return None;
        }
        let numbers = arguments
            .iter()
            .map(|arg| arg.as_expression().map(|e| ctx.get_side_free_number_value(e))?)
            .collect::<Option<Vec<_>>>()?;
        let result = if numbers.iter().any(|n| n.is_nan()) {
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
        };
        Some(ctx.ast.expression_numeric_literal(span, result, None, NumberBase::Decimal))
    }

    /// `[].concat(a).concat(b)` -> `[].concat(a, b)`
    /// `"".concat(a).concat(b)` -> `"".concat(a, b)`
    fn try_fold_concat_chain(&mut self, node: &mut Expression<'a>, ctx: Ctx<'a, '_>) {
        let original_span = if let Expression::CallExpression(root_call_expr) = node {
            root_call_expr.span
        } else {
            return;
        };

        if matches!(ctx.parent(), Ancestor::StaticMemberExpressionObject(_)) {
            return;
        }

        let mut current_node: &mut Expression = node;
        let mut collected_arguments = ctx.ast.vec();
        let new_root_callee: &mut Expression<'a>;
        loop {
            let Expression::CallExpression(ce) = current_node else {
                return;
            };
            let Expression::StaticMemberExpression(member) = &ce.callee else {
                return;
            };
            if member.optional || member.property.name != "concat" {
                return;
            }

            // We don't need to check if the arguments has a side effect here.
            //
            // The only side effect Array::concat / String::concat can cause is throwing an error when the created array is too long.
            // With the compressor assumption, that error can be moved.
            //
            // For example, if we have `[].concat(a).concat(b)`, the steps before the compression is:
            // 1. evaluate `a`
            // 2. `[].concat(a)` creates `[a]`
            // 3. evaluate `b`
            // 4. `.concat(b)` creates `[a, b]`
            //
            // The steps after the compression (`[].concat(a, b)`) is:
            // 1. evaluate `a`
            // 2. evaluate `b`
            // 3. `[].concat(a, b)` creates `[a, b]`
            //
            // The error that has to be thrown in the second step before the compression will be thrown in the third step.

            let CallExpression { callee, arguments, .. } = ce.as_mut();
            collected_arguments.push(arguments);

            // [].concat() or "".concat()
            let is_root_expr_concat = {
                let Expression::StaticMemberExpression(member) = callee else { unreachable!() };
                matches!(
                    &member.object,
                    Expression::ArrayExpression(_) | Expression::StringLiteral(_)
                )
            };
            if is_root_expr_concat {
                new_root_callee = callee;
                break;
            }

            let Expression::StaticMemberExpression(member) = callee else { unreachable!() };
            current_node = &mut member.object;
        }

        if collected_arguments.len() <= 1 {
            return;
        }

        *node = ctx.ast.expression_call(
            original_span,
            ctx.ast.move_expression(new_root_callee),
            Option::<TSTypeParameterInstantiation>::None,
            ctx.ast.vec_from_iter(
                collected_arguments.into_iter().rev().flat_map(|arg| ctx.ast.move_vec(arg)),
            ),
            false,
        );
        self.mark_current_function_as_changed();
    }

    /// `[].concat(1, 2)` -> `[1, 2]`
    fn try_fold_concat(
        span: Span,
        args: &mut Arguments<'a>,
        callee: &mut Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        // let concat chaining reduction handle it first
        if let Ancestor::StaticMemberExpressionObject(parent_member) = ctx.parent() {
            if parent_member.property().name.as_str() == "concat" {
                return None;
            }
        }

        let object = match callee {
            Expression::StaticMemberExpression(member) => &mut member.object,
            Expression::ComputedMemberExpression(member) => &mut member.object,
            _ => unreachable!(),
        };
        let Expression::ArrayExpression(array_expr) = object else { return None };

        let can_merge_until = args
            .iter()
            .enumerate()
            .take_while(|(_, argument)| match argument {
                Argument::SpreadElement(_) => false,
                match_expression!(Argument) => {
                    let argument = argument.to_expression();
                    if argument.is_literal() {
                        true
                    } else {
                        matches!(argument, Expression::ArrayExpression(_))
                    }
                }
            })
            .map(|(i, _)| i)
            .last();

        if let Some(can_merge_until) = can_merge_until {
            for argument in args.drain(..=can_merge_until) {
                let argument = argument.into_expression();
                if argument.is_literal() {
                    array_expr.elements.push(ArrayExpressionElement::from(argument));
                } else {
                    let Expression::ArrayExpression(mut argument_array) = argument else {
                        unreachable!()
                    };
                    array_expr.elements.append(&mut argument_array.elements);
                }
            }
        }

        if args.is_empty() {
            Some(ctx.ast.move_expression(object))
        } else if can_merge_until.is_some() {
            Some(ctx.ast.expression_call(
                span,
                ctx.ast.move_expression(callee),
                Option::<TSTypeParameterInstantiation>::None,
                ctx.ast.move_vec(args),
                false,
            ))
        } else {
            None
        }
    }

    fn try_fold_known_property_access(&mut self, node: &mut Expression<'a>, ctx: Ctx<'a, '_>) {
        let (name, object, span) = match &node {
            Expression::StaticMemberExpression(member) if !member.optional => {
                (member.property.name.as_str(), &member.object, member.span)
            }
            Expression::ComputedMemberExpression(member) if !member.optional => {
                match &member.expression {
                    Expression::StringLiteral(s) => (s.value.as_str(), &member.object, member.span),
                    _ => return,
                }
            }
            _ => return,
        };
        let Expression::Identifier(ident) = object else { return };

        if !ctx.is_global_reference(ident) {
            return;
        }

        let replacement = match ident.name.as_str() {
            "Number" => self.try_fold_number_constants(name, span, ctx),
            _ => None,
        };
        if let Some(replacement) = replacement {
            self.mark_current_function_as_changed();
            *node = replacement;
        }
    }

    /// replace `Number.*` constants
    fn try_fold_number_constants(
        &self,
        name: &str,
        span: Span,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let num = |span: Span, n: f64| {
            ctx.ast.expression_numeric_literal(span, n, None, NumberBase::Decimal)
        };
        // [neg] base ** exponent [op] a
        let pow_with_expr =
            |span: Span, base: f64, exponent: f64, op: BinaryOperator, a: f64| -> Expression<'a> {
                ctx.ast.expression_binary(
                    span,
                    ctx.ast.expression_binary(
                        SPAN,
                        num(SPAN, base),
                        BinaryOperator::Exponential,
                        num(SPAN, exponent),
                    ),
                    op,
                    num(SPAN, a),
                )
            };

        Some(match name {
            "POSITIVE_INFINITY" => num(span, f64::INFINITY),
            "NEGATIVE_INFINITY" => num(span, f64::NEG_INFINITY),
            "NaN" => num(span, f64::NAN),
            "MAX_SAFE_INTEGER" => {
                #[allow(clippy::cast_precision_loss)]
                if self.target < ESTarget::ES2016 {
                    num(span, 2.0f64.powf(53.0) - 1.0)
                } else {
                    // 2**53 - 1
                    pow_with_expr(span, 2.0, 53.0, BinaryOperator::Subtraction, 1.0)
                }
            }
            "MIN_SAFE_INTEGER" => {
                #[allow(clippy::cast_precision_loss)]
                if self.target < ESTarget::ES2016 {
                    num(span, -(2.0f64.powf(53.0) - 1.0))
                } else {
                    // -(2**53 - 1)
                    ctx.ast.expression_unary(
                        span,
                        UnaryOperator::UnaryNegation,
                        pow_with_expr(SPAN, 2.0, 53.0, BinaryOperator::Subtraction, 1.0),
                    )
                }
            }
            "EPSILON" => {
                if self.target < ESTarget::ES2016 {
                    return None;
                }
                // 2**-52
                ctx.ast.expression_binary(
                    span,
                    num(SPAN, 2.0),
                    BinaryOperator::Exponential,
                    num(SPAN, -52.0),
                )
            }
            _ => return None,
        })
    }

    fn try_fold_array_of(
        span: Span,
        arguments: &mut Arguments<'a>,
        name: &str,
        object: &Expression<'a>,
        ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if !Self::validate_global_reference(object, "Array", ctx) {
            return None;
        }
        if name != "of" {
            return None;
        }
        Some(ctx.ast.expression_array(
            span,
            ctx.ast.vec_from_iter(arguments.drain(..).map(ArrayExpressionElement::from)),
            None,
        ))
    }
}

/// Port from: <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/PeepholeReplaceKnownMethodsTest.java>
#[cfg(test)]
mod test {
    use oxc_syntax::es_target::ESTarget;

    use crate::{
        tester::{run, test, test_same},
        CompressOptions,
    };

    fn test_es2015(code: &str, expected: &str) {
        let opts = CompressOptions { target: ESTarget::ES2015, ..CompressOptions::default() };
        assert_eq!(run(code, Some(opts)), run(expected, None));
    }

    fn test_value(code: &str, expected: &str) {
        test(format!("x = {code}").as_str(), format!("x = {expected}").as_str());
    }

    fn test_same_value(code: &str) {
        test_same(format!("x = {code}").as_str());
    }

    #[test]
    fn test_string_index_of() {
        test("x = 'abcdef'.indexOf('g')", "x = -1");
        test("x = 'abcdef'.indexOf('b')", "x = 1");
        test("x = 'abcdefbe'.indexOf('b', 2)", "x = 6");
        test("x = 'abcdef'.indexOf('bcd')", "x = 1");
        test("x = 'abcdefsdfasdfbcdassd'.indexOf('bcd', 4)", "x = 13");

        test("x = 'abcdef'.lastIndexOf('b')", "x = 1");
        test("x = 'abcdefbe'.lastIndexOf('b')", "x = 6");
        test("x = 'abcdefbe'.lastIndexOf('b', 5)", "x = 1");

        // Both elements must be strings. Don't do anything if either one is not
        // string.
        // TODO: cast first arg to a string, and fold if possible.
        // test("x = 'abc1def'.indexOf(1)", "x = 3");
        // test("x = 'abcNaNdef'.indexOf(NaN)", "x = 3");
        // test("x = 'abcundefineddef'.indexOf(undefined)", "x = 3");
        // test("x = 'abcnulldef'.indexOf(null)", "x = 3");
        // test("x = 'abctruedef'.indexOf(true)", "x = 3");

        // The following test case fails with JSC_PARSE_ERROR. Hence omitted.
        // test_same("x = 1.indexOf('bcd');");
        test_same("x = NaN.indexOf('bcd')");
        test("x = undefined.indexOf('bcd')", "x = (void 0).indexOf('bcd')");
        test_same("x = null.indexOf('bcd')");
        test_same("x = (!0).indexOf('bcd')");
        test_same("x = (!1).indexOf('bcd')");

        // Avoid dealing with regex or other types.
        test_same("x = 'abcdef'.indexOf(/b./)");
        test_same("x = 'abcdef'.indexOf({a:2})");
        test_same("x = 'abcdef'.indexOf([1,2])");

        // Template Strings
        test("x = `abcdef`.indexOf('b')", "x = 1");
        test_same("x = `Hello ${name}`.indexOf('a')");
        test_same("x = tag `Hello ${name}`.indexOf('a')");
    }

    #[test]
    #[ignore]
    fn test_string_join_add_sparse() {
        test("x = [,,'a'].join(',')", "x = ',,a'");
    }

    #[test]
    #[ignore]
    fn test_no_string_join() {
        test_same("x = [].join(',',2)");
        test_same("x = [].join(f)");
    }

    #[test]
    #[ignore]
    fn test_string_join_add() {
        test("x = ['a', 'b', 'c'].join('')", "x = \"abc\"");
        test("x = [].join(',')", "x = \"\"");
        test("x = ['a'].join(',')", "x = \"a\"");
        test("x = ['a', 'b', 'c'].join(',')", "x = \"a,b,c\"");
        test("x = ['a', foo, 'b', 'c'].join(',')", "x = [\"a\",foo,\"b,c\"].join()");
        test("x = [foo, 'a', 'b', 'c'].join(',')", "x = [foo,\"a,b,c\"].join()");
        test("x = ['a', 'b', 'c', foo].join(',')", "x = [\"a,b,c\",foo].join()");

        // Works with numbers
        test("x = ['a=', 5].join('')", "x = \"a=5\"");
        test("x = ['a', '5'].join(7)", "x = \"a75\"");

        // Works on boolean
        test("x = ['a=', false].join('')", "x = \"a=false\"");
        test("x = ['a', '5'].join(true)", "x = \"atrue5\"");
        test("x = ['a', '5'].join(false)", "x = \"afalse5\"");

        // Only optimize if it's a size win.
        test(
            "x = ['a', '5', 'c'].join('a very very very long chain')",
            "x = [\"a\",\"5\",\"c\"].join(\"a very very very long chain\")",
        );

        // Template strings
        test("x = [`a`, `b`, `c`].join(``)", "x = 'abc'");
        test("x = [`a`, `b`, `c`].join('')", "x = 'abc'");

        // TODO(user): Its possible to fold this better.
        test_same("x = ['', foo].join('-')");
        test_same("x = ['', foo, ''].join()");

        test(
            "x = ['', '', foo, ''].join(',')", //
            "x = [ ','  , foo, ''].join()",
        );
        test(
            "x = ['', '', foo, '', ''].join(',')", //
            "x = [ ',',   foo,  ','].join()",
        );

        test(
            "x = ['', '', foo, '', '', bar].join(',')", //
            "x = [ ',',   foo,  ',',   bar].join()",
        );

        test(
            "x = [1,2,3].join('abcdef')", //
            "x = '1abcdef2abcdef3'",
        );

        test("x = [1,2].join()", "x = '1,2'");
        test("x = [null,undefined,''].join(',')", "x = ',,'");
        test("x = [null,undefined,0].join(',')", "x = ',,0'");
        // This can be folded but we don't currently.
        test_same("x = [[1,2],[3,4]].join()"); // would like: "x = '1,2,3,4'"
    }

    #[test]
    #[ignore]
    fn test_string_join_add_b1992789() {
        test("x = ['a'].join('')", "x = \"a\"");
        test_same("x = [foo()].join('')");
        test_same("[foo()].join('')");
        test("[null].join('')", "''");
    }

    #[test]
    fn test_fold_string_replace() {
        test("x = 'c'.replace('c','x')", "x = 'x'");
        test("x = 'ac'.replace('c','x')", "x = 'ax'");
        test("x = 'ca'.replace('c','x')", "x = 'xa'");
        test("x = 'ac'.replace('c','xxx')", "x = 'axxx'");
        test("x = 'ca'.replace('c','xxx')", "x = 'xxxa'");

        // only one instance replaced
        test("x = 'acaca'.replace('c','x')", "x = 'axaca'");
        test("x = 'ab'.replace('','x')", "x = 'xab'");

        test_same("'acaca'.replace(/c/,'x')"); // this will affect the global RegExp props
        test_same("'acaca'.replace(/c/g,'x')"); // this will affect the global RegExp props

        // not a literal
        test_same("x.replace('x','c')");

        test_same("'Xyz'.replace('Xyz', '$$')"); // would fold to '$'
        test_same("'PreXyzPost'.replace('Xyz', '$&')"); // would fold to 'PreXyzPost'
        test_same("'PreXyzPost'.replace('Xyz', '$`')"); // would fold to 'PrePrePost'
        test_same("'PreXyzPost'.replace('Xyz', '$\\'')"); // would fold to  'PrePostPost'
        test_same("'PreXyzPostXyz'.replace('Xyz', '$\\'')"); // would fold to 'PrePostXyzPostXyz'
        test_same("'123'.replace('2', '$`')"); // would fold to '113'
    }

    #[test]
    fn test_fold_string_replace_all() {
        test("x = 'abcde'.replaceAll('bcd','c')", "x = 'ace'");
        test("x = 'abcde'.replaceAll('c','xxx')", "x = 'abxxxde'");
        test("x = 'abcde'.replaceAll('xxx','c')", "x = 'abcde'");
        test("x = 'ab'.replaceAll('','x')", "x = 'xaxbx'");

        test("x = 'c_c_c'.replaceAll('c','x')", "x = 'x_x_x'");

        test_same("x = 'acaca'.replaceAll(/c/,'x')"); // this should throw
        test_same("x = 'acaca'.replaceAll(/c/g,'x')"); // this will affect the global RegExp props

        // not a literal
        test_same("x.replaceAll('x','c')");

        test_same("'Xyz'.replaceAll('Xyz', '$$')"); // would fold to '$'
        test_same("'PreXyzPost'.replaceAll('Xyz', '$&')"); // would fold to 'PreXyzPost'
        test_same("'PreXyzPost'.replaceAll('Xyz', '$`')"); // would fold to 'PrePrePost'
        test_same("'PreXyzPost'.replaceAll('Xyz', '$\\'')"); // would fold to  'PrePostPost'
        test_same("'PreXyzPostXyz'.replaceAll('Xyz', '$\\'')"); // would fold to 'PrePostXyzPost'
        test_same("'123'.replaceAll('2', '$`')"); // would fold to '113'
    }

    #[test]
    fn test_fold_string_substring() {
        test("x = 'abcde'.substring(0,2)", "x = 'ab'");
        test("x = 'abcde'.substring(1,2)", "x = 'b'");
        test("x = 'abcde'.substring(2)", "x = 'cde'");

        // we should be leaving negative, out-of-bound, and inverted indices alone for now
        test_same("x = 'abcde'.substring(-1)");
        test_same("x = 'abcde'.substring(1, -2)");
        test_same("x = 'abcde'.substring(1, 2, 3)");
        test_same("x = 'abcde'.substring(2, 0)");
        test_same("x = 'a'.substring(0, 2)");

        // Template strings
        test("x = `abcdef`.substring(0,2)", "x = 'ab'");
        test_same("x = `abcdef ${abc}`.substring(0,2)");
    }

    #[test]
    fn test_fold_string_slice() {
        test("x = 'abcde'.slice(0,2)", "x = 'ab'");
        test("x = 'abcde'.slice(1,2)", "x = 'b'");
        test("x = 'abcde'.slice(2)", "x = 'cde'");

        // we should be leaving negative, out-of-bound, and inverted indices alone for now
        test_same("x = 'abcde'.slice(-1)");
        test_same("x = 'abcde'.slice(1, -2)");
        test_same("x = 'abcde'.slice(1, 2, 3)");
        test_same("x = 'abcde'.slice(2, 0)");
        test_same("x = 'a'.slice(0, 2)");

        // Template strings
        test("x = `abcdef`.slice(0, 2)", "x = 'ab'");
        test_same("x = `abcdef ${abc}`.slice(0,2)");
    }

    #[test]
    fn test_fold_string_char_at() {
        test("x = 'abcde'.charAt(0)", "x = 'a'");
        test("x = 'abcde'.charAt(1)", "x = 'b'");
        test("x = 'abcde'.charAt(2)", "x = 'c'");
        test("x = 'abcde'.charAt(3)", "x = 'd'");
        test("x = 'abcde'.charAt(4)", "x = 'e'");
        test("x = 'abcde'.charAt(5)", "x = ''");
        test("x = 'abcde'.charAt(-1)", "x = ''");
        test("x = 'abcde'.charAt()", "x = 'a'");
        test_same("x = 'abcde'.charAt(0, ++z)");
        test_same("x = 'abcde'.charAt(y)");
        test_same("x = 'abcde'.charAt(null)"); // or x = 'a'
        test_same("x = 'abcde'.charAt(!0)"); // or x = 'b'
                                             // test("x = '\\ud834\udd1e'.charAt(0)", "x = '\\ud834'");
                                             // test("x = '\\ud834\udd1e'.charAt(1)", "x = '\\udd1e'");

        // Template strings
        test("x = `abcdef`.charAt(0)", "x = 'a'");
        test_same("x = `abcdef ${abc}`.charAt(0)");
    }

    #[test]
    fn test_fold_string_char_code_at() {
        test("x = 'abcde'.charCodeAt()", "x = 97");
        test("x = 'abcde'.charCodeAt(0)", "x = 97");
        test("x = 'abcde'.charCodeAt(1)", "x = 98");
        test("x = 'abcde'.charCodeAt(2)", "x = 99");
        test("x = 'abcde'.charCodeAt(3)", "x = 100");
        test("x = 'abcde'.charCodeAt(4)", "x = 101");
        test_same("x = 'abcde'.charCodeAt(5)");
        test("x = 'abcde'.charCodeAt(-1)", "x = NaN");
        test_same("x = 'abcde'.charCodeAt(y)");
        test("x = 'abcde'.charCodeAt()", "x = 97");
        test("x = 'abcde'.charCodeAt(0, ++z)", "x = 97");
        test("x = 'abcde'.charCodeAt(null)", "x = 97");
        test("x = 'abcde'.charCodeAt(true)", "x = 98");
        // test("x = '\\ud834\udd1e'.charCodeAt(0)", "x = 55348");
        // test("x = '\\ud834\udd1e'.charCodeAt(1)", "x = 56606");
        test("x = `abcdef`.charCodeAt(0)", "x = 97");
        test_same("x = `abcdef ${abc}`.charCodeAt(0)");
    }

    #[test]
    #[ignore]
    fn test_fold_string_split() {
        // late = false;
        test("x = 'abcde'.split('foo')", "x = ['abcde']");
        test("x = 'abcde'.split()", "x = ['abcde']");
        test("x = 'abcde'.split(null)", "x = ['abcde']");
        test("x = 'a b c d e'.split(' ')", "x = ['a','b','c','d','e']");
        test("x = 'a b c d e'.split(' ', 0)", "x = []");
        test("x = 'abcde'.split('cd')", "x = ['ab','e']");
        test("x = 'a b c d e'.split(' ', 1)", "x = ['a']");
        test("x = 'a b c d e'.split(' ', 3)", "x = ['a','b','c']");
        test("x = 'a b c d e'.split(null, 1)", "x = ['a b c d e']");
        test("x = 'aaaaa'.split('a')", "x = ['', '', '', '', '', '']");
        test("x = 'xyx'.split('x')", "x = ['', 'y', '']");

        // Empty separator
        test("x = 'abcde'.split('')", "x = ['a','b','c','d','e']");
        test("x = 'abcde'.split('', 3)", "x = ['a','b','c']");

        // Empty separator AND empty string
        test("x = ''.split('')", "x = []");

        // Separator equals string
        test("x = 'aaa'.split('aaa')", "x = ['','']");
        test("x = ' '.split(' ')", "x = ['','']");

        test_same("x = 'abcde'.split(/ /)");
        test_same("x = 'abcde'.split(' ', -1)");

        // Template strings
        test_same("x = `abcdef`.split()");
        test_same("x = `abcdef ${abc}`.split()");

        // late = true;
        // test_same("x = 'a b c d e'.split(' ')");
    }

    #[test]
    #[ignore]
    fn test_join_bug() {
        test("var x = [].join();", "var x = '';");
        test_same("var x = [x].join();");
        test_same("var x = [x,y].join();");
        test_same("var x = [x,y,z].join();");

        // test_same(
        // lines(
        // "shape['matrix'] = [",
        // "    Number(headingCos2).toFixed(4),",
        // "    Number(-headingSin2).toFixed(4),",
        // "    Number(headingSin2 * yScale).toFixed(4),",
        // "    Number(headingCos2 * yScale).toFixed(4),",
        // "    0,",
        // "    0",
        // "  ].join()"));
    }

    #[test]
    #[ignore]
    fn test_join_spread1() {
        test_same("var x = [...foo].join('');");
        test_same("var x = [...someMap.keys()].join('');");
        test_same("var x = [foo, ...bar].join('');");
        test_same("var x = [...foo, bar].join('');");
        test_same("var x = [...foo, 'bar'].join('');");
        test_same("var x = ['1', ...'2', '3'].join('');");
        test_same("var x = ['1', ...['2'], '3'].join('');");
    }

    #[test]
    #[ignore]
    fn test_join_spread2() {
        test("var x = [...foo].join(',');", "var x = [...foo].join();");
        test("var x = [...someMap.keys()].join(',');", "var x = [...someMap.keys()].join();");
        test("var x = [foo, ...bar].join(',');", "var x = [foo, ...bar].join();");
        test("var x = [...foo, bar].join(',');", "var x = [...foo, bar].join();");
        test("var x = [...foo, 'bar'].join(',');", "var x = [...foo, 'bar'].join();");
        test("var x = ['1', ...'2', '3'].join(',');", "var x = ['1', ...'2', '3'].join();");
        test("var x = ['1', ...['2'], '3'].join(',');", "var x = ['1', ...['2'], '3'].join();");
    }

    #[test]
    fn test_to_upper() {
        test("x = 'a'.toUpperCase()", "x = 'A'");
        test("x = 'A'.toUpperCase()", "x = 'A'");
        test("x = 'aBcDe'.toUpperCase()", "x = 'ABCDE'");

        test("x = `abc`.toUpperCase()", "x = 'ABC'");
        test_same("`a ${bc}`.toUpperCase()");

        /*
         * Make sure things aren't totally broken for non-ASCII strings, non-exhaustive.
         *
         * <p>This includes things like:
         *
         * <ul>
         *   <li>graphemes with multiple code-points
         *   <li>graphemes represented by multiple graphemes in other cases
         *   <li>graphemes whose case changes are not round-trippable
         *   <li>graphemes that change case in a position sentitive way
         * </ul>
         */
        test("x = '\u{0049}'.toUpperCase()", "x = '\u{0049}'");
        test("x = '\u{0069}'.toUpperCase()", "x = '\u{0049}'");
        test("x = '\u{0130}'.toUpperCase()", "x = '\u{0130}'");
        test("x = '\u{0131}'.toUpperCase()", "x = '\u{0049}'");
        test("x = '\u{0049}\u{0307}'.toUpperCase()", "x = '\u{0049}\u{0307}'");
        test("x = 'ß'.toUpperCase()", "x = 'SS'");
        test("x = 'SS'.toUpperCase()", "x = 'SS'");
        test("x = 'σ'.toUpperCase()", "x = 'Σ'");
        test("x = 'σς'.toUpperCase()", "x = 'ΣΣ'");
    }

    #[test]
    fn test_to_lower() {
        test("x = 'A'.toLowerCase()", "x = 'a'");
        test("x = 'a'.toLowerCase()", "x = 'a'");
        test("x = 'aBcDe'.toLowerCase()", "x = 'abcde'");

        test("x = `ABC`.toLowerCase()", "x = 'abc'");
        test_same("`A ${BC}`.toLowerCase()");

        /*
         * Make sure things aren't totally broken for non-ASCII strings, non-exhaustive.
         *
         * <p>This includes things like:
         *
         * <ul>
         *   <li>graphemes with multiple code-points
         *   <li>graphemes with multiple representations
         *   <li>graphemes represented by multiple graphemes in other cases
         *   <li>graphemes whose case changes are not round-trippable
         *   <li>graphemes that change case in a position sentitive way
         * </ul>
         */
        test("x = '\u{0049}'.toLowerCase()", "x = '\u{0069}'");
        test("x = '\u{0069}'.toLowerCase()", "x = '\u{0069}'");
        test("x = '\u{0130}'.toLowerCase()", "x = '\u{0069}\u{0307}'");
        test("x = '\u{0131}'.toLowerCase()", "x = '\u{0131}'");
        test("x = '\u{0049}\u{0307}'.toLowerCase()", "x = '\u{0069}\u{0307}'");
        test("x = 'ß'.toLowerCase()", "x = 'ß'");
        test("x = 'SS'.toLowerCase()", "x = 'ss'");
        test("x = 'Σ'.toLowerCase()", "x = 'σ'");
        test("x = 'ΣΣ'.toLowerCase()", "x = 'σς'");
    }

    #[test]
    fn test_fold_math_functions_bug() {
        test_same("Math[0]()");
    }

    #[test]
    fn test_fold_math_functions_abs() {
        test_same_value("Math.abs(Math.random())");

        test_value("Math.abs('-1')", "1");
        test_value("Math.abs(-2)", "2");
        test_value("Math.abs(null)", "0");
        test_value("Math.abs('')", "0");
        test_value("Math.abs(NaN)", "NaN");
        test_value("Math.abs(-0)", "0");
        test_value("Math.abs(-Infinity)", "Infinity");
        // TODO
        // test_value("Math.abs([])", "0");
        // test_value("Math.abs([2])", "2");
        // test_value("Math.abs([1,2])", "NaN");
        test_value("Math.abs({})", "NaN");
        test_value("Math.abs('string');", "NaN");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_imul() {
        test_same_value("Math.imul(Math.random(),2)");
        test_value("Math.imul(-1,1)", "-1");
        test_value("Math.imul(2,2)", "4");
        test_value("Math.imul(2)", "0");
        test_value("Math.imul(2,3,5)", "6");
        test_value("Math.imul(0xfffffffe, 5)", "-10");
        test_value("Math.imul(0xffffffff, 5)", "-5");
        test_value("Math.imul(0xfffffffffffff34f, 0xfffffffffff342)", "13369344");
        test_value("Math.imul(0xfffffffffffff34f, -0xfffffffffff342)", "-13369344");
        test_value("Math.imul(NaN, 2)", "0");
    }

    #[test]
    fn test_fold_math_functions_ceil() {
        test_same_value("Math.ceil(Math.random())");

        test_value("Math.ceil(1)", "1");
        test_value("Math.ceil(1.5)", "2");
        test_value("Math.ceil(1.3)", "2");
        test_value("Math.ceil(-1.3)", "-1");
    }

    #[test]
    fn test_fold_math_functions_floor() {
        test_same_value("Math.floor(Math.random())");

        test_value("Math.floor(1)", "1");
        test_value("Math.floor(1.5)", "1");
        test_value("Math.floor(1.3)", "1");
        test_value("Math.floor(-1.3)", "-2");
    }

    #[test]
    fn test_fold_math_functions_fround() {
        test_same_value("Math.fround(Math.random())");

        test_value("Math.fround(NaN)", "NaN");
        test_value("Math.fround(Infinity)", "Infinity");
        test_value("Math.fround(-Infinity)", "-Infinity");
        test_value("Math.fround(1)", "1");
        test_value("Math.fround(0)", "0");
        test_value("Math.fround(16777217)", "16777216");
        test_value("Math.fround(16777218)", "16777218");
    }

    #[test]
    fn test_fold_math_functions_fround_j2cl() {
        test_same_value("Math.fround(1.2)");
    }

    #[test]
    fn test_fold_math_functions_round() {
        test_same_value("Math.round(Math.random())");
        test_value("Math.round(NaN)", "NaN");
        test_value("Math.round(3)", "3");
        test_value("Math.round(3.5)", "4");
        test_value("Math.round(-3.5)", "-3");
    }

    #[test]
    fn test_fold_math_functions_sign() {
        test_same_value("Math.sign(Math.random())");
        test_value("Math.sign(NaN)", "NaN");
        test_value("Math.sign(0.0)", "0");
        test_value("Math.sign(-0.0)", "-0");
        test_value("Math.sign(0.01)", "1");
        test_value("Math.sign(-0.01)", "-1");
        test_value("Math.sign(3.5)", "1");
        test_value("Math.sign(-3.5)", "-1");
    }

    #[test]
    fn test_fold_math_functions_trunc() {
        test_same_value("Math.trunc(Math.random())");
        test_value("Math.sign(NaN)", "NaN");
        test_value("Math.trunc(3.5)", "3");
        test_value("Math.trunc(-3.5)", "-3");
        test_value("Math.trunc(0.5)", "0");
        test_value("Math.trunc(-0.5)", "-0");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_clz32() {
        test("Math.clz32(0)", "32");
        let mut x = 1;
        for i in (0..=31).rev() {
            test(&format!("{x}.leading_zeros()"), &i.to_string());
            test(&format!("{}.leading_zeros()", 2 * x - 1), &i.to_string());
            x *= 2;
        }
        test("Math.clz32('52')", "26");
        test("Math.clz32([52])", "26");
        test("Math.clz32([52, 53])", "32");

        // Overflow cases
        test("Math.clz32(0x100000000)", "32");
        test("Math.clz32(0x100000001)", "31");

        // NaN -> 0
        test("Math.clz32(NaN)", "32");
        test("Math.clz32('foo')", "32");
        test("Math.clz32(Infinity)", "32");
    }

    #[test]
    fn test_fold_math_functions_max() {
        test_same_value("Math.max(Math.random(), 1)");

        test_value("Math.max()", "-Infinity");
        test_value("Math.max(0)", "0");
        test_value("Math.max(0, 1)", "1");
        test_value("Math.max(0, 1, -1, 200)", "200");
        test_value("Math.max(0, -1, -Infinity)", "0");
        test_value("Math.max(0, -1, -Infinity, NaN)", "NaN");
        test_value("Math.max(0, -0)", "0");
        test_value("Math.max(-0, 0)", "0");
        test_same_value("Math.max(...a, 1)");
    }

    #[test]
    fn test_fold_math_functions_min() {
        test_same_value("Math.min(Math.random(), 1)");

        test_value("Math.min()", "Infinity");
        test_value("Math.min(3)", "3");
        test_value("Math.min(0, 1)", "0");
        test_value("Math.min(0, 1, -1, 200)", "-1");
        test_value("Math.min(0, -1, -Infinity)", "-Infinity");
        test_value("Math.min(0, -1, -Infinity, NaN)", "NaN");
        test_value("Math.min(0, -0)", "-0");
        test_value("Math.min(-0, 0)", "-0");
        test_same_value("Math.min(...a, 1)");
    }

    #[test]
    #[ignore]
    fn test_fold_math_functions_pow() {
        test("Math.pow(1, 2)", "1");
        test("Math.pow(2, 0)", "1");
        test("Math.pow(2, 2)", "4");
        test("Math.pow(2, 32)", "4294967296");
        test("Math.pow(Infinity, 0)", "1");
        test("Math.pow(Infinity, 1)", "Infinity");
        test("Math.pow('a', 33)", "NaN");
    }

    #[test]
    #[ignore]
    fn test_fold_number_functions_is_safe_integer() {
        test("Number.isSafeInteger(1)", "true");
        test("Number.isSafeInteger(1.5)", "false");
        test("Number.isSafeInteger(9007199254740991)", "true");
        test("Number.isSafeInteger(9007199254740992)", "false");
        test("Number.isSafeInteger(-9007199254740991)", "true");
        test("Number.isSafeInteger(-9007199254740992)", "false");
    }

    #[test]
    #[ignore]
    fn test_fold_number_functions_is_finite() {
        test("Number.isFinite(1)", "true");
        test("Number.isFinite(1.5)", "true");
        test("Number.isFinite(NaN)", "false");
        test("Number.isFinite(Infinity)", "false");
        test("Number.isFinite(-Infinity)", "false");
        test_same("Number.isFinite('a')");
    }

    #[test]
    #[ignore]
    fn test_fold_number_functions_is_nan() {
        test("Number.isNaN(1)", "false");
        test("Number.isNaN(1.5)", "false");
        test("Number.isNaN(NaN)", "true");
        test_same("Number.isNaN('a')");
        // unknown function may have side effects
        test_same("Number.isNaN(+(void unknown()))");
    }

    #[test]
    #[ignore]
    fn test_fold_parse_numbers() {
        // Template Strings
        test_same("x = parseInt(`123`)");
        test_same("x = parseInt(` 123`)");
        test_same("x = parseInt(`12 ${a}`)");
        test_same("x = parseFloat(`1.23`)");

        // setAcceptedLanguage(LanguageMode.ECMASCRIPT5);

        test("x = parseInt('123')", "x = 123");
        test("x = parseInt(' 123')", "x = 123");
        test("x = parseInt('123', 10)", "x = 123");
        test("x = parseInt('0xA')", "x = 10");
        test("x = parseInt('0xA', 16)", "x = 10");
        test("x = parseInt('07', 8)", "x = 7");
        test("x = parseInt('08')", "x = 8");
        test("x = parseInt('0')", "x = 0");
        test("x = parseInt('-0')", "x = -0");
        test("x = parseFloat('0')", "x = 0");
        test("x = parseFloat('1.23')", "x = 1.23");
        test("x = parseFloat('-1.23')", "x = -1.23");
        test("x = parseFloat('1.2300')", "x = 1.23");
        test("x = parseFloat(' 0.3333')", "x = 0.3333");
        test("x = parseFloat('0100')", "x = 100");
        test("x = parseFloat('0100.000')", "x = 100");

        // Mozilla Dev Center test cases
        test("x = parseInt(' 0xF', 16)", "x = 15");
        test("x = parseInt(' F', 16)", "x = 15");
        test("x = parseInt('17', 8)", "x = 15");
        test("x = parseInt('015', 10)", "x = 15");
        test("x = parseInt('1111', 2)", "x = 15");
        test("x = parseInt('12', 13)", "x = 15");
        test("x = parseInt(15.99, 10)", "x = 15");
        test("x = parseInt(-15.99, 10)", "x = -15");
        // Java's Integer.parseInt("-15.99", 10) throws an exception, because of the decimal point.
        test_same("x = parseInt('-15.99', 10)");
        test("x = parseFloat('3.14')", "x = 3.14");
        test("x = parseFloat(3.14)", "x = 3.14");
        test("x = parseFloat(-3.14)", "x = -3.14");
        test("x = parseFloat('-3.14')", "x = -3.14");
        test("x = parseFloat('-0')", "x = -0");

        // Valid calls - unable to fold
        test_same("x = parseInt('FXX123', 16)");
        test_same("x = parseInt('15*3', 10)");
        test_same("x = parseInt('15e2', 10)");
        test_same("x = parseInt('15px', 10)");
        test_same("x = parseInt('-0x08')");
        test_same("x = parseInt('1', -1)");
        test_same("x = parseFloat('3.14more non-digit characters')");
        test_same("x = parseFloat('314e-2')");
        test_same("x = parseFloat('0.0314E+2')");
        test_same("x = parseFloat('3.333333333333333333333333')");

        // Invalid calls
        test_same("x = parseInt('0xa', 10)");
        test_same("x = parseInt('')");

        // setAcceptedLanguage(LanguageMode.ECMASCRIPT3);
        test_same("x = parseInt('08')");
    }

    #[test]
    #[ignore]
    fn test_fold_parse_octal_numbers() {
        // setAcceptedLanguage(LanguageMode.ECMASCRIPT5);

        test("x = parseInt('021', 8)", "x = 17");
        test("x = parseInt('-021', 8)", "x = -17");
    }

    #[test]
    #[ignore]
    fn test_replace_with_char_at() {
        // enableTypeCheck();
        // replaceTypesWithColors();
        // disableCompareJsDoc();

        fold_string_typed("a.substring(0, 1)", "a.charAt(0)");
        test_same_string_typed("a.substring(-4, -3)");
        test_same_string_typed("a.substring(i, j + 1)");
        test_same_string_typed("a.substring(i, i + 1)");
        test_same_string_typed("a.substring(1, 2, 3)");
        test_same_string_typed("a.substring()");
        test_same_string_typed("a.substring(1)");
        test_same_string_typed("a.substring(1, 3, 4)");
        test_same_string_typed("a.substring(-1, 3)");
        test_same_string_typed("a.substring(2, 1)");
        test_same_string_typed("a.substring(3, 1)");

        fold_string_typed("a.slice(4, 5)", "a.charAt(4)");
        test_same_string_typed("a.slice(-2, -1)");
        fold_string_typed("var /** number */ i; a.slice(0, 1)", "var /** number */ i; a.charAt(0)");
        test_same_string_typed("a.slice(i, j + 1)");
        test_same_string_typed("a.slice(i, i + 1)");
        test_same_string_typed("a.slice(1, 2, 3)");
        test_same_string_typed("a.slice()");
        test_same_string_typed("a.slice(1)");
        test_same_string_typed("a.slice(1, 3, 4)");
        test_same_string_typed("a.slice(-1, 3)");
        test_same_string_typed("a.slice(2, 1)");
        test_same_string_typed("a.slice(3, 1)");

        // enableTypeCheck();

        test_same("function f(/** ? */ a) { a.substring(0, 1); }");
        // test_same(lines(
        //     "/** @constructor */ function A() {};",
        //     "A.prototype.substring = function(begin, end) {};",
        //     "function f(/** !A */ a) { a.substring(0, 1); }",
        // ));
        // test_same(lines(
        //     "/** @constructor */ function A() {};",
        //     "A.prototype.slice = function(begin, end) {};",
        //     "function f(/** !A */ a) { a.slice(0, 1); }",
        // ));

        // useTypes = false;
        test_same_string_typed("a.substring(0, 1)");
        test_same_string_typed("''.substring(i, i + 1)");
    }

    #[test]
    fn test_fold_concat_chaining() {
        // array
        test("x = [1,2].concat(1).concat(2,['abc']).concat('abc')", "x = [1,2,1,2,'abc','abc']");
        test("x = [].concat(['abc']).concat(1).concat([2,3])", "x = ['abc',1,2,3]");

        test("var x, y; [1].concat(x).concat(y)", "var x, y; [1].concat(x, y)");
        test("var y; [1].concat(x).concat(y)", "var y; [1].concat(x, y)"); // x might have a getter that updates y, but that side effect is preserved correctly
        test("var x; [1].concat(x.a).concat(x)", "var x; [1].concat(x.a, x)"); // x.a might have a getter that updates x, but that side effect is preserved correctly

        // string
        test("'1'.concat(1).concat(2,['abc']).concat('abc')", "'1'.concat(1,2,['abc'],'abc')");
        test("''.concat(['abc']).concat(1).concat([2,3])", "''.concat(['abc'],1,[2,3])");
        test_same("''.concat(1)");

        test("var x, y; ''.concat(x).concat(y)", "var x, y; ''.concat(x, y)");
        test("var y; ''.concat(x).concat(y)", "var y; ''.concat(x, y)"); // x might have a getter that updates y, but that side effect is preserved correctly
        test("var x; ''.concat(x.a).concat(x)", "var x; ''.concat(x.a, x)"); // x.a might have a getter that updates x, but that side effect is preserved correctly

        // other
        test("x = []['concat'](1)", "x = [1]");
        test_same("obj.concat([1,2]).concat(1)");
    }

    #[test]
    fn test_remove_array_literal_from_front_of_concat() {
        test("x = [].concat([1,2,3],1)", "x = [1,2,3,1]");

        test_same("[1,2,3].concat(foo())");
        // Call method with the same name as Array.prototype.concat
        test_same("obj.concat([1,2,3])");

        test("x = [].concat(1,[1,2,3])", "x = [1,1,2,3]");
        test("x = [].concat(1)", "x = [1]");
        test("x = [].concat([1])", "x = [1]");

        // Chained folding of empty array lit
        test("x = [].concat([], [1,2,3], [4])", "x = [1,2,3,4]");
        test("x = [].concat([]).concat([1]).concat([2,3])", "x = [1,2,3]");

        test("x = [].concat(1, x)", "x = [1].concat(x)"); // x might be an array or an object with `Symbol.isConcatSpreadable`
        test("x = [].concat(1, ...x)", "x = [1].concat(...x)");
        test_same("x = [].concat(x, 1)");
    }

    #[test]
    fn test_array_of_spread() {
        // Here, since our tests are fully opened, the dce may automatically optimize it into a simple array, instead of simply substitute the function call.
        test("x = Array.of(...['a', 'b', 'c'])", "x = ['a', 'b', 'c']");
        test("x = Array.of(...['a', 'b', 'c',])", "x = ['a', 'b', 'c']");
        test("x = Array.of(...['a'], ...['b', 'c'])", "x = ['a', 'b', 'c']");
        test("x = Array.of('a', ...['b', 'c'])", "x = ['a', 'b', 'c']");
        test("x = Array.of('a', ...['b', 'c'])", "x = ['a', 'b', 'c']");
    }

    #[test]
    fn test_array_of_no_spread() {
        test("x = Array.of('a', 'b', 'c')", "x = ['a', 'b', 'c']");
        test("x = Array.of('a', ['b', 'c'])", "x = ['a', ['b', 'c']]");
        test("x = Array.of('a', ['b', 'c'],)", "x = ['a', ['b', 'c']]");
    }

    #[test]
    fn test_array_of_no_args() {
        test("x = Array.of()", "x = []");
    }

    #[test]
    fn test_array_of_no_change() {
        test_same("x = Array.of.apply(window, ['a', 'b', 'c'])");
        test_same("x = ['a', 'b', 'c']");
        test_same("x = [Array.of, 'a', 'b', 'c']");
    }

    #[test]
    fn test_fold_array_bug() {
        test_same("Array[123]()");
    }

    fn test_same_string_typed(js: &str) {
        fold_string_typed(js, js);
    }

    fn fold_string_typed(js: &str, expected: &str) {
        let left = "function f(/** string */ a) {".to_string() + js + "}";
        let right = "function f(/** string */ a) {".to_string() + expected + "}";
        test(left.as_str(), right.as_str());
    }

    #[test]
    fn test_fold_string_from_char_code() {
        test("x = String.fromCharCode()", "x = ''");
        test("x = String.fromCharCode(0)", "x = '\\0'");
        test("x = String.fromCharCode(120)", "x = 'x'");
        test("x = String.fromCharCode(120, 121)", "x = 'xy'");
        test_same("String.fromCharCode(55358, 56768)");
        test("x = String.fromCharCode(0x10000)", "x = '\\0'");
        test("x = String.fromCharCode(0x10078, 0x10079)", "x = 'xy'");
        test("x = String.fromCharCode(0x1_0000_FFFF)", "x = '\u{ffff}'");
        test("x = String.fromCharCode(NaN)", "x = '\\0'");
        test("x = String.fromCharCode(-Infinity)", "x = '\\0'");
        test("x = String.fromCharCode(Infinity)", "x = '\\0'");
        test("x = String.fromCharCode(null)", "x = '\\0'");
        test("x = String.fromCharCode(undefined)", "x = '\\0'");
        test("x = String.fromCharCode('123')", "x = '{'");
        test_same("String.fromCharCode(x)");
        test("x = String.fromCharCode('x')", "x = '\\0'");
        test("x = String.fromCharCode('0.5')", "x = '\\0'");

        test_same("x = Unknown.fromCharCode('0.5')");
    }

    #[test]
    fn test_to_string() {
        test("x = false['toString']()", "x = 'false';");
        test("x = false.toString()", "x = 'false';");
        test("x = true.toString()", "x = 'true';");
        test("x = 'xy'.toString()", "x = 'xy';");
        test("x = 0 .toString()", "x = '0';");
        test("x = 123 .toString()", "x = '123';");
        test("x = NaN.toString()", "x = 'NaN';");
        test("x = NaN.toString(2)", "x = 'NaN';");
        test("x = Infinity.toString()", "x = 'Infinity';");
        test("x = Infinity.toString(2)", "x = 'Infinity';");
        test("x = (-Infinity).toString(2)", "x = '-Infinity';");
        test("x = 1n.toString()", "x = '1'");
        test_same("254n.toString(16);"); // unimplemented
                                         // test("/a\\\\b/ig.toString()", "'/a\\\\\\\\b/ig';");
        test_same("null.toString()"); // type error

        test("x = 100 .toString(0)", "x = 100 .toString(0)");
        test("x = 100 .toString(1)", "x = 100 .toString(1)");
        test("x = 100 .toString(2)", "x = '1100100'");
        test("x = 100 .toString(5)", "x = '400'");
        test("x = 100 .toString(8)", "x = '144'");
        test("x = 100 .toString(13)", "x = '79'");
        test("x = 100 .toString(16)", "x = '64'");
        test("x = 10000 .toString(19)", "x = '18d6'");
        test("x = 10000 .toString(23)", "x = 'iki'");
        test("x = 1000000 .toString(29)", "x = '1c01m'");
        test("x = 1000000 .toString(31)", "x = '12hi2'");
        test("x = 1000000 .toString(36)", "x = 'lfls'");
        test("x = 0 .toString(36)", "x = '0'");
        test("x = 0.5.toString()", "x = '0.5'");

        test("false.toString(b)", "(!1).toString(b)");
        test("true.toString(b)", "(!0).toString(b)");
        test("'xy'.toString(b)", "'xy'.toString(b)");
        test("123 .toString(b)", "123 .toString(b)");
        test("1e99.toString(b)", "1e99.toString(b)");
        test("/./.toString(b)", "/./.toString(b)");
    }

    #[test]
    fn test_fold_pow() {
        test("Math.pow(2, 3)", "2 ** 3");
        test("Math.pow(a, 3)", "+(a) ** 3");
        test("Math.pow(2, b)", "2 ** +b");
        test("Math.pow(a, b)", "+(a) ** +b");
        test("Math.pow(2n, 3n)", "+(2n) ** +3n"); // errors both before and after
        test("Math.pow(a + b, c)", "+(a + b) ** +c");
        test_same("Math.pow()");
        test_same("Math.pow(1)");
        test_same("Math.pow(...a, 1)");
        test_same("Math.pow(1, ...a)");
        test_same("Math.pow(1, 2, 3)");
        test_es2015("Math.pow(2, 3)", "Math.pow(2, 3)");
        test_same("Unknown.pow(1, 2)");
    }

    #[test]
    fn test_fold_roots() {
        test_same("v = Math.sqrt()");
        test_same("v = Math.sqrt(1, 2)");
        test_same("v = Math.sqrt(...a)");
        test_same("v = Math.sqrt(a)"); // a maybe -0
        test_same("v = Math.sqrt(2n)");
        test("v = Math.sqrt(Infinity)", "v = Infinity");
        test("v = Math.sqrt(NaN)", "v = NaN");
        test("v = Math.sqrt(0)", "v = 0");
        test("v = Math.sqrt(-0)", "v = -0");
        test("v = Math.sqrt(-1)", "v = NaN");
        test("v = Math.sqrt(-Infinity)", "v = NaN");
        test("v = Math.sqrt(1)", "v = 1");
        test("v = Math.sqrt(4)", "v = 2");
        test_same("v = Math.sqrt(2)");
        test("v = Math.cbrt(1)", "v = 1");
        test("v = Math.cbrt(8)", "v = 2");
        test_same("v = Math.cbrt(2)");
        test_same("Unknown.sqrt(1)");
        test_same("Unknown.cbrt(1)");
    }

    #[test]
    fn test_number_constants() {
        test("v = Number.POSITIVE_INFINITY", "v = Infinity");
        test("v = Number.NEGATIVE_INFINITY", "v = -Infinity");
        test("v = Number.NaN", "v = NaN");
        test("v = Number.MAX_SAFE_INTEGER", "v = 2**53-1");
        test("v = Number.MIN_SAFE_INTEGER", "v = -(2**53-1)");
        test("v = Number.EPSILON", "v = 2**-52");

        test_same("Number.POSITIVE_INFINITY = 1");
        test_same("Number.NEGATIVE_INFINITY = 1");
        test_same("Number.NaN = 1");
        test_same("Number.MAX_SAFE_INTEGER = 1");
        test_same("Number.MIN_SAFE_INTEGER = 1");
        test_same("Number.EPSILON = 1");

        test_es2015("v = Number.MAX_SAFE_INTEGER", "v = 9007199254740991");
        test_es2015("v = Number.MIN_SAFE_INTEGER", "v = -9007199254740991");
        test_es2015("v = Number.EPSILON", "v = Number.EPSILON");
    }
}
