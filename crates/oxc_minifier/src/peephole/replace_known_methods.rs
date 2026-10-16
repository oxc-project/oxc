use std::{borrow::Cow, fmt::Write, mem};

use cow_utils::CowUtils;

use oxc_allocator::{ArenaBox, ArenaVec, GetAllocator, TakeIn};
use oxc_ast::{ast::*, builder::AstBuilder};
use oxc_compat::{ESFeature, EngineTargets};
use oxc_ecmascript::{
    GlobalContext, ToBigInt, ToIntegerIndex,
    constant_evaluation::{ConstantEvaluation, ConstantValue, DetermineValueType, ValueType},
    side_effects::{
        MayHaveSideEffects, MayHaveSideEffectsContext, PropertyReadSideEffects,
        is_regexp_syntax_supported,
    },
};
use oxc_span::SPAN;
use oxc_str::{JSStr, JSStrBuilder};
use oxc_syntax::reference::ReferenceId;

use crate::{TraverseCtx, generated::ancestor::Ancestor};

use super::PeepholeOptimizations;

type Arguments<'a> = ArenaVec<'a, Argument<'a>>;

/// Minimize With Known Methods
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeReplaceKnownMethods.java>
impl<'a> PeepholeOptimizations {
    pub fn replace_known_global_methods(node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::CallExpression(ce) = node else { return };

        // Use constant evaluation for known method calls
        if let Some(constant_value) = ce.evaluate_value(ctx) {
            let new_expr = ctx.value_to_expr(ce.span, constant_value);
            ctx.replace_expression(node, new_expr);
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
                    Expression::StringLiteral(s) => {
                        let Some(name) = s.value.as_str() else { return };
                        (name, &member.object)
                    }
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
            ctx.replace_expression(node, replacement);
        }
    }

    /// `Math.pow(a, b)` -> `+(a) ** +b`
    fn try_fold_pow(
        span: Span,
        arguments: &mut Arguments<'a>,
        object: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !ctx.supports_feature(ESFeature::ES2016ExponentiationOperator) {
            return None;
        }
        if !Self::validate_global_reference(object, "Math", ctx)
            || !Self::validate_arguments(arguments, 2)
        {
            return None;
        }

        let second_arg = arguments.pop().expect("checked len above").into_expression();
        let first_arg = arguments.pop().expect("checked len above").into_expression();

        let wrap_with_unary_plus_if_needed = |expr: Expression<'a>| {
            if expr.value_type(ctx).is_number() {
                expr
            } else {
                Expression::new_unary_expression(SPAN, UnaryOperator::UnaryPlus, expr, ctx)
            }
        };

        Some(Expression::new_binary_expression(
            span,
            // see [`PeepholeOptimizations::is_binary_operator_that_does_number_conversion`] why it does not require `wrap_with_unary_plus_if_needed` here
            first_arg,
            BinaryOperator::Exponential,
            wrap_with_unary_plus_if_needed(second_arg),
            ctx,
        ))
    }

    fn try_fold_array_of(
        span: Span,
        arguments: &mut Arguments<'a>,
        name: &str,
        object: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !Self::validate_global_reference(object, "Array", ctx) {
            return None;
        }
        if name != "of" {
            return None;
        }
        Some(Expression::new_array_expression(
            span,
            ArenaVec::from_iter_in(arguments.drain(..).map(ArrayExpressionElement::from), ctx),
            ctx,
        ))
    }

    /// `[].concat(a).concat(b)` -> `[].concat(a, b)`
    /// `"".concat(a).concat(b)` -> `"".concat(a, b)`
    /// `"".concat(a).concat(b())` -> `` `${a}${b()}` ``
    pub fn replace_concat_chain(node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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
        let mut collected_arguments = ArenaVec::new_in(ctx);
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

            // Merging the calls moves the evaluation of later arguments before the earlier calls
            // read their own arguments. For example, if we have `[].concat(a).concat(b)`, the
            // steps before the compression are:
            // 1. evaluate `a`
            // 2. `[].concat(a)` reads `a` and creates `[a]`
            // 3. evaluate `b`
            // 4. `.concat(b)` creates `[a, b]`
            //
            // The steps after the compression (`[].concat(a, b)`) are:
            // 1. evaluate `a`
            // 2. evaluate `b`
            // 3. `[].concat(a, b)` reads `a` and creates `[a, b]`
            //
            // `concat_calls_keep_order` below decides when step 2 can move after step 3.
            // The error thrown when the created array is too long can move,
            // because of the compressor assumption.

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

        let base_string = match &*new_root_callee {
            Expression::StaticMemberExpression(member) => match &member.object {
                Expression::StringLiteral(base) => Some(base.value),
                _ => None,
            },
            _ => None,
        };
        let arguments_in_order = || collected_arguments.iter().rev().flat_map(|args| args.iter());
        let calls_in_order = collected_arguments.iter().rev().map(|args| &**args);
        if !Self::concat_calls_keep_order(calls_in_order, base_string.is_some(), ctx) {
            // Calls in a chain read their arguments one call at a time, like a template literal
            // coerces one substitution at a time. So the chain folds to a template when every
            // call keeps the order on its own: `"".concat(a).concat(b())` -> `${a}${b()}`.
            if let Some(base) = base_string
                && ctx.supports_feature(ESFeature::ES2015TemplateLiterals)
                && collected_arguments.iter().all(|args| Self::concat_keeps_order(args.iter(), ctx))
            {
                let expression_count = arguments_in_order()
                    .filter(|arg| !matches!(arg, Argument::StringLiteral(_)))
                    .count();
                let argument_count = arguments_in_order().count();
                if Self::is_concat_shorter(
                    collected_arguments.len(),
                    argument_count,
                    expression_count,
                ) {
                    return;
                }
                let arguments = collected_arguments.into_iter().rev().flat_map(|arg| arg.drain(..));
                let template =
                    Self::concat_template(original_span, base, arguments, expression_count, ctx);
                ctx.replace_expression(node, template);
            }
            return;
        }

        let new_expr = Expression::new_call_expression(
            original_span,
            new_root_callee.take_in(ctx),
            None,
            ArenaVec::from_iter_in(
                collected_arguments.into_iter().rev().flat_map(|arg| arg.take_in(ctx)),
                ctx,
            ),
            false,
            ctx,
        );
        ctx.replace_expression(node, new_expr);
    }

    /// `[].concat(1, 2)` -> `[1, 2]`
    /// `"".concat(a, "b")` -> "`${a}b`"
    fn try_fold_concat(
        span: Span,
        args: &mut Arguments<'a>,
        callee: &mut Expression<'a>,
        ctx: &TraverseCtx<'a>,
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
                    Some(object.take_in(ctx))
                } else if can_merge_until.is_some() {
                    Some(Expression::new_call_expression(
                        span,
                        callee.take_in(ctx),
                        None,
                        args.take_in(ctx),
                        false,
                        ctx,
                    ))
                } else {
                    None
                }
            }
            Expression::StringLiteral(base_str) => {
                if !ctx.supports_feature(ESFeature::ES2015TemplateLiterals)
                    || args.is_empty()
                    || !Self::concat_keeps_order(args.iter(), ctx)
                {
                    return None;
                }

                let expression_count =
                    args.iter().filter(|arg| !matches!(arg, Argument::StringLiteral(_))).count();
                if Self::is_concat_shorter(1, args.len(), expression_count) {
                    return None;
                }
                Some(Self::concat_template(
                    span,
                    base_str.value,
                    args.drain(..),
                    expression_count,
                    ctx,
                ))
            }
            _ => None,
        }
    }

    /// Whether the arguments of one `String.prototype.concat` call can be coerced in evaluation
    /// order, one at a time, as a template literal does.
    ///
    /// A call evaluates every argument before `concat` coerces any. Coercion is pure by
    /// assumption, so the orders differ only when an argument evaluated after a coerced value has
    /// side effects that could change that value, as in `''.concat(arr, arr.pop())`.
    fn concat_keeps_order<'b>(
        arguments: impl IntoIterator<Item = &'b Argument<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> bool
    where
        'a: 'b,
    {
        let ctx = WritesHaveSideEffects(ctx);
        let mut read = false;
        for argument in arguments {
            let Some(expression) = argument.as_expression() else {
                return false;
            };
            if read && expression.may_have_side_effects(&ctx) {
                return false;
            }
            read |= Self::concat_reads(expression, true, &ctx);
        }
        true
    }

    /// Whether merging chained `concat` calls into one keeps every read before the side effects
    /// of later calls.
    ///
    /// A call evaluates all of its arguments before it reads any, so merging only delays the reads
    /// of earlier calls past the evaluation of later calls' arguments. A spread argument counts
    /// as both, since it iterates when evaluated and its elements are read afterwards.
    fn concat_calls_keep_order<'b>(
        calls: impl IntoIterator<Item = &'b Arguments<'a>>,
        is_string: bool,
        ctx: &TraverseCtx<'a>,
    ) -> bool
    where
        'a: 'b,
    {
        let ctx = WritesHaveSideEffects(ctx);
        let mut read = false;
        for arguments in calls {
            let mut expressions = arguments.iter().map(Argument::as_expression);
            if read && expressions.clone().any(|e| e.is_none_or(|e| e.may_have_side_effects(&ctx)))
            {
                return false;
            }
            read |= expressions.any(|e| e.is_none_or(|e| Self::concat_reads(e, is_string, &ctx)));
        }
        true
    }

    /// Whether `concat` reads state through an argument after evaluating it: coercing it to a
    /// string, or reading its elements. A primitive value has no state to read. An array literal
    /// is a fresh array that no other argument can reach, but coercing it reads its elements.
    fn concat_reads(
        expression: &Expression<'a>,
        is_string: bool,
        ctx: &impl GlobalContext<'a>,
    ) -> bool {
        if expression.is_literal()
            || !is_string && matches!(expression, Expression::ArrayExpression(_))
        {
            return false;
        }
        let value_type = expression.value_type(ctx);
        value_type.is_object() || value_type.is_undetermined()
    }

    /// Whether `call_count` chained `concat` calls with these arguments are shorter than the
    /// template literal that replaces them.
    ///
    /// This counts a comma after every argument but the last of the chain, which favors the
    /// template by one byte for a single call.
    fn is_concat_shorter(
        call_count: usize,
        argument_count: usize,
        expression_count: usize,
    ) -> bool {
        let string_count = argument_count - expression_count;
        ".concat()".len() * call_count + argument_count + "''".len() * string_count
            < "${}".len() * expression_count + (call_count - 1)
    }

    /// Build the template literal for `base.concat(...arguments)`,
    /// folding string literal arguments into the quasis.
    fn concat_template(
        span: Span,
        base: JSStr<'a>,
        arguments: impl IntoIterator<Item = Argument<'a>>,
        expression_count: usize,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let ast = &ctx.ast;
        let mut expressions = ArenaVec::with_capacity_in(expression_count, ast);
        let mut quasis = ArenaVec::with_capacity_in(expression_count + 1, ast);
        let mut cooked = JSStrBuilder::new_in(ctx);
        cooked.push_js_str(base);
        for argument in arguments {
            if let Argument::StringLiteral(lit) = argument {
                cooked.push_js_str(lit.value);
            } else {
                let quasi = mem::replace(&mut cooked, JSStrBuilder::new_in(ctx));
                quasis.push(Self::template_element(quasi.into_js_str(), false, ast));
                expressions.push(argument.into_expression());
            }
        }
        let tail = cooked.into_js_str();
        if expressions.is_empty() {
            return Expression::new_string_literal(span, tail, None, ast);
        }
        quasis.push(Self::template_element(tail, true, ast));
        Expression::new_template_literal(span, quasis, expressions, ast)
    }

    /// A template quasi holding `cooked`.
    /// The raw text borrows the cooked value when nothing needs escaping.
    fn template_element(
        cooked: JSStr<'a>,
        tail: bool,
        ast: &AstBuilder<'a>,
    ) -> TemplateElement<'a> {
        let raw = Str::from_cow_in(&Self::escape_string_for_template_literal(cooked), ast);
        TemplateElement::new(SPAN, TemplateElementValue { raw, cooked: Some(cooked) }, tail, ast)
    }

    /// Escape a string value as template literal raw text.
    /// Raw text is source, so a lone surrogate is written as a `\u` escape.
    pub fn escape_string_for_template_literal(value: JSStr<'_>) -> Cow<'_, str> {
        let Some(s) = value.as_str() else {
            let mut raw = String::with_capacity(value.len());
            for c in value.chars() {
                match c.to_char() {
                    Some(c @ ('\\' | '`' | '$')) => {
                        raw.push('\\');
                        raw.push(c);
                    }
                    Some('\r') => raw.push_str("\\r"),
                    Some(c) => raw.push(c),
                    None => {
                        let _ = write!(raw, "\\u{:04x}", c.to_u32());
                    }
                }
            }
            return Cow::Owned(raw);
        };
        if s.contains(['\\', '`', '$', '\r']) {
            Cow::Owned(
                s.cow_replace("\\", "\\\\")
                    .cow_replace("`", "\\`")
                    .cow_replace("$", "\\$")
                    .cow_replace('\r', "\\r")
                    .into_owned(),
            )
        } else {
            Cow::Borrowed(s)
        }
    }

    pub fn replace_known_property_access(node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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
                        let Some(name) = s.value.as_str() else { return };
                        (name, &mut member.object, span)
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
                                ctx.replace_expression(node, replacement);
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
                                ctx.replace_expression(node, replacement);
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
                    if regex.regex.pattern.pattern.is_none()
                        && let Ok(pattern) = regex.parse_pattern(ctx.allocator())
                    {
                        regex.regex.pattern.pattern = Some(ArenaBox::new_in(pattern, ctx));
                    }
                    if let Some(pattern) = &regex.regex.pattern.pattern
                        && is_regexp_syntax_supported(
                            pattern,
                            regex.regex.flags.to_inline_string().as_str(),
                            ctx,
                        )
                    {
                        Some(Expression::new_string_literal(
                            span,
                            regex.regex.pattern.text,
                            None,
                            ctx,
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
            ctx.replace_expression(node, replacement);
        }
    }

    /// replace `Number.*` constants
    fn try_fold_number_constants(
        name: &str,
        span: Span,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let num = |span: Span, n: f64| {
            Expression::new_numeric_literal(span, n, None, NumberBase::Decimal, ctx)
        };
        // [neg] base ** exponent [op] a
        let pow_with_expr =
            |span: Span, base: f64, exponent: f64, op: BinaryOperator, a: f64| -> Expression<'a> {
                Expression::new_binary_expression(
                    span,
                    Expression::new_binary_expression(
                        SPAN,
                        num(SPAN, base),
                        BinaryOperator::Exponential,
                        num(SPAN, exponent),
                        ctx,
                    ),
                    op,
                    num(SPAN, a),
                    ctx,
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
                    Expression::new_unary_expression(
                        span,
                        UnaryOperator::UnaryNegation,
                        pow_with_expr(SPAN, 2.0, 53.0, BinaryOperator::Subtraction, 1.0),
                        ctx,
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
                Expression::new_binary_expression(
                    span,
                    num(SPAN, 2.0),
                    BinaryOperator::Exponential,
                    num(SPAN, -52.0),
                    ctx,
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
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if object.may_have_side_effects(ctx) {
            return None;
        }

        match object {
            Expression::StringLiteral(s) => {
                let index = property as usize;
                // std's UTF-8 decoder is faster for ordinary strings.
                let unit = match s.value.as_str() {
                    Some(value) => value.encode_utf16().nth(index),
                    None => s.value.encode_utf16().nth(index),
                }?;
                let mut builder = JSStrBuilder::new_in(ctx);
                builder.push_code_unit(unit);
                s.span = span;
                s.value = builder.into_js_str();
                s.raw = None;
                Some(object.take_in(ctx))
            }
            Expression::ArrayExpression(array_expr) => {
                let length_until_spread =
                    array_expr.elements.iter().take_while(|el| !el.is_spread()).count();
                if (property as usize) < length_until_spread {
                    match &array_expr.elements[property as usize] {
                        ArrayExpressionElement::SpreadElement(_) => unreachable!(),
                        ArrayExpressionElement::Elision(_) => {
                            Some(Expression::new_void_0(span, ctx))
                        }
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

    fn validate_global_reference(
        expr: &Expression<'a>,
        target: &str,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        let Expression::Identifier(ident) = expr else { return false };
        ctx.is_global_reference(ident) && ident.name == target
    }

    fn validate_arguments(args: &Arguments, expected_len: usize) -> bool {
        (args.len() == expected_len) && args.iter().all(Argument::is_expression)
    }
}

/// A side-effect context that counts every property write as a side effect.
///
/// `propertyWriteSideEffects: false` asserts that a write runs no setter, not that it leaves the
/// object unchanged, and a later write can change a value that `concat` reads afterwards.
struct WritesHaveSideEffects<'b, 'a>(&'b TraverseCtx<'a>);

impl<'a> GlobalContext<'a> for WritesHaveSideEffects<'_, 'a> {
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> bool {
        self.0.is_global_reference(reference)
    }

    fn get_constant_value_for_reference_id(
        &self,
        reference_id: ReferenceId,
    ) -> Option<ConstantValue<'a>> {
        self.0.get_constant_value_for_reference_id(reference_id)
    }

    fn value_type_for_reference_id(&self, reference_id: ReferenceId) -> Option<ValueType> {
        self.0.value_type_for_reference_id(reference_id)
    }
}

impl<'a> MayHaveSideEffectsContext<'a> for WritesHaveSideEffects<'_, 'a> {
    fn engine_targets(&self) -> Option<&EngineTargets> {
        self.0.engine_targets()
    }

    fn annotations(&self) -> bool {
        self.0.annotations()
    }

    fn manual_pure_functions(&self, callee: &Expression) -> bool {
        self.0.manual_pure_functions(callee)
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        self.0.property_read_side_effects()
    }

    fn property_write_side_effects(&self) -> bool {
        true
    }

    fn unknown_global_side_effects(&self) -> bool {
        self.0.unknown_global_side_effects()
    }
}
