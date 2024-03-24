use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-global-assign): Read-only global '{0}' should not be modified.")]
#[diagnostic(severity(warning))]
struct NoGlobalAssignDiagnostic(
    CompactStr,
    #[label("Read-only global '{0}' should not be modified.")] pub Span,
);

#[derive(Debug, Default, Clone)]
pub struct NoGlobalAssign(Box<NoGlobalAssignConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoGlobalAssignConfig {
    excludes: Vec<CompactStr>,
}

impl std::ops::Deref for NoGlobalAssign {
    type Target = NoGlobalAssignConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow modifications to read-only global variables.
    ///
    /// ### Why is this bad?
    /// In almost all cases, you don’t want to assign a value to these global variables as doing so could result in losing access to important functionality.
    ///
    /// ### Example
    /// ```javascript
    /// Object = null
    /// ```
    NoGlobalAssign,
    correctness
);

impl Rule for NoGlobalAssign {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self(Box::new(NoGlobalAssignConfig {
            excludes: obj
                .and_then(|v| v.get("exceptions"))
                .and_then(serde_json::Value::as_array)
                .unwrap_or(&vec![])
                .iter()
                .map(serde_json::Value::as_str)
                .filter(std::option::Option::is_some)
                .map(|x| CompactStr::from(x.unwrap()))
                .collect::<Vec<CompactStr>>(),
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let symbol_table = ctx.symbols();
        for reference_id_list in ctx.scopes().root_unresolved_references().values() {
            for &reference_id in reference_id_list {
                let reference = symbol_table.get_reference(reference_id);
                if reference.is_write() && symbol_table.is_global_reference(reference_id) {
                    let name = reference.name();

                    if !self.excludes.contains(name) && ctx.env_contains_var(name) {
                        ctx.diagnostic(NoGlobalAssignDiagnostic(name.clone(), reference.span()));
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("string='1';", None),
        ("var string;", None),
        ("Object = 0;", Some(serde_json::json!([{ "exceptions": ["Object"] }]))),
        ("top = 0;", None),
        // ("onload = 0;", None), // env: { browser: true }
        ("require = 0;", None),
        ("window[parseInt('42', 10)] = 99;", None),
        // ("a = 1", None), // globals: { a: true } },
        // ("/*global a:true*/ a = 1", None),
    ];

    let fail = vec![
        ("String = 'hello world';", None),
        ("String++;", None),
        ("({Object = 0, String = 0} = {});", None),
        // ("top = 0;", None), // env: { browser: true },
        // ("require = 0;", None), // env: { node: true },
        ("function f() { Object = 1; }", None),
        // ("/*global b:false*/ function f() { b = 1; }", None),
        // ("/*global b:false*/ function f() { b++; }", None),
        // ("/*global b*/ b = 1;", None),
        ("Array = 1;", None),
    ];

    Tester::new(NoGlobalAssign::NAME, pass, fail).test_and_snapshot();
}
