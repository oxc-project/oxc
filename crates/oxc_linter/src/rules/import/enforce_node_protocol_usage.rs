use oxc_ast::{
    AstKind,
    ast::{Expression, TSModuleReference},
};
use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::DefaultRuleConfig,
    rule::Rule,
    rules::unicorn::prefer_node_protocol::{NodeProtocolMode, check_node_protocol_path},
};

#[derive(Debug, Default, PartialEq, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum NodeProtocolUsageMode {
    /// Require the `node:` protocol for Node.js builtin modules.
    #[default]
    Always,
    /// Disallow the `node:` protocol for Node.js builtin modules.
    Never,
}

impl From<NodeProtocolUsageMode> for NodeProtocolMode {
    fn from(mode: NodeProtocolUsageMode) -> Self {
        match mode {
            NodeProtocolUsageMode::Always => Self::Always,
            NodeProtocolUsageMode::Never => Self::Never,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnforceNodeProtocolUsage(NodeProtocolUsageMode);

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
    ///
    /// Examples of **incorrect** code for this rule with the `"never"` option:
    /// ```javascript
    /// import fs from "node:fs";
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"never"` option:
    /// ```javascript
    /// import fs from "fs";
    /// ```
    EnforceNodeProtocolUsage,
    import,
    restriction,
    fix,
    config = NodeProtocolUsageMode,
    version = "next",
);

impl Rule for EnforceNodeProtocolUsage {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

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
            AstKind::ExportAllDeclaration(export) => {
                Some((export.source.value, export.source.span))
            }
            _ => return,
        };
        let Some((string_lit_value, span)) = string_lit_value_with_span else {
            return;
        };
        check_node_protocol_path(string_lit_value, span, ctx, self.0.into());
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"import fs from "node:fs";"#, None),
        (r#"import fs from "./fs";"#, None),
        (r#"const fs = require("node:fs/promises");"#, None),
        (r#"const fs = require("unicorn");"#, None),
        (r#"import fs from "node:fs";"#, Some(serde_json::json!(["always"]))),
        (r#"export * from "node:fs";"#, Some(serde_json::json!(["always"]))),
        (r#"import fs from "fs";"#, Some(serde_json::json!(["never"]))),
        (r#"export * from "fs";"#, Some(serde_json::json!(["never"]))),
    ];

    let fail = vec![
        (r#"import fs from "fs";"#, None),
        (r#"export {promises} from "fs";"#, None),
        (r#"export * from "fs";"#, None),
        (r#"const fs = require("fs/promises");"#, None),
        ("async function foo() { const fs = await import('assert/strict'); }", None),
        (r#"import fs from "fs";"#, Some(serde_json::json!(["always"]))),
        (r#"export * as fs from "fs";"#, Some(serde_json::json!(["always"]))),
        (r#"import fs from "node:fs";"#, Some(serde_json::json!(["never"]))),
        (r#"export * from "node:fs";"#, Some(serde_json::json!(["never"]))),
    ];

    let fix = vec![
        (r#"import fs from "fs";"#, r#"import fs from "node:fs";"#, None),
        (
            r#"const fs = require("fs/promises");"#,
            r#"const fs = require("node:fs/promises");"#,
            None,
        ),
        (r#"export * from "fs";"#, r#"export * from "node:fs";"#, None),
        (
            r#"import fs from "node:fs";"#,
            r#"import fs from "fs";"#,
            Some(serde_json::json!(["never"])),
        ),
        (
            r#"export * from "node:fs";"#,
            r#"export * from "fs";"#,
            Some(serde_json::json!(["never"])),
        ),
    ];

    Tester::new(EnforceNodeProtocolUsage::NAME, EnforceNodeProtocolUsage::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
