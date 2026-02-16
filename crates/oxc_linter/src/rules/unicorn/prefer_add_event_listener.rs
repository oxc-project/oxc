use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_add_event_listener_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `addEventListener()` over their `on`-function counterparts.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferAddEventListener;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of `.addEventListener()` and `.removeEventListener()` over their `on`-function counterparts.
    ///
    /// For example, `foo.addEventListener('click', handler);` is preferred over `foo.onclick = handler;` for HTML DOM Events.
    ///
    /// ### Why is this bad?
    ///
    /// There are [numerous advantages of using `addEventListener`](https://stackoverflow.com/questions/6348494/addeventlistener-vs-onclick/35093997#35093997). Some of these advantages include registering unlimited event handlers and optionally having the event handler invoked only once.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo.onclick = () => {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// foo.addEventListener('click', () => {});
    /// ```
    PreferAddEventListener,
    unicorn,
    suspicious,
    pending
);

impl Rule for PreferAddEventListener {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assignment_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = assignment_expr.left.as_member_expression() else {
            return;
        };

        if member_expr.is_computed() {
            return;
        }

        let Some((span, name)) = member_expr.static_property_info() else {
            return;
        };

        if !name.starts_with("on") {
            return;
        }

        if !DOM_EVENT_TYPE_NAMES.contains(name.trim_start_matches("on")) {
            return;
        }

        ctx.diagnostic(prefer_add_event_listener_diagnostic(span));
    }
}

