use oxc_ast::{
    AstKind,
    ast::{TSAsExpression, TSType, TSTypeAssertion, TSTypeName, TSTypeReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_vi_mocked_diagnostic(span: Span, mock_type: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `vi.mocked()` over type assertions using `{mock_type}`"))
        .with_help("Use `vi.mocked()` instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferViMocked;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require `vi.mocked()` over Vitest mock type assertions.
    ///
    /// ### Why is this bad?
    ///
    /// When working with mocks of functions using Vitest, it's recommended to use the `vi.mocked()` helper function to properly type the mocked functions.
    /// This rule enforces the use of `vi.mocked()` for better type safety and readability.
    ///
    /// Restricted types:
    /// - `Mock`
    /// - `MockedFunction`
    /// - `MockedClass`
    /// - `MockedObject`
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// ;(foo as Mock).mockReturnValue(1)
    /// const mock = (foo as Mock).mockReturnValue(1)
    /// ;(foo as unknown as Mock).mockReturnValue(1)
    /// ;(Obj.foo as Mock).mockReturnValue(1)
    /// ;([].foo as Mock).mockReturnValue(1)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// vi.mocked(foo).mockReturnValue(1)
    /// const mock = vi.mocked(foo).mockReturnValue(1)
    /// vi.mocked(Obj.foo).mockReturnValue(1)
    /// vi.mocked([].foo).mockReturnValue(1)
    /// ```
    PreferViMocked,
    vitest,
    style,
    fix,
    version = "next",
    short_description = "Require `vi.mocked()` over Vitest mock type assertions."
);

impl Rule for PreferViMocked {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSAsExpression(ts_expr) = node.kind() {
            if !matches!(ctx.nodes().parent_kind(node.id()), AstKind::TSAsExpression(_)) {
                check_ts_as_expression(ts_expr, ctx);
            }
        } else if let AstKind::TSTypeAssertion(assert_type) = node.kind() {
            check_assert_type(assert_type, ctx);
        }
    }
}

const MOCK_TYPES: [&str; 4] = ["Mock", "MockedFunction", "MockedClass", "MockedObject"];

fn check_ts_as_expression(as_expr: &TSAsExpression, ctx: &LintContext<'_>) {
    let TSType::TSTypeReference(ts_reference) = &as_expr.type_annotation else {
        return;
    };
    let arg_span = as_expr.expression.get_inner_expression().span();
    check(ts_reference, arg_span, as_expr.span, ctx);
}

fn check_assert_type(assert_type: &TSTypeAssertion, ctx: &LintContext<'_>) {
    let TSType::TSTypeReference(ts_reference) = &assert_type.type_annotation else {
        return;
    };
    let arg_span = assert_type.expression.get_inner_expression().span();
    check(ts_reference, arg_span, assert_type.span, ctx);
}

fn check(ts_reference: &TSTypeReference, arg_span: Span, span: Span, ctx: &LintContext<'_>) {
    let TSTypeName::IdentifierReference(ident_ref) = &ts_reference.type_name else {
        return;
    };

    let mock_type = ident_ref.name.as_str();
    if !MOCK_TYPES.contains(&mock_type) {
        return;
    }

    ctx.diagnostic_with_fix(prefer_vi_mocked_diagnostic(span, mock_type), |fixer| {
        let span_source_code = fixer.source_range(arg_span);
        fixer.replace(span, format!("vi.mocked({span_source_code})"))
    });
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo();",
        "vi.mocked(foo).mockReturnValue(1);",
        "bar.mockReturnValue(1);",
        "sinon.stub(foo).returns(1);",
        "foo.mockImplementation(() => 1);",
        "obj.foo();",
        "mockFn.mockReturnValue(1);",
        "arr[0]();",
        "obj.foo.mockReturnValue(1);",
        r#"vi.spyOn(obj, "foo").mockReturnValue(1);"#,
        "(foo as Mock.vi).mockReturnValue(1);",
        "type MockType = Mock;
            const mockFn = vi.fn();
            (mockFn as MockType).mockReturnValue(1);",
    ];

    let fail = vec![
        "(foo as Mock).mockReturnValue(1);",
        "(foo as unknown as string as unknown as Mock).mockReturnValue(1);",
        "(foo as unknown as Mock as unknown as Mock).mockReturnValue(1);",
        "(<Mock>foo).mockReturnValue(1);",
        "(foo as Mock).mockImplementation(1);",
        "(foo as unknown as Mock).mockReturnValue(1);",
        "(<Mock>foo as unknown).mockReturnValue(1);",
        "(Obj.foo as Mock).mockReturnValue(1);",
        "([].foo as Mock).mockReturnValue(1);",
        "(foo as MockedFunction).mockReturnValue(1);",
        "(foo as MockedFunction).mockImplementation(1);",
        "(foo as unknown as MockedFunction).mockReturnValue(1);",
        "(Obj.foo as MockedFunction).mockReturnValue(1);",
        "(new Array(0).fill(null).foo as MockedFunction).mockReturnValue(1);",
        "(vi.fn(() => foo) as MockedFunction).mockReturnValue(1);",
        "const mockedUseFocused = useFocused as MockedFunction<typeof useFocused>;",
        "const filter = (MessageService.getMessage as Mock).mock.calls[0][0];",
        "class A {}
            (foo as MockedClass<A>)",
        "(foo as MockedObject<{method: () => void}>)",
        r#"(Obj["foo"] as MockedFunction).mockReturnValue(1);"#,
        "(
            new Array(100)
              .fill(undefined)
              .map(x => x.value)
              .filter(v => !!v).myProperty as MockedFunction<{
              method: () => void;
            }>
            ).mockReturnValue(1);",
    ];

    let fix = vec![
        ("(foo as Mock).mockReturnValue(1);", "(vi.mocked(foo)).mockReturnValue(1);"),
        (
            "(foo as unknown as string as unknown as Mock).mockReturnValue(1);",
            "(vi.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(foo as unknown as Mock as unknown as Mock).mockReturnValue(1);",
            "(vi.mocked(foo)).mockReturnValue(1);",
        ),
        ("(<Mock>foo).mockReturnValue(1);", "(vi.mocked(foo)).mockReturnValue(1);"),
        ("(foo as Mock).mockImplementation(1);", "(vi.mocked(foo)).mockImplementation(1);"),
        ("(foo as unknown as Mock).mockReturnValue(1);", "(vi.mocked(foo)).mockReturnValue(1);"),
        (
            "(<Mock>foo as unknown).mockReturnValue(1);",
            "(vi.mocked(foo) as unknown).mockReturnValue(1);",
        ),
        ("(Obj.foo as Mock).mockReturnValue(1);", "(vi.mocked(Obj.foo)).mockReturnValue(1);"),
        ("([].foo as Mock).mockReturnValue(1);", "(vi.mocked([].foo)).mockReturnValue(1);"),
        ("(foo as MockedFunction).mockReturnValue(1);", "(vi.mocked(foo)).mockReturnValue(1);"),
        (
            "(foo as MockedFunction).mockImplementation(1);",
            "(vi.mocked(foo)).mockImplementation(1);",
        ),
        (
            "(foo as unknown as MockedFunction).mockReturnValue(1);",
            "(vi.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(Obj.foo as MockedFunction).mockReturnValue(1);",
            "(vi.mocked(Obj.foo)).mockReturnValue(1);",
        ),
        (
            "(new Array(0).fill(null).foo as MockedFunction).mockReturnValue(1);",
            "(vi.mocked(new Array(0).fill(null).foo)).mockReturnValue(1);",
        ),
        (
            "(vi.fn(() => foo) as MockedFunction).mockReturnValue(1);",
            "(vi.mocked(vi.fn(() => foo))).mockReturnValue(1);",
        ),
        (
            "const mockedUseFocused = useFocused as MockedFunction<typeof useFocused>;",
            "const mockedUseFocused = vi.mocked(useFocused);",
        ),
        (
            "const filter = (MessageService.getMessage as Mock).mock.calls[0][0];",
            "const filter = (vi.mocked(MessageService.getMessage)).mock.calls[0][0];",
        ),
        (
            "class A {}
            (foo as MockedClass<A>)",
            "class A {}
            (vi.mocked(foo))",
        ),
        ("(foo as MockedObject<{method: () => void}>)", "(vi.mocked(foo))"),
        (
            r#"(Obj["foo"] as MockedFunction).mockReturnValue(1);"#,
            r#"(vi.mocked(Obj["foo"])).mockReturnValue(1);"#,
        ),
        (
            "(
            new Array(100)
              .fill(undefined)
              .map(x => x.value)
              .filter(v => !!v).myProperty as MockedFunction<{
              method: () => void;
            }>
            ).mockReturnValue(1);",
            "(
            vi.mocked(new Array(100)
              .fill(undefined)
              .map(x => x.value)
              .filter(v => !!v).myProperty)
            ).mockReturnValue(1);",
        ),
    ];

    Tester::new(PreferViMocked::NAME, PreferViMocked::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .expect_fix(fix)
        .test_and_snapshot();
}
