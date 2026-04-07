use oxc_ast::{
    AstKind,
    ast::{ArrayExpressionElement, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{get_declaration_of_variable, is_method_call},
    context::LintContext,
    fixer::Fix,
    rule::Rule,
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

        let Some((conversion_span, maybe_set)) =
            get_set_node(member_expr.object.without_parentheses())
        else {
            return;
        };

        if !is_set(maybe_set.get_inner_expression(), ctx) {
            return;
        }

        if ctx.comments_range(conversion_span.start..conversion_span.end).count()
            > ctx.comments_range(maybe_set.span().start..maybe_set.span().end).count()
        {
            ctx.diagnostic(prefer_set_size_diagnostic(span));
            return;
        }

        ctx.diagnostic_with_fix(prefer_set_size_diagnostic(span), |fixer| {
            let replacement = maybe_set.span().source_text(ctx.source_text());

            let mut fix = fixer
                .new_fix_with_capacity(2)
                .with_message("Replace array conversion with direct `Set.size` access");
            fix.push(Fix::new(replacement.to_string(), conversion_span));
            fix.push(Fix::new("size", span));
            fix
        });
    }
}

fn get_set_node<'a>(expression: &'a Expression<'a>) -> Option<(Span, &'a Expression<'a>)> {
    // `[...set].length`
    if let Expression::ArrayExpression(array_expr) = expression
        && array_expr.elements.len() == 1
        && let ArrayExpressionElement::SpreadElement(spread_element) = &array_expr.elements[0]
    {
        return Some((array_expr.span, &spread_element.argument));
    }

    // `Array.from(set).length`
    if let Expression::CallExpression(call_expr) = expression
        && is_array_from_call(call_expr)
    {
        let set_expr = call_expr.arguments.first()?.as_expression()?;
        return Some((call_expr.span, set_expr));
    }

    None
}

fn is_array_from_call(call_expr: &CallExpression) -> bool {
    if call_expr.optional {
        return false;
    }

    let Some(callee_member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    if callee_member_expr.optional() || callee_member_expr.is_computed() {
        return false;
    }

    is_method_call(call_expr, Some(&["Array"]), Some(&["from"]), Some(1), Some(1))
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
        "Array.from(new Set(array)).length",
        "const foo = new Set([]);
            console.log(Array.from(foo).length);",
        "Array.from((( new Set(array) ))).length",
        "(( Array.from(new Set(array)) )).length",
        "Array.from(/* comment */ new Set(array)).length",
        "Array.from(new /* comment */ Set(array)).length",
        "function isUnique(array) {
                return Array.from(new Set(array)).length === array.length
            }",
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
        (r"Array.from(new Set(array)).length", r"new Set(array).size"),
        (r"Array.from((( new Set(array) ))).length", r"(( new Set(array) )).size"),
        (r"(( Array.from(new Set(array)) )).length", r"(( new Set(array) )).size"),
        (r"Array.from(new /* comment */ Set(array)).length", r"new /* comment */ Set(array).size"),
        (
            r"const foo = new Set([]);
                console.log(Array.from(foo).length);",
            r"const foo = new Set([]);
                console.log(foo.size);",
        ),
        (
            r"function isUnique(array) {
                return Array.from(new Set(array)).length === array.length
            }",
            r"function isUnique(array) {
                return new Set(array).size === array.length
            }",
        ),
        (
            r"foo
                ;[...new Set(array)].length",
            r"foo
                ;new Set(array).size",
        ),
    ];

    Tester::new(PreferSetSize::NAME, PreferSetSize::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
