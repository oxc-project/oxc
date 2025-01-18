use oxc_ast::{
    ast::{TSAsExpression, TSType, TSTypeAssertion, TSTypeName, TSTypeReference},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use phf::{phf_set, Set};

use crate::{ast_util::outermost_paren_parent, context::LintContext, rule::Rule, AstNode};

fn use_jest_mocked(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `jest.mocked()` over `fn as jest.Mock`.")
        .with_help("Prefer `jest.mocked()`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferJestMocked;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When working with mocks of functions using Jest, it's recommended to use the
    /// `jest.mocked()` helper function to properly type the mocked functions. This rule
    /// enforces the use of `jest.mocked()` for better type safety and readability.
    ///
    /// Restricted types:
    ///
    ///
    /// - `jest.Mock`
    /// - `jest.MockedFunction`
    /// - `jest.MockedClass`
    /// - `jest.MockedObject`
    ///
    /// ### Examples
    ///
    /// ```typescript
    /// // invalid
    /// (foo as jest.Mock).mockReturnValue(1);
    /// const mock = (foo as jest.Mock).mockReturnValue(1);
    /// (foo as unknown as jest.Mock).mockReturnValue(1);
    /// (Obj.foo as jest.Mock).mockReturnValue(1);
    /// ([].foo as jest.Mock).mockReturnValue(1);
    ///
    /// // valid
    /// jest.mocked(foo).mockReturnValue(1);
    /// const mock = jest.mocked(foo).mockReturnValue(1);
    /// jest.mocked(Obj.foo).mockReturnValue(1);
    /// jest.mocked([].foo).mockReturnValue(1);
    /// ```
    PreferJestMocked,
    jest,
    style,
    conditional_fix
);

impl Rule for PreferJestMocked {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::TSAsExpression(ts_expr) = node.kind() {
            if !matches!(ctx.nodes().parent_kind(node.id()), Some(AstKind::TSAsExpression(_))) {
                Self::check_ts_as_expression(node, ts_expr, ctx);
            }
        } else if let AstKind::TSTypeAssertion(assert_type) = node.kind() {
            Self::check_assert_type(node, assert_type, ctx);
        }
    }
}

const MOCK_TYPES: Set<&'static str> = phf_set! {
    "Mock",
    "MockedFunction",
    "MockedClass",
    "MockedObject",
};

impl PreferJestMocked {
    fn check_ts_as_expression<'a>(
        node: &AstNode<'a>,
        as_expr: &TSAsExpression,
        ctx: &LintContext<'a>,
    ) {
        let TSType::TSTypeReference(ts_reference) = &as_expr.type_annotation else {
            return;
        };
        let arg_span = as_expr.expression.get_inner_expression().span();
        Self::check(node, ts_reference, arg_span, as_expr.span, ctx);
    }

    fn check_assert_type<'a>(
        node: &AstNode<'a>,
        assert_type: &TSTypeAssertion,
        ctx: &LintContext<'a>,
    ) {
        let TSType::TSTypeReference(ts_reference) = &assert_type.type_annotation else {
            return;
        };
        let arg_span = assert_type.expression.get_inner_expression().span();
        Self::check(node, ts_reference, arg_span, assert_type.span, ctx);
    }

    fn check<'a>(
        node: &AstNode<'a>,
        ts_reference: &TSTypeReference,
        arg_span: Span,
        span: Span,
        ctx: &LintContext<'a>,
    ) {
        let TSTypeName::QualifiedName(qualified_name) = &ts_reference.type_name else {
            return;
        };
        let TSTypeName::IdentifierReference(ident) = &qualified_name.left else {
            return;
        };

        if !&ident.name.eq_ignore_ascii_case("jest")
            || !MOCK_TYPES.contains(qualified_name.right.name.as_str())
        {
            return;
        }

        if can_fix(node, ctx) {
            ctx.diagnostic_with_fix(use_jest_mocked(span), |fixer| {
                let span_source_code = fixer.source_range(arg_span);
                fixer.replace(span, format!("jest.mocked({span_source_code})"))
            });
        } else {
            ctx.diagnostic(use_jest_mocked(span));
        }
    }
}

