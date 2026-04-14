use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_setter_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Setter cannot return a value")
        .with_help("Remove the return statement or ensure it does not return a value.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSetterReturn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Setters cannot return values.
    ///
    /// This rule can be disabled for TypeScript code, as the TypeScript compiler
    /// enforces this check.
    ///
    /// ### Why is this bad?
    ///
    /// While returning a value from a setter does not produce an error, the returned value is
    /// being ignored. Therefore, returning a value from a setter is either unnecessary or a
    /// possible error, since the returned value cannot be used.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class URL {
    ///   set origin() {
    ///     return true;
    ///   }
    /// }
    /// ```
    NoSetterReturn,
    eslint,
    correctness,
    version = "0.0.3",
);

impl Rule for NoSetterReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ReturnStatement(stmt) = node.kind() else {
            return;
        };
        if stmt.argument.is_none() {
            return;
        }

        for scope_id in ctx.scoping().scope_ancestors(node.scope_id()) {
            let flags = ctx.scoping().scope_flags(scope_id);
            if flags.is_set_accessor() {
                ctx.diagnostic(no_setter_return_diagnostic(stmt.span));
            } else if flags.is_function() {
                break;
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function foo() { return 1; }",
        "function set(val) { return 1; }",
        "var foo = function() { return 1; };",
        "var foo = function set() { return 1; };",
        "var set = function() { return 1; };",
        "var set = function set(val) { return 1; };",
        "var set = val => { return 1; };",
        "var set = val => 1;",
        "({ set a(val) { }}); function foo() { return 1; }",
        "({ set a(val) { }}); (function () { return 1; });",
        "({ set a(val) { }}); (() => { return 1; });",
        "({ set a(val) { }}); (() => 1);",
        "return 1;",
        "return 1;",
        "return 1; function foo(){ return 1; } return 1;",
        "function foo(){} return 1; var bar = function*(){ return 1; }; return 1; var baz = () => {}; return 1;",
        "({ set foo(val) { return; } })",
        "({ set foo(val) { if (val) { return; } } })",
        "class A { set foo(val) { return; } }",
        "(class { set foo(val) { if (val) { return; } else { return; } return; } })",
        "class A { set foo(val) { try {} catch(e) { return; } } }",
        "({ get foo() { return 1; } })",
        "({ get set() { return 1; } })",
        "({ set(val) { return 1; } })",
        "({ set: function(val) { return 1; } })",
        "({ foo: function set(val) { return 1; } })",
        "({ set: function set(val) { return 1; } })",
        "({ set: (val) => { return 1; } })",
        "({ set: (val) => 1 })",
        "set = { foo(val) { return 1; } };",
        "class A { constructor(val) { return 1; } }",
        "class set { constructor(val) { return 1; } }",
        "class set { foo(val) { return 1; } }",
        "var set = class { foo(val) { return 1; } }",
        "(class set { foo(val) { return 1; } })",
        "class A { get foo() { return val; } }",
        "class A { get set() { return val; } }",
        "class A { set(val) { return 1; } }",
        "class A { static set(val) { return 1; } }",
        "({ set: set = function set(val) { return 1; } } = {})",
        "({ set: set = (val) => 1 } = {})",
        "class C { set; foo() { return 1; } }",
        "({ set foo(val) { function foo(val) { return 1; } } })",
        "({ set foo(val) { var foo = function(val) { return 1; } } })",
        "({ set foo(val) { var foo = (val) => { return 1; } } })",
        "({ set foo(val) { var foo = (val) => 1; } })",
        "({ set [function() { return 1; }](val) {} })",
        "({ set [() => { return 1; }](val) {} })",
        "({ set [() => 1](val) {} })",
        "({ set foo(val = function() { return 1; }) {} })",
        "({ set foo(val = v => 1) {} })",
        "(class { set foo(val) { function foo(val) { return 1; } } })",
        "(class { set foo(val) { var foo = function(val) { return 1; } } })",
        "(class { set foo(val) { var foo = (val) => { return 1; } } })",
        "(class { set foo(val) { var foo = (val) => 1; } })",
        "(class { set [function() { return 1; }](val) {} })",
        "(class { set [() => { return 1; }](val) {} })",
        "(class { set [() => 1](val) {} })",
        "(class { set foo(val = function() { return 1; }) {} })",
        "(class { set foo(val = (v) => 1) {} })",
        "Object.defineProperty(foo, 'bar', { set(val) { return; } })",
        "Reflect.defineProperty(foo, 'bar', { set(val) { if (val) { return; } } })",
        "Object.defineProperties(foo, { bar: { set(val) { try { return; } catch(e){} } } })",
        "Object.create(foo, { bar: { set: function(val) { return; } } })",
        "x = { set(val) { return 1; } }",
        "x = { foo: { set(val) { return 1; } } }",
        "Object.defineProperty(foo, 'bar', { value(val) { return 1; } })",
        "Reflect.defineProperty(foo, 'bar', { value: function set(val) { return 1; } })",
        "Object.defineProperties(foo, { bar: { [set](val) { return 1; } } })",
        "Object.create(foo, { bar: { 'set ': function(val) { return 1; } } })",
        "Object.defineProperty(foo, 'bar', { [`set `]: (val) => { return 1; } })",
        "Reflect.defineProperty(foo, 'bar', { Set(val) { return 1; } })",
        "Object.defineProperties(foo, { bar: { value: (val) => 1 } })",
        "Object.create(foo, { set: { value: function(val) { return 1; } } })",
        "Object.defineProperty(foo, 'bar', { baz(val) { return 1; } })",
        "Reflect.defineProperty(foo, 'bar', { get(val) { return 1; } })",
        "Object.create(foo, { set: function(val) { return 1; } })",
        "Object.defineProperty(foo, { set: (val) => 1 })",
        "Object.defineProperty(foo, 'bar', { set(val) { function foo() { return 1; } } })",
        "Reflect.defineProperty(foo, 'bar', { set(val) { var foo = function() { return 1; } } })",
        "Object.defineProperties(foo, { bar: { set(val) { () => { return 1 }; } } })",
        "Object.create(foo, { bar: { set: (val) => { (val) => 1; } } })",
        "Object.defineProperty(foo, 'bar', 'baz', { set(val) { return 1; } })",
        "Object.defineProperty(foo, { set(val) { return 1; } }, 'bar')",
        "Object.defineProperty({ set(val) { return 1; } }, foo, 'bar')",
        "Reflect.defineProperty(foo, 'bar', 'baz', { set(val) { return 1; } })",
        "Reflect.defineProperty(foo, { set(val) { return 1; } }, 'bar')",
        "Reflect.defineProperty({ set(val) { return 1; } }, foo, 'bar')",
        "Object.defineProperties(foo, bar, { baz: { set(val) { return 1; } } })",
        "Object.defineProperties({ bar: { set(val) { return 1; } } }, foo)",
        "Object.create(foo, bar, { baz: { set(val) { return 1; } } })",
        "Object.create({ bar: { set(val) { return 1; } } }, foo)",
        "Object.DefineProperty(foo, 'bar', { set(val) { return 1; } })",
        "Reflect.DefineProperty(foo, 'bar', { set(val) { if (val) { return 1; } } })",
        "Object.DefineProperties(foo, { bar: { set(val) { try { return 1; } catch(e){} } } })",
        "Object.Create(foo, { bar: { set: function(val) { return 1; } } })",
        "object.defineProperty(foo, 'bar', { set(val) { return 1; } })",
        "reflect.defineProperty(foo, 'bar', { set(val) { if (val) { return 1; } } })",
        "Reflect.defineProperties(foo, { bar: { set(val) { try { return 1; } catch(e){} } } })",
        "object.create(foo, { bar: { set: function(val) { return 1; } } })",
        "Reflect.defineProperty(foo, 'bar', { set(val) { if (val) { return 1; } } })",
        "/* globals Object:off */ Object.defineProperty(foo, 'bar', { set(val) { return 1; } })",
        "Object.defineProperties(foo, { bar: { set(val) { try { return 1; } catch(e){} } } })",
        "let Object; Object.defineProperty(foo, 'bar', { set(val) { return 1; } })",
        "function f() { Reflect.defineProperty(foo, 'bar', { set(val) { if (val) { return 1; } } }); var Reflect;}",
        "function f(Object) { Object.defineProperties(foo, { bar: { set(val) { try { return 1; } catch(e){} } } }) }",
        "if (x) { const Object = getObject(); Object.create(foo, { bar: { set: function(val) { return 1; } } }) }",
        "x = function Object() { Object.defineProperty(foo, 'bar', { set(val) { return 1; } }) }",
    ];

    let fail = vec![
        "({ set a(val){ return val + 1; } })",
        "({ set a(val) { return 1; } })",
        "class A { set a(val) { return 1; } }",
        "class A { static set a(val) { return 1; } }",
        "(class { set a(val) { return 1; } })",
        "({ set a(val) { return val; } })",
        "class A { set a(val) { return undefined; } }",
        "(class { set a(val) { return null; } })",
        "({ set a(val) { return x + y; } })",
        "class A { set a(val) { return foo(); } }",
        "(class { set a(val) { return this._a; } })",
        "({ set a(val) { return this.a; } })",
        "({ set a(val) { if (foo) { return 1; }; } })",
        "class A { set a(val) { try { return 1; } catch(e) {} } }",
        "(class { set a(val) { while (foo){ if (bar) break; else return 1; } } })",
        "({ set a(val) { return 1; }, set b(val) { return 1; } })",
        "class A { set a(val) { return 1; } set b(val) { return 1; } }",
        "(class { set a(val) { return 1; } static set b(val) { return 1; } })",
        "({ set a(val) { if(val) { return 1; } else { return 2 }; } })",
        "class A { set a(val) { switch(val) { case 1: return x; case 2: return y; default: return z } } }",
        "(class { static set a(val) { if (val > 0) { this._val = val; return val; } return false; } })",
        "({ set a(val) { if(val) { return 1; } else { return; }; } })",
        "class A { set a(val) { switch(val) { case 1: return x; case 2: return; default: return z } } }",
        "(class { static set a(val) { if (val > 0) { this._val = val; return; } return false; } })",
        "({ set a(val) { function b(){} return b(); } })",
        "class A { set a(val) { return () => {}; } }",
        "(class { set a(val) { function b(){ return 1; } return 2; } })",
        "({ set a(val) { function b(){ return; } return 1; } })",
        "class A { set a(val) { var x = function() { return 1; }; return 2; } }",
        "(class { set a(val) { var x = () => { return; }; return 2; } })",
        "function f(){}; ({ set a(val) { return 1; } });",
        "x = function f(){}; class A { set a(val) { return 1; } };",
        "x = () => {}; A = class { set a(val) { return 1; } };",
        "return; ({ set a(val) { return 1; } }); return 2;",
        // "Object.defineProperty(foo, 'bar', { set(val) { return 1; } })",
        // "Reflect.defineProperty(foo, 'bar', { set(val) { return 1; } })",
        // "Object.defineProperties(foo, { baz: { set(val) { return 1; } } })",
        // "Object.create(null, { baz: { set(val) { return 1; } } })",
        // "Object.defineProperty(foo, 'bar', { set: val => val })",
        // "Reflect.defineProperty(foo, 'bar', { set: val => f(val) })",
        // "Object.defineProperties(foo, { baz: { set: val => a + b } })",
        // "Object.create({}, { baz: { set: val => this._val } })",
        // "Object.defineProperty(foo, 'bar', { set(val) { if (val) { return; } return false; }, get(val) { return 1; } })",
        // "Reflect.defineProperty(foo, 'bar', { set(val) { try { return f(val) } catch (e) { return e }; } })",
        // "Object.defineProperties(foo, { bar: { get(){ return null; }, set(val) { return null; } } })",
        // "Object.create(null, { baz: { set(val) { return this._val; return; return undefined; } } })",
        // "Object.defineProperties(foo, { baz: { set(val) { return 1; } }, bar: { set(val) { return 1; } } })",
        // "Object.create({}, { baz: { set(val) { return 1; } }, bar: { set: (val) => 1 } })",
        // "Object['defineProperty'](foo, 'bar', { set: function bar(val) { return 1; } })",
        // "Reflect.defineProperty(foo, 'bar', { 'set'(val) { return 1; } })",
        // "Object[`defineProperties`](foo, { baz: { ['set'](val) { return 1; } } })",
        // "Object.create({}, { baz: { [`set`]: (val) => { return 1; } } })",
        // "Object.defineProperty(foo, 'bar', { set: function Object(val) { return 1; } })",
        // "Object.defineProperty(foo, 'bar', { set: function(Object) { return 1; } })",
        // "Object?.defineProperty(foo, 'bar', { set(val) { return 1; } })",
        // "(Object?.defineProperty)(foo, 'bar', { set(val) { return 1; } })",
    ];

    Tester::new(NoSetterReturn::NAME, NoSetterReturn::PLUGIN, pass, fail)
        .change_rule_path_extension("js")
        .test_and_snapshot();
}
