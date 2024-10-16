use std::borrow::Cow;

use cow_utils::CowUtils;
use itertools::Itertools;
use once_cell::sync::Lazy;
use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeName, JSXElementName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use phf::{phf_map, phf_set, Map, Set};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserialize;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::get_jsx_attribute_name,
    AstNode,
};

fn invalid_prop_on_tag(span: Span, prop: &str, tag: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid property found")
        .with_help(format!("Property '{prop}' is only allowed on: {tag}"))
        .with_label(span)
}

fn data_lowercase_required(span: Span, suggested_prop: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "React does not recognize data-* props with uppercase characters on a DOM element",
    )
    .with_help(format!("Use '{suggested_prop}' instead"))
    .with_label(span)
}

fn unknown_prop_with_standard_name(span: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unknown property found")
        .with_help(format!("Use '{x1}' instead"))
        .with_label(span)
}

fn unknown_prop(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unknown property found")
        .with_help("Remove unknown property")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnknownProperty(Box<NoUnknownPropertyConfig>);

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoUnknownPropertyConfig {
    #[serde(default)]
    ignore: FxHashSet<Cow<'static, str>>,
    #[serde(default)]
    require_data_lowercase: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow usage of unknown DOM property.
    ///
    /// ### Why is this bad?
    /// You can use unknown property name that has no effect.
    ///
    /// ### Example
    /// ```jsx
    ///  // Unknown properties
    ///  const Hello = <div class="hello">Hello World</div>;
    ///  const Alphabet = <div abc="something">Alphabet</div>;
    ///
    ///  // Invalid aria-* attribute
    ///  const IconButton = <div aria-foo="bar" />;
    /// ```
    NoUnknownProperty,
    restriction,
    pending
);

const ATTRIBUTE_TAGS_MAP: Map<&'static str, Set<&'static str>> = phf_map! {
    "abbr" => phf_set! {"th", "td"},
    "charset" => phf_set! {"meta"},
    "checked" => phf_set! {"input"},
    // image is required for SVG support, all other tags are HTML.
    "crossOrigin" => phf_set! {"script", "img", "video", "audio", "link", "image"},
    "displaystyle" => phf_set! {"math"},
    // https://html.spec.whatwg.org/multipage/links.html#downloading-resources
    "download" => phf_set! {"a", "area"},
    "fill" => phf_set! {
         // https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/fill
         // Fill color
         "altGlyph",
         "circle",
         "ellipse",
         "g",
         "line",
         "marker",
         "mask",
         "path",
         "polygon",
         "polyline",
         "rect",
         "svg",
         "symbol",
         "text",
         "textPath",
         "tref",
         "tspan",
         "use",
         // Animation final state
         "animate",
         "animateColor",
         "animateMotion",
         "animateTransform",
         "set",
    },
    "focusable" => phf_set! {"svg"},
    "imageSizes" => phf_set! {"link"},
    "imageSrcSet" => phf_set! {"link"},
    "property" => phf_set! {"meta"},
    "viewBox" => phf_set! {"marker", "pattern", "svg", "symbol", "view"},
    "as" => phf_set! {"link"},
    "align" => phf_set! {
        "applet", "caption", "col", "colgroup", "hr", "iframe", "img", "table", "tbody", "td",
        "tfoot", "th", "thead", "tr",
    },
    // deprecated, but known
    "valign" => phf_set! {"tr", "td", "th", "thead", "tbody", "tfoot", "colgroup", "col"}, // deprecated, but known
    "noModule" => phf_set! {"script"},
    // Media events allowed only on audio and video tags, see https://github.com/facebook/react/blob/256aefbea1449869620fb26f6ec695536ab453f5/CHANGELOG.md#notable-enhancements
    "onAbort" => phf_set! {"audio", "video"},
    "onCancel" => phf_set! {"dialog"},
    "onCanPlay" => phf_set! {"audio", "video"},
    "onCanPlayThrough" => phf_set! {"audio", "video"},
    "onClose" => phf_set! {"dialog"},
    "onDurationChange" => phf_set! {"audio", "video"},
    "onEmptied" => phf_set! {"audio", "video"},
    "onEncrypted" => phf_set! {"audio", "video"},
    "onEnded" => phf_set! {"audio", "video"},
    "onError" => phf_set! {"audio", "video", "img", "link", "source", "script", "picture", "iframe"},
    "onLoad" => phf_set! {"script", "img", "link", "picture", "iframe", "object", "source"},
    "onLoadedData" => phf_set! {"audio", "video"},
    "onLoadedMetadata" => phf_set! {"audio", "video"},
    "onLoadStart" => phf_set! {"audio", "video"},
    "onPause" => phf_set! {"audio", "video"},
    "onPlay" => phf_set! {"audio", "video"},
    "onPlaying" => phf_set! {"audio", "video"},
    "onProgress" => phf_set! {"audio", "video"},
    "onRateChange" => phf_set! {"audio", "video"},
    "onResize" => phf_set! {"audio", "video"},
    "onSeeked" => phf_set! {"audio", "video"},
    "onSeeking" => phf_set! {"audio", "video"},
    "onStalled" => phf_set! {"audio", "video"},
    "onSuspend" => phf_set! {"audio", "video"},
    "onTimeUpdate" => phf_set! {"audio", "video"},
    "onVolumeChange" => phf_set! {"audio", "video"},
    "onWaiting" => phf_set! {"audio", "video"},
    "autoPictureInPicture" => phf_set! {"video"},
    "controls" => phf_set! {"audio", "video"},
    "controlsList" => phf_set! {"audio", "video"},
    "disablePictureInPicture" => phf_set! {"video"},
    "disableRemotePlayback" => phf_set! {"audio", "video"},
    "loop" => phf_set! {"audio", "video"},
    "muted" => phf_set! {"audio", "video"},
    "playsInline" => phf_set! {"video"},
    "allowFullScreen" => phf_set! {"iframe", "video"},
    "webkitAllowFullScreen" => phf_set! {"iframe", "video"},
    "mozAllowFullScreen" => phf_set! {"iframe", "video"},
    "poster" => phf_set! {"video"},
    "preload" => phf_set! {"audio", "video"},
    "scrolling" => phf_set! {"iframe"},
    "returnValue" => phf_set! {"dialog"},
    "webkitDirectory" => phf_set! {"input"},
};

const DOM_PROPERTIES_NAMES: Set<&'static str> = phf_set! {
    // Global attributes - can be used on any HTML/DOM element
    // See https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes
    "dir", "draggable", "hidden", "id", "lang", "nonce", "part", "slot", "style", "title", "translate", "inert",
    // Element specific attributes
    // See https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes (includes global attributes too)
    // To be considered if these should be added also to ATTRIBUTE_TAGS_MAP
    "accept", "action", "allow", "alt", "as", "async", "buffered", "capture", "challenge", "cite", "code", "cols",
    "content", "coords", "csp", "data", "decoding", "default", "defer", "disabled", "form",
    "headers", "height", "high", "href", "icon", "importance", "integrity", "kind", "label",
    "language", "loading", "list", "loop", "low", "manifest", "max", "media", "method", "min", "multiple", "muted",
    "name", "open", "optimum", "pattern", "ping", "placeholder", "poster", "preload", "profile",
    "rel", "required", "reversed", "role", "rows", "sandbox", "scope", "seamless", "selected", "shape", "size", "sizes",
    "span", "src", "start", "step", "summary", "target", "type", "value", "width", "wmode", "wrap",
    // SVG attributes
    // See https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute
    "accumulate", "additive", "alphabetic", "amplitude", "ascent", "azimuth", "bbox", "begin",
    "bias", "by", "clip", "color", "cursor", "cx", "cy", "d", "decelerate", "descent", "direction",
    "display", "divisor", "dur", "dx", "dy", "elevation", "end", "exponent", "fill", "filter",
    "format", "from", "fr", "fx", "fy", "g1", "g2", "hanging", "hreflang", "ideographic",
    "in", "in2", "intercept", "k", "k1", "k2", "k3", "k4", "kerning", "local", "mask", "mode",
    "offset", "opacity", "operator", "order", "orient", "orientation", "origin", "overflow", "path",
    "points", "r", "radius", "restart", "result", "rotate", "rx", "ry", "scale",
    "seed", "slope", "spacing", "speed", "stemh", "stemv", "string", "stroke", "to", "transform",
    "u1", "u2", "unicode", "values", "version", "visibility", "widths", "x", "x1", "x2", "xmlns",
    "y", "y1", "y2", "z",
    // OpenGraph meta tag attributes
    "property",
    // React specific attributes
    "ref", "key", "children",
    // Non-standard
    "results", "security",
    // Video specific
    "controls",
    // TWO WORD DOM_PROPERTIES_NAMES

    // Global attributes - can be used on any HTML/DOM element
    // See https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes
    "accessKey", "autoCapitalize", "autoFocus", "contentEditable", "enterKeyHint", "exportParts",
    "inputMode", "itemID", "itemRef", "itemProp", "itemScope", "itemType", "spellCheck", "tabIndex",
    // Element specific attributes
    // See https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes (includes global attributes too)
    // To be considered if these should be added also to ATTRIBUTE_TAGS_MAP
    "acceptCharset", "autoComplete", "autoPlay", "border", "cellPadding", "cellSpacing", "classID", "codeBase",
    "colSpan", "contextMenu", "dateTime", "encType", "formAction", "formEncType", "formMethod", "formNoValidate", "formTarget",
    "frameBorder", "hrefLang", "httpEquiv", "imageSizes", "imageSrcSet", "isMap", "keyParams", "keyType", "marginHeight", "marginWidth",
    "maxLength", "mediaGroup", "minLength", "noValidate", "onAnimationEnd", "onAnimationIteration", "onAnimationStart",
    "onBlur", "onChange", "onClick", "onContextMenu", "onCopy", "onCompositionEnd", "onCompositionStart",
    "onCompositionUpdate", "onCut", "onDoubleClick", "onDrag", "onDragEnd", "onDragEnter", "onDragExit", "onDragLeave",
    "onError", "onFocus", "onInput", "onKeyDown", "onKeyPress", "onKeyUp", "onLoad", "onWheel", "onDragOver",
    "onDragStart", "onDrop", "onMouseDown", "onMouseEnter", "onMouseLeave", "onMouseMove", "onMouseOut", "onMouseOver",
    "onMouseUp", "onPaste", "onScroll", "onSelect", "onSubmit", "onToggle", "onTransitionEnd", "radioGroup", "readOnly", "referrerPolicy",
    "rowSpan", "srcDoc", "srcLang", "srcSet", "useMap",
    // SVG attributes
    // See https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute
    "crossOrigin", "accentHeight", "alignmentBaseline", "arabicForm", "attributeName",
    "attributeType", "baseFrequency", "baselineShift", "baseProfile", "calcMode", "capHeight",
    "clipPathUnits", "clipPath", "clipRule", "colorInterpolation", "colorInterpolationFilters",
    "colorProfile", "colorRendering", "contentScriptType", "contentStyleType", "diffuseConstant",
    "dominantBaseline", "edgeMode", "enableBackground", "fillOpacity", "fillRule", "filterRes",
    "filterUnits", "floodColor", "floodOpacity", "fontFamily", "fontSize", "fontSizeAdjust",
    "fontStretch", "fontStyle", "fontVariant", "fontWeight", "glyphName",
    "glyphOrientationHorizontal", "glyphOrientationVertical", "glyphRef", "gradientTransform",
    "gradientUnits", "horizAdvX", "horizOriginX", "imageRendering", "kernelMatrix",
    "kernelUnitLength", "keyPoints", "keySplines", "keyTimes", "lengthAdjust", "letterSpacing",
    "lightingColor", "limitingConeAngle", "markerEnd", "markerMid", "markerStart", "markerHeight",
    "markerUnits", "markerWidth", "maskContentUnits", "maskUnits", "mathematical", "numOctaves",
    "overlinePosition", "overlineThickness", "panose1", "paintOrder", "pathLength",
    "patternContentUnits", "patternTransform", "patternUnits", "pointerEvents", "pointsAtX",
    "pointsAtY", "pointsAtZ", "preserveAlpha", "preserveAspectRatio", "primitiveUnits",
    "refX", "refY", "rendering-intent", "repeatCount", "repeatDur",
    "requiredExtensions", "requiredFeatures", "shapeRendering", "specularConstant",
    "specularExponent", "spreadMethod", "startOffset", "stdDeviation", "stitchTiles", "stopColor",
    "stopOpacity", "strikethroughPosition", "strikethroughThickness", "strokeDasharray",
    "strokeDashoffset", "strokeLinecap", "strokeLinejoin", "strokeMiterlimit", "strokeOpacity",
    "strokeWidth", "surfaceScale", "systemLanguage", "tableValues", "targetX", "targetY",
    "textAnchor", "textDecoration", "textRendering", "textLength", "transformOrigin",
    "underlinePosition", "underlineThickness", "unicodeBidi", "unicodeRange", "unitsPerEm",
    "vAlphabetic", "vHanging", "vIdeographic", "vMathematical", "vectorEffect", "vertAdvY",
    "vertOriginX", "vertOriginY", "viewBox", "viewTarget", "wordSpacing", "writingMode", "xHeight",
    "xChannelSelector", "xlinkActuate", "xlinkArcrole", "xlinkHref", "xlinkRole", "xlinkShow",
    "xlinkTitle", "xlinkType", "xmlBase", "xmlLang", "xmlnsXlink", "xmlSpace", "yChannelSelector",
    "zoomAndPan",
    // Safari/Apple specific, no listing available
    "autoCorrect", // https://stackoverflow.com/questions/47985384/html-autocorrect-for-text-input-is-not-working
    "autoSave", // https://stackoverflow.com/questions/25456396/what-is-autosave-attribute-supposed-to-do-how-do-i-use-it
    // React specific attributes https://reactjs.org/docs/dom-elements.html#differences-in-attributes
    "className", "dangerouslySetInnerHTML", "defaultValue", "defaultChecked", "htmlFor",
    // Events" capture events
    "onBeforeInput",
    "onInvalid", "onReset", "onTouchCancel", "onTouchEnd", "onTouchMove", "onTouchStart", "suppressContentEditableWarning", "suppressHydrationWarning",
    "onAbort", "onCanPlay", "onCanPlayThrough", "onDurationChange", "onEmptied", "onEncrypted", "onEnded",
    "onLoadedData", "onLoadedMetadata", "onLoadStart", "onPause", "onPlay", "onPlaying", "onProgress", "onRateChange", "onResize",
    "onSeeked", "onSeeking", "onStalled", "onSuspend", "onTimeUpdate", "onVolumeChange", "onWaiting",
    "onCopyCapture", "onCutCapture", "onPasteCapture", "onCompositionEndCapture", "onCompositionStartCapture", "onCompositionUpdateCapture",
    "onFocusCapture", "onBlurCapture", "onChangeCapture", "onBeforeInputCapture", "onInputCapture", "onResetCapture", "onSubmitCapture",
    "onInvalidCapture", "onLoadCapture", "onErrorCapture", "onKeyDownCapture", "onKeyPressCapture", "onKeyUpCapture",
    "onAbortCapture", "onCanPlayCapture", "onCanPlayThroughCapture", "onDurationChangeCapture", "onEmptiedCapture", "onEncryptedCapture",
    "onEndedCapture", "onLoadedDataCapture", "onLoadedMetadataCapture", "onLoadStartCapture", "onPauseCapture", "onPlayCapture",
    "onPlayingCapture", "onProgressCapture", "onRateChangeCapture", "onSeekedCapture", "onSeekingCapture", "onStalledCapture", "onSuspendCapture",
    "onTimeUpdateCapture", "onVolumeChangeCapture", "onWaitingCapture", "onSelectCapture", "onTouchCancelCapture", "onTouchEndCapture",
    "onTouchMoveCapture", "onTouchStartCapture", "onScrollCapture", "onWheelCapture", "onAnimationEndCapture",
    "onAnimationStartCapture", "onTransitionEndCapture",
    "onAuxClick", "onAuxClickCapture", "onClickCapture", "onContextMenuCapture", "onDoubleClickCapture",
    "onDragCapture", "onDragEndCapture", "onDragEnterCapture", "onDragExitCapture", "onDragLeaveCapture",
    "onDragOverCapture", "onDragStartCapture", "onDropCapture", "onMouseDownCapture",
    "onMouseMoveCapture", "onMouseOutCapture", "onMouseOverCapture", "onMouseUpCapture",
    // Video specific
    "autoPictureInPicture", "controlsList", "disablePictureInPicture", "disableRemotePlayback",

    // React on props
    "onGotPointerCaptureCapture",
    "onLostPointerCapture",
    "onLostPointerCaptureCapture",
    "onPointerCancel",
    "onPointerCancelCapture",
    "onPointerDown",
    "onPointerDownCapture",
    "onPointerEnter",
    "onPointerEnterCapture",
    "onPointerLeave",
    "onPointerLeaveCapture",
    "onPointerMove",
    "onPointerMoveCapture",
    "onPointerOut",
    "onPointerOutCapture",
    "onPointerOver",
    "onPointerOverCapture",
    "onPointerUp",
    "onPointerUpCapture",
};

