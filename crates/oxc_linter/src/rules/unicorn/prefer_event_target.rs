use oxc_allocator::{GetAddress, UnstableAddress};
use oxc_ast::{
    AstKind,
    ast::{Argument, BindingPattern, Expression, IdentifierReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode, ast_util::get_declaration_of_variable, context::LintContext, rule::Rule,
    utils::is_import_from_module,
};

const IGNORED_PACKAGES: [&str; 2] = ["@angular/core", "eventemitter3"];

fn prefer_event_target_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `EventTarget` over `EventEmitter`")
        .with_help("Change `EventEmitter` to `EventTarget`. EventEmitters are only available in Node.js, while EventTargets are also available in browsers.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferEventTarget;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers `EventTarget` over `EventEmitter`.
    ///
    /// This rule reduces the bundle size and makes your code more cross-platform friendly.
    ///
    /// See the [differences](https://nodejs.org/api/events.html#eventtarget-and-event-api) between `EventEmitter` and `EventTarget`.
    ///
    /// ### Why is this bad?
    ///
    /// While [`EventEmitter`](https://nodejs.org/api/events.html#class-eventemitter) is only available in Node.js, [`EventTarget`](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget) is also available in _Deno_ and browsers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class Foo extends EventEmitter {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// class Foo extends OtherClass {}
    /// ```
    PreferEventTarget,
    unicorn,
    pedantic
);

impl Rule for PreferEventTarget {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IdentifierReference(ident) = node.kind() else {
            return;
        };

        if ident.name.as_str() != "EventEmitter" {
            return;
        }

        match ctx.nodes().parent_kind(node.id()) {
            AstKind::Class(_) => {}
            AstKind::NewExpression(new_expr) => {
                let Expression::Identifier(callee_ident) = &new_expr.callee else {
                    return;
                };

                if ident.unstable_address() != callee_ident.address() {
                    return;
                }
            }
            _ => return,
        }

        if is_event_emitter_from_ignored_package(ident, ctx) {
            return;
        }

        ctx.diagnostic(prefer_event_target_diagnostic(ident.span));
    }
}

fn is_ignored_package(source: &str) -> bool {
    IGNORED_PACKAGES.contains(&source)
}

fn is_await_import_or_require_from_ignored_packages(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::CallExpression(call_expr) => {
            !call_expr.optional
                && call_expr.callee.is_specific_id("require")
                && call_expr.arguments.len() == 1
                && match &call_expr.arguments[0] {
                    Argument::StringLiteral(source) => is_ignored_package(source.value.as_str()),
                    Argument::TemplateLiteral(source) => source
                        .single_quasi()
                        .is_some_and(|source| is_ignored_package(source.as_str())),
                    _ => false,
                }
        }
        Expression::AwaitExpression(await_expr) => match await_expr.argument.get_inner_expression()
        {
            Expression::ImportExpression(import_expr) => {
                match import_expr.source.get_inner_expression() {
                    Expression::StringLiteral(source) => is_ignored_package(source.value.as_str()),
                    Expression::TemplateLiteral(source) => source
                        .single_quasi()
                        .is_some_and(|source| is_ignored_package(source.as_str())),
                    _ => false,
                }
            }
            _ => false,
        },
        _ => false,
    }
}

fn is_event_emitter_member_access_from_ignored_packages(expr: &Expression) -> bool {
    let Some(member_expr) = expr.get_inner_expression().as_member_expression() else {
        return false;
    };

    !member_expr.optional()
        && !member_expr.is_computed()
        && member_expr.static_property_name() == Some("EventEmitter")
        && is_await_import_or_require_from_ignored_packages(member_expr.object())
}

fn is_event_emitter_from_ignored_package<'a>(
    ident: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    if IGNORED_PACKAGES.iter().any(|package_name| is_import_from_module(ident, package_name, ctx)) {
        return true;
    }

    let Some(declaration_node) = get_declaration_of_variable(ident, ctx) else {
        return false;
    };

    let AstKind::VariableDeclarator(var_decl) = declaration_node.kind() else {
        return false;
    };

    if let BindingPattern::ObjectPattern(object_pattern) = &var_decl.id
        && object_pattern.properties.iter().any(|property| {
            property.key.is_specific_static_name("EventEmitter")
                && property
                    .value
                    .get_identifier_name()
                    .is_some_and(|name| name.as_str() == "EventEmitter")
        })
        && var_decl.init.as_ref().is_some_and(is_await_import_or_require_from_ignored_packages)
    {
        return true;
    }

    if var_decl.id.get_identifier_name().is_some_and(|name| name.as_str() == "EventEmitter")
        && let Some(init) = &var_decl.init
        && is_event_emitter_member_access_from_ignored_packages(init)
    {
        return true;
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class Foo {}",
        "class Foo extends OtherClass {}",
        "class Foo extends EventTarget {}",
        "const Foo = class extends EventTarget {}",
        "const Foo = class extends foo.EventTarget {}",
        "const Foo = class extends foo.bar.EventTarget {}",
        "class Foo extends foo.EventEmitter {}",
        "class Foo extends foo.bar.EventEmitter {}",
        "class EventEmitter extends Foo {}",
        "const Foo = class EventEmitter extends Foo {}",
        "new Foo(EventEmitter)",
        "new foo.EventEmitter()",
        r#"import { EventEmitter } from "@angular/core"; class Foo extends EventEmitter {}"#,
        r#"const { EventEmitter } = require("@angular/core"); class Foo extends EventEmitter {}"#,
        r#"let { EventEmitter } = require("@angular/core"); class Foo extends EventEmitter {}"#,
        r#"const EventEmitter = require("@angular/core").EventEmitter; class Foo extends EventEmitter {}"#,
        r#"var EventEmitter = require("eventemitter3").EventEmitter; class Foo extends EventEmitter {}"#,
        r#"import EventEmitter from "eventemitter3"; class Foo extends EventEmitter {}"#,
        r#"import { EventEmitter } from "eventemitter3"; class Foo extends EventEmitter {}"#,
        r#"async function f() { const { EventEmitter } = await import("eventemitter3"); class Foo extends EventEmitter {} }"#,
        r"async function f() { const { EventEmitter } = await import(`eventemitter3`); class Foo extends EventEmitter {} }",
        r#"async function f() { const EventEmitter = (await import("eventemitter3")).EventEmitter; class Foo extends EventEmitter {} }"#,
        r"async function f() { const EventEmitter = (await import(`@angular/core`)).EventEmitter; class Foo extends EventEmitter {} }",
        "EventTarget()",
        "new EventTarget",
        "const target = new EventTarget;",
        "const target = EventTarget()",
        "const target = new Foo(EventEmitter);",
        "EventEmitter()",
        "const emitter = EventEmitter()",
    ];

    let fail = vec![
        "class Foo extends EventEmitter {}",
        "class Foo extends EventEmitter { someMethod() {} }",
        "const Foo = class extends EventEmitter {}",
        "class Foo extends EventEmitter {
				addListener() {}
				removeListener() {}
			}",
        "new EventEmitter",
        "const emitter = new EventEmitter;",
        "for (const {EventEmitter} of []) {new EventEmitter}",
        "for (const EventEmitter of []) {new EventEmitter}",
    ];

    Tester::new(PreferEventTarget::NAME, PreferEventTarget::PLUGIN, pass, fail).test_and_snapshot();
}
