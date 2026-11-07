use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, ComputedMemberExpression, Expression, SpreadElement},
};
use oxc_ast_visit::{Visit, walk::walk_expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::{could_be_asi_hazard, is_method_call, outermost_paren_parent},
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{is_same_expression, pad_fix_with_token_boundary},
};

fn warn() -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use useless `undefined`.")
        .with_help("Consider removing `undefined` or using `null` instead.")
}

fn no_useless_undefined_diagnostic(span: Span) -> OxcDiagnostic {
    warn().with_label(span)
}

fn no_useless_undefined_diagnostic_spans(spans: Vec<Span>) -> OxcDiagnostic {
    warn().with_labels(spans)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoUselessUndefined {
    /// Whether to check for useless `undefined` in function call arguments.
    check_arguments: bool,
    /// Whether to check for useless `undefined` in arrow function bodies.
    check_arrow_function_body: bool,
}

impl Default for NoUselessUndefined {
    fn default() -> Self {
        Self { check_arguments: true, check_arrow_function_body: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents usage of `undefined` in cases where it would be useless.
    ///
    /// ::: warning
    /// This rule can conflict with the default behaviors of the `eslint/array-callback-return`
    /// and `eslint/getter-return` rules. For both rules, you can set
    /// the `allowImplicit` option to avoid conflicts.
    /// :::
    ///
    /// ### Why is this bad?
    ///
    /// `undefined` is the default value for new variables, parameters,
    /// return statements, etc, so specifying `undefined` in these cases
    /// is pointless.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let foo = undefined;
    /// const noop = () => undefined;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let foo;
    /// const noop = () => {};
    /// ```
    NoUselessUndefined,
    unicorn,
    pedantic,
    recommended,
    fix_suggestion,
    config = NoUselessUndefined,
    version = "0.6.1",
    short_description = "Disallow useless `undefined`.",
);

// Create a static set for all function names
static FUNCTION_NAMES: &[&str] = &[
    "add",
    // `React.createContext(undefined)`
    "createContext",
    "equal",
    "has",
    "include",
    "includes",
    "is",
    "not",
    "notEqual",
    "notPropertyVal",
    "notSame",
    "notStrictEqual",
    "property",
    "propertyVal",
    "push",
    // https://vuejs.org/api/reactivity-core.html#ref
    "ref",
    "same",
    "set",
    "strictEqual",
    "strictNotSame",
    "strictSame",
    "toBe",
    "toContain",
    "toContainEqual",
    "toEqual",
    "toHaveBeenCalledWith",
    "unshift",
    // `React.useRef(undefined)`
    "useRef",
];

fn is_match_ignore_func_name(name: &str) -> bool {
    // Check if the name is in the static set
    FUNCTION_NAMES.contains(&name)
        // `setState(undefined)`
        || name.starts_with("set")
}

fn should_ignore(callee: &Expression) -> bool {
    match callee {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            is_match_ignore_func_name(name)
        }
        Expression::StaticMemberExpression(static_assertions) => {
            let name = static_assertions.property.name.as_str();
            is_match_ignore_func_name(name)
        }
        _ => false,
    }
}

fn is_function_bind_call(call_expr: &CallExpression) -> bool {
    !call_expr.optional && is_method_call(call_expr, None, Some(&["bind"]), None, None)
}

fn is_has_function_return_type(node: &AstNode, ctx: &LintContext<'_>) -> bool {
    match node.kind() {
        AstKind::Program(_) => false,
        AstKind::ArrowFunctionExpression(arrow_func_express) => {
            arrow_func_express.return_type.is_some()
        }
        AstKind::Function(func) => func.return_type.is_some(),
        _ => is_has_function_return_type(ctx.nodes().parent_node(node.id()), ctx),
    }
}

impl Rule for NoUselessUndefined {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ConditionalExpression(conditional) => {
                let is_undefined = |expression: &Expression| {
                    matches!(expression.without_parentheses(), Expression::Identifier(ident)
                        if ident.is_undefined() && ctx.is_reference_to_global_variable(ident))
                };
                let consequent_undefined = is_undefined(&conditional.consequent);
                if consequent_undefined == is_undefined(&conditional.alternate) {
                    return;
                }
                let (undefined, branch) = if consequent_undefined {
                    (&conditional.consequent, &conditional.alternate)
                } else {
                    (&conditional.alternate, &conditional.consequent)
                };
                let Expression::ComputedMemberExpression(access) = branch.get_inner_expression()
                else {
                    return;
                };
                if access.optional
                    || indexed_access_test_validity(&conditional.test, access, ctx)
                        != Some(!consequent_undefined)
                    || !is_safe_indexed_access_part(&access.object)
                    || !is_safe_indexed_access_part(&access.expression)
                {
                    return;
                }
                let diagnostic = no_useless_undefined_diagnostic(undefined.span())
                    .with_help("Use the indexed access directly.");
                if ctx.has_comments_between(conditional.span) || is_reference_context(node, ctx) {
                    ctx.diagnostic(diagnostic);
                    return;
                }
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    let mut replacement = fixer.source_range(branch.span()).to_string();
                    if (replacement.starts_with('<')
                        || matches!(
                            access.object.get_inner_expression(),
                            Expression::ObjectExpression(_) | Expression::FunctionExpression(_)
                        ))
                        && !replacement.starts_with('(')
                    {
                        replacement = format!("({replacement})");
                    }
                    if replacement.starts_with(['[', '(', '/', '+', '-', '`'])
                        && could_be_asi_hazard(node, ctx)
                    {
                        replacement.insert(0, ';');
                    }
                    pad_fix_with_token_boundary(
                        ctx.source_text(),
                        conditional.span,
                        &mut replacement,
                    );
                    fixer.replace(conditional.span, replacement)
                });
            }
            AstKind::IdentifierReference(undefined_literal) if undefined_literal.is_undefined() => {
                let Some(parent_node) = outermost_paren_parent(node, ctx) else {
                    return;
                };
                let parent_node_kind = parent_node.kind();

                match parent_node_kind {
                    // `return undefined`
                    AstKind::ReturnStatement(ret_stmt) => {
                        if is_has_function_return_type(parent_node, ctx) {
                            return;
                        }
                        ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(undefined_literal.span),
                            |fixer| {
                                let delete_span = if let Some(comment) = ctx
                                    .comments_range(ret_stmt.span.start..ret_stmt.span.end)
                                    .next_back()
                                {
                                    Span::new(comment.span.end, undefined_literal.span.end)
                                } else {
                                    Span::new(ret_stmt.span().start + 6, undefined_literal.span.end)
                                };
                                fixer.delete_range(delete_span)
                            },
                        );
                    }
                    // `yield undefined`
                    AstKind::YieldExpression(yield_expr) => {
                        if yield_expr.delegate {
                            return;
                        }
                        ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(undefined_literal.span),
                            |fixer| fixer.replace(yield_expr.span, "yield"),
                        );
                    }
                    // `() => undefined`
                    AstKind::ArrowFunctionExpression(arrow) if arrow.is_expression() => {
                        if !self.check_arrow_function_body {
                            return;
                        }

                        if is_has_function_return_type(parent_node, ctx) {
                            return;
                        }

                        ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(undefined_literal.span),
                            |fixer| fixer.replace(arrow.body.span(), "{}"),
                        );
                    }
                    // `let foo = undefined` / `var foo = undefined`
                    AstKind::VariableDeclarator(variable_declarator) => {
                        let grand_parent_node = ctx.nodes().parent_node(parent_node.id());
                        let grand_parent_node_kind = grand_parent_node.kind();
                        let AstKind::VariableDeclaration(declaration) = grand_parent_node_kind
                        else {
                            return;
                        };
                        if declaration.kind.is_const() {
                            return;
                        }
                        if is_has_function_return_type(parent_node, ctx) {
                            return;
                        }
                        ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(undefined_literal.span),
                            |fixer| {
                                fixer.delete_range(Span::new(
                                    variable_declarator.id.span().end,
                                    undefined_literal.span.end,
                                ))
                            },
                        );
                    }
                    // `const {foo = undefined} = {}`
                    AstKind::AssignmentPattern(assign_pattern) => {
                        let left = &assign_pattern.left;
                        let delete_span = Span::new(left.span().end, undefined_literal.span.end);
                        if is_has_function_return_type(parent_node, ctx) {
                            return;
                        }
                        ctx.diagnostic_with_fix(
                            no_useless_undefined_diagnostic(undefined_literal.span),
                            |fixer| fixer.delete_range(delete_span),
                        );
                    }
                    // `function foo(bar = undefined) {}`
                    AstKind::FormalParameter(assign_pattern) => {
                        if let Some(initializer) = &assign_pattern.initializer
                            && initializer.span() == undefined_literal.span
                        {
                            let formal_parameters = ctx.nodes().parent_node(parent_node.id());
                            let AstKind::FormalParameters(formal_parameters) =
                                formal_parameters.kind()
                            else {
                                return;
                            };

                            // Removing the default initializer makes this parameter required. Do
                            // not produce a TypeScript-invalid signature by placing it after an
                            // optional parameter.
                            let has_preceding_optional_parameter = formal_parameters
                                .items
                                .iter()
                                .take_while(|parameter| parameter.span != assign_pattern.span)
                                .any(|parameter| parameter.optional);
                            if has_preceding_optional_parameter {
                                return;
                            }

                            let left = &assign_pattern
                                .type_annotation
                                .as_ref()
                                .map_or(assign_pattern.pattern.span().end, |type_annotation| {
                                    type_annotation.span.end
                                });
                            let delete_span = Span::new(*left, undefined_literal.span.end);
                            if is_has_function_return_type(parent_node, ctx) {
                                return;
                            }
                            ctx.diagnostic_with_fix(
                                no_useless_undefined_diagnostic(undefined_literal.span),
                                |fixer| fixer.delete_range(delete_span),
                            );
                        }
                    }
                    _ => {}
                }
            }
            AstKind::CallExpression(call_expr) => {
                if !self.check_arguments {
                    return;
                }

                let arguments = &call_expr.arguments;
                if !arguments
                    .last()
                    .and_then(Argument::as_expression)
                    .is_some_and(Expression::is_undefined)
                {
                    return;
                }

                if should_ignore(&call_expr.callee) {
                    return;
                }

                // Ignore arguments in `Function#bind()`, but not `this` argument
                if is_function_bind_call(call_expr) && arguments.len() != 1 {
                    return;
                }
                let mut undefined_args_spans = Vec::new();
                for i in (0..arguments.len()).rev() {
                    let arg = &arguments[i];
                    if arg.as_expression().is_some_and(Expression::is_undefined) {
                        let span = arg.span();
                        undefined_args_spans.push(span);
                    } else {
                        break;
                    }
                }

                if undefined_args_spans.is_empty() {
                    return;
                }
                undefined_args_spans.reverse();
                let first_undefined_span = undefined_args_spans[0];
                let last_undefined_span = undefined_args_spans[undefined_args_spans.len() - 1];
                let mut start = first_undefined_span.start;
                let mut end = last_undefined_span.end;

                let remaining_count = arguments.len() - undefined_args_spans.len();

                if remaining_count > 0 {
                    let previous_argument = &arguments[remaining_count - 1];
                    start = previous_argument.span().end;
                }
                // If all arguments removed, and there is trailing comma, we need remove it.
                if remaining_count == 0 {
                    end = call_expr.span.end - 1;
                }

                let delete_span = Span::new(start, end);
                ctx.diagnostic_with_fix(
                    no_useless_undefined_diagnostic_spans(undefined_args_spans),
                    |fixer| fixer.delete_range(delete_span),
                );
            }
            _ => {}
        }
    }
}

