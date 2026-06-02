use std::borrow::Cow;

use crate::{
    FixKind,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, get_node_name, parse_general_jest_fn_call,
    },
};
use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, FunctionBody, Statement},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PreferExpectAssertionsConfig {
    pub only_functions_with_async_keyword: bool,
    pub only_functions_with_expect_in_callback: bool,
    pub only_functions_with_expect_in_loop: bool,
}

impl PreferExpectAssertionsConfig {
    #[inline]
    pub fn compute_config(&self) -> bool {
        self.only_functions_with_async_keyword
            || self.only_functions_with_expect_in_callback
            || self.only_functions_with_expect_in_loop
    }
}

pub const DOCUMENTATION: &str = r#"### What it does

Enforces that every test has either `expect.assertions(<number>)` or
`expect.hasAssertions()` as its first expression.

### Why is this bad?

Without explicit assertion counts, tests with asynchronous code,
callbacks, or loops may pass even if some `expect` calls are never
reached, silently hiding bugs.

### Examples

Examples of **incorrect** code for this rule:
```javascript
test('no assertions', () => {
  // ...
});
test('assertions not first', () => {
  expect(true).toBe(true);
  // ...
});
```

Examples of **correct** code for this rule:
```javascript
test('with assertion count', () => {
  expect.assertions(1);
  expect(true).toBe(true);
});
test('with hasAssertions', () => {
  expect.hasAssertions();
  expect(true).toBe(true);
});
```

///Examples of **incorrect** code with `{ "onlyFunctionsWithAsyncKeyword": true }`:
```javascript
test('fetches data', async () => {
  const data = await fetchData();
  expect(data).toBe('peanut butter');
});
```

Examples of **correct** code with `{ "onlyFunctionsWithAsyncKeyword": true }`:
```javascript
test('fetches data', async () => {
  expect.assertions(1);
  const data = await fetchData();
  expect(data).toBe('peanut butter');
});
```

Examples of **incorrect** code with `{ "onlyFunctionsWithExpectInLoop": true }`:
```javascript
test('all numbers are greater than zero', () => {
  for (const number of getNumbers()) {
    expect(number).toBeGreaterThan(0);
  }
});
```

Examples of **correct** code with `{ "onlyFunctionsWithExpectInLoop": true }`:
```javascript
test('all numbers are greater than zero', () => {
  expect.hasAssertions();
  for (const number of getNumbers()) {
    expect(number).toBeGreaterThan(0);
  }
});
```

Examples of **incorrect** code with `{ "onlyFunctionsWithExpectInCallback": true }`:
```javascript
test('callback test', () => {
  fetchData((data) => {
    expect(data).toBe('peanut butter');
  });
});
```

Examples of **correct** code with `{ "onlyFunctionsWithExpectInCallback": true }`:
```javascript
test('callback test', () => {
  expect.assertions(1);
  fetchData((data) => {
    expect(data).toBe('peanut butter');
  });
});
```
"#;

fn expect_shadowed_by_parameter(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "`expect` is shadowed by a callback parameter and cannot be used for assertions.",
    )
    .with_help("Rename the parameter to avoid shadowing the global `expect`.")
    .with_label(span)
}

pub fn has_assertions_takes_no_arguments(span: Span, prefix: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{prefix}.hasAssertions` expects no arguments."))
        .with_help(format!("Remove the arguments from `{prefix}.hasAssertions()`."))
        .with_label(span)
}

fn assertions_requires_one_argument(span: Span, prefix: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{prefix}.assertions` expects a single argument of type number."))
        .with_help(format!("Pass a single numeric argument to `{prefix}.assertions()`."))
        .with_label(span)
}

fn assertions_requires_number_argument(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This argument should be a number.")
        .with_help("Replace this argument with a numeric literal.")
        .with_label(span)
}

pub fn resolve_expect_local_name(ctx: &LintContext<'_>, sources: &[&str]) -> CompactStr {
    for entry in &ctx.module_record().import_entries {
        if entry.is_type {
            continue;
        }

        let source = entry.module_request.name();
        if !sources.contains(&source) {
            continue;
        }

        let crate::module_record::ImportImportName::Name(import_name) = &entry.import_name else {
            continue;
        };
        if import_name.name() == "expect" {
            return CompactStr::from(entry.local_name.name());
        }
    }
    CompactStr::from("expect")
}

