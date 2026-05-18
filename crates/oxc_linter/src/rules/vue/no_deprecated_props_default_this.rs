use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, MemberExpression, ObjectExpression, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_deprecated_props_default_this_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Props default value factory functions no longer have access to `this`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecatedPropsDefaultThis;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow deprecated `this` access in props default function (in Vue.js 3.0.0+).
    ///
    /// ### Why is this bad?
    ///
    /// In Vue.js 3.0.0+, props default factory functions no longer have access to
    /// `this`. They are invoked before the component instance is created, so
    /// `this` is `undefined`. The factory should rely on its first argument (the
    /// raw props passed by the parent) instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: {
    ///     a: String,
    ///     b: {
    ///       default() {
    ///         return this.a
    ///       }
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: {
    ///     a: String,
    ///     b: {
    ///       default(props) {
    ///         return props.a
    ///       }
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    NoDeprecatedPropsDefaultThis,
    vue,
    correctness,
    pending,
    version = "next",
);

impl Rule for NoDeprecatedPropsDefaultThis {
    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ThisExpression(this_expr) = node.kind() else { return };

        let nodes = ctx.nodes();

        // Walk up to the lexical `this` binding. Arrow functions are skipped
        // because they bind `this` lexically.
        let mut binding_fn_id = None;
        for ancestor in nodes.ancestors(node.id()) {
            if matches!(ancestor.kind(), AstKind::Function(_)) {
                binding_fn_id = Some(ancestor.id());
                break;
            }
        }
        let Some(fn_id) = binding_fn_id else { return };

        let prop_node = nodes.parent_node(fn_id);
        let AstKind::ObjectProperty(default_prop) = prop_node.kind() else { return };
        if !default_prop.key.is_specific_static_name("default") {
            return;
        }

        // `type: Function` means `default` is the function value itself, not a
        // factory that returns a value, so `this` inside is not the deprecated case.
        let opts_node = nodes.parent_node(prop_node.id());
        let AstKind::ObjectExpression(opts) = opts_node.kind() else { return };
        if has_function_type(opts) {
            return;
        }

        let in_props = nodes.ancestors(fn_id).any(|ancestor| {
            matches!(
                ancestor.kind(),
                AstKind::ObjectProperty(op) if op.key.is_specific_static_name("props")
            )
        });
        if !in_props {
            return;
        }

        let in_vue_component =
            nodes.ancestors(fn_id).any(|ancestor| is_vue_component_root(ancestor.kind()));
        if !in_vue_component {
            return;
        }

        ctx.diagnostic(no_deprecated_props_default_this_diagnostic(this_expr.span));
    }
}

fn is_vue_component_root(kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::ExportDefaultDeclaration(_) => true,
        AstKind::CallExpression(call) => is_vue_component_definition_call(call),
        AstKind::NewExpression(new_expr) => {
            new_expr.callee.get_identifier_reference().is_some_and(|ident| ident.name == "Vue")
        }
        _ => false,
    }
}

fn is_vue_component_definition_call(call: &CallExpression<'_>) -> bool {
    let callee = call.callee.get_inner_expression();

    if let Expression::Identifier(ident) = callee {
        return matches!(
            ident.name.as_str(),
            "defineComponent" | "component" | "createApp" | "defineNuxtComponent"
        );
    }

    let Some(MemberExpression::StaticMemberExpression(static_member)) =
        callee.as_member_expression()
    else {
        return false;
    };
    let prop_name = static_member.property.name.as_str();
    if let Expression::Identifier(obj_ident) = static_member.object.get_inner_expression()
        && obj_ident.name == "Vue"
    {
        return matches!(prop_name, "component" | "mixin" | "extend");
    }
    matches!(prop_name, "component" | "mixin")
}

fn has_function_type(opts: &ObjectExpression<'_>) -> bool {
    for property_kind in &opts.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = property_kind else { continue };
        if !prop.key.is_specific_static_name("type") {
            continue;
        }
        if is_function_identifier(&prop.value) {
            return true;
        }
        if let Expression::ArrayExpression(arr) = prop.value.get_inner_expression() {
            return arr
                .elements
                .iter()
                .any(|el| el.as_expression().is_some_and(is_function_identifier));
        }
    }
    false
}

fn is_function_identifier(expr: &Expression<'_>) -> bool {
    if let Expression::Identifier(ident) = expr.get_inner_expression() {
        return ident.name == "Function";
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
                    <template><div /></template>
                    <script>
                    export default {
                      props: {
                        a: String,
                        b: {
                          default (props) {
                            return props.a
                          }
                        }
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <template><div /></template>
                    <script>
                    export default {
                      props: {
                        a: String,
                        b: {
                          default: () => {
                            return this.a
                          }
                        }
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <template><div /></template>
                    <script>
                    export default {
                      props: {
                        a: String,
                        b: {
                          default () {
                            return function () {
                              return this.a
                            }
                          }
                        }
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <template><div /></template>
                    <script>
                    const Foo = {
                      props: {
                        a: String,
                        b: {
                          default () {
                            return this.a
                          }
                        }
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <template>
                    <button @click="printMessage">Print message</button>
                  </template>

                  <script>

                  export default {
                    name: 'App',
                    props: {
                      message: String,
                      printMessage: {
                        type: Function,
                        default() {
                          console.log(this.message);
                        }
                      }
                    }
                  }
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "
                    <template><div /></template>
                    <script>
                    export default {
                      props: {
                        a: String,
                        b: {
                          default () {
                            return this.a
                          }
                        }
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <template><div /></template>
                    <script>
                    export default {
                      props: {
                        a: String,
                        b: {
                          default () {
                            return () => this.a
                          }
                        }
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <template><div /></template>
                    <script>
                    export default {
                      props: {
                        a: String,
                        b: {
                          default () {
                            return this.a
                          }
                        },
                        c: {
                          default () {
                            return this.a
                          }
                        }
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(
        NoDeprecatedPropsDefaultThis::NAME,
        NoDeprecatedPropsDefaultThis::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
