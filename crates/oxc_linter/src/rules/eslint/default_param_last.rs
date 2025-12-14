use oxc_ast::{AstKind, ast::FormalParameter, ast::Function};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn default_param_last_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Default parameters should be last")
        .with_help("Enforce default parameters to be last.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DefaultParamLast;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires default parameters in functions to be the last ones.
    ///
    /// ### Why is this bad?
    ///
    /// Placing default parameters last allows function calls to omit optional trailing arguments,
    /// which improves readability and consistency. This rule applies equally to JavaScript and
    /// TypeScript functions.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /* default-param-last: "error" */
    ///
    /// function f(a = 0, b) {}
    /// function f(a, b = 0, c) {}
    /// function createUser(isAdmin = false, id) {}
    /// createUser(undefined, "tabby")
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /* default-param-last: "error" */
    ///
    /// function f(a, b = 0) {}
    /// function f(a = 0, b = 0) {}
    /// function createUser(id, isAdmin = false) {}
    /// createUser("tabby")
    /// ```
    ///
    /// Examples of **incorrect** TypeScript code for this rule:
    /// ```ts
    /// /* default-param-last: "error" */
    ///
    /// function greet(message: string = "Hello", name: string) {}
    /// function combine(a: number = 1, b: number, c: number) {}
    /// function combine(a: number, b: number = 2, c: number) {}
    /// function combine(a: number = 1, b?: number, c: number) {}
    /// ```
    ///
    /// Examples of **correct** TypeScript code for this rule:
    /// ```ts
    /// /* default-param-last: "error" */
    ///
    /// function greet(name: string, message: string = "Hello") {}
    /// function combine(a: number, b: number = 2, c: number = 3) {}
    /// function combine(a: number, b?: number, c: number = 3) {}
    /// ```
    DefaultParamLast,
    eslint,
    style
);

impl Rule for DefaultParamLast {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(function) if is_function_decl_or_expr(function) => {
                check_params(&function.params.items, ctx);
            }
            AstKind::ArrowFunctionExpression(function) => check_params(&function.params.items, ctx),
            _ => {}
        }
    }
}

fn is_function_decl_or_expr(function: &Function) -> bool {
    function.is_declaration() || function.is_expression()
}

