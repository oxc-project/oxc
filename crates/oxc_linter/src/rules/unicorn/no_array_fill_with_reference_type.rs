use oxc_ast::{
    AstKind,
    ast::{Argument, BindingPattern, Expression, IdentifierReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{get_declaration_of_variable, is_method_call},
    context::LintContext,
    rule::Rule,
    utils::is_regexp_callee,
};

fn no_array_fill_with_reference_type_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use a reference value as the fill value.")
        .with_help("Use `Array.from()` or `.map()` to create a distinct value for each element.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayFillWithReferenceType;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using reference values as `Array#fill()` values.
    ///
    /// ### Why is this bad?
    ///
    /// `Array#fill()` reuses the same value for every array element. When the
    /// fill value is an object, array, class, or most constructed objects, all
    /// elements point at the same reference and mutating one element mutates the
    /// shared value observed by the others.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const rows = new Array(3).fill({});
    /// rows[0].selected = true; // Every row now has `selected`.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const rows = Array.from({ length: 3 }, () => ({}));
    /// ```
    NoArrayFillWithReferenceType,
    unicorn,
    suspicious,
    version = "next",
);

impl Rule for NoArrayFillWithReferenceType {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["fill"]), Some(1), None) {
            return;
        }

        if call_expr.optional {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };
        if member_expr.optional() {
            return;
        }

        let Some(fill_value) = call_expr.arguments.first() else {
            return;
        };

        if !is_reference_fill_value(fill_value, ctx) {
            return;
        }

        ctx.diagnostic(no_array_fill_with_reference_type_diagnostic(fill_value.span()));
    }
}

fn is_reference_fill_value<'a>(fill_value: &'a Argument<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(fill_value) = fill_value.as_expression() else {
        return false;
    };

    if is_reference_expression(Some(fill_value), ctx) {
        return true;
    }

    let initializer = get_const_variable_initializer(fill_value, ctx);

    is_reference_expression(initializer, ctx)
}

fn is_reference_expression<'a>(
    fill_value: Option<&'a Expression<'a>>,
    ctx: &LintContext<'a>,
) -> bool {
    let Some(fill_value) = fill_value.map(Expression::get_inner_expression) else {
        return false;
    };

    match fill_value {
        Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_)
        | Expression::ClassExpression(_) => true,
        Expression::NewExpression(new_expr) => {
            !is_regexp_callee(new_expr.callee.get_inner_expression(), ctx)
        }
        _ => false,
    }
}

fn get_const_variable_initializer<'a>(
    fill_value: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a Expression<'a>> {
    let Expression::Identifier(ident) = fill_value.get_inner_expression() else {
        return None;
    };

    let declaration = get_declaration_of_variable(ident, ctx.semantic())?;
    let AstKind::VariableDeclarator(decl) = declaration.kind() else {
        return None;
    };

    if !decl.kind.is_const() || !is_same_binding_identifier(&decl.id, ident) {
        return None;
    }

    decl.init.as_ref()
}

fn is_same_binding_identifier(
    binding: &BindingPattern<'_>,
    ident: &IdentifierReference<'_>,
) -> bool {
    binding.get_binding_identifier().is_some_and(|binding_ident| binding_ident.name == ident.name)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "array.fill(0)",
        r#"array.fill("x")"#,
        "array.fill(false)",
        "array.fill(null)",
        "array.fill(undefined)",
        "array.fill(10n)",
        r#"array.fill(Symbol("x"))"#,
        r#"array.fill(Symbol.for("x"))"#,
        "array.fill(Symbol.iterator)",
        "array.fill(`x`)",
        "array.fill(() => {})",
        "array.fill(function () {})",
        "array.fill(/x/)",
        r#"array.fill(new RegExp("x"))"#,
        r#"const value = new RegExp("x"); array.fill(value)"#,
        "let value = {}; value = 1; array.fill(value)",
        "var value = {}; array.fill(value)",
        "const value = {}; const alias = value; array.fill(alias)",
        "array.fill(object.value)",
        "array.fill(this.value)",
        "array?.fill({})",
        "array.fill?.({})",
        "array[fill]({})",
        "Array.from({length: 3}, () => value)",
        "Array.from({length: 3}).map(() => value)",
        "const {value = {}} = object; array.fill(value)",
        "function foo(value = {}) { array.fill(value); }",
        "array.fill(/x/ as RegExp)",
        "array.fill((() => {}) as Function)",
        "array.fill(object.value as Foo)",
        "const value = {}; const alias = value as Foo; array.fill(alias)",
    ];

    let fail = vec![
        "new Array(3).fill({})",
        "Array(3).fill([])",
        "Array.from({length: 3}).fill(new Map())",
        "[1, 2, 3].fill(new Set())",
        "const value = {}; array.fill(value)",
        "const value = []; array.fill(value)",
        "const value = new Map(); array.fill(value)",
        "const value = new class {}; array.fill(value)",
        "const RegExp = class {}; array.fill(new RegExp())",
        "array.fill(class {})",
        "const value = {};
            array.fill(value, 1);",
        "array.fill({} as Foo)",
        "array.fill(<Foo>{})",
        "array.fill({} satisfies Foo)",
        "array.fill({}!)",
        "const value = {} as Foo; array.fill(value)",
        "const value = {}; array.fill(value!)",
    ];

    Tester::new(
        NoArrayFillWithReferenceType::NAME,
        NoArrayFillWithReferenceType::PLUGIN,
        pass,
        fail,
    )
    .change_rule_path_extension("ts")
    .test_and_snapshot();
}