pub trait PreferExpectAssertionsRuleImpl {
    fn check_node<'a>(
        &self,
        jest_node: &PossibleJestNode<'a, '_>,
        file_expect_prefix: &str,
        covered_describe_ids: &mut Vec<NodeId>,
        ctx: &LintContext<'a>,
    ) {
        let node = jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(general) = parse_general_jest_fn_call(call_expr, jest_node, ctx) else {
            return;
        };

        let Some(kind) = general.kind.to_general() else {
            return;
        };

        match kind {
            JestGeneralFnKind::Hook if general.name.ends_with("Each") => {
                Self::check_each_hook(
                    call_expr,
                    node.id(),
                    file_expect_prefix,
                    covered_describe_ids,
                    ctx,
                );
            }
            JestGeneralFnKind::Test => {
                self.check_test(
                    call_expr,
                    node.id(),
                    file_expect_prefix,
                    covered_describe_ids,
                    ctx,
                );
            }
            _ => {}
        }
    }

    fn check_each_hook(
        call_expr: &CallExpression<'_>,
        hook_node_id: NodeId,
        file_expect_prefix: &str,
        covered_describe_ids: &mut Vec<NodeId>,
        ctx: &LintContext<'_>,
    ) {
        let Some(body) = find_test_callback(call_expr).and_then(callback_body) else {
            return;
        };

        let mut scanner = HookScanner::new(file_expect_prefix);
        scanner.visit_function_body(body);

        if !scanner.has_expect_has_assertions {
            return;
        }

        if let Some(args_span) = scanner.has_assertions_invalid_args_span {
            let call_span = scanner.has_assertions_call_span.unwrap();
            let delete_span = Span::new(args_span.start, call_span.end - 1);
            let fixer = RuleFixer::new(FixKind::Suggestion, ctx);
            let suggestion = fixer.delete_range(delete_span).with_message("Remove extra arguments");
            ctx.diagnostic_with_suggestions(
                has_assertions_takes_no_arguments(args_span, file_expect_prefix),
                [suggestion],
            );
        }

        // Find the nearest ancestor describe that contains this hook.
        // If no describe parent exists, use ROOT to indicate top-level coverage.
        let parent_describe_id = ctx
            .nodes()
            .ancestors(hook_node_id)
            .find(|n| matches!(n.kind(), AstKind::CallExpression(c) if is_describe_call(c)))
            .map_or(NodeId::ROOT, oxc_semantic::AstNode::id);

        if !covered_describe_ids.contains(&parent_describe_id) {
            covered_describe_ids.push(parent_describe_id);
        }
    }

    fn check_test<'a>(
        &self,
        call_expr: &'a CallExpression<'a>,
        test_node_id: NodeId,
        file_expect_prefix: &str,
        covered_describe_ids: &[NodeId],
        ctx: &LintContext<'a>,
    ) {
        if call_expr.arguments.len() < 2 {
            return;
        }

        let Some(callback) = find_test_callback(call_expr) else {
            return;
        };

        let Some(body) = callback_body(callback) else {
            return;
        };

        if is_covered_by_hook(test_node_id, covered_describe_ids, ctx) {
            return;
        }

        let Some(expected_resolved) = self.resolve_expect(callback, file_expect_prefix, ctx) else {
            ctx.diagnostic(expect_shadowed_by_parameter(call_expr.callee.span()));
            return;
        };

        let prefix = expected_resolved.as_ref();

        if self.has_options() && !self.should_check_node(body, is_async_callback(callback), prefix)
        {
            return;
        }

        if Self::check_first_statement(body, prefix, ctx) {
            return;
        }
        let insert_pos = Span::new(body.span.start + 1, body.span.start + 1);
        let fixer = RuleFixer::new(FixKind::Suggestion, ctx);
        let suggestions = [
            fixer
                .insert_text_before_range(insert_pos, format!("{prefix}.hasAssertions();"))
                .with_message(format!("Add `{prefix}.hasAssertions()`")),
            fixer
                .insert_text_before_range(insert_pos, format!("{prefix}.assertions();"))
                .with_message(format!("Add `{prefix}.assertions(<number of assertions>)`")),
        ];

        self.report_have_expect_assertions(call_expr.span, prefix, suggestions, ctx);
    }

    fn check_first_statement(body: &FunctionBody<'_>, prefix: &str, ctx: &LintContext<'_>) -> bool {
        let Some(Statement::ExpressionStatement(first_expr_stmt)) = body.statements.first() else {
            return false;
        };

        let Expression::CallExpression(first_call) = &first_expr_stmt.expression else {
            return false;
        };

        let name = get_node_name(&first_call.callee);

        if name.ends_with("hasAssertions") {
            validate_has_assertions_args(first_call, prefix, ctx);
            true
        } else if name.ends_with("assertions") {
            validate_assertions_args(first_call, prefix, ctx);
            true
        } else {
            false
        }
    }

    fn has_options(&self) -> bool;
    fn resolve_expect<'a, 'r>(
        &self,
        callback: &Expression<'a>,
        file_expect_prefix: &'r str,
        ctx: &LintContext<'a>,
    ) -> Option<Cow<'r, str>>;
    fn report_have_expect_assertions(
        &self,
        span: Span,
        prefix: &str,
        suggestions: [RuleFix; 2],
        ctx: &LintContext<'_>,
    );

    fn should_check_node(&self, body: &FunctionBody<'_>, is_async: bool, prefix: &str) -> bool;
}

