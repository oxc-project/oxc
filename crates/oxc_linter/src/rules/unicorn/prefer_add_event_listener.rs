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
    "animationEnd",
    "animationStart",
    "animationiteration",
    "appinstalled",
    "auxclick",
    "beforeblur",
    "beforecopy",
    "beforecut",
    "beforeinput",
    "beforeinstallprompt",
    "beforepaste",
    "beforeprint",
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
    "contextmenu",
    "controllerchange",
    "copy",
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
    "search",
    "seeked",
    "seeking",
    "select",
    "selectionchange",
    "selectstart",
    "show",
    "sizechanged",
    "sourceclosed",
    "sourceended",
    "sourceopen",
    "stalled",
    "statechange",
    "storage",
    "submit",
    "suspend",
    "text",
    "textInput",
    "textinput",
    "timeupdate",
    "toggle",
    "touchcancel",
    "touchend",
    "touchmove",
    "touchstart",
    "transitionend",
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
        (r"foo.addEventListener('click', () => {})", None),
        (r"foo.removeEventListener('click', onClick)", None),
        (r"foo.onclick", None),
        (r"foo[onclick] = () => {}", None),
        (r#"foo["onclick"] = () => {}"#, None),
        (r"foo.onunknown = () => {}", None),
        (r"foo.setCallBack = () => {console.log('foo')}", None),
        (r"setCallBack = () => {console.log('foo')}", None),
        (r"foo.onclick.bar = () => {}", None),
        (r"foo['x'] = true;", None),
    ];

    let fail = vec![
        (r"foo.onclick = () => {}", None),
        (r"foo.onclick = 1", None),
        (r"foo.bar.onclick = onClick", None),
        (r"const bar = null; foo.onclick = bar;", None),
        (r"foo.onkeydown = () => {}", None),
        (r"foo.ondragend = () => {}", None),
        (r"foo.onclick = null", None),
        (r"foo.onclick = undefined", None),
        (r"window.onbeforeunload = null", None),
        (r"window.onbeforeunload = undefined", None),
        (r"window.onbeforeunload = foo", None),
        (r"window.onbeforeunload = () => 'foo'", None),
        (r"myWorker.port.onmessage = function(e) {}", None),
        (r"((foo)).onclick = ((0, listener))", None),
        (r"window.onload = window.onunload = function() {};", None),
        (r"window.onunload ??= function() {};", None),
        (r"window.onunload ||= function() {};", None),
        (r"window.onunload += function() {};", None),
        (r"(el as HTMLElement).onmouseenter = onAnchorMouseEnter;", None),
    ];

    Tester::new(PreferAddEventListener::NAME, PreferAddEventListener::PLUGIN, pass, fail)
        .test_and_snapshot();
}
