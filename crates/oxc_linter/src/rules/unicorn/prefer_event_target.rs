use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{AstNode, context::LintContext, module_record::ImportImportName, rule::Rule};

fn prefer_event_target_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `EventTarget` over `EventEmitter`")
        .with_help("Change `EventEmitter` to `EventTarget`. EventEmitters are only available in Node.js, while EventTargets are also available in browsers.")
        .with_label(span)
}

// Packages that should be ignored because they provide their own EventEmitter
const IGNORED_PACKAGES: &[CompactStr] =
    &[CompactStr::new_const("@angular/core"), CompactStr::new_const("eventemitter3")];
const EVENT_EMITTER: CompactStr = CompactStr::new_const("EventEmitter");

/// Check if EventEmitter is imported from an ignored package (module-scoped check)
fn is_event_emitter_from_ignored_package(ctx: &LintContext) -> bool {
    ctx.module_record().import_entries.iter().any(|import| {
        if !IGNORED_PACKAGES.contains(&import.module_request.name) {
            return false;
        }

        match &import.import_name {
            ImportImportName::Name(name_span) => name_span.name == EVENT_EMITTER,
            ImportImportName::Default(_) => import.local_name.name == EVENT_EMITTER,
            ImportImportName::NamespaceObject => false,
        }
    })
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
    /// Note: EventEmitter imported from packages like `@angular/core` or `eventemitter3` are allowed, as they provide their own implementation.
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
    ///
    /// // EventEmitter from ignored packages is allowed
    /// import { EventEmitter } from "@angular/core";
    /// class Foo extends EventEmitter {}
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

                if !std::ptr::eq(ident, callee_ident.as_ref()) {
                    return;
                }
            }
            _ => return,
        }

        // Check if EventEmitter is from an ES6 import from an ignored package
        if is_event_emitter_from_ignored_package(ctx) {
            return;
        }

        ctx.diagnostic(prefer_event_target_diagnostic(ident.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"class Foo {}",
        r"class Foo extends OtherClass {}",
        r"class Foo extends EventTarget {}",
        r"const Foo = class extends EventTarget {}",
        r"const Foo = class extends foo.EventTarget {}",
        r"const Foo = class extends foo.bar.EventTarget {}",
        r"class Foo extends foo.EventEmitter {}",
        r"class Foo extends foo.bar.EventEmitter {}",
        r"class EventEmitter extends Foo {}",
        r"const Foo = class EventEmitter extends Foo {}",
        r"new Foo(EventEmitter)",
        r"new foo.EventEmitter()",
        r"EventTarget()",
        r"new EventTarget",
        r"const target = new EventTarget;",
        r"const target = EventTarget()",
        r"const target = new Foo(EventEmitter);",
        r"EventEmitter()",
        r"const emitter = EventEmitter()",
        // EventEmitter from ignored packages should be allowed - ES6 imports
        r#"import { EventEmitter } from "@angular/core";
           class Foo extends EventEmitter {}"#,
        r#"import { EventEmitter } from "eventemitter3";
           class Foo extends EventEmitter {}"#,
        // TODO: CommonJS require does not work yet.
        // r#"const { EventEmitter } = require("@angular/core");
        // class Foo extends EventEmitter {}"#,
    ];

    let fail = vec![
        r"class Foo extends EventEmitter {}",
        r"class Foo extends EventEmitter { someMethod() {} }",
        r"const Foo = class extends EventEmitter {}",
        r"new EventEmitter",
        r"const emitter = new EventEmitter;",
        r"for (const {EventEmitter} of []) {new EventEmitter}",
        r"for (const EventEmitter of []) {new EventEmitter}",
        // From a random package, it should fail.
        r#"import { EventEmitter } from "foobar";
           class Foo extends EventEmitter {}"#,
    ];

    Tester::new(PreferEventTarget::NAME, PreferEventTarget::PLUGIN, pass, fail).test_and_snapshot();
}
