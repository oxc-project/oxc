use oxc_ast::{
    ast::{Expression, ModuleDeclaration},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::NODEJS_BUILTINS;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn prefer_node_protocol_diagnostic(span0: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-unicorn(prefer-node-protocol): Prefer using the `node:` protocol when importing Node.js builtin modules.")
        .with_help(format!("Prefer `node:{x1}` over `{x1}`."))
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNodeProtocol;

declare_oxc_lint!(
    /// ### What it does
    /// Prefer using the `node:protocol` when importing Node.js builtin modules
    ///
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// import fs from "fs";
    /// // Good
    /// import fs from "node:fs";
    /// ```
    PreferNodeProtocol,
    restriction
);

impl Rule for PreferNodeProtocol {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let string_lit_value_with_span = match node.kind() {
            AstKind::ImportExpression(import) => match import.source {
                Expression::StringLiteral(ref str_lit) => {
                    Some((str_lit.value.clone(), str_lit.span))
                }
                _ => None,
            },
            AstKind::CallExpression(call) if !call.optional => {
                call.common_js_require().map(|s| (s.value.clone(), s.span))
            }
            AstKind::ModuleDeclaration(ModuleDeclaration::ImportDeclaration(import)) => {
                Some((import.source.value.clone(), import.source.span))
            }
            AstKind::ModuleDeclaration(ModuleDeclaration::ExportNamedDeclaration(export)) => {
                export.source.as_ref().map(|item| (item.value.clone(), item.span))
            }
            _ => None,
        };
        let Some((string_lit_value, span)) = string_lit_value_with_span else {
            return;
        };
        let module_name = if let Some((prefix, postfix)) = string_lit_value.split_once('/') {
            // `e.g. ignore "assert/"`
            if postfix.is_empty() {
                string_lit_value.to_string()
            } else {
                prefix.to_string()
            }
        } else {
            string_lit_value.to_string()
        };
        if module_name.starts_with("node:")
            || NODEJS_BUILTINS.binary_search(&module_name.as_str()).is_err()
        {
            return;
        }

        ctx.diagnostic(prefer_node_protocol_diagnostic(span, &string_lit_value));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import unicorn from "unicorn";"#,
        r#"import fs from "./fs";"#,
        r#"import fs from "unknown-builtin-module";"#,
        r#"import fs from "node:fs";"#,
        r#"import "punycode / ";"#,
        r#"const fs = require("node:fs");"#,
        r#"const fs = require("node:fs/promises");"#,
        r"const fs = require(fs);",
        r#"const fs = notRequire("fs");"#,
        r#"const fs = foo.require("fs");"#,
        r#"const fs = require.resolve("fs");"#,
        r"const fs = require(`fs`);",
        r#"const fs = require?.("fs");"#,
        r#"const fs = require("fs", extra);"#,
        r"const fs = require();",
        r#"const fs = require(...["fs"]);"#,
        r#"const fs = require("unicorn");"#,
    ];

    let fail = vec![
        r#"import fs from "fs";"#,
        r#"export {promises} from "fs";"#,
        r#"import fs from "fs/promises";"#,
        r#"export {default} from "fs/promises";"#,
        r#"import {promises} from "fs";"#,
        r#"export {default as promises} from "fs";"#,
        r"import {promises} from 'fs';",
        r#"import "buffer";"#,
        r#"import "child_process";"#,
        r#"import "timers/promises";"#,
        r#"const {promises} = require("fs")"#,
        r"const fs = require('fs/promises')",
        r#"export fs from "fs";"#,
        r"await import('assert/strict')",
    ];

    Tester::new(PreferNodeProtocol::NAME, pass, fail).test_and_snapshot();
}
