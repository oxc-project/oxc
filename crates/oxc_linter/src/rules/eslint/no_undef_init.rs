use crate::ast_util::is_shadowed;
use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_ast::ast::{VariableDeclarationKind, VariableDeclarator};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

fn no_undef_init(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("It's not necessary to initialize {name} to undefined."))
        .with_label(span.label("A variable that is declared and not initialized to any value automatically gets the value of undefined."))
}

#[derive(Debug, Default, Clone)]
pub struct NoUndefInit;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow initializing variables to `undefined`.
    ///
    /// ### Why is this bad?
    ///
    /// A variable that is declared and not initialized to any value automatically gets the value of `undefined`.
    /// It’s considered a best practice to avoid initializing variables to `undefined`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var a = undefined;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var a;
    /// ```
    NoUndefInit,
    style,
    fix
);

impl Rule for NoUndefInit {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::VariableDeclaration(var_decl) = node.kind() {
            let faulty_declarations: Vec<&VariableDeclarator> = var_decl
                .declarations
                .iter()
                .filter(|decl| {
                    decl.init.as_ref().is_some_and(oxc_ast::ast::Expression::is_undefined)
                })
                .collect::<Vec<&VariableDeclarator>>();

            if var_decl.kind != VariableDeclarationKind::Const
                && !faulty_declarations.is_empty()
                && !is_shadowed(node.scope_id(), "undefined", ctx)
            {
                for decl in faulty_declarations {
                    if decl.kind == VariableDeclarationKind::Var
                        || decl.id.kind.is_destructuring_pattern()
                        || ctx
                            .semantic()
                            .trivias()
                            .has_comments_between(Span::new(decl.id.span().start, decl.span().end))
                    {
                        let diagnostic_span = Span::new(decl.id.span().end, decl.span().end);
                        let variable_name = &ctx.source_text()[decl.id.span()];
                        ctx.diagnostic(no_undef_init(variable_name, diagnostic_span));
                        continue;
                    }
                    let identifier = decl.id.get_binding_identifier().unwrap();
                    if let Some(init) = &decl.init {
                        let diagnostic_span = Span::new(identifier.span.end, init.span().end);
                        ctx.diagnostic_with_fix(
                            no_undef_init(identifier.name.as_str(), diagnostic_span),
                            |fixer| fixer.delete_range(diagnostic_span),
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a;",
        "const foo = undefined", // { "ecmaVersion": 6 },
        "var undefined = 5; var foo = undefined;",
        "class C { field = undefined; }", // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        "var a = undefined;",
        "var a = undefined, b = 1;",
        "var a = 1, b = undefined, c = 5;",
        "var [a] = undefined;", // { "ecmaVersion": 6 },
        "var {a} = undefined;", // { "ecmaVersion": 6 },
        "for(var i in [1,2,3]){var a = undefined; for(var j in [1,2,3]){}}",
        "let a = undefined;",               // { "ecmaVersion": 6 },
        "let a = undefined, b = 1;",        // { "ecmaVersion": 6 },
        "let a = 1, b = undefined, c = 5;", // { "ecmaVersion": 6 },
        "let [a] = undefined;",             // { "ecmaVersion": 6 },
        "let {a} = undefined;",             // { "ecmaVersion": 6 },
        "for(var i in [1,2,3]){let a = undefined; for(var j in [1,2,3]){}}", // { "ecmaVersion": 6 },
        "let /* comment */a = undefined;", // { "ecmaVersion": 6 },
        "let a/**/ = undefined;",          // { "ecmaVersion": 6 },
        "let a /**/ = undefined;",         // { "ecmaVersion": 6 },
        "let a//
			= undefined;",         // { "ecmaVersion": 6 },
        "let a = /**/undefined;",          // { "ecmaVersion": 6 },
        "let a = //
			undefined;",        // { "ecmaVersion": 6 },
        "let a = undefined/* comment */;", // { "ecmaVersion": 6 },
        "let a = undefined/* comment */, b;", // { "ecmaVersion": 6 },
        "let a = undefined//comment
			, b;", // { "ecmaVersion": 6 }
    ];

    let fix = vec![
        ("let a = undefined;", "let a;", None),
        ("let a = undefined, b = 1;", "let a, b = 1;", None),
        ("let a = 1, b = undefined, c = 5;", "let a = 1, b, c = 5;", None),
        (
            "for(var i in [1,2,3]){let a = undefined; for(var j in [1,2,3]){}}",
            "for(var i in [1,2,3]){let a; for(var j in [1,2,3]){}}",
            None,
        ),
        ("let /* comment */a = undefined;", "let /* comment */a;", None),
        ("let a = undefined/* comment */;", "let a/* comment */;", None),
        ("let a = undefined/* comment */, b;", "let a/* comment */, b;", None),
        (
            "let a = undefined//comment
			, b;",
            "let a//comment
			, b;",
            None,
        ),
    ];
    Tester::new(NoUndefInit::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