fn static_number(expression: &Expression) -> Option<f64> {
    match expression.get_inner_expression() {
        Expression::NumericLiteral(number) => Some(number.value),
        Expression::UnaryExpression(unary) => match unary.operator {
            UnaryOperator::UnaryPlus => static_number(&unary.argument),
            UnaryOperator::UnaryNegation => static_number(&unary.argument).map(|value| -value),
            _ => None,
        },
        _ => None,
    }
}

fn is_safe_integer(number: f64) -> bool {
    number.fract() == 0.0 && number.abs() <= 9_007_199_254_740_991.0
}

#[expect(clippy::float_cmp, reason = "Numeric literal values must match exactly")]
fn is_same_indexed_access_reference(
    left: &Expression,
    right: &Expression,
    ctx: &LintContext,
) -> bool {
    let left = left.get_inner_expression();
    let right = right.get_inner_expression();
    match (left, right) {
        (Expression::NumericLiteral(left), Expression::NumericLiteral(right)) => {
            return left.value == right.value;
        }
        (Expression::BigIntLiteral(left), Expression::BigIntLiteral(right)) => {
            return left.value == right.value;
        }
        _ => {}
    }
    (left.is_literal()
        || matches!(
            left,
            Expression::Identifier(_) | Expression::ThisExpression(_) | Expression::Super(_)
        )
        || matches!(left, Expression::TemplateLiteral(template) if template.expressions.is_empty()))
        && is_same_expression(left, right, ctx)
}

