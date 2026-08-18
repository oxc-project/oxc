//! Known global identifiers, properties, and methods for side-effect analysis.
//!
//! This module contains lookup tables that classify JavaScript globals by their
//! side-effect behavior: whether accessing them, reading their properties, or
//! calling their methods is side-effect-free.

use oxc_ast::ast::*;
use oxc_compat::ESFeature;
use oxc_regular_expression::{
    LiteralParser, Options, RegexUnsupportedFlags, RegexUnsupportedPatterns, ast::Pattern,
    has_unsupported_regular_expression_flags, has_unsupported_regular_expression_pattern,
};

use super::context::MayHaveSideEffectsContext;

/// Validate that a RegExp constructor call cannot throw on the target engines.
///
/// Returns `true` if the pattern and flags are valid and supported by the target engines
/// (pure/side-effect free), and `false` if invalid, unsupported, or not statically known.
///
/// A modern parser accepting a pattern is not enough: constructor calls are commonly used for
/// feature detection, and valid newer syntax can still throw a `SyntaxError` on an older engine.
///
/// See <https://github.com/oxc-project/oxc/issues/18050>
pub fn is_valid_regexp<'a>(
    args: &[Argument<'a>],
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> bool {
    // Extract pattern from first argument
    let (pattern, is_regexp_literal) = match args.first() {
        // No arguments: `RegExp()` is valid, returns /(?:)/
        None => ("", false),
        Some(arg) => match arg.as_expression() {
            // A RegExp literal is already known to contain a valid pattern. Replacement flags are
            // validated below because support for that constructor form varies by target.
            Some(Expression::RegExpLiteral(_)) => ("", true),
            // String literal: extract the pattern to validate
            Some(Expression::StringLiteral(s)) if !s.lone_surrogates => (s.value.as_str(), false),
            // Non-literal argument: can't statically determine, assume side effects
            _ => return false,
        },
    };

    // Extract flags from second argument
    let flags = match args.get(1) {
        None => None,
        Some(arg) => match arg.as_expression() {
            Some(Expression::StringLiteral(s)) => Some(s.value.as_str()),
            // Non-literal flags: can't statically determine, assume side effects
            _ => return false,
        },
    };

    if is_regexp_literal {
        // ES5 throws whenever a RegExp object and flags are both supplied.
        return regexp_flags_are_supported(flags.unwrap_or_default(), ctx)
            && (flags.is_none()
                || supports_es_feature(ctx, ESFeature::ES2015RegExpConstructorCanAlterFlags));
    }

    // The parser performs complete syntax validation beyond the compatibility checks above.
    let allocator = oxc_allocator::Allocator::default();
    LiteralParser::new(&allocator, pattern, flags, Options::default())
        .parse()
        .is_ok_and(|pattern| is_regexp_syntax_supported(&pattern, flags.unwrap_or_default(), ctx))
}

/// Whether parsed RegExp syntax is supported by every configured target.
pub fn is_regexp_syntax_supported<'a>(
    pattern: &Pattern<'_>,
    flags: &str,
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> bool {
    regexp_flags_are_supported(flags, ctx)
        && !has_unsupported_regular_expression_pattern(pattern, &unsupported_regexp_patterns(ctx))
}

fn regexp_flags_are_supported<'a>(flags: &str, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
    !has_unsupported_regular_expression_flags(flags, &unsupported_regexp_flags(ctx))
}

fn unsupported_regexp_flags<'a>(ctx: &impl MayHaveSideEffectsContext<'a>) -> RegexUnsupportedFlags {
    RegexUnsupportedFlags {
        sticky: !supports_es_feature(ctx, ESFeature::ES2015StickyRegex),
        unicode: !supports_es_feature(ctx, ESFeature::ES2015UnicodeRegex),
        dot_all: !supports_es_feature(ctx, ESFeature::ES2018DotallRegex),
        match_indices: !supports_es_feature(ctx, ESFeature::ES2022MatchIndicesRegex),
        unicode_sets: !supports_es_feature(ctx, ESFeature::ES2024UnicodeSetsRegex),
    }
}

