use lazy_regex::Regex;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;

use oxc_ast::{
    AstKind, match_member_expression,
    ast::{CallExpression, Expression, FormalParameter, Function},
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use oxc_syntax::scope::ScopeFlags;

use crate::{
    ast_util::get_declaration_of_variable,
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, convert_pattern, get_node_name,
        is_type_of_jest_fn_call,
    },
};

fn expect_expect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test has no assertions")
        .with_help("Add assertion(s) in this Test")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

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
";
#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ExpectExpectConfig {
    /// A list of function names that should be treated as assertion functions.
    ///
    /// NOTE: The default value is `["expect"]` for Jest and
    /// `["expect", "expectTypeOf", "assert", "assertType"]` for Vitest.
    #[serde(rename = "assertFunctionNames")]
    assert_function_names_jest: Vec<CompactStr>,
    #[serde(skip)]
    #[schemars(skip)]
    assert_function_matchers_jest: Vec<AssertFunctionMatcher>,
    #[serde(skip)]
    #[schemars(skip)]
    assert_function_matchers_vitest: Vec<AssertFunctionMatcher>,
    /// An array of function names that should also be treated as test blocks.
    additional_test_block_functions: Vec<CompactStr>,
}

impl Default for ExpectExpectConfig {
    fn default() -> Self {
        let assert_function_names_jest = default_assert_function_names_jest();
        let assert_function_names_vitest = default_assert_function_names_vitest();
        Self {
            assert_function_matchers_jest: compile_assert_function_matchers(
                &assert_function_names_jest,
            ),
            assert_function_matchers_vitest: compile_assert_function_matchers(
                &assert_function_names_vitest,
            ),
            assert_function_names_jest,
            additional_test_block_functions: vec![],
        }
    }
}

fn default_assert_function_names_jest() -> Vec<CompactStr> {
    vec!["expect".into()]
}

fn default_assert_function_names_vitest() -> Vec<CompactStr> {
    vec!["expect".into(), "expectTypeOf".into(), "assert".into(), "assertType".into()]
}

#[derive(Debug, Clone)]
enum AssertFunctionMatcher {
    Exact(CompactStr),
    Pattern(Regex),
}

impl AssertFunctionMatcher {
    fn new(name: &CompactStr) -> Self {
        if is_exact_assert_function_name(name) {
            Self::Exact(name.clone())
        } else {
            Self::Pattern(
                Regex::new(&convert_pattern(name))
                    .expect("failed to compile expect-expect assert function pattern"),
            )
        }
    }

    fn is_match(&self, name: &str) -> bool {
        match self {
            Self::Exact(expected) => is_exact_assert_function_match(name, expected),
            Self::Pattern(pattern) => pattern.is_match(name),
        }
    }

    /// Match a simple identifier callee without allocating a node-name string.
    fn is_match_ident(&self, ident: &str) -> bool {
        match self {
            Self::Exact(expected) => ident.eq_ignore_ascii_case(expected),
            // Patterns may need dotted names; fall back to full-name matching.
            Self::Pattern(_) => false,
        }
    }
}

fn is_exact_assert_function_name(name: &str) -> bool {
    name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.'))
}

fn is_exact_assert_function_match(name: &str, expected: &str) -> bool {
    if name.len() == expected.len() {
        return name.eq_ignore_ascii_case(expected);
    }

    name.as_bytes().get(expected.len()).is_some_and(|byte| *byte == b'.')
        && name.get(..expected.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(expected))
}

fn compile_assert_function_matchers(
    assert_function_names: &[CompactStr],
) -> Vec<AssertFunctionMatcher> {
    assert_function_names.iter().map(AssertFunctionMatcher::new).collect()
}

/// True if any matcher needs `get_node_name` (patterns or dotted exact names like `assert.ok`).
fn matchers_need_full_name(matchers: &[AssertFunctionMatcher]) -> bool {
    matchers.iter().any(|m| match m {
        AssertFunctionMatcher::Exact(name) => name.contains('.'),
        AssertFunctionMatcher::Pattern(_) => true,
    })
}

/// Check whether a call callee is an assertion, avoiding `get_node_name` allocation
/// for the common `expect(...)` / `assert(...)` identifier case.
fn is_assertion_call(
    call_expr: &CallExpression<'_>,
    matchers: &[AssertFunctionMatcher],
    need_full_name: bool,
) -> bool {
    let callee = call_expr.callee.get_inner_expression();

    // Fast path: bare identifier (`expect`, `assert`, …)
    if let Expression::Identifier(ident) = callee {
        let name = ident.name.as_str();
        if matchers.iter().any(|m| m.is_match_ident(name)) {
            return true;
        }
        // Exact matchers with a prefix rule: `expect` matches `expect` only for idents;
        // dotted exact names never apply to a plain identifier.
        if !need_full_name {
            return false;
        }
    }

    // Member calls like `expect.soft(...)` / `assert.ok(...)` need the dotted chain.
    // Also used when custom patterns are configured.
    let needs_chain = need_full_name
        || matches!(callee, match_member_expression!(Expression))
        || matches!(callee, Expression::CallExpression(_));
    if needs_chain {
        let name = get_node_name(&call_expr.callee);
        return matchers.iter().any(|m| m.is_match(&name));
    }

    false
}

