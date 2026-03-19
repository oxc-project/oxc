use std::mem::discriminant;

use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, BinaryOperator, Expression, IfStatement, MemberExpression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{is_same_expression, is_same_member_expression},
};

fn prefer_ternary_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer ternary expressions over simple `if-else` statements.")
        .with_help("Rewrite this `if`/`else` as a ternary expression.")
        .with_label(span)
}

#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PreferTernaryOption {
    /// Always enforce ternary usage when the branches can be safely merged.
    #[default]
    Always,
    /// Only enforce ternary usage when the condition and both branches are single-line.
    OnlySingleLine,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferTernary(PreferTernaryOption);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers ternary expressions over simple `if`/`else` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Simple `if`/`else` branches for the same operation are often shorter and
    /// clearer when expressed as a ternary.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (test) { return a; } else { return b; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// return test ? a : b;
    /// ```
    PreferTernary,
    unicorn,
    style,
    pending,
    config = PreferTernaryOption
);

impl Rule for PreferTernary {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_statement) = node.kind() else {
            return;
        };

        if matches!(if_statement.test.get_inner_expression(), Expression::ConditionalExpression(_))
            || is_else_if_branch(node, if_statement, ctx)
        {
            return;
        }

        let Some(if_alternate) = &if_statement.alternate else {
            return;
        };

        let consequent = get_node_body_statement(&if_statement.consequent);
        let alternate = get_node_body_statement(if_alternate);

        if self.0 == PreferTernaryOption::OnlySingleLine
            && (!is_single_line_body(&consequent, ctx)
                || !is_single_line_body(&alternate, ctx)
                || !is_single_line_expression(&if_statement.test, ctx))
        {
            return;
        }

        if is_mergeable(
            MergeNode::Body(consequent),
            MergeNode::Body(alternate),
            MergeOptions { check_throw_statement: true, strict: true },
            ctx,
        ) {
            ctx.diagnostic(prefer_ternary_diagnostic(if_statement.span));
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BodyNode<'a> {
    Statement(&'a Statement<'a>),
    Expression(&'a Expression<'a>),
}

#[derive(Debug, Clone, Copy)]
enum MergeNode<'a> {
    Body(BodyNode<'a>),
    Expression(&'a Expression<'a>),
    Undefined,
}

#[derive(Debug, Clone, Copy)]
struct MergeOptions {
    check_throw_statement: bool,
    strict: bool,
}

fn get_node_body_statement<'a>(statement: &'a Statement<'a>) -> BodyNode<'a> {
    match statement {
        Statement::ExpressionStatement(expression_statement) => {
            BodyNode::Expression(expression_statement.expression.get_inner_expression())
        }
        Statement::BlockStatement(block_statement) => {
            let mut non_empty = block_statement
                .body
                .iter()
                .filter(|statement| !matches!(statement, Statement::EmptyStatement(_)));
            if let Some(single) = non_empty.next()
                && non_empty.next().is_none()
            {
                return get_node_body_statement(single);
            }
            BodyNode::Statement(statement)
        }
        _ => BodyNode::Statement(statement),
    }
}

fn is_single_line_expression(expression: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    let span = expression.get_inner_expression().span();
    !ctx.source_range(span).contains('\n')
}

fn is_single_line_body(body: &BodyNode<'_>, ctx: &LintContext<'_>) -> bool {
    let span = match body {
        BodyNode::Expression(expression) => expression.get_inner_expression().span(),
        BodyNode::Statement(statement) => statement.span(),
    };
    !ctx.source_range(span).contains('\n')
}

fn is_else_if_branch(
    node: &AstNode<'_>,
    if_statement: &IfStatement<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    let AstKind::IfStatement(parent_if_statement) = ctx.nodes().parent_kind(node.id()) else {
        return false;
    };

    parent_if_statement
        .alternate
        .as_ref()
        .is_some_and(|alternate| alternate.span() == if_statement.span)
}

