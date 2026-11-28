use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, ParsedJestFnCallNew, PossibleJestNode,
        collect_possible_jest_call_node, parse_jest_fn_call, test_framework_imported_name,
    },
};

fn consistent_method(preferred_method: &str, other_method: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce `test` and `it` usage conventions")
        .with_help(format!("Prefer using {preferred_method:?} instead of {other_method:?}"))
        .with_label(span)
}

fn consistent_method_within_describe(
    preferred_method: &str,
    other_method: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce `test` and `it` usage conventions")
        .with_help(format!(
            "Prefer using {preferred_method:?} instead of {other_method:?} within describe"
        ))
        .with_label(span)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
enum TestCaseName {
    IT,
    Test,
}

impl TestCaseName {
    pub fn as_str(self) -> &'static str {
        match self {
            TestCaseName::IT => "it",
            TestCaseName::Test => "test",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ConsistentTestItConfig {
    /// Decides whether to use `test` or `it` within a `describe` scope.
    /// If only `fn` is provided, this will default to the value of `fn`.
    ///
    /// Examples of **incorrect** code for `{ "withinDescribe": "test" }`:
    /// ```javascript
    /// describe('foo', function () {
    ///     it('bar');
    /// });
    /// ```
    ///
    /// Examples of **correct** code for `{ "withinDescribe": "test" }`:
    /// ```javascript
    /// describe('foo', function () {
    ///     test('bar');
    /// });
    /// ```
    within_describe: TestCaseName,
    /// Decides whether to use `test` or `it`.
    ///
    /// Examples of **incorrect** code for `{ "fn": "test" }`:
    /// ```javascript
    /// it('foo');
    /// it.only('foo');
    /// ```
    ///
    /// Examples of **correct** code for `{ "fn": "test" }`:
    /// ```javascript
    /// test('foo');
    /// test.only('foo');
    /// ```
    ///
    /// Examples of **incorrect** code for `{ "fn": "it" }`:
    /// ```javascript
    /// test('foo');
    /// test.only('foo');
    /// ```
    ///
    /// Examples of **correct** code for `{ "fn": "it" }`:
    /// ```javascript
    /// it('foo');
    /// it.only('foo');
    /// ```
    r#fn: TestCaseName,
}

impl Default for ConsistentTestItConfig {
    fn default() -> Self {
        Self { within_describe: TestCaseName::IT, r#fn: TestCaseName::Test }
    }
}

impl ConsistentTestItConfig {
    #[expect(clippy::unnecessary_wraps)] // TODO: fail on serde error
    pub fn from_configuration(value: &serde_json::Value) -> Result<Self, serde_json::error::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }

        let config_value = value.get(0).unwrap_or(value);

        let mut config = ConsistentTestItConfig::deserialize(config_value).unwrap_or_default();

        // If withinDescribe wasn't provided, default it to the value of `fn` only if fn was explicitly provided
        if config_value.get("withinDescribe").is_none() && config_value.get("fn").is_some() {
            config.within_describe = config.r#fn;
        }

        Ok(config)
    }

    pub fn run_once(self, ctx: &LintContext) {
        let mut describe_stack = vec![];
        let mut possible_jest_nodes = collect_possible_jest_call_node(ctx);
        possible_jest_nodes.sort_unstable_by_key(|n| n.node.id());

        for possible_jest_node in &possible_jest_nodes {
            self.run(&mut describe_stack, possible_jest_node, ctx);
        }
    }
}

/// Whether the identifier the call is made through (`myTest` in `myTest.skip(...)`) is a local
/// binding: declared in the file and not an import.
fn callee_is_local(possible_jest_node: &PossibleJestNode, ctx: &LintContext) -> bool {
    let AstKind::CallExpression(call) = possible_jest_node.node.kind() else {
        return false;
    };
    let mut expr = call.callee.get_inner_expression();
    let ident = loop {
        expr = match expr {
            Expression::Identifier(ident) => break ident,
            Expression::StaticMemberExpression(member) => member.object.get_inner_expression(),
            Expression::ComputedMemberExpression(member) => member.object.get_inner_expression(),
            Expression::CallExpression(call) => call.callee.get_inner_expression(),
            Expression::TaggedTemplateExpression(tagged) => tagged.tag.get_inner_expression(),
            _ => return false,
        };
    };
    let scoping = ctx.scoping();
    scoping
        .get_reference(ident.reference_id())
        .symbol_id()
        .is_some_and(|symbol_id| !scoping.symbol_flags(symbol_id).is_import())
}

/// Whether renaming the call to `name` (`test`, `xit`, `test.only`...) keeps it the test framework's
/// function of that name (#29181):
/// - nothing in scope is called that: the global function, which only a call to a global may rely on
///   (renaming an imported `it` to `test` would leave it unbound when Vitest globals are off);
/// - something is: only when it is the framework's own function by that name, imported from it, not
///   a local, nor another import aliased to that name.
///
/// Nor is a call renamed whose own name is a local binding, such as a `test.extend(...)` result:
/// the framework's function under the new name would not have what that binding adds.
fn rename_keeps_binding(
    possible_jest_node: &PossibleJestNode,
    name: &str,
    ctx: &LintContext,
) -> bool {
    if callee_is_local(possible_jest_node, ctx) {
        return false;
    }
    let root = name.split_once('.').map_or(name, |(root, _)| root);
    match ctx.scoping().find_binding(possible_jest_node.node.scope_id(), root.into()) {
        None => possible_jest_node.original.is_none(),
        Some(symbol_id) => test_framework_imported_name(ctx, symbol_id) == Some(root),
    }
}

impl ConsistentTestItConfig {
    fn run<'a>(
        self,
        describe_stack: &mut Vec<Span>,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let node_span = node.span();
        while describe_stack.last().is_some_and(|span| !span.contains_inclusive(node_span)) {
            describe_stack.pop();
        }

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(ParsedJestFnCallNew::GeneralJest(jest_fn_call)) =
            parse_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        if matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe)) {
            describe_stack.push(node_span);
            return;
        }

        let is_test = matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Test));
        let is_within_describe = !describe_stack.is_empty();
        let fn_to_str = self.r#fn.as_str();

        if is_test && !is_within_describe && !jest_fn_call.name.ends_with(&fn_to_str) {
            let opposite_test_keyword = Self::get_opposite_test_case(self.r#fn);
            if let Some((span, prefer_test_name)) = Self::get_prefer_test_name_and_span(
                call_expr.callee.get_inner_expression(),
                &jest_fn_call.name,
                fn_to_str,
            ) {
                let diagnostic = consistent_method(fn_to_str, opposite_test_keyword, span);
                if rename_keeps_binding(possible_jest_node, &prefer_test_name, ctx) {
                    ctx.diagnostic_with_fix(diagnostic, |fixer| {
                        fixer.replace(span, prefer_test_name)
                    });
                } else {
                    ctx.diagnostic(diagnostic);
                }
            }
        }

        let describe_to_str = self.within_describe.as_str();

        if is_test && is_within_describe && !jest_fn_call.name.ends_with(&describe_to_str) {
            let opposite_test_keyword = Self::get_opposite_test_case(self.within_describe);
            if let Some((span, prefer_test_name)) = Self::get_prefer_test_name_and_span(
                call_expr.callee.get_inner_expression(),
                &jest_fn_call.name,
                describe_to_str,
            ) {
                let diagnostic =
                    consistent_method_within_describe(describe_to_str, opposite_test_keyword, span);
                if rename_keeps_binding(possible_jest_node, &prefer_test_name, ctx) {
                    ctx.diagnostic_with_fix(diagnostic, |fixer| {
                        fixer.replace(span, prefer_test_name)
                    });
                } else {
                    ctx.diagnostic(diagnostic);
                }
            }
        }
    }

    fn get_opposite_test_case(test_case_name: TestCaseName) -> &'static str {
        if matches!(test_case_name, TestCaseName::Test) { "it" } else { "test" }
    }

    fn get_prefer_test_name_and_span<'s>(
        expr: &Expression,
        test_name: &str,
        fix_jest_name: &'s str,
    ) -> Option<(Span, Cow<'s, str>)> {
        match expr {
            Expression::Identifier(ident) => {
                if ident.name.eq("fit") {
                    return Some((ident.span(), Cow::Borrowed("test.only")));
                }

                let prefer_test_name = match test_name.chars().next() {
                    Some('x') => Cow::Owned(format!("x{fix_jest_name}")),
                    Some('f') => Cow::Owned(format!("f{fix_jest_name}")),
                    _ => Cow::Borrowed(fix_jest_name),
                };
                Some((ident.span(), prefer_test_name))
            }
            Expression::StaticMemberExpression(expr) => {
                Self::get_prefer_test_name_and_span(&expr.object, test_name, fix_jest_name)
            }
            Expression::CallExpression(call_expr) => Self::get_prefer_test_name_and_span(
                call_expr.callee.get_inner_expression(),
                test_name,
                fix_jest_name,
            ),
            Expression::TaggedTemplateExpression(expr) => Self::get_prefer_test_name_and_span(
                expr.tag.get_inner_expression(),
                test_name,
                fix_jest_name,
            ),
            _ => None,
        }
    }
}
