use oxc_ast::{
    AstKind,
    ast::{BindingIdentifier, BindingPattern, Expression, VariableDeclaration, VariableDeclarator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ScopeId, Scoping};
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule, utils::has_ambient_typescript_ancestor};

fn no_var_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected var, use let or const instead.")
        .with_help("Replace var with let or const")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoVar;

// doc: https://github.com/eslint/eslint/blob/v9.9.1/docs/src/rules/no-var.md
// code: https://github.com/eslint/eslint/blob/v9.9.1/lib/rules/no-var.js
// test: https://github.com/eslint/eslint/blob/v9.9.1/tests/lib/rules/no-var.js

declare_oxc_lint!(
    /// ### What it does
    ///
    /// ECMAScript 2015 allows programmers to create variables with block scope
    /// instead of function scope using the `let` and `const` keywords.  Block
    /// scope is common in many other programming languages and helps
    /// programmers avoid mistakes.
    ///
    /// ### Why is this bad?
    ///
    /// Using `var` in an ES2015 environment triggers this error
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var x = "y";
    /// var CONFIG = {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let x = "y";
    /// const CONFIG = {};
    /// ```
    NoVar,
    eslint,
    restriction,
    conditional_fix,
    version = "0.1.1",
    short_description = "Enforce using `let` or `const` over `var`.",
);

impl Rule for NoVar {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::VariableDeclaration(dec) = node.kind()
            && dec.kind.is_var()
        {
            // Skip TypeScript ambient declarations (declare global/module/namespace)
            if has_ambient_typescript_ancestor(node.id(), ctx.nodes()) {
                return;
            }

            let is_written_to = dec.declarations.iter().any(|v| is_written_to(&v.id, ctx));
            let var_offset =
                ctx.find_next_token_within(dec.span.start, dec.span.end, "var").unwrap();
            let var_start = dec.span.start + var_offset;
            let var_keyword_span = Span::sized(var_start, 3);
            ctx.diagnostic_with_fix(no_var_diagnostic(var_keyword_span), |fixer| {
                if !can_fix(node, dec, ctx) {
                    return fixer.noop();
                }

                // Given the following code:
                // ```js
                // var a = undefined;
                // var b = null;
                // ```
                // We could just replace the `var` keyword with `const` (since neither are reassigned)
                // However, when users are also using rules such as unicorn/no-null or unicorn/no-useless-undefined,
                // those rules may also provide fixes that remove the initializers, resulting in:
                // ```js
                // const a;
                // const b;
                // ```
                // which is invalid syntax.
                // To avoid such conflicts, we replace the entire declaration span,
                // so that other rules do not attempt to provide fixes within the same span.
                let new_keyword = if dec.declare
                    || is_written_to
                    || !dec.declarations.iter().all(|v| v.init.is_some())
                {
                    "let"
                } else {
                    "const"
                };

                // Replace the entire declaration span to prevent fix conflicts with other rules
                let source = fixer.source_range(dec.span);
                // var_offset is relative to dec.span.start, so we need to skip past "var" (3 chars)
                let after_var = &source[(var_offset as usize + 3)..];
                let before_var = &source[..(var_offset as usize)];
                let fixed_source = format!("{before_var}{new_keyword}{after_var}");
                fixer.replace(dec.span, fixed_source)
            });
        }
    }
}

/// Whether replacing `var` with `let` or `const` keeps the program valid and its
/// behavior unchanged. Ported from `canFix` in ESLint's `no-var`.
fn can_fix<'a>(node: &AstNode<'a>, dec: &VariableDeclaration<'a>, ctx: &LintContext<'a>) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    let is_loop_assignee = match parent.kind() {
        AstKind::ForInStatement(stmt) => stmt.left.span() == dec.span,
        AstKind::ForOfStatement(stmt) => stmt.left.span() == dec.span,
        _ => false,
    };
    let is_for_init = matches!(
        parent.kind(),
        AstKind::ForStatement(stmt) if stmt.init.as_ref().is_some_and(|init| init.span() == dec.span)
    );

    match parent.kind() {
        // `let` in a `case` clause is reported by `no-case-declarations`.
        AstKind::SwitchCase(_) => return false,
        AstKind::Program(_)
        | AstKind::BlockStatement(_)
        | AstKind::FunctionBody(_)
        | AstKind::StaticBlock(_)
        | AstKind::TSModuleBlock(_) => {}
        // A lexical declaration is a syntax error in statement position,
        // e.g. `if (foo) let bar = 1;`.
        _ if !is_loop_assignee && !is_for_init => return false,
        _ => {}
    }

    let in_loop = is_in_loop(node, ctx);
    // A `var` keeps its value from the previous iteration, but a `let` without an
    // initializer starts each iteration as `undefined`.
    if in_loop && !is_loop_assignee && !dec.declarations.iter().all(|decl| decl.init.is_some()) {
        return false;
    }

    dec.declarations.iter().all(|decl| {
        decl.id.all_binding_identifiers(&mut |ident| {
            can_fix_binding(ident, decl, node, parent.span(), in_loop, ctx)
        })
    })
}

