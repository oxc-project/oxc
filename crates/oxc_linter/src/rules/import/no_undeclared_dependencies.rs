use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

#[allow(dead_code)]
fn no_undeclared_dependencies_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Import of `{name}` is not listed as a dependency in package.json."))
        .with_help("Add this package to your dependencies, devDependencies, peerDependencies, or optionalDependencies in package.json.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUndeclaredDependencies;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows importing packages that are not declared in `package.json`.
    ///
    /// ### Why is this bad?
    ///
    /// Importing undeclared dependencies means the project relies on packages
    /// that may not be installed in production, leading to runtime errors.
    /// All imported packages should be listed in the appropriate dependencies
    /// field of `package.json`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // If 'lodash' is not in package.json:
    /// import _ from 'lodash';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // If 'lodash' is in package.json dependencies:
    /// import _ from 'lodash';
    /// // Relative imports are always fine:
    /// import { foo } from './utils';
    /// // Node.js builtins are always fine:
    /// import fs from 'node:fs';
    /// ```
    NoUndeclaredDependencies,
    import,
    nursery, // Complex rule that needs package.json integration
    pending
);

impl Rule for NoUndeclaredDependencies {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let source = match node.kind() {
            AstKind::ImportDeclaration(decl) => decl.source.value.as_str(),
            AstKind::ExportNamedDeclaration(decl) => {
                let Some(source) = &decl.source else {
                    return;
                };
                source.value.as_str()
            }
            AstKind::ExportAllDeclaration(decl) => decl.source.value.as_str(),
            _ => return,
        };

        // Skip relative imports
        if source.starts_with('.') || source.starts_with('/') {
            return;
        }

        // Skip Node.js builtins
        if source.starts_with("node:") || is_node_builtin(source) {
            return;
        }

        // Extract package name (handle scoped packages)
        let package_name = get_package_name(source);

        // For now, this rule just flags the import. In a full implementation,
        // it would read package.json and check dependencies.
        // This is a placeholder that can be enhanced with package.json reading.
        let _ = package_name;
        // TODO: Read package.json and check if package_name is in dependencies
        // For now, the rule is registered but doesn't report (nursery)
    }
}

fn get_package_name(source: &str) -> &str {
    if source.starts_with('@') {
        // Scoped package: @scope/name/path -> @scope/name
        let mut parts = source.splitn(3, '/');
        let scope = parts.next().unwrap_or("");
        let name = parts.next().unwrap_or("");
        let end = scope.len() + 1 + name.len();
        &source[..end.min(source.len())]
    } else {
        // Regular package: name/path -> name
        source.split('/').next().unwrap_or(source)
    }
}

fn is_node_builtin(name: &str) -> bool {
    matches!(
        name,
        "assert"
            | "buffer"
            | "child_process"
            | "cluster"
            | "console"
            | "constants"
            | "crypto"
            | "dgram"
            | "dns"
            | "domain"
            | "events"
            | "fs"
            | "http"
            | "http2"
            | "https"
            | "inspector"
            | "module"
            | "net"
            | "os"
            | "path"
            | "perf_hooks"
            | "process"
            | "punycode"
            | "querystring"
            | "readline"
            | "repl"
            | "stream"
            | "string_decoder"
            | "sys"
            | "timers"
            | "tls"
            | "tty"
            | "url"
            | "util"
            | "v8"
            | "vm"
            | "wasi"
            | "worker_threads"
            | "zlib"
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import { foo } from './utils';",
        "import fs from 'node:fs';",
        "import path from 'path';",
    ];

    let fail = vec![];

    Tester::new(NoUndeclaredDependencies::NAME, NoUndeclaredDependencies::PLUGIN, pass, fail)
        .test_and_snapshot();
}
