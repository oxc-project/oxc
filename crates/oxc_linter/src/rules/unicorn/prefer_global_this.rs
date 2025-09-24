use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_global_this_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `globalThis` over environment-specific global aliases like `window`, `self`, and `global`.")
        .with_help("Replace the alias with `globalThis`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferGlobalThis;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of [`globalThis`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/globalThis) instead of
    /// environment‑specific global object aliases (`window`, `self`, or `global`).
    /// Using the standard `globalThis` makes your code portable across browsers, Web Workers, Node.js,
    /// and future JavaScript runtimes.
    ///
    /// ### Why is this bad?
    ///
    /// • **Portability** – `window` is only defined in browser main threads, `self` is used in Web Workers,
    /// and `global` is Node‑specific.  Choosing the wrong alias causes runtime crashes when the code is
    /// executed outside of its original environment.
    /// • **Clarity** – `globalThis` clearly communicates that you are referring to the global object itself
    /// rather than a particular platform.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // Browser‑only
    /// window.alert("Hi");
    ///
    /// // Node‑only
    /// if (typeof global.Buffer !== "undefined") {}
    ///
    /// // Web Worker‑only
    /// self.postMessage("done");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// globalThis.alert("Hi");
    ///
    /// if (typeof globalThis.Buffer !== "undefined") {}
    ///
    /// globalThis.postMessage("done");
    /// ```
    PreferGlobalThis,
    unicorn,
    style,
    pending
);

impl Rule for PreferGlobalThis {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IdentifierReference(ident) = node.kind() else { return };

        if !matches!(ident.name.as_str(), "window" | "self" | "global")
            || is_computed_member_expression_object(node, ctx)
            || !ctx.scoping().root_unresolved_references().contains_key(&ident.name.as_str())
        {
            return;
        }