#[expect(clippy::float_cmp, reason = "Bounds and offsets are validated as safe integers")]
fn indexed_access_test_validity(
    test: &Expression,
    access: &ComputedMemberExpression,
    ctx: &LintContext,
) -> Option<bool> {
    let Expression::BinaryExpression(test) = test.get_inner_expression() else {
        return None;
    };
    let mut operator = test.operator;
    if !operator.is_compare() {
        return None;
    }
    let index = access.expression.get_inner_expression();
    let (index, offset) = match index {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Subtraction => {
            match static_number(&binary.right) {
                Some(offset) if is_safe_integer(offset) && offset > 0.0 => {
                    (binary.left.get_inner_expression(), offset)
                }
                _ => (index, 0.0),
            }
        }
        _ => (index, 0.0),
    };
    let left = test.left.get_inner_expression();
    let right = test.right.get_inner_expression();
    let boundary = if is_same_indexed_access_reference(left, index, ctx) {
        right
    } else if is_same_indexed_access_reference(right, index, ctx) {
        operator = match operator {
            BinaryOperator::LessThan => BinaryOperator::GreaterThan,
            BinaryOperator::LessEqualThan => BinaryOperator::GreaterEqualThan,
            BinaryOperator::GreaterThan => BinaryOperator::LessThan,
            BinaryOperator::GreaterEqualThan => BinaryOperator::LessEqualThan,
            _ => return None,
        };
        left
    } else {
        return None;
    };
    if let Some(boundary) = static_number(boundary).filter(|value| is_safe_integer(*value)) {
        return match operator {
            BinaryOperator::GreaterEqualThan if boundary == offset => Some(true),
            BinaryOperator::GreaterThan if boundary == offset - 1.0 => Some(true),
            BinaryOperator::LessThan if boundary == offset => Some(false),
            BinaryOperator::LessEqualThan if boundary == offset - 1.0 => Some(false),
            _ => None,
        };
    }
    if offset != 0.0 {
        return None;
    }
    let (length, minus_one) = match boundary {
        Expression::BinaryExpression(binary)
            if binary.operator == BinaryOperator::Subtraction
                && static_number(&binary.right) == Some(1.0) =>
        {
            (binary.left.get_inner_expression(), true)
        }
        _ => (boundary, false),
    };
    let Expression::StaticMemberExpression(length) = length else {
        return None;
    };
    if length.optional
        || length.property.name != "length"
        || !is_same_indexed_access_reference(
            length.object.get_inner_expression(),
            access.object.get_inner_expression(),
            ctx,
        )
    {
        return None;
    }
    match (operator, minus_one) {
        (BinaryOperator::LessThan, false) | (BinaryOperator::LessEqualThan, true) => Some(true),
        (BinaryOperator::GreaterEqualThan, false) | (BinaryOperator::GreaterThan, true) => {
            Some(false)
        }
        _ => None,
    }
}

