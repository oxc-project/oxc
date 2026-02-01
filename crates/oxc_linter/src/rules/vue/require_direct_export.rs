use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, CallExpression, ExportDefaultDeclarationKind, Expression,
        Function, FunctionBody, ReturnStatement,
    },
    match_expression,
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::is_method_call,
    context::{ContextHost, LintContext},
    frameworks::FrameworkOptions,
    rule::{DefaultRuleConfig, Rule},
};

fn require_direct_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected the component literal to be directly exported.")
        .with_help(
            "Export the component object directly instead of assigning it to a variable first.",
        )
        .with_label(span)
}

fn disallow_functional_component_function_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Functional component functions are not allowed.")
        .with_help(
            "Export a component object directly instead of using a functional component function.",
        )
        .with_label(span)
}

fn missing_function_return_value_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Function component must return a value.").with_label(span)
}

#[derive(Debug, Clone, Default, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RequireDirectExport {
    /// When set `true`, disallow functional component functions.
    disallow_functional_component_function: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires that the component object be directly exported.
    ///
    /// ### Why is this bad?
    ///
    /// Indirect exports can make it harder to understand the component structure
    /// and may cause issues with Vue's component system.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// const A = {};
    /// export default A;
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script>
    /// export default function () {}
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {};
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script>
    /// export default function (props) {
    ///   return h('div', props.msg);
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `disallowFunctionalComponentFunction: true` option:
    /// ```vue
    /// <script>
    /// export default (props) => h('div', props.msg)
    /// </script>
    /// ```
    RequireDirectExport,
    vue,
    style,
    config = RequireDirectExport,
);

impl Rule for RequireDirectExport {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExportDefaultDeclaration(export_decl) = node.kind() else {
            return;
        };

        match &export_decl.declaration {
            // e.g. export default function () {}
            ExportDefaultDeclarationKind::FunctionDeclaration(func_decl) => {
                if self.disallow_functional_component_function {
                    ctx.diagnostic(disallow_functional_component_function_diagnostic(
                        export_decl.span,
                    ));
                } else if !has_function_return_value(func_decl) {
                    ctx.diagnostic(missing_function_return_value_diagnostic(export_decl.span));
                }
            }
            match_expression!(ExportDefaultDeclarationKind) => {
                let expr = export_decl.declaration.to_expression();
                check_expression(expr, export_decl.span, ctx, self);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
            && ctx.frameworks_options() != FrameworkOptions::VueSetup
    }
}

fn check_expression<'a>(
    expr: &Expression<'a>,
    span: Span,
    ctx: &LintContext<'a>,
    rule: &RequireDirectExport,
) {
    let inner_expr = expr.get_inner_expression();

    match inner_expr {
        // e.g. export default Foo;
        Expression::Identifier(_) => {
            ctx.diagnostic(require_direct_export_diagnostic(span));
        }
        // e.g. export default (props) => { return h('div', props.msg) }
        Expression::ArrowFunctionExpression(arrow_func) => {
            if rule.disallow_functional_component_function {
                ctx.diagnostic(disallow_functional_component_function_diagnostic(span));
            } else if !has_arrow_function_return_value(arrow_func) {
                ctx.diagnostic(missing_function_return_value_diagnostic(span));
            }
        }
        Expression::FunctionExpression(func) => {
            if rule.disallow_functional_component_function {
                ctx.diagnostic(disallow_functional_component_function_diagnostic(span));
            } else if !has_function_return_value(func) {
                ctx.diagnostic(missing_function_return_value_diagnostic(span));
            }
        }
        // Check for CallExpression (Vue.extend, defineComponent)
        Expression::CallExpression(call_expr) => {
            check_call_expression(call_expr, span, ctx);
        }
        _ => {}
    }
}

fn check_call_expression<'a>(call_expr: &CallExpression<'a>, span: Span, ctx: &LintContext<'a>) {
    // Check for defineComponent
    if let Some(callee_ident) = call_expr.callee.get_identifier_reference()
        && callee_ident.name.as_str() == "defineComponent"
        && !has_object_expression_argument(call_expr)
    {
        ctx.diagnostic(require_direct_export_diagnostic(span));
        return;
    }

    if is_method_call(call_expr, Some(&["Vue"]), Some(&["extend"]), None, None)
        && !has_object_expression_argument(call_expr)
    {
        ctx.diagnostic(require_direct_export_diagnostic(span));
    }
}

fn has_object_expression_argument(call_expr: &CallExpression) -> bool {
    call_expr
        .arguments
        .first()
        .and_then(|arg| arg.as_expression())
        .is_some_and(|expr| matches!(expr.get_inner_expression(), Expression::ObjectExpression(_)))
}

fn has_function_return_value(func: &Function) -> bool {
    func.body.as_deref().is_some_and(find_return_value)
}

