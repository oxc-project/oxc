use oxc_ast::{
    AstKind,
    ast::{ArrayExpressionElement, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode, ast_util::get_declaration_of_variable, context::LintContext, fixer::Fix, rule::Rule,
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
        let AstKind::StaticMemberExpression(member_expr) = node.kind() else {
            return;
        };

        let (span, property) = member_expr.static_property_info();
        if property != "length" || member_expr.optional {
            return;
        }

        let Expression::ArrayExpression(array_expr) = member_expr.object.without_parentheses()
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

        ctx.diagnostic_with_fix(prefer_set_size_diagnostic(span), |fixer| {
            let mut fix = fixer
                .new_fix_with_capacity(3)
                .with_message("Remove spread and replace with `Set.size`");
            // remove [...
            fix.push(Fix::delete(Span::new(array_expr.span.start, spread_element.span.start + 3)));
            // remove everything after the end of the spread element (including the `]` )
            fix.push(Fix::delete(Span::new(spread_element.span.end, array_expr.span.end)));
            // replace .length with .size
            fix.push(Fix::new("size", span));
            fix
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

    if !var_decl.id.is_binding_identifier() {
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
        "new Set(foo).size",
        "for (const foo of bar) console.log([...foo].length)",
        "[...new Set(array), foo].length",
        "[foo, ...new Set(array), ].length",
        "[...new Set(array)].notLength",
        "[...new Set(array)]?.length",
        "[...new Set(array)][length]",
        r#"[...new Set(array)]["length"]"#,
        "[...new NotSet(array)].length",
        "[...Set(array)].length",
        "const foo = new NotSet([]);[...foo].length;",
        "let foo = new Set([]);[...foo].length;",
        "const {foo} = new Set([]);[...foo].length;",
        "const [foo] = new Set([]);[...foo].length;",
        "[...foo].length",
        "var foo = new Set(); var foo = new Set(); [...foo].length",
        "[,].length",
        "Array.from(foo).length",
        "Array.from(new NotSet(array)).length",
        "Array.from(Set(array)).length",
        "Array.from(new Set(array)).notLength",
        "Array.from(new Set(array))?.length",
        "Array.from(new Set(array))[length]",
        r#"Array.from(new Set(array))["length"]"#,
        "Array.from(new Set(array), mapFn).length",
        "Array?.from(new Set(array)).length",
        "Array.from?.(new Set(array)).length",
        "const foo = new NotSet([]);Array.from(foo).length;",
        "let foo = new Set([]);Array.from(foo).length;",
        "const {foo} = new Set([]);Array.from(foo).length;",
        "const [foo] = new Set([]);Array.from(foo).length;",
        "var foo = new Set(); var foo = new Set(); Array.from(foo).length",
        "NotArray.from(new Set(array)).length",
    ];

    let fail = vec![
        "[...new Set(array)].length",
        "const foo = new Set([]);
            console.log([...foo].length);",
        "function isUnique(array) {
                return[...new Set(array)].length === array.length
            }",
        "[...new Set(array),].length",
        "[...(( new Set(array) ))].length",
        "(( [...new Set(array)] )).length",
        "foo
            ;[...new Set(array)].length",
        "[/* comment */...new Set(array)].length",
        "[...new /* comment */ Set(array)].length",
        // TODO: Update the rule to handle Array.from cases.
        // "Array.from(new Set(array)).length",
        // "const foo = new Set([]);
        //     console.log(Array.from(foo).length);",
        // "Array.from((( new Set(array) ))).length",
        // "(( Array.from(new Set(array)) )).length",
        // "Array.from(/* comment */ new Set(array)).length",
        // "Array.from(new /* comment */ Set(array)).length",
        // "function isUnique(array) {
        //         return Array.from(new Set(array)).length === array.length
        //     }",
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
