use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

fn no_native_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Promise is not defined.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNative;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require creating a `Promise` constructor before using it in an ES5 environment.
    ///
    /// ### Why is this bad?
    ///
    /// Ensure that `Promise` is included fresh in each file instead of relying on the
    /// existence of a native promise implementation. Helpful if you want to use
    /// `bluebird` or if you don't intend to use an ES6 Promise shim.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const x = Promise.resolve('bad')
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const Promise = require('bluebird')
    /// const x = Promise.resolve('good')
    /// ```
    NoNative,
    promise,
    restriction,
);

impl Rule for NoNative {
    fn run_once(&self, ctx: &LintContext) {
        let scoping = ctx.scoping();
        let unresolved_promise_references = scoping.root_unresolved_references().get("Promise");
        if let Some(reference_ids) = unresolved_promise_references {
            if let Some(reference_id) = reference_ids.first() {
                let reference = ctx.scoping().get_reference(*reference_id);
                let node = ctx.nodes().get_node(reference.node_id());
                ctx.diagnostic(no_native_diagnostic(node.span()));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"var Promise = null; function x() { return Promise.resolve("hi"); }"#,
        r#"var Promise = window.Promise || require("bluebird"); var x = Promise.reject();"#,
        r#"import Promise from "bluebird"; var x = Promise.reject();"#,
    ];

    let fail = vec!["new Promise(function(reject, resolve) { })", "Promise.resolve()"];

    Tester::new(NoNative::NAME, NoNative::PLUGIN, pass, fail).test_and_snapshot();
}