fn can_fix_binding<'a>(
    ident: &BindingIdentifier<'a>,
    decl: &VariableDeclarator<'a>,
    node: &AstNode<'a>,
    scope_span: Span,
    in_loop: bool,
    ctx: &LintContext<'a>,
) -> bool {
    let scoping = ctx.scoping();
    let symbol_id = ident.symbol_id();

    if ident.name == "let" || !scoping.symbol_redeclarations(symbol_id).is_empty() {
        return false;
    }

    let source_type = ctx.source_type();
    if scoping.symbol_scope_id(symbol_id) == scoping.root_scope_id()
        && !source_type.is_module()
        && !source_type.is_commonjs()
    {
        return false;
    }

    // `try {} catch (e) { var e = 1; }` assigns to the catch parameter. With `let`,
    // it would be a syntax error, or a new binding in a nested block.
    if scoping.find_binding(node.scope_id(), ident.name) != Some(symbol_id) {
        return false;
    }

    let var_scope = scoping.symbol_scope_id(symbol_id);
    let var_function_scope = enclosing_function_scope(scoping, var_scope);
    let init = decl.init.as_ref().map(Expression::without_parentheses);
    let loop_right = match ctx.nodes().parent_kind(node.id()) {
        AstKind::ForInStatement(stmt) => Some(stmt.right.span()),
        AstKind::ForOfStatement(stmt) => Some(stmt.right.span()),
        _ => None,
    };

    ctx.symbol_references(symbol_id).all(|reference| {
        let reference_node = ctx.nodes().get_node(reference.node_id());
        let span = reference_node.span();

        if !scope_span.contains_inclusive(span)
            || span.start < decl.span.start
            || loop_right.is_some_and(|right| right.contains_inclusive(span))
        {
            return false;
        }

        // `var a = a`, `var {a = a} = {}` and `var {a = b, b} = {}` read the
        // binding before it is initialized.
        if decl.id.span().contains_inclusive(span)
            && ctx.nodes().ancestors(reference.node_id()).any(|ancestor| {
                matches!(ancestor.kind(), AstKind::AssignmentPattern(pattern)
                    if pattern.left.span().contains_inclusive(ident.span)
                        && pattern.right.span().contains_inclusive(span))
            })
        {
            return false;
        }

        if let Some(init) = init
            && (span.start < ident.span.start
                || (!init.is_function() && init.span().contains_inclusive(span)))
        {
            return false;
        }

        // All loop iterations share one `var`, but each iteration gets its own `let`,
        // so a closure would see a different binding.
        if in_loop
            && (enclosing_function_scope(scoping, reference_node.scope_id()) != var_function_scope
                || ctx
                    .nodes()
                    .ancestors(reference.node_id())
                    .take_while(|ancestor| !ancestor.span().contains_inclusive(node.span()))
                    .any(|ancestor| {
                        let value = match ancestor.kind() {
                            AstKind::PropertyDefinition(property) if !property.r#static => {
                                &property.value
                            }
                            AstKind::AccessorProperty(property) if !property.r#static => {
                                &property.value
                            }
                            _ => return false,
                        };
                        value.as_ref().is_some_and(|value| value.span().contains_inclusive(span))
                    }))
        {
            return false;
        }

        !is_in_function_referenced_before(
            reference_node.scope_id(),
            var_scope,
            decl.span.start,
            ctx,
        )
    })
}

/// Whether `node` is inside a loop of its own function.
fn is_in_loop(node: &AstNode, ctx: &LintContext) -> bool {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::DoWhileStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
            | AstKind::ForStatement(_)
            | AstKind::WhileStatement(_) => return true,
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => return false,
            _ => {}
        }
    }
    false
}

