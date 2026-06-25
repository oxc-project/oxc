use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, CallExpression, Expression, ImportDeclaration,
        ImportDeclarationSpecifier, PropertyKey, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

const BUFFER_MODULES: [&str; 2] = ["buffer", "node:buffer"];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum PreferGlobalBufferMode {
    #[default]
    Always,
    Never,
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct PreferGlobalBuffer(PreferGlobalBufferMode);

fn prefer_global_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(r#"Unexpected use of 'require("buffer").Buffer'. Use the global variable 'Buffer' instead."#)
        .with_help("Use the global `Buffer` variable instead of importing it from the `buffer` module.")
        .with_label(span)
}

fn prefer_module_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Unexpected use of the global variable 'Buffer'. Use 'require(\"buffer\").Buffer' instead.",
    )
    .with_help("Import `Buffer` from the `buffer` module instead of using the global variable.")
    .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces either the global `Buffer` variable or `require("buffer").Buffer`.
    ///
    /// ### Why is this bad?
    ///
    /// The `Buffer` class of the `buffer` module is also available as a global variable in Node.js.
    /// Using one style consistently improves readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `"always"` option:
    /// ```js
    /// const { Buffer } = require("buffer");
    /// const b = Buffer.alloc(16);
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `"always"` option:
    /// ```js
    /// const b = Buffer.alloc(16);
    /// ```
    PreferGlobalBuffer,
    node,
    style,
    config = PreferGlobalBufferMode,
    version = "next",
    short_description = "Enforce either `Buffer` or `require(\"buffer\").Buffer`.",
);

impl Rule for PreferGlobalBuffer {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match self.0 {
            PreferGlobalBufferMode::Always => match node.kind() {
                AstKind::VariableDeclarator(var_decl) => {
                    if let Some(span) = module_buffer_binding_span(var_decl) {
                        ctx.diagnostic(prefer_global_diagnostic(span));
                    }
                }
                AstKind::ImportDeclaration(import_decl) => {
                    if let Some(span) = imported_buffer_span(import_decl) {
                        ctx.diagnostic(prefer_global_diagnostic(span));
                    }
                }
                _ => {}
            },
            PreferGlobalBufferMode::Never => {
                let AstKind::IdentifierReference(ident) = node.kind() else {
                    return;
                };
                if ident.name != "Buffer" || !ident.is_global_reference(ctx.scoping()) {
                    return;
                }
                ctx.diagnostic(prefer_module_diagnostic(ident.span));
            }
        }
    }
}

fn module_buffer_binding_span(var_decl: &VariableDeclarator) -> Option<Span> {
    let init = var_decl.init.as_ref()?;

    if binding_pattern_imports_buffer(&var_decl.id) && is_buffer_module_init(init) {
        return Some(var_decl.span);
    }

    if is_require_buffer_member(init) {
        return Some(init.span());
    }

    None
}

fn binding_pattern_imports_buffer(pattern: &BindingPattern) -> bool {
    let BindingPattern::ObjectPattern(object_pattern) = pattern else {
        return false;
    };

    object_pattern.properties.iter().any(|property| {
        matches!(&property.key, PropertyKey::StaticIdentifier(key) if key.name == "Buffer")
    })
}

fn is_buffer_module_init(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::CallExpression(call) => {
            get_require_module_name(call).is_some_and(is_buffer_module_name)
                || is_process_get_builtin_module_call(call, &BUFFER_MODULES)
        }
        _ => false,
    }
}

fn is_require_buffer_member(expr: &Expression) -> bool {
    let Expression::StaticMemberExpression(member) = expr.get_inner_expression() else {
        return false;
    };

    member.property.name.as_str() == "Buffer"
        && matches!(
            member.object.get_inner_expression(),
            Expression::CallExpression(call)
                if get_require_module_name(call).is_some_and(is_buffer_module_name)
        )
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

fn imported_buffer_span(import_decl: &ImportDeclaration) -> Option<Span> {
    if !is_buffer_module_name(import_decl.source.value.as_str()) {
        return None;
    }

    let specifiers = import_decl.specifiers.as_ref()?;

    specifiers.iter().find_map(|specifier| {
        let ImportDeclarationSpecifier::ImportSpecifier(import_specifier) = specifier else {
            return None;
        };

        if import_specifier.imported.name().as_str() == "Buffer" {
            Some(import_specifier.span)
        } else {
            None
        }
    })
}

fn is_buffer_module_name(name: &str) -> bool {
    BUFFER_MODULES.contains(&name)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var b = Buffer.alloc(10)", None),
        ("var b = Buffer.alloc(10)", Some(serde_json::json!(["always"]))),
        (
            "var { Buffer } = require('buffer'); var b = Buffer.alloc(10)",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var { Buffer } = require('node:buffer'); var b = Buffer.alloc(10)",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var { Buffer } = process.getBuiltinModule('buffer'); var b = Buffer.alloc(10)",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var { Buffer } = process.getBuiltinModule('node:buffer'); var b = Buffer.alloc(10)",
            Some(serde_json::json!(["never"])),
        ),
    ];

    let fail = vec![
        ("var { Buffer } = require('buffer'); var b = Buffer.alloc(10)", None),
        ("var { Buffer } = require('node:buffer'); var b = Buffer.alloc(10)", None),
        (
            "var { Buffer } = require('buffer'); var b = Buffer.alloc(10)",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var { Buffer } = require('node:buffer'); var b = Buffer.alloc(10)",
            Some(serde_json::json!(["always"])),
        ),
        ("var { Buffer } = process.getBuiltinModule('buffer'); var b = Buffer.alloc(10)", None),
        (
            "var { Buffer } = process.getBuiltinModule('node:buffer'); var b = Buffer.alloc(10)",
            None,
        ),
        ("var b = Buffer.alloc(10)", Some(serde_json::json!(["never"]))),
    ];

    Tester::new(PreferGlobalBuffer::NAME, PreferGlobalBuffer::PLUGIN, pass, fail)
        .test_and_snapshot();
}
