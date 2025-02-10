use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

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

const DOM_EVENT_TYPE_NAMES: phf::Set<&'static str> = phf::phf_set!(
    // Mouse events
    "click",
    "rightclick",
    "dblclick",
    "auxclick",
    "mousedown",
    "mouseup",
    "mouseover",
    "mouseout",
    "mousemove",
    "mouseenter",
    "mouseleave",
    // Non-existent event; will never fire. This exists as a mouse counterpart to
    // POINTERCANCEL.
    "mousecancel",
    // Selection events.
    // https://www.w3.org/TR/selection-api/
    "selectionchange",
    "selectstart", // IE, Safari, Chrome
    // Wheel events
    // http://www.w3.org/TR/DOM-Level-3-Events/#events-wheelevents
    "wheel",
    // Key events
    "keypress",
    "keydown",
    "keyup",
    // Focus
    "blur",
    "focus",
    "deactivate", // IE only
    "focusin",
    "focusout",
    // Forms
    "change",
    "reset",
    "select",
    "submit",
    "input",
    "propertychange", // IE only
    // Drag and drop
    "dragstart",
    "drag",
    "dragenter",
    "dragover",
    "dragleave",
    "drop",
    "dragend",
    // Touch events
    // Note that other touch events exist, but we should follow the W3C list here.
    // http://www.w3.org/TR/touch-events/#list-of-touchevent-types
    "touchstart",
    "touchmove",
    "touchend",
    "touchcancel",
    // Misc
    "beforeunload",
    "consolemessage",
    "contextmenu",
    "devicechange",
    "devicemotion",
    "deviceorientation",
    "DOMContentLoaded",
    "error",
    "help",
    "load",
    "losecapture",
    "orientationchange",
    "readystatechange",
    "resize",
    "scroll",
    "unload",
    // Media events
    "canplay",
    "canplaythrough",
    "durationchange",
    "emptied",
    "ended",
    "loadeddata",
    "loadedmetadata",
    "pause",
    "play",
    "playing",
    "progress",
    "ratechange",
    "seeked",
    "seeking",
    "stalled",
    "suspend",
    "timeupdate",
    "volumechange",
    "waiting",
    // Media Source Extensions events
    // https://www.w3.org/TR/media-source/#mediasource-events
    "sourceopen",
    "sourceended",
    "sourceclosed",
    // https://www.w3.org/TR/media-source/#sourcebuffer-events
    "abort",
    "update",
    "updatestart",
    "updateend",
    // HTML 5 History events
    // See http://www.w3.org/TR/html5/browsers.html#event-definitions-0
    "hashchange",
    "pagehide",
    "pageshow",
    "popstate",
    // Copy and Paste
    // Support is limited. Make sure it works on your favorite browser
    // before using.
    // http://www.quirksmode.org/dom/events/cutcopypaste.html
    "copy",
    "paste",
    "cut",
    "beforecopy",
    "beforecut",
    "beforepaste",
    // HTML5 online/offline events.
    // http://www.w3.org/TR/offline-webapps/#related
    "online",
    "offline",
    // HTML 5 worker events
    "message",
    "connect",
    // Service Worker Events - ServiceWorkerGlobalScope context
    // See https://w3c.github.io/ServiceWorker/#execution-context-events
    // message event defined in worker events section
    "install",
    "activate",
    "fetch",
    "foreignfetch",
    "messageerror",
    // Service Worker Events - Document context
    // See https://w3c.github.io/ServiceWorker/#document-context-events
    "statechange",
    "updatefound",
    "controllerchange",
    // CSS animation events.
    "AnimationStart",
    "oanimationstart",
    "animationStart",
    "AnimationEnd",
    "oanimationend",
    "animationEnd",
    "AnimationIteration",
    "oanimationiteration",
    "animationiteration",
    // CSS transition events. Based on the browser support described at:
    // https://developer.mozilla.org/en/css/css_transitions#Browser_compatibility
    "webkitTransitionEnd",
    "otransitionend",
    "transitionend",
    // W3C Pointer Events
    // http://www.w3.org/TR/pointerevents/
    "pointerdown",
    "pointerup",
    "pointercancel",
    "pointermove",
    "pointerover",
    "pointerout",
    "pointerenter",
    "pointerleave",
    "gotpointercapture",
    "lostpointercapture",
    // IE specific events.
    // See http://msdn.microsoft.com/en-us/library/ie/hh772103(v=vs.85).aspx
    // these events will be supplanted in IE11.
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
    // Native IMEs/input tools events.
    "text",
    // The textInput event is supported in IE9+, but only in lower case. All other
    // browsers use the camel-case event name.
    "textinput",
    "textInput",
    "compositionstart",
    "compositionupdate",
    "compositionend",
    // The beforeinput event is initially only supported in Safari. See
    // https://bugs.chromium.org/p/chromium/issues/detail?id=342670 for Chrome
    // implementation tracking.
    "beforeinput",
    // Webview tag events
    // See https://developer.chrome.com/apps/tags/webview
    "exit",
    "loadabort",
    "loadcommit",
    "loadredirect",
    "loadstart",
    "loadstop",
    "responsive",
    "sizechanged",
    "unresponsive",
    // HTML5 Page Visibility API.	See details at
    // `goog.labs.dom.PageVisibilityMonitor`.
    "visibilitychange",
    // LocalStorage event.
    "storage",
    // DOM Level 2 mutation events (deprecated).
    "DOMSubtreeModified",
    "DOMNodeInserted",
    "DOMNodeRemoved",
    "DOMNodeRemovedFromDocument",
    "DOMNodeInsertedIntoDocument",
    "DOMAttrModified",
    "DOMCharacterDataModified",
    // Print events.
    "beforeprint",
    "afterprint",
    // Web app manifest events.
    "beforeinstallprompt",
    "appinstalled",
    // https://github.com/facebook/react/blob/cae635054e17a6f107a39d328649137b83f25972/packages/react-dom/src/events/DOMEventNames.js#L12
    "afterblur",
    "beforeblur",
    "cancel",
    "close",
    "dragexit",
    "encrypted",
    "fullscreenchange",
    "invalid",
    "toggle",
    // https://github.com/sindresorhus/eslint-plugin-unicorn/pull/147
    "search",
    "open",
    "show"
);

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
