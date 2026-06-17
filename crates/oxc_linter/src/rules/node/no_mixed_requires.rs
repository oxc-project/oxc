use oxc_ast::{
    AstKind,
    ast::{Expression, VariableDeclarator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{MixedTupleRuleConfig, Rule},
};

fn no_mix_require_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not mix 'require' and other declarations.").with_label(span)
}

fn no_mix_core_module_file_computed_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not mix core, module, file and computed requires.").with_label(span)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclarationType {
    Require,
    Uninitialized,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModuleType {
    Core,
    File,
    Module,
    Computed,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
struct NoMixedRequiresOptions {
    grouping: bool,
    allow_call: bool,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
pub struct NoMixedRequires(MixedTupleRuleConfig<bool, NoMixedRequiresOptions>);

// This list is generated using `require("module").builtinModules` and was last updated using Node v13.8.0.
// See <https://github.com/eslint-community/eslint-plugin-n/blob/master/lib/rules/no-mixed-requires.js>.
const BUILTIN_MODULES: &[&str] = &[
    "_http_agent",
    "_http_client",
    "_http_common",
    "_http_incoming",
    "_http_outgoing",
    "_http_server",
    "_stream_duplex",
    "_stream_passthrough",
    "_stream_readable",
    "_stream_transform",
    "_stream_wrap",
    "_stream_writable",
    "_tls_common",
    "_tls_wrap",
    "assert",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "string_decoder",
    "sys",
    "timers",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "worker_threads",
    "zlib",
];

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows `require` calls to be mixed with regular variable declarations.
    ///
    /// ### Why is this bad?
    ///
    /// In the Node.js community it is often customary to separate initializations with calls to
    /// `require` modules from other variable declarations, sometimes also grouping them by the type
    /// of module.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var fs = require('fs'),
    ///     i = 0;
    ///
    /// var async = require('async'),
    ///     debug = require('diagnostics').someFunction('my-module'),
    ///     eslint = require('eslint');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var eventEmitter = require('events').EventEmitter,
    ///     myUtils = require('./utils'),
    ///     util = require('util'),
    ///     bar = require(getBarModuleName());
    ///
    /// var foo = 42,
    ///     bar = 'baz';
    /// ```
    NoMixedRequires,
    node,
    style,
    config = NoMixedRequires,
    version = "next",
    short_description = "Disallow `require` calls to be mixed with regular variable declarations.",
);

impl Rule for NoMixedRequires {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<NoMixedRequires>(value)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(var_decl) = node.kind() else {
            return;
        };

        let grouping = if self.0.0 { true } else { self.0.1.grouping };
        let allow_call = self.0.1.allow_call;

        if is_mixed(&var_decl.declarations, allow_call) {
            ctx.diagnostic(no_mix_require_diagnostic(var_decl.span));
            return;
        }

        if grouping && !is_grouped(&var_decl.declarations, allow_call) {
            ctx.diagnostic(no_mix_core_module_file_computed_diagnostic(var_decl.span));
        }
    }
}

fn get_declaration_type(init: Option<&Expression>, allow_call: bool) -> DeclarationType {
    let Some(init) = init else {
        return DeclarationType::Uninitialized;
    };

    if is_require_call(init, allow_call) {
        return DeclarationType::Require;
    }

    if allow_call
        && let Expression::CallExpression(call) = init
        && call.callee.is_call_expression()
    {
        return get_declaration_type(Some(&call.callee), allow_call);
    }

    if let Some(member) = init.as_member_expression() {
        return get_declaration_type(Some(&member.object()), allow_call);
    }

    DeclarationType::Other
}

fn infer_module_type(init: &Expression) -> ModuleType {
    if let Some(member) = init.as_member_expression() {
        return infer_module_type(&member.object());
    }

    let Expression::CallExpression(call) = init else {
        return ModuleType::Module;
    };

    if call.arguments.is_empty() {
        return ModuleType::Computed;
    }

    let Some(arg) = call.arguments.first() else {
        return ModuleType::Computed;
    };

    let Some(value) = arg.as_expression().and_then(|expr| match expr {
        Expression::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => None,
    }) else {
        return ModuleType::Computed;
    };

    if BUILTIN_MODULES.contains(&value) {
        return ModuleType::Core;
    }

    if value.starts_with("./") || value.starts_with("../") || value.starts_with("/") {
        return ModuleType::File;
    }

    ModuleType::Module
}

