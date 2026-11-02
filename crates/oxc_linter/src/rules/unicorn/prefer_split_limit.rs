use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentTargetPattern, BindingPattern, CallExpression, ChainElement,
        Expression, MemberExpression, UnaryOperator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

/// `index + 1` must stay below the maximum limit accepted by `String#split()`.
const MAXIMUM_INDEX: f64 = 4_294_967_294.0;

fn prefer_split_limit_diagnostic(span: Span, limit: u64) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `String#split()` with a limit.")
        .with_help(format!("Pass `{limit}` as the second argument of `split()`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSplitLimit;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer passing a limit to `String#split()` when only the leading parts are used.
    ///
    /// ### Why is this bad?
    ///
    /// Without a limit, `split()` scans the whole string and allocates every part, even
    /// though only the first few are read.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const first = string.split(',')[0];
    /// const second = string.split(',').at(1);
    /// const [key, value] = string.split('=');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const first = string.split(',', 1)[0];
    /// const second = string.split(',', 2).at(1);
    /// const [key, value] = string.split('=', 2);
    /// const [head, ...tail] = string.split(',');
    /// ```
    PreferSplitLimit,
    unicorn,
    perf,
    fix,
    version = "next",
    short_description = "Prefer `String#split()` with a limit.",
);

impl Rule for PreferSplitLimit {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ComputedMemberExpression(member) => {
                if is_left_hand_side(node, ctx) {
                    return;
                }
                let Some(index) = get_index(&member.expression) else { return };
                check(member.span, &member.object, index + 1, ctx);
            }
            AstKind::CallExpression(call) => {
                let Some(MemberExpression::StaticMemberExpression(member)) =
                    call.callee.without_parentheses().as_member_expression()
                else {
                    return;
                };
                if member.property.name != "at" {
                    return;
                }
                let [argument] = call.arguments.as_slice() else { return };
                let Some(index) = argument.as_expression().and_then(get_index) else { return };
                check(call.span, &member.object, index + 1, ctx);
            }
            AstKind::VariableDeclarator(declarator) => {
                let BindingPattern::ArrayPattern(pattern) = &declarator.id else { return };
                let Some(init) = &declarator.init else { return };
                if pattern.rest.is_none() && !pattern.elements.is_empty() {
                    check(pattern.span, init, pattern.elements.len() as u64, ctx);
                }
            }
            AstKind::AssignmentExpression(assignment) => {
                let Some(AssignmentTargetPattern::ArrayAssignmentTarget(target)) =
                    assignment.left.as_assignment_target_pattern()
                else {
                    return;
                };
                if target.rest.is_none()
                    && !target.elements.is_empty()
                    && is_value_not_usable(node, ctx)
                {
                    let limit = target.elements.len() as u64;
                    check(target.span, &assignment.right, limit, ctx);
                }
            }
            _ => {}
        }
    }
}

fn check<'a>(span: Span, expression: &Expression<'a>, limit: u64, ctx: &LintContext<'a>) {
    let Some(call) = get_split_call_without_limit(expression) else { return };
    let diagnostic = prefer_split_limit_diagnostic(span, limit);
    let close_paren = call.span.end - 1;
    let tail = Span::new(call.arguments[0].span().end, close_paren);
    if ctx.has_comments_between(tail) {
        ctx.diagnostic(diagnostic);
        return;
    }
    let text = if ctx.source_range(tail).contains(',') {
        format!(" {limit},")
    } else {
        format!(", {limit}")
    };
    ctx.diagnostic_with_fix(diagnostic, |fixer| {
        fixer.insert_text_before_range(Span::empty(close_paren), text)
    });
}

fn get_split_call_without_limit<'a, 'b>(
    expression: &'b Expression<'a>,
) -> Option<&'b CallExpression<'a>> {
    let call = match expression.without_parentheses() {
        Expression::CallExpression(call) => call,
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => call,
            _ => return None,
        },
        _ => return None,
    };
    let Some(MemberExpression::StaticMemberExpression(member)) =
        call.callee.without_parentheses().as_member_expression()
    else {
        return None;
    };
    if member.property.name != "split" {
        return None;
    }
    let [separator] = call.arguments.as_slice() else { return None };
    is_supported_separator(separator).then_some(&**call)
}

fn is_supported_separator(argument: &Argument) -> bool {
    match argument {
        Argument::StringLiteral(literal) => !literal.value.is_empty(),
        Argument::RegExpLiteral(_) => true,
        Argument::TemplateLiteral(literal) => {
            literal.single_quasi().is_some_and(|quasi| !quasi.is_empty())
        }
        _ => false,
    }
}

