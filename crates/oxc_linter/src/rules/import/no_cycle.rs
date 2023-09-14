use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};
use oxc_syntax::module_record::ModuleRecord;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(no-cycle): Dependency cycle detected")]
#[diagnostic(severity(warning), help("These paths form a cycle: \n{1}"))]
struct NoCycleDiagnostic(#[label] Span, String);

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-cycle.md>
#[derive(Debug, Default, Clone)]
pub struct NoCycle;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that there is no resolvable path back to this module via its dependencies.
    ///
    /// This includes cycles of depth 1 (imported module imports me) to "∞" (or Infinity),
    /// if the maxDepth option is not set.
    ///
    /// ### Why is this bad?
    ///
    /// Dependency cycles lead to confusing architectures where bugs become hard to find.
    ///
    /// It is common to import an `undefined` value that is caused by a cyclic dependency.
    ///
    /// ### Example
    /// ```javascript
    /// // dep-b.js
    /// import './dep-a.js'
    /// export function b() { /* ... */ }
    /// ```
    ///
    /// ```javascript
    /// // dep-a.js
    /// import { b } from './dep-b.js' // reported: Dependency cycle detected.
    /// ```
    NoCycle,
    pedantic
);

impl Rule for NoCycle {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();

        let mut visited: HashSet<PathBuf> = HashSet::new();
        let mut stack: Vec<(Atom, PathBuf)> = Vec::new();
        let cwd = std::env::current_dir().unwrap();

        let needle = &module_record.resolved_absolute_path;
        if visit(module_record, &mut visited, &mut stack, needle) {
            let span = module_record.requested_modules.get(&stack[0].0).unwrap()[0];
            let help = stack
                .into_iter()
                .map(|(specifier, path)| {
                    let path =
                        path.strip_prefix(&cwd).unwrap().to_string_lossy().replace('\\', "/");
                    format!("-> {specifier} - {path}")
                })
                .collect::<Vec<_>>()
                .join("\n");
            ctx.diagnostic(NoCycleDiagnostic(span, help));
        }
    }
}

/// Walks ModuleRecord and returns the path stack
/// if there is a cycle
fn visit(
    module_record: &Arc<ModuleRecord>,
    visited: &mut HashSet<PathBuf>,
    stack: &mut Vec<(Atom, PathBuf)>,
    needle: &Path,
) -> bool {
    let path = &module_record.resolved_absolute_path;
    if path.components().any(|c| match c {
        std::path::Component::Normal(p) => p == std::ffi::OsStr::new("node_modules"),
        _ => false,
    }) {
        return false;
    }
    for module_record_ref in &module_record.loaded_modules {
        let resolved_absolute_path = &module_record_ref.resolved_absolute_path;
        if !visited.insert(resolved_absolute_path.clone()) {
            continue;
        }
        stack.push((module_record_ref.key().clone(), resolved_absolute_path.clone()));
        if needle == resolved_absolute_path {
            return true;
        }
        if visit(module_record_ref.value(), visited, stack, needle) {
            return true;
        }
        stack.pop();
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import foo from './foo.js'",
        "import _ from 'lodash'",
        "import foo from '@scope/foo'",
        "require('./foo')",
        "require('../foo')",
        "require('foo')",
        "require('./')",
        "require('@scope/foo')",
        "require('./bar/index')",
        "require('./bar')",
    ];

    let fail = vec![
        "import { foo } from './es6/depth-one'",
        "const { foo } = require('./es6/depth-one')",
        "import { foo } from './es6/depth-one-reexport'",
        "import { foo } from './es6/depth-two'",
        "import { foo } from './es6/depth-three-star'",
        "import { foo } from './es6/depth-three-indirect'",
        "import { foo } from './intermediate-ignore'",
        "import { foo } from './ignore'",
    ];

    Tester::new_without_config(NoCycle::NAME, pass, fail)
        .change_rule_path("cycles/depth-zero.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