fn has_arrow_function_return_value(arrow_func: &ArrowFunctionExpression) -> bool {
    // If expression is true, it's an expression body (() => expr), which always returns a value
    if arrow_func.expression {
        return true;
    }
    find_return_value(&arrow_func.body)
}

fn find_return_value(body: &FunctionBody) -> bool {
    let mut finder = ReturnFinder::new();
    finder.visit_function_body(body);
    finder.found
}

struct ReturnFinder {
    found: bool,
}

impl ReturnFinder {
    fn new() -> Self {
        Self { found: false }
    }
}

impl<'a> Visit<'a> for ReturnFinder {
    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        if self.found {
            return;
        }
        if it.argument.is_some() {
            self.found = true;
        }
    }

    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {}
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("", None, None, Some(PathBuf::from("test.vue"))),
        (
            "
            <script>
export default function (props) {
  if (props.show) {
    return h('div', props.msg)
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
export default (props) => {
  if (props.show) {
    return h('div', props.msg)
  }
}
</script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        ("export default {}", None, None, Some(PathBuf::from("test.vue"))),
        (
            "export default {}",
            Some(serde_json::json!([{ "disallowFunctionalComponentFunction": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        ("export default Foo", None, None, Some(PathBuf::from("test.js"))),
        (
            "
                  import { h } from 'vue'
                  export default function (props) {
                    return h('div', `Hello! ${props.name}`)
                  }
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  import { h } from 'vue'
                  export default function Component () {
                    return h('div')
                  }
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  import { h } from 'vue'
                  export default (props) => {
                    return h('div', `Hello! ${props.name}`)
                  }
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  import { h } from 'vue'
                  export default (props => h('div', props.msg))
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        ("export default (() => 2)", None, None, Some(PathBuf::from("test.vue"))),
        (
            "
                  import Vue from 'vue'
                  export default (Vue.extend({}))
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  import { defineComponent } from 'vue'
                  export default defineComponent({})
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
                  const A = {};
                  export default A
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  const A = {};
                  export default (A)
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  function A(props) {
                    return h('div', props.msg)
                  };
                  export default A
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default function NoReturn() {} </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default function () {} </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        ("<script>export default () => {} </script>", None, None, Some(PathBuf::from("test.vue"))),
        (
            "<script>export default () => {
                    const foo = () => {
                      return b
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                export default () => {
                    return
                }
            </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  function A(props) {
                    return h('div', props.msg)
                  };
                  export default A
            </script>",
            Some(serde_json::json!([{ "disallowFunctionalComponentFunction": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import { h } from 'vue'
                  export default function (props) {
                    return h('div', `Hello! ${props.name}`)
                  }
                  </script>",
            Some(serde_json::json!([{ "disallowFunctionalComponentFunction": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  export default (props) => {
                    if (props.show) {
                        return;
                    }
                    return;
                  }
                  </script>",
            Some(serde_json::json!([{ "disallowFunctionalComponentFunction": false }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  export default function(props) {
                    if (props.show) {
                        return;
                    }
                    return;
                  }
                  </script>",
            Some(serde_json::json!([{ "disallowFunctionalComponentFunction": false }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import { h } from 'vue'
                  export default (props) => {
                    if (props.show) {
                        return h('div', `Hello! ${props.name}`)
                    }
                  }
                  </script>",
            Some(serde_json::json!([{ "disallowFunctionalComponentFunction": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  import { h } from 'vue'
                  export default function Component () {
                    return h('div')
                  }
                  </script>",
            Some(serde_json::json!([{ "disallowFunctionalComponentFunction": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  import { h } from 'vue'
                  export default (props) => {
                    return h('div', `Hello! ${props.name}`)
                  }
                  </script>",
            Some(serde_json::json!([{ "disallowFunctionalComponentFunction": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  import { h } from 'vue'
                  export default props => h('div', props.msg)
                  </script>",
            Some(serde_json::json!([{ "disallowFunctionalComponentFunction": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  import Vue from 'vue'
                  export default Vue.extend()
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  import Vue from 'vue'
                  const A = {}
                  export default Vue.extend(A)
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  import Vue from 'vue'
                  export default Vue.extend(2)
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
             import { defineComponent } from 'vue'
             export default defineComponent()
             </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  import { defineComponent } from 'vue'
                  export default defineComponent(2)
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  import { defineComponent } from 'vue'
                  export default (defineComponent())
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  import { defineComponent } from 'vue'
                  const A = {}
                  export default defineComponent(A)
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  import { defineComponent } from 'vue'
                  const A = {}
                  export default (defineComponent(A))
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(RequireDirectExport::NAME, RequireDirectExport::PLUGIN, pass, fail)
        .test_and_snapshot();
}
