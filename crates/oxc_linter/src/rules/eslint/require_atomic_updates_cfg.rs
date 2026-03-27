use oxc_cfg::{
    EdgeType,
    graph::{
        Direction,
        visit::{Control, DfsEvent, set_depth_first_search},
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn variable_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Possible race condition: `{name}` might be reassigned based on an outdated value of `{name}`."
    ))
    .with_help("Use a local variable to store the intermediate value or refactor to avoid compound assignment across an `await` or `yield`.")
    .with_label(span)
}

fn property_diagnostic(value: &str, object: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Possible race condition: `{value}` might be assigned based on an outdated state of `{object}`."
    ))
    .with_help("Use a local variable to store the intermediate value or refactor to avoid property assignment across an `await` or `yield`.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireAtomicUpdatesCfg;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows assignments that can lead to race conditions due to usage of
    /// `await` or `yield`.
    ///
    /// ### Why is this bad?
    ///
    /// When writing asynchronous code, it is possible to create subtle race
    /// conditions that can lead to unexpected results.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let totalLength = 0;
    /// async function addLength(url) {
    ///     totalLength += await getLength(url);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let totalLength = 0;
    /// async function addLength(url) {
    ///     const length = await getLength(url);
    ///     totalLength += length;
    /// }
    /// ```
    RequireAtomicUpdatesCfg,
    eslint,
    suspicious,
    none,
);

impl Rule for RequireAtomicUpdatesCfg {
    fn run_once(&self, ctx: &LintContext) {
        let cfg = ctx.cfg();
        let graph = cfg.graph();
        let nodes = ctx.nodes();

        // TODO: implement CFG-based analysis
        // 1. Find async/generator functions
        // 2. Walk their CFG basic blocks
        // 3. Track which variables are read (marked "fresh")
        // 4. When an await/yield instruction is encountered, mark fresh variables as "outdated"
        // 5. If an outdated variable is then assigned using its own value, report a diagnostic


    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("let foo; async function x() { foo += bar; }", None),
        ("let foo; async function x() { foo = foo + bar; }", None),
        ("let foo; async function x() { foo = await bar + foo; }", None),
        ("async function x() { let foo; foo += await bar; }", None),
        ("let foo; async function x() { foo = (await result)(foo); }", None),
        ("let foo; async function x() { foo = bar(await something, foo) }", None),
        ("function* x() { let foo; foo += yield bar; }", None),
        ("const foo = {}; async function x() { foo.bar = await baz; }", None),
        ("const foo = []; async function x() { foo[x] += 1;  }", None),
        ("let foo; function* x() { foo = bar + foo; }", None),
        ("async function x() { let foo; bar(() => baz += 1); foo += await amount; }", None),
        ("let foo; async function x() { foo = condition ? foo : await bar; }", None),
        ("async function x() { let foo; bar(() => { let foo; blah(foo); }); foo += await result; }", None),
        ("let foo; async function x() { foo = foo + 1; await bar; }", None),
        ("async function x() { foo += await bar; }", None),
    ];

    let fail = vec![
        ("let foo; async function x() { foo += await amount; }", None),
        ("let foo; async function x() { foo = foo + await amount; }", None),
        ("let foo; async function x() { foo += bar + await amount; }", None),
        ("let foo; function* x() { foo += yield baz }", None),
        ("let foo; async function x() { foo = bar(foo, await something) }", None),
    ];

    Tester::new(RequireAtomicUpdatesCfg::NAME, RequireAtomicUpdatesCfg::PLUGIN, pass, fail)
        .test_and_snapshot();
}
