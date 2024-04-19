use oxc_ast::{ast::Statement, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-import(no-barrel-file): \
            Avoid barrel files, they slow down performance, \
            and cause large module graphs with modules that go unused."
)]
#[diagnostic(severity(warning), help("For more information visit this link: <https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-7/>"))]
struct NoBarrelFileDiagnostic(#[label] pub Span);

/// Minimum amount of exports to consider module as barrelfile
const AMOUNT_OF_EXPORTS_TO_CONSIDER_MODULE_AS_BARREL: u8 = 3;

/// <https://github.com/thepassle/eslint-plugin-barrel-files/blob/main/docs/rules/avoid-barrel-files.md>
#[derive(Debug, Default, Clone)]
pub struct NoBarrelFile;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of barrel files.
    ///
    /// ### Example
    ///
    /// Invalid:
    /// ```javascript
    /// export { foo } from 'foo';
    /// export { bar } from 'bar';
    /// export { baz } from 'baz';
    /// export { qux } from 'qux';
    /// ```
    /// Valid:
    /// ```javascript
    /// export type { foo } from './foo.js';
    /// ```
    NoBarrelFile,
    nursery
);

impl Rule for NoBarrelFile {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let semantic = ctx.semantic();
        let mod_rec = semantic.module_record();
        let root = semantic.nodes().root_node();

        let AstKind::Program(program) = root.kind() else { unreachable!() };

        let declarations = program.body.iter().fold(0, |acc, node| match node {
            Statement::Declaration(_) => acc + 1,
            _ => acc,
        });
        let exports = mod_rec.star_export_entries.len() + mod_rec.indirect_export_entries.len();

        if exports > declarations
            && exports > AMOUNT_OF_EXPORTS_TO_CONSIDER_MODULE_AS_BARREL as usize
        {
            ctx.diagnostic(NoBarrelFileDiagnostic(program.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"export type * from "foo";"#,
        r#"export type { foo } from "foo";"#,
        r#"export type * from "foo";
           export type { bar } from bar;"#,
    ];

    let fail = vec![
        r#"export * from "foo";
           export * from "bar";"#,
        r#"export { foo } from "foo";
           export { bar } from "bar";"#,
        r#"export { default as module1 } from "./module1";"#,
        r#"export { foo, type Bar } from "foo";"#,
        r#"import { foo, bar, baz } from "../feature";
           export { foo, bar, baz };"#,
    ];

    Tester::new(NoBarrelFile::NAME, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
