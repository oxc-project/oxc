use std::borrow::Cow;

use itertools::Itertools;
use oxc_ast::{
    AstKind,
    ast::{AccessorProperty, Expression, PropertyDefinition, TSAccessibility},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{LintContext, rule::Rule};

fn class_methods_use_this_diagnostic(span: Span, name: Option<Cow<'_, str>>) -> OxcDiagnostic {
    let method_name_str = name.map_or(String::new(), |name| format!(" `{name}`"));
    OxcDiagnostic::warn(format!("Expected method{method_name_str} to have this."))
        .with_help(format!("Consider converting method{method_name_str} to a static method."))
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct ClassMethodsUseThisConfig {
    except_methods: Vec<MethodException>,
    enforce_for_class_fields: bool,
    ignore_override_methods: bool,
    ignore_classes_with_implements: Option<IgnoreClassWithImplements>,
}

impl Default for ClassMethodsUseThisConfig {
    fn default() -> Self {
        Self {
            except_methods: Vec::new(),
            enforce_for_class_fields: true,
            ignore_override_methods: false,
            ignore_classes_with_implements: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClassMethodsUseThis(Box<ClassMethodsUseThisConfig>);

#[derive(Debug, Clone)]
struct MethodException {
    name: CompactStr,
    private: bool,
}

#[derive(Debug, Clone)]
enum IgnoreClassWithImplements {
    All,
    PublicFields,
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
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        Self(Box::new(ClassMethodsUseThisConfig {
            except_methods: obj
                .and_then(|o| o.get("exceptMethods"))
                .and_then(|v| v.as_array())
                .map_or(Vec::new(), |a| {
                    a.iter()
                        .filter_map(|method| {
                            let method = method.as_str()?;
                            match method.strip_prefix("#") {
                                Some(method) => {
                                    Some(MethodException { name: method.into(), private: true })
                                }
                                None => {
                                    Some(MethodException { name: method.into(), private: false })
                                }
                            }
                        })
                        .collect_vec()
                }),
            enforce_for_class_fields: obj
                .and_then(|o| o.get("enforceForClassFields"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
            ignore_override_methods: obj
                .and_then(|o| o.get("ignoreOverrideMethods"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            ignore_classes_with_implements: obj
                .and_then(|o| o.get("ignoreClassesWithImplements"))
                .and_then(|v| v.as_str())
                .map(|s| match s {
                    "public-fields" => IgnoreClassWithImplements::PublicFields,
                    _ => IgnoreClassWithImplements::All,
                }),
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let config = &self.0;
        let function_pair = match node.kind() {
            AstKind::AccessorProperty(accessor) => {
                if accessor.r#static
                    || !config.enforce_for_class_fields
                    || (config.ignore_override_methods && accessor.r#override)
                    || self.check_ignore_classes_with_implements(
                        node,
                        ctx,
                        accessor.accessibility,
                        accessor.key.is_private_identifier(),
                    )
                {
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
                if method_definition.r#static
                    || method_definition.kind.is_constructor()
                    || (config.ignore_override_methods && method_definition.r#override)
                    || self.check_ignore_classes_with_implements(
                        node,
                        ctx,
                        method_definition.accessibility,
                        method_definition.key.is_private_identifier(),
                    )
                {
                    return;
                }
                let Some(function_body) = method_definition.value.body.as_ref() else { return };
                Some((function_body, &method_definition.key))
            }
            AstKind::PropertyDefinition(property_definition) => {
                if property_definition.r#static
                    || !config.enforce_for_class_fields
                    || (config.ignore_override_methods && property_definition.r#override)
                    || self.check_ignore_classes_with_implements(
                        node,
                        ctx,
                        property_definition.accessibility,
                        property_definition.key.is_private_identifier(),
                    )
                {
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
        if let Some(name_str) = name.name() {
            if config.except_methods.iter().any(|method| {
                method.name == name_str && method.private == name.is_private_identifier()
            }) {
                return;
            }
        }
        let mut finder = ThisFinder::new();
        finder.visit_function_body(function_body);
        if !finder.has_this {
            ctx.diagnostic(class_methods_use_this_diagnostic(name.span(), name.name()));
        }
    }
}

impl ClassMethodsUseThis {
    fn check_ignore_classes_with_implements(
        &self,
        node: &AstNode<'_>,
        ctx: &LintContext<'_>,
        accessibility: Option<TSAccessibility>,
        is_private: bool,
    ) -> bool {
        let config = &self.0;
        let Some(ignore_classes_with_implements) = &config.ignore_classes_with_implements else {
            return false;
        };
        let mut current_node = node;
        loop {
            current_node = ctx.nodes().parent_node(current_node.id());
            let AstKind::Class(class) = current_node.kind() else {
                continue;
            };
            if class.implements.is_empty() {
                return false;
            }
            return match ignore_classes_with_implements {
                IgnoreClassWithImplements::All => true,
                IgnoreClassWithImplements::PublicFields => accessibility
                    .map_or(!is_private, |accessibility| accessibility == TSAccessibility::Public),
            };
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
        (
            "class A { foo() {this} bar() {} }",
            Some(serde_json::json!([{ "exceptMethods": ["bar"] }])),
            None,
        ),
        (
            "class A { \"foo\"() { } }",
            Some(serde_json::json!([{ "exceptMethods": ["foo"] }])),
            None,
        ),
        ("class A { 42() { } }", Some(serde_json::json!([{ "exceptMethods": ["42"] }])), None),
        ("class A { #bar() {} }", Some(serde_json::json!([{ "exceptMethods": ["#bar"] }])), None),
        (
            "class A { foo = function () {} }",
            Some(serde_json::json!([{ "enforceForClassFields": false }])),
            None,
        ),
        (
            "class A { foo = () => {} }",
            Some(serde_json::json!([{ "enforceForClassFields": false }])),
            None,
        ),
        (
            "class Foo { override method() {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { private override method() {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { protected override method() {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { override accessor method = () => {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { override get getter(): number {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { private override get getter(): number {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { protected override get getter(): number {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { override set setter(v: number) {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { private override set setter(v: number) {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { protected override set setter(v: number) {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo implements Bar { override method() {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "all" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { private override method() {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "public-fields" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { protected override method() {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "public-fields" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { override get getter(): number {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "all" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { private override get getter(): number {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "public-fields" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { protected override get getter(): number {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "public-fields" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { override set setter(v: number) {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "all" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { private override set setter(v: number) {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "public-fields" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { protected override set setter(v: number) {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "public-fields" }]),
            ),
            None,
        ),
        (
            "class Foo { override property = () => {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { private override property = () => {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo { protected override property = () => {} }",
            Some(serde_json::json!([{ "ignoreOverrideMethods": true }])),
            None,
        ),
        (
            "class Foo implements Bar { override property = () => {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "all" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { private override property = () => {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "public-fields" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { protected override property = () => {} }",
            Some(
                serde_json::json!([{ "ignoreOverrideMethods": true, "ignoreClassesWithImplements": "public-fields" }]),
            ),
            None,
        ),
        (
            "class Foo implements Bar { method() {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "all" }])),
            None,
        ),
        (
            "class Foo implements Bar { accessor method = () => {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "all" }])),
            None,
        ),
        (
            "class Foo implements Bar { get getter() {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "all" }])),
            None,
        ),
        (
            "class Foo implements Bar { set setter(value: string) {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "all" }])),
            None,
        ),
        (
            "class Foo implements Bar { property = () => {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "all" }])),
            None,
        ),
        (
            "class A { accessor foo = function () {} }",
            Some(serde_json::json!([{ "enforceForClassFields": false }])),
            None,
        ),
        (
            "class A { accessor foo = () => {} }",
            Some(serde_json::json!([{ "enforceForClassFields": false }])),
            None,
        ),
        (
            "class A { override foo = () => {} }",
            Some(serde_json::json!([{ "enforceForClassFields": false }])),
            None,
        ),
        (
            "class Foo implements Bar { property = () => {} }",
            Some(serde_json::json!([{ "enforceForClassFields": false }])),
            None,
        ),
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
        (
            "class A { foo() {} bar() {} }",
            Some(serde_json::json!([{ "exceptMethods": ["bar"] }])),
            None,
        ),
        (
            "class A { foo() {} hasOwnProperty() {} }",
            Some(serde_json::json!([{ "exceptMethods": ["foo"] }])),
            None,
        ),
        ("class A { [foo]() {} }", Some(serde_json::json!([{ "exceptMethods": ["foo"] }])), None),
        (
            "class A { #foo() { } foo() {} #bar() {} }",
            Some(serde_json::json!([{ "exceptMethods": ["#foo"] }])),
            None,
        ),
        (
            "class Foo implements Bar { #method() {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { private method() {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { protected method() {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { get #getter(): number {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { private get getter(): number {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { protected get getter(): number {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { set #setter(v: number) {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { private set setter(v: number) {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { protected set setter(v: number) {} }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { #property = () => {}; }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { private property = () => {}; }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
        (
            "class Foo implements Bar { protected property = () => {}; }",
            Some(serde_json::json!([{ "ignoreClassesWithImplements": "public-fields" }])),
            None,
        ),
    ];

    Tester::new(ClassMethodsUseThis::NAME, ClassMethodsUseThis::PLUGIN, pass, fail)
        .test_and_snapshot();
}
