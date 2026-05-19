use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectExpression, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_vue_component_options_object};

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
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ThisExpression(this_expr) = node.kind() else { return };

        let nodes = ctx.nodes();

        // Walk up to the lexical `this` binding. Arrow functions are skipped
        // because they bind `this` lexically.
        let Some(fn_id) = nodes
            .ancestors(node.id())
            .find(|ancestor| matches!(ancestor.kind(), AstKind::Function(_)))
            .map(AstNode::id)
        else {
            return;
        };

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

        let prop_definition_node = nodes.parent_node(opts_node.id());
        if !matches!(prop_definition_node.kind(), AstKind::ObjectProperty(_)) {
            return;
        }

        let props_object_node = nodes.parent_node(prop_definition_node.id());
        if !matches!(props_object_node.kind(), AstKind::ObjectExpression(_)) {
            return;
        }

        let props_node = nodes.parent_node(props_object_node.id());
        let AstKind::ObjectProperty(props_prop) = props_node.kind() else { return };
        if !props_prop.key.is_specific_static_name("props") {
            return;
        }

        let component_options_node = nodes.parent_node(props_node.id());
        if !is_vue_component_options_object(component_options_node, ctx) {
            return;
        }

        ctx.diagnostic(no_deprecated_props_default_this_diagnostic(this_expr.span));
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
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
    expr.get_inner_expression().is_specific_id("Function")
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
        (
            "
                    <template><div /></template>
                    <script>
                    export default {
                      props: {
                        a: {
                          validator () {
                            return {
                              default () {
                                return this.a
                              }
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
                    export default {
                      props: {
                        a: {
                          nested: {
                            default () {
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