fn can_fix<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    outermost_paren_parent(node, ctx)
        .is_some_and(|parent| !matches!(parent.kind(), AstKind::SimpleAssignmentTarget(_)))
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        ("foo();", None, None, None),
        ("jest.mocked(foo).mockReturnValue(1);", None, None, None),
        ("bar.mockReturnValue(1);", None, None, None),
        ("sinon.stub(foo).returns(1);", None, None, None),
        ("foo.mockImplementation(() => 1);", None, None, None),
        ("obj.foo();", None, None, None),
        ("mockFn.mockReturnValue(1);", None, None, None),
        ("arr[0]();", None, None, None),
        ("obj.foo.mockReturnValue(1);", None, None, None),
        ("jest.spyOn(obj, 'foo').mockReturnValue(1);", None, None, None),
        ("(foo as Mock.jest).mockReturnValue(1);", None, None, None),
        (
            "
                type MockType = jest.Mock;
                const mockFn = jest.fn();
                (mockFn as MockType).mockReturnValue(1);
            ",
            None,
            None,
            None,
        ),
    ];

    let fail = vec![
        ("(foo as jest.Mock).mockReturnValue(1);", None, None, None),
        (
            "(foo as unknown as string as unknown as jest.Mock).mockReturnValue(1);",
            None,
            None,
            None,
        ),
        (
            "(foo as unknown as jest.Mock as unknown as jest.Mock).mockReturnValue(1);",
            None,
            None,
            None,
        ),
        (
            "(<jest.Mock>foo).mockReturnValue(1);",
            None,
            None,
            Some(PathBuf::from("/prefer-jest-mocked.ts")),
        ),
        ("(foo as jest.Mock).mockImplementation(1);", None, None, None),
        ("(foo as unknown as jest.Mock).mockReturnValue(1);", None, None, None),
        (
            "(<jest.Mock>foo as unknown).mockReturnValue(1);",
            None,
            None,
            Some(PathBuf::from("/prefer-jest-mocked.ts")),
        ),
        ("(Obj.foo as jest.Mock).mockReturnValue(1);", None, None, None),
        ("([].foo as jest.Mock).mockReturnValue(1);", None, None, None),
        ("(foo as jest.MockedFunction).mockReturnValue(1);", None, None, None),
        ("(foo as jest.MockedFunction).mockImplementation(1);", None, None, None),
        ("(foo as unknown as jest.MockedFunction).mockReturnValue(1);", None, None, None),
        ("(Obj.foo as jest.MockedFunction).mockReturnValue(1);", None, None, None),
        (
            "(new Array(0).fill(null).foo as jest.MockedFunction).mockReturnValue(1);",
            None,
            None,
            None,
        ),
        ("(jest.fn(() => foo) as jest.MockedFunction).mockReturnValue(1);", None, None, None),
        (
            "const mockedUseFocused = useFocused as jest.MockedFunction<typeof useFocused>;",
            None,
            None,
            None,
        ),
        (
            "const filter = (MessageService.getMessage as jest.Mock).mock.calls[0][0];",
            None,
            None,
            None,
        ),
        (
            "
                class A {}
                (foo as jest.MockedClass<A>)
            ",
            None,
            None,
            None,
        ),
        ("(foo as jest.MockedObject<{method: () => void}>)", None, None, None),
        ("(Obj['foo'] as jest.MockedFunction).mockReturnValue(1);", None, None, None),
        (
            "
                (
                new Array(100)
                    .fill(undefined)
                    .map(x => x.value)
                    .filter(v => !!v).myProperty as jest.MockedFunction<{
                    method: () => void;
                }>
                ).mockReturnValue(1);
            ",
            None,
            None,
            None,
        ),
    ];

    let fix = vec![
        ("(foo as jest.Mock).mockReturnValue(1);", "(jest.mocked(foo)).mockReturnValue(1);"),
        (
            "(foo as unknown as string as unknown as jest.Mock).mockReturnValue(1);",
            "(jest.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(foo as unknown as jest.Mock as unknown as jest.Mock).mockReturnValue(1);",
            "(jest.mocked(foo)).mockReturnValue(1);",
        ),
        // Note: couldn't fix
        // Todo: this need to fixer support option configuration.
        // (
        //     "(<jest.Mock>foo).mockReturnValue(1);",
        //     "(jest.mocked(foo)).mockReturnValue(1);",
        // ),
        ("(foo as jest.Mock).mockImplementation(1);", "(jest.mocked(foo)).mockImplementation(1);"),
        (
            "(foo as unknown as jest.Mock).mockReturnValue(1);",
            "(jest.mocked(foo)).mockReturnValue(1);",
        ),
        // Note: couldn't fix
        // Todo: this need to fixer support option configuration.
        // (
        //     "(<jest.Mock>foo as unknown).mockReturnValue(1);",
        //     "(jest.mocked(foo) as unknown).mockReturnValue(1);",
        // ),
        (
            "(Obj.foo as jest.Mock).mockReturnValue(1);",
            "(jest.mocked(Obj.foo)).mockReturnValue(1);",
        ),
        ("([].foo as jest.Mock).mockReturnValue(1);", "(jest.mocked([].foo)).mockReturnValue(1);"),
        (
            "(foo as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(foo as jest.MockedFunction).mockImplementation(1);",
            "(jest.mocked(foo)).mockImplementation(1);",
        ),
        (
            "(foo as unknown as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(foo)).mockReturnValue(1);",
        ),
        (
            "(Obj.foo as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(Obj.foo)).mockReturnValue(1);",
        ),
        (
            "(new Array(0).fill(null).foo as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(new Array(0).fill(null).foo)).mockReturnValue(1);",
        ),
        (
            "(jest.fn(() => foo) as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(jest.fn(() => foo))).mockReturnValue(1);",
        ),
        (
            "const mockedUseFocused = useFocused as jest.MockedFunction<typeof useFocused>;",
            "const mockedUseFocused = jest.mocked(useFocused);",
        ),
        (
            "const filter = (MessageService.getMessage as jest.Mock).mock.calls[0][0];",
            "const filter = (jest.mocked(MessageService.getMessage)).mock.calls[0][0];",
        ),
        (
            "
                class A {}
                (foo as jest.MockedClass<A>)
            ",
            "
                class A {}
                (jest.mocked(foo))
            ",
        ),
        ("(foo as jest.MockedObject<{method: () => void}>)", "(jest.mocked(foo))"),
        (
            "(Obj['foo'] as jest.MockedFunction).mockReturnValue(1);",
            "(jest.mocked(Obj['foo'])).mockReturnValue(1);",
        ),
        (
            "
                (
                new Array(100)
                    .fill(undefined)
                    .map(x => x.value)
                    .filter(v => !!v).myProperty as jest.MockedFunction<{
                    method: () => void;
                }>
                ).mockReturnValue(1);
            ",
            "
                (
                jest.mocked(new Array(100)
                    .fill(undefined)
                    .map(x => x.value)
                    .filter(v => !!v).myProperty)
                ).mockReturnValue(1);
            ",
        ),
        // we can't fix this case, as fixing it would result in a syntax error
        // (we'd be attempting attempting to assign `jest.fn()` to invalid left-hand side)
        ("(foo as jest.Mock) = jest.fn();", "(foo as jest.Mock) = jest.fn();"),
    ];

    Tester::new(PreferJestMocked::NAME, PreferJestMocked::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
