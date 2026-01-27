use cow_utils::CowUtils;
use lazy_regex::Regex;
use rustc_hash::FxHashSet;

use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, FormalParameter, Function, Statement},
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{CompactStr, GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use schemars::JsonSchema;

use crate::{
    ast_util::get_declaration_of_variable,
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, get_node_name, is_type_of_jest_fn_call,
    },
};

fn expect_expect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test has no assertions")
        .with_help("Add assertion(s) in this Test")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r#"
### What it does

This rule triggers when there is no call made to `expect` in a test, ensure that there is at least one `expect` call made in a test.

### Why is this bad?

People may forget to add assertions.

### Examples

Examples of **incorrect** code for this rule:
```javascript
it('should be a test', () => {
    console.log('no assertion');
});
test('should assert something', () => {});
```

This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/v1.1.9/docs/rules/expect-expect.md),
to use it, add the following configuration to your `.oxlintrc.json`:

```json
{
  "rules": {
     "vitest/expect-expect": "error"
  }
}
```
"#;
#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ExpectExpectConfig {
    /// A list of function names that should be treated as assertion functions.
    ///
    /// NOTE: The default value is `["expect"]` for Jest and
    /// `["expect", "expectTypeOf", "assert", "assertType"]` for Vitest.
    #[serde(rename = "assertFunctionNames")]
    assert_function_names_jest: Vec<CompactStr>,
    #[schemars(skip)] // Skipped because this field isn't exposed to the user.
    assert_function_names_vitest: Vec<CompactStr>,
    /// An array of function names that should also be treated as test blocks.
    additional_test_block_functions: Vec<CompactStr>,
}

impl Default for ExpectExpectConfig {
    fn default() -> Self {
        Self {
            assert_function_names_jest: vec!["expect".into()],
            assert_function_names_vitest: vec![
                "expect".into(),
                "expectTypeOf".into(),
                "assert".into(),
                "assertType".into(),
            ],
            additional_test_block_functions: vec![],
        }
    }
}

impl ExpectExpectConfig {
    #[expect(clippy::unnecessary_wraps)] // TODO: fail on serde_json::Error
    pub fn from_configuration(value: &serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let default_assert_function_names_jest = vec!["expect".into()];
        let default_assert_function_names_vitest =
            vec!["expect".into(), "expectTypeOf".into(), "assert".into(), "assertType".into()];
        let config = value.get(0);

        let assert_function_names = config
            .and_then(|config| config.get("assertFunctionNames"))
            .and_then(serde_json::Value::as_array)
            .map(|v| {
                v.iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(convert_pattern)
                    .collect::<Vec<_>>()
            });

        let assert_function_names_jest =
            assert_function_names.clone().unwrap_or(default_assert_function_names_jest);
        let assert_function_names_vitest =
            assert_function_names.unwrap_or(default_assert_function_names_vitest);

        let additional_test_block_functions = config
            .and_then(|config| config.get("additionalTestBlockFunctions"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Ok(Self {
            assert_function_names_jest,
            assert_function_names_vitest,
            additional_test_block_functions,
        })
    }

    pub fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(self, jest_node, ctx);
    }
}

fn run<'a>(
    rule: &ExpectExpectConfig,
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
) {
    let node = possible_jest_node.node;
    if let AstKind::CallExpression(call_expr) = node.kind() {
        let name = get_node_name(&call_expr.callee);
        if is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Test)],
        ) || rule.additional_test_block_functions.contains(&name)
        {
            if let Some(member_expr) = call_expr.callee.as_member_expression() {
                let Some(property_name) = member_expr.static_property_name() else {
                    return;
                };
                if property_name == "todo" {
                    return;
                }
                if property_name == "skip" && ctx.frameworks().is_vitest() {
                    return;
                }
            }

            let assert_function_names = if ctx.frameworks().is_vitest() {
                &rule.assert_function_names_vitest
            } else {
                &rule.assert_function_names_jest
            };

            let mut visitor = AssertionVisitor::new(ctx, assert_function_names);

            // Visit each argument of the test call
            for argument in &call_expr.arguments {
                if let Some(expr) = argument.as_expression() {
                    visitor.check_expression(expr);
                    if visitor.found_assertion {
                        return;
                    }
                }
            }

            if !visitor.found_assertion {
                ctx.diagnostic(expect_expect_diagnostic(call_expr.callee.span()));
            }
        }
    }
}