const ARIA_PROPERTIES: Set<&'static str> = phf_set! {
    // See https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes
    // Global attributes
    "aria-atomic", "aria-braillelabel", "aria-brailleroledescription", "aria-busy", "aria-controls", "aria-current",
    "aria-describedby", "aria-description", "aria-details",
    "aria-disabled", "aria-dropeffect", "aria-errormessage", "aria-flowto", "aria-grabbed", "aria-haspopup",
    "aria-hidden", "aria-invalid", "aria-keyshortcuts", "aria-label", "aria-labelledby", "aria-live",
    "aria-owns", "aria-relevant", "aria-roledescription",
    // Widget attributes
    "aria-autocomplete", "aria-checked", "aria-expanded", "aria-level", "aria-modal", "aria-multiline", "aria-multiselectable",
    "aria-orientation", "aria-placeholder", "aria-pressed", "aria-readonly", "aria-required", "aria-selected",
    "aria-sort", "aria-valuemax", "aria-valuemin", "aria-valuenow", "aria-valuetext",
    // Relationship attributes
    "aria-activedescendant", "aria-colcount", "aria-colindex", "aria-colindextext", "aria-colspan",
    "aria-posinset", "aria-rowcount", "aria-rowindex", "aria-rowindextext", "aria-rowspan", "aria-setsize",
};

