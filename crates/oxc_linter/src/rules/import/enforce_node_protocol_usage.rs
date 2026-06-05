use oxc_ast::{
    AstKind,
    ast::{Expression, TSModuleReference},
};
use oxc_macros::declare_oxc_lint;

use crate::{
    AstNode, context::LintContext, rule::Rule,
    rules::unicorn::prefer_node_protocol::check_node_protocol_path,
};

#[derive(Debug, Default, Clone)]
pub struct EnforceNodeProtocolUsage;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using the `node:` protocol when importing Node.js builtin modules.
    ///
    /// ### Why is this bad?
    ///
    /// Node.js builtin modules should be imported using the `node:` protocol to avoid ambiguity with local modules.
    ///
    /// ### Examples
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
    EnforceNodeProtocolUsage,
    import,
    restriction,
    fix,
    version = "next",
);

impl Rule for EnforceNodeProtocolUsage {
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
            AstKind::ImportDeclaration(import) => Some((import.source.value, import.source.span)),
            AstKind::ExportNamedDeclaration(export) => {
                export.source.as_ref().map(|item| (item.value, item.span))
            }
            _ => return,
        };
        let Some((string_lit_value, span)) = string_lit_value_with_span else {
            return;
        };
        check_node_protocol_path(string_lit_value, span, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import fs from "node:fs";"#,
        r#"import fs from "./fs";"#,
        r#"const fs = require("node:fs/promises");"#,
        r#"const fs = require("unicorn");"#,
    ];

    let fail = vec![
        r#"import fs from "fs";"#,
        r#"export {promises} from "fs";"#,
        r#"const fs = require("fs/promises");"#,
        "async function foo() { const fs = await import('assert/strict'); }",
    ];

    let fix = vec![
        (r#"import fs from "fs";"#, r#"import fs from "node:fs";"#),
        (r#"const fs = require("fs/promises");"#, r#"const fs = require("node:fs/promises");"#),
    ];

    Tester::new(EnforceNodeProtocolUsage::NAME, EnforceNodeProtocolUsage::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