fn get_index(expression: &Expression) -> Option<u64> {
    let Expression::NumericLiteral(literal) = expression.without_parentheses() else {
        return None;
    };
    let value = literal.value;
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    (value >= 0.0 && value.fract() == 0.0 && value < MAXIMUM_INDEX).then_some(value as u64)
}

/// Whether `node`, seen through parentheses and TypeScript wrappers, is written to.
fn is_left_hand_side(node: &AstNode, ctx: &LintContext) -> bool {
    let mut span = node.span();
    for parent in ctx.nodes().ancestors(node.id()) {
        match parent.kind() {
            AstKind::ParenthesizedExpression(_)
            | AstKind::ChainExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_) => span = parent.span(),
            AstKind::AssignmentExpression(assignment) => return assignment.left.span() == span,
            AstKind::ForInStatement(statement) => return statement.left.span() == span,
            AstKind::ForOfStatement(statement) => return statement.left.span() == span,
            AstKind::UnaryExpression(unary) => return unary.operator == UnaryOperator::Delete,
            AstKind::UpdateExpression(_)
            | AstKind::ArrayAssignmentTarget(_)
            | AstKind::ObjectAssignmentTarget(_)
            | AstKind::AssignmentTargetRest(_)
            | AstKind::AssignmentTargetWithDefault(_)
            | AstKind::AssignmentTargetPropertyProperty(_) => return true,
            _ => return false,
        }
    }
    false
}

/// Whether the value of an assignment is discarded, so the missing parts can't be observed.
fn is_value_not_usable(node: &AstNode, ctx: &LintContext) -> bool {
    let mut span = node.span();
    for parent in ctx.nodes().ancestors(node.id()) {
        match parent.kind() {
            AstKind::ParenthesizedExpression(_) => span = parent.span(),
            AstKind::ExpressionStatement(_) => return true,
            AstKind::SequenceExpression(sequence) => {
                if sequence.expressions.last().is_some_and(|last| last.span() != span) {
                    return true;
                }
                span = parent.span();
            }
            _ => return false,
        }
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "string.split(',')",
        "string.split(',', 1)[0]",
        "string.split(',', 2).at(1)",
        "const [a, b] = string.split(',', 2)",
        "string.split('')[0]",
        "string.split(``)[0]",
        "string.split(separator)[0]",
        "string.split(`${a},`)[0]",
        "string.split()[0]",
        "string.split(...args)[0]",
        "string.split(',')[index]",
        "string.split(',')[-1]",
        "string.split(',')[1.5]",
        "string.split(',')[4294967294]",
        "string.split(',').length",
        "string.split(',').at(-1)",
        "string.split(',').at(index)",
        "string.split(',').at()",
        "string.split(',').at(0, 1)",
        "string[split](',')[0]",
        "string.notSplit(',')[0]",
        "const [a, ...rest] = string.split(',')",
        "const [] = string.split(',')",
        "[a, ...rest] = string.split(',')",
        "foo = [a, b] = string.split(',')",
        "string.split(',')[0] = 1",
        "string.split(',')[0]++",
        "delete string.split(',')[0]",
        "[string.split(',')[0]] = foo",
        "({a: string.split(',')[0]} = foo)",
        "for (string.split(',')[0] of foo);",
    ];

    let fail = vec![
        "string.split(',')[0]",
        "string.split(/,/)[2]",
        "string.split(`,`)[0]",
        "string?.split(',')[0]",
        "(string.split(','))[0]",
        "string.split(',').at(1)",
        "string.split(',').at(0)",
        "const [a, b] = string.split(',')",
        "const [, b] = string.split(',')",
        "[a, b] = string.split(',')",
        "foo, [a, b] = string.split(',')",
        "string.split(',',)[0]",
        "string.split(',' /* comment */)[0]",
        "foo(string.split(',')[0], bar)",
    ];

    let fix = vec![
        ("string.split(',')[0]", "string.split(',', 1)[0]"),
        ("string.split(/,/)[2]", "string.split(/,/, 3)[2]"),
        ("string?.split(',')[0]", "string?.split(',', 1)[0]"),
        ("string.split(',').at(1)", "string.split(',', 2).at(1)"),
        ("const [a, b] = string.split(',')", "const [a, b] = string.split(',', 2)"),
        ("const [, b] = string.split(',')", "const [, b] = string.split(',', 2)"),
        ("[a, b] = string.split(',')", "[a, b] = string.split(',', 2)"),
        ("string.split(',',)[0]", "string.split(',', 1,)[0]"),
        ("string.split(',' /* comment */)[0]", "string.split(',' /* comment */)[0]"),
    ];

    Tester::new(PreferSplitLimit::NAME, PreferSplitLimit::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
