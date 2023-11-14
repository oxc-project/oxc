use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode, Fix};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(empty-brace-spaces): No spaces inside empty pair of braces allowed")]
#[diagnostic(severity(warning), help("There should be no spaces or new lines inside a pair of empty braces as it affects the overall readability of the code."))]
struct EmptyBraceSpacesDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct EmptyBraceSpaces;

declare_oxc_lint!(
    /// ### What it does
    /// Removes the extra spaces or new line characters inside a pair of braces that does not contain additional code.
    ///
    /// ### Why is this bad?
    /// There should be no spaces inside a pair of braces as it affects the overall readability of the code.
    ///
    /// ### Example
    /// ```javascript
    /// const a = {  };
    /// class A {
    /// }
    /// ```
    EmptyBraceSpaces,
    style
);

impl Rule for EmptyBraceSpaces {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StaticBlock(static_block) => {
                let Span { start, end } = static_block.span;
                if static_block.body.is_empty() && end - start > 9 {
                    // length of "static {}"
                    ctx.diagnostic_with_fix(EmptyBraceSpacesDiagnostic(static_block.span), || {
                        Fix::new("static {}", static_block.span)
                    });
                }
            }
            AstKind::ObjectExpression(obj) => {
                remove_empty_braces_spaces(ctx, obj.properties.is_empty(), obj.span);
            }
            AstKind::FunctionBody(fb) => {
                remove_empty_braces_spaces(ctx, fb.is_empty(), fb.span);
            }
            AstKind::Class(class) => {
                remove_empty_braces_spaces(ctx, class.body.body.is_empty(), class.body.span);
            }
            AstKind::BlockStatement(block_stmt) => {
                remove_empty_braces_spaces(ctx, block_stmt.body.is_empty(), block_stmt.span);
            }
            AstKind::CatchClause(catch_clause) => {
                remove_empty_braces_spaces(
                    ctx,
                    catch_clause.body.body.is_empty(),
                    catch_clause.body.span,
                );
            }
            _ => (),
        };
    }
}

fn remove_empty_braces_spaces(ctx: &LintContext, is_empty_body: bool, span: Span) {
    let Span { start, end } = span;
    if is_empty_body && end - start > 2 {
        // length of "{}"
        ctx.diagnostic_with_fix(EmptyBraceSpacesDiagnostic(span), || Fix::new("{}", span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class A {\nstatic {}\n}",
        "function b() {}",
        "class A {}",
        "const x = () => {};",
        "with (foo) {}",
        "for(let i = 0; i < 3; i += 1) {}",
        "\nif(true){}",
        "try{}catch{}",
        "try{\nconsole.log(\'test\');\n} catch {}",
    ];

    let fail = vec![
        "try{console.log('Hello');}finally{ \n}",
        "const a = {\n};",
        "return {\n\n};",
        "const x = () => {\n\n};",
        "class A {\n}",
        "function a(){ }",
        "do { }while(true)",
        "class A {\nstatic { }\n}",
        "with (foo) {   }",
        "for(let i = 0; i <3; i += 1) {\n  }",
        "\nif (true) {\n}",
        "\nif (true) {   }",
        "try{  }catch{  }",
        "try{\nconsole.log(\'test\');\n}catch{ }\n",
    ];

    let fix = vec![
        ("const a = {\n};", "const a = {};", None),
        ("return {\n\n};", "return {};", None),
        ("const x = () => {\n\n};", "const x = () => {};", None),
        ("class A {\n}", "class A {}", None),
        ("function a(){ }", "function a(){}", None),
        ("do { }while(true)", "do {}while(true)", None),
        ("class A {\nstatic { }\n}", "class A {\nstatic {}\n}", None),
        ("with (foo) {   }", "with (foo) {}", None),
        ("\nif (true) {\n}", "\nif (true) {}", None),
        ("\nif (true) {   }", "\nif (true) {}", None),
        ("try{  }catch{  }", "try{}catch{}", None),
        ("for(let i = 0; i <3; i += 1) {\n  }", "for(let i = 0; i <3; i += 1) {}", None),
        ("try{console.log('Hello');}finally{ \n}", "try{console.log('Hello');}finally{}", None),
        (
            "try{\nconsole.log(\'test\');\n}catch{ }\n",
            "try{\nconsole.log(\'test\');\n}catch{}\n",
            None,
        ),
    ];

    Tester::new_without_config(EmptyBraceSpaces::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