const DOM_ATTRIBUTES_TO_CAMEL: Map<&'static str, &'static str> = phf_map! {
    "accept-charset" => "acceptCharset",
    "class" => "className",
    "http-equiv" => "httpEquiv",
    "crossorigin" => "crossOrigin",
    "for" => "htmlFor",
    "nomodule" => "noModule",
    // svg
    "accent-height" => "accentHeight",
    "alignment-baseline" => "alignmentBaseline",
    "arabic-form" => "arabicForm",
    "baseline-shift" => "baselineShift",
    "cap-height" => "capHeight",
    "clip-path" => "clipPath",
    "clip-rule" => "clipRule",
    "color-interpolation" => "colorInterpolation",
    "color-interpolation-filters" => "colorInterpolationFilters",
    "color-profile" => "colorProfile",
    "color-rendering" => "colorRendering",
    "dominant-baseline" => "dominantBaseline",
    "enable-background" => "enableBackground",
    "fill-opacity" => "fillOpacity",
    "fill-rule" => "fillRule",
    "flood-color" => "floodColor",
    "flood-opacity" => "floodOpacity",
    "font-family" => "fontFamily",
    "font-size" => "fontSize",
    "font-size-adjust" => "fontSizeAdjust",
    "font-stretch" => "fontStretch",
    "font-style" => "fontStyle",
    "font-variant" => "fontVariant",
    "font-weight" => "fontWeight",
    "glyph-name" => "glyphName",
    "glyph-orientation-horizontal" => "glyphOrientationHorizontal",
    "glyph-orientation-vertical" => "glyphOrientationVertical",
    "horiz-adv-x" => "horizAdvX",
    "horiz-origin-x" => "horizOriginX",
    "image-rendering" => "imageRendering",
    "letter-spacing" => "letterSpacing",
    "lighting-color" => "lightingColor",
    "marker-end" => "markerEnd",
    "marker-mid" => "markerMid",
    "marker-start" => "markerStart",
    "overline-position" => "overlinePosition",
    "overline-thickness" => "overlineThickness",
    "paint-order" => "paintOrder",
    "panose-1" => "panose1",
    "pointer-events" => "pointerEvents",
    "rendering-intent" => "renderingIntent",
    "shape-rendering" => "shapeRendering",
    "stop-color" => "stopColor",
    "stop-opacity" => "stopOpacity",
    "strikethrough-position" => "strikethroughPosition",
    "strikethrough-thickness" => "strikethroughThickness",
    "stroke-dasharray" => "strokeDasharray",
    "stroke-dashoffset" => "strokeDashoffset",
    "stroke-linecap" => "strokeLinecap",
    "stroke-linejoin" => "strokeLinejoin",
    "stroke-miterlimit" => "strokeMiterlimit",
    "stroke-opacity" => "strokeOpacity",
    "stroke-width" => "strokeWidth",
    "text-anchor" => "textAnchor",
    "text-decoration" => "textDecoration",
    "text-rendering" => "textRendering",
    "underline-position" => "underlinePosition",
    "underline-thickness" => "underlineThickness",
    "unicode-bidi" => "unicodeBidi",
    "unicode-range" => "unicodeRange",
    "units-per-em" => "unitsPerEm",
    "v-alphabetic" => "vAlphabetic",
    "v-hanging" => "vHanging",
    "v-ideographic" => "vIdeographic",
    "v-mathematical" => "vMathematical",
    "vector-effect" => "vectorEffect",
    "vert-adv-y" => "vertAdvY",
    "vert-origin-x" => "vertOriginX",
    "vert-origin-y" => "vertOriginY",
    "word-spacing" => "wordSpacing",
    "writing-mode" => "writingMode",
    "x-height" => "xHeight",
    "xlink:actuate" => "xlinkActuate",
    "xlink:arcrole" => "xlinkArcrole",
    "xlink:href" => "xlinkHref",
    "xlink:role" => "xlinkRole",
    "xlink:show" => "xlinkShow",
    "xlink:title" => "xlinkTitle",
    "xlink:type" => "xlinkType",
    "xml:base" => "xmlBase",
    "xml:lang" => "xmlLang",
    "xml:space" => "xmlSpace",
};