fn check_params<'a>(params: &'a [FormalParameter<'a>], ctx: &LintContext<'a>) {
    let mut seen_plain = false;
    for param in params.iter().rev() {
        let is_default = param.pattern.kind.is_assignment_pattern() || param.pattern.optional;

        if !is_default {
            seen_plain = true;
        } else if seen_plain {
            ctx.diagnostic(default_param_last_diagnostic(param.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function f() {}",
        "function f(a) {}",
        "function f(a = 5) {}",
        "function f(a, b) {}",
        "function f(a, b = 5) {}",
        "function f(a, b = 5, c = 5) {}",
        "function f(a, b = 5, ...c) {}",
        "const f = () => {}",
        "const f = (a) => {}",
        "const f = (a = 5) => {}",
        "const f = function f() {}",
        "const f = function f(a) {}",
        "const f = function f(a = 5) {}",
        "function fn(a: string = 'a', b?: string) {}",
        "function foo() {}",
        "function foo(a: number) {}",
        "function foo(a = 1) {}",
        "function foo(a?: number) {}",
        "function foo(a: number, b: number) {}",
        "function foo(a: number, b: number, c?: number) {}",
        "function foo(a: number, b = 1) {}",
        "function foo(a: number, b = 1, c = 1) {}",
        "function foo(a: number, b = 1, c?: number) {}",
        "function foo(a: number, b?: number, c = 1) {}",
        "function foo(a: number, b = 1, ...c) {}",
        "const foo = function () {};",
        "const foo = function (a: number) {};",
        "const foo = function (a = 1) {};",
        "const foo = function (a?: number) {};",
        "const foo = function (a: number, b: number) {};",
        "const foo = function (a: number, b: number, c?: number) {};",
        "const foo = function (a: number, b = 1) {};",
        "const foo = function (a: number, b = 1, c = 1) {};",
        "const foo = function (a: number, b = 1, c?: number) {};",
        "const foo = function (a: number, b?: number, c = 1) {};",
        "const foo = function (a: number, b = 1, ...c) {};",
        "const foo = () => {};",
        "const foo = (a: number) => {};",
        "const foo = (a = 1) => {};",
        "const foo = (a?: number) => {};",
        "const foo = (a: number, b: number) => {};",
        "const foo = (a: number, b: number, c?: number) => {};",
        "const foo = (a: number, b = 1) => {};",
        "const foo = (a: number, b = 1, c = 1) => {};",
        "const foo = (a: number, b = 1, c?: number) => {};",
        "const foo = (a: number, b?: number, c = 1) => {};",
        "const foo = (a: number, b = 1, ...c) => {};",
        "
        class Foo {
            constructor(a: number, b: number, c: number) {}
        }",
        "
        class Foo {
            constructor(a: number, b?: number, c = 1) {}
        }",
        "
        class Foo {
            constructor(a: number, b = 1, c?: number) {}
        }",
        "
        class Foo {
            constructor(
                public a: number,
                protected b: number,
                private c: number,
            ) {}
        }",
        "
        class Foo {
            constructor(
                public a: number,
                protected b?: number,
                private c = 10,
            ) {}
        }",
        "
        class Foo {
            constructor(
                public a: number,
                protected b = 10,
                private c?: number,
            ) {}
        }",
        "
        class Foo {
            constructor(
                a: number,
                protected b?: number,
                private c = 0,
            ) {}
        }",
        "
        class Foo {
            constructor(
                a: number,
                b?: number,
                private c = 0,
            ) {}
        }",
        "
        class Foo {
            constructor(
                a: number,
                private b?: number,
                c = 0,
            ) {}
        }",
    ];

    let fail = vec![
        "function f(a = 5, b) {}",
        "function f(a = 5, b = 6, c) {}",
        "function f (a = 5, b, c = 6, d) {}",
        "function f(a = 5, b, c = 5) {}",
        "const f = (a = 5, b, ...c) => {}",
        "const f = function f (a, b = 5, c) {}",
        "const f = (a = 5, { b }) => {}",
        "const f = ({ a } = {}, b) => {}",
        "const f = ({ a, b } = { a: 1, b: 2 }, c) => {}",
        "const f = ([a] = [], b) => {}",
        "const f = ([a, b] = [1, 2], c) => {}",
        "function foo(a = 1, b: number) {}",
        "function foo(a = 1, b = 2, c: number) {}",
        "function foo(a = 1, b: number, c = 2, d: number) {}",
        "function foo(a = 1, b: number, c = 2) {}",
        "function foo(a = 1, b: number, ...c) {}",
        "function foo(a?: number, b: number) {}",
        "function foo(a: number, b?: number, c: number) {}",
        "function foo(a = 1, b?: number, c: number) {}",
        "function foo(a = 1, { b }) {}",
        "function foo({ a } = {}, b) {}",
        "function foo({ a, b } = { a: 1, b: 2 }, c) {}",
        "const foo = function (a = 1, b: number) {};",
        "const foo = function (a = 1, b = 2, c: number) {};",
        "const foo = function (a = 1, b: number, c = 2, d: number) {};",
        "const foo = function (a = 1, b: number, c = 2) {};",
        "const foo = function (a = 1, b: number, ...c) {};",
        "const foo = function (a?: number, b: number) {};",
        "const foo = function (a: number, b?: number, c: number) {};",
        "const foo = function (a = 1, b?: number, c: number) {};",
        "const foo = function (a = 1, { b }) {};",
        "const foo = function ({ a } = {}, b) {};",
        "const foo = function ({ a, b } = { a: 1, b: 2 }, c) {};",
        "const foo = (a = 1, b: number) => {};",
        "const foo = (a = 1, b = 2, c: number) => {};",
        "const foo = (a = 1, b: number, c = 2, d: number) => {};",
        "const foo = (a = 1, b: number, c = 2) => {};",
        "const foo = (a = 1, b: number, ...c) => {};",
        "const foo = (a?: number, b: number) => {};",
        "const foo = (a: number, b?: number, c: number) => {};",
        "const foo = (a = 1, b?: number, c: number) => {};",
        "
        class Foo {
            constructor(
                public a?: number,
                private b: number,
            ) {}
        }",
        "
        class Foo {
            constructor(a = 0, b: number) {}
        }",
        "class Foo {
            constructor(a?: number, b: number) {}
        }",
        "class Foo {
            constructor(
                public a = 0,
                private b: number,
            ) {}
        }",
    ];

    Tester::new(DefaultParamLast::NAME, DefaultParamLast::PLUGIN, pass, fail).test_and_snapshot();
}
