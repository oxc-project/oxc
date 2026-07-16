use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, AwaitExpression, ForOfStatement, Function, VariableDeclaration,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_top_level_await_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Top-level `await` is forbidden in published modules.")
        .with_help("Move the `await` inside an `async` function, as ES modules with top-level `await` cannot be loaded with `require(esm)`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoTopLevelAwaitConfig {
    /// If `true`, top-level `await` is allowed in files that start with a
    /// hashbang (`#!`), which marks them as executable scripts rather than
    /// importable modules.
    ignore_bin: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct NoTopLevelAwait(Box<NoTopLevelAwaitConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of top-level `await`, including `for await...of` loops
    /// and `await using` declarations that are not nested inside a function.
    ///
    /// ### Why is this bad?
    ///
    /// Node.js v20.19 introduced `require(esm)`, but ES modules with top-level
    /// `await` cannot be loaded with `require(esm)`. Avoiding top-level `await`
    /// keeps a published module loadable from both CommonJS `require()` and ESM
    /// `import`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const foo = await import('foo');
    ///
    /// for await (const e of asyncIterate()) {
    ///     // ...
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// async function fn() {
    ///     const foo = await import('foo');
    /// }
    /// ```
    ///
    /// ### Options
    ///
    /// #### ignoreBin
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// When set to `true`, top-level `await` is allowed in files that begin with
    /// a hashbang (`#!`), since those are executable scripts rather than
    /// importable modules.
    ///
    /// Example of **correct** code for this rule with `{ "ignoreBin": true }`:
    /// ```js
    /// #!/usr/bin/env node
    /// const foo = await import('foo');
    /// ```
    NoTopLevelAwait,
    node,
    restriction,
    config = NoTopLevelAwaitConfig,
    version = "next",
    short_description = "Disallow top-level `await` in published modules.",
);

impl Rule for NoTopLevelAwait {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Program(program) = node.kind() else {
            return;
        };

        // `ignoreBin`: skip executable scripts that start with a hashbang.
        if self.0.ignore_bin && ctx.source_text().starts_with("#!") {
            return;
        }

        // Top-level `await` can only appear outside of any function, so walk just
        // the top-level code and stop descending at every function boundary. This
        // keeps the rule off the per-node dispatch path for the ubiquitous
        // `VariableDeclaration` and `ForOfStatement` nodes (most of which live
        // inside functions); we visit only the small slice of code that can
        // actually hold a top-level `await`.
        let mut visitor = TopLevelAwaitVisitor { spans: vec![] };
        visitor.visit_program(program);

        for span in visitor.spans {
            ctx.diagnostic(no_top_level_await_diagnostic(span));
        }
    }
}

/// Collects the spans of top-level `await` expressions, `for await...of` loops,
/// and `await using` declarations. Recursion stops at function boundaries, so
/// everything it visits is guaranteed to be top-level.
struct TopLevelAwaitVisitor {
    spans: Vec<Span>,
}

impl<'a> Visit<'a> for TopLevelAwaitVisitor {
    fn visit_await_expression(&mut self, expr: &AwaitExpression<'a>) {
        self.spans.push(expr.span);
        walk::walk_await_expression(self, expr);
    }

    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        if stmt.r#await {
            self.spans.push(stmt.span);
        }
        walk::walk_for_of_statement(self, stmt);
    }

    fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'a>) {
        if decl.kind.is_await() {
            self.spans.push(decl.span);
        }
        walk::walk_variable_declaration(self, decl);
    }

    // Do not descend into functions — their contents are not top-level.
    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _func: &ArrowFunctionExpression<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("import * as foo from 'foo'", None),
        ("for (const e of iterate()) { /* ... */ }", None),
        // Plain `using` (without `await`) is not top-level `await`.
        ("using foo = x", None),
        // `await` nested inside any function form is allowed.
        ("async function fn () { const foo = await import('foo') }", None),
        ("async function fn () { for await (const e of asyncIterate()) { /* ... */ } }", None),
        ("const fn = async () => await import('foo')", None),
        ("const fn = async () => { for await (const e of asyncIterate()) { /* ... */ } }", None),
        ("async function f() { await using foo = x }", None),
        ("class A { async foo () { const bar = await import('bar') } }", None),
        ("const o = { async foo () { const bar = await import('bar') } }", None),
        // A top-level `await` inside a nested function is allowed even when the
        // outer function is not async.
        ("function outer () { return async function () { await import('foo') } }", None),
        (
            "#!/usr/bin/env node\nconst foo = await import('foo')",
            Some(serde_json::json!([{ "ignoreBin": true }])),
        ),
        (
            "#!/usr/bin/env node\nfor await (const e of asyncIterate()) { /* ... */ }",
            Some(serde_json::json!([{ "ignoreBin": true }])),
        ),
        (
            "#!/usr/bin/env node\nawait using foo = x",
            Some(serde_json::json!([{ "ignoreBin": true }])),
        ),
    ];

    let fail = vec![
        ("const foo = await import('foo')", None),
        // A bare top-level `await` expression statement (not inside a declaration).
        ("await import('foo')", None),
        ("for await (const e of asyncIterate()) { /* ... */ }", None),
        ("await using foo = x", None),
        // `await` nested in non-function statements (block, `if`, loop) is still top-level.
        ("{ await import('foo') }", None),
        ("if (cond) { await import('foo') }", None),
        ("for (const e of iterate()) { await handle(e) }", None),
        ("{ await using foo = x }", None),
        // `await` reached through export declarations is still top-level.
        ("export const foo = await import('foo')", None),
        ("export default await import('foo')", None),
        // `ignoreBin` without a hashbang still reports.
        ("const foo = await import('foo')", Some(serde_json::json!([{ "ignoreBin": true }]))),
        // A hashbang without `ignoreBin` still reports.
        ("#!/usr/bin/env node\nconst foo = await import('foo')", None),
    ];

    // Top-level `await` is only valid in ES modules, so parse the fixtures as modules.
    Tester::new(NoTopLevelAwait::NAME, NoTopLevelAwait::PLUGIN, pass, fail)
        .change_rule_path_extension("mts")
        .test_and_snapshot();
}