fn is_mergeable<'a>(
    consequent: MergeNode<'a>,
    alternate: MergeNode<'a>,
    options: MergeOptions,
    ctx: &LintContext<'a>,
) -> bool {
    if !same_merge_kind(consequent, alternate) {
        return !options.strict;
    }

    match (consequent, alternate) {
        (
            MergeNode::Body(BodyNode::Statement(Statement::ReturnStatement(consequent))),
            MergeNode::Body(BodyNode::Statement(Statement::ReturnStatement(alternate))),
        ) if !is_ternary_option_expression(consequent.argument.as_ref())
            && !is_ternary_option_expression(alternate.argument.as_ref()) =>
        {
            is_mergeable(
                consequent.argument.as_ref().map_or(MergeNode::Undefined, MergeNode::Expression),
                alternate.argument.as_ref().map_or(MergeNode::Undefined, MergeNode::Expression),
                MergeOptions { strict: false, ..options },
                ctx,
            )
        }
        (
            MergeNode::Body(BodyNode::Expression(Expression::YieldExpression(consequent))),
            MergeNode::Body(BodyNode::Expression(Expression::YieldExpression(alternate))),
        ) if consequent.delegate == alternate.delegate
            && !is_ternary_option_expression(consequent.argument.as_ref())
            && !is_ternary_option_expression(alternate.argument.as_ref()) =>
        {
            is_mergeable(
                consequent.argument.as_ref().map_or(MergeNode::Undefined, MergeNode::Expression),
                alternate.argument.as_ref().map_or(MergeNode::Undefined, MergeNode::Expression),
                MergeOptions { strict: false, ..options },
                ctx,
            )
        }
        (
            MergeNode::Body(BodyNode::Expression(Expression::AwaitExpression(consequent))),
            MergeNode::Body(BodyNode::Expression(Expression::AwaitExpression(alternate))),
        ) if !is_ternary_expression(&consequent.argument)
            && !is_ternary_expression(&alternate.argument) =>
        {
            is_mergeable(
                MergeNode::Expression(&consequent.argument),
                MergeNode::Expression(&alternate.argument),
                MergeOptions { strict: false, ..options },
                ctx,
            )
        }
        (
            MergeNode::Body(BodyNode::Statement(Statement::ThrowStatement(consequent))),
            MergeNode::Body(BodyNode::Statement(Statement::ThrowStatement(alternate))),
        ) if options.check_throw_statement
            && !is_ternary_expression(&consequent.argument)
            && !is_ternary_expression(&alternate.argument) =>
        {
            true
        }
        (
            MergeNode::Body(BodyNode::Expression(Expression::AssignmentExpression(consequent))),
            MergeNode::Body(BodyNode::Expression(Expression::AssignmentExpression(alternate))),
        ) if consequent.operator == alternate.operator
            && !is_ternary_expression(&consequent.right)
            && !is_ternary_expression(&alternate.right)
            && is_same_assignment_target(&consequent.left, &alternate.left, ctx) =>
        {
            is_mergeable(
                MergeNode::Expression(&consequent.right),
                MergeNode::Expression(&alternate.right),
                MergeOptions { strict: false, ..options },
                ctx,
            )
        }
        _ => !options.strict,
    }
}

fn same_merge_kind(consequent: MergeNode<'_>, alternate: MergeNode<'_>) -> bool {
    match (consequent, alternate) {
        (MergeNode::Undefined, MergeNode::Undefined) => true,
        (MergeNode::Expression(consequent), MergeNode::Expression(alternate))
        | (
            MergeNode::Body(BodyNode::Expression(consequent)),
            MergeNode::Body(BodyNode::Expression(alternate)),
        ) => {
            discriminant(consequent.get_inner_expression())
                == discriminant(alternate.get_inner_expression())
        }
        (
            MergeNode::Body(BodyNode::Statement(consequent)),
            MergeNode::Body(BodyNode::Statement(alternate)),
        ) => discriminant(consequent) == discriminant(alternate),
        _ => false,
    }
}

fn is_ternary_option_expression(expression: Option<&Expression<'_>>) -> bool {
    expression.is_some_and(is_ternary_expression)
}

fn is_ternary_expression(expression: &Expression<'_>) -> bool {
    matches!(expression.get_inner_expression(), Expression::ConditionalExpression(_))
}