fn is_safe_indexed_access_part(expression: &Expression) -> bool {
    struct OptionalChainVisitor {
        found: bool,
    }
    impl<'a> Visit<'a> for OptionalChainVisitor {
        fn visit_expression(&mut self, expression: &Expression<'a>) {
            if self.found {
                return;
            }
            if matches!(expression, Expression::ChainExpression(_)) {
                self.found = true;
            } else {
                walk_expression(self, expression);
            }
        }
    }

    struct SafetyVisitor {
        safe: bool,
    }
    impl<'a> Visit<'a> for SafetyVisitor {
        fn visit_expression(&mut self, expression: &Expression<'a>) {
            if !self.safe {
                return;
            }
            match expression {
                Expression::CallExpression(_)
                | Expression::ImportExpression(_)
                | Expression::V8IntrinsicExpression(_)
                | Expression::NewExpression(_)
                | Expression::TaggedTemplateExpression(_)
                | Expression::AssignmentExpression(_)
                | Expression::UpdateExpression(_)
                | Expression::AwaitExpression(_)
                | Expression::YieldExpression(_)
                | Expression::ComputedMemberExpression(_)
                | Expression::StaticMemberExpression(_)
                | Expression::PrivateFieldExpression(_)
                | Expression::ChainExpression(_)
                | Expression::ClassExpression(_)
                | Expression::JSXElement(_)
                | Expression::JSXFragment(_) => self.safe = false,
                Expression::UnaryExpression(unary) if unary.operator.is_delete() => {
                    self.safe = false;
                }
                Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
                    let mut visitor = OptionalChainVisitor { found: false };
                    visitor.visit_expression(expression);
                    self.safe = !visitor.found;
                }
                _ => walk_expression(self, expression),
            }
        }

        fn visit_spread_element(&mut self, _: &SpreadElement<'a>) {
            self.safe = false;
        }
    }
    let mut visitor = SafetyVisitor { safe: true };
    visitor.visit_expression(expression);
    visitor.safe
}

