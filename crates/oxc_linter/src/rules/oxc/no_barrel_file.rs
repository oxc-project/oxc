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
use oxc_semantic::{AstNode, ModuleRecord};
use oxc_span::{SourceType, Span};
use oxc_syntax::module_graph_visitor::{ModuleGraphVisitorBuilder, VisitFoldWhile};
use std::path::{Path, PathBuf};

use crate::{context::LintContext, rule::Rule};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error, Diagnostic)]
enum NoBarrelFileDiagnostic {
    #[error(
        "oxc(no-barrel-file): \
            Avoid barrel files, they slow down performance, \
            and cause large module graphs with modules that go unused."
    )]
    #[diagnostic(severity(warning), help("For more information visit this link: <https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-7/>"))]
    BarrelFile(#[label] Span),
    #[error(
        "oxc(no-barrel-file): \
            Avoid barrel files, they slow down performance, \
            and cause large module graphs with modules that go unused.\n\
            Loading this barrel file results in importing at least {1} modules."
    )]
    #[diagnostic(severity(warning), help("For more information visit this link: <https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-7/>"))]
    BarrelFileWithDetails(#[label] Span, i32),
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
            let diag = if let Some(count) = count_loaded_modules(module_record) {
                NoBarrelFileDiagnostic::BarrelFileWithDetails(program.span, count)
            } else {
                NoBarrelFileDiagnostic::BarrelFile(program.span)
            };

            ctx.diagnostic(diag);
        }
    }

    fn run(&self, node: &AstNode<'_>, ctx: &LintContext<'_>) {
        let AstKind::ImportDeclaration(import) = node.kind() else { return };
        if is_facade_import(ctx.file_path().to_str().unwrap(), import.source.value.as_str()) {
            ctx.diagnostic(NoBarrelFileDiagnostic::BarrelImport(import.source.span));
        }
    }
}

fn count_loaded_modules(module_record: &ModuleRecord) -> Option<i32> {
    if module_record.loaded_modules.is_empty() {
        None
    } else {
        Some(
            ModuleGraphVisitorBuilder::default()
                .visit_fold(0, module_record, |acc, _, _| VisitFoldWhile::Next(acc + 1))
                .result,
        )
    }
}

/// Returns `false` if can't confirm a file is a facade.
fn is_facade_import(filename: &str, source: &str) -> bool {
    let Some(ref potential_barrel) = try_resolve_path(filename, source) else { return false };

    if !potential_barrel.file_name().is_some_and(|name| name.to_string_lossy().starts_with("index"))
    {
        return false;
    }

    let Ok(source) = std::fs::read_to_string(potential_barrel) else { return false };
    let Ok(source_type) = SourceType::from_path(potential_barrel) else { return false };

    let allocator = Allocator::default();
    let ParserReturn { ref program, .. } = Parser::new(&allocator, &source, source_type).parse();
    let ModuleLexer { facade, .. } = ModuleLexer::new().build(program);
    facade
}

fn try_resolve_path<P: AsRef<Path>>(from: P, to: P) -> Option<PathBuf> {
    const EXTENSIONS: [&str; 6] = ["js", "ts", "jsx", "tsx", "cjs", "mjs"];
    fn try_extensions(path: &Path) -> Option<PathBuf> {
        EXTENSIONS
            .iter()
            .flat_map(|ext| [(path.join("index"), ext), (path.to_path_buf(), ext)])
            .map(|(mut path, ext)| {
                path.set_extension(ext);
                path
            })
            .find(|fullpath| fullpath.canonicalize().is_ok_and(|it| it.exists()))
    }

    let cwd: &Path = from.as_ref().parent()?;
    let to = to.as_ref();

    // TODO: check if path is a package.
    // Is relative in respect to the node paths? Detects paths starting with a dot(`.`).
    let path = if to.iter().next().is_some_and(|seg| seg.to_string_lossy().starts_with('.')) {
        cwd.join(to)
    } else {
        // TODO: We need to have root of the project to resolve most of absolute paths.
        to.to_path_buf()
    };

    if path.extension().is_some() && path.exists() {
        Some(path)
    } else {
        try_extensions(&path)
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
        r#"import { foo, bar, baz } from "../import/export-star/models"
            foo("noop")"#,
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
        r#"import { foo, bar, baz } from "../import/export-star/models";
           export { foo };
           export { bar };"#,
    ];

    Tester::new(NoBarrelFile::NAME, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
