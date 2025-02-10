use oxc_ast::{
    ast::{ArrayExpressionElement, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    ast_util::get_declaration_of_variable, context::LintContext, fixer::Fix, rule::Rule, AstNode,
};

fn prefer_set_size_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Use `Set#size` instead of converting a `Set` to an array and using its `length` property.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSetSize;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `Set#size` over `Set#length` when the `Set` is converted to an array.
    ///
    /// ### Why is this bad?
    ///
    /// Using `Set#size` is more readable and performant.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const length = [...new Set([1, 2, 3])].length;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const size = new Set([1, 2, 3]).size;
    /// ```
    PreferSetSize,
    unicorn,
    correctness,
    fix
);

impl Rule for PreferSetSize {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MemberExpression(member_expr) = node.kind() else {
            return;
        };

        let Some((span, property)) = member_expr.static_property_info() else {
            return;
        };

        if property != "length" || member_expr.optional() || member_expr.is_computed() {
            return;
        }

        let Expression::ArrayExpression(array_expr) = member_expr.object().without_parentheses()
        else {
            return;
        };

        if array_expr.elements.len() != 1 {
            return;
        }

        let ArrayExpressionElement::SpreadElement(spread_element) = &array_expr.elements[0] else {
            return;
        };

        let maybe_set = &spread_element.argument.get_inner_expression();

        if !is_set(maybe_set, ctx) {
            return;
        }

        ctx.diagnostic_with_fix(prefer_set_size_diagnostic(span), |_fixer| {
            vec![
                // remove [...
                Fix::delete(Span::new(array_expr.span.start, spread_element.span.start + 3)),
                // remove everything after the end of the spread element (including the `]` )
                Fix::delete(Span::new(spread_element.span.end, array_expr.span.end)),
                // replace .length with .size
                Fix::new("size", span),
            ]
        });
    }
}

fn is_set<'a>(maybe_set: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    if let Expression::NewExpression(new_expr) = maybe_set {
        if let Expression::Identifier(identifier) = &new_expr.callee {
            return identifier.name == "Set";
        }
        return false;
    }

    let Expression::Identifier(ident) = maybe_set else {
        return false;
    };

    let Some(maybe_decl) = get_declaration_of_variable(ident, ctx) else {
        return false;
    };

    let AstKind::VariableDeclarator(var_decl) = maybe_decl.kind() else {
        return false;
    };

    if !var_decl.kind.is_const() {
        return false;
    }

    if !var_decl.id.kind.is_binding_identifier() {
        return false;
    }

    let Some(init) = &var_decl.init else {
        return false;
    };

    is_new_set(init)
}

fn is_new_set(expr: &Expression) -> bool {
    if let Expression::NewExpression(new_expr) = expr {
        if let Expression::Identifier(identifier) = &new_expr.callee {
            return identifier.name == "Set";
        }
        return false;
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"new Set(foo).size",
        r"for (const foo of bar) console.log([...foo].length)",
        r"[...new Set(array), foo].length",
        r"[foo, ...new Set(array), ].length",
        r"[...new Set(array)].notLength",
        r"[...new Set(array)]?.length",
        r"[...new Set(array)][length]",
        r#"[...new Set(array)]["length"]"#,
        r"[...new NotSet(array)].length",
        r"[...Set(array)].length",
        r"const foo = new NotSet([]);[...foo].length;",
        r"let foo = new Set([]);[...foo].length;",
        r"const {foo} = new Set([]);[...foo].length;",
        r"const [foo] = new Set([]);[...foo].length;",
        r"[...foo].length",
        r"var foo = new Set(); var foo = new Set(); [...foo].length",
        r"[,].length",
    ];

    let fail = vec![
        r"[...new Set(array)].length",
        r"[...new Set(array),].length",
        r"[...(( new Set(array) ))].length",
        r"(( [...new Set(array)] )).length",
        r"[/* comment */...new Set(array)].length",
        r"const foo = new Set([]); [...foo].length;",
        r"[...new /* comment */ Set(array)].length",
    ];

    let fix = vec![
        (r"[...new Set(array)].length", r"new Set(array).size"),
        (r"[...new Set(array),].length", r"new Set(array).size"),
        (r"[...(( new Set(array) ))].length", r"(( new Set(array) )).size"),
        (r"[...(( new Set(array as foo) ))].length", r"(( new Set(array as foo) )).size"),
        (r"[...(( new Set(array) as foo ))].length", r"(( new Set(array) as foo )).size"),
        (
            r"[...(( new Set(array) as foo )     )     ]    .length;",
            r"(( new Set(array) as foo )     )    .size;",
        ),
    ];

    Tester::new(PreferSetSize::NAME, PreferSetSize::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
