use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, MemberExpression},
};
use oxc_cfg::{
    EdgeType, ErrorEdgeKind, InstructionKind, ReturnInstructionKind,
    graph::{
        Direction,
        visit::{Control, DfsEvent, set_depth_first_search},
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    module_record::ImportImportName,
    rule::{DefaultRuleConfig, Rule},
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

        if !is_vue_computed_getter(node, ctx) {
            return;
        }

        if !definitely_returns_in_all_codepaths(node, ctx, self.0.treat_undefined_as_unspecified) {
            ctx.diagnostic(return_in_computed_property_diagnostic(span));
        }
    }
}

fn is_vue_computed_getter(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let nodes = ctx.nodes();
    let parent = nodes.parent_node(node.id());

    let AstKind::ObjectProperty(prop) = parent.kind() else {
        // `computed(() => {...})` / `computed(function() {...})` —
        // the function is the direct argument of a `computed(...)` call.
        return matches!(
            parent.kind(),
            AstKind::CallExpression(call) if is_vue_computed_call(call, ctx)
        );
    };

    // `set` accessors are setters — they don't need to return.
    if prop.key.is_specific_static_name("set") {
        return false;
    }

    let opts_node = nodes.parent_node(parent.id());
    if !matches!(opts_node.kind(), AstKind::ObjectExpression(_)) {
        return false;
    }
    let outer = nodes.parent_node(opts_node.id());

    // Case A: `computed: { foo() {...} }` or `computed: { foo: function() {...} }`
    //   function -> ObjectProperty(foo) -> ObjectExpression(computed body)
    //            -> ObjectProperty(computed) [outer]
    if let AstKind::ObjectProperty(outer_prop) = outer.kind()
        && outer_prop.key.is_specific_static_name("computed")
    {
        return is_under_vue_root(outer, ctx);
    }

    // The remaining cases assume `prop` is the `get` accessor of a get/set object.
    if !prop.key.is_specific_static_name("get") {
        return false;
    }

    // Case B: `computed: { foo: { get() {...} } }`
    //   get() -> ObjectProperty(get) -> ObjectExpression(get/set obj)
    //         -> ObjectProperty(foo) [outer]
    //         -> ObjectExpression(computed body)
    //         -> ObjectProperty(computed)
    if let AstKind::ObjectProperty(_) = outer.kind() {
        let computed_body = nodes.parent_node(outer.id());
        if !matches!(computed_body.kind(), AstKind::ObjectExpression(_)) {
            return false;
        }
        let computed_prop = nodes.parent_node(computed_body.id());
        if let AstKind::ObjectProperty(cp) = computed_prop.kind()
            && cp.key.is_specific_static_name("computed")
        {
            return is_under_vue_root(computed_prop, ctx);
        }
        return false;
    }

    // Case C: `computed({ get() {...}, set() {} })`
    //   get() -> ObjectProperty(get) -> ObjectExpression(get/set obj)
    //         -> CallExpression(computed) [outer]
    if let AstKind::CallExpression(call) = outer.kind() {
        return is_vue_computed_call(call, ctx);
    }

    false
}

fn is_under_vue_root(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    ctx.nodes().ancestors(node.id()).any(|a| is_vue_component_root(a.kind()))
}

fn is_vue_computed_call(call: &CallExpression<'_>, ctx: &LintContext<'_>) -> bool {
    let Expression::Identifier(ident) = call.callee.get_inner_expression() else {
        return false;
    };
    if ident.name != "computed" {
        return false;
    }

    let scoping = ctx.scoping();
    let Some(symbol_id) = scoping.get_reference(ident.reference_id()).symbol_id() else {
        return false;
    };

    ctx.module_record().import_entries.iter().any(|entry| {
        if entry.module_request.name() != "vue" {
            return false;
        }
        let imported_name = match &entry.import_name {
            ImportImportName::Name(name_span) => name_span.name(),
            _ => return false,
        };
        if imported_name != "computed" {
            return false;
        }
        scoping.get_root_binding(entry.local_name.name().into()) == Some(symbol_id)
    })
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

fn definitely_returns_in_all_codepaths(
    node: &AstNode<'_>,
    ctx: &LintContext<'_>,
    treat_undefined_as_unspecified: bool,
) -> bool {
    let cfg = ctx.cfg();
    let graph = cfg.graph();

    let output =
        set_depth_first_search(graph, Some(ctx.nodes().cfg_id(node.id())), |event| match event {
            DfsEvent::TreeEdge(a, b) => {
                if graph.edges_connecting(a, b).any(|e| {
                    matches!(
                        e.weight(),
                        EdgeType::Normal
                            | EdgeType::Jump
                            | EdgeType::Error(ErrorEdgeKind::Explicit)
                    )
                }) {
                    Control::Continue
                } else {
                    Control::Prune
                }
            }
            DfsEvent::Discover(basic_block_id, _) => {
                let return_instruction =
                    cfg.basic_block(basic_block_id).instructions().iter().find(|it| {
                        match it.kind {
                            InstructionKind::Return(_) | InstructionKind::Throw => true,
                            InstructionKind::ImplicitReturn
                            | InstructionKind::Break(_)
                            | InstructionKind::Continue(_)
                            | InstructionKind::Iteration(_)
                            | InstructionKind::Unreachable
                            | InstructionKind::Condition
                            | InstructionKind::Statement => false,
                        }
                    });

                let does_return = return_instruction.is_some_and(|ret| {
                    !matches!(
                        ret.kind,
                        InstructionKind::Return(ReturnInstructionKind::ImplicitUndefined)
                            if treat_undefined_as_unspecified
                    )
                });

                if graph.edges_directed(basic_block_id, Direction::Outgoing).any(|e| {
                    matches!(
                        e.weight(),
                        EdgeType::Jump
                            | EdgeType::Normal
                            | EdgeType::Backedge
                            | EdgeType::Error(ErrorEdgeKind::Explicit)
                    )
                }) {
                    Control::Continue
                } else if does_return {
                    Control::Prune
                } else {
                    Control::Break(())
                }
            }
            _ => Control::Continue,
        });

    output.break_value().is_none()
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
    ];

    Tester::new(ReturnInComputedProperty::NAME, ReturnInComputedProperty::PLUGIN, pass, fail)
        .test_and_snapshot();
}