fn enclosing_function_scope(scoping: &Scoping, scope_id: ScopeId) -> ScopeId {
    scoping
        .scope_ancestors(scope_id)
        .find(|&scope_id| {
            let flags = scoping.scope_flags(scope_id);
            flags.is_function() || flags.is_top()
        })
        .unwrap_or(scope_id)
}

/// Whether a reference in `reference_scope` is inside a function declaration that
/// is referenced before `declaration_start` and could be invoked before initialization.
fn is_in_function_referenced_before(
    reference_scope: ScopeId,
    var_scope: ScopeId,
    declaration_start: u32,
    ctx: &LintContext,
) -> bool {
    let scoping = ctx.scoping();
    for scope_id in scoping.scope_ancestors(reference_scope) {
        if scope_id == var_scope {
            break;
        }
        let AstKind::Function(func) = ctx.nodes().kind(scoping.get_node_id(scope_id)) else {
            continue;
        };
        let Some(id) = func.id.as_ref().filter(|_| func.is_declaration()) else {
            continue;
        };
        if ctx.symbol_references(id.symbol_id()).any(|reference| {
            ctx.nodes().get_node(reference.node_id()).span().start < declaration_start
        }) {
            return true;
        }
    }
    false
}

fn is_written_to(binding_pat: &BindingPattern, ctx: &LintContext) -> bool {
    match &binding_pat {
        BindingPattern::BindingIdentifier(binding_ident) => ctx
            .semantic()
            .symbol_references(binding_ident.symbol_id())
            .any(oxc_semantic::Reference::is_write),
        BindingPattern::ObjectPattern(object_pat) => {
            if object_pat.properties.iter().any(|prop| is_written_to(&prop.value, ctx)) {
                return true;
            }

            if let Some(rest) = &object_pat.rest {
                is_written_to(&rest.argument, ctx)
            } else {
                false
            }
        }
        BindingPattern::AssignmentPattern(_) => true,
        BindingPattern::ArrayPattern(array_pat) => {
            array_pat.elements.iter().flatten().any(|elem| is_written_to(elem, ctx))
                || array_pat.rest.as_ref().is_some_and(|rest| is_written_to(&rest.argument, ctx))
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("let moo = 'car';", None),
        ("const JOE = 'schmoe';", None),
        ("declare module 'testModule' { var x: string; }", None),
        ("declare namespace MyNamespace { var y: number; }", None),
        ("declare global { var __TEST_DECLARE_GLOBAL__: boolean | undefined; }", None),
    ];

    let fail = vec![
        ("var foo = bar;", None),
        ("var foo = bar, toast = most;", None),
        ("var foo = bar; let toast = most;", None),
        ("for (var a of b) { console.log(a); }", None),
        ("for (var a in b) { console.log(a); }", None),
        ("for (let a of b) { var c = 1; console.log(c); }", None),
        ("for (var i = 0; i < list.length; ++i) { foo(i) }", None),
        ("for (var i = 0, i = 0; false;);", None),
        ("var i = 0; for (var i = 1; false;); console.log(i);", None),
        ("var a, b, c; var a;", None),
        ("var a; if (b) { var a; }", None),
        ("if (foo) { var a, b, c; } a;", None),
        ("for (var i = 0; i < 10; ++i) {} i;", None),
        ("for (var a in obj) {} a;", None),
        ("for (var a of list) {} a;", None),
        ("switch (a) { case 0: var b = 1 }", None),
        ("for (var a of b) { arr.push(() => a); }", None),
        ("for (let a of b) { var c; console.log(c); c = 'hello'; }", None),
        ("var a = a", None),
        ("var {a = a} = {}", None),
        ("var {a = b, b} = {}", None),
        ("var {a, b = a} = {}", None),
        ("var a = b, b = 1", None),
        ("var a = b; var b = 1", None),
        ("function foo() { a } var a = 1; foo()", None),
        ("if (foo) var bar = 1;", None),
        ("var foo = 1", None),
        ("{ var foo = 1 }", None),
        ("if (true) { var foo = 1 }", None),
        ("var foo = 1", None),
        ("declare var foo: 2;", None),
        ("function foo() { var let; }", None),
        ("function foo() { var { let } = {}; }", None),
        (
            "var fx = function (i = 0) { if (i < 5) { return fx(i + 1); } console.log(i); }; fx();",
            None,
        ),
        ("var foo = function () { foo() };", None),
        ("var foo = () => foo();", None),
        ("var foo = (function () { foo(); })();", None),
        ("var foo = bar(function () { foo(); });", None),
        ("var bar = foo, foo = function () { foo(); };", None),
        ("var bar = foo; var foo = function () { foo(); };", None),
        ("var { foo = foo } = function () { foo(); };", None),
        ("var { bar = foo, foo } = function () { foo(); };", None),
        ("var bar = function () { foo(); }; var foo = function() {};", None),
    ];

    let fix = vec![
        ("var foo", "let foo"),
        ("var foo; foo += 1", "let foo; foo += 1"),
        ("var foo,bar; bar = 'que'", "let foo,bar; bar = 'que'"),
        ("var { a } = {}; a = fn()", "let { a } = {}; a = fn()"),
        ("var { a } = {}; let b = a", "const { a } = {}; let b = a"),
        ("var foo = 1", "const foo = 1"),
        ("var foo = 1, bar = 2", "const foo = 1, bar = 2"),
        ("var foo = 1, bar", "let foo = 1, bar"),
        // TODO: implement a correct fixer for this case.
        // we need to add a `let a;` to the parent of both scopes
        // then change `var a = undefined` into `a = undefined`
        (
            "function play(index: number) { if (index > 1) { var a = undefined } else { var a = undefined } console.log(a) }",
            "function play(index: number) { if (index > 1) { var a = undefined } else { var a = undefined } console.log(a) }",
        ),
        ("declare var foo: 2;", "declare let foo: 2;"),
        ("var x = 1; var x = 2; console.log(x);", "var x = 1; var x = 2; console.log(x);"),
        ("var x; var x;", "var x; var x;"),
        ("var x = 1, x = 2;", "var x = 1, x = 2;"),
        (
            "function f() { var x = 1; var x = 2; return x; }",
            "function f() { var x = 1; var x = 2; return x; }",
        ),
        ("var x = 1; var { x, y } = obj;", "var x = 1; var { x, y } = obj;"),
        ("var [x, ...rest] = arr; var rest = [];", "var [x, ...rest] = arr; var rest = [];"),
        ("function f(x) { var x = 1; }", "function f(x) { var x = 1; }"),
        (
            "function f() { var x = 1; function x() {} }",
            "function f() { var x = 1; function x() {} }",
        ),
        (
            "var x = 1; function f() { var x = 2; return x; }",
            "const x = 1; function f() { const x = 2; return x; }",
        ),
        ("for (var a of b) { console.log(a); }", "for (let a of b) { console.log(a); }"),
        ("for (var a in b) { console.log(a); }", "for (let a in b) { console.log(a); }"),
        (
            "for (let a of b) { var c = 1; console.log(c); }",
            "for (let a of b) { const c = 1; console.log(c); }",
        ),
        (
            "for (var i = 0; i < list.length; ++i) { foo(i) }",
            "for (let i = 0; i < list.length; ++i) { foo(i) }",
        ),
        ("for (var i = 0, i = 0; false;);", "for (var i = 0, i = 0; false;);"),
        (
            "var i = 0; for (var i = 1; false;); console.log(i);",
            "var i = 0; for (var i = 1; false;); console.log(i);",
        ),
        ("var a, b, c; var a;", "var a, b, c; var a;"),
        ("var a; if (b) { var a; }", "var a; if (b) { var a; }"),
        ("if (foo) { var a, b, c; } a;", "if (foo) { var a, b, c; } a;"),
        ("for (var i = 0; i < 10; ++i) {} i;", "for (var i = 0; i < 10; ++i) {} i;"),
        ("for (var a in obj) {} a;", "for (var a in obj) {} a;"),
        ("for (var a of list) {} a;", "for (var a of list) {} a;"),
        ("switch (a) { case 0: var b = 1 }", "switch (a) { case 0: var b = 1 }"),
        ("for (var a of b) { arr.push(() => a); }", "for (var a of b) { arr.push(() => a); }"),
        (
            "for (let a of b) { var c; console.log(c); c = 'hello'; }",
            "for (let a of b) { var c; console.log(c); c = 'hello'; }",
        ),
        ("var a = a", "var a = a"),
        ("var {a = a} = {}", "var {a = a} = {}"),
        ("var {a = b, b} = {}", "var {a = b, b} = {}"),
        ("var {a, b = a} = {}", "let {a, b = a} = {}"),
        ("var a = b, b = 1", "var a = b, b = 1"),
        ("var a = b; var b = 1", "const a = b; var b = 1"),
        ("function foo() { a } var a = 1; foo()", "function foo() { a } var a = 1; foo()"),
        ("if (foo) var bar = 1;", "if (foo) var bar = 1;"),
        (
            "var fx = function (i = 0) { if (i < 5) { return fx(i + 1); } console.log(i); }; fx();",
            "const fx = function (i = 0) { if (i < 5) { return fx(i + 1); } console.log(i); }; fx();",
        ),
        ("var foo = function () { foo() };", "const foo = function () { foo() };"),
        ("var foo = () => foo();", "const foo = () => foo();"),
        ("var foo = (function () { foo(); })();", "var foo = (function () { foo(); })();"),
        ("var foo = bar(function () { foo(); });", "var foo = bar(function () { foo(); });"),
        (
            "var bar = foo, foo = function () { foo(); };",
            "var bar = foo, foo = function () { foo(); };",
        ),
        (
            "var bar = foo; var foo = function () { foo(); };",
            "const bar = foo; var foo = function () { foo(); };",
        ),
        (
            "var { foo = foo } = function () { foo(); };",
            "var { foo = foo } = function () { foo(); };",
        ),
        (
            "var { bar = foo, foo } = function () { foo(); };",
            "var { bar = foo, foo } = function () { foo(); };",
        ),
        (
            "var bar = function () { foo(); }; var foo = function() {};",
            "const bar = function () { foo(); }; var foo = function() {};",
        ),
        (
            "export function a() { console.log(o); var o; return o; }",
            "export function a() { console.log(o); var o; return o; }",
        ),
        (
            "function test() { console.log(x); var x = 1; }",
            "function test() { console.log(x); var x = 1; }",
        ),
        (
            "function test() { if (foo) { console.log(x); } var x = 1; }",
            "function test() { if (foo) { console.log(x); } var x = 1; }",
        ),
        (
            "function a() { if (something) { console.log(o); } var o; return o; }",
            "function a() { if (something) { console.log(o); } var o; return o; }",
        ),
        (
            "function b() { if (something) { console.log(o); var o; return o; } }",
            "function b() { if (something) { console.log(o); var o; return o; } }",
        ),
        (
            "function test() { var y = x; var x = 1; }",
            "function test() { const y = x; var x = 1; }",
        ),
        (
            "var a = 1; function test() { console.log(a); var a = 2; }",
            "const a = 1; function test() { console.log(a); var a = 2; }",
        ),
        (
            "function f() { try {} catch (e) { var e = 1; } }",
            "function f() { try {} catch (e) { var e = 1; } }",
        ),
        (
            "function f() { try {} catch (e) { var e; } }",
            "function f() { try {} catch (e) { var e; } }",
        ),
        (
            "function f() { try {} catch (e) { { var e = 1; } } }",
            "function f() { try {} catch (e) { { var e = 1; } } }",
        ),
        (
            "function f() { try {} catch (e) { try {} catch (x) { var e = 1; } } }",
            "function f() { try {} catch (e) { try {} catch (x) { var e = 1; } } }",
        ),
        (
            "function f() { try {} catch (e) { for (var e of []) {} } }",
            "function f() { try {} catch (e) { for (var e of []) {} } }",
        ),
        (
            "function f() { try {} catch (e) { var x = 1; } }",
            "function f() { try {} catch (e) { const x = 1; } }",
        ),
        (
            "function f() { try {} catch (e) {} var e = 1; }",
            "function f() { try {} catch (e) {} const e = 1; }",
        ),
        (
            "function f() { try {} catch (e) { function g() { var e = 1; } } }",
            "function f() { try {} catch (e) { function g() { const e = 1; } } }",
        ),
        (
            "function f() { try {} catch (e) { class C { static { var e = 1; } } } }",
            "function f() { try {} catch (e) { class C { static { const e = 1; } } } }",
        ),
        (
            "function wrap() { foo(); var a = 1; function foo() { console.log(a); } }",
            "function wrap() { foo(); var a = 1; function foo() { console.log(a); } }",
        ),
        (
            "function wrap() { var a = 1; foo(); function foo() { console.log(a); } }",
            "function wrap() { const a = 1; foo(); function foo() { console.log(a); } }",
        ),
        (
            "function wrap() { bar(); var a = 1; function bar() { function inner() { console.log(a); } inner(); } }",
            "function wrap() { bar(); var a = 1; function bar() { function inner() { console.log(a); } inner(); } }",
        ),
        (
            "function wrap() { var b = foo(), a = 1; function foo() { console.log(a); } }",
            "function wrap() { var b = foo(), a = 1; function foo() { console.log(a); } }",
        ),
        (
            "function wrap() { foo(); var a = 1; var b = 2; function foo() { console.log(a); } }",
            "function wrap() { foo(); var a = 1; const b = 2; function foo() { console.log(a); } }",
        ),
        (
            "function wrap() { function foo() {} foo(); if (bar) { var a = 1; function foo() { console.log(a); } foo (); } }",
            "function wrap() { function foo() {} foo(); if (bar) { const a = 1; function foo() { console.log(a); } foo (); } }",
        ),
        // ESLint does not fix exports either: during a circular import, reading an
        // exported `let` or `const` before it is initialized throws.
        ("export var foo = 1;", "export var foo = 1;"),
        ("var {a: {b} = {b}} = {};", "var {a: {b} = {b}} = {};"),
        ("var [[a] = [a]] = [];", "var [[a] = [a]] = [];"),
        ("for (var {a = a} of [{}]) {}", "for (var {a = a} of [{}]) {}"),
        ("for (var a of [a]) {}", "for (var a of [a]) {}"),
        (
            "function wrap() { new foo(); var a = 1; function foo() { console.log(a); } }",
            "function wrap() { new foo(); var a = 1; function foo() { console.log(a); } }",
        ),
        (
            "function wrap() { foo.call(null); var a = 1; function foo() { console.log(a); } }",
            "function wrap() { foo.call(null); var a = 1; function foo() { console.log(a); } }",
        ),
        (
            "function wrap() { foo.apply(null, []); var a = 1; function foo() { console.log(a); } }",
            "function wrap() { foo.apply(null, []); var a = 1; function foo() { console.log(a); } }",
        ),
        (
            "function wrap() { foo``; var a = 1; function foo() { console.log(a); } }",
            "function wrap() { foo``; var a = 1; function foo() { console.log(a); } }",
        ),
        (
            "function wrap() { consume(foo); var a = 1; function foo() { console.log(a); } }",
            "function wrap() { consume(foo); var a = 1; function foo() { console.log(a); } }",
        ),
        ("for (var a in { [a]: 1 }) {}", "for (var a in { [a]: 1 }) {}"),
        (
            "for (var i of [1, 2]) { classes.push(class { x = i; }); }",
            "for (var i of [1, 2]) { classes.push(class { x = i; }); }",
        ),
        (
            "for (var i = 0; i < 2; i++) { classes.push(class { x = i; }); }",
            "for (var i = 0; i < 2; i++) { classes.push(class { x = i; }); }",
        ),
        (
            "for (var i of [1, 2]) { classes.push(class { accessor x = i; }); }",
            "for (var i of [1, 2]) { classes.push(class { accessor x = i; }); }",
        ),
        (
            "for (var i of [1, 2]) { classes.push(class { static x = i; }); }",
            "for (let i of [1, 2]) { classes.push(class { static x = i; }); }",
        ),
        (
            "for (var i of [1, 2]) { classes.push(class { [i] = 0; }); }",
            "for (let i of [1, 2]) { classes.push(class { [i] = 0; }); }",
        ),
        (
            "function f() { var [...xs] = []; xs = [1]; return xs; }",
            "function f() { let [...xs] = []; xs = [1]; return xs; }",
        ),
        ("var [a, ...rest] = arr; rest = [];", "let [a, ...rest] = arr; rest = [];"),
    ];

    let script_fix = vec![
        ("function foo() { var let; }", "function foo() { var let; }"),
        ("function foo() { var { let } = {}; }", "function foo() { var { let } = {}; }"),
        (
            "var auditX = 1; console.log(globalThis.auditX);",
            "var auditX = 1; console.log(globalThis.auditX);",
        ),
        ("{ var foo = 1 }", "{ var foo = 1 }"),
        ("declare var foo: 2;", "declare var foo: 2;"),
        ("function f() { var x = 1; return x; }", "function f() { const x = 1; return x; }"),
    ];

    Tester::new(NoVar::NAME, NoVar::PLUGIN, pass, fail).expect_fix(script_fix).test_and_snapshot();
    Tester::new(NoVar::NAME, NoVar::PLUGIN, Vec::<&str>::new(), vec![])
        .change_rule_path_extension("mts")
        .expect_fix(fix)
        .test();
}