fn is_covered_by_hook(
    test_node_id: NodeId,
    covered_describe_ids: &[NodeId],
    ctx: &LintContext<'_>,
) -> bool {
    if covered_describe_ids.is_empty() {
        return false;
    }
    if covered_describe_ids.contains(&NodeId::ROOT) {
        return true;
    }
    ctx.nodes().ancestors(test_node_id).any(|ancestor| {
        matches!(ancestor.kind(), AstKind::CallExpression(c) if is_describe_call(c))
            && covered_describe_ids.contains(&ancestor.id())
    })
}

pub fn should_check(
    config: &PreferExpectAssertionsConfig,
    body: &FunctionBody<'_>,
    is_async: bool,
    prefix: &str,
) -> bool {
    if config.only_functions_with_async_keyword && is_async {
        return true;
    }

    if !config.only_functions_with_expect_in_callback && !config.only_functions_with_expect_in_loop
    {
        return false;
    }

    let mut scanner = BodyScanner::new(prefix);
    scanner.visit_function_body(body);

    let has_callback =
        config.only_functions_with_expect_in_callback && scanner.has_expect_in_callback;
    let has_loop = config.only_functions_with_expect_in_loop && scanner.has_expect_in_loop;

    has_callback || has_loop
}

struct HookScanner {
    /// The expected callee name, e.g. `"expect.hasAssertions"` or `"e.hasAssertions"`.
    expected_name: CompactStr,
    has_expect_has_assertions: bool,
    has_assertions_invalid_args_span: Option<Span>,
    has_assertions_call_span: Option<Span>,
}

impl HookScanner {
    pub fn new(prefix: &str) -> Self {
        Self {
            expected_name: CompactStr::from(format!("{prefix}.hasAssertions")),
            has_expect_has_assertions: false,
            has_assertions_invalid_args_span: None,
            has_assertions_call_span: None,
        }
    }
}

impl<'a> Visit<'a> for HookScanner {
    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        if get_node_name(&call_expr.callee) == self.expected_name.as_str() {
            self.has_expect_has_assertions = true;
            if !call_expr.arguments.is_empty() {
                self.has_assertions_invalid_args_span = call_expr.arguments_span();
                self.has_assertions_call_span = Some(call_expr.span);
            }
        }
        oxc_ast_visit::walk::walk_call_expression(self, call_expr);
    }
}

struct BodyScanner {
    /// The expect prefix to match (e.g. `"expect"`, `"e"`, `"ctx.expect"`).
    prefix: CompactStr,
    /// Precomputed `"prefix."` for starts_with checks, avoiding allocation per call.
    prefix_dot: CompactStr,
    expression_depth: i32,
    in_loop: bool,
    has_expect_in_callback: bool,
    has_expect_in_loop: bool,
}

impl BodyScanner {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: CompactStr::from(prefix),
            prefix_dot: CompactStr::from(format!("{prefix}.")),
            expression_depth: -1,
            in_loop: false,
            has_expect_in_callback: false,
            has_expect_in_loop: false,
        }
    }

    fn visit_loop(&mut self, walk: impl FnOnce(&mut Self)) {
        let was = self.in_loop;
        self.in_loop = true;
        walk(self);
        self.in_loop = was;
    }

    fn is_expect_call(&self, call_expr: &CallExpression<'_>) -> bool {
        let name = get_node_name(&call_expr.callee);
        name == self.prefix.as_str() || name.starts_with(self.prefix_dot.as_str())
    }
}

