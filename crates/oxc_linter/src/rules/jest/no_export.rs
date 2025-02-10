use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::is_jest_file};

fn no_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not export from a test file.")
        .with_help("If you want to share code between tests, move it into a separate file and import it from there.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoExport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents using exports if a file has one or more tests in it.
    ///
    /// ### Why is this bad?
    ///
    /// This rule aims to eliminate duplicate runs of tests by exporting things from test files.
    ///  If you import from a test file, then all the tests in that file will be run in each imported instance.
    /// so bottom line, don't export from a test, but instead move helper functions into a separate file when they need to be shared across tests.
    ///
    /// ### Example
    /// ```javascript
    /// export function myHelper() {}
    /// describe('a test', () => {
    ///   expect(1).toBe(1);
    /// });
    /// ```
    NoExport,
    jest,
    correctness
);

impl Rule for NoExport {
    fn run_once(&self, ctx: &LintContext) {
        // only used in jest files
        if !is_jest_file(ctx) {
            return;
        }

        for span in ctx.module_record().exported_bindings.values() {
            ctx.diagnostic(no_export_diagnostic(*span));
        }

        if let Some(span) = ctx.module_record().export_default {
            ctx.diagnostic(no_export_diagnostic(span));
        }
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "describe('a test', () => { expect(1).toBe(1); })",
            None,
            None,
            Some(PathBuf::from("foo.test.js")),
        ),
        ("window.location = 'valid'", None, None, None),
        ("module.somethingElse = 'foo';", None, None, None),
        ("export const myThing = 'valid'", None, None, Some(PathBuf::from("foo.js"))),
        ("export default function () {}", None, None, Some(PathBuf::from("foo.js"))),
        ("module.exports = function(){}", None, None, None),
        ("module.exports.myThing = 'valid';", None, None, None),
    ];

    let fail = vec![
        (
            "export const myThing = 'invalid'; test('a test', () => { expect(1).toBe(1);});",
            None,
            None,
            Some(PathBuf::from("foo.test.js")),
        ),
        (
            "
              export const myThing = 'invalid';

              test.each()('my code', () => {
                expect(1).toBe(1);
              });
            ",
            None,
            None,
            Some(PathBuf::from("foo.test.js")),
        ),
        (
            "
              export const myThing = 'invalid';

              test.each``('my code', () => {
                expect(1).toBe(1);
              });
            ",
            None,
            None,
            Some(PathBuf::from("foo.test.js")),
        ),
        (
            "
              export const myThing = 'invalid';
              test.only.each``('my code', () => {
                expect(1).toBe(1);
              });
            ",
            None,
            None,
            Some(PathBuf::from("foo.test.js")),
        ),
        (
            "export default function() {};  test('a test', () => { expect(1).toBe(1);});",
            None,
            None,
            Some(PathBuf::from("foo.test.js")),
        ),
        (
            "
              const foo = 1;
              const bar = 2;
              test('a test', () => {})

              export {foo, bar};
            ",
            None,
            None,
            Some(PathBuf::from("foo.test.js")),
        ),
        // TODO: support `module.exports`
        // ("module.exports['invalid'] = function() {};  test('a test', () => { expect(1).toBe(1);});", None),
        // ("module.exports = function() {}; ;  test('a test', () => { expect(1).toBe(1);});", None),
        // ("module.export.invalid = function() {}; ;  test('a test', () => { expect(1).toBe(1);});", None)
    ];

    Tester::new(NoExport::NAME, NoExport::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