fn unsupported_regexp_patterns<'a>(
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> RegexUnsupportedPatterns {
    RegexUnsupportedPatterns {
        named_capture_groups: !supports_es_feature(ctx, ESFeature::ES2018NamedCapturingGroupsRegex),
        duplicate_named_capture_groups: !supports_es_feature(
            ctx,
            ESFeature::ES2025DuplicateNamedCapturingGroupsRegex,
        ),
        unicode_property_escapes: !supports_es_feature(ctx, ESFeature::ES2018UnicodePropertyRegex),
        look_behind_assertions: !supports_es_feature(ctx, ESFeature::ES2018LookbehindRegex),
        pattern_modifiers: !supports_es_feature(ctx, ESFeature::ES2025RegexpModifiers),
    }
}

fn supports_es_feature<'a>(ctx: &impl MayHaveSideEffectsContext<'a>, feature: ESFeature) -> bool {
    ctx.engine_targets().is_some_and(|target| target.supports_es_feature(feature))
}

#[rustfmt::skip]
pub(super) fn is_pure_global_function(name: &str) -> bool {
    matches!(name, "decodeURI" | "decodeURIComponent" | "encodeURI" | "encodeURIComponent"
            | "escape" | "isFinite" | "isNaN" | "parseFloat" | "parseInt")
}

/// Constructors that are side-effect-free when called as functions (not `new`),
/// provided all arguments are side-effect-free.
///
/// Note: `Number`, `Symbol`, `BigInt`, and Error types are NOT included here because
/// they require special-case argument validation in `CallExpression`:
/// - `Number(Symbol())` throws TypeError (`ToNumeric` on Symbol)
/// - `Symbol(Symbol())` throws TypeError (`ToString` on Symbol)
/// - `Error(Symbol())` throws TypeError (`ToString` on Symbol)
/// - `BigInt(1.5)`, `BigInt(undefined)`, etc. throw for invalid values
///
/// `String` IS included because `String()` has special Symbol handling
/// (`String(Symbol())` returns `"Symbol()"` without throwing), and per the
/// "Coercion Methods Are Pure" assumption, `ToPrimitive` on objects is safe.
///
/// `Date` IS included because `Date()` as a function ignores all arguments
/// and just returns the current date as a string.
#[rustfmt::skip]
pub(super) fn is_pure_callable_constructor(name: &str) -> bool {
    matches!(name, "Date" | "Boolean" | "Object" | "String")
}

/// Constructors that are unconditionally side-effect-free with any arguments.
///
/// - `Object`: wraps/returns any argument, no coercion
/// - `Boolean`: `ToBoolean` is a purely internal operation, no user code
///
/// Note: Error types call `ToString(msg)` which throws on Symbol. They need argument
/// validation (Symbol check). `String`, `Number`, `Date`, `ArrayBuffer` also need checks.
#[rustfmt::skip]
pub(super) fn is_unconditionally_pure_constructor(name: &str) -> bool {
    matches!(name, "Object" | "Boolean")
}

/// Whether the name is an Error constructor.
#[rustfmt::skip]
pub(super) fn is_error_constructor(name: &str) -> bool {
    matches!(name, "Error" | "EvalError" | "RangeError" | "ReferenceError"
            | "SyntaxError" | "TypeError" | "URIError")
}

/// Whether the name matches any TypedArray constructor name.
///
/// See <https://tc39.es/ecma262/multipage/indexed-collections.html#sec-typedarray-objects> for the list of TypedArrays.
#[rustfmt::skip]
pub fn is_typed_array_constructor(name: &str) -> bool {
    matches!(name, "Int8Array" | "Uint8Array" | "Uint8ClampedArray"
            | "Int16Array" | "Uint16Array" | "Int32Array" | "Uint32Array"
            | "Float32Array" | "Float64Array" | "BigInt64Array" | "BigUint64Array")
}

