use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{definitely_returns_in_all_codepaths, is_vue_component_options_object},
};

fn require_render_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected to return a value in render function.")
        .with_help("All code paths inside a render function must return a value.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireRenderReturn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that a `render` function always returns a value.
    ///
    /// ### Why is this bad?
    ///
    /// A Vue component's `render` function must produce a VNode tree. If a
    /// code path falls through without returning, Vue receives `undefined`
    /// and silently renders nothing.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   render() {
    ///     if (foo) {
    ///       return h('div')
    ///     }
    ///     // falls through without returning
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   render() {
    ///     return h('div')
    ///   }
    /// }
    /// </script>
    /// ```
    RequireRenderReturn,
    vue,
    correctness,
    version = "next",
);

impl Rule for RequireRenderReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(_) => {}
            AstKind::ArrowFunctionExpression(arrow) => {
                // Expression-body arrow (`render: () => x`) implicitly returns.
                if arrow.expression {
                    return;
                }
            }
            _ => return,
        }

        let Some(render_prop_key_span) = render_property_key_span(node, ctx) else {
            return;
        };

        if !definitely_returns_in_all_codepaths(node, ctx, true) {
            ctx.diagnostic(require_render_return_diagnostic(render_prop_key_span));
        }
    }
}

/// Return the `render` key span if `node` is the function value of a `render:`
/// property on a Vue component options object.
fn render_property_key_span(node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<Span> {
    let nodes = ctx.nodes();
    let parent = nodes.parent_node(node.id());
    let AstKind::ObjectProperty(prop) = parent.kind() else {
        return None;
    };
    if !prop.key.is_specific_static_name("render") {
        return None;
    }

    let opts_node = nodes.parent_node(parent.id());
    if !matches!(opts_node.kind(), AstKind::ObjectExpression(_)) {
        return None;
    }
    if !is_vue_component_options_object(opts_node, ctx) {
        return None;
    }

    Some(prop.key.span())
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "Vue.component('test', {
                    ...foo,
                    render() {
                      return {}
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "Vue.component('test', {
                    foo() {
                      return {}
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "Vue.component('test', {
                    foo: {}
                  })",
            None,
            None,
            None,
        ),
        (
            "Vue.component('test', {
                    render: foo
                  })",
            None,
            None,
            None,
        ),
        (
            "Vue.component('test', {
                    render() {
                      return <div></div>
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "<script>
                  export default {
                    render() {
                      return {}
                    }
                  }
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  export default {
                    render() {
                      const foo = function () {}
                      return foo
                    }
                  }
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  export default {
                    render() {
                      if (a) {
                        if (b) {

                        }
                        if (c) {
                          return true
                        } else {
                          return foo
                        }
                      } else {
                        return foo
                      }
                    }
                  }
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  export default {
                    render: () => null
                  }
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  export default {
                    render() {
                      if (a) {
                        return `<div>a</div>`
                      } else {
                        return `<span>a</span>`
                      }
                    }
                  }
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  export default {
                    render(h) {
                      const options = []
                      this.matches.forEach(function (match) {
                        options.push(match)
                      })
                      return h('div', options)
                    }
                  }
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "<script>
                  export default {
                    render() {
                    }
                  }
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  export default {
                    render: function () {
                      if (foo) {
                        return h('div', 'hello')
                      }
                    }
                  }
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "Vue.component('test', {
                    render: function () {
                      if (a) {
                        return
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "app.component('test', {
                    render: function () {
                      if (a) {
                        return
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "Vue.component('test2', {
                    render: function () {
                      if (a) {
                        return h('div', 'hello')
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
        (
            "Vue.component('test2', {
                    render: function () {
                      if (a) {

                      } else {
                        return h('div', 'hello')
                      }
                    }
                  })",
            None,
            None,
            None,
        ),
    ];

    Tester::new(RequireRenderReturn::NAME, RequireRenderReturn::PLUGIN, pass, fail)
        .test_and_snapshot();
}
