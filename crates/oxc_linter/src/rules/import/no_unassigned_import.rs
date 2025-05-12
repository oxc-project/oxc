use fast_glob::glob_match;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_unassigned_import_diagnostic(span: Span, msg: &str) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn(msg.to_string()).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnassignedImport {
    allow: Vec<CompactStr>,
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoUnassignedImport,
    import,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoUnassignedImport {
    fn from_configuration(value: Value) -> Self {
        let obj = value.get(0);
        Self {
            allow: obj
                .and_then(|v| v.get("allow"))
                .and_then(Value::as_array)
                .map(|v| v.iter().filter_map(Value::as_str).map(CompactStr::from).collect())
                .unwrap_or_default(),
        }
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import_decl) => {
                if !import_decl.specifiers.is_none() {
                    return;
                }

                let source_str = import_decl.source.value.as_str();
                if self.allow.iter().any(|pattern| glob_match(pattern.as_str(), source_str)) {
                    return;
                }
                ctx.diagnostic(no_unassigned_import_diagnostic(
                    import_decl.span,
                    "Imported module should be assigned",
                ));
            }
            AstKind::ExpressionStatement(statement) => {
                let Expression::CallExpression(call_expr) = &statement.expression else {
                    return;
                };
                if !call_expr.is_require_call() {
                    return;
                }
                let first_arge = &call_expr.arguments[0];
                if let Argument::StringLiteral(source_str) = first_arge {
                    if self
                        .allow
                        .iter()
                        .any(|pattern| glob_match(pattern.as_str(), source_str.value.as_str()))
                    {
                        return;
                    }
                    ctx.diagnostic(no_unassigned_import_diagnostic(
                        call_expr.span,
                        "A `require()` style import is forbidden.",
                    ));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("import _ from 'foo'", None),
        ("import foo from 'foo'", None),
        ("import foo, { bar } from 'foo'", None),
        ("import * as _ from 'foo'", None),
    ];

    let fail = vec![
        ("require('should')", None),
        ("import 'foo'", None),
        ("import '../styles/app.css'", Some(json!([{ "allow": ["/styles/*.css"]}]))),
    ];

    Tester::new(NoUnassignedImport::NAME, NoUnassignedImport::PLUGIN, pass, fail)
        .test_and_snapshot();
}
