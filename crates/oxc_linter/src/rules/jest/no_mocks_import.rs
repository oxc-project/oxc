use std::path::PathBuf;

use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(jest/no-mocks-import): Mocks should not be manually imported from a `__mocks__` directory.")]
#[diagnostic(
    severity(warning),
    help("Instead use `jest.mock` and import from the original module path.")
)]
struct NoMocksImportDiagnostic(#[label] pub Span);

/// <https://github.com/jest-community/eslint-plugin-jest/blob/main/docs/rules/no-mocks-import.md>
#[derive(Debug, Default, Clone)]
pub struct NoMocksImport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule reports imports from a path containing a __mocks__ component.
    ///
    /// ### Example
    /// ```javascript
    /// import thing from './__mocks__/index';
    /// require('./__mocks__/index');
    /// require('__mocks__');
    ///
    NoMocksImport,
    restriction
);

impl Rule for NoMocksImport {
    fn run_once(&self, ctx: &LintContext) {
        let module_records = ctx.semantic().module_record();

        for import_entry in &module_records.import_entries {
            let module_specifier = import_entry.module_request.name().as_str();
            if contains_mocks_dir(module_specifier) {
                ctx.diagnostic(NoMocksImportDiagnostic(import_entry.module_request.span()));
            }
        }

        let Some(require_reference_ids) = ctx.scopes().root_unresolved_references().get("require")
        else {
            return;
        };

        for reference_id in require_reference_ids {
            let reference = ctx.symbols().get_reference(*reference_id);
            let Some(parent) = ctx.nodes().parent_node(reference.node_id()) else {
                return;
            };
            let AstKind::CallExpression(call_expr) = parent.kind() else {
                return;
            };

            let Some(Argument::Expression(Expression::StringLiteral(string_literal))) =
                call_expr.arguments.get(0)
            else {
                return;
            };

            if contains_mocks_dir(&string_literal.value) {
                ctx.diagnostic(NoMocksImportDiagnostic(string_literal.span));
            }
        }
    }
}

fn contains_mocks_dir(value: &str) -> bool {
    PathBuf::from(value).components().any(|c| match c {
        std::path::Component::Normal(p) => p == std::ffi::OsStr::new("__mocks__"),
        _ => false,
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("import something from 'something'", None),
        ("require('somethingElse')", None),
        ("require('./__mocks__.js')", None),
        ("require('./__mocks__x')", None),
        ("require('./__mocks__x/x')", None),
        ("require('./x__mocks__')", None),
        ("require('./x__mocks__/x')", None),
        ("require()", None),
        ("var path = './__mocks__.js'; require(path)", None),
        ("entirelyDifferent(fn)", None),
    ];

    let fail = vec![
        ("require('./__mocks__')", None),
        ("require('./__mocks__/')", None),
        ("require('./__mocks__/index')", None),
        ("require('__mocks__')", None),
        ("require('__mocks__/')", None),
        ("require('__mocks__/index')", None),
        ("import thing from './__mocks__/index'", None),
    ];

    Tester::new(NoMocksImport::NAME, pass, fail).test_and_snapshot();
}
