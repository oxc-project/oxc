use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression,
        ObjectPropertyKind, PropertyKey,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule};

fn no_deprecated_destroyed_lifecycle_diagnostic(
    span: Span,
    deprecated: &str,
    replacement: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The `{deprecated}` lifecycle hook is deprecated. Use `{replacement}` instead."
    ))
    .with_help(format!("Replace `{deprecated}` with `{replacement}`."))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecatedDestroyedLifecycle;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using deprecated `destroyed` and `beforeDestroy` lifecycle hooks in Vue.js 3.0.0+.
    ///
    /// ### Why is this bad?
    ///
    /// In Vue.js 3.0.0+, the `destroyed` and `beforeDestroy` lifecycle hooks have been renamed
    /// to `unmounted` and `beforeUnmount` respectively. Using the old names is deprecated and
    /// may cause confusion or compatibility issues.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   beforeDestroy() {},
    ///   destroyed() {},
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   beforeUnmount() {},
    ///   unmounted() {},
    /// }
    /// </script>
    /// ```
    NoDeprecatedDestroyedLifecycle,
    vue,
    correctness,
    fix
);

impl Rule for NoDeprecatedDestroyedLifecycle {
    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExportDefaultDeclaration(export_default_decl) = node.kind() else { return };

        match &export_default_decl.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(obj_expr) => {
                check_object_properties(obj_expr, ctx);
            }
            ExportDefaultDeclarationKind::CallExpression(call_expr) => {
                check_define_component(call_expr, ctx);
            }
            _ => {}
        }
    }
}

/// Check if this is a `defineComponent({ ... })` call and check its properties
fn check_define_component<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    let Some(ident) = call_expr.callee.get_identifier_reference() else {
        return;
    };

    if ident.name != "defineComponent" {
        return;
    }

    let Some(first_arg) = call_expr.arguments.first() else {
        return;
    };

    let Some(Expression::ObjectExpression(obj_expr)) = first_arg.as_expression() else {
        return;
    };

    check_object_properties(obj_expr, ctx);
}

/// Check object properties for deprecated lifecycle hooks
fn check_object_properties<'a>(obj_expr: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
    let mut existing_keys = FxHashSet::default();
    for prop in &obj_expr.properties {
        let ObjectPropertyKind::ObjectProperty(obj_prop) = prop else {
            continue;
        };
        if let Some(name) = obj_prop.key.static_name() {
            existing_keys.insert(name.into_owned());
        }
    }

    for prop in &obj_expr.properties {
        let ObjectPropertyKind::ObjectProperty(obj_prop) = prop else {
            continue;
        };

        let Some((key_name, key_span, is_shorthand, key_type)) = get_property_key_info(obj_prop)
        else {
            continue;
        };

        let replacement = match key_name.as_str() {
            "beforeDestroy" => "beforeUnmount",
            "destroyed" => "unmounted",
            _ => continue,
        };

        // If the replacement hook already exists in this object, an autofix could create
        // duplicate keys and change semantics (later keys overwrite earlier ones).
        if existing_keys.contains(replacement) {
            ctx.diagnostic(no_deprecated_destroyed_lifecycle_diagnostic(
                key_span,
                &key_name,
                replacement,
            ));
            continue;
        }

        existing_keys.insert(replacement.to_string());

        ctx.diagnostic_with_fix(
            no_deprecated_destroyed_lifecycle_diagnostic(key_span, &key_name, replacement),
            |fixer| {
                // Format replacement based on key type
                let formatted_replacement = match key_type {
                    KeyType::Identifier => {
                        if is_shorthand {
                            // For shorthand properties like `beforeDestroy,` we need to convert to
                            // `beforeUnmount: beforeDestroy,`
                            format!("{replacement}:{key_name}")
                        } else {
                            replacement.to_string()
                        }
                    }
                    KeyType::StringLiteral => {
                        // Preserve single quotes for string literals
                        format!("'{replacement}'")
                    }
                    KeyType::TemplateLiteral => {
                        // Preserve backticks for template literals
                        format!("`{replacement}`")
                    }
                };
                fixer.replace(key_span, formatted_replacement)
            },
        );
    }
}