/// Whether a collection constructor (`Map`, `Set`, `WeakMap`, `WeakSet`) call is pure.
///
/// These constructors iterate their argument via `Symbol.iterator`, which can have side effects
/// when the argument is a variable reference (custom iterators, proxies, etc.).
/// Only provably safe arguments are considered pure:
/// - No arguments: `new Set()`, `new Map()`
/// - `null` or `undefined`: `new Set(null)`, `new Map(undefined)`
/// - Array literals: `new Set([1,2,3])`, `new Map([[k,v]])`
///
/// Following esbuild and Rollup behavior.
pub(super) fn is_pure_collection_constructor<'a>(
    name: &str,
    args: &[Argument<'a>],
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> bool {
    if !matches!(name, "Set" | "Map" | "WeakSet" | "WeakMap") {
        return false;
    }
    match args.first() {
        // No arguments: always pure
        None => true,
        Some(arg) => match arg.as_expression() {
            Some(Expression::NullLiteral(_)) => true,
            Some(Expression::Identifier(id))
                if id.name == "undefined" && ctx.is_global_reference(id) =>
            {
                true
            }
            Some(Expression::ArrayExpression(arr)) => {
                // For Map/WeakMap, each element must also be an array literal (key-value pair)
                if matches!(name, "Map" | "WeakMap") {
                    arr.elements
                        .iter()
                        .all(|el| matches!(el, ArrayExpressionElement::ArrayExpression(_)))
                } else {
                    true
                }
            }
            _ => false,
        },
    }
}

/// Whether the name matches any known global constructors.
///
/// <https://tc39.es/ecma262/multipage/global-object.html#sec-constructor-properties-of-the-global-object>
pub(super) fn is_known_global_constructor(name: &str) -> bool {
    // technically, we need to exclude the constructors that are not supported by the target
    matches!(
        name,
        "AggregateError"
            | "Array"
            | "ArrayBuffer"
            | "BigInt"
            | "BigInt64Array"
            | "BigUint64Array"
            | "Boolean"
            | "DataView"
            | "Date"
            | "Error"
            | "EvalError"
            | "FinalizationRegistry"
            | "Float32Array"
            | "Float64Array"
            | "Function"
            | "Int8Array"
            | "Int16Array"
            | "Int32Array"
            | "Iterator"
            | "Map"
            | "Number"
            | "Object"
            | "Promise"
            | "Proxy"
            | "RangeError"
            | "ReferenceError"
            | "RegExp"
            | "Set"
            | "SharedArrayBuffer"
            | "String"
            | "Symbol"
            | "SyntaxError"
            | "TypeError"
            | "Uint8Array"
            | "Uint8ClampedArray"
            | "Uint16Array"
            | "Uint32Array"
            | "URIError"
            | "WeakMap"
            | "WeakSet"
    )
}

/// Whether the name matches any known global identifier that is side-effect-free to access.
///
/// This list is ported from Rolldown's `GLOBAL_IDENT` set, which mirrors Rollup's `knownGlobals`.
/// It includes browser/host-specific APIs (e.g. `document`, `window`, DOM classes) intentionally,
/// matching Rollup's behavior of assuming these globals exist in the target environment.
/// `NaN`, `Infinity`, `undefined` are excluded since they are already handled as special cases.
pub(super) fn is_known_global_identifier(name: &str) -> bool {
    KNOWN_GLOBAL_IDENTIFIERS.binary_search(&name).is_ok()
}

/// Sorted table backing [`is_known_global_identifier`].
///
/// A binary search over a static table compiles to a fraction of the code a `matches!` over
/// hundreds of string literals expands to while keeping lookups logarithmic. Keep the entries
/// sorted (byte order) and unique; the table is checked in a test.
#[rustfmt::skip]
static KNOWN_GLOBAL_IDENTIFIERS: &[&str] = &[
    "AbortController", "AbortSignal", "AggregateError", "AnalyserNode", "Animation",
    "AnimationEffect", "AnimationEvent", "AnimationPlaybackEvent", "AnimationTimeline", "Array",
    "ArrayBuffer", "Attr", "Audio", "AudioBuffer", "AudioBufferSourceNode", "AudioDestinationNode",
    "AudioListener", "AudioNode", "AudioParam", "AudioProcessingEvent", "AudioScheduledSourceNode",
    "BarProp", "BeforeUnloadEvent", "BigInt", "BiquadFilterNode", "Blob", "BlobEvent", "Boolean",
    "ByteLengthQueuingStrategy", "CDATASection", "CSS", "CSSAnimation", "CSSFontFaceRule",
    "CSSImportRule", "CSSKeyframeRule", "CSSKeyframesRule", "CSSMediaRule", "CSSNamespaceRule",
    "CSSPageRule", "CSSRule", "CSSRuleList", "CSSStyleDeclaration", "CSSStyleRule",
    "CSSStyleSheet", "CSSSupportsRule", "CSSTransition", "CanvasGradient", "CanvasPattern",
    "CanvasRenderingContext2D", "ChannelMergerNode", "ChannelSplitterNode", "CharacterData",
    "ClipboardEvent", "CloseEvent", "Comment", "CompositionEvent", "ConvolverNode",
    "CountQueuingStrategy", "Crypto", "CustomElementRegistry", "CustomEvent", "DOMException",
    "DOMImplementation", "DOMMatrix", "DOMMatrixReadOnly", "DOMParser", "DOMPoint",
    "DOMPointReadOnly", "DOMQuad", "DOMRect", "DOMRectList", "DOMRectReadOnly", "DOMStringList",
    "DOMStringMap", "DOMTokenList", "DataTransfer", "DataTransferItem", "DataTransferItemList",
    "DataView", "Date", "DelayNode", "Document", "DocumentFragment", "DocumentTimeline",
    "DocumentType", "DragEvent", "DynamicsCompressorNode", "Element", "Error", "ErrorEvent",
    "EvalError", "Event", "EventSource", "EventTarget", "File", "FileList", "FileReader",
    "Float32Array", "Float64Array", "FocusEvent", "FontFace", "FormData", "Function", "GainNode",
    "Gamepad", "GamepadButton", "GamepadEvent", "Geolocation", "GeolocationPositionError",
    "HTMLAllCollection", "HTMLAnchorElement", "HTMLAreaElement", "HTMLAudioElement",
    "HTMLBRElement", "HTMLBaseElement", "HTMLBodyElement", "HTMLButtonElement",
    "HTMLCanvasElement", "HTMLCollection", "HTMLDListElement", "HTMLDataElement",
    "HTMLDataListElement", "HTMLDetailsElement", "HTMLDirectoryElement", "HTMLDivElement",
    "HTMLDocument", "HTMLElement", "HTMLEmbedElement", "HTMLFieldSetElement", "HTMLFontElement",
    "HTMLFormControlsCollection", "HTMLFormElement", "HTMLFrameElement", "HTMLFrameSetElement",
    "HTMLHRElement", "HTMLHeadElement", "HTMLHeadingElement", "HTMLHtmlElement",
    "HTMLIFrameElement", "HTMLImageElement", "HTMLInputElement", "HTMLLIElement",
    "HTMLLabelElement", "HTMLLegendElement", "HTMLLinkElement", "HTMLMapElement",
    "HTMLMarqueeElement", "HTMLMediaElement", "HTMLMenuElement", "HTMLMetaElement",
    "HTMLMeterElement", "HTMLModElement", "HTMLOListElement", "HTMLObjectElement",
    "HTMLOptGroupElement", "HTMLOptionElement", "HTMLOptionsCollection", "HTMLOutputElement",
    "HTMLParagraphElement", "HTMLParamElement", "HTMLPictureElement", "HTMLPreElement",
    "HTMLProgressElement", "HTMLQuoteElement", "HTMLScriptElement", "HTMLSelectElement",
    "HTMLSlotElement", "HTMLSourceElement", "HTMLSpanElement", "HTMLStyleElement",
    "HTMLTableCaptionElement", "HTMLTableCellElement", "HTMLTableColElement", "HTMLTableElement",
    "HTMLTableRowElement", "HTMLTableSectionElement", "HTMLTemplateElement", "HTMLTextAreaElement",
    "HTMLTimeElement", "HTMLTitleElement", "HTMLTrackElement", "HTMLUListElement",
    "HTMLUnknownElement", "HTMLVideoElement", "HashChangeEvent", "Headers", "History", "IDBCursor",
    "IDBCursorWithValue", "IDBDatabase", "IDBFactory", "IDBIndex", "IDBKeyRange", "IDBObjectStore",
    "IDBOpenDBRequest", "IDBRequest", "IDBTransaction", "IDBVersionChangeEvent", "Image",
    "ImageData", "InputEvent", "Int16Array", "Int32Array", "Int8Array", "IntersectionObserver",
    "IntersectionObserverEntry", "Intl", "JSON", "KeyboardEvent", "KeyframeEffect", "Location",
    "Map", "Math", "MediaCapabilities", "MediaElementAudioSourceNode", "MediaEncryptedEvent",
    "MediaError", "MediaList", "MediaQueryList", "MediaQueryListEvent", "MediaRecorder",
    "MediaSource", "MediaStream", "MediaStreamAudioDestinationNode", "MediaStreamAudioSourceNode",
    "MediaStreamTrack", "MediaStreamTrackEvent", "MessageChannel", "MessageEvent", "MessagePort",
    "MimeType", "MimeTypeArray", "MouseEvent", "MutationEvent", "MutationObserver",
    "MutationRecord", "NamedNodeMap", "Navigator", "Node", "NodeFilter", "NodeIterator",
    "NodeList", "Notification", "Number", "Object", "OfflineAudioCompletionEvent", "Option",
    "OscillatorNode", "PageTransitionEvent", "Path2D", "Performance", "PerformanceEntry",
    "PerformanceMark", "PerformanceMeasure", "PerformanceNavigation", "PerformanceObserver",
    "PerformanceObserverEntryList", "PerformanceResourceTiming", "PerformanceTiming",
    "PeriodicWave", "Plugin", "PluginArray", "PointerEvent", "PopStateEvent",
    "ProcessingInstruction", "ProgressEvent", "Promise", "PromiseRejectionEvent", "Proxy",
    "RTCCertificate", "RTCDTMFSender", "RTCDTMFToneChangeEvent", "RTCDataChannel",
    "RTCDataChannelEvent", "RTCIceCandidate", "RTCPeerConnection", "RTCPeerConnectionIceEvent",
    "RTCRtpReceiver", "RTCRtpSender", "RTCRtpTransceiver", "RTCSessionDescription",
    "RTCStatsReport", "RTCTrackEvent", "RadioNodeList", "Range", "RangeError", "ReadableStream",
    "ReferenceError", "Reflect", "RegExp", "Request", "ResizeObserver", "ResizeObserverEntry",
    "Response", "SVGAElement", "SVGAngle", "SVGAnimateElement", "SVGAnimateMotionElement",
    "SVGAnimateTransformElement", "SVGAnimatedAngle", "SVGAnimatedBoolean",
    "SVGAnimatedEnumeration", "SVGAnimatedInteger", "SVGAnimatedLength", "SVGAnimatedLengthList",
    "SVGAnimatedNumber", "SVGAnimatedNumberList", "SVGAnimatedPreserveAspectRatio",
    "SVGAnimatedRect", "SVGAnimatedString", "SVGAnimatedTransformList", "SVGAnimationElement",
    "SVGCircleElement", "SVGClipPathElement", "SVGComponentTransferFunctionElement",
    "SVGDefsElement", "SVGDescElement", "SVGElement", "SVGEllipseElement", "SVGFEBlendElement",
    "SVGFEColorMatrixElement", "SVGFEComponentTransferElement", "SVGFECompositeElement",
    "SVGFEConvolveMatrixElement", "SVGFEDiffuseLightingElement", "SVGFEDisplacementMapElement",
    "SVGFEDistantLightElement", "SVGFEDropShadowElement", "SVGFEFloodElement", "SVGFEFuncAElement",
    "SVGFEFuncBElement", "SVGFEFuncGElement", "SVGFEFuncRElement", "SVGFEGaussianBlurElement",
    "SVGFEImageElement", "SVGFEMergeElement", "SVGFEMergeNodeElement", "SVGFEMorphologyElement",
    "SVGFEOffsetElement", "SVGFEPointLightElement", "SVGFESpecularLightingElement",
    "SVGFESpotLightElement", "SVGFETileElement", "SVGFETurbulenceElement", "SVGFilterElement",
    "SVGForeignObjectElement", "SVGGElement", "SVGGeometryElement", "SVGGradientElement",
    "SVGGraphicsElement", "SVGImageElement", "SVGLength", "SVGLengthList", "SVGLineElement",
    "SVGLinearGradientElement", "SVGMPathElement", "SVGMarkerElement", "SVGMaskElement",
    "SVGMatrix", "SVGMetadataElement", "SVGNumber", "SVGNumberList", "SVGPathElement",
    "SVGPatternElement", "SVGPoint", "SVGPointList", "SVGPolygonElement", "SVGPolylineElement",
    "SVGPreserveAspectRatio", "SVGRadialGradientElement", "SVGRect", "SVGRectElement",
    "SVGSVGElement", "SVGScriptElement", "SVGSetElement", "SVGStopElement", "SVGStringList",
    "SVGStyleElement", "SVGSwitchElement", "SVGSymbolElement", "SVGTSpanElement",
    "SVGTextContentElement", "SVGTextElement", "SVGTextPathElement", "SVGTextPositioningElement",
    "SVGTitleElement", "SVGTransform", "SVGTransformList", "SVGUnitTypes", "SVGUseElement",
    "SVGViewElement", "Screen", "ScriptProcessorNode", "SecurityPolicyViolationEvent", "Selection",
    "Set", "ShadowRoot", "SourceBuffer", "SourceBufferList", "SpeechSynthesisEvent",
    "SpeechSynthesisUtterance", "StaticRange", "Storage", "StorageEvent", "String", "StyleSheet",
    "StyleSheetList", "Symbol", "SyntaxError", "Text", "TextDecoder", "TextEncoder", "TextMetrics",
    "TextTrack", "TextTrackCue", "TextTrackCueList", "TextTrackList", "TimeRanges", "TrackEvent",
    "TransitionEvent", "TreeWalker", "TypeError", "UIEvent", "URIError", "URL", "URLSearchParams",
    "Uint16Array", "Uint32Array", "Uint8Array", "Uint8ClampedArray", "VTTCue", "ValidityState",
    "VisualViewport", "WaveShaperNode", "WeakMap", "WeakSet", "WebAssembly", "WebGLActiveInfo",
    "WebGLBuffer", "WebGLContextEvent", "WebGLFramebuffer", "WebGLProgram", "WebGLQuery",
    "WebGLRenderbuffer", "WebGLRenderingContext", "WebGLSampler", "WebGLShader",
    "WebGLShaderPrecisionFormat", "WebGLSync", "WebGLTexture", "WebGLUniformLocation",
    "WebKitCSSMatrix", "WebSocket", "WheelEvent", "Window", "Worker", "XMLDocument",
    "XMLHttpRequest", "XMLHttpRequestEventTarget", "XMLHttpRequestUpload", "XMLSerializer",
    "XPathEvaluator", "XPathExpression", "XPathResult", "XSLTProcessor", "alert", "atob", "blur",
    "btoa", "cancelAnimationFrame", "captureEvents", "clearInterval", "clearTimeout", "close",
    "closed", "confirm", "console", "customElements", "decodeURI", "decodeURIComponent",
    "devicePixelRatio", "document", "encodeURI", "encodeURIComponent", "escape", "event", "fetch",
    "find", "focus", "frameElement", "frames", "getComputedStyle", "getSelection", "globalThis",
    "history", "indexedDB", "isFinite", "isNaN", "isSecureContext", "length", "location",
    "locationbar", "matchMedia", "menubar", "moveBy", "moveTo", "name", "navigator", "onabort",
    "onafterprint", "onanimationend", "onanimationiteration", "onanimationstart", "onbeforeprint",
    "onbeforeunload", "onblur", "oncanplay", "oncanplaythrough", "onchange", "onclick",
    "oncontextmenu", "oncuechange", "ondblclick", "ondrag", "ondragend", "ondragenter",
    "ondragleave", "ondragover", "ondragstart", "ondrop", "ondurationchange", "onemptied",
    "onended", "onerror", "onfocus", "ongotpointercapture", "onhashchange", "oninput", "oninvalid",
    "onkeydown", "onkeypress", "onkeyup", "onlanguagechange", "onload", "onloadeddata",
    "onloadedmetadata", "onloadstart", "onlostpointercapture", "onmessage", "onmousedown",
    "onmouseenter", "onmouseleave", "onmousemove", "onmouseout", "onmouseover", "onmouseup",
    "onoffline", "ononline", "onpagehide", "onpageshow", "onpause", "onplay", "onplaying",
    "onpointercancel", "onpointerdown", "onpointerenter", "onpointerleave", "onpointermove",
    "onpointerout", "onpointerover", "onpointerup", "onpopstate", "onprogress", "onratechange",
    "onrejectionhandled", "onreset", "onresize", "onscroll", "onseeked", "onseeking", "onselect",
    "onstalled", "onstorage", "onsubmit", "onsuspend", "ontimeupdate", "ontoggle",
    "ontransitioncancel", "ontransitionend", "ontransitionrun", "ontransitionstart",
    "onunhandledrejection", "onunload", "onvolumechange", "onwaiting", "onwebkitanimationend",
    "onwebkitanimationiteration", "onwebkitanimationstart", "onwebkittransitionend", "onwheel",
    "open", "opener", "origin", "outerHeight", "outerWidth", "parent", "parseFloat", "parseInt",
    "performance", "personalbar", "postMessage", "print", "prompt", "queueMicrotask",
    "releaseEvents", "requestAnimationFrame", "resizeBy", "resizeTo", "screen", "screenLeft",
    "screenTop", "screenX", "screenY", "scroll", "scrollBy", "scrollTo", "scrollbars", "self",
    "setInterval", "setTimeout", "speechSynthesis", "status", "statusbar", "stop", "toolbar",
    "top", "unescape", "webkitURL", "window",
];

#[rustfmt::skip]
fn is_pure_math_method(method: &str) -> bool {
    matches!(method,
        "abs" | "acos" | "acosh" | "asin" | "asinh" | "atan" | "atan2" | "atanh"
        | "cbrt" | "ceil" | "clz32" | "cos" | "cosh" | "exp" | "expm1" | "floor"
        | "fround" | "hypot" | "imul" | "log" | "log10" | "log1p" | "log2" | "max"
        | "min" | "pow" | "random" | "round" | "sign" | "sin" | "sinh" | "sqrt"
        | "tan" | "tanh" | "trunc"
    )
}

/// Whether calling `Global.method()` is side-effect-free (given pure arguments).
///
/// This is distinct from `is_known_global_property` which checks property READ safety.
/// For example, `Object.freeze` is safe to read but NOT safe to call (it mutates).
#[rustfmt::skip]
pub(super) fn is_pure_global_method_call(object: &str, method: &str) -> bool {
    match object {
        "Array" => matches!(method, "isArray" | "of"),
        "ArrayBuffer" => method == "isView",
        "Date" => matches!(method, "now" | "parse" | "UTC"),
        "Math" => is_pure_math_method(method),
        "Number" => matches!(method, "isFinite" | "isInteger" | "isNaN" | "isSafeInteger" | "parseFloat" | "parseInt"),
        // Only `Object.is` is unconditionally pure. The introspection methods
        // (`keys`, `getOwnPropertyDescriptor`, ...) can trigger Proxy traps on their
        // target, and `Object.create` reads its `properties` argument; both are
        // handled in `CallExpression::may_have_side_effects`.
        "Object" => method == "is",
        // `String.raw(template)` reads `template.raw` and throws a TypeError on a
        // missing/non-template argument, which is never provably safe — kept as not-pure.
        // `fromCharCode`/`fromCodePoint` coerce via `ToNumber` (throw-checked in
        // `CallExpression::may_have_side_effects`).
        "String" => matches!(method, "fromCharCode" | "fromCodePoint"),
        // `Symbol.keyFor(sym)` requires a Symbol argument and throws otherwise, which is
        // never provable — kept as not-pure.
        "Symbol" => method == "for",
        // `URL.canParse(url)` has a required first argument (throws with none); the
        // arg-count check lives in `CallExpression::may_have_side_effects`.
        "URL" => method == "canParse",
        _ if is_typed_array_constructor(object) => method == "of",
        _ => false,
    }
}

/// Whether the property read on a known global is side-effect-free.
///
/// For example, `Math.PI`, `console.log`, `Object.keys` are all side-effect-free to read.
/// Lists ported from Rolldown's global_reference.rs.
#[rustfmt::skip]
pub(super) fn is_known_global_property(global: &str, property: &str) -> bool {
    match global {
        "Math" => matches!(property, "E" | "LN10" | "LN2" | "LOG10E" | "LOG2E" | "PI" | "SQRT1_2" | "SQRT2")
            || is_pure_math_method(property),
        "console" => matches!(property,
            "assert" | "clear" | "count" | "countReset" | "debug" | "dir" | "dirxml"
            | "error" | "group" | "groupCollapsed" | "groupEnd" | "info" | "log"
            | "table" | "time" | "timeEnd" | "timeLog" | "trace" | "warn"
        ),
        "Object" => matches!(property,
            "assign" | "create" | "defineProperties" | "defineProperty" | "entries"
            | "freeze" | "fromEntries" | "getOwnPropertyDescriptor"
            | "getOwnPropertyDescriptors" | "getOwnPropertyNames"
            | "getOwnPropertySymbols" | "getPrototypeOf" | "is" | "isExtensible"
            | "isFrozen" | "isSealed" | "keys" | "preventExtensions" | "prototype"
            | "seal" | "setPrototypeOf" | "values"
        ),
        "Reflect" => matches!(property,
            "apply" | "construct" | "defineProperty" | "deleteProperty" | "get"
            | "getOwnPropertyDescriptor" | "getPrototypeOf" | "has" | "isExtensible"
            | "ownKeys" | "preventExtensions" | "set" | "setPrototypeOf"
        ),
        "Symbol" => matches!(property,
            "asyncDispose" | "asyncIterator" | "dispose" | "hasInstance"
            | "isConcatSpreadable" | "iterator" | "match" | "matchAll" | "replace"
            | "search" | "species" | "split" | "toPrimitive" | "toStringTag"
            | "unscopables"
        ),
        "JSON" => matches!(property, "parse" | "stringify"),
        _ => false,
    }
}

/// Whether a 3-level property chain on a known global is side-effect-free.
///
/// For example, `Object.prototype.hasOwnProperty` is side-effect-free to read.
/// List ported from Rolldown's `OBJECT_PROTOTYPE_THIRD_PROP`.
#[rustfmt::skip]
pub(super) fn is_known_global_property_deep(global: &str, middle: &str, property: &str) -> bool {
    global == "Object" && middle == "prototype" && matches!(property,
        "__defineGetter__" | "__defineSetter__" | "__lookupGetter__" | "__lookupSetter__"
        | "hasOwnProperty" | "isPrototypeOf" | "propertyIsEnumerable" | "toLocaleString"
        | "toString" | "unwatch" | "valueOf" | "watch"
    )
}

#[cfg(test)]
mod test {
    use super::{KNOWN_GLOBAL_IDENTIFIERS, is_known_global_identifier};

    #[test]
    fn known_global_identifiers_table_is_sorted_and_unique() {
        // `binary_search` relies on byte-order sorting and duplicates are pointless.
        for pair in KNOWN_GLOBAL_IDENTIFIERS.windows(2) {
            assert!(pair[0] < pair[1], "`{}` must sort before `{}`", pair[0], pair[1]);
        }
    }

    #[test]
    fn known_global_identifiers_lookup() {
        assert!(is_known_global_identifier("Array"));
        assert!(is_known_global_identifier("window"));
        assert!(is_known_global_identifier("AbortController"));
        assert!(is_known_global_identifier("webkitURL"));
        assert!(!is_known_global_identifier("NaN"));
        assert!(!is_known_global_identifier("foo"));
        assert!(!is_known_global_identifier(""));
    }
}
