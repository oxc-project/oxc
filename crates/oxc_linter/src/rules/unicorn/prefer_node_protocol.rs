use oxc_ast::{
    ast::{Expression, ModuleDeclaration, TSModuleReference},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::NODEJS_BUILTINS;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn prefer_node_protocol_diagnostic(span: Span, module_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer using the `node:` protocol when importing Node.js builtin modules.")
        .with_help(format!("Prefer `node:{module_name}` over `{module_name}`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNodeProtocol;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer using the `node:protocol` when importing Node.js builtin modules
    ///
    /// ### Why is this bad?
    ///
    /// Node.js builtin modules should be imported using the `node:` protocol to avoid ambiguity with local modules.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import fs from "fs";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import fs from "node:fs";
    /// ```
    PreferNodeProtocol,
    unicorn,
    restriction,
    fix
);

impl Rule for PreferNodeProtocol {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let string_lit_value_with_span = match node.kind() {
            AstKind::ImportExpression(import) => match &import.source {
                Expression::StringLiteral(str_lit) => Some((str_lit.value, str_lit.span)),
                _ => None,
            },
            AstKind::TSImportEqualsDeclaration(import) => match &import.module_reference {
                TSModuleReference::ExternalModuleReference(external) => {
                    Some((external.expression.value, external.expression.span))
                }
                _ => None,
            },
            AstKind::CallExpression(call) if !call.optional => {
                call.common_js_require().map(|s| (s.value, s.span))
            }
            AstKind::ModuleDeclaration(ModuleDeclaration::ImportDeclaration(import)) => {
                Some((import.source.value, import.source.span))
            }
            AstKind::ModuleDeclaration(ModuleDeclaration::ExportNamedDeclaration(export)) => {
                export.source.as_ref().map(|item| (item.value, item.span))
            }
            _ => None,
        };
        let Some((string_lit_value, span)) = string_lit_value_with_span else {
            return;
        };
        let module_name = if let Some((prefix, postfix)) = string_lit_value.split_once('/') {
            // `e.g. ignore "assert/"`
            if postfix.is_empty() {
                string_lit_value.as_str()
            } else {
                prefix
            }
        } else {
            string_lit_value.as_str()
        };
        if module_name.starts_with("node:") || NODEJS_BUILTINS.binary_search(&module_name).is_err()
        {
            return;
        }

        ctx.diagnostic_with_fix(
            prefer_node_protocol_diagnostic(span, &string_lit_value),
            |fixer| {
                // Smallest module name is 2 chars, plus 2 for quotes = 4.
                debug_assert!(
                    span.size() >= 4,
                    "node stdlib module name should be at least 4 chars long"
                );
                // We're replacing inside the string literal, shift to account for quotes.
                let span = span.shrink_left(1).shrink_right(1);
                fixer.replace(span, format!("node:{string_lit_value}"))
            },
        );
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
        r#"import * as fs from "node:fs";"#,
        r#"import "punycode / ";"#,
        r#"import fs = require("node:fs");"#,
        r#"import type fs = require("node:fs");"#,
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
        r#"import * as fs from "fs";"#,
        r#"import fs = require("fs");"#,
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

    let fix = vec![
        (r#"import fs from "fs";"#, r#"import fs from "node:fs";"#, None),
        (r#"import * as fs from "fs";"#, r#"import * as fs from "node:fs";"#, None),
        (r"import fs from 'fs';", r"import fs from 'node:fs';", None),
        (r"const fs = require('fs');", r"const fs = require('node:fs');", None),
        (r"import fs = require('fs');", r"import fs = require('node:fs');", None),
        (r#"import "child_process";"#, r#"import "node:child_process";"#, None),
        (r#"import fs from "fs/promises";"#, r#"import fs from "node:fs/promises";"#, None),
    ];

    Tester::new(PreferNodeProtocol::NAME, PreferNodeProtocol::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