impl ExpectExpectConfig {
    #[expect(clippy::unnecessary_wraps)]
    pub fn from_configuration(value: &serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let default_assert_function_names_jest = default_assert_function_names_jest();
        let default_assert_function_names_vitest = default_assert_function_names_vitest();
        let config = value.get(0);

        let assert_function_names = config
            .and_then(|config| config.get("assertFunctionNames"))
            .and_then(serde_json::Value::as_array)
            .map(|v| {
                v.iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(CompactStr::from)
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

        Ok(ExpectExpectConfig {
            assert_function_matchers_jest: compile_assert_function_matchers(
                &assert_function_names_jest,
            ),
            assert_function_matchers_vitest: compile_assert_function_matchers(
                &assert_function_names_vitest,
            ),
            assert_function_names_jest,
            additional_test_block_functions,
        })
    }

    pub fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(self, jest_node, ctx, ctx.frameworks().is_vitest());
    }

    pub fn run_on_vitest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(self, jest_node, ctx, true);
    }
}

fn run<'a>(
    rule: &ExpectExpectConfig,
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
    use_vitest_assertions: bool,
) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };

    // Cheap bail-outs before jest-fn classification when possible.
    if let Some(member_expr) = call_expr.callee.as_member_expression() {
        let Some(property_name) = member_expr.static_property_name() else {
            return;
        };
        // `test.todo` / `it.todo` never need assertions; `test.skip` under Vitest is ignored.
        if property_name == "todo" {
            return;
        }
        if property_name == "skip" && use_vitest_assertions {
            return;
        }
    }

    let is_test_call = is_type_of_jest_fn_call(
        call_expr,
        possible_jest_node,
        ctx,
        &[JestFnKind::General(JestGeneralFnKind::Test)],
    );

    if !is_test_call {
        if rule.additional_test_block_functions.is_empty() {
            return;
        }
        let name = get_node_name(&call_expr.callee);
        if !rule.additional_test_block_functions.iter().any(|n| n == &name) {
            return;
        }
    }

    let assert_function_matchers = if use_vitest_assertions {
        &rule.assert_function_matchers_vitest
    } else {
        &rule.assert_function_matchers_jest
    };
    let need_full_name = matchers_need_full_name(assert_function_matchers);

    let mut visitor = AssertionVisitor::new(ctx, assert_function_matchers, need_full_name);

    // Visit each argument of the test call (title + callback, etc.)
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

struct AssertionVisitor<'a, 'b> {
    ctx: &'b LintContext<'a>,
    assert_function_matchers: &'b [AssertFunctionMatcher],
    need_full_name: bool,
    /// Only allocated when resolving identifiers to function declarations (cycle guard).
    visited_decls: Option<FxHashSet<Span>>,
    found_assertion: bool,
}

impl<'a, 'b> AssertionVisitor<'a, 'b> {
    fn new(
        ctx: &'b LintContext<'a>,
        assert_function_matchers: &'b [AssertFunctionMatcher],
        need_full_name: bool,
    ) -> Self {
        Self {
            ctx,
            assert_function_matchers,
            need_full_name,
            visited_decls: None,
            found_assertion: false,
        }
    }

    fn check_expression(&mut self, expr: &Expression<'a>) {
        match expr.get_inner_expression() {
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
            Expression::AwaitExpression(await_expr) => {
                self.check_expression(&await_expr.argument);
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
        // Cycle guard only when we chase declarations (uncommon vs walking the test body).
        let span = function.span;
        let visited = self.visited_decls.get_or_insert_with(FxHashSet::default);
        if !visited.insert(span) {
            return;
        }
        if let Some(body) = &function.body {
            self.visit_function_body(body);
        }
    }
}

impl<'a> Visit<'a> for AssertionVisitor<'a, '_> {
    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        if is_assertion_call(call_expr, self.assert_function_matchers, self.need_full_name) {
            self.found_assertion = true;
            return;
        }

        // Walk callee + args once (no separate argument pre-pass — avoids double traversal).
        walk::walk_call_expression(self, call_expr);
    }

    fn visit_expression(&mut self, expr: &Expression<'a>) {
        // Prefer our specialized checks for constructs that contain assertions or callbacks.
        match expr.get_inner_expression() {
            Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::CallExpression(_)
            | Expression::Identifier(_)
            | Expression::AwaitExpression(_)
            | Expression::ArrayExpression(_) => {
                self.check_expression(expr);
            }
            _ => {
                if !self.found_assertion {
                    walk::walk_expression(self, expr);
                }
            }
        }
    }

    fn visit_expression_statement(&mut self, stmt: &oxc_ast::ast::ExpressionStatement<'a>) {
        self.check_expression(&stmt.expression);
        // `check_expression` already covers functions/calls/idents; only walk other kinds.
        if !self.found_assertion
            && !matches!(
                stmt.expression.get_inner_expression(),
                Expression::FunctionExpression(_)
                    | Expression::ArrowFunctionExpression(_)
                    | Expression::CallExpression(_)
                    | Expression::Identifier(_)
                    | Expression::AwaitExpression(_)
                    | Expression::ArrayExpression(_)
            )
        {
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
        self.visit_statement(&if_stmt.consequent);
        if self.found_assertion {
            return;
        }
        if let Some(alternate) = &if_stmt.alternate {
            self.visit_statement(alternate);
        }
    }

    // Nested function *declarations* are ignored unless referenced (via `check_identifier`).
    // Callback *expressions* are handled in `check_expression` / `visit_expression`.
    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_formal_parameter(&mut self, _param: &FormalParameter<'a>) {}
}