// Can refer to the following sources for the list of event handler names, compare
// this array against any new `onx` functions introduced in browsers:
// - https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes#list_of_global_event_handler_attributes
// - https://github.com/mdn/browser-compat-data/blob/d5d5f2e21ef3f798784d1f5f75bde7c7f10f250e/api/Element.json
// - https://github.com/microsoft/TypeScript-DOM-lib-generator/blob/f915ac0c987300d75af41bfe4a34bb29a0fb941f/baselines/dom.generated.d.ts
//
// Please avoid adding new events that are not implemented in at least two major browser engines!
// Last updated: Nov 2025
const DOM_EVENT_TYPE_NAMES: phf::Set<&'static str> = phf::phf_set![
    "AnimationEnd",
    "AnimationIteration",
    "AnimationStart",
    "DOMAttrModified",
    "DOMCharacterDataModified",
    "DOMContentLoaded",
    "DOMNodeInserted",
    "DOMNodeInsertedIntoDocument",
    "DOMNodeRemoved",
    "DOMNodeRemovedFromDocument",
    "DOMSubtreeModified",
    "MSGestureChange",
    "MSGestureEnd",
    "MSGestureHold",
    "MSGestureStart",
    "MSGestureTap",
    "MSGotPointerCapture",
    "MSInertiaStart",
    "MSLostPointerCapture",
    "MSPointerCancel",
    "MSPointerDown",
    "MSPointerEnter",
    "MSPointerHover",
    "MSPointerLeave",
    "MSPointerMove",
    "MSPointerOut",
    "MSPointerOver",
    "MSPointerUp",
    "abort",
    "activate",
    "afterblur",
    "afterprint",
    "animationcancel",
    "animationend",
    "animationiteration",
    "animationstart",
    "appinstalled",
    "auxclick",
    "beforeblur",
    "beforecopy",
    "beforecut",
    "beforeinput",
    "beforeinstallprompt",
    "beforematch",
    "beforepaste",
    "beforeprint",
    "beforetoggle",
    "beforeunload",
    "blur",
    "cancel",
    "canplay",
    "canplaythrough",
    "change",
    "click",
    "close",
    "compositionend",
    "compositionstart",
    "compositionupdate",
    "connect",
    "consolemessage",
    "contextlost",
    "contextmenu",
    "contextrestored",
    "controllerchange",
    "copy",
    "cuechange",
    "cut",
    "dblclick",
    "deactivate",
    "devicechange",
    "devicemotion",
    "deviceorientation",
    "drag",
    "dragend",
    "dragenter",
    "dragexit",
    "dragleave",
    "dragover",
    "dragstart",
    "drop",
    "durationchange",
    "emptied",
    "encrypted",
    "ended",
    "error",
    "exit",
    "fetch",
    "focus",
    "focusin",
    "focusout",
    "foreignfetch",
    "formdata",
    "fullscreenchange",
    "gotpointercapture",
    "hashchange",
    "help",
    "input",
    "install",
    "invalid",
    "keydown",
    "keypress",
    "keyup",
    "load",
    "loadabort",
    "loadcommit",
    "loadeddata",
    "loadedmetadata",
    "loadredirect",
    "loadstart",
    "loadstop",
    "losecapture",
    "lostpointercapture",
    "message",
    "messageerror",
    "mousecancel",
    "mousedown",
    "mouseenter",
    "mouseleave",
    "mousemove",
    "mouseout",
    "mouseover",
    "mouseup",
    "oanimationend",
    "oanimationiteration",
    "oanimationstart",
    "offline",
    "online",
    "open",
    "orientationchange",
    "otransitionend",
    "pagehide",
    "pageshow",
    "paste",
    "pause",
    "play",
    "playing",
    "pointercancel",
    "pointerdown",
    "pointerenter",
    "pointerleave",
    "pointermove",
    "pointerout",
    "pointerover",
    "pointerrawupdate",
    "pointerup",
    "popstate",
    "progress",
    "propertychange",
    "ratechange",
    "readystatechange",
    "reset",
    "resize",
    "responsive",
    "rightclick",
    "scroll",
    "scrollend",
    "search",
    "securitypolicyviolation",
    "seeked",
    "seeking",
    "select",
    "selectionchange",
    "selectstart",
    "show",
    "sizechanged",
    "slotchange",
    "sourceclosed",
    "sourceended",
    "sourceopen",
    "stalled",
    "statechange",
    "storage",
    "submit",
    "suspend",
    "text",
    "textinput",
    "textInput",
    "timeupdate",
    "toggle",
    "touchcancel",
    "touchend",
    "touchmove",
    "touchstart",
    "transitioncancel",
    "transitionend",
    "transitionrun",
    "transitionstart",
    "unload",
    "unresponsive",
    "update",
    "updateend",
    "updatefound",
    "updatestart",
    "visibilitychange",
    "volumechange",
    "waiting",
    "webkitTransitionEnd",
    "wheel",
];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("foo.addEventListener('click', () => {})", None),
        ("foo.removeEventListener('click', onClick)", None),
        ("foo.onclick", None),
        ("foo[onclick] = () => {}", None),
        (r#"foo["onclick"] = () => {}"#, None),
        ("foo.onunknown = () => {}", None),
        ("foo.setCallBack = () => {console.log('foo')}", None),
        ("setCallBack = () => {console.log('foo')}", None),
        ("foo.onclick.bar = () => {}", None),
        ("foo['x'] = true;", None),
        // TODO: Uncomment these tests after we introduce support for the excludedPackages option.
        // https://github.com/sindresorhus/eslint-plugin-unicorn/blob/0bf85e0df741255ac2d347eefc57daf4362ff0a0/docs/rules/prefer-add-event-listener.md#excludedpackages
        // (
        //     "const Koa = require('koa');
        //     const app = new Koa();
        //     app.onerror = () => {};",
        //     None,
        // ),
        // (
        //     "const sax = require('sax');
        //     const parser = sax.parser();
        //     parser.onerror = () => {};",
        //     None,
        // ),
        // (
        //     "import Koa from 'koa';
        //     const app = new Koa();
        //     app.onerror = () => {};",
        //     None,
        // ),
        // (
        //     "import sax from 'sax';
        //     const parser = sax.parser();
        //     parser.onerror = () => {};",
        //     None,
        // ),
        // (
        //     "import {sax as foo} from 'sax';
        //     const parser = foo.parser();
        //     parser.onerror = () => {};",
        //     None,
        // ),
        // (
        //     "const foo = require('foo');
        //     foo.onerror = () => {};",
        //     Some(serde_json::json!(excludeFooOptions)),
        // ),
        // (
        //     "import foo from 'foo';
        //     foo.onclick = () => {};",
        //     Some(serde_json::json!(excludeFooOptions)),
        // ),
    ];

    let fail = vec![
        ("foo.onclick = () => {}", None),
        ("foo.onclick = 1", None),
        ("foo.bar.onclick = onClick", None),
        ("const bar = null; foo.onclick = bar;", None),
        ("foo.onkeydown = () => {}", None),
        ("foo.ondragend = () => {}", None),
        (
            "foo.onclick = function (e) {
                console.log(e);
            }",
            None,
        ),
        ("foo.onclick = null", None),
        ("foo.onclick = undefined", None),
        ("window.onbeforeunload = null", None),
        ("window.onbeforeunload = undefined", None),
        ("window.onbeforeunload = foo", None),
        ("window.onbeforeunload = () => 'foo'", None),
        (
            "window.onbeforeunload = () => {
                return bar;
            }",
            None,
        ),
        (
            "window.onbeforeunload = function () {
                return 'bar';
            }",
            None,
        ),
        (
            "window.onbeforeunload = function () {
                return;
            }",
            None,
        ),
        (
            "window.onbeforeunload = function () {
                (() => {
                    return 'foo';
                })();
            }",
            None,
        ),
        (
            "window.onbeforeunload = e => {
                console.log(e);
            }",
            None,
        ),
        (
            "const foo = require('foo');
            foo.onerror = () => {};",
            None,
        ),
        (
            "import foo from 'foo';
            foo.onerror = () => {};",
            None,
        ),
        (
            "foo.onerror = () => {};
            function bar() {
                const koa = require('koa');
                koa.onerror = () => {};
            }",
            None,
        ),
        // (
        //     "const Koa = require('koa');
        //     const app = new Koa();
        //     app.onerror = () => {};",
        //     Some(serde_json::json!(excludeFooOptions)),
        // ),
        // (
        //     "import {Koa as Foo} from 'koa';
        //     const app = new Foo();
        //     app.onerror = () => {};",
        //     Some(serde_json::json!(excludeFooOptions)),
        // ),
        // (
        //     "const sax = require('sax');
        //     const parser = sax.parser();
        //     parser.onerror = () => {};",
        //     Some(serde_json::json!(excludeFooOptions)),
        // ),
        ("myWorker.port.onmessage = function(e) {}", None),
        ("((foo)).onclick = ((0, listener))", None),
        ("window.onload = window.onunload = function() {};", None),
        ("window.onunload ??= function() {};", None),
        ("window.onunload ||= function() {};", None),
        ("window.onunload += function() {};", None),
        ("foo.onclick = true", None),
        ("foo.onclick = 'bar'", None),
        ("foo.onclick = `bar`", None),
        ("foo.onclick = {}", None),
        ("foo.onclick = []", None),
        ("foo.onclick = void 0", None),
        ("foo.onclick = new Handler()", None),
        ("(el as HTMLElement).onmouseenter = onAnchorMouseEnter;", None),
    ];

    // TODO: Implement autofix and use these tests.
    // let _fix = vec![(
    //     "(el as HTMLElement).onmouseenter = onAnchorMouseEnter;",
    //     "(el as HTMLElement).addEventListener('mouseenter', onAnchorMouseEnter);",
    // )];

    Tester::new(PreferAddEventListener::NAME, PreferAddEventListener::PLUGIN, pass, fail)
        .test_and_snapshot();
}