fn is_same_assignment_target(
    left: &AssignmentTarget<'_>,
    right: &AssignmentTarget<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    if let (
        AssignmentTarget::AssignmentTargetIdentifier(left),
        AssignmentTarget::AssignmentTargetIdentifier(right),
    ) = (left, right)
    {
        return left.name == right.name;
    }

    if let (Some(left_member), Some(right_member)) =
        (left.as_member_expression(), right.as_member_expression())
    {
        if let (Some(left_name), Some(right_name)) =
            (member_static_property_name(left_member), member_static_property_name(right_member))
        {
            return left_name == right_name
                && is_same_expression(
                    left_member.object().get_inner_expression(),
                    right_member.object().get_inner_expression(),
                    ctx,
                );
        }

        return is_same_member_expression(left_member, right_member, ctx);
    }

    match (left.get_expression(), right.get_expression()) {
        (Some(left), Some(right)) => is_same_expression(left, right, ctx),
        _ => false,
    }
}

fn member_static_property_name(member: &MemberExpression<'_>) -> Option<String> {
    if let Some(name) = member.static_property_name() {
        return Some(name.to_string());
    }

    let MemberExpression::ComputedMemberExpression(computed) = member else {
        return None;
    };

    static_string_value(computed.expression.get_inner_expression())
}