const DOM_PROPERTIES_IGNORE_CASE: [&str; 5] = [
    "charset",
    "allowFullScreen",
    "webkitAllowFullScreen",
    "mozAllowFullScreen",
    "webkitDirectory",
];

static DOM_PROPERTIES_LOWER_MAP: Lazy<FxHashMap<String, &'static str>> = Lazy::new(|| {
    DOM_PROPERTIES_NAMES
        .iter()
        .map(|it| (it.cow_to_lowercase().into_owned(), *it))
        .collect::<FxHashMap<_, _>>()
});

/// Checks if an attribute name is a valid `data-*` attribute:
/// - Name starts with "data-" and has alphanumeric words (browsers require lowercase, but React and TS lowercase them),
/// - Does not start with any casing of "xml"
/// - Words are separated by hyphens (-) (which is also called "kebab case" or "dash case")
fn is_valid_data_attr(name: &str) -> bool {
    if !name.starts_with("data-") {
        return false;
    }

    if name.cow_to_lowercase().starts_with("data-xml") {
        return false;
    }

    let data_name = &name["data-".len()..];
    if data_name.is_empty() {
        return false;
    }

    data_name.chars().all(|c| c != ':')
}

/// Checks if a tag name matches the HTML tag conventions.
fn matches_html_tag_conventions(tag: &str) -> bool {
    tag.char_indices().all(|(i, c)| if i == 0 { c.is_ascii_lowercase() } else { c != '-' })
}

