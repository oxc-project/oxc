use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-event-target): Prefer `EventTarget` over `EventEmitter`")]
#[diagnostic(severity(warning), help("Change `EventEmitter` to `EventTarget`. EventEmitters are only available in Node.js, while EventTargets are also available in browsers."))]
struct PreferEventTargetDiagnostic(#[label] pub Span);

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
    /// ### Example
    /// ```javascript
    /// // Bad
    /// class Foo extends EventEmitter {}
    ///
    /// // Good
    /// class Foo extends OtherClass {}
    /// ```
    PreferEventTarget,
    pedantic
);

impl Rule for PreferEventTarget {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IdentifierReference(ident) = node.kind() else { return };

        if ident.name.as_str() != "EventEmitter" {
            return;
        }

        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        match parent.kind() {
            AstKind::ClassHeritage(_) => {}
            AstKind::NewExpression(new_expr) => {
                let Expression::Identifier(callee_ident) = &new_expr.callee else {
                    return;
                };

                if ident as *const _ != callee_ident.0 as *const _ {
                    return;
                }
            }
            _ => return,
        };

        ctx.diagnostic(PreferEventTargetDiagnostic(ident.span));
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

    Tester::new_without_config(PreferEventTarget::NAME, pass, fail).test_and_snapshot();
}
