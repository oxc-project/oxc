// Based on this gist.
// <https://gist.github.com/developit/a306951af9c0cfdf5925f126428887eb#file-no-barrel-js-L126>

use oxc_allocator::Allocator;
use oxc_ast::{ast::Statement, match_module_declaration, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_module_lexer::ModuleLexer;
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::AstNode;
use oxc_span::{SourceType, Span};
use oxc_syntax::module_graph_visitor::{ModuleGraphVisitorBuilder, VisitFoldWhile};
use std::path::{Path, PathBuf};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
enum NoBarrelFileDiagnostic {
    #[error(
        "oxc(no-barrel-file): \
            Avoid barrel files, they slow down performance, \
            and cause large module graphs with modules that go unused.{1}"
    )]
    #[diagnostic(severity(warning), help("For more information visit this link: <https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-7/>"))]
    BarrelFile(#[label] Span, String),
    #[error("oxc(no-barrel-file): Don't import from barrel files.")]
    #[diagnostic(severity(warning), help("For more information visit this link: <https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-7/>"))]
    BarrelImport(#[label] Span),
}

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

        if program.body.iter().all(|node| {
            matches! {
                node,
                match_module_declaration!(Statement) if !node.to_module_declaration().is_type()
            }
        }) {
            let misc = if !module_record.loaded_modules.is_empty() {
                let loaded_modules_count = ModuleGraphVisitorBuilder::default()
                    .visit_fold(0, module_record, |acc, _, _| VisitFoldWhile::Next(acc + 1))
                    .result;
                format!("\nLoading this barrel file results in importing at least {loaded_modules_count:?} modules.")
            } else {
                String::default()
            };

            ctx.diagnostic(NoBarrelFileDiagnostic::BarrelFile(program.span, misc));
        }
    }

    fn run(&self, node: &AstNode<'_>, ctx: &LintContext<'_>) {
        let AstKind::ImportDeclaration(import) = node.kind() else { return };
        if is_facade_import(ctx.file_path().to_str().unwrap(), import.source.value.as_str()) {
            ctx.diagnostic(NoBarrelFileDiagnostic::BarrelImport(import.source.span))
        }
    }
}

/// Returns `false` if can't confirm a file is a facade.
fn is_facade_import(filename: &str, source: &str) -> bool {
    let Some(ref potential_barrel) = try_resolve_path(filename, source) else { return false };
    let Ok(source) = std::fs::read_to_string(potential_barrel) else { return false };
    let Ok(source_type) = SourceType::from_path(potential_barrel) else { return false };

    let allocator = Allocator::default();
    let ParserReturn { ref program, .. } = Parser::new(&allocator, &source, source_type).parse();
    let ModuleLexer { facade, .. } = ModuleLexer::new().build(program);
    facade
}

fn try_resolve_path<'a, P: AsRef<Path>>(from: P, to: P) -> Option<PathBuf> {
    const EXTENSIONS: [&str; 6] = ["js", "ts", "jsx", "tsx", "cjs", "mjs"];
    fn try_extensions<'a>(path: PathBuf) -> Option<PathBuf> {
        EXTENSIONS
            .iter()
            .flat_map(|ext| [path.join("index").join(ext), path.join(ext)])
            .find(|fullpath| fullpath.exists())
    }

    let cwd: &Path = from.as_ref().parent()?.as_ref();
    let to = to.as_ref();

    // TODO: check if path is a package.
    let path = if to.starts_with(".") { cwd.join(to) } else { to.to_path_buf() };

    let result = if path.extension().is_some() && path.exists() {
        Some(path)
    } else if let Some(path) = try_extensions(path) {
        Some(path)
    } else {
        None
    };

    result.map(|path| path.to_owned())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"export type * from "foo";"#,
        r#"export type { foo } from "foo";"#,
        r#"export type * from "foo";
           export type { bar } from "bar";"#,
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
        r#"import { foo, bar, baz } from "../feature";
           export { foo };
           export { bar };"#,
    ];

    Tester::new(NoBarrelFile::NAME, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