fn normalize_attribute_case(name: &str) -> &str {
    DOM_PROPERTIES_IGNORE_CASE
        .iter()
        .find(|camel_name| camel_name.eq_ignore_ascii_case(name))
        .unwrap_or(&name)
}
fn has_uppercase(name: &str) -> bool {
    name.contains(char::is_uppercase)
}

impl Rule for NoUnknownProperty {
    fn from_configuration(value: serde_json::Value) -> Self {
        value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .map_or_else(Self::default, |value| Self(Box::new(value)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(el) = &node.kind() else {
            return;
        };
        let JSXElementName::Identifier(ident) = &el.name else {
            return;
        };
        let el_type = ident.name.as_str();

        // fbt/fbs nodes are bonkers, let's not go there
        if !el_type.starts_with(char::is_lowercase) || el_type == "fbt" || el_type == "fbs" {
            return;
        }

        let is_valid_html_tag = matches_html_tag_conventions(el_type)
            && el.attributes.iter().all(|attr| {
                let JSXAttributeItem::Attribute(jsx_attr) = attr else {
                    return true;
                };
                let JSXAttributeName::Identifier(ident) = &jsx_attr.name else {
                    return true;
                };
                ident.name.as_str() != "is"
            });

        el.attributes
            .iter()
            .filter_map(|attr| match &attr {
                JSXAttributeItem::Attribute(regular) => Some(&**regular),
                JSXAttributeItem::SpreadAttribute(_) => None,
            })
            .for_each(|attr| {
                let span = attr.name.span();
                let actual_name = get_jsx_attribute_name(&attr.name);
                if self.0.ignore.contains(&(actual_name)) {
                    return;
                };
                if is_valid_data_attr(&actual_name) {
                    if self.0.require_data_lowercase && has_uppercase(&actual_name) {
                        ctx.diagnostic(data_lowercase_required(
                            span,
                            &actual_name.cow_to_lowercase(),
                        ));
                    }
                    return;
                };
                if ARIA_PROPERTIES.contains(&actual_name) || !is_valid_html_tag {
                    return;
                };
                let name = normalize_attribute_case(&actual_name);
                if let Some(tags) = ATTRIBUTE_TAGS_MAP.get(name) {
                    if !tags.contains(el_type) {
                        ctx.diagnostic(invalid_prop_on_tag(
                            span,
                            &actual_name,
                            &tags.iter().join(", "),
                        ));
                    }
                    return;
                }

                if DOM_PROPERTIES_NAMES.contains(name) {
                    return;
                }

                DOM_PROPERTIES_LOWER_MAP
                    .get(&name.cow_to_lowercase().into_owned())
                    .or_else(|| DOM_ATTRIBUTES_TO_CAMEL.get(name))
                    .map_or_else(
                        || {
                            ctx.diagnostic(unknown_prop(span));
                        },
                        |prop| {
                            ctx.diagnostic(unknown_prop_with_standard_name(span, prop));
                        },
                    );
            });
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"<App class="bar" />;"#, None),
        (r#"<App for="bar" />;"#, None),
        (r#"<App someProp="bar" />;"#, None),
        (r#"<Foo.bar for="bar" />;"#, None),
        (r#"<App accept-charset="bar" />;"#, None),
        (r#"<App http-equiv="bar" />;"#, None),
        (r#"<App xlink:href="bar" />;"#, None),
        (r#"<App clip-path="bar" />;"#, None),
        (r#"<div className="bar"></div>;"#, None),
        (r"<div onMouseDown={this._onMouseDown}></div>;", None),
        (r#"<a href="someLink" download="foo">Read more</a>"#, None),
        (r#"<area download="foo" />"#, None),
        (r#"<img src="cat_keyboard.jpeg" alt="A cat sleeping on a keyboard" align="top" />"#, None),
        (r#"<input type="password" required />"#, None),
        (r#"<input ref={this.input} type="radio" />"#, None),
        (r#"<input type="file" webkitdirectory="" />"#, None),
        (r#"<input type="file" webkitDirectory="" />"#, None),
        (r#"<div inert children="anything" />"#, None),
        (r#"<iframe scrolling="?" onLoad={a} onError={b} align="top" />"#, None),
        (r#"<input key="bar" type="radio" />"#, None),
        (r"<button disabled>You cannot click me</button>;", None),
        (
            r#"<svg key="lock" viewBox="box" fill={10} d="d" stroke={1} strokeWidth={2} strokeLinecap={3} strokeLinejoin={4} transform="something" clipRule="else" x1={5} x2="6" y1="7" y2="8"></svg>"#,
            None,
        ),
        (r#"<g fill="\#7B82A0" fillRule="evenodd"></g>"#, None),
        (r#"<mask fill="\#7B82A0"></mask>"#, None),
        (r#"<symbol fill="\#7B82A0"></symbol>"#, None),
        (r#"<meta property="og:type" content="website" />"#, None),
        (
            r#"<input type="checkbox" checked={checked} disabled={disabled} id={id} onChange={onChange} />"#,
            None,
        ),
        (r"<video playsInline />", None),
        (r"<img onError={foo} onLoad={bar} />", None),
        (r"<picture inert={false} onError={foo} onLoad={bar} />", None),
        (r"<iframe onError={foo} onLoad={bar} />", None),
        (r"<script onLoad={bar} onError={foo} />", None),
        (r"<source onLoad={bar} onError={foo} />", None),
        (r"<link onLoad={bar} onError={foo} />", None),
        (
            r#"<link rel="preload" as="image" href="someHref" imageSrcSet="someImageSrcSet" imageSizes="someImageSizes" />"#,
            None,
        ),
        (r"<object onLoad={bar} />", None),
        (r"<video allowFullScreen webkitAllowFullScreen mozAllowFullScreen />", None),
        (r"<iframe allowFullScreen webkitAllowFullScreen mozAllowFullScreen />", None),
        (r#"<table border="1" />"#, None),
        (r#"<th abbr="abbr" />"#, None),
        (r#"<td abbr="abbr" />"#, None),
        (r"<div onPointerDown={this.onDown} onPointerUp={this.onUp} />", None),
        (r#"<input type="checkbox" defaultChecked={this.state.checkbox} />"#, None),
        (
            r"<div onTouchStart={this.startAnimation} onTouchEnd={this.stopAnimation} onTouchCancel={this.cancel} onTouchMove={this.move} onMouseMoveCapture={this.capture} onTouchCancelCapture={this.log} />",
            None,
        ),
        (r#"<meta charset="utf-8" />;"#, None),
        (r#"<meta charSet="utf-8" />;"#, None),
        (r#"<div class="foo" is="my-elem"></div>;"#, None),
        (r#"<div {...this.props} class="foo" is="my-elem"></div>;"#, None),
        (r#"<atom-panel class="foo"></atom-panel>;"#, None),
        (r#"<div data-foo="bar"></div>;"#, None),
        (r#"<div data-foo-bar="baz"></div>;"#, None),
        (r#"<div data-parent="parent"></div>;"#, None),
        (r#"<div data-index-number="1234"></div>;"#, None),
        (r#"<div data-e2e-id="5678"></div>;"#, None),
        (r#"<div data-testID="bar" data-under_sCoRe="bar" />;"#, None),
        (
            r#"<div data-testID="bar" data-under_sCoRe="bar" />;"#,
            Some(serde_json::json!([{ "requireDataLowercase": false }])),
        ),
        (r#"<div class="bar"></div>;"#, Some(serde_json::json!([{ "ignore": ["class"] }]))),
        (r#"<div someProp="bar"></div>;"#, Some(serde_json::json!([{ "ignore": ["someProp"] }]))),
        (r"<div css={{flex: 1}}></div>;", Some(serde_json::json!([{ "ignore": ["css"] }]))),
        (r#"<button aria-haspopup="true">Click me to open pop up</button>;"#, None),
        (r#"<button aria-label="Close" onClick={someThing.close} />;"#, None),
        (r"<script crossOrigin noModule />", None),
        (r"<audio crossOrigin />", None),
        (r"<svg focusable><image crossOrigin /></svg>", None),
        (r"<details onToggle={this.onToggle}>Some details</details>", None),
        (
            r#"<path fill="pink" d="M 10,30 A 20,20 0,0,1 50,30 A 20,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 10,30 z"></path>"#,
            None,
        ),
        (r#"<line fill="pink" x1="0" y1="80" x2="100" y2="20"></line>"#, None),
        (r#"<link as="audio">Audio content</link>"#, None),
        (
            r#"<video controlsList="nodownload" controls={this.controls} loop={true} muted={false} src={this.videoSrc} playsInline={true} onResize={this.onResize}></video>"#,
            None,
        ),
        (
            r#"<audio controlsList="nodownload" controls={this.controls} crossOrigin="anonymous" disableRemotePlayback loop muted preload="none" src="something" onAbort={this.abort} onDurationChange={this.durationChange} onEmptied={this.emptied} onEnded={this.end} onError={this.error} onResize={this.onResize}></audio>"#,
            None,
        ),
        (
            r#"<marker id={markerId} viewBox="0 0 2 2" refX="1" refY="1" markerWidth="1" markerHeight="1" orient="auto" />"#,
            None,
        ),
        (r#"<pattern id="pattern" viewBox="0,0,10,10" width="10%" height="10%" />"#, None),
        (r#"<symbol id="myDot" width="10" height="10" viewBox="0 0 2 2" />"#, None),
        (r#"<view id="one" viewBox="0 0 100 100" />"#, None),
        (r#"<hr align="top" />"#, None),
        (r#"<applet align="top" />"#, None),
        (r#"<marker fill="\#000" />"#, None),
        (
            r#"<dialog onClose={handler} open id="dialog" returnValue="something" onCancel={handler2} />"#,
            None,
        ),
        (
            r#"
			        <table align="top">
			          <caption align="top">Table Caption</caption>
			          <colgroup valign="top" align="top">
			            <col valign="top" align="top"/>
			          </colgroup>
			          <thead valign="top" align="top">
			            <tr valign="top" align="top">
			              <th valign="top" align="top">Header</th>
			              <td valign="top" align="top">Cell</td>
			            </tr>
			          </thead>
			          <tbody valign="top" align="top" />
			          <tfoot valign="top" align="top" />
			        </table>
			      "#,
            None,
        ),
        (r#"<fbt desc="foo" doNotExtract />;"#, None),
        (r#"<fbs desc="foo" doNotExtract />;"#, None),
        (r#"<math displaystyle="true" />;"#, None),
        (
            r#"
			        <div className="App" data-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash="customValue">
			          Hello, world!
			        </div>
			      "#,
            None,
        ),
    ];

    let fail = vec![
        (r#"<div allowTransparency="true" />"#, None),
        (r#"<div hasOwnProperty="should not be allowed property"></div>;"#, None),
        (r#"<div abc="should not be allowed property"></div>;"#, None),
        (r#"<div aria-fake="should not be allowed property"></div>;"#, None),
        (r#"<div someProp="bar"></div>;"#, None),
        (r#"<div class="bar"></div>;"#, None),
        (r#"<div for="bar"></div>;"#, None),
        (r#"<div accept-charset="bar"></div>;"#, None),
        (r#"<div http-equiv="bar"></div>;"#, None),
        (r#"<div accesskey="bar"></div>;"#, None),
        (r#"<div onclick="bar"></div>;"#, None),
        (r#"<div onmousedown="bar"></div>;"#, None),
        (r#"<div onMousedown="bar"></div>;"#, None),
        (r#"<use xlink:href="bar" />;"#, None),
        (r#"<rect clip-path="bar" />;"#, None),
        (r"<script crossorigin nomodule />", None),
        (r"<div crossorigin />", None),
        (r"<div crossOrigin />", None),
        (r#"<div as="audio" />"#, None),
        (
            r"<div onAbort={this.abort} onDurationChange={this.durationChange} onEmptied={this.emptied} onEnded={this.end} onResize={this.resize} onError={this.error} />",
            None,
        ),
        (r"<div onLoad={this.load} />", None),
        (r#"<div fill="pink" />"#, None),
        (
            r"<div controls={this.controls} loop={true} muted={false} src={this.videoSrc} playsInline={true} allowFullScreen></div>",
            None,
        ),
        (r#"<div download="foo" />"#, None),
        (r#"<div imageSrcSet="someImageSrcSet" />"#, None),
        (r#"<div imageSizes="someImageSizes" />"#, None),
        (r#"<div data-xml-anything="invalid" />"#, None),
        (
            r#"<div data-testID="bar" data-under_sCoRe="bar" />;"#,
            Some(serde_json::json!([{ "requireDataLowercase": true }])),
        ),
        (r#"<div abbr="abbr" />"#, None),
        (r#"<div webkitDirectory="" />"#, None),
        (r#"<div webkitdirectory="" />"#, None),
        (
            r#"
			        <div className="App" data-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash:c="customValue">
			          Hello, world!
			        </div>
			      "#,
            None,
        ),
    ];

    Tester::new(NoUnknownProperty::NAME, pass, fail).test_and_snapshot();
}
