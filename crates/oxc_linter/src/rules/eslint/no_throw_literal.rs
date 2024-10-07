use oxc_ast::{
    ast::{AssignmentOperator, Expression, LogicalOperator, TSType},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_throw_literal_diagnostic(span: Span, is_undef: bool) -> OxcDiagnostic {
    let message =
        if is_undef { "Do not throw undefined" } else { "Expected an error object to be thrown" };

    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoThrowLiteral;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows throwing literals or non-Error objects as exceptions.
    ///
    /// ### Why is this bad?
    ///
    /// It is considered good practice to only throw the Error object itself or an object using
    /// the Error object as base objects for user-defined exceptions. The fundamental benefit of
    /// Error objects is that they automatically keep track of where they were built and originated.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// throw "error";
    ///
    /// throw 0;
    ///
    /// throw undefined;
    ///
    /// throw null;
    ///
    /// var err = new Error();
    /// throw "an " + err;
    /// // err is recast to a string literal
    ///
    /// var err = new Error();
    /// throw `${err}`
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// throw new Error();
    ///
    /// throw new Error("error");
    ///
    /// var e = new Error("error");
    /// throw e;
    ///
    /// try {
    ///     throw new Error("error");
    /// } catch (e) {
    ///     throw e;
    /// }
    /// ```
    NoThrowLiteral,
    pedantic,
    conditional_suggestion,
);

const SPECIAL_IDENTIFIERS: [&str; 3] = ["undefined", "Infinity", "NaN"];
impl Rule for NoThrowLiteral {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ThrowStatement(stmt) = node.kind() else {
            return;
        };

        let expr = &stmt.argument;

        match expr.get_inner_expression() {
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => {
                let span = expr.span();
                ctx.diagnostic_with_suggestion(no_throw_literal_diagnostic(span, false), |fixer| {
                    fixer.replace(
                        span,
                        format!("new Error({})", span.source_text(ctx.source_text())),
                    )
                });
            }
            Expression::Identifier(id) if SPECIAL_IDENTIFIERS.contains(&id.name.as_str()) => {
                ctx.diagnostic(no_throw_literal_diagnostic(expr.span(), true));
            }
            expr if !Self::could_be_error(ctx, expr) => {
                ctx.diagnostic(no_throw_literal_diagnostic(expr.span(), false));
            }
            _ => {}
        }
    }
}

impl NoThrowLiteral {
    fn could_be_error(ctx: &LintContext, expr: &Expression) -> bool {
        match expr.get_inner_expression() {
            Expression::NewExpression(_)
            | Expression::AwaitExpression(_)
            | Expression::CallExpression(_)
            | Expression::ChainExpression(_)
            | Expression::YieldExpression(_)
            | Expression::PrivateFieldExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::TaggedTemplateExpression(_) => true,
            Expression::AssignmentExpression(expr) => {
                if matches!(
                    expr.operator,
                    AssignmentOperator::Assign | AssignmentOperator::LogicalAnd
                ) {
                    return Self::could_be_error(ctx, &expr.right);
                }

                if matches!(
                    expr.operator,
                    AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish
                ) {
                    return expr
                        .left
                        .get_expression()
                        .map_or(true, |expr| Self::could_be_error(ctx, expr))
                        || Self::could_be_error(ctx, &expr.right);
                }

                false
            }
            Expression::SequenceExpression(expr) => {
                expr.expressions.last().is_some_and(|expr| Self::could_be_error(ctx, expr))
            }
            Expression::LogicalExpression(expr) => {
                if matches!(expr.operator, LogicalOperator::And) {
                    return Self::could_be_error(ctx, &expr.right);
                }

                Self::could_be_error(ctx, &expr.left) || Self::could_be_error(ctx, &expr.right)
            }
            Expression::ConditionalExpression(expr) => {
                Self::could_be_error(ctx, &expr.consequent)
                    || Self::could_be_error(ctx, &expr.alternate)
            }
            Expression::Identifier(ident) => {
                let Some(ref_id) = ident.reference_id() else {
                    return true;
                };
                let reference = ctx.symbols().get_reference(ref_id);
                let Some(symbol_id) = reference.symbol_id() else {
                    return true;
                };
                let decl = ctx.nodes().get_node(ctx.symbols().get_declaration(symbol_id));
                match decl.kind() {
                    AstKind::VariableDeclarator(decl) => {
                        if let Some(init) = &decl.init {
                            Self::could_be_error(ctx, init)
                        } else {
                            // TODO: warn about throwing undefined
                            false
                        }
                    }
                    AstKind::Function(_)
                    | AstKind::Class(_)
                    | AstKind::TSModuleDeclaration(_)
                    | AstKind::TSEnumDeclaration(_) => false,
                    AstKind::FormalParameter(param) => {
                        !param.pattern.type_annotation.as_ref().is_some_and(|annot| {
                            is_definitely_non_error_type(&annot.type_annotation)
                        })
                    }
                    _ => true,
                }
            }
            _ => false,
        }
    }
}