        if let AstKind::StaticMemberExpression(e) = ctx.nodes().parent_kind(node.id())
            && let Expression::Identifier(ident) = &e.object
        {
            if ident.name == "self" && WEB_WORKER_SPECIFIC_APIS.contains(&e.property.name.as_str())
            {
                return;
            }

            if ident.name == "window" && WINDOW_SPECIFIC_APIS.contains(&e.property.name.as_str()) {
                if matches!(
                    e.property.name.as_str(),
                    "addEventListener" | "removeEventListener" | "dispatchEvent"
                ) {
                    if let Some(AstKind::CallExpression(call_expr)) =
                        ctx.nodes().ancestor_kinds(node.id()).nth(1)
                    {
                        if let Some(Expression::StringLiteral(lit)) =
                            call_expr.arguments.first().and_then(|arg| arg.as_expression())
                            && WINDOW_SPECIFIC_EVENTS.contains(&lit.value.as_str())
                        {
                            return;
                        }
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
        }

        ctx.diagnostic(prefer_global_this_diagnostic(ident.span));
    }
}

/// `window[foo]`, `self[bar]`, etc. are allowed.
fn is_computed_member_expression_object(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let AstKind::ComputedMemberExpression(member_expr) = ctx.nodes().parent_kind(node.id()) else {
        return false;
    };
    let Expression::Identifier(obj_ident) = &member_expr.object.get_inner_expression() else {
        return false;
    };
    obj_ident.span == node.kind().span()
}

const WEB_WORKER_SPECIFIC_APIS: &[&str] = &[
    // https://html.spec.whatwg.org/multipage/workers.html#the-workerglobalscope-common-interface
    "addEventListener",
    "removeEventListener",
    "dispatchEvent",
    "self",
    "location",
    "navigator",
    "onerror",
    "onlanguagechange",
    "onoffline",
    "ononline",
    "onrejectionhandled",
    "onunhandledrejection",
    // https://html.spec.whatwg.org/multipage/workers.html#dedicated-workers-and-the-dedicatedworkerglobalscope-interface
    "name",
    "postMessage",
    "onconnect",
];

const WINDOW_SPECIFIC_APIS: &[&str] = &[
    // Properties and methods
    // https://html.spec.whatwg.org/multipage/nav-history-apis.html#the-window-object
    "name",
    "locationbar",
    "menubar",
    "personalbar",
    "scrollbars",
    "statusbar",
    "toolbar",
    "status",
    "close",
    "closed",
    "stop",
    "focus",
    "blur",
    "frames",
    "length",
    "top",
    "opener",
    "parent",
    "frameElement",
    "open",
    "originAgentCluster",
    "postMessage",
    // Events commonly associated with "window"
    "onresize",
    "onblur",
    "onfocus",
    "onload",
    "onscroll",
    "onscrollend",
    "onwheel",
    "onbeforeunload",
    "onmessage",
    "onmessageerror",
    "onpagehide",
    "onpagereveal",
    "onpageshow",
    "onpageswap",
    "onunload",
    // To add/remove/dispatch events that are commonly associated with "window"
    // https://www.w3.org/TR/DOM-Level-2-Events/events.html#Events-flow
    "addEventListener",
    "removeEventListener",
    "dispatchEvent",
    // https://dom.spec.whatwg.org/#idl-index
    "event", // Deprecated and quirky, best left untouched
    // https://drafts.csswg.org/cssom-view/#idl-index
    "screen",
    "visualViewport",
    "moveTo",
    "moveBy",
    "resizeTo",
    "resizeBy",
    "innerWidth",
    "innerHeight",
    "outerWidth",
    "outerHeight",
    "scrollX",
    "pageXOffset",
    "scrollY",
    "pageYOffset",
    "scroll",
    "scrollTo",
    "scrollBy",
    "screenX",
    "screenLeft",
    "screenY",
    "screenTop",
    "screenWidth",
    "screenHeight",
    "devicePixelRatio",
];

//  Allow `on<event>` where <event> is in the windowSpecificEvents set from reference implementation.
const WINDOW_SPECIFIC_EVENTS: &[&str] = &[
    "resize",
    "blur",
    "focus",
    "load",
    "scroll",
    "scrollend",
    "wheel",
    "beforeunload",
    "message",
    "messageerror",
    "pagehide",
    "pagereveal",
    "pageshow",
    "pageswap",
    "unload",
];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "globalThis",
        "globalThis.foo",
        "globalThis[foo]",
        "globalThis.foo()",
        "const { foo } = globalThis",
        "function foo (window) {}",
        "function foo (global) {}",
        "var foo = function foo (window) {}",
        "var foo = function foo (global) {}",
        "var window = {}",
        "let global = {}",
        "const global = {}",
        "function foo (window) {
				window.foo();
			}",
        "var window = {};
			function foo () {
				window.foo();
			}",
        "foo.window",
        "foo.global",
        r#"import window from "xxx""#,
        r#"import * as window from "xxx""#,
        r#"import window, {foo} from "xxx""#,
        r#"export { window }  from "xxx""#,
        r#"export * as window from "xxx";"#,
        "try {
			} catch (window) {}",
        r#"window.name = "foo""#,
        "window.addEventListener",
        "window.innerWidth",
        "window.innerHeight",
        "self.location",
        "self.navigator",
        r#"window.addEventListener("resize", () => {})"#,
        "window.onresize = function () {}",
        "const {window} = jsdom()
			window.jQuery = jQuery;",
        "({ foo: window.name } =  {})",
        "[window.name] = []",
        "window[foo]",
        "window[title]",
        r#"window["foo"]"#,
    ];

    let fail = vec![
        "global",
        "self",
        "window",
        "window.foo",
        "window.foo()",
        "window > 10",
        "10 > window",
        "window ?? 10",
        "10 ?? window",
        "window.foo = 123",
        "window = 123",
        "obj.a = window",
        "function* gen() {
			  yield window
			}",
        "async function gen() {
			  await window
			}",
        "window ? foo : bar",
        "foo ? window : bar",
        "foo ? bar : window",
        "function foo() {
			  return window
			}",
        "new window()",
        "const obj = {
				foo: window.foo,
				bar: window.bar,
				window: window
			}",
        "function sequenceTest() {
				let x, y;
				x = (y = 10, y + 5, window);
				console.log(x, y);
			}",
        "window`Hello ${42} World`",
        "tag`Hello ${window.foo} World`",
        "var str = `hello ${window.foo} world!`",
        "delete window.foo",
        "++window",
        "++window.foo",
        "for (var attr in window) {
			}",
        "for (window.foo = 0; i < 10; window.foo++) {
			}",
        "for (const item of window.foo) {
			}",
        "for (const item of window) {
			}",
        "switch (window) {}",
        "switch (true) {
				case window:
					break;
			}",
        "switch (true) {
				case window.foo:
					break;
			}",
        "while (window) {
			}",
        "do {} while (window) {}",
        "if (window) {}",
        "throw window",
        "var foo = window",
        "function foo (name = window) {
			}",
        "self.innerWidth",
        "self.innerHeight",
        "window.crypto",
        r#"window.addEventListener("play", () => {})"#,
        "window.onplay = function () {}",
        "function greet({ name = window.foo }) {}",
        "({ foo: window.foo } =  {})",
        "[window.foo] = []",
        "foo[window]",
        "foo[window.foo]",
        r#"typeof window !== "undefined""#,
        r#"typeof self !== "undefined""#,
        r#"typeof global !== "undefined""#,
        r#"typeof window.something === "function""#,
        r#"typeof self.something === "function""#,
        r#"typeof global.something === "function""#,
        "global.global_did_not_declare_in_language_options",
        "window.window_did_not_declare_in_language_options",
        "self.self_did_not_declare_in_language_options",
    ];

    Tester::new(PreferGlobalThis::NAME, PreferGlobalThis::PLUGIN, pass, fail).test_and_snapshot();
}