fn is_reference_context(node: &AstNode, ctx: &LintContext) -> bool {
    let mut expression = node;
    for parent in ctx.nodes().ancestors(node.id()) {
        match parent.kind() {
            AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSInstantiationExpression(_) => expression = parent,
            AstKind::CallExpression(call) => return call.callee.span() == expression.span(),
            AstKind::TaggedTemplateExpression(tagged) => {
                return tagged.tag.span() == expression.span();
            }
            AstKind::UnaryExpression(unary) => return unary.operator.is_delete(),
            _ => return false,
        }
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;
    let options_ignore_arguments = || Some(serde_json::json!([{ "checkArguments": false }]));
    let options_ignore_arrow_function_body =
        || Some(serde_json::json!([{ "checkArrowFunctionBody": false }]));
    let pass = vec![
        ("function foo() {return;}", None),
        ("const foo = () => {};", None),
        ("let foo;", None),
        ("var foo;", None),
        ("const foo = undefined;", None),
        ("foo();", None),
        ("foo(bar,);", None),
        ("foo(undefined, bar);", None),
        ("const {foo} = {};", None),
        ("function foo({bar} = {}) {}", None),
        ("function foo(bar) {}", None),
        // I guess nobody uses this, but `yield* undefined;` is valid code, and `yield*;` is not
        (r"function* foo() {yield* undefined;}", None),
        // Ignored
        ("if (Object.is(foo, undefined)){}", None),
        ("t.is(foo, undefined)", None),
        ("assert.equal(foo, undefined, message)", None),
        ("assert.notEqual(foo, undefined, message)", None),
        ("assert.strictEqual(foo, undefined, message)", None),
        ("assert.notStrictEqual(foo, undefined, message)", None),
        ("assert.propertyVal(foo, 'bar', undefined, message)", None),
        ("assert.notPropertyVal(foo, 'bar', undefined, message)", None),
        ("expect(foo).not(undefined)", None),
        ("expect(foo).to.have.property('bar', undefined)", None),
        ("expect(foo).toBe(undefined)", None),
        ("expect(foo).toContain(undefined)", None),
        ("expect(foo).toContainEqual(undefined)", None),
        ("expect(foo).toEqual(undefined)", None),
        ("t.same(foo, undefined)", None),
        ("t.notSame(foo, undefined)", None),
        ("t.strictSame(foo, undefined)", None),
        ("t.strictNotSame(foo, undefined)", None),
        ("expect(someFunction).toHaveBeenCalledWith(1, 2, undefined);", None),
        ("set.add(undefined);", None),
        ("map.set(foo, undefined);", None),
        ("array.push(foo, undefined);", None),
        ("array.push(undefined);", None),
        ("array.unshift(foo, undefined);", None),
        ("array.unshift(undefined);", None),
        ("createContext(undefined);", None),
        ("React.createContext(undefined);", None),
        ("useRef(undefined);", None),
        ("React.useRef(undefined);", None),
        ("setState(undefined)", None),
        ("setState?.(undefined)", None),
        ("props.setState(undefined)", None),
        ("props.setState?.(undefined)", None),
        ("array.includes(undefined)", None),
        ("set.has(undefined)", None),
        // `Function#bind()`
        ("foo.bind(bar, undefined);", None),
        ("foo.bind(...bar, undefined);", None),
        ("foo.bind(...[], undefined);", None),
        ("foo.bind(...[undefined], undefined);", None),
        ("foo.bind(bar, baz, undefined);", None),
        ("foo?.bind(bar, undefined);", None),
        // `checkArguments: false`
        ("foo(undefined, undefined);", options_ignore_arguments()),
        ("foo.bind(undefined);", options_ignore_arguments()),
        (
            "function run(name?: string) { return name; } run(undefined);",
            options_ignore_arguments(),
        ),
        // `checkArrowFunctionBody: false`
        ("const foo = () => undefined", options_ignore_arrow_function_body()),
        ("const x = { a: undefined }", None),
        // https://github.com/zeit/next.js/blob/3af0fe5cf2542237f34d106872d104c3606b1858/packages/next/build/utils.ts#L620
        ("prerenderPaths?.add(entry)", None),
        (
            r#"
            function getThing(): string | undefined {
                if (someCondition) {
                    return "hello world";
                }

                return undefined;
            }
        "#,
            None,
        ),
        (
            r#"
            function getThing(): string | undefined {
                if (someCondition) {
                    return "hello world";
                } else if (anotherCondition) {
                    return undefined;
                }

                return undefined;
            }
        "#,
            None,
        ),
        ("const foo = (): undefined => {return undefined;}", None),
        ("const foo = (): undefined => undefined;", None),
        ("const foo = (): string => undefined;", None),
        ("const foo = function (): undefined {return undefined}", None),
        ("export function foo(): undefined {return undefined}", None),
        (
            r"
                    const object = {
                        method(): undefined {
                            return undefined;
                        }
                    }
                ",
            None,
        ),
        (
            r"
            class A {
                method(): undefined {
                    return undefined;
                }
            }
        ",
            None,
        ),
        (
            r"
            const A = class A {
                method(): undefined {
                    return undefined
                }
            };
        ",
            None,
        ),
        (
            r"
            class A {
                static method(): undefined {
                    return undefined
                }
            }
        ",
            None,
        ),
        (
            r"
            class A {
                get method(): undefined {
                    return undefined;
                }
            }
        ",
            None,
        ),
        (
            r"
            class A {
                static get method(): undefined {
                    return undefined;
                }
            }
        ",
            None,
        ),
        (
            r"
            class A {
                #method(): undefined {
                    return undefined;
                }
            }
        ",
            None,
        ),
        (
            r"
            class A {
                private method(): undefined {
                    return undefined;
                }
            }
        ",
            None,
        ),
        ("createContext<T>(undefined);", None),
        ("React.createContext<T>(undefined);", None),
        ("useRef<T>(undefined);", None),
        ("React.useRef<T>(undefined);", None),
        (
            "
            const y: any = {}
            y.foo = undefined
        ",
            None,
        ),
        (
            "
            class Foo {
                public x: number | undefined = undefined
            }
        ",
            None,
        ),
        ("function foo(optional?: string, required: string = undefined) {}", None),
        (
            "function getBackLinks(subject: string, limit = 16, cursor?: string, reverse = false, ttl: number | undefined = undefined) {}",
            None,
        ),
    ];

    let fail = vec![
        (
            r"
        foo(
            undefined,
            bar,
            undefined,
            undefined,
            undefined,
            undefined,
        )
        ",
            None,
        ),
        (r"function foo([bar = undefined] = []) {}", None),
        (
            r"
            foo(
                undefined,
                bar,
                undefined,
                undefined,
                undefined,
                undefined,
            )
        ",
            None,
        ),
        ("function foo([bar = undefined] = []) {}", None),
        ("foo(bar, undefined, undefined);", None),
        ("let a = undefined, b = 2;", None),
        (
            r"
        function foo() {
            return /* */ (
                /* */
                (
                    /* */
                    undefined
                    /* */
                )
                /* */
            ) /* */ ;
        }
        ",
            None,
        ),
        (
            r"
        function * foo() {
            yield /* */ (
                /* */
                (
                    /* */
                    undefined
                    /* */
                )
                /* */
            ) /* */ ;
        }
        ",
            None,
        ),
        (
            r"
        const foo = () => /* */ (
            /* */
            (
                /* */
                undefined
                /* */
            )
            /* */
        );
        ",
            None,
        ),
        ("foo.bind(undefined)", None),
        ("bind(foo, undefined)", None),
        ("foo.bind?.(bar, undefined)", None),
        ("foo[bind](bar, undefined)", None),
        ("foo.notBind(bar, undefined)", None),
    ];

    let fix = vec![
        (r"function foo() {return undefined;}", r"function foo() {return;}", None),
        ("const foo = () => undefined;", "const foo = () => {};", None),
        ("const foo = () => {return undefined;};", "const foo = () => {return;};", None),
        ("function foo() {return       undefined;}", "function foo() {return;}", None),
        (
            "function foo() {return /* comment */ undefined;}",
            "function foo() {return /* comment */;}",
            None,
        ),
        ("function* foo() {yield undefined;}", "function* foo() {yield;}", None),
        ("function* foo() {yield                 undefined;}", "function* foo() {yield;}", None),
        ("let a = undefined;", "let a;", None),
        ("let a = undefined, b = 2;", "let a, b = 2;", None),
        ("var a = undefined;", "var a;", None),
        ("var a = undefined, b = 2;", "var a, b = 2;", None),
        ("foo(undefined);", "foo();", None),
        ("foo(undefined, undefined);", "foo();", None),
        ("foo(undefined,);", "foo();", None),
        ("foo(undefined, undefined,);", "foo();", None),
        ("foo(bar, undefined);", "foo(bar);", None),
        ("foo(bar, undefined, undefined);", "foo(bar);", None),
        ("foo(undefined, bar, undefined);", "foo(undefined, bar);", None),
        ("foo(bar, undefined,);", "foo(bar,);", None),
        ("foo(undefined, bar, undefined,);", "foo(undefined, bar,);", None),
        ("foo(bar, undefined, undefined,);", "foo(bar,);", None),
        ("foo(undefined, bar, undefined, undefined,);", "foo(undefined, bar,);", None),
        (
            "foo(undefined, bar, undefined, undefined, undefined, undefined,);",
            "foo(undefined, bar,);",
            None,
        ),
        ("const {foo = undefined} = {};", "const {foo} = {};", None),
        ("const [foo = undefined] = [];", "const [foo] = [];", None),
        ("function foo(bar = undefined) {}", "function foo(bar) {}", None),
        ("function foo({bar = undefined}) {}", "function foo({bar}) {}", None),
        ("function foo({bar = undefined} = {}) {}", "function foo({bar} = {}) {}", None),
        ("function foo([bar = undefined]) {}", "function foo([bar]) {}", None),
        ("function foo([bar = undefined] = []) {}", "function foo([bar] = []) {}", None),
        (
            "function foo(x: string | undefined = undefined) { }",
            "function foo(x: string | undefined) { }",
            None,
        ),
        ("return undefined;", "return;", None),
        (
            r"
            function foo():undefined {
                    function nested() {
                        return undefined;
                    }

                    return nested();
                }
        ",
            r"
            function foo():undefined {
                    function nested() {
                        return;
                    }

                    return nested();
                }
        ",
            None,
        ),
    ];

    Tester::new(NoUselessUndefined::NAME, NoUselessUndefined::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_config_array_format() {
    use crate::tester::Tester;

    let pass = vec![
        (r"foo(undefined);", Some(serde_json::json!([{ "checkArguments": false }]))),
        (
            r"const foo = () => undefined;",
            Some(serde_json::json!([{ "checkArrowFunctionBody": false }])),
        ),
    ];
    let fail = vec![
        (r"foo(undefined);", Some(serde_json::json!([{ "checkArguments": true }]))),
        (
            r"const foo = () => undefined;",
            Some(serde_json::json!([{ "checkArrowFunctionBody": true }])),
        ),
    ];
    let fix = vec![
        (r"foo(undefined);", r"foo();", Some(serde_json::json!([{ "checkArguments": true }]))),
        (
            r"const foo = () => undefined;",
            r"const foo = () => {};",
            Some(serde_json::json!([{ "checkArrowFunctionBody": true }])),
        ),
    ];

    Tester::new(NoUselessUndefined::NAME, NoUselessUndefined::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}

#[test]
fn test_issue_14368() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r"function run(name) { return name; } run(undefined);",
            Some(serde_json::json!([{ "checkArguments": false }])),
        ),
        (
            r"function run(name?: string) { return name; } run(undefined);",
            Some(serde_json::json!([{ "checkArguments": false }])),
        ),
    ];
    let fail = vec![(
        r"function run(name) { return name; } run(undefined);",
        Some(serde_json::json!([{ "checkArguments": true }])),
    )];
    let fix = vec![(
        r"function run(name) { return name; } run(undefined);",
        r"function run(name) { return name; } run();",
        Some(serde_json::json!([{ "checkArguments": true }])),
    )];

    Tester::new(NoUselessUndefined::NAME, NoUselessUndefined::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}

#[test]
fn test_trailing_arguments() {
    use crate::tester::Tester;

    let pass = vec![
        "call(undefined, ...values);",
        "call(...values);",
        "call?.(value);",
        "call.bind(receiver);",
        "call.bind(receiver, undefined);",
    ];
    let fail =
        vec!["call(...values, undefined, undefined);", "call?.(value, undefined, undefined);"];
    let fix = vec![
        ("call(...values, undefined, undefined);", "call(...values);", None),
        ("call?.(value, undefined, undefined);", "call?.(value);", None),
    ];
    Tester::new(NoUselessUndefined::NAME, NoUselessUndefined::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}

#[test]
fn test_guarded_indexed_access() {
    use crate::tester::Tester;

    let fail = vec![
        "function a(array, index) { return index >= 0 ? array[index] : undefined; }",
        "function b(array, index) { return index > -1 ? array[index] : undefined; }",
        "function c(array, index) { return index < 0 ? undefined : array[index]; }",
        "function d(array, index) { return index < array.length ? array[index] : undefined; }",
        "function e(array, index) { return array.length > index ? array[index] : undefined; }",
        "function f(array, index) { return index >= array.length ? undefined : array[index]; }",
        "function g(array, index) { return index <= array.length - 1 ? array[index] : undefined; }",
        "function h(array, index) { return index >= 1 ? array[index - 1] : undefined; }",
    ];
    Tester::new(NoUselessUndefined::NAME, NoUselessUndefined::PLUGIN, vec![], fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("guarded_indexed_access")
        .test_and_snapshot();
}

#[test]
fn test_indexed_access_bounds() {
    use crate::{fixer::FixKind, tester::Tester};

    let mut pass = vec![];
    let mut fail = vec![];
    let mut fix = vec![];
    for (test, access, valid) in [
        ("index >= 0", "array[index]", true),
        ("index > -1", "array[index]", true),
        ("index < 0", "array[index]", false),
        ("index <= -1", "array[index]", false),
        ("0 <= index", "array[index]", true),
        ("-1 < index", "array[index]", true),
        ("0 > index", "array[index]", false),
        ("-1 >= index", "array[index]", false),
        ("index >= 5", "array[index - 5]", true),
        ("index > 4", "array[index - 5]", true),
        ("index < 5", "array[index - 5]", false),
        ("index <= 4", "array[index - 5]", false),
        ("5 <= index", "array[index - 5]", true),
        ("4 < index", "array[index - 5]", true),
        ("5 > index", "array[index - 5]", false),
        ("4 >= index", "array[index - 5]", false),
        ("index < array.length", "array[index]", true),
        ("index <= array.length - 1", "array[index]", true),
        ("index >= array.length", "array[index]", false),
        ("index > array.length - 1", "array[index]", false),
        ("array.length > index", "array[index]", true),
        ("array.length - 1 >= index", "array[index]", true),
        ("array.length <= index", "array[index]", false),
        ("array.length - 1 < index", "array[index]", false),
    ] {
        let (consequent, alternate) =
            if valid { (access, "undefined") } else { ("undefined", access) };
        let source = format!("const value = {test} ? {consequent} : {alternate};");
        fail.push(source.clone());
        pass.push(format!("const value = {test} ? {alternate} : {consequent};"));
        fix.push((source, format!("const value = {access};"), None, FixKind::Suggestion));
    }
    Tester::new(NoUselessUndefined::NAME, NoUselessUndefined::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}

#[test]
fn test_indexed_access_safety() {
    use crate::{fixer::FixKind, tester::Tester};

    let pass = vec![
        "const value = index >= 0 ? array[index] : other;",
        "const value = index >= 0 ? array[index] : (undefined as Foo);",
        "const value = index >= 0 ? object.property : undefined;",
        "const value = index >= 0 ? array?.[index] : undefined;",
        "const value = index >= 0 ? object?.items[index] : undefined;",
        "const value = index >= 0 ? getArray()[index] : undefined;",
        "const value = index >= 0 ? import('x')[index] : undefined;",
        "const value = index >= 0 ? [() => foo?.bar][index] : undefined;",
        "const value = index >= 0 ? function() { return foo?.bar; }[index] : undefined;",
        "const value = index >= 0 ? object.items[index] : undefined;",
        "const value = index >= 0 ? array[index++] : undefined;",
        "const value = index >= 0 ? array[getIndex()] : undefined;",
        "const value = index >= 0 ? array[object.index] : undefined;",
        "const value = index < otherArray.length ? array[index] : undefined;",
        "const value = index >= 0 ? array[index + 1] : undefined;",
        "const value = index >= 0 ? array[otherIndex] : undefined;",
        "const value = index >= 5 ? array[index] : undefined;",
        "const value = index < array.length ? array[index - 1] : undefined;",
        "const value = foo?.bar >= 0 ? array[foo.bar] : undefined;",
        "const value = condition ? array[index] : undefined;",
        "const value = index + 1 >= 0 ? array[index + 1] : undefined;",
        "const value = index - 1 >= 0 ? array[index - 1] : undefined;",
        "const value = index >= object?.minimum ? array[index] : undefined;",
        "const value = index == 0 ? array[index] : undefined;",
        "const value = index >= 0.5 ? array[index - 0.5] : undefined;",
        "const value = index >= 9007199254740992 ? array[index - 9007199254740992] : undefined;",
        "const value = index >= 0 ? array[index - -1] : undefined;",
        "const value = index >= 0 ? array[index - offset] : undefined;",
        "const value = index < array?.length ? array[index] : undefined;",
        "const value = index < array['length'] ? array[index] : undefined;",
        "const value = index <= array.length ? array[index] : undefined;",
        "const value = index < array.length - 1 ? array[index] : undefined;",
        "function f(undefined, array, index) { return index >= 0 ? array[index] : undefined; }",
    ];
    let no_suggestion = [
        "const value = index >= 0 ? array[index] /* keep */ : undefined;",
        "const value = index /* keep */ >= 0 ? array[index] : undefined;",
        "const value = index >= 0 ? array[index] : /* keep */ undefined;",
        "(index >= 0 ? array[index] : undefined)();",
        "(index >= 0 ? array[index] : undefined)?.();",
        "(index >= 0 ? array[index] : undefined)`template`;",
        "delete (index >= 0 ? array[index] : undefined);",
        "((index >= 0 ? array[index] : undefined) as Function)();",
    ];
    let mut fail = no_suggestion.to_vec();
    let mut fix = vec![
        (
            "const value = 0n < array.length ? array[0n] : undefined;",
            "const value = array[0n];",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = 0x0n >= 0 ? array[0n] : undefined;",
            "const value = array[0n];",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = index >= 0 ? [() => foo.bar][index] : undefined;",
            "const value = [() => foo.bar][index];",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = index >= 0 ? (array[index] as string) : undefined;",
            "const value = (array[index] as string);",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = (index as number) >= 0 ? array[index as number] : undefined;",
            "const value = array[index as number];",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = index! >= 0 ? array[index!] : undefined;",
            "const value = array[index!];",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = (index satisfies number) >= 0 ? array[index satisfies number] : undefined;",
            "const value = array[index satisfies number];",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = <number>index >= 0 ? array[<number>index] : undefined;",
            "const value = array[<number>index];",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = index >= (0 as number) ? array[index] : undefined;",
            "const value = array[index];",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = index >= 5 ? array[index - (5 as number)] : undefined;",
            "const value = array[index - (5 as number)];",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = index <= array.length - (1 as number) ? array[index] : undefined;",
            "const value = array[index];",
            None,
            FixKind::Suggestion,
        ),
        (
            "function f() { return(index) >= 0 ? array[index] : undefined; }",
            "function f() { return array[index]; }",
            None,
            FixKind::Suggestion,
        ),
        (
            "a()\nindex >= 0 ? [array][index] : undefined;",
            "a()\n;[array][index];",
            None,
            FixKind::Suggestion,
        ),
        (
            "a()\nindex >= 0 ? <number>array[index] : undefined;",
            "a()\n;(<number>array[index]);",
            None,
            FixKind::Suggestion,
        ),
        (
            "if (condition) index >= 0 ? <number>array[index] : undefined;",
            "if (condition) (<number>array[index]);",
            None,
            FixKind::Suggestion,
        ),
        (
            "function f() { return(index) >= 0 ? <number>array[index] : undefined; }",
            "function f() { return(<number>array[index]); }",
            None,
            FixKind::Suggestion,
        ),
        ("index >= 0 ? {}[index] : undefined;", "({}[index]);", None, FixKind::Suggestion),
        (
            "index >= 0 ? function() {}[index] : undefined;",
            "(function() {}[index]);",
            None,
            FixKind::Suggestion,
        ),
        (
            "if (condition) index >= 0 ? [array][index] : undefined;",
            "if (condition) [array][index];",
            None,
            FixKind::Suggestion,
        ),
        (
            "const value = index >= 0 ? array[index] : undefined;",
            "const value = index >= 0 ? array[index] : undefined;",
            None,
            FixKind::Fix,
        ),
        (
            "const value = index >= 0 ? array[index] : undefined;",
            "const value = array[index];",
            Some(serde_json::json!([{ "checkArguments": false, "checkArrowFunctionBody": false }])),
            FixKind::Suggestion,
        ),
    ];
    fail.extend(fix.iter().map(|(source, _, _, _)| *source));
    fix.extend(no_suggestion.iter().map(|source| (*source, *source, None, FixKind::Suggestion)));
    Tester::new(NoUselessUndefined::NAME, NoUselessUndefined::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .expect_fix(fix)
        .test();
}
