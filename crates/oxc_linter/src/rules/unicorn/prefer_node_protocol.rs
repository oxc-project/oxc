use oxc_ast::{
    ast::{Argument, CallExpression, Expression, ModuleDeclaration},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, utils::NODE_BUILTINS_MODULE, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-node-protocol):")]
#[diagnostic(severity(warning), help(""))]
struct PreferNodeProtocolDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferNodeProtocol;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    PreferNodeProtocol,
    correctness
);

impl Rule for PreferNodeProtocol {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        node.kind();
        let (string_lit_value_with_span) = match node.kind() {
            AstKind::ImportExpression(import) => match import.source {
                Expression::StringLiteral(ref str_lit) => {
                    Some((str_lit.value.clone(), str_lit.span))
                }
                _ => None,
            },
            AstKind::CallExpression(call) if !call.optional => get_static_require_arg(ctx, call),
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
        let module_name = if let Some((prefix, postfix)) = string_lit_value.split_once("/") {
            if !postfix.is_empty() {
                prefix.to_string()
            } else {
                string_lit_value.to_string()
            }
        } else {
            string_lit_value.to_string()
        };
        dbg!(&module_name);
        if module_name.starts_with("node:") || !NODE_BUILTINS_MODULE.contains(&module_name) {
            return;
        }

        ctx.diagnostic(PreferNodeProtocolDiagnostic(span));
    }
}

fn get_static_require_arg<'a>(
    ctx: &LintContext<'a>,
    call: &CallExpression<'a>,
) -> Option<(Atom, Span)> {
    let Expression::Identifier(ref id) = call.callee else { return None };
    match call.arguments.as_slice() {
        [Argument::Expression(Expression::StringLiteral(str))] if id.name == "require" => ctx
            .semantic()
            .scopes()
            .root_unresolved_references()
            .contains_key(&id.name)
            .then(|| (str.value.clone(), str.span)),
        _ => None,
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
        r#"import "punycode/";"#,
        r#"const fs = require("node:fs");"#,
        r#"const fs = require("node:fs/promises");"#,
        r#"const fs = require(fs);"#,
        r#"const fs = notRequire("fs");"#,
        r#"const fs = foo.require("fs");"#,
        r#"const fs = require.resolve("fs");"#,
        r#"const fs = require(`fs`);"#,
        r#"const fs = require?.("fs");"#,
        r#"const fs = require("fs", extra);"#,
        r#"const fs = require();"#,
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
        r#"import {promises} from 'fs';"#,
        r#"import "buffer";"#,
        r#"import "child_process";"#,
        r#"import "timers/promises";"#,
        r#"const {promises} = require("fs")"#,
        r#"const fs = require('fs/promises')"#,
        r#"export fs from "fs";"#,
        r#"await import('assert/strict')"#,
    ];

    Tester::new_without_config(PreferNodeProtocol::NAME, pass, fail).test_and_snapshot();
}
