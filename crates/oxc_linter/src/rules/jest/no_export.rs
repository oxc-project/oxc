use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, jest_ast_util::is_jest_file, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(jest/no-export): Do not export from a test file.")]
#[diagnostic(severity(warning), help("If you want to share code between tests, move it into a separate file and import it from there."))]
struct NoExportDiagnostic(#[label] pub Span);

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
    restriction
);

impl Rule for NoExport {
    fn run_once(&self, ctx: &LintContext) {
        // only used in jest files
        if !is_jest_file(ctx) {
            return;
        }

        for local_export in &ctx.semantic().module_record().local_export_entries {
            ctx.diagnostic(NoExportDiagnostic(local_export.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("describe('a test', () => { expect(1).toBe(1); })", None),
        ("window.location = 'valid'", None),
        ("module.somethingElse = 'foo';", None),
        ("export const myThing = 'valid'", None),
        ("export default function () {}", None),
        ("module.exports = function(){}", None),
        ("module.exports.myThing = 'valid';", None),
    ];

    let fail = vec![
        ("export const myThing = 'invalid'; test('a test', () => { expect(1).toBe(1);});", None),
        (
            "
              export const myThing = 'invalid';
      
              test.each()('my code', () => {
                expect(1).toBe(1);
              });
            ",
            None,
        ),
        (
            "
              export const myThing = 'invalid';
      
              test.each``('my code', () => {
                expect(1).toBe(1);
              });
            ",
            None,
        ),
        (
            "
              export const myThing = 'invalid';
              test.only.each``('my code', () => {
                expect(1).toBe(1);
              });
            ",
            None,
        ),
        ("export default function() {};  test('a test', () => { expect(1).toBe(1);});", None),
        // TODO: support `module.exports`
        // ("module.exports['invalid'] = function() {};  test('a test', () => { expect(1).toBe(1);});", None),
        // ("module.exports = function() {}; ;  test('a test', () => { expect(1).toBe(1);});", None),
        // ("module.export.invalid = function() {}; ;  test('a test', () => { expect(1).toBe(1);});", None)
    ];

    Tester::new(NoExport::NAME, pass, fail).test_and_snapshot();
}
