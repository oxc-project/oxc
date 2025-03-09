use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn empty_brace_spaces_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("No spaces inside empty pair of braces allowed")
        .with_help("There should be no spaces or new lines inside a pair of empty braces as it affects the overall readability of the code.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct EmptyBraceSpaces;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Removes the extra spaces or new line characters inside a pair of braces
    /// that does not contain additional code. This ensures that braces are clean
    /// and do not contain unnecessary spaces or newlines.
    ///
    /// ### Why is this bad?
    ///
    /// Extra spaces inside braces can negatively impact the readability of the code.
    /// Keeping braces clean and free of unnecessary characters improves consistency and
    /// makes the code easier to understand and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const a = {  };
    /// class A {
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const a = {};
    /// class A {}
    /// ```
    EmptyBraceSpaces,
    unicorn,
    style,
    fix
);

impl Rule for EmptyBraceSpaces {
    #[expect(clippy::cast_possible_truncation)]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (is_empty_body, span) = match node.kind() {
            AstKind::StaticBlock(static_block) => {
                let span = static_block.span;

                if static_block.body.is_empty() && !ctx.has_comments_between(span) {
                    // Skip the first 6 chars (static block prefix)
                    let static_block_src = &span.source_text(ctx.source_text())[6..];
                    let left_curly_brace = static_block_src.find('{').unwrap();
                    let static_block_body = &static_block_src[left_curly_brace..];

                    let whitespace_count =
                        static_block_body.chars().take_while(|c| c.is_whitespace()).count();
                    let static_leading_count = (whitespace_count + left_curly_brace + 8) as u32;

                    if span.end - span.start > static_leading_count {
                        ctx.diagnostic_with_fix(empty_brace_spaces_diagnostic(span), |fixer| {
                            fixer.replace(span, "static {}")
                        });
                    }
                }

                return;
            }
            AstKind::ObjectExpression(obj) => (obj.properties.is_empty(), obj.span),
            AstKind::FunctionBody(fb) => (fb.is_empty(), fb.span),
            AstKind::Class(class) => (class.body.body.is_empty(), class.body.span),
            AstKind::BlockStatement(block_stmt) => (block_stmt.body.is_empty(), block_stmt.span),
            _ => return,
        };

        if is_empty_body && span.end - span.start > 2 && !ctx.has_comments_between(span) {
            ctx.diagnostic_with_fix(empty_brace_spaces_diagnostic(span), |fixer| {
                fixer.replace(span, "{}")
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"{}",
        r"function foo(){}",
        r"if(foo) {}",
        r"if(foo) {} else if (bar) {}",
        r"if(foo) {} else {}",
        r"for(;;){}",
        r"for(foo in bar){}",
        r"for(foo of bar){}",
        r"switch (foo) {case bar: {}}",
        r"switch (foo) {default: {}}",
        r"try {} catch(foo){}",
        r"try {} catch(foo){}",
        r"try {} catch(foo){} finally {}",
        r"do {} while (foo)",
        r"while (foo){}",
        r"foo = () => {}",
        r"foo = function (){}",
        r"function foo(){}",
        r"foo = {}",
        r"class Foo {bar() {}}",
        r"foo = class {bar() {}}",
        r"class Foo {static  {}}",
        r"class Foo {}",
        r"foo = class {}",
        r"{/* comment */}",
        r"function foo(){/* comment */}",
        r"if(foo) {/* comment */}",
        r"if(foo) {} else if (bar) {/* comment */}",
        r"if(foo) {} else {/* comment */}",
        r"for(;;){/* comment */}",
        r"for(foo in bar){/* comment */}",
        r"for(foo of bar){/* comment */}",
        r"switch (foo) {case bar: {/* comment */}}",
        r"switch (foo) {default: {/* comment */}}",
        r"try {/* comment */} catch(foo){}",
        r"try {} catch(foo){/* comment */}",
        r"try {} catch(foo){} finally {/* comment */}",
        r"do {/* comment */} while (foo)",
        r"while (foo){/* comment */}",
        r"foo = () => {/* comment */}",
        r"foo = function (){/* comment */}",
        r"function foo(){/* comment */}",
        r"foo = {/* comment */}",
        r"class Foo {bar() {/* comment */}}",
        r"foo = class {bar() {/* comment */}}",
        r"class Foo {static  {/* comment */}}",
        r"class Foo {/* comment */}",
        r"foo = class {/* comment */}",
        "{\n\t// comment \n}",
        "function foo(){\n\t// comment \n}",
        "if(foo) {\n\t// comment \n}",
        "if(foo) {} else if (bar) {\n\t// comment \n}",
        "if(foo) {} else {\n\t// comment \n}",
        "for(;;){\n\t// comment \n}",
        "for(foo in bar){\n\t// comment \n}",
        "for(foo of bar){\n\t// comment \n}",
        "switch (foo) {case bar: {\n\t// comment \n}}",
        "switch (foo) {default: {\n\t// comment \n}}",
        "try {\n\t// comment \n} catch(foo){}",
        "try {} catch(foo){\n\t// comment \n}",
        "try {} catch(foo){} finally {\n\t// comment \n}",
        "do {\n\t// comment \n} while (foo)",
        "while (foo){\n\t// comment \n}",
        "foo = () => {\n\t// comment \n}",
        "foo = function (){\n\t// comment \n}",
        "function foo(){\n\t// comment \n}",
        "foo = {\n\t// comment \n}",
        "class Foo {bar() {\n\t// comment \n}}",
        "foo = class {bar() {\n\t// comment \n}}",
        "class Foo {static  {\n\t// comment \n}}",
        "class Foo {\n\t// comment \n}",
        "foo = class {\n\t// comment \n}",
        r"{unicorn}",
        r"function foo(){unicorn}",
        r"if(foo) {unicorn}",
        r"if(foo) {} else if (bar) {unicorn}",
        r"if(foo) {} else {unicorn}",
        r"for(;;){unicorn}",
        r"for(foo in bar){unicorn}",
        r"for(foo of bar){unicorn}",
        r"switch (foo) {case bar: {unicorn}}",
        r"switch (foo) {default: {unicorn}}",
        r"try {unicorn} catch(foo){}",
        r"try {} catch(foo){unicorn}",
        r"try {} catch(foo){} finally {unicorn}",
        r"do {unicorn} while (foo)",
        r"while (foo){unicorn}",
        r"foo = () => {unicorn}",
        r"foo = function (){unicorn}",
        r"function foo(){unicorn}",
        r"foo = {unicorn}",
        r"class Foo {bar() {unicorn}}",
        r"foo = class {bar() {unicorn}}",
        r"class Foo {static  {unicorn}}",
        r"class Foo {bar() {}}",
        r"foo = class {bar() {}}",
        r"with (foo) {}",
        r"switch (foo) {   }",
        r"const {   } = foo",
        r#"import {   } from "foo""#,
    ];

    let fail = vec![
        r"{ }",
        r"function foo(){ }",
        r"if(foo) { }",
        r"if(foo) {} else if (bar) { }",
        r"if(foo) {} else { }",
        r"for(;;){ }",
        r"for(foo in bar){ }",
        r"for(foo of bar){ }",
        r"switch (foo) {case bar: { }}",
        r"switch (foo) {default: { }}",
        r"try { } catch(foo){}",
        r"try {} catch(foo){ }",
        r"try {} catch(foo){} finally { }",
        r"do { } while (foo)",
        r"while (foo){ }",
        r"foo = () => { }",
        r"foo = function (){ }",
        r"function foo(){ }",
        r"foo = { }",
        r"class Foo {bar() { }}",
        r"foo = class {bar() { }}",
        r"class Foo {static  { }}",
        r"class Foo { }",
        r"foo = class { }",
        "{\t}",
        "function foo(){\t}",
        "if(foo) {\t}",
        "if(foo) {} else if (bar) {\t}",
        "if(foo) {} else {\t}",
        "for(;;){\t}",
        "for(foo in bar){\t}",
        "for(foo of bar){\t}",
        "switch (foo) {case bar: {\t}}",
        "switch (foo) {default: {\t}}",
        "try {\t} catch(foo){}",
        "try {} catch(foo){\t}",
        "try {} catch(foo){} finally {\t}",
        "do {\t} while (foo)",
        "while (foo){\t}",
        "foo = () => {\t}",
        "foo = () => {\t}",
        "foo = function (){\t}",
        "function foo(){\t}",
        "foo = {\t}",
        "class Foo {bar() {\t}}",
        "foo = class {bar() {\t}}",
        "class Foo {static  {\t}}",
        "class Foo {\t}",
        "foo = class {\t}",
        "{ \t \t }",
        "function foo(){ \t \t }",
        "if(foo) { \t \t }",
        "if(foo) {} else if (bar) { \t \t }",
        "if(foo) {} else { \t \t }",
        "for(;;){ \t \t }",
        "for(foo in bar){ \t \t }",
        "for(foo of bar){ \t \t }",
        "switch (foo) {case bar: { \t \t }}",
        "switch (foo) {default: { \t \t }}",
        "try { \t \t } catch(foo){}",
        "try {} catch(foo){ \t \t }",
        "try {} catch(foo){} finally { \t \t }",
        "do { \t \t } while (foo)",
        "while (foo){ \t \t }",
        "foo = () => { \t \t }",
        "foo = function (){ \t \t }",
        "function foo(){ \t \t }",
        "foo = { \t \t }",
        "class Foo {bar() { \t \t }}",
        "foo = class {bar() { \t \t }}",
        "class Foo {static  { \t \t }}",
        "class Foo { \t \t }",
        "foo = class { \t \t }",
        "{\n\n}",
        "function foo(){\n\n}",
        "if(foo) {\n\n}",
        "if(foo) {} else if (bar) {\n\n}",
        "if(foo) {} else {\n\n}",
        "for(;;){\n\n}",
        "for(foo in bar){\n\n}",
        "for(foo of bar){\n\n}",
        "switch (foo) {case bar: {\n\n}}",
        "switch (foo) {default: {\n\n}}",
        "try {\n\n} catch(foo){}",
        "try {} catch(foo){\n\n}",
        "try {} catch(foo){} finally {\n\n}",
        "do {\n\n} while (foo)",
        "while (foo){\n\n}",
        "foo = () => {\n\n}",
        "foo = function (){\n\n}",
        "function foo(){\n\n}",
        "foo = {\n\n}",
        "class Foo {bar() {\n\n}}",
        "foo = class {bar() {\n\n}}",
        "class Foo {static  {\n\n}}",
        "class Foo {\n\n}",
        "foo = class {\n\n}",
        "{\r\n}",
        "function foo(){\r\n}",
        "if(foo) {\r\n}",
        "if(foo) {} else if (bar) {\r\n}",
        "if(foo) {} else {\r\n}",
        "for(;;){\r\n}",
        "for(foo in bar){\r\n}",
        "for(foo of bar){\r\n}",
        "switch (foo) {case bar: {\r\n}}",
        "switch (foo) {default: {\r\n}}",
        "try {\r\n} catch(foo){}",
        "try {} catch(foo){\r\n}",
        "try {} catch(foo){} finally {\r\n}",
        "do {\r\n} while (foo)",
        "while (foo){\r\n}",
        "foo = () => {\r\n}",
        "foo = function (){\r\n}",
        "function foo(){\r\n}",
        "foo = {\r\n}",
        "class Foo {bar() {\r\n}}",
        "foo = class {bar() {\r\n}}",
        "class Foo {static  {\r\n}}",
        "class Foo {\r\n}",
        "foo = class {\r\n}",
        "with (foo) {     }",
    ];

    let fix = vec![
        ("const a = {\n};", "const a = {};", None),
        ("return {\n\n};", "return {};", None),
        ("const x = () => {\n\n};", "const x = () => {};", None),
        ("class A {\n}", "class A {}", None),
        ("function a(){ }", "function a(){}", None),
        ("do { }while(true)", "do {}while(true)", None),
        ("class A {\nstatic { }\n}", "class A {\nstatic {}\n}", None),
        ("class A {\nstatic{ }\n}", "class A {\nstatic {}\n}", None),
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

    Tester::new(EmptyBraceSpaces::NAME, EmptyBraceSpaces::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