/// Type of property key to determine correct fix format
#[derive(Clone, Copy)]
enum KeyType {
    Identifier,
    StringLiteral,
    TemplateLiteral,
}

/// Get the property key name, span, whether it's a shorthand property, and key type
fn get_property_key_info(
    prop: &oxc_ast::ast::ObjectProperty,
) -> Option<(String, Span, bool, KeyType)> {
    match &prop.key {
        PropertyKey::StaticIdentifier(ident) => {
            Some((ident.name.to_string(), ident.span, prop.shorthand, KeyType::Identifier))
        }
        PropertyKey::StringLiteral(lit) => {
            Some((lit.value.to_string(), lit.span, false, KeyType::StringLiteral))
        }
        PropertyKey::TemplateLiteral(tpl) if tpl.is_no_substitution_template() => {
            tpl.single_quasi().map(|s| (s.to_string(), tpl.span, false, KeyType::TemplateLiteral))
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
			      <script>
			      export default {
			        unmounted () {},
			        beforeUnmount () {},
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        unmounted,
			        beforeUnmount,
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        beforeCreate,
			        created,
			        beforeMount,
			        mounted,
			        beforeUpdate,
			        updated,
			        activated,
			        deactivated,
			        beforeUnmount,
			        unmounted,
			        errorCaptured,
			        renderTracked,
			        renderTriggered,
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        beforeUnmount:beforeDestroy,
			        unmounted:destroyed,
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        ...beforeDestroy,
			        ...destroyed,
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        [beforeDestroy] () {},
			        [destroyed] () {},
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "
			      <script>
			      export default {
			        beforeDestroy () {},
			        destroyed () {},
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        beforeDestroy,
			        destroyed,
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        beforeCreate,
			        created,
			        beforeMount,
			        mounted,
			        beforeUpdate,
			        updated,
			        activated,
			        deactivated,
			        beforeDestroy,
			        destroyed,
			        errorCaptured,
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        ['beforeDestroy']() {},
			        ['destroyed']() {},
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        [`beforeDestroy`]() {},
			        [`destroyed`]() {},
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fix = vec![
        (
            "
			      <script>
			      export default {
			        beforeDestroy () {},
			        destroyed () {},
			      }
			      </script>
			      ",
            "
			      <script>
			      export default {
			        beforeUnmount () {},
			        unmounted () {},
			      }
			      </script>
			      ",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        beforeDestroy,
			        destroyed,
			      }
			      </script>
			      ",
            "
			      <script>
			      export default {
			        beforeUnmount:beforeDestroy,
			        unmounted:destroyed,
			      }
			      </script>
			      ",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        beforeCreate,
			        created,
			        beforeMount,
			        mounted,
			        beforeUpdate,
			        updated,
			        activated,
			        deactivated,
			        beforeDestroy,
			        destroyed,
			        errorCaptured,
			      }
			      </script>
			      ",
            "
			      <script>
			      export default {
			        beforeCreate,
			        created,
			        beforeMount,
			        mounted,
			        beforeUpdate,
			        updated,
			        activated,
			        deactivated,
			        beforeUnmount:beforeDestroy,
			        unmounted:destroyed,
			        errorCaptured,
			      }
			      </script>
			      ",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        ['beforeDestroy']() {},
			        ['destroyed']() {},
			      }
			      </script>
			      ",
            "
			      <script>
			      export default {
			        ['beforeUnmount']() {},
			        ['unmounted']() {},
			      }
			      </script>
			      ",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        [`beforeDestroy`]() {},
			        [`destroyed`]() {},
			      }
			      </script>
			      ",
            "
			      <script>
			      export default {
			        [`beforeUnmount`]() {},
			        [`unmounted`]() {},
			      }
			      </script>
			      ",
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];
    Tester::new(
        NoDeprecatedDestroyedLifecycle::NAME,
        NoDeprecatedDestroyedLifecycle::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
