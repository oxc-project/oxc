use std::path::PathBuf;

use oxc_ast::{AstKind, ast::Argument};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_mocks_import_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Mocks should not be manually imported from a `__mocks__` directory.")
        .with_help("Instead use `jest.mock` and import from the original module path.")
        .with_label(span)
}

/// <https://github.com/jest-community/eslint-plugin-jest/blob/v28.9.0/docs/rules/no-mocks-import.md>
#[derive(Debug, Default, Clone)]
pub struct NoMocksImport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule reports imports from a path containing a __mocks__ component.
    ///
    /// ### Why is this bad?
    ///
    /// Manually importing mocks from a `__mocks__` directory can lead to unexpected behavior
    /// and breaks Jest's automatic mocking system. Jest is designed to automatically resolve
    /// and use mocks from `__mocks__` directories when `jest.mock()` is called. Directly
    /// importing from these directories bypasses Jest's module resolution system and can cause
    /// inconsistencies between test and production environments.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// import thing from './__mocks__/index';
    /// require('./__mocks__/index');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// import thing from 'thing';
    /// require('thing');
    /// ```
    NoMocksImport,
    jest,
    style
);

impl Rule for NoMocksImport {
    fn run_once(&self, ctx: &LintContext) {
        let module_records = ctx.module_record();

        for import_entry in &module_records.import_entries {
            let module_specifier = import_entry.module_request.name();
            if contains_mocks_dir(module_specifier) {
                ctx.diagnostic(no_mocks_import_diagnostic(import_entry.module_request.span));
            }
        }

        let Some(require_reference_ids) = ctx.scoping().root_unresolved_references().get("require")
        else {
            return;
        };

        for &reference_id in require_reference_ids {
            let reference = ctx.scoping().get_reference(reference_id);
            let AstKind::CallExpression(call_expr) = ctx.nodes().parent_kind(reference.node_id())
            else {
                return;
            };

            let Some(Argument::StringLiteral(string_literal)) = call_expr.arguments.first() else {
                return;
            };

            if contains_mocks_dir(&string_literal.value) {
                ctx.diagnostic(no_mocks_import_diagnostic(string_literal.span));
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

    Tester::new(NoMocksImport::NAME, NoMocksImport::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
