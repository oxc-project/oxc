use std::{borrow::Cow, fmt::Write, mem};

use cow_utils::CowUtils;

use oxc_allocator::{ArenaBox, ArenaVec, GetAllocator, StringBuilder, TakeIn};
use oxc_ast::{ast::*, builder::AstBuilder};
use oxc_compat::ESFeature;
use oxc_ecmascript::{
    ArrayJoin, ToBigInt, ToIntegerIndex, ToJsString,
    constant_evaluation::{ConstantEvaluation, DetermineValueType, ValueType},
    side_effects::{MayHaveSideEffects, is_regexp_syntax_supported},
};
use oxc_span::SPAN;
use oxc_str::{JSStr, JSStrBuilder};

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
            "join" => Self::try_fold_array_join(*span, arguments, object, ctx),
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

            // For arrays, creating the intermediate result can throw when it is too long.
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

        let Expression::StaticMemberExpression(member) = &*new_root_callee else { unreachable!() };
        // Convert the chain directly when each call can interleave its own argument
        // evaluations and coercions. Keeping the calls' coercion boundaries permits
        // later calls to mutate earlier arguments, even when flattening is unsafe.
        if let Expression::StringLiteral(base) = &member.object
            && ctx.supports_feature(ESFeature::ES2015TemplateLiterals)
            && collected_arguments
                .iter()
                .all(|args| Self::string_concat_can_interleave(args.iter(), ctx))
        {
            let argument_count = collected_arguments.iter().map(|args| args.len()).sum::<usize>();
            let expression_count = collected_arguments
                .iter()
                .flat_map(|args| args.iter())
                .filter(|arg| !matches!(arg, Argument::StringLiteral(_)))
                .count();
            let string_count = argument_count - expression_count;
            // If flattening is unsafe, the alternative is the whole original chain.
            let call_count = if Self::string_concat_can_interleave(
                collected_arguments.iter().rev().flat_map(|args| args.iter()),
                ctx,
            ) {
                1
            } else {
                collected_arguments.len()
            };
            if ".concat()".len() * call_count + argument_count + "''".len() * string_count
                >= "${}".len() * expression_count
            {
                let replacement = Self::fold_string_concat(
                    original_span,
                    base.value,
                    collected_arguments.into_iter().rev().flat_map(|args| args.take_in(ctx)),
                    expression_count,
                    ctx,
                );
                ctx.replace_expression(node, replacement);
                return;
            }
        }
        if matches!(member.object, Expression::StringLiteral(_))
            && !Self::string_concat_can_interleave(
                collected_arguments.iter().rev().flat_map(|args| args.iter()),
                ctx,
            )
        {
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
                    || !Self::string_concat_can_interleave(args.iter(), ctx)
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

                Some(Self::fold_string_concat(
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

    /// Build a template after checking that each original call can interleave its arguments.
    fn fold_string_concat(
        span: Span,
        base: JSStr<'a>,
        arguments: impl Iterator<Item = Argument<'a>>,
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

    /// `concat` evaluates every argument before coercing any of them. Templates interleave
    /// evaluation and coercion, and flattening a chain delays the inner calls' coercions.
    /// An earlier object's coercion can observe mutations made by a later argument.
    fn string_concat_can_interleave<'b>(
        arguments: impl Iterator<Item = &'b Argument<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> bool
    where
        'a: 'b,
    {
        let mut may_coerce_object = false;
        for argument in arguments {
            let Some(argument) = argument.as_expression() else { return false };
            if may_coerce_object && argument.may_have_side_effects(ctx) {
                return false;
            }
            may_coerce_object |=
                matches!(argument.value_type(ctx), ValueType::Object | ValueType::Undetermined);
        }
        true
    }

    /// `['a','b'].join()` -> `'a,b'`
    fn try_fold_array_join(
        span: Span,
        arguments: &Arguments<'a>,
        object: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let Expression::ArrayExpression(array) = object else { return None };
        if array.may_have_side_effects(ctx) || arguments.len() > 1 {
            return None;
        }

        let separator = if let Some(argument) = arguments.first() {
            let separator = argument.as_expression()?;
            if separator.may_have_side_effects(ctx) {
                return None;
            }
            if separator.value_type(ctx).is_undefined() {
                None
            } else {
                Some(separator.to_js_string(ctx)?)
            }
        } else {
            None
        };
        let separator = separator.as_deref().unwrap_or(",");

        let per_element = array.elements.len().saturating_sub(1) * 3;
        let syntax_savings = per_element + "[].join()".len();

        let added_size = separator.len() * (array.elements.len().saturating_sub(2));

        if syntax_savings <= added_size {
            return None;
        }

        let joined = array.array_join(ctx, Some(separator))?;
        Some(Expression::new_string_literal(span, Str::from_str_in(&joined, ctx), None, ctx))
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

    /// Join untagged template raw text without forming an interpolation or a decimal escape
    /// across a boundary. Empty parts do not separate the neighboring source text.
    pub(super) fn concat_template_literal_raw<const N: usize>(
        mut parts: [&str; N],
        ctx: &impl GetAllocator<'a>,
    ) -> Str<'a> {
        let mut suffixes = [""; N];
        for index in 0..N {
            let Some(next) =
                parts[index + 1..].iter().find_map(|part| part.as_bytes().first().copied())
            else {
                break;
            };
            let raw = parts[index];
            if raw.ends_with('$') && next == b'{' {
                let backslashes =
                    raw[..raw.len() - 1].bytes().rev().take_while(|&byte| byte == b'\\').count();
                if backslashes % 2 == 0 {
                    // Escape the opening brace, leaving any preceding backslashes unchanged.
                    suffixes[index] = "\\";
                }
            } else if raw.ends_with("\\0") && next.is_ascii_digit() {
                let prefix = &raw[..raw.len() - 1];
                let backslashes = prefix.bytes().rev().take_while(|&byte| byte == b'\\').count();
                if backslashes % 2 == 1 {
                    parts[index] = prefix;
                    suffixes[index] = "x00";
                }
            }
        }
        if suffixes.iter().all(|suffix| suffix.is_empty()) {
            return Str::from_strs_array_in(parts, ctx);
        }
        let capacity = parts
            .iter()
            .zip(suffixes)
            .try_fold(0usize, |length, (part, suffix)| {
                length.checked_add(part.len())?.checked_add(suffix.len())
            })
            .expect("template literal raw text is too long");
        let mut raw = StringBuilder::with_capacity_in(capacity, ctx.allocator());
        for (part, suffix) in parts.into_iter().zip(suffixes) {
            raw.push_str(part);
            raw.push_str(suffix);
        }
        Str::from(raw.into_str())
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