struct AssertionVisitor<'a, 'b> {
    ctx: &'b LintContext<'a>,
    assert_function_names: &'b [CompactStr],
    visited: FxHashSet<Span>,
    found_assertion: bool,
}

impl<'a, 'b> AssertionVisitor<'a, 'b> {
    fn new(ctx: &'b LintContext<'a>, assert_function_names: &'b [CompactStr]) -> Self {
        Self { ctx, assert_function_names, visited: FxHashSet::default(), found_assertion: false }
    }

    fn check_expression(&mut self, expr: &Expression<'a>) {
        // Avoid infinite loops by tracking visited expressions
        if !self.visited.insert(expr.span()) {
            return;
        }

        match expr {
            Expression::FunctionExpression(fn_expr) => {
                if let Some(body) = &fn_expr.body {
                    self.visit_function_body(body);
                }
            }
            Expression::ArrowFunctionExpression(arrow_expr) => {
                self.visit_function_body(&arrow_expr.body);
            }
            Expression::CallExpression(call_expr) => {
                self.visit_call_expression(call_expr);
            }
            Expression::Identifier(ident) => {
                self.check_identifier(ident);
            }
            Expression::AwaitExpression(expr) => {
                self.check_expression(&expr.argument);
            }
            Expression::ArrayExpression(array_expr) => {
                for element in &array_expr.elements {
                    if let Some(element_expr) = element.as_expression() {
                        self.check_expression(element_expr);
                        if self.found_assertion {
                            return;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_identifier(&mut self, ident: &oxc_ast::ast::IdentifierReference<'a>) {
        let Some(node) = get_declaration_of_variable(ident, self.ctx) else {
            return;
        };
        let AstKind::Function(function) = node.kind() else {
            return;
        };
        if let Some(body) = &function.body {
            self.visit_function_body(body);
        }
    }
}

impl<'a> Visit<'a> for AssertionVisitor<'a, '_> {
    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        let name = get_node_name(&call_expr.callee);
        if matches_assert_function_name(&name, self.assert_function_names) {
            self.found_assertion = true;
            return;
        }

        for argument in &call_expr.arguments {
            if let Some(expr) = argument.as_expression() {
                self.check_expression(expr);
                if self.found_assertion {
                    return;
                }
            }
        }

        walk::walk_call_expression(self, call_expr);
    }

    fn visit_expression_statement(&mut self, stmt: &oxc_ast::ast::ExpressionStatement<'a>) {
        self.check_expression(&stmt.expression);
        if !self.found_assertion {
            walk::walk_expression_statement(self, stmt);
        }
    }

    fn visit_block_statement(&mut self, block: &oxc_ast::ast::BlockStatement<'a>) {
        for stmt in &block.body {
            self.visit_statement(stmt);
            if self.found_assertion {
                return;
            }
        }
    }

    fn visit_if_statement(&mut self, if_stmt: &oxc_ast::ast::IfStatement<'a>) {
        if let Statement::BlockStatement(block_stmt) = &if_stmt.consequent {
            self.visit_block_statement(block_stmt);
        }
        if self.found_assertion {
            return;
        }
        if let Some(alternate) = &if_stmt.alternate {
            self.visit_statement(alternate);
        }
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_formal_parameter(&mut self, _param: &FormalParameter<'a>) {}
}

/// Checks if node names returned by getNodeName matches any of the given star patterns
fn matches_assert_function_name(name: &str, patterns: &[CompactStr]) -> bool {
    patterns.iter().any(|pattern| Regex::new(pattern).unwrap().is_match(name))
}

fn convert_pattern(pattern: &str) -> CompactStr {
    // Pre-process pattern, e.g.
    // request.*.expect -> request.[a-z\\d]*.expect
    // request.**.expect -> request.[a-z\\d\\.]*.expect
    // request.**.expect* -> request.[a-z\\d\\.]*.expect[a-z\\d]*
    let pattern = pattern
        .split('.')
        .map(|p| {
            if p == "**" {
                CompactStr::from("[a-z\\d\\.]*")
            } else {
                p.cow_replace('*', "[a-z\\d]*").into()
            }
        })
        .collect::<Vec<_>>()
        .join("\\.");

    // 'a.b.c' -> /^a\.b\.c(\.|$)/iu
    format!("(?ui)^{pattern}(\\.|$)").into()
}
