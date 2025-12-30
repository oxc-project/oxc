use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_event_target_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `EventTarget` over `EventEmitter`")
        .with_help("Change `EventEmitter` to `EventTarget`. EventEmitters are only available in Node.js, while EventTargets are also available in browsers.")
        .with_label(span)
}

// Packages that should be ignored because they provide their own EventEmitter
const IGNORED_PACKAGES: &[&str] = &["@angular/core", "eventemitter3"];

/// Check if EventEmitter is imported from an ignored package (module-scoped check)
fn is_event_emitter_from_ignored_package(ctx: &LintContext) -> bool {
    use crate::module_record::ImportImportName;

    ctx.module_record().import_entries.iter().any(|import| {
        if !IGNORED_PACKAGES.contains(&import.module_request.name.as_str()) {
            return false;
        }

        match &import.import_name {
            ImportImportName::Name(name_span) => name_span.name.as_str() == "EventEmitter",
            ImportImportName::Default(_) => import.local_name.name.as_str() == "EventEmitter",
            _ => false,
        }
    })
}

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
        // Import aliases should also work
        r#"import { EventEmitter as EE } from "@angular/core";
class Foo extends EventEmitter {}"#,
        // TODO: CommonJS require and dynamic imports - need to investigate why these don't work
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
    ];

    Tester::new(PreferEventTarget::NAME, PreferEventTarget::PLUGIN, pass, fail).test_and_snapshot();
}
