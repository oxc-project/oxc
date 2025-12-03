use oxc_ast::{
    AstKind,
    ast::{Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_spy_on_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Overwriting a property to mock it is discouraged")
        .with_help("Use vi.spyOn(object, property) instead of reassigning the property")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSpyOn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that all mocking is done using `vi.spyOn`.
    ///
    /// ### Why is this bad?
    ///
    /// Directly reassigning properties (e.g., obj.fn = vi.fn()) can break getters/setters,
    /// lose original property descriptors, and fail with ESM read-only bindings.
    /// Using `vi.spyOn()` makes the mocked property explicit, preserves the original descriptor,
    /// and provides a built-in API to restore the original method without manually backing it up.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Date.now = vi.fn()
    /// Date.now = vi.fn(() => 10)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// vi.spyOn(Date, 'now')
    /// vi.spyOn(Date, 'now').mockImplementation(() => 10)
    /// ```
    PreferSpyOn,
    vitest,
    style,
    suggestion,
);

fn is_vi_fn_call(node: &Expression<'_>) -> bool {
    match node {
        Expression::StaticMemberExpression(static_expression) => {
            is_vi_fn_call(&static_expression.object)
        }
        Expression::CallExpression(call) => call.callee.is_specific_member_access("vi", "fn"),
        _ => false,
    }
}

fn get_callee<'a>(node: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    match node {
        Expression::CallExpression(call) => Some(&call.callee),
        _ => None,
    }
}

fn find_node_object<'a>(node: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    if let Some(callee) = get_callee(node) {
        return find_node_object(callee);
    }

    if let Some(member) = node.as_member_expression() {
        return Some(member.object());
    }

    None
}

fn get_vitest_fn_call<'a>(node: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    let call = node.is_call_expression() || node.is_member_expression();

    if !call {
        return None;
    }

    let object = find_node_object(node)?;

    match object {
        Expression::Identifier(_) => {
            let is_call_expression = node.is_call_expression() && is_vi_fn_call(node);
            if is_call_expression { Some(node) } else { None }
        }
        _ => get_vitest_fn_call(object),
    }
}

fn parent_has_mock_implementation_call<'a>(assignment: &'a Expression<'a>) -> bool {
    let Some(outer_callee) = get_callee(assignment) else {
        return false;
    };

    let Some(static_member) = outer_callee.as_member_expression() else {
        return false;
    };

    let Some(property_name) = static_member.static_property_name() else {
        return false;
    };

    property_name == "mockImplementation"
}

fn auto_fix_mock_implementation<'a>(
    right_assignment: &'a Expression<'a>,
    fn_call_node: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> String {
    if parent_has_mock_implementation_call(right_assignment) {
        return String::new();
    }

    match fn_call_node {
        Expression::CallExpression(call) => {
            let Some(arguments_span) = call.arguments_span() else {
                return ".mockImplementation()".to_string();
            };

            format!(".mockImplementation({})", ctx.source_range(arguments_span))
        }
        _ => String::new(),
    }
}

impl Rule for PreferSpyOn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::AssignmentExpression(assignment) = node.kind() {
            if !assignment.left.is_member_expression() {
                return;
            }

            let result = get_vitest_fn_call(&assignment.right);

