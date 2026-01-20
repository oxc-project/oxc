use std::borrow::Cow;

use cow_utils::CowUtils;

use oxc_allocator::{Box, TakeIn};
use oxc_ast::{NONE, ast::*};
use oxc_compat::ESFeature;
use oxc_ecmascript::{
    StringCharAt, StringCharAtResult, ToBigInt, ToIntegerIndex,
    constant_evaluation::{ConstantEvaluation, DetermineValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_regular_expression::{
    RegexUnsupportedPatterns, has_unsupported_regular_expression_pattern,
};
use oxc_span::SPAN;
use oxc_traverse::Ancestor;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

type Arguments<'a> = oxc_allocator::Vec<'a, Argument<'a>>;

/// Minimize With Known Methods
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeReplaceKnownMethods.java>
impl<'a> PeepholeOptimizations {
    pub fn replace_known_global_methods(node: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::CallExpression(ce) = node else { return };

        // Use constant evaluation for known method calls
        if let Some(constant_value) = ce.evaluate_value(ctx) {
            ctx.state.changed = true;
            *node = ctx.value_to_expr(ce.span, constant_value);
            return;
        }

        // Handle special cases not suitable for constant evaluation
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
            "concat" => Self::try_fold_concat(*span, arguments, callee, ctx),
            "pow" => Self::try_fold_pow(*span, arguments, object, ctx),
            "of" => Self::try_fold_array_of(*span, arguments, name, object, ctx),
            _ => None,
        };
        if let Some(replacement) = replacement {
            ctx.state.changed = true;
            *node = replacement;
        }
    }

    /// `Math.pow(a, b)` -> `+(a) ** +b`
    fn try_fold_pow(
        span: Span,
        arguments: &mut Arguments<'a>,
        object: &Expression<'a>,
        ctx: &Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if !ctx.supports_feature(ESFeature::ES2016ExponentiationOperator) {
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
            if expr.value_type(ctx).is_number() {
                expr.take_in(ctx.ast)
            } else {
                ctx.ast.expression_unary(SPAN, UnaryOperator::UnaryPlus, expr.take_in(ctx.ast))
            }
        };

        Some(ctx.ast.expression_binary(
            span,
            // see [`PeepholeOptimizations::is_binary_operator_that_does_number_conversion`] why it does not require `wrap_with_unary_plus_if_needed` here
            first_arg.take_in(ctx.ast),
            BinaryOperator::Exponential,
            wrap_with_unary_plus_if_needed(second_arg),
        ))
    }

    fn try_fold_array_of(
        span: Span,
        arguments: &mut Arguments<'a>,
        name: &str,
        object: &Expression<'a>,
        ctx: &Ctx<'a, '_>,
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
        ))
    }

    /// `[].concat(a).concat(b)` -> `[].concat(a, b)`
    /// `"".concat(a).concat(b)` -> `"".concat(a, b)`
    pub fn replace_concat_chain(node: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let original_span = if let Expression::CallExpression(root_call_expr) = node {
            root_call_expr.span
        } else {
            return;
        };

        if matches!(ctx.parent(), Ancestor::StaticMemberExpressionObject(member) if member.property().name == "concat")
        {
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
            new_root_callee.take_in(ctx.ast),
            NONE,
            ctx.ast.vec_from_iter(
                collected_arguments.into_iter().rev().flat_map(|arg| arg.take_in(ctx.ast)),
            ),
            false,
        );
        ctx.state.changed = true;
    }

    /// `[].concat(1, 2)` -> `[1, 2]`
    /// `"".concat(a, "b")` -> "`${a}b`"
    fn try_fold_concat(
        span: Span,
        args: &mut Arguments<'a>,
        callee: &mut Expression<'a>,
        ctx: &Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        // let concat chaining reduction handle it first
        if let Ancestor::StaticMemberExpressionObject(parent_member) = ctx.parent()
            && parent_member.property().name.as_str() == "concat"
        {
            return None;
        }

        let object = match callee {
            Expression::StaticMemberExpression(member) => &mut member.object,
            Expression::ComputedMemberExpression(member) => &mut member.object,
            _ => unreachable!(),
        };
        match object {
            Expression::ArrayExpression(array_expr) => {
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
                    Some(object.take_in(ctx.ast))
                } else if can_merge_until.is_some() {
                    Some(ctx.ast.expression_call(
                        span,
                        callee.take_in(ctx.ast),
                        NONE,
                        args.take_in(ctx.ast),
                        false,
                    ))
                } else {
                    None
                }
            }
            Expression::StringLiteral(base_str) => {
                if !ctx.supports_feature(ESFeature::ES2015TemplateLiterals)
                    || args.is_empty()
                    || !args.iter().all(Argument::is_expression)
                {
                    return None;
                }

                let expression_count =
                    args.iter().filter(|arg| !matches!(arg, Argument::StringLiteral(_))).count();
                let string_count = args.len() - expression_count;

                // whether it is shorter to use `String::concat`
                if ".concat()".len() + args.len() + "''".len() * string_count
                    < "${}".len() * expression_count
                {
                    return None;
                }

                let mut quasi_strs: Vec<Cow<'a, str>> =
                    vec![Cow::Borrowed(base_str.value.as_str())];
                let mut expressions = ctx.ast.vec_with_capacity(expression_count);
                let mut pushed_quasi = true;
                for argument in args.drain(..) {
                    if let Argument::StringLiteral(str_lit) = argument {
                        if pushed_quasi {
                            let last_quasi = quasi_strs
                                .last_mut()
                                .expect("last element should exist because pushed_quasi is true");
                            last_quasi.to_mut().push_str(&str_lit.value);
                        } else {
                            quasi_strs.push(Cow::Borrowed(str_lit.value.as_str()));
                        }
                        pushed_quasi = true;
                    } else {
                        if !pushed_quasi {
                            // need a pair
                            quasi_strs.push(Cow::Borrowed(""));
                        }
                        // checked that all the arguments are expression above
                        expressions.push(argument.into_expression());
                        pushed_quasi = false;
                    }
                }
                if !pushed_quasi {
                    quasi_strs.push(Cow::Borrowed(""));
                }

                if expressions.is_empty() {
                    debug_assert_eq!(quasi_strs.len(), 1);
                    return Some(ctx.ast.expression_string_literal(
                        span,
                        ctx.ast.atom_from_cow(&quasi_strs.pop().unwrap()),
                        None,
                    ));
                }

                let mut quasis = ctx.ast.vec_from_iter(quasi_strs.into_iter().map(|s| {
                    let cooked = ctx.ast.atom_from_cow(&s);
                    ctx.ast.template_element(
                        SPAN,
                        TemplateElementValue {
                            raw: ctx.ast.atom(&Self::escape_string_for_template_literal(&s)),
                            cooked: Some(cooked),
                        },
                        false,
                        false, // raw is already escaped by escape_string_for_template_literal
                    )
                }));
                if let Some(last_quasi) = quasis.last_mut() {
                    last_quasi.tail = true;
                }

                debug_assert_eq!(quasis.len(), expressions.len() + 1);
                Some(ctx.ast.expression_template_literal(span, quasis, expressions))
            }
            _ => None,
        }
    }

    pub fn escape_string_for_template_literal(s: &str) -> Cow<'_, str> {
        if s.contains(['\\', '`', '$', '\r']) {
            Cow::Owned(
                s.cow_replace("\\", "\\\\")
                    .cow_replace("`", "\\`")
                    .cow_replace("$", "\\$")
                    .cow_replace("\r\n", "\\r\n")
                    .into_owned(),
            )
        } else {
            Cow::Borrowed(s)
        }
    }

    pub fn replace_known_property_access(node: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        // property access should be kept to keep `this` value
        if matches!(
            ctx.parent(),
            Ancestor::CallExpressionCallee(_) | Ancestor::TaggedTemplateExpressionTag(_)
        ) {
            return;
        }

        let (name, object, span) = match node {
            Expression::StaticMemberExpression(member) if !member.optional => {
                let span = member.span;
                (member.property.name.as_str(), &mut member.object, span)
            }
            Expression::ComputedMemberExpression(member) if !member.optional => {
                match &member.expression {
                    Expression::StringLiteral(s) => {
                        let span = member.span;
                        (s.value.as_str(), &mut member.object, span)
                    }
                    Expression::NumericLiteral(n) => {
                        if let Some(integer_index) = n.value.to_integer_index() {
                            let span = member.span;
                            if let Some(replacement) = Self::try_fold_integer_index_access(
                                &mut member.object,
                                integer_index,
                                span,
                                ctx,
                            ) {
                                ctx.state.changed = true;
                                *node = replacement;
                            }
                        }
                        return;
                    }
                    Expression::BigIntLiteral(b) => {
                        if !b.is_negative()
                            && let Some(integer_index) =
                                b.to_big_int(ctx).and_then(ToIntegerIndex::to_integer_index)
                        {
                            let span = member.span;
                            if let Some(replacement) = Self::try_fold_integer_index_access(
                                &mut member.object,
                                integer_index,
                                span,
                                ctx,
                            ) {
                                ctx.state.changed = true;
                                *node = replacement;
                            }
                        }
                        return;
                    }
                    _ => return,
                }
            }
            _ => return,
        };

        let replacement = match object {
            Expression::Identifier(ident) => {
                if !ctx.is_global_reference(ident) {
                    return;
                }
                match ident.name.as_str() {
                    "Number" => Self::try_fold_number_constants(name, span, ctx),
                    _ => None,
                }
            }
            Expression::RegExpLiteral(regex) => match name {
                "source" => {
                    const ES2015_UNSUPPORTED_FLAGS: RegExpFlags = RegExpFlags::G
                        .union(RegExpFlags::I)
                        .union(RegExpFlags::M)
                        .union(RegExpFlags::S)
                        .union(RegExpFlags::Y)
                        .complement();
                    const ES2015_UNSUPPORTED_PATTERNS: RegexUnsupportedPatterns =
                        RegexUnsupportedPatterns {
                            look_behind_assertions: true,
                            named_capture_groups: true,
                            unicode_property_escapes: true,
                            pattern_modifiers: true,
                        };

                    if regex.regex.pattern.pattern.is_none()
                        && let Ok(pattern) = regex.parse_pattern(ctx.ast.allocator)
                    {
                        regex.regex.pattern.pattern = Some(Box::new_in(pattern, ctx.ast.allocator));
                    }
                    if let Some(pattern) = &regex.regex.pattern.pattern
                        // for now, only replace regexes that are supported by ES2015 to preserve the syntax error
                        // we can check whether each feature is supported for the target range to improve this
                        && regex.regex.flags.intersection(ES2015_UNSUPPORTED_FLAGS).is_empty()
                        && !has_unsupported_regular_expression_pattern(
                            pattern,
                            &ES2015_UNSUPPORTED_PATTERNS,
                        )
                    {
                        Some(ctx.ast.expression_string_literal(
                            span,
                            regex.regex.pattern.text,
                            None,
                        ))
                    } else {
                        // the pattern might be invalid, keep it as-is to preserve the error
                        None
                    }
                }
                _ => None,
            },
            _ => return,
        };
        if let Some(replacement) = replacement {
            ctx.state.changed = true;
            *node = replacement;
        }
    }

    /// replace `Number.*` constants
    fn try_fold_number_constants(
        name: &str,
        span: Span,
        ctx: &Ctx<'a, '_>,
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
                if ctx.supports_feature(ESFeature::ES2016ExponentiationOperator) {
                    // 2**53 - 1
                    pow_with_expr(span, 2.0, 53.0, BinaryOperator::Subtraction, 1.0)
                } else {
                    num(span, 2.0f64.powi(53) - 1.0)
                }
            }
            "MIN_SAFE_INTEGER" => {
                if ctx.supports_feature(ESFeature::ES2016ExponentiationOperator) {
                    // -(2**53 - 1)
                    ctx.ast.expression_unary(
                        span,
                        UnaryOperator::UnaryNegation,
                        pow_with_expr(SPAN, 2.0, 53.0, BinaryOperator::Subtraction, 1.0),
                    )
                } else {
                    num(span, -(2.0f64.powi(53) - 1.0))
                }
            }
            "EPSILON" => {
                if !ctx.supports_feature(ESFeature::ES2016ExponentiationOperator) {
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

    /// Compress `"abc"[0]` to `"a"` and `[0,1,2][1]` to `1`
    fn try_fold_integer_index_access(
        object: &mut Expression<'a>,
        property: u32,
        span: Span,
        ctx: &Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if object.may_have_side_effects(ctx) {
            return None;
        }

        match object {
            Expression::StringLiteral(s) => {
                if let StringCharAtResult::Value(c) =
                    s.value.as_str().char_at(Some(property.into()))
                {
                    s.span = span;
                    s.value = ctx.ast.atom(&c.to_string());
                    s.raw = None;
                    Some(object.take_in(ctx.ast))
                } else {
                    None
                }
            }
            Expression::ArrayExpression(array_expr) => {
                let length_until_spread =
                    array_expr.elements.iter().take_while(|el| !el.is_spread()).count();
                if (property as usize) < length_until_spread {
                    match &array_expr.elements[property as usize] {
                        ArrayExpressionElement::SpreadElement(_) => unreachable!(),
                        ArrayExpressionElement::Elision(_) => Some(ctx.ast.void_0(span)),
                        match_expression!(ArrayExpressionElement) => {
                            let element = array_expr.elements.swap_remove(property as usize);
                            Some(element.into_expression())
                        }
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn validate_global_reference(expr: &Expression<'a>, target: &str, ctx: &Ctx<'a, '_>) -> bool {
        let Expression::Identifier(ident) = expr else { return false };
        ctx.is_global_reference(ident) && ident.name == target
    }

    fn validate_arguments(args: &Arguments, expected_len: usize) -> bool {
        (args.len() == expected_len) && args.iter().all(Argument::is_expression)
    }
}