fn static_string_value(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(literal) => {
            literal.single_quasi().map(|quasi| quasi.to_string())
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left = static_string_value(binary.left.get_inner_expression())?;
            let right = static_string_value(binary.right.get_inner_expression())?;
            Some(format!("{left}{right}"))
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let only_single_line_options = vec!["only-single-line"];

    let pass = vec![
        (
            "function unicorn() {
                if(a ? b : c){
                    return a;
                } else{
                    return b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(test){
                    return a ? b : c;
                } else{
                    return b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(test){
                    return a;
                } else{
                    return a ? b : c;
                }
            }",
            None,
        ),
        (
            "function* unicorn() {
                if(test){
                    yield* a;
                } else{
                    yield b;
                }
            }",
            None,
        ),
        (
            "function* unicorn() {
                if(a ? b : c){
                    yield a;
                } else{
                    yield b;
                }
            }",
            None,
        ),
        (
            "function* unicorn() {
                if(test){
                    yield a ? b : c;
                } else{
                    yield b;
                }
            }",
            None,
        ),
        (
            "function* unicorn() {
                if(test){
                    yield a;
                } else{
                    yield a ? b : c;
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if(a ? b : c){
                    await a;
                } else{
                    await b;
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if(test){
                    await a ? b : c;
                } else{
                    await b;
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if(test){
                    await a;
                } else{
                    await a ? b : c;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(a ? b : c){
                    throw a;
                } else{
                    throw b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if (test) {
                    throw a ? b : c;
                } else {
                    throw b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if (test) {
                    throw a;
                } else {
                    throw a ? b : c;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(test){
                    foo = a;
                } else{
                    bar = b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(test){
                    foo = a;
                } else{
                    foo *= b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(test){
                    foo().bar = a;
                } else{
                    foo().bar = b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(a ? b : c){
                    foo = a;
                } else{
                    foo = b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(test){
                    foo = a ? b : c;
                } else{
                    foo = b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(test){
                    foo = a;
                } else{
                    foo = a ? b : c;
                }
            }",
            None,
        ),
        (
            "if (test) {
                a = {
                    multiline: 'in consequent'
                };
            } else{
                a = foo;
            }",
            Some(serde_json::json!(only_single_line_options)),
        ),
        (
            "if (test) {
                a = foo;
            } else{
                a = {
                    multiline: 'in alternate'
                };
            }",
            Some(serde_json::json!(only_single_line_options)),
        ),
        (
            "if (
                test({
                    multiline: 'in test'
                })
            ) {
                a = foo;
            } else{
                a = bar;
            }",
            Some(serde_json::json!(only_single_line_options)),
        ),
        (
            "if (test) {
                a = foo; b = 1;
            } else{
                a = bar;
            }",
            Some(serde_json::json!(only_single_line_options)),
        ),
        ("if (a) {b}", None),
        ("if (a) {} else {b}", None),
        ("if (a) {} else {}", None),
        (
            "if (test) {
                a();
            } else {
                b();
            }",
            None,
        ),
        (
            "function foo(){
                if (a) {
                    return 1;
                } else if (b) {
                    return 2;
                } else if (c) {
                    return 3;
                } else {
                    return 4;
                }
            }",
            None,
        ),
    ];

    let fail = vec![
        (
            "function unicorn() {
                if(test){
                    return a;
                } else{
                    return b;
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if(test){
                    return await a;
                } else{
                    return b;
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if(test){
                    return await a;
                } else{
                    return await b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(test){
                    return;
                } else{
                    return b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(test){
                    return;
                } else{
                    return;
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if(test){
                    return;
                } else{
                    return await b;
                }
            }",
            None,
        ),
        (
            "async function* unicorn() {
                if(test){
                    return yield await (foo = a);
                } else{
                    return yield await (foo = b);
                }
            }",
            None,
        ),
        (
            "function* unicorn() {
                if(test){
                    yield a;
                } else{
                    yield b;
                }
            }",
            None,
        ),
        (
            "function* unicorn() {
                if(test){
                    yield;
                } else{
                    yield b;
                }
            }",
            None,
        ),
        (
            "function* unicorn() {
                if(test){
                    yield;
                } else{
                    yield;
                }
            }",
            None,
        ),
        (
            "async function* unicorn() {
                if(test){
                    yield;
                } else{
                    yield await b;
                }
            }",
            None,
        ),
        (
            "function* unicorn() {
                if(test){
                    yield* a;
                } else{
                    yield* b;
                }
            }",
            None,
        ),
        (
            "async function* unicorn() {
                if(test){
                    yield await a;
                } else{
                    yield b;
                }
            }",
            None,
        ),
        (
            "async function* unicorn() {
                if(test){
                    yield await a;
                } else{
                    yield await b;
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if(test){
                    await doSomething1();
                } else{
                    await doSomething2();
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if(test){
                    await a;
                } else{
                    await b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if (test) {
                    throw new Error('a');
                } else{
                    throw new TypeError('a');
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                /* comment cause wrong indention */ if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                                                    if (test) {
                                                        throw a;
                                                    } else {
                                                        throw b;
                                                    }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if (test) {
                    throw await a;
                } else {
                    throw b;
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if (test) {
                    throw await a;
                } else {
                    throw await b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                const error = new Error();
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
                try {} catch(error) {
                    const error_ = new TypeError(error);
                    throw error_;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
                function foo() {
                    throw error;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                const error = test ? a : b;
                throw error;
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }",
            None,
        ),
        (
            "function outer() {
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
                function inner() {
                    if (test) {
                        throw a;
                    } else {
                        throw b;
                    }
                }
            }",
            None,
        ),
        (
            "function outer() {
                const error = test ? a : b;
                throw error;
                function inner() {
                    if (test) {
                        throw a;
                    } else {
                        throw b;
                    }
                }
            }",
            None,
        ),
        ("while (foo) if (test) {throw a} else {throw b}", None),
        (
            "function unicorn() {
                if(test){
                    foo = a;
                } else{
                    foo = b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if(test){
                    foo *= a;
                } else{
                    foo *= b;
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if(test){
                    foo = await a;
                } else{
                    foo = b;
                }
            }",
            None,
        ),
        (
            "async function unicorn() {
                if(test){
                    foo = await a;
                } else{
                    foo = await b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if (test) {
                    foo.bar = a;
                } else{
                    foo.bar = b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                a()
                if (test) {
                    (foo)['b' + 'ar'] = a
                } else{
                    foo.bar = b
                }
            }",
            None,
        ),
        (
            "async function* unicorn() {
                if(test){
                    foo = yield await a;
                } else{
                    foo = yield await b;
                }
            }",
            None,
        ),
        (
            "if(test){
                $0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 =
                _STOP_ =
                $0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 =
                1;
            } else{
                $0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 =
                _STOP_2_ =
                $0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 =
                2;
            }",
            None,
        ),
        (
            "if (test) {
                a = foo;
            } else {
                a = bar;
            }",
            Some(serde_json::json!(only_single_line_options)),
        ),
        (
            "if (
                (
                    test
                )
            ) {
                a = foo;
            } else {
                a = bar;
            }",
            Some(serde_json::json!(only_single_line_options)),
        ),
        (
            "if (test) {
                (
                    a = foo
                );
            } else {
                a = bar;
            }",
            Some(serde_json::json!(only_single_line_options)),
        ),
        (
            "if (test) {
                a = foo
                ;
            } else {
                a = bar;
            }",
            Some(serde_json::json!(only_single_line_options)),
        ),
        (
            "if (test) {
                ;;;;;;
                a = foo;
                ;;;;;;
            } else {
                a = bar;
            }",
            Some(serde_json::json!(only_single_line_options)),
        ),
        (
            "function unicorn() {
                // There is an empty block inside consequent
                if (test) {
                    ;
                    return a;
                } else {
                    return b;
                }
            }",
            None,
        ),
        (
            "function unicorn() {
                if (test) {
                    foo = a
                } else foo = b;
            }",
            None,
        ),
        (
            "function unicorn() {
                if (test) return a;
                else return b;
            }",
            None,
        ),
        (
            "if (a = b) {
                foo = 1;
            } else foo = 2;",
            None,
        ),
        (
            "function* unicorn() {
                if (yield a) {
                    foo = 1;
                } else foo = 2;
            }",
            None,
        ),
        (
            "function* unicorn() {
                if (yield* a) {
                    foo = 1;
                } else foo = 2;
            }",
            None,
        ),
        (
            "function foo(){
                if (a) {
                    return 1;
                } else {
                    if (b) {
                        return 2;
                    } else {
                        return 3;
                    }
                }
            }",
            None,
        ),
        (
            "function foo(){
                if (a) {
                    if (b) {
                        return 1;
                    } else {
                        return 2;
                    }
                } else {
                    return 3;
                }
            }",
            None,
        ),
        ("if (test) {foo = /* comment */1;} else {foo = 2;}", None),
        (
            "function *foo(bool) {
                if (!bool) {
                    yield call(
                        setOnTop,
                        false,
                    );
                } else {
                    yield call(
                        setOnTop,
                        true,
                        'normal',
                    ); // Keep this comment.
                }
            }",
            None,
        ),
    ];

    let _fix = vec![
        ("function unicorn() {
                if(test){
                    return a;
                } else{
                    return b;
                }
            }", "function unicorn() {
                return test ? a : b;
            }", None),
("async function unicorn() {
                if(test){
                    return await a;
                } else{
                    return b;
                }
            }", "async function unicorn() {
                return test ? (await a) : b;
            }", None),
("async function unicorn() {
                if(test){
                    return await a;
                } else{
                    return await b;
                }
            }", "async function unicorn() {
                return await (test ? a : b);
            }", None),
("function unicorn() {
                if(test){
                    return;
                } else{
                    return b;
                }
            }", "function unicorn() {
                return test ? undefined : b;
            }", None),
("function unicorn() {
                if(test){
                    return;
                } else{
                    return;
                }
            }", "function unicorn() {
                return test ? undefined : undefined;
            }", None),
("async function unicorn() {
                if(test){
                    return;
                } else{
                    return await b;
                }
            }", "async function unicorn() {
                return test ? undefined : (await b);
            }", None),
("async function* unicorn() {
                if(test){
                    return yield await (foo = a);
                } else{
                    return yield await (foo = b);
                }
            }", "async function* unicorn() {
                return yield (await (foo = test ? a : b));
            }", None),
("function* unicorn() {
                if(test){
                    yield a;
                } else{
                    yield b;
                }
            }", "function* unicorn() {
                yield (test ? a : b);
            }", None),
("function* unicorn() {
                if(test){
                    yield;
                } else{
                    yield b;
                }
            }", "function* unicorn() {
                yield (test ? undefined : b);
            }", None),
("function* unicorn() {
                if(test){
                    yield;
                } else{
                    yield;
                }
            }", "function* unicorn() {
                yield (test ? undefined : undefined);
            }", None),
("async function* unicorn() {
                if(test){
                    yield;
                } else{
                    yield await b;
                }
            }", "async function* unicorn() {
                yield (test ? undefined : (await b));
            }", None),
("function* unicorn() {
                if(test){
                    yield* a;
                } else{
                    yield* b;
                }
            }", "function* unicorn() {
                yield* (test ? a : b);
            }", None),
("async function* unicorn() {
                if(test){
                    yield await a;
                } else{
                    yield b;
                }
            }", "async function* unicorn() {
                yield (test ? (await a) : b);
            }", None),
("async function* unicorn() {
                if(test){
                    yield await a;
                } else{
                    yield await b;
                }
            }", "async function* unicorn() {
                yield (await (test ? a : b));
            }", None),
("async function unicorn() {
                if(test){
                    await doSomething1();
                } else{
                    await doSomething2();
                }
            }", "async function unicorn() {
                await (test ? doSomething1() : doSomething2());
            }", None),
("async function unicorn() {
                if(test){
                    await a;
                } else{
                    await b;
                }
            }", "async function unicorn() {
                await (test ? a : b);
            }", None),
("function unicorn() {
                if (test) {
                    throw new Error('a');
                } else{
                    throw new TypeError('a');
                }
            }", "function unicorn() {
                const error = test ? new Error('a') : new TypeError('a');
                throw error;
            }", None),
("function unicorn() {
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }", "function unicorn() {
                const error = test ? a : b;
                throw error;
            }", None),
("function unicorn() {
                /* comment cause wrong indention */ if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }", "function unicorn() {
                /* comment cause wrong indention */ const error = test ? a : b;
             throw error;
            }", None),
("function unicorn() {
                                                    if (test) {
                                                        throw a;
                                                    } else {
                                                        throw b;
                                                    }
            }", "function unicorn() {
                                                    const error = test ? a : b;
                                                    throw error;
            }", None),
("async function unicorn() {
                if (test) {
                    throw await a;
                } else {
                    throw b;
                }
            }", "async function unicorn() {
                const error = test ? (await a) : b;
                throw error;
            }", None),
("async function unicorn() {
                if (test) {
                    throw await a;
                } else {
                    throw await b;
                }
            }", "async function unicorn() {
                const error = test ? (await a) : (await b);
                throw error;
            }", None),
("function unicorn() {
                const error = new Error();
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }", "function unicorn() {
                const error = new Error();
                const error_ = test ? a : b;
                throw error_;
            }", None),
("function unicorn() {
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
                try {} catch(error) {
                    const error_ = new TypeError(error);
                    throw error_;
                }
            }", "function unicorn() {
                const error__ = test ? a : b;
                throw error__;
                try {} catch(error) {
                    const error_ = new TypeError(error);
                    throw error_;
                }
            }", None),
("function unicorn() {
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
                function foo() {
                    throw error;
                }
            }", "function unicorn() {
                const error_ = test ? a : b;
                throw error_;
                function foo() {
                    throw error;
                }
            }", None),
("function unicorn() {
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }", "function unicorn() {
                const error = test ? a : b;
                throw error;
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }", None),
("function unicorn() {
                const error = test ? a : b;
                throw error;
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
            }", "function unicorn() {
                const error = test ? a : b;
                throw error;
                const error_ = test ? a : b;
                throw error_;
            }", None),
("function outer() {
                if (test) {
                    throw a;
                } else {
                    throw b;
                }
                function inner() {
                    if (test) {
                        throw a;
                    } else {
                        throw b;
                    }
                }
            }", "function outer() {
                const error = test ? a : b;
                throw error;
                function inner() {
                    if (test) {
                        throw a;
                    } else {
                        throw b;
                    }
                }
            }", None),
("function outer() {
                const error = test ? a : b;
                throw error;
                function inner() {
                    if (test) {
                        throw a;
                    } else {
                        throw b;
                    }
                }
            }", "function outer() {
                const error = test ? a : b;
                throw error;
                function inner() {
                    const error_ = test ? a : b;
                    throw error_;
                }
            }", None),
("while (foo) if (test) {throw a} else {throw b}", "while (foo) {
             const error = test ? a : b;
             throw error;
            }", None),
("function unicorn() {
                if(test){
                    foo = a;
                } else{
                    foo = b;
                }
            }", "function unicorn() {
                foo = test ? a : b;
            }", None),
("function unicorn() {
                if(test){
                    foo *= a;
                } else{
                    foo *= b;
                }
            }", "function unicorn() {
                foo *= test ? a : b;
            }", None),
("async function unicorn() {
                if(test){
                    foo = await a;
                } else{
                    foo = b;
                }
            }", "async function unicorn() {
                foo = test ? (await a) : b;
            }", None),
("async function unicorn() {
                if(test){
                    foo = await a;
                } else{
                    foo = await b;
                }
            }", "async function unicorn() {
                foo = await (test ? a : b);
            }", None),
("function unicorn() {
                if (test) {
                    foo.bar = a;
                } else{
                    foo.bar = b;
                }
            }", "function unicorn() {
                foo.bar = test ? a : b;
            }", None),
("function unicorn() {
                a()
                if (test) {
                    (foo)['b' + 'ar'] = a
                } else{
                    foo.bar = b
                }
            }", "function unicorn() {
                a()
                ;(foo)['b' + 'ar'] = test ? a : b;
            }", None),
("async function* unicorn() {
                if(test){
                    foo = yield await a;
                } else{
                    foo = yield await b;
                }
            }", "async function* unicorn() {
                foo = yield (await (test ? a : b));
            }", None),
("if(test){
                $0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 =
                _STOP_ =
                $0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 =
                1;
            } else{
                $0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 =
                _STOP_2_ =
                $0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 =
                2;
            }", "$0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 = test ? (_STOP_ =
                $0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 =
                1) : (_STOP_2_ =
                $0 |= $1 ^= $2 &= $3 >>>= $4 >>= $5 <<= $6 %= $7 /= $8 *= $9 **= $10 -= $11 += $12 =
                2);", None),
("if (test) {
                a = foo;
            } else {
                a = bar;
            }", "a = test ? foo : bar;", Some(serde_json::json!(only_single_line_options))),
("if (
                (
                    test
                )
            ) {
                a = foo;
            } else {
                a = bar;
            }", "a = (
                    test
                ) ? foo : bar;", Some(serde_json::json!(only_single_line_options))),
("if (test) {
                (
                    a = foo
                );
            } else {
                a = bar;
            }", "a = test ? foo : bar;", Some(serde_json::json!(only_single_line_options))),
("if (test) {
                a = foo
                ;
            } else {
                a = bar;
            }", "a = test ? foo : bar;", Some(serde_json::json!(only_single_line_options))),
("if (test) {
                ;;;;;;
                a = foo;
                ;;;;;;
            } else {
                a = bar;
            }", "a = test ? foo : bar;", Some(serde_json::json!(only_single_line_options))),
("function unicorn() {
                // There is an empty block inside consequent
                if (test) {
                    ;
                    return a;
                } else {
                    return b;
                }
            }", "function unicorn() {
                // There is an empty block inside consequent
                return test ? a : b;
            }", None),
("function unicorn() {
                if (test) {
                    foo = a
                } else foo = b;
            }", "function unicorn() {
                foo = test ? a : b;
            }", None),
("function unicorn() {
                if (test) return a;
                else return b;
            }", "function unicorn() {
                return test ? a : b;
            }", None),
("if (a = b) {
                foo = 1;
            } else foo = 2;", "foo = (a = b) ? 1 : 2;", None),
("function* unicorn() {
                if (yield a) {
                    foo = 1;
                } else foo = 2;
            }", "function* unicorn() {
                foo = (yield a) ? 1 : 2;
            }", None),
("function* unicorn() {
                if (yield* a) {
                    foo = 1;
                } else foo = 2;
            }", "function* unicorn() {
                foo = (yield* a) ? 1 : 2;
            }", None),
("function foo(){
                if (a) {
                    return 1;
                } else {
                    if (b) {
                        return 2;
                    } else {
                        return 3;
                    }
                }
            }", "function foo(){
                if (a) {
                    return 1;
                } else {
                    return b ? 2 : 3;
                }
            }", None),
("function foo(){
                if (a) {
                    if (b) {
                        return 1;
                    } else {
                        return 2;
                    }
                } else {
                    return 3;
                }
            }", "function foo(){
                if (a) {
                    return b ? 1 : 2;
                } else {
                    return 3;
                }
            }", None)
    ];

    Tester::new(PreferTernary::NAME, PreferTernary::PLUGIN, pass, fail).test_and_snapshot();
}
