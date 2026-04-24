use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_deprecated_dollar_listeners_api_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `$listeners` is deprecated.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecatedDollarListenersApi;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using deprecated `$listeners` (in Vue.js 3.0.0+).
    ///
    /// ### Why is this bad?
    ///
    /// The `$listeners` object was removed in Vue 3. Component v-on listeners
    /// are now part of `$attrs`, so code that reads `this.$listeners` in a
    /// Vue component will break when upgrading to Vue 3.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   computed: {
    ///     foo () {
    ///       return this.$listeners
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
    ///   methods: {
    ///     click () {
    ///       this.$emit('click')
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    NoDeprecatedDollarListenersApi,
    vue,
    correctness,
    version = "next",
);

impl Rule for NoDeprecatedDollarListenersApi {
    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StaticMemberExpression(outer) = node.kind() else {
            return;
        };
        if outer.property.name != "$listeners" {
            return;
        }
        let in_vue = ctx.nodes().ancestors(node.id()).any(|a| match a.kind() {
            AstKind::ExportDefaultDeclaration(_) => true,
            AstKind::CallExpression(call) => call
                .callee
                .get_identifier_reference()
                .is_some_and(|ident| ident.name == "defineComponent"),
            _ => false,
        });
        if !in_vue {
            return;
        }
        match outer.object.get_inner_expression() {
            Expression::ThisExpression(_) => {}
            Expression::Identifier(ident) => {
                let scoping = ctx.scoping();
                let reference = scoping.get_reference(ident.reference_id());
                let Some(symbol_id) = reference.symbol_id() else { return };

                let declaration = ctx.symbol_declaration(symbol_id);
                let AstKind::VariableDeclarator(decl) = declaration.kind() else { return };

                let BindingPattern::BindingIdentifier(_) = &decl.id else { return };
                let Some(init) = &decl.init else { return };
                let Expression::ThisExpression(_) = init.get_inner_expression() else { return };
            }
            _ => return,
        }

        ctx.diagnostic(no_deprecated_dollar_listeners_api_diagnostic(outer.property.span));
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
                      mounted () {
                        this.$emit('start')
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
                    <script>
                    export default {
                      methods: {
                        click () {
                          this.$emit('click')
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
                    <script>
                    export default {
                    }
                    const another = function () {
                      console.log(this.$listeners)
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
                      methods: {
                        click ($listeners) {
                          foo.$listeners
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
                    <script>
                    export default {
                      computed: {
                        foo () {
                          const {vm} = this
                          return vm.$listeners
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
                    <script>
                    export default {
                      computed: {
                        foo () {
                          return this.$listeners
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
                    <script>
                    export default {
                      computed: {
                        foo () {
                          fn(this.$listeners)
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
                    <script>
                    export default {
                      computed: {
                        foo () {
                          const vm = this
                          return vm.$listeners
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
                    <script>
                    export default {
                      computed: {
                        foo () {
                          const vm = this
                          function fn() {
                            return vm.$listeners
                          }
                          return fn()
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
                    <script>
                    export default {
                      computed: {
                        foo () {
                          const vm = this
                          const a = vm?.$listeners
                          const b = this?.$listeners
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
                    <script>
                    export default defineComponent({
                      computed: {
                        foo () {
                          return this.$listeners
                        }
                      }
                    })
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    defineComponent({
                      computed: {
                        foo () {
                          return this.$listeners
                        }
                      }
                    })
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(
        NoDeprecatedDollarListenersApi::NAME,
        NoDeprecatedDollarListenersApi::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
