use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-setter-return): Setter cannot return a value")]
struct NoSetterReturnDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoSetterReturn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Setters cannot return values.
    ///
    /// ### Why is this bad?
    ///
    /// While returning a value from a setter does not produce an error, the returned value is
    /// being ignored. Therefore, returning a value from a setter is either unnecessary or a
    /// possible error, since the returned value cannot be used.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// class URL {
    ///   set origin() {
    ///     return true;
    ///   }
    /// }
    /// ```
    NoSetterReturn,
    correctness
);

impl Rule for NoSetterReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ReturnStatement(stmt) = node.kind() else { return };
        if stmt.argument.is_some()
            && ctx.scopes().get_flags(node.scope_id()).is_set_accessor()
        {
            ctx.diagnostic(NoSetterReturnDiagnostic(stmt.span));
        }
    }
}

#[allow(clippy::too_many_lines)]
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function foo() { return 1; }", None),
        ("function set(val) { return 1; }", None),
        ("var foo = function() { return 1; };", None),
        ("var foo = function set() { return 1; };", None),
        ("var set = function() { return 1; };", None),
        ("var set = function set(val) { return 1; };", None),
        ("var set = val => { return 1; };", None),
        ("var set = val => 1;", None),
        ("({ set a(val) { }}); function foo() { return 1; }", None),
        ("({ set a(val) { }}); (function () { return 1; });", None),
        ("({ set a(val) { }}); (() => { return 1; });", None),
        ("({ set a(val) { }}); (() => 1);", None),
        ("return 1;", None),
        ("return 1;", None),
        ("return 1; function foo(){ return 1; } return 1;", None),
        (
            "function foo(){} return 1; var bar = function*(){ return 1; }; return 1; var baz = () => {}; return 1;",
            None,
        ),
        ("({ set foo(val) { return; } })", None),
        ("({ set foo(val) { if (val) { return; } } })", None),
        ("class A { set foo(val) { return; } }", None),
        ("(class { set foo(val) { if (val) { return; } else { return; } return; } })", None),
        ("class A { set foo(val) { try {} catch(e) { return; } } }", None),
        ("({ get foo() { return 1; } })", None),
        ("({ get set() { return 1; } })", None),
        ("({ set(val) { return 1; } })", None),
        ("({ set: function(val) { return 1; } })", None),
        ("({ foo: function set(val) { return 1; } })", None),
        ("({ set: function set(val) { return 1; } })", None),
        ("({ set: (val) => { return 1; } })", None),
        ("({ set: (val) => 1 })", None),
        ("set = { foo(val) { return 1; } };", None),
        ("class A { constructor(val) { return 1; } }", None),
        ("class set { constructor(val) { return 1; } }", None),
        ("class set { foo(val) { return 1; } }", None),
        ("var set = class { foo(val) { return 1; } }", None),
        ("(class set { foo(val) { return 1; } })", None),
        ("class A { get foo() { return val; } }", None),
        ("class A { get set() { return val; } }", None),
        ("class A { set(val) { return 1; } }", None),
        ("class A { static set(val) { return 1; } }", None),
        ("({ set: set = function set(val) { return 1; } } = {})", None),
        ("({ set: set = (val) => 1 } = {})", None),
        ("class C { set; foo() { return 1; } }", None),
        ("({ set foo(val) { function foo(val) { return 1; } } })", None),
        ("({ set foo(val) { var foo = function(val) { return 1; } } })", None),
        ("({ set foo(val) { var foo = (val) => { return 1; } } })", None),
        ("({ set foo(val) { var foo = (val) => 1; } })", None),
        ("({ set [function() { return 1; }](val) {} })", None),
        ("({ set [() => { return 1; }](val) {} })", None),
        ("({ set [() => 1](val) {} })", None),
        ("({ set foo(val = function() { return 1; }) {} })", None),
        ("({ set foo(val = v => 1) {} })", None),
        ("(class { set foo(val) { function foo(val) { return 1; } } })", None),
        ("(class { set foo(val) { var foo = function(val) { return 1; } } })", None),
        ("(class { set foo(val) { var foo = (val) => { return 1; } } })", None),
        ("(class { set foo(val) { var foo = (val) => 1; } })", None),
        ("(class { set [function() { return 1; }](val) {} })", None),
        ("(class { set [() => { return 1; }](val) {} })", None),
        ("(class { set [() => 1](val) {} })", None),
        ("(class { set foo(val = function() { return 1; }) {} })", None),
        ("(class { set foo(val = (v) => 1) {} })", None),
        ("Object.defineProperty(foo, 'bar', { set(val) { return; } })", None),
        ("Reflect.defineProperty(foo, 'bar', { set(val) { if (val) { return; } } })", None),
        (
            "Object.defineProperties(foo, { bar: { set(val) { try { return; } catch(e){} } } })",
            None,
        ),
        ("Object.create(foo, { bar: { set: function(val) { return; } } })", None),
        ("x = { set(val) { return 1; } }", None),
        ("x = { foo: { set(val) { return 1; } } }", None),
        ("Object.defineProperty(foo, 'bar', { value(val) { return 1; } })", None),
        ("Reflect.defineProperty(foo, 'bar', { value: function set(val) { return 1; } })", None),
        ("Object.defineProperties(foo, { bar: { [set](val) { return 1; } } })", None),
        ("Object.create(foo, { bar: { 'set ': function(val) { return 1; } } })", None),
        ("Object.defineProperty(foo, 'bar', { [`set `]: (val) => { return 1; } })", None),
        ("Reflect.defineProperty(foo, 'bar', { Set(val) { return 1; } })", None),
        ("Object.defineProperties(foo, { bar: { value: (val) => 1 } })", None),
        ("Object.create(foo, { set: { value: function(val) { return 1; } } })", None),
        ("Object.defineProperty(foo, 'bar', { baz(val) { return 1; } })", None),
        ("Reflect.defineProperty(foo, 'bar', { get(val) { return 1; } })", None),
        ("Object.create(foo, { set: function(val) { return 1; } })", None),
        ("Object.defineProperty(foo, { set: (val) => 1 })", None),
        ("Object.defineProperty(foo, 'bar', { set(val) { function foo() { return 1; } } })", None),
        (
            "Reflect.defineProperty(foo, 'bar', { set(val) { var foo = function() { return 1; } } })",
            None,
        ),
        ("Object.defineProperties(foo, { bar: { set(val) { () => { return 1 }; } } })", None),
        ("Object.create(foo, { bar: { set: (val) => { (val) => 1; } } })", None),
        ("Object.defineProperty(foo, 'bar', 'baz', { set(val) { return 1; } })", None),
        ("Object.defineProperty(foo, { set(val) { return 1; } }, 'bar')", None),
        ("Object.defineProperty({ set(val) { return 1; } }, foo, 'bar')", None),
        ("Reflect.defineProperty(foo, 'bar', 'baz', { set(val) { return 1; } })", None),
        ("Reflect.defineProperty(foo, { set(val) { return 1; } }, 'bar')", None),
        ("Reflect.defineProperty({ set(val) { return 1; } }, foo, 'bar')", None),
        ("Object.defineProperties(foo, bar, { baz: { set(val) { return 1; } } })", None),
        ("Object.defineProperties({ bar: { set(val) { return 1; } } }, foo)", None),
        ("Object.create(foo, bar, { baz: { set(val) { return 1; } } })", None),
        ("Object.create({ bar: { set(val) { return 1; } } }, foo)", None),
        ("Object.DefineProperty(foo, 'bar', { set(val) { return 1; } })", None),
        ("Reflect.DefineProperty(foo, 'bar', { set(val) { if (val) { return 1; } } })", None),
        (
            "Object.DefineProperties(foo, { bar: { set(val) { try { return 1; } catch(e){} } } })",
            None,
        ),
        ("Object.Create(foo, { bar: { set: function(val) { return 1; } } })", None),
        ("object.defineProperty(foo, 'bar', { set(val) { return 1; } })", None),
        ("reflect.defineProperty(foo, 'bar', { set(val) { if (val) { return 1; } } })", None),
        (
            "Reflect.defineProperties(foo, { bar: { set(val) { try { return 1; } catch(e){} } } })",
            None,
        ),
        ("object.create(foo, { bar: { set: function(val) { return 1; } } })", None),
        ("Reflect.defineProperty(foo, 'bar', { set(val) { if (val) { return 1; } } })", None),
        (
            "/* globals Object:off */ Object.defineProperty(foo, 'bar', { set(val) { return 1; } })",
            None,
        ),
        (
            "Object.defineProperties(foo, { bar: { set(val) { try { return 1; } catch(e){} } } })",
            None,
        ),
        ("let Object; Object.defineProperty(foo, 'bar', { set(val) { return 1; } })", None),
        (
            "function f() { Reflect.defineProperty(foo, 'bar', { set(val) { if (val) { return 1; } } }); var Reflect;}",
            None,
        ),
        (
            "function f(Object) { Object.defineProperties(foo, { bar: { set(val) { try { return 1; } catch(e){} } } }) }",
            None,
        ),
        (
            "if (x) { const Object = getObject(); Object.create(foo, { bar: { set: function(val) { return 1; } } }) }",
            None,
        ),
        (
            "x = function Object() { Object.defineProperty(foo, 'bar', { set(val) { return 1; } }) }",
            None,
        ),
    ];

    let fail = vec![
        ("({ set a(val){ return val + 1; } })", None),
        ("({ set a(val) { return 1; } })", None),
        ("class A { set a(val) { return 1; } }", None),
        ("class A { static set a(val) { return 1; } }", None),
        ("(class { set a(val) { return 1; } })", None),
        ("({ set a(val) { return val; } })", None),
        ("class A { set a(val) { return undefined; } }", None),
        ("(class { set a(val) { return null; } })", None),
        ("({ set a(val) { return x + y; } })", None),
        ("class A { set a(val) { return foo(); } }", None),
        ("(class { set a(val) { return this._a; } })", None),
        ("({ set a(val) { return this.a; } })", None),
        ("({ set a(val) { if (foo) { return 1; }; } })", None),
        ("class A { set a(val) { try { return 1; } catch(e) {} } }", None),
        ("(class { set a(val) { while (foo){ if (bar) break; else return 1; } } })", None),
        ("({ set a(val) { return 1; }, set b(val) { return 1; } })", None),
        ("class A { set a(val) { return 1; } set b(val) { return 1; } }", None),
        ("(class { set a(val) { return 1; } static set b(val) { return 1; } })", None),
        ("({ set a(val) { if(val) { return 1; } else { return 2 }; } })", None),
        (
            "class A { set a(val) { switch(val) { case 1: return x; case 2: return y; default: return z } } }",
            None,
        ),
        (
            "(class { static set a(val) { if (val > 0) { this._val = val; return val; } return false; } })",
            None,
        ),
        ("({ set a(val) { if(val) { return 1; } else { return; }; } })", None),
        (
            "class A { set a(val) { switch(val) { case 1: return x; case 2: return; default: return z } } }",
            None,
        ),
        (
            "(class { static set a(val) { if (val > 0) { this._val = val; return; } return false; } })",
            None,
        ),
        ("({ set a(val) { function b(){} return b(); } })", None),
        ("class A { set a(val) { return () => {}; } }", None),
        ("(class { set a(val) { function b(){ return 1; } return 2; } })", None),
        ("({ set a(val) { function b(){ return; } return 1; } })", None),
        ("class A { set a(val) { var x = function() { return 1; }; return 2; } }", None),
        ("(class { set a(val) { var x = () => { return; }; return 2; } })", None),
        ("function f(){}; ({ set a(val) { return 1; } });", None),
        ("x = function f(){}; class A { set a(val) { return 1; } };", None),
        ("x = () => {}; A = class { set a(val) { return 1; } };", None),
        ("return; ({ set a(val) { return 1; } }); return 2;", None),
        // ("Object.defineProperty(foo, 'bar', { set(val) { return 1; } })", None),
        // ("Reflect.defineProperty(foo, 'bar', { set(val) { return 1; } })", None),
        // ("Object.defineProperties(foo, { baz: { set(val) { return 1; } } })", None),
        // ("Object.create(null, { baz: { set(val) { return 1; } } })", None),
        // ("Object.defineProperty(foo, 'bar', { set: val => val })", None),
        // ("Reflect.defineProperty(foo, 'bar', { set: val => f(val) })", None),
        // ("Object.defineProperties(foo, { baz: { set: val => a + b } })", None),
        // ("Object.create({}, { baz: { set: val => this._val } })", None),
        // (
        // "Object.defineProperty(foo, 'bar', { set(val) { if (val) { return; } return false; }, get(val) { return 1; } })",
        // None,
        // ),
        // (
        // "Reflect.defineProperty(foo, 'bar', { set(val) { try { return f(val) } catch (e) { return e }; } })",
        // None,
        // ),
        // (
        // "Object.defineProperties(foo, { bar: { get(){ return null; }, set(val) { return null; } } })",
        // None,
        // ),
        // (
        // "Object.create(null, { baz: { set(val) { return this._val; return; return undefined; } } })",
        // None,
        // ),
        // (
        // "Object.defineProperties(foo, { baz: { set(val) { return 1; } }, bar: { set(val) { return 1; } } })",
        // None,
        // ),
        // ("Object.create({}, { baz: { set(val) { return 1; } }, bar: { set: (val) => 1 } })", None),
        // ("Object['defineProperty'](foo, 'bar', { set: function bar(val) { return 1; } })", None),
        // ("Reflect.defineProperty(foo, 'bar', { 'set'(val) { return 1; } })", None),
        // ("Object[`defineProperties`](foo, { baz: { ['set'](val) { return 1; } } })", None),
        // ("Object.create({}, { baz: { [`set`]: (val) => { return 1; } } })", None),
        // ("Object.defineProperty(foo, 'bar', { set: function Object(val) { return 1; } })", None),
        // ("Object.defineProperty(foo, 'bar', { set: function(Object) { return 1; } })", None),
        // ("Object?.defineProperty(foo, 'bar', { set(val) { return 1; } })", None),
        // ("(Object?.defineProperty)(foo, 'bar', { set(val) { return 1; } })", None),
    ];

    Tester::new(NoSetterReturn::NAME, pass, fail).test_and_snapshot();
}
