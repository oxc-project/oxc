use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{definitely_returns_in_all_codepaths, get_computed_getter_context},
};

fn return_in_computed_property_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected to return a value in computed property.")
        .with_help("All code paths inside a computed getter must return a value.")
        .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct ReturnInComputedPropertyConfig {
    /// When `true` (default), `return;` (without a value) is treated as a missing return.
    /// Set to `false` to allow bare `return;` as if it returned a value.
    treat_undefined_as_unspecified: bool,
}

impl Default for ReturnInComputedPropertyConfig {
    fn default() -> Self {
        Self { treat_undefined_as_unspecified: true }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ReturnInComputedProperty(ReturnInComputedPropertyConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that a `return` statement is present in every computed property.
    ///
    /// ### Why is this bad?
    ///
    /// A Vue computed property is a getter that must produce a value. Forgetting
    /// to return turns the value into `undefined`, which silently breaks
    /// templates and reactive code that depend on the computed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   computed: {
    ///     foo() {
    ///       // missing return
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
    ///   computed: {
    ///     foo() {
    ///       return this.bar
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    ReturnInComputedProperty,
    vue,
    correctness,
    config = ReturnInComputedProperty,
    version = "1.63.0",
    short_description = "Enforce that a `return` statement is present in every computed property.",
);

impl Rule for ReturnInComputedProperty {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let span = match node.kind() {
            AstKind::Function(func) => func.span,
            AstKind::ArrowFunctionExpression(arrow) => {
                // Expression-body arrow (`() => x`) implicitly returns its expression,
                // so it always has a return on every code path.
                if arrow.expression {
                    return;
                }
                arrow.span
            }
            _ => return,
        };

        if get_computed_getter_context(node, ctx).is_none() {
            return;
        }

        if !definitely_returns_in_all_codepaths(node, ctx, self.0.treat_undefined_as_unspecified) {
            ctx.diagnostic(return_in_computed_property_diagnostic(span));
        }
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
                  computed: {
                    foo () {
                      return true
                    },
                    bar: function () {
                      return false
                    },
                    bar3: {
                      set () {
                        return true
                      },
                      get () {
                        return true
                      }
                    },
                    bar4 () {
                      if (foo) {
                        return true
                      } else {
                        return false
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
                <script>
                export default {
                  computed: {
                    foo () {
                      const options = []
                      this.matches.forEach((match) => {
                        options.push(match)
                      })
                      return options
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
                      const options = []
                      this.matches.forEach(function (match) {
                        options.push(match)
                      })
                      return options
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
                    foo: {
                      get () {
                        return
                      }
                    }
                  }
                }
                </script>
            ",
            Some(serde_json::json!([{ "treatUndefinedAsUnspecified": false }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                import {computed} from 'vue'
                export default {
                  setup() {
                    const foo = computed(() => true)
                    const bar = computed(function() {
                      return false
                    })
                    const bar3 = computed({
                      set: () => true,
                      get: () => true
                    })
                    const bar4 = computed(() => {
                      if (foo) {
                        return true
                      } else {
                        return false
                      }
                    })
                    const foo2 = computed(() => {
                      const options = []
                      this.matches.forEach((match) => {
                        options.push(match)
                      })
                      return options
                    })
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
                import {computed} from 'vue'
                export default {
                  setup() {
                    const foo = computed({
                      get: () => {
                        return
                      }
                    })
                  }
                }
                </script>
            ",
            Some(serde_json::json!([{ "treatUndefinedAsUnspecified": false }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                import { computed } from 'other-lib'
                computed(() => {})
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                const computed = (fn) => fn
                computed(() => {})
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
                    foo: function () {
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
                    foo: function () {
                      if (a) {
                        return
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
                <script>
                export default {
                  computed: {
                    foo: {
                      set () {
                      },
                      get () {
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
                <script>
                export default {
                  computed: {
                    foo: function () {
                      function bar () {
                        return this.baz * 2
                      }
                      bar()
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
                    },
                    bar () {
                      return
                    }
                  }
                }
                </script>
            ",
            Some(serde_json::json!([{ "treatUndefinedAsUnspecified": false }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  computed: {
                    foo () {
                      return
                    }
                  }
                }
                </script>
            ",
            Some(serde_json::json!([{ "treatUndefinedAsUnspecified": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                // @vue/component
                export default {
                  computed: {
                      my_FALSE_test() {
                          let aa = 2;
                          this.my_id = aa;
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
                import {computed} from 'vue'
                export default {
                  setup() {
                    const foo = computed(() => {})
                    const foo2 = computed(function() {})
                    const foo3 = computed(() => {
                      if (a) {
                        return
                      }
                    })
                    const foo4 = computed({
                      set: () => {},
                      get: () => {}
                    })
                    const foo5 = computed(() => {
                      const bar = () => {
                        return this.baz * 2
                      }
                      bar()
                    })
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
                import {computed} from 'vue'
                export default {
                  setup() {
                    const foo = computed(() => {})
                    const baz = computed(() => {
                      return
                    })
                  }
                }
                </script>
            ",
            Some(serde_json::json!([{ "treatUndefinedAsUnspecified": false }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  'computed': {
                    foo() {
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
                new Vue({
                  computed: {
                    foo() {
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

    Tester::new(ReturnInComputedProperty::NAME, ReturnInComputedProperty::PLUGIN, pass, fail)
        .test_and_snapshot();
}