impl<'a> Visit<'a> for BodyScanner {
    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        if self.is_expect_call(call_expr) {
            if self.expression_depth > 0 {
                self.has_expect_in_callback = true;
            }
            if self.in_loop {
                self.has_expect_in_loop = true;
            }
        }
        oxc_ast_visit::walk::walk_call_expression(self, call_expr);
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        self.expression_depth += 1;
        oxc_ast_visit::walk::walk_function_body(self, body);
        self.expression_depth -= 1;
    }

    fn visit_for_statement(&mut self, it: &oxc_ast::ast::ForStatement<'a>) {
        self.visit_loop(|s| oxc_ast_visit::walk::walk_for_statement(s, it));
    }
    fn visit_for_in_statement(&mut self, it: &oxc_ast::ast::ForInStatement<'a>) {
        self.visit_loop(|s| oxc_ast_visit::walk::walk_for_in_statement(s, it));
    }
    fn visit_for_of_statement(&mut self, it: &oxc_ast::ast::ForOfStatement<'a>) {
        self.visit_loop(|s| oxc_ast_visit::walk::walk_for_of_statement(s, it));
    }
    fn visit_while_statement(&mut self, it: &oxc_ast::ast::WhileStatement<'a>) {
        self.visit_loop(|s| oxc_ast_visit::walk::walk_while_statement(s, it));
    }
    fn visit_do_while_statement(&mut self, it: &oxc_ast::ast::DoWhileStatement<'a>) {
        self.visit_loop(|s| oxc_ast_visit::walk::walk_do_while_statement(s, it));
    }
}

fn validate_has_assertions_args(call: &CallExpression<'_>, prefix: &str, ctx: &LintContext<'_>) {
    if call.arguments.is_empty() {
        return;
    }
    if let Some(args_span) = call.arguments_span() {
        let delete_span = Span::new(args_span.start, call.span.end - 1);
        let fixer = RuleFixer::new(FixKind::Suggestion, ctx);
        let suggestion = fixer.delete_range(delete_span).with_message("Remove extra arguments");
        ctx.diagnostic_with_suggestions(
            has_assertions_takes_no_arguments(args_span, prefix),
            [suggestion],
        );
    }
}

pub fn validate_assertions_args(call: &CallExpression<'_>, prefix: &str, ctx: &LintContext<'_>) {
    match call.arguments.len() {
        0 => {
            ctx.diagnostic(assertions_requires_one_argument(call.callee.span(), prefix));
        }
        1 => {
            let arg = &call.arguments[0];
            if !matches!(arg, Argument::NumericLiteral(_)) {
                ctx.diagnostic(assertions_requires_number_argument(arg.span()));
            }
        }
        _ => {
            let extra_start = call.arguments[0].span().end;
            let extra_end = call.span.end - 1;
            let extra_span = Span::new(extra_start, extra_end);
            let fixer = RuleFixer::new(FixKind::Suggestion, ctx);
            let suggestion = fixer.delete_range(extra_span).with_message("Remove extra arguments");
            ctx.diagnostic_with_suggestions(
                assertions_requires_one_argument(extra_span, prefix),
                [suggestion],
            );
        }
    }
}

pub fn is_describe_call(call_expr: &CallExpression<'_>) -> bool {
    let callee_name = match &call_expr.callee {
        Expression::Identifier(ident) => ident.name.as_str(),
        Expression::StaticMemberExpression(member) => {
            member.object.get_identifier_reference().map_or("", |id| id.name.as_str())
        }
        Expression::TaggedTemplateExpression(tagged) => match &tagged.tag {
            Expression::StaticMemberExpression(member) => {
                member.object.get_identifier_reference().map_or("", |id| id.name.as_str())
            }
            _ => "",
        },
        _ => "",
    };

    JestFnKind::from(callee_name)
        .to_general()
        .is_some_and(|jest_kind| matches!(jest_kind, JestGeneralFnKind::Describe))
}

fn find_test_callback<'a>(call_expr: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    call_expr.arguments.iter().rev().filter_map(|arg| arg.as_expression()).find(|expr| {
        matches!(expr, Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_))
    })
}

fn callback_body<'a>(callback: &'a Expression<'a>) -> Option<&'a FunctionBody<'a>> {
    match callback {
        Expression::FunctionExpression(func) => func.body.as_ref().map(AsRef::as_ref),
        Expression::ArrowFunctionExpression(func) => Some(&func.body),
        _ => None,
    }
}

fn is_async_callback(callback: &Expression<'_>) -> bool {
    match callback {
        Expression::FunctionExpression(func) => func.r#async,
        Expression::ArrowFunctionExpression(func) => func.r#async,
        _ => false,
    }
}
