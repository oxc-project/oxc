use oxc_ast::{ast::FormalParameter, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn default_param_last_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Default parameters should be last")
        .with_help("Enforce default parameters to be last.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DefaultParamLast;

declare_oxc_lint!(
    /// ### What it does
    /// Enforce default parameters to be last
    ///
    /// ### Why is this bad?
    /// Putting default parameter at last allows function calls to omit optional tail arguments.
    ///
    /// ### Example
    /// ```javascript
    /// // Correct: optional argument can be omitted
    /// function createUser(id, isAdmin = false) {}
    /// createUser("tabby")
    ///
    /// // Incorrect: optional argument can **not** be omitted
    /// function createUser(isAdmin = false, id) {}
    /// createUser(undefined, "tabby")
    /// ```
    DefaultParamLast,
    eslint,
    style
);

impl Rule for DefaultParamLast {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(function) => {
                if !function.is_declaration() && !function.is_expression() {
                    return;
                }
                check_params(&function.params.items, ctx);
            }
            AstKind::ArrowFunctionExpression(function) => check_params(&function.params.items, ctx),
            _ => {}
        }
    }
}

fn check_params<'a>(items: &'a [FormalParameter<'a>], ctx: &LintContext<'a>) {
    let mut has_seen_plain_param = false;
    for param in items.iter().rev() {
        if !param.pattern.kind.is_assignment_pattern() && !param.pattern.optional {
            has_seen_plain_param = true;
            continue;
        }
        if has_seen_plain_param
            && (param.pattern.kind.is_assignment_pattern() || param.pattern.optional)
        {
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
