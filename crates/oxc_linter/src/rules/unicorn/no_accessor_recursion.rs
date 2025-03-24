use oxc_ast::{
    AstKind,
    ast::{
        BindingPatternKind, Expression, MemberExpression, MethodDefinition, MethodDefinitionKind,
        ObjectProperty, PropertyKey, PropertyKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_accessor_recursion_diagnostic(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Disallow recursive access to `this` within {kind}."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAccessorRecursion;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoAccessorRecursion,
    unicorn,
    correctness,
);

impl Rule for NoAccessorRecursion {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclarator(decl) => {
                // "const { baz } = this"
                if let Some(init) = &decl.init {
                    if !matches!(init.without_parentheses(), Expression::ThisExpression(_)) {
                        return;
                    }
                    let Some(func) = get_closest_function(node, ctx) else {
                        return;
                    };
                    if !is_parent_property_or_method_def(func, ctx) {
                        return;
                    }
                    let Some(key_name) = get_property_or_method_def_name(func, ctx) else {
                        return;
                    };
                    if let BindingPatternKind::ObjectPattern(obj_pattern) = &decl.id.kind {
                        let exist = obj_pattern.properties.iter().find(|ident| {
                            ident.key.name().is_some_and(|name| name.to_string() == key_name)
                        });
                        if exist.is_some() {
                            ctx.diagnostic(no_accessor_recursion_diagnostic(
                                decl.span(),
                                "getters",
                            ));
                        }
                    }
                }
            }
            AstKind::MemberExpression(member_expr) => {
                if !matches!(member_expr.object(), Expression::ThisExpression(_)) {
                    return;
                }
                let Some(expr_key_name) = get_member_expr_key_name(member_expr) else {
                    return;
                };
                let Some(func) = get_closest_function(node, ctx) else {
                    return;
                };
                if !is_parent_property_or_method_def(func, ctx) {
                    return;
                }
                if let Some(prop_or_method) = ctx.nodes().parent_node(func.id()) {
                    match prop_or_method.kind() {
                        AstKind::ObjectProperty(property) => {
                            let Some(prop_key_name) = property.key.name() else {
                                return;
                            };
                            let is_property_read = {
                                if matches!(
                                    member_expr,
                                    MemberExpression::PrivateFieldExpression(_)
                                ) {
                                    matches!(&property.key, PropertyKey::PrivateIdentifier(_))
                                        && prop_key_name.as_ref() == expr_key_name
                                } else {
                                    prop_key_name.as_ref() == expr_key_name
                                }
                            };
                            if property.kind == PropertyKind::Get && is_property_read {
                                ctx.diagnostic(no_accessor_recursion_diagnostic(
                                    member_expr.span(),
                                    "getters",
                                ));
                            }
                            if property.kind == PropertyKind::Set {
                                ctx.diagnostic(no_accessor_recursion_diagnostic(
                                    member_expr.span(),
                                    "setters",
                                ));
                            }
                        }
                        AstKind::MethodDefinition(method_def) => {
                            let Some(prop_key_name) = method_def.key.name() else {
                                return;
                            };
                            let is_property_read = {
                                if matches!(
                                    member_expr,
                                    MemberExpression::PrivateFieldExpression(_)
                                ) {
                                    matches!(&method_def.key, PropertyKey::PrivateIdentifier(_))
                                        && prop_key_name.as_ref() == expr_key_name
                                } else {
                                    prop_key_name.as_ref() == expr_key_name
                                }
                            };
                            if method_def.kind == MethodDefinitionKind::Get && is_property_read {
                                ctx.diagnostic(no_accessor_recursion_diagnostic(
                                    member_expr.span(),
                                    "getters",
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn get_member_expr_key_name<'a>(expr: &'a MemberExpression) -> Option<&'a str> {
    match expr {
        MemberExpression::ComputedMemberExpression(_)
        | MemberExpression::StaticMemberExpression(_) => expr.static_property_name(),
        MemberExpression::PrivateFieldExpression(priv_field) => {
            Some(priv_field.field.name.as_str())
        }
    }
}

fn is_parent_property_or_method_def<'a>(node: &'a AstNode<'a>, ctx: &'a LintContext) -> bool {
    if let Some(parent) = ctx.nodes().parent_node(node.id()) {
        match parent.kind() {
            AstKind::ObjectProperty(obj_prop) => {
                !obj_prop.computed && matches!(obj_prop.kind, PropertyKind::Get | PropertyKind::Set)
            }
            AstKind::MethodDefinition(method_def) => {
                !method_def.computed
                    && matches!(
                        method_def.kind,
                        MethodDefinitionKind::Get | MethodDefinitionKind::Set
                    )
            }
            _ => false,
        }
    } else {
        false
    }
}

fn get_closest_function<'a>(node: &AstNode, ctx: &'a LintContext) -> Option<&'a AstNode<'a>> {
    let mut parent = ctx.nodes().parent_node(node.id())?;

    loop {
        match parent.kind() {
            AstKind::Function(_) => {
                break;
            }
            _ => {
                parent = ctx.nodes().parent_node(parent.id())?;
            }
        }
    }
    Some(parent)
}

fn get_property_or_method_def_name<'a>(
    node: &'a AstNode<'a>,
    ctx: &'a LintContext,
) -> Option<String> {
    let parent = ctx.nodes().parent_node(node.id())?;
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
        r"
            const foo = {
                set bar(value) {
                    this._bar = value;
                }
            };
        ",
    ];

    let fail = vec![
        r"
            const foo = {
                get bar(value) {
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
    ];

    Tester::new(NoAccessorRecursion::NAME, NoAccessorRecursion::PLUGIN, pass, fail)
        .test_and_snapshot();
}
