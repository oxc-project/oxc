use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
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
        let span = match node.kind() {
            AstKind::AwaitExpression(await_expr) => await_expr.span,
            AstKind::ForOfStatement(for_of) if for_of.r#await => for_of.span,
            AstKind::VariableDeclaration(decl) if decl.kind.is_await() => decl.span,
            _ => return,
        };

        // Only report when not nested inside a function (i.e. top-level).
        if ctx
            .nodes()
            .ancestor_kinds(node.id())
            .any(|kind| matches!(kind, AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)))
        {
            return;
        }

        // `ignoreBin`: skip executable scripts that start with a hashbang.
        if self.0.ignore_bin && ctx.source_text().starts_with("#!") {
            return;
        }

        ctx.diagnostic(no_top_level_await_diagnostic(span));
    }
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