fn is_mixed(declarations: &[VariableDeclarator], allow_call: bool) -> bool {
    let mut has_require = false;
    let mut has_uninitialized = false;
    let mut has_other = false;

    for declaration in declarations {
        match get_declaration_type(declaration.init.as_ref(), allow_call) {
            DeclarationType::Require => has_require = true,
            DeclarationType::Uninitialized => has_uninitialized = true,
            DeclarationType::Other => has_other = true,
        }
    }

    has_require && (has_uninitialized || has_other)
}

fn is_grouped(declarations: &[VariableDeclarator], allow_call: bool) -> bool {
    let mut found = [false, false, false, false];

    for declaration in declarations {
        if let Some(init) = declaration.init.as_ref()
            && get_declaration_type(Some(init), allow_call) == DeclarationType::Require
        {
            match infer_module_type(init) {
                ModuleType::Core => found[0] = true,
                ModuleType::File => found[1] = true,
                ModuleType::Module => found[2] = true,
                ModuleType::Computed => found[3] = true,
            }
        }
    }

    found.iter().filter(|&&found| found).count() <= 1
}

fn is_require_call(expr: &Expression, allow_call: bool) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };

    if call.callee.is_specific_id("require") {
        return true;
    }

    if allow_call && call.callee.is_call_expression() {
        return is_require_call(&call.callee, allow_call);
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var a, b = 42, c = doStuff()", Some(serde_json::json!([false]))),
        (
            "var a = require(42), b = require(), c = require('y'), d = require(doStuff())",
            Some(serde_json::json!([false])),
        ),
        ("var fs = require('fs'), foo = require('foo')", Some(serde_json::json!([false]))),
        (
            "var exec = require('child_process').exec, foo = require('foo')",
            Some(serde_json::json!([false])),
        ),
        ("var fs = require('fs'), foo = require('./foo')", Some(serde_json::json!([false]))),
        ("var foo = require('foo'), foo2 = require('./foo')", Some(serde_json::json!([false]))),
        (
            "var emitter = require('events').EventEmitter, fs = require('fs')",
            Some(serde_json::json!([false])),
        ),
        ("var foo = require(42), bar = require(getName())", Some(serde_json::json!([false]))),
        ("var foo = require(42), bar = require(getName())", Some(serde_json::json!([true]))),
        (
            "var fs = require('fs'), foo = require('./foo')",
            Some(serde_json::json!([{ "grouping": false }])),
        ),
        ("var foo = require('foo'), bar = require(getName())", Some(serde_json::json!([false]))),
        ("var a;", Some(serde_json::json!([true]))),
        (
            "var async = require('async'), debug = require('diagnostics')('my-module')",
            Some(serde_json::json!([{ "allowCall": true }])),
        ),
    ];

    let fail = vec![
        ("var fs = require('fs'), foo = 42", Some(serde_json::json!([false]))),
        ("var fs = require('fs'), foo", Some(serde_json::json!([false]))),
        (
            "var a = require(42), b = require(), c = require('y'), d = require(doStuff())",
            Some(serde_json::json!([true])),
        ),
        ("var fs = require('fs'), foo = require('foo')", Some(serde_json::json!([true]))),
        (
            "var fs = require('fs'), foo = require('foo')",
            Some(serde_json::json!([{ "grouping": true }])),
        ),
        (
            "var exec = require('child_process').exec, foo = require('foo')",
            Some(serde_json::json!([true])),
        ),
        ("var fs = require('fs'), foo = require('./foo')", Some(serde_json::json!([true]))),
        ("var foo = require('foo'), foo2 = require('./foo')", Some(serde_json::json!([true]))),
        ("var foo = require('foo'), bar = require(getName())", Some(serde_json::json!([true]))),
        (
            "var async = require('async'), debug = require('diagnostics').someFun('my-module')",
            Some(serde_json::json!([{ "allowCall": true }])),
        ),
    ];

    Tester::new(NoMixedRequires::NAME, NoMixedRequires::PLUGIN, pass, fail).test_and_snapshot();
}
