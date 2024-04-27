use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::module_graph_visitor::{ModuleGraphVisitorBuilder, VisitFoldWhile};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "oxc(no-barrel-file): \
            Avoid barrel files, they slow down performance, \
            and cause large module graphs with modules that go unused.\n\
            Loading this barrel file results in importing {1:?} modules."
)]
#[diagnostic(severity(warning), help("For more information visit this link: <https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-7/>"))]
struct NoBarrelFileDiagnostic(#[label] pub Span, pub u32);

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
        let module_record = semantic.module_record();
        let Some(root) = semantic.nodes().root_node() else {
            // Return early if the semantic's root node isn't set.
            // It usually means we are running on an empty or invalid file.
            return;
        };

        let AstKind::Program(program) = root.kind() else { unreachable!() };

        let declarations =
            program
                .body
                .iter()
                .fold(0, |acc, node| if node.is_declaration() { acc + 1 } else { acc });
        let exports =
            module_record.star_export_entries.len() + module_record.indirect_export_entries.len();

        if exports > declarations
            && exports > AMOUNT_OF_EXPORTS_TO_CONSIDER_MODULE_AS_BARREL as usize
        {
            let loaded_modules_count = ModuleGraphVisitorBuilder::default()
                .visit_fold(0, module_record, |acc, _, _| VisitFoldWhile::Next(acc + 1))
                .result;
            ctx.diagnostic(NoBarrelFileDiagnostic(program.span, loaded_modules_count));
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
           export type { bar } from "bar";"#,
        r#"import { foo, bar, baz } from "../feature";
           export { foo };
           export { bar };"#,
    ];

    let fail = vec![
        r#"export * from "./deep/a.js";
           export * from "./deep/b.js";
           export * from "./deep/c.js";
           export * from "./deep/d.js";"#,
        r#"export { foo } from "foo";
           export { bar } from "bar";
           export { baz } from "baz";
           export { qux } from "qux";"#,
        r#"export { default as module1 } from "./module1";
           export { default as module2 } from "./module2";
           export { default as module3 } from "./module3";
           export { default as module4 } from "./module4";"#,
        r#"export { foo, type Foo } from "foo";
           export { bar, type Bar } from "bar";
           export { baz, type Baz } from "baz";
           export { qux, type Qux } from "qux";"#,
    ];

    Tester::new(NoBarrelFile::NAME, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
