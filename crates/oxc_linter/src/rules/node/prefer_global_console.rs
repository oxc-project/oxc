use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier,
        VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

const CONSOLE_MODULES: [&str; 2] = ["console", "node:console"];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum PreferGlobalConsoleMode {
    #[default]
    Always,
    Never,
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct PreferGlobalConsole(PreferGlobalConsoleMode);

fn prefer_global_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        r#"Unexpected use of 'require("console")'. Use the global variable 'console' instead."#,
    )
    .with_help(
        "Use the global `console` variable instead of importing it from the `console` module.",
    )
    .with_label(span)
}

fn prefer_module_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Unexpected use of the global variable 'console'. Use 'require(\"console\")' instead.",
    )
    .with_help("Import `console` from the `console` module instead of using the global variable.")
    .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces either the global `console` variable or `require("console")`.
    ///
    /// ### Why is this bad?
    ///
    /// The `console` module is also available as a global variable in Node.js.
    /// Using one style consistently improves readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `"always"` option:
    /// ```js
    /// const console = require("console");
    /// console.log("hello");
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `"always"` option:
    /// ```js
    /// console.log("hello");
    /// ```
    PreferGlobalConsole,
    node,
    style,
    config = PreferGlobalConsoleMode,
    version = "next",
    short_description = "Enforce either `console` or `require(\"console\")`.",
);

impl Rule for PreferGlobalConsole {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match self.0 {
            PreferGlobalConsoleMode::Always => match node.kind() {
                AstKind::VariableDeclarator(var_decl) => {
                    if let Some(span) = module_console_binding_span(var_decl) {
                        ctx.diagnostic(prefer_global_diagnostic(span));
                    }
                }
                AstKind::ImportDeclaration(import_decl) => {
                    if let Some(span) = imported_console_span(import_decl) {
                        ctx.diagnostic(prefer_global_diagnostic(span));
                    }
                }
                _ => {}
            },
            PreferGlobalConsoleMode::Never => {
                let AstKind::IdentifierReference(ident) = node.kind() else {
                    return;
                };
                if ident.name != "console" || !ident.is_global_reference(ctx.scoping()) {
                    return;
                }
                ctx.diagnostic(prefer_module_diagnostic(ident.span));
            }
        }
    }
}

fn module_console_binding_span(var_decl: &VariableDeclarator) -> Option<Span> {
    let init = var_decl.init.as_ref()?;

    if is_console_module_init(init) {
        return Some(var_decl.span);
    }

    None
}

fn is_console_module_init(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::CallExpression(call) => {
            get_require_module_name(call).is_some_and(is_console_module_name)
                || is_process_get_builtin_module_call(call, &CONSOLE_MODULES)
        }
        _ => false,
    }
}

fn get_require_module_name<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    if !call.callee.is_specific_id("require") || call.arguments.len() != 1 {
        return None;
    }

    match &call.arguments[0] {
        Argument::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => None,
    }
}

fn is_process_get_builtin_module_call(call: &CallExpression, modules: &[&str]) -> bool {
    if call.optional || call.arguments.len() != 1 || call.arguments[0].is_spread() {
        return false;
    }

    let Expression::StaticMemberExpression(callee_member) = &call.callee else {
        return false;
    };

    if callee_member.optional {
        return false;
    }

    let Expression::Identifier(process_ident) = &callee_member.object else {
        return false;
    };

    if process_ident.name != "process" || callee_member.property.name != "getBuiltinModule" {
        return false;
    }

    matches!(
        &call.arguments[0],
        Argument::StringLiteral(lit) if modules.contains(&lit.value.as_str())
    )
}

fn imported_console_span(import_decl: &ImportDeclaration) -> Option<Span> {
    if !is_console_module_name(import_decl.source.value.as_str()) {
        return None;
    }

    let specifiers = import_decl.specifiers.as_ref()?;

    specifiers.iter().find_map(|specifier| match specifier {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(default_specifier) => {
            Some(default_specifier.span)
        }
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace_specifier) => {
            Some(namespace_specifier.span)
        }
        ImportDeclarationSpecifier::ImportSpecifier(_) => None,
    })
}

fn is_console_module_name(name: &str) -> bool {
    CONSOLE_MODULES.contains(&name)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("console.log(10)", None),
        ("console.log(10)", Some(serde_json::json!(["always"]))),
        ("var console = require('console'); console.log(10)", Some(serde_json::json!(["never"]))),
        (
            "var console = require('node:console'); console.log(10)",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var console = process.getBuiltinModule('console'); console.log(10)",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var console = process.getBuiltinModule('node:console'); console.log(10)",
            Some(serde_json::json!(["never"])),
        ),
    ];

    let fail = vec![
        ("var console = require('console'); console.log(10)", None),
        ("var console = require('node:console'); console.log(10)", None),
        ("var console = require('console'); console.log(10)", Some(serde_json::json!(["always"]))),
        (
            "var console = require('node:console'); console.log(10)",
            Some(serde_json::json!(["always"])),
        ),
        ("var console = process.getBuiltinModule('console'); console.log(10)", None),
        ("var console = process.getBuiltinModule('node:console'); console.log(10)", None),
        ("console.log(10)", Some(serde_json::json!(["never"]))),
    ];

    Tester::new(PreferGlobalConsole::NAME, PreferGlobalConsole::PLUGIN, pass, fail)
        .test_and_snapshot();
}
