use oxc_ast::{
    AstKind, MemberExpressionKind,
    ast::{
        BindingPatternKind, MethodDefinition, MethodDefinitionKind, ObjectProperty, PropertyKey,
        PropertyKind, UpdateExpression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_accessor_recursion_diagnostic(span: Span, kind: &str) -> OxcDiagnostic {
    let method_kind = match kind {
        "setters" => "set",
        _ => "get",
    };
    OxcDiagnostic::warn(format!("Disallow recursive access to `this` within {kind}."))
        .with_help(format!(
            "Remove this property access, or remove `{method_kind}` from the method"
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAccessorRecursion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow recursive access to this within getters and setters
    ///
    /// ### Why is this bad?
    ///
    /// This rule prevents recursive access to this within getter and setter methods in objects and classes,
    ///  avoiding infinite recursion and stack overflow errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const foo = {
    /// 	get bar() {
    /// 		return this.bar;
    /// 	}
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const foo = {
    /// 	get bar() {
    /// 		return this.baz;
    /// 	}
    /// };
    /// ```
    NoAccessorRecursion,
    unicorn,
    suspicious,
);

impl Rule for NoAccessorRecursion {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ThisExpression(this_expr) = node.kind() else {
            return;
        };

        let Some(target) = ctx.nodes().ancestors(node.id()).find(|n| match n.kind() {
            member_expr if member_expr.is_member_expression_kind() => {
                let Some(member_expr) = member_expr.as_member_expression_kind() else {
                    return false;
                };
                member_expr.object().without_parentheses().span() == this_expr.span()
            }
            AstKind::VariableDeclarator(decl) => decl
                .init
                .as_ref()
                .is_some_and(|init| init.without_parentheses().span() == this_expr.span()),
            _ => false,
        }) else {
            return;
        };
        // find the nearest MemberExpression or VariableDeclarator
        let Some(nearest_func) = get_nearest_function(node, ctx) else {
            return;
        };
        let func_parent = ctx.nodes().parent_node(nearest_func.id());
        if !is_property_or_method_def(func_parent) {
            return;
        }

        match target.kind() {
            AstKind::VariableDeclarator(decl) => {
                let Some(key_name) = get_property_or_method_def_name(func_parent) else {
                    return;
                };
                if let BindingPatternKind::ObjectPattern(obj_pattern) = &decl.id.kind {
                    let exist = obj_pattern
                        .properties
                        .iter()
                        .any(|ident| ident.key.name().is_some_and(|name| name == key_name));
                    if exist {
                        ctx.diagnostic(no_accessor_recursion_diagnostic(decl.span(), "getters"));
                    }
                }
            }
            member_expr if member_expr.is_member_expression_kind() => {
                let Some(member_expr) = member_expr.as_member_expression_kind() else {
                    return;
                };
                let Some(expr_key_name) = get_member_expr_key_name(&member_expr) else {
                    return;
                };
                match func_parent.kind() {
                    // e.g. "const foo = { get bar() { return this.bar }}"
                    AstKind::ObjectProperty(property) => {
                        let Some(prop_key_name) = property.key.name() else {
                            return;
                        };
                        let is_same_key = {
                            if matches!(member_expr, MemberExpressionKind::PrivateField(_)) {
                                matches!(&property.key, PropertyKey::PrivateIdentifier(_))
                                    && prop_key_name.as_ref() == expr_key_name
                            } else {
                                prop_key_name.as_ref() == expr_key_name
                            }
                        };
                        if !is_same_key {
                            return;
                        }
                        if property.kind == PropertyKind::Get {
                            ctx.diagnostic(no_accessor_recursion_diagnostic(
                                member_expr.span(),
                                "getters",
                            ));
                        }

                        if property.kind == PropertyKind::Set && is_property_write(target, ctx) {
                            ctx.diagnostic(no_accessor_recursion_diagnostic(
                                member_expr.span(),
                                "setters",
                            ));
                        }
                    }
                    // e.g. "class Foo { get bar(value) { return this.bar } }"
                    AstKind::MethodDefinition(method_def) => {
                        let Some(prop_key_name) = method_def.key.name() else {
                            return;
                        };
                        let is_same_key = {
                            if matches!(member_expr, MemberExpressionKind::PrivateField(_)) {
                                matches!(&method_def.key, PropertyKey::PrivateIdentifier(_))
                                    && prop_key_name.as_ref() == expr_key_name
                            } else {
                                prop_key_name.as_ref() == expr_key_name
                            }
                        };
                        if !is_same_key {
                            return;
                        }
                        if method_def.kind == MethodDefinitionKind::Get {
                            ctx.diagnostic(no_accessor_recursion_diagnostic(
                                member_expr.span(),
                                "getters",
                            ));
                        }

                        if method_def.kind == MethodDefinitionKind::Set
                            && is_property_write(target, ctx)
                        {
                            ctx.diagnostic(no_accessor_recursion_diagnostic(
                                member_expr.span(),
                                "setters",
                            ));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

// Check if the property is written
// e.g. "this.bar = value"
fn is_property_write<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // Check a few parent levels up for assignment contexts
    for ancestor in ctx.nodes().ancestors(node.id()).take(3) {
        match ancestor.kind() {
            // e.g. "++this.bar"
            AstKind::UpdateExpression(UpdateExpression { argument, .. }) => {
                if argument.span() == node.span() {
                    return true;
                }
            }
            // e.g. "this.bar = 1"
            AstKind::AssignmentTargetPropertyIdentifier(assign_target) => {
                if assign_target.span() == node.span() {
                    return true;
                }
            }
            // e.g. "[this.bar] = array"
            AstKind::ArrayAssignmentTarget(assign_target) => {
                if assign_target.span.contains_inclusive(node.span()) {
                    return true;
                }
            }
            AstKind::AssignmentTargetWithDefault(assign_target) => {
                if assign_target.span.contains_inclusive(node.span()) {
                    return true;
                }
            }
            // e.g. "({property: this.bar} = object)"
            AstKind::ObjectAssignmentTarget(assign_target) => {
                if assign_target.span.contains_inclusive(node.span()) {
                    return true;
                }
            }
            AstKind::AssignmentTargetPropertyProperty(assign_target) => {
                if assign_target.span.contains_inclusive(node.span()) {
                    return true;
                }
            }
            // Main assignment expression check
            AstKind::AssignmentExpression(assign_expr) => {
                if let Some(simple_target) = assign_expr.left.as_simple_assignment_target() {
                    let is_target = match simple_target {
                        oxc_ast::ast::SimpleAssignmentTarget::ComputedMemberExpression(
                            member_expr,
                        ) => member_expr.span == node.span(),
                        oxc_ast::ast::SimpleAssignmentTarget::StaticMemberExpression(
                            member_expr,
                        ) => member_expr.span == node.span(),
                        oxc_ast::ast::SimpleAssignmentTarget::PrivateFieldExpression(
                            member_expr,
                        ) => member_expr.span == node.span(),
                        _ => false,
                    };
                    if is_target {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

fn get_member_expr_key_name<'a>(expr: &'a MemberExpressionKind) -> Option<&'a str> {
    match expr {
        MemberExpressionKind::Computed(expr) => {
            expr.static_property_name().map(|name| name.as_str())
        }
        MemberExpressionKind::Static(expr) => Some(expr.property.name.as_str()),
        MemberExpressionKind::PrivateField(priv_field) => Some(priv_field.field.name.as_str()),
    }
}

fn is_property_or_method_def<'a>(parent: &'a AstNode<'a>) -> bool {
    match parent.kind() {
        AstKind::ObjectProperty(obj_prop) => {
            !obj_prop.computed && matches!(obj_prop.kind, PropertyKind::Get | PropertyKind::Set)
        }
        AstKind::MethodDefinition(method_def) => {
            !method_def.computed
                && matches!(method_def.kind, MethodDefinitionKind::Get | MethodDefinitionKind::Set)
        }
        _ => false,
    }
}

fn get_nearest_function<'a>(node: &AstNode, ctx: &'a LintContext) -> Option<&'a AstNode<'a>> {
    let mut parent = ctx.nodes().parent_node(node.id());
    loop {
        match parent.kind() {
            AstKind::Program(_) | AstKind::Function(_) => break,
            // If a class is declared in the accessor, ignore it
            // e.g. "let foo = { get bar() { class baz { } } }"
            AstKind::Class(_) => {
                return None;
            }
            _ => {
                parent = ctx.nodes().parent_node(parent.id());
            }
        }
    }
    if matches!(parent.kind(), AstKind::Function(_)) { Some(parent) } else { None }
}

fn get_property_or_method_def_name<'a>(parent: &'a AstNode<'a>) -> Option<String> {
    match parent.kind() {
        AstKind::ObjectProperty(ObjectProperty { key, .. })
        | AstKind::MethodDefinition(MethodDefinition { key, .. }) => Some(key.name()?.to_string()),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "console.log(this)",
        "function foo () { this.bar }",
        "function foo () { this.foo }",
        "function foo (value) { this.bar = value }",
        "this.foo = foo",
        "{this.foo = foo;}",
        "this.foo = function () { this.foo }",
        "const foo = () => this.foo",
        r"
            const foo = {
                bar() {
                    this.bar = void 0;
                    return this.bar;
                }
            };
        ",
        r"
            class Foo {
                foo() {
                    this.foo = void 0;
                    return this.foo;
                }
            }
        ",
        r"
            class Foo {
                set bar(value) {
                    this.bar.baz = value;
                }
            }
        ",
        r"
            class Foo {
                get bar() {
                    const self = this;
                    return self.bar;
                }
            }
        ",
        r"
            class Foo {
                set bar(value) {
                    const self = this;
                    return self.bar = value;
                }
            }
        ",
        r"
            const foo = {
                get bar() {
                    function baz() {
                        return this.bar;
                    }
                }
            };
        ",
        r"
            const foo = {
                get bar() {
                    const qux = {
                        get quux () {
                            return this.bar;
                        }
                    }
                }
            };
        ",
        r"
            const foo = {
                get bar() {
                    return this[bar];
                }
            };
        ",
        r"
            const foo = {
                get [bar]() {
                    return this.bar;
                }
            };
        ",
        r"
            const foo = {
                set bar(value) {
                    a = this.bar;
                }
            };
        ",
        r"
            class Foo{
                get bar() {
                    return this.#bar;
                }

                get #bar() {
                    return 0;
                }
            }
        ",
        r"
            class Foo{
                get bar() {
                    const {[bar]: bar} = this;
                }
            }
        ",
        r"
            const foo = {
                set bar(value) {
                    this._bar = value;
                }
            };
        ",
        r"
            const foo = {
                get bar() {
                    class Foo {
                        static {
                            this.bar
                        }
                    }
                }
            };
        ",
        r"
            const foo = {
                get bar() {
                    class Foo {
                        bar = 1;
                        baz = this.bar;
                    }
                }
            };
        ",
        r"
            const foo = {
                set bar(value) {
                    ({property: this._bar} = object)
                }
            };
        ",
        r"
            const foo = {
                get bar() {
                    class Foo {
                        set bar(val) {
                        this.bar;
                        }
                    }
                }
            };
        ",
    ];

    let fail = vec![
        r"
            const foo = {
                get bar() {
                    this.bar
                }
            };
        ",
        r"
            const foo = {
                get bar() {
                    const { bar } = this;
                }
            };
        ",
        r"
            class Foo {
        		get bar() {
        			return this.bar;
        		}
        	}
        ",
        r"
            const foo = {
        		get bar() {
        			return this.bar.baz;
        		}
        	};
        ",
        r"
            const foo = {
                set bar(value) {
                    ({property: this.bar} = object)
                }
            };
        ",
        r"
            const foo = {
                set bar(value) {
                    this.bar = value;
                }
            };
        ",
        r"
            class Foo {
                set bar(value) {
                    this.bar = value;
                }
            }
        ",
        r"
            const foo = {
                get bar() {
                    if (true) {
                        return this.bar;
                    }
                }
            };
        ",
        r"
            const foo = {
                get bar() {
                    const baz = () => {
                        return this.bar;
                    }
                }
            };
        ",
        r"
            const foo = {
                get bar() {
                    const baz = () => {
                        return () => {
                            return this.bar;
                        }
                    }
                }
            };
        ",
        r"
            const foo = {
                get bar() {
                    a = this.bar;
                }
            };
        ",
        r"
            class Foo{
                get bar() {
                    return this.#bar;
                }

                get #bar() {
                    return this.#bar
                }
            }
        ",
        r"
            class Foo{
                static get bar() {
                    return this.bar;
                }
            }
        ",
        r"
            class Foo{
                get bar() {
                    const {bar} = this;
                }
                get baz() {
                    const {baz: baz1} = this;
                }
            }
        ",
        r"
            class Foo {
                set bar(v) {
                    ++this.bar;
                }
            }
        ",
        r"
            class Foo {
                set bar(v) {
                    this.bar--;
                }
            }
        ",
        r"
            class Foo {
                set bar(v) {
                    [this.bar] = array;
                }
            }
        ",
        r"
            class Foo {
                set bar(v) {
                    [this.bar = defaultValue] = array;
                }
            }
        ",
        r"
            class Foo {
                set bar(v) {
                    ({property: this.bar} = object);
                }
            }
        ",
        r"
            class Foo {
                set bar(v) {
                    ({property: this.bar = defaultValue} = object);
                }
            }
        ",
    ];

    Tester::new(NoAccessorRecursion::NAME, NoAccessorRecursion::PLUGIN, pass, fail)
        .test_and_snapshot();
}
