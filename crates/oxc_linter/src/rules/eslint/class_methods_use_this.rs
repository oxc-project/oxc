use oxc_ast::{
    AstKind,
    ast::{AccessorProperty, MethodDefinition, MethodDefinitionKind, PropertyDefinition},
};
use oxc_cfg::{BlockNodeId, EdgeType, visit::neighbors_filtered_by_edge_weight};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

#[derive(Debug, Default, Clone)]
pub struct ClassMethodsUseThis;

use crate::{LintContext, rule::Rule};

fn class_methods_use_this_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected method to have this.")
        .with_help("Consider converting method to a static method.")
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
    fn run_once(&self, ctx: &LintContext) {
        let mut wanted_nodes = Vec::new();
        let mut basic_blocks_with_this_called = FxHashSet::<BlockNodeId>::default();
        let mut basic_blocks_with_functions = FxHashSet::<BlockNodeId>::default();
        for node in ctx.nodes() {
            match node.kind() {
                AstKind::ThisExpression(_) | AstKind::Super(_) => {
                    basic_blocks_with_this_called.insert(node.cfg_id());
                }
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    let parent = ctx.nodes().parent_kind(node.id());
                    if let Some(MethodDefinition { kind, r#static, .. }) =
                        parent.as_method_definition()
                    {
                        if !r#static && *kind != MethodDefinitionKind::Constructor {
                            wanted_nodes.push(node);
                        }
                    } else if let Some(PropertyDefinition { r#static, .. }) =
                        parent.as_property_definition()
                    {
                        if !r#static {
                            wanted_nodes.push(node);
                        }
                    } else if let Some(AccessorProperty { r#static, .. }) =
                        parent.as_accessor_property()
                    {
                        if !r#static {
                            wanted_nodes.push(node);
                        }
                    } else if node.kind().as_function().is_some() {
                        basic_blocks_with_functions.insert(node.cfg_id());
                    }
                }
                _ => {}
            }
        }
        let cfg = ctx.cfg();
        for node in wanted_nodes {
            let output = neighbors_filtered_by_edge_weight(
                &cfg.graph,
                node.cfg_id(),
                &|edge| match edge {
                    EdgeType::Jump | EdgeType::Normal | EdgeType::NewFunction => None,
                    EdgeType::Unreachable
                    | EdgeType::Join
                    | EdgeType::Error(_)
                    | EdgeType::Finalize
                    | EdgeType::Backedge => Some(false),
                },
                &mut |basic_block_id, _| {
                    if basic_blocks_with_functions.contains(basic_block_id) {
                        (false, false)
                    } else if basic_blocks_with_this_called.contains(basic_block_id) {
                        (true, false)
                    } else {
                        (false, true)
                    }
                },
            );
            let has_this = output.iter().any(|y| *y);
            if !has_this {
                ctx.diagnostic(class_methods_use_this_diagnostic(node.span()));
            }
        }
    }
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
        // ("class A { foo () { return class { foo = this }; } }", None, None),
        ("class A { foo () { return function () { foo = this }; } }", None, None),
        // ("class A { foo () { return class { static { this; } } } }", None, None),
        ("class Foo { private method() {} }", None, None),
        ("class Foo { protected method() {} }", None, None),
        ("class Foo { accessor method = function () {} }", None, None),
        ("class Foo { accessor method = () => {} }", None, None),
        ("class Foo { private accessor method = () => {} }", None, None),
        ("class Foo { protected accessor method = () => {} }", None, None),
        // ("class A { foo () { return class { accessor bar = this }; } }", None, None),
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
