use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{AccessorProperty, Expression, PropertyDefinition},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

#[derive(Debug, Default, Clone)]
pub struct ClassMethodsUseThis;

use crate::{LintContext, rule::Rule};

fn class_methods_use_this_diagnostic(span: Span, name: Option<Cow<'_, str>>) -> OxcDiagnostic {
    let method_name_str = name.map_or(String::new(), |name| format!(" `{name}`"));
    OxcDiagnostic::warn(format!("Expected method{method_name_str} to have this."))
        .with_help(format!("Consider converting method{method_name_str} to a static method."))
        .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that class methods utilize this.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// class A {
    ///   foo() {
    ///     console.log("Hello World");
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// class A {
    ///     foo() {
    ///         this.bar = "Hello World"; // OK, this is used
    ///     }
    /// }
    ///
    /// class B {
    ///     constructor() {
    ///         // OK. constructor is exempt
    ///     }
    /// }
    ///
    /// class C {
    ///     static foo() {
    ///         // OK. static methods aren't expected to use this.
    ///     }
    /// }
    /// ```
    ClassMethodsUseThis,
    eslint,
    restriction,
);

impl Rule for ClassMethodsUseThis {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let function_pair = match node.kind() {
            AstKind::AccessorProperty(accessor) => {
                if accessor.r#static {
                    return;
                }
                accessor.value.as_ref().and_then(|value| match value {
                    Expression::ArrowFunctionExpression(arrow_function) => {
                        Some((&arrow_function.body, &accessor.key))
                    }
                    Expression::FunctionExpression(function_expression) => {
                        Some((function_expression.body.as_ref()?, &accessor.key))
                    }
                    _ => None,
                })
            }
            AstKind::MethodDefinition(method_definition) => {
                if method_definition.r#static || method_definition.kind.is_constructor() {
                    return;
                }
                let Some(function_body) = method_definition.value.body.as_ref() else { return };
                Some((function_body, &method_definition.key))
            }
            AstKind::PropertyDefinition(property_definition) => {
                if property_definition.r#static {
                    return;
                }
                property_definition.value.as_ref().and_then(|value| match value {
                    Expression::ArrowFunctionExpression(arrow_function) => {
                        Some((&arrow_function.body, &property_definition.key))
                    }
                    Expression::FunctionExpression(function_expression) => {
                        Some((function_expression.body.as_ref()?, &property_definition.key))
                    }
                    _ => None,
                })
            }
            _ => None,
        };
        let Some((function_body, name)) = function_pair else { return };
        let mut finder = ThisFinder::new();
        finder.visit_function_body(function_body);
        if !finder.has_this {
            ctx.diagnostic(class_methods_use_this_diagnostic(name.span(), name.name()));
        }
    }
}

struct ThisFinder {
    has_this: bool,
}

impl ThisFinder {
    fn new() -> Self {
        Self { has_this: false }
    }
}

impl Visit<'_> for ThisFinder {
    fn visit_this_expression(&mut self, _it: &oxc_ast::ast::ThisExpression) {
        self.has_this = true;
    }

    fn visit_super(&mut self, _it: &oxc_ast::ast::Super) {
        self.has_this = true;
    }

    fn visit_function(
        &mut self,
        _it: &oxc_ast::ast::Function<'_>,
        _flags: oxc_semantic::ScopeFlags,
    ) {
    }

    fn visit_static_block(&mut self, _it: &oxc_ast::ast::StaticBlock<'_>) {}

    fn visit_property_definition(&mut self, it: &PropertyDefinition<'_>) {
        self.visit_property_key(&it.key);
    }

    fn visit_accessor_property(&mut self, _it: &AccessorProperty<'_>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("class A { constructor() {} }", None, None),
        ("class A { foo() {this} }", None, None),
        ("class A { foo() {this.bar = 'bar';} }", None, None),
        ("class A { foo() {bar(this);} }", None, None),
        ("class A extends B { foo() {super.foo();} }", None, None),
        ("class A { foo() { if(true) { return this; } } }", None, None),
        ("class A { static foo() {} }", None, None),
        ("({ a(){} });", None, None),
        ("class A { foo() { () => this; } }", None, None),
        ("({ a: function () {} });", None, None),
        ("class A { foo = function() {this} }", None, None),
        ("class A { foo = () => {this} }", None, None),
        ("class A { foo = () => {super.toString} }", None, None),
        ("class A { static foo = function() {} }", None, None),
        ("class A { static foo = () => {} }", None, None),
        ("class A { foo() { return class { [this.foo] = 1 }; } }", None, None),
        ("class A { static {} }", None, None),
        ("class A { accessor foo = function() {this} }", None, None),
        ("class A { accessor foo = () => {this} }", None, None),
        ("class A { accessor foo = 1; }", None, None),
        ("class A { static accessor foo = function() {} }", None, None),
        ("class A { static accessor foo = () => {} }", None, None),
    ];

    let fail = vec![
        ("class A { foo() {} }", None, None),
        ("class A { foo() {/**this**/} }", None, None),
        ("class A { foo() {var a = function () {this};} }", None, None),
        ("class A { foo() {var a = function () {var b = function(){this}};} }", None, None),
        ("class A { foo() {window.this} }", None, None),
        ("class A { foo() {that.this = 'this';} }", None, None),
        ("class A { foo() { () => undefined; } }", None, None),
        (
            "class A { foo(){} 'bar'(){} 123(){} [`baz`](){} [a](){} [f(a)](){} get quux(){} set[a](b){} *quuux(){} }",
            None,
            None,
        ),
        ("class A { foo = function() {} }", None, None),
        ("class A { foo = () => {} }", None, None),
        ("class A { #foo = function() {} }", None, None),
        ("class A { #foo = () => {} }", None, None),
        ("class A { #foo() {} }", None, None),
        ("class A { get #foo() {} }", None, None),
        ("class A { set #foo(x) {} }", None, None),
        ("class A { foo () { return class { foo = this }; } }", None, None),
        ("class A { foo () { return function () { foo = this }; } }", None, None),
        ("class A { foo () { return class { static { this; } } } }", None, None),
        ("class Foo { private method() {} }", None, None),
        ("class Foo { protected method() {} }", None, None),
        ("class Foo { accessor method = function () {} }", None, None),
        ("class Foo { accessor method = () => {} }", None, None),
        ("class Foo { private accessor method = () => {} }", None, None),
        ("class Foo { protected accessor method = () => {} }", None, None),
        ("class A { foo () { return class { accessor bar = this }; } }", None, None),
        ("class Derived extends Base { override method() {} }", None, None),
        ("class Derived extends Base { property = () => {} }", None, None),
        ("class Derived extends Base { public property = () => {} }", None, None),
        ("class Derived extends Base { override property = () => {} }", None, None),
        ("class Foo { private get getter(): number {} }", None, None),
        ("class Foo { protected get getter(): number {} }", None, None),
        ("class Foo { private set setter(b: number) {} }", None, None),
        ("class Foo { protected set setter(b: number) {} }", None, None),
        ("function fn() { this.foo = 303; class Foo { method() {} } }", None, None),
        ("class Foo implements Bar { override property = () => {}; }", None, None),
    ];

    Tester::new(ClassMethodsUseThis::NAME, ClassMethodsUseThis::PLUGIN, pass, fail)
        .test_and_snapshot();
}