fn is_definitely_non_error_type(ty: &TSType) -> bool {
    match ty {
        TSType::TSNumberKeyword(_)
        | TSType::TSStringKeyword(_)
        | TSType::TSBooleanKeyword(_)
        | TSType::TSNullKeyword(_)
        | TSType::TSUndefinedKeyword(_) => true,
        TSType::TSUnionType(union) => union.types.iter().all(is_definitely_non_error_type),
        TSType::TSIntersectionType(intersect) => {
            intersect.types.iter().all(is_definitely_non_error_type)
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "throw new Error();",
        "throw new Error('error');",
        "throw Error('error');",
        "var e = new Error(); throw e;",
        "try {throw new Error();} catch (e) {throw e;};",
        "throw a;",
        "throw foo();",
        "throw new foo();",
        "throw foo.bar;",
        "throw foo[bar];",
        "class C { #field; foo() { throw foo.#field; } }", // { "ecmaVersion": 2022 },
        "throw foo = new Error();",
        "throw foo.bar ||= 'literal'",  // { "ecmaVersion": 2021 },
        "throw foo[bar] ??= 'literal'", // { "ecmaVersion": 2021 },
        "throw 1, 2, new Error();",
        "throw 'literal' && new Error();",
        "throw new Error() || 'literal';",
        "throw foo ? new Error() : 'literal';",
        "throw foo ? 'literal' : new Error();",
        "throw tag `${foo}`;", // { "ecmaVersion": 6 },
        "function* foo() { var index = 0; throw yield index++; }", // { "ecmaVersion": 6 },
        "async function foo() { throw await bar; }", // { "ecmaVersion": 8 },
        "throw obj?.foo",      // { "ecmaVersion": 2020 },
        "throw obj?.foo()",    // { "ecmaVersion": 2020 }
        "throw obj?.foo() as string",
        "throw obj?.foo() satisfies Direction",
        // local reference resolution
        "const err = new Error(); throw err;",
        "function main(x) { throw x; }", // cannot determine type of x
        "function main(x: any) { throw x; }",
        "function main(x: TypeError) { throw x; }",
    ];

    let fail = vec![
        "throw 'error';",
        "throw 0;",
        "throw false;",
        "throw null;",
        "throw {};",
        "throw undefined;",
        "throw Infinity;",
        "throw NaN;",
        "throw 'a' + 'b';",
        "var b = new Error(); throw 'a' + b;",
        "throw foo = 'error';",
        "throw foo += new Error();",
        "throw foo &= new Error();",
        "throw foo &&= 'literal'", // { "ecmaVersion": 2021 },
        "throw new Error(), 1, 2, 3;",
        "throw 'literal' && 'not an Error';",
        "throw foo && 'literal'",
        "throw foo ? 'not an Error' : 'literal';",
        "throw `${err}`;", // { "ecmaVersion": 6 }
        "throw 0 as number",
        "throw 'error' satisfies Error",
        // local reference resolution
        "let foo = 'foo'; throw foo;",
        "let foo = 'foo' as unknown as Error; throw foo;",
        "function foo() {}; throw foo;",
        "const foo = () => {}; throw foo;",
        "class Foo {}\nthrow Foo;",
        "function main(x: number) { throw x; }",
        "function main(x: string) { throw x; }",
        "function main(x: string | number) { throw x; }",
    ];

    let fix = vec![
        ("throw 'error';", "throw new Error('error');"),
        ("throw `${err}`;", "throw new Error(`${err}`);"),
        ("throw 'error' satisfies Error", "throw new Error('error' satisfies Error)"),
    ];

    Tester::new(NoThrowLiteral::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
