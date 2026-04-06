use oxc_ast::{
    AstKind,
    ast::{ExportDefaultDeclarationKind, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn avoid_barrel_files_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Avoid barrel files, they slow down performance, and cause large module graphs with modules that go unused.",
    )
    .with_help("Prefer importing and re-exporting only the concrete symbols you actually need.")
    .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct AvoidBarrelFiles(Box<AvoidBarrelFilesConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct AvoidBarrelFilesConfig {
    amount_of_exports_to_consider_module_as_barrel: usize,
}

impl Default for AvoidBarrelFilesConfig {
    fn default() -> Self {
        Self { amount_of_exports_to_consider_module_as_barrel: 3 }
    }
}

impl std::ops::Deref for AvoidBarrelFiles {
    type Target = AvoidBarrelFilesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects files that mostly exist to re-export other modules.
    ///
    /// ### Why is this bad?
    ///
    /// Barrel files tend to widen module graphs and force runtimes or bundlers
    /// to load more code than consumers actually need.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export { foo } from "./foo";
    /// export { bar } from "./bar";
    /// export { baz } from "./baz";
    /// export { qux } from "./qux";
    /// ```
    AvoidBarrelFiles,
    oxc,
    restriction,
    none,
    config = AvoidBarrelFilesConfig
);

impl Rule for AvoidBarrelFiles {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let Some(program) = ctx.semantic().nodes().iter().find_map(|node| match node.kind() {
            AstKind::Program(program) => Some(program),
            _ => None,
        }) else {
            return;
        };

        let mut declarations = 0usize;
        let mut exports = 0usize;

        for statement in &program.body {
            count_statement(statement, &mut declarations, &mut exports);
        }

        let threshold = self.amount_of_exports_to_consider_module_as_barrel;
        if exports > declarations && exports > threshold {
            let span = program.span;
            ctx.diagnostic(avoid_barrel_files_diagnostic(span));
        }
    }
}

fn count_statement(statement: &Statement<'_>, declarations: &mut usize, exports: &mut usize) {
    match statement {
        Statement::VariableDeclaration(variable_decl) => {
            *declarations += variable_decl.declarations.len();
        }
        Statement::FunctionDeclaration(_) | Statement::ClassDeclaration(_) => {
            *declarations += 1;
        }
        Statement::TSTypeAliasDeclaration(_) | Statement::TSInterfaceDeclaration(_) => {
            *declarations += 1;
        }
        Statement::ExportNamedDeclaration(export_named) => {
            *exports += export_named.specifiers.len();
        }
        Statement::ExportAllDeclaration(export_all) => {
            if !export_all.is_typescript_syntax() {
                *exports += 1;
            }
        }
        Statement::ExportDefaultDeclaration(export_default) => match &export_default.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(_) => {
                *declarations += 1;
            }
            ExportDefaultDeclarationKind::CallExpression(_) => {
                *declarations += 1;
            }
            ExportDefaultDeclarationKind::ObjectExpression(object_expr) => {
                *exports += object_expr.properties.len();
            }
            _ => {
                *exports += 1;
            }
        },
        _ => {}
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (
            r#"
            let foo;
            export { foo };
            "#,
            None,
        ),
        (
            r#"
            let foo, bar;
            export { foo, bar };
            "#,
            None,
        ),
        (
            r#"
            let foo, bar, baz;
            export { foo, bar, baz };
            "#,
            None,
        ),
        (
            r#"
            export default function Foo() {
              return "bar";
            }
            "#,
            Some(json!([{ "amountOfExportsToConsiderModuleAsBarrel": 0 }])),
        ),
        (
            r#"
            export default defineFoo({});
            "#,
            Some(json!([{ "amountOfExportsToConsiderModuleAsBarrel": 0 }])),
        ),
    ];

    let fail = vec![
        (
            r#"
            import { bar, baz, qux } from "foo";
            let foo;
            export { foo, bar, baz, qux };
            "#,
            None,
        ),
        (
            r#"
            export * from "foo";
            export * from "bar";
            export * from "baz";
            export * from "qux";
            "#,
            None,
        ),
        (
            r#"export { foo, bar, baz } from "foo";"#,
            Some(json!([{ "amountOfExportsToConsiderModuleAsBarrel": 2 }])),
        ),
        (r#"export default { var1, var2, var3, var4 };"#, None),
    ];

    Tester::new(AvoidBarrelFiles::NAME, AvoidBarrelFiles::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