            if let Some(fn_call_node) = result {
                ctx.diagnostic_with_suggestion(
                    prefer_spy_on_diagnostic(assignment.span),
                    |fixer| {
                        let mock_implementation =
                            auto_fix_mock_implementation(&assignment.right, fn_call_node, ctx);
                        let left_expression = assignment.left.to_member_expression();
                        let fixer = fixer.for_multifix();
                        let mut rule_fixes = fixer.new_fix_with_capacity(3);

                        match left_expression {
                            MemberExpression::ComputedMemberExpression(left)
                                if left.object.is_identifier_reference() =>
                            {
                                rule_fixes
                                    .push(fixer.insert_text_before_range(left.span, "vi.spyOn("));

                                let identifier = {
                                    match &left.object {
                                        Expression::Identifier(id) => id.span,
                                        _ => Span::empty(0),
                                    }
                                };

                                let property = {
                                    match &left.expression {
                                        Expression::StringLiteral(property) => property.span,
                                        Expression::TemplateLiteral(property) => property.span,
                                        Expression::BinaryExpression(property) => property.span,
                                        Expression::Identifier(property) => property.span,
                                        _ => Span::empty(0),
                                    }
                                };

                                rule_fixes.push(
                                    fixer.replace(Span::new(identifier.end, property.start), ", "),
                                );

                                let call_span = {
                                    match fn_call_node {
                                        Expression::CallExpression(call) => call.span,
                                        _ => Span::empty(0),
                                    }
                                };

                                let end_spy = format!("){mock_implementation}");

                                rule_fixes.push(
                                    fixer.replace(Span::new(property.end, call_span.end), end_spy),
                                );
                            }
                            MemberExpression::StaticMemberExpression(left) => {
                                rule_fixes
                                    .push(fixer.insert_text_before_range(left.span, "vi.spyOn("));

                                let identifier = {
                                    match &left.object {
                                        Expression::Identifier(id) => id.span,
                                        Expression::StaticMemberExpression(chain) => chain.span,
                                        _ => Span::empty(0),
                                    }
                                };

                                rule_fixes.push(fixer.replace(
                                    Span::new(identifier.end, left.property.span.start),
                                    ", '",
                                ));

                                let call_span = {
                                    match fn_call_node {
                                        Expression::CallExpression(call) => call.span,
                                        _ => Span::empty(0),
                                    }
                                };

                                let end_spy = format!("'){mock_implementation}");

                                rule_fixes.push(fixer.replace(
                                    Span::new(left.property.span.end, call_span.end),
                                    end_spy,
                                ));
                            }
                            _ => {
                                let _ = fixer.noop();
                            }
                        }
                        rule_fixes.with_message("Convert to \"vi.spyOn\"")
                    },
                );
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Date.now = () => 10",
        "window.fetch = vi.fn",
        "Date.now = fn()",
        "obj.mock = vi.something()",
        "const mock = vi.fn()",
        "mock = vi.fn()",
        "const mockObj = { mock: vi.fn() }",
        "mockObj = { mock: vi.fn() }",
        "window[`${name}`] = vi[`fn${expression}`]()",
    ];

    let fail = vec![
        "obj.a = vi.fn(); const test = 10;",
        "Date['now'] = vi['fn']()",
        "window[`${name}`] = vi[`fn`]()",
        "obj['prop' + 1] = vi['fn']()",
        "obj.one.two = vi.fn(); const test = 10;",
        "obj.a = vi.fn(() => 10,)", // { "parserOptions": { "ecmaVersion": 2017 } },
        "obj.a.b = vi.fn(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
        "window.fetch = vi.fn(() => ({})).one.two().three().four",
        "foo[bar] = vi.fn().mockReturnValue(undefined)",
        "
			        foo.bar = vi.fn().mockImplementation(baz => baz)
			        foo.bar = vi.fn(a => b).mockImplementation(baz => baz)
			      ",
    ];

    let fix = vec![
        (
            "obj.a = vi.fn(); const test = 10;",
            "vi.spyOn(obj, 'a').mockImplementation(); const test = 10;",
            None,
        ),
        ("Date['now'] = vi['fn']()", "vi.spyOn(Date, 'now').mockImplementation()", None),
        (
            "window[`${name}`] = vi[`fn`]()",
            "vi.spyOn(window, `${name}`).mockImplementation()",
            None,
        ),
        ("obj['prop' + 1] = vi['fn']()", "vi.spyOn(obj, 'prop' + 1).mockImplementation()", None),
        (
            "obj.one.two = vi.fn(); const test = 10;",
            "vi.spyOn(obj.one, 'two').mockImplementation(); const test = 10;",
            None,
        ),
        ("obj.a = vi.fn(() => 10,)", "vi.spyOn(obj, 'a').mockImplementation(() => 10)", None),
        (
            "obj.a.b = vi.fn(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            "vi.spyOn(obj.a, 'b').mockImplementation(() => ({})).mockReturnValue('default').mockReturnValueOnce('first call'); test();",
            None,
        ),
        (
            "window.fetch = vi.fn(() => ({})).one.two().three().four",
            "vi.spyOn(window, 'fetch').mockImplementation(() => ({})).one.two().three().four",
            None,
        ),
        (
            "foo[bar] = vi.fn().mockReturnValue(undefined)",
            "vi.spyOn(foo, bar).mockImplementation().mockReturnValue(undefined)",
            None,
        ),
        (
            "
			        foo.bar = vi.fn().mockImplementation(baz => baz)
			        foo.bar = vi.fn(a => b).mockImplementation(baz => baz)
			      ",
            "
			        vi.spyOn(foo, 'bar').mockImplementation(baz => baz)
			        vi.spyOn(foo, 'bar').mockImplementation(baz => baz)
			      ",
            None,
        ),
    ];
    Tester::new(PreferSpyOn::NAME, PreferSpyOn::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
