use oxc_ast::ast::*;

use crate::{
    ToBigInt, ToIntegerIndex,
    constant_evaluation::DetermineValueType,
    to_numeric::ToNumeric,
    to_primitive::{ToPrimitive, ToPrimitiveResult},
};

use super::{MayHaveSideEffects, PropertyReadSideEffects, context::MayHaveSideEffectsContext};

impl<'a> MayHaveSideEffects<'a> for Expression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            Expression::Identifier(ident) => ident.may_have_side_effects(ctx),
            Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::MetaProperty(_)
            | Expression::ThisExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::Super(_) => false,
            Expression::TemplateLiteral(e) => e.may_have_side_effects(ctx),
            Expression::UnaryExpression(e) => e.may_have_side_effects(ctx),
            Expression::LogicalExpression(e) => e.may_have_side_effects(ctx),
            Expression::ParenthesizedExpression(e) => e.expression.may_have_side_effects(ctx),
            Expression::ConditionalExpression(e) => {
                if e.test.may_have_side_effects(ctx) {
                    return true;
                }
                // typeof x === 'undefined' ? fallback : x
                if is_side_effect_free_unbound_identifier_ref(&e.alternate, &e.test, false, ctx) {
                    return e.consequent.may_have_side_effects(ctx);
                }
                // typeof x !== 'undefined' ? x : fallback
                if is_side_effect_free_unbound_identifier_ref(&e.consequent, &e.test, true, ctx) {
                    return e.alternate.may_have_side_effects(ctx);
                }
                e.consequent.may_have_side_effects(ctx) || e.alternate.may_have_side_effects(ctx)
            }
            Expression::SequenceExpression(e) => {
                e.expressions.iter().any(|e| e.may_have_side_effects(ctx))
            }
            Expression::BinaryExpression(e) => e.may_have_side_effects(ctx),
            Expression::ObjectExpression(object_expr) => {
                object_expr.properties.iter().any(|property| property.may_have_side_effects(ctx))
            }
            Expression::ArrayExpression(e) => e.may_have_side_effects(ctx),
            Expression::ClassExpression(e) => e.may_have_side_effects(ctx),
            Expression::PrivateInExpression(e) => {
                if e.right.may_have_side_effects(ctx) {
                    return true;
                }
                // `#x in y` throws when `y` is not an object.
                !e.right.value_type(ctx).is_object()
            }
            Expression::ChainExpression(e) => e.expression.may_have_side_effects(ctx),
            match_member_expression!(Expression) => {
                self.to_member_expression().may_have_side_effects(ctx)
            }
            Expression::CallExpression(e) => e.may_have_side_effects(ctx),
            Expression::NewExpression(e) => e.may_have_side_effects(ctx),
            Expression::TaggedTemplateExpression(e) => e.may_have_side_effects(ctx),
            Expression::AssignmentExpression(e) => e.may_have_side_effects(ctx),
            Expression::UpdateExpression(e) => e.may_have_side_effects(ctx),
            _ => true,
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for IdentifierReference<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self.name.as_str() {
            "NaN" | "Infinity" | "undefined" => false,
            // Reading global variables may have a side effect.
            // NOTE: It should also return true when the reference might refer to a reference value created by a with statement
            // NOTE: we ignore TDZ errors
            _ => ctx.unknown_global_side_effects() && ctx.is_global_reference(self)
                && !is_known_global_identifier(self.name.as_str()),
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for TemplateLiteral<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.expressions.iter().any(|e| {
            // ToString is called for each expression.
            // If the expression is a Symbol or ToPrimitive returns a Symbol, an error is thrown.
            // ToPrimitive returns the value as-is for non-Object values, so we can use it instead of ToString here.
            e.to_primitive(ctx).is_symbol() != Some(false) || e.may_have_side_effects(ctx)
        })
    }
}

impl<'a> MayHaveSideEffects<'a> for UnaryExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self.operator {
            UnaryOperator::Delete => true,
            UnaryOperator::Void | UnaryOperator::LogicalNot => {
                self.argument.may_have_side_effects(ctx)
            }
            UnaryOperator::Typeof => {
                if matches!(&self.argument, Expression::Identifier(_)) {
                    false
                } else {
                    self.argument.may_have_side_effects(ctx)
                }
            }
            UnaryOperator::UnaryPlus => {
                // ToNumber throws an error when the argument is Symbol / BigInt / an object that
                // returns Symbol or BigInt from ToPrimitive
                self.argument.to_primitive(ctx).is_symbol_or_bigint() != Some(false)
                    || self.argument.may_have_side_effects(ctx)
            }
            UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                // ToNumeric throws an error when the argument is Symbol / an object that
                // returns Symbol from ToPrimitive
                self.argument.to_primitive(ctx).is_symbol() != Some(false)
                    || self.argument.may_have_side_effects(ctx)
            }
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for BinaryExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self.operator {
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan => {
                self.left.may_have_side_effects(ctx) || self.right.may_have_side_effects(ctx)
            }
            BinaryOperator::Instanceof => {
                // When the following conditions are met, instanceof won't throw `TypeError`.
                // - the right hand side is a known global reference which is a function
                // - the left hand side is not a proxy
                if let Expression::Identifier(right_ident) = &self.right {
                    let name = right_ident.name.as_str();
                    // Any known global non-constructor functions can be allowed here.
                    // But because non-constructor functions are not likely to be used, we ignore them.
                    if is_known_global_constructor(name)
                        && ctx.is_global_reference(right_ident)
                        && !self.left.value_type(ctx).is_undetermined()
                    {
                        return false;
                    }
                }
                // instanceof can throw `TypeError`
                true
            }
            BinaryOperator::In => {
                // in can throw `TypeError`
                true
            }
            BinaryOperator::Addition => {
                let left = self.left.to_primitive(ctx);
                let right = self.right.to_primitive(ctx);
                if left.is_string() == Some(true) || right.is_string() == Some(true) {
                    // If either side is a string, ToString is called for both sides.
                    let other_side = if left.is_string() == Some(true) { right } else { left };
                    // ToString() for Symbols throws an error.
                    return other_side.is_symbol() != Some(false)
                        || self.left.may_have_side_effects(ctx)
                        || self.right.may_have_side_effects(ctx);
                }

                let left_to_numeric_type = left.to_numeric(ctx);
                let right_to_numeric_type = right.to_numeric(ctx);
                if (left_to_numeric_type.is_number() && right_to_numeric_type.is_number())
                    || (left_to_numeric_type.is_bigint() && right_to_numeric_type.is_bigint())
                {
                    self.left.may_have_side_effects(ctx) || self.right.may_have_side_effects(ctx)
                } else {
                    true
                }
            }
            BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::ShiftLeft
            | BinaryOperator::BitwiseOR
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::Exponential
            | BinaryOperator::ShiftRightZeroFill => {
                let left_to_numeric_type = self.left.to_numeric(ctx);
                let right_to_numeric_type = self.right.to_numeric(ctx);
                if left_to_numeric_type.is_bigint() && right_to_numeric_type.is_bigint() {
                    if self.operator == BinaryOperator::ShiftRightZeroFill {
                        true
                    } else if matches!(
                        self.operator,
                        BinaryOperator::Exponential
                            | BinaryOperator::Division
                            | BinaryOperator::Remainder
                    ) {
                        if let Expression::BigIntLiteral(right) = &self.right {
                            match self.operator {
                                BinaryOperator::Exponential => {
                                    right.is_negative() || self.left.may_have_side_effects(ctx)
                                }
                                BinaryOperator::Division | BinaryOperator::Remainder => {
                                    right.is_zero() || self.left.may_have_side_effects(ctx)
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            true
                        }
                    } else {
                        self.left.may_have_side_effects(ctx)
                            || self.right.may_have_side_effects(ctx)
                    }
                } else if left_to_numeric_type.is_number() && right_to_numeric_type.is_number() {
                    self.left.may_have_side_effects(ctx) || self.right.may_have_side_effects(ctx)
                } else {
                    true
                }
            }
        }
    }
}

/// Validate a RegExp constructor call using the regex parser.
///
/// Returns `true` if the pattern and flags are valid (pure/side-effect free),
/// `false` if invalid or cannot be statically determined.
///
/// Invalid patterns like `RegExp("[")` or invalid flags like `RegExp("a", "xyz")` throw SyntaxError,
/// so they are NOT pure.
///
/// See <https://github.com/oxc-project/oxc/issues/18050>
pub fn is_valid_regexp(args: &[Argument<'_>]) -> bool {
    // Extract pattern from first argument
    let pattern = match args.first() {
        // No arguments: `RegExp()` is valid, returns /(?:)/
        None => "",
        Some(arg) => match arg.as_expression() {
            // RegExp literal argument: `RegExp(/foo/)` is always valid
            Some(Expression::RegExpLiteral(_)) => return true,
            // String literal: extract the pattern to validate
            Some(Expression::StringLiteral(s)) => s.value.as_str(),
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

    // Use the regex parser to validate the pattern and flags
    let allocator = oxc_allocator::Allocator::default();
    oxc_regular_expression::LiteralParser::new(
        &allocator,
        pattern,
        flags,
        oxc_regular_expression::Options::default(),
    )
    .parse()
    .is_ok()
}

#[rustfmt::skip]
fn is_pure_global_function(name: &str) -> bool {
    matches!(name, "decodeURI" | "decodeURIComponent" | "encodeURI" | "encodeURIComponent"
            | "escape" | "isFinite" | "isNaN" | "parseFloat" | "parseInt")
}

#[rustfmt::skip]
fn is_pure_call(name: &str) -> bool {
    matches!(name, "Date" | "Boolean" | "Error" | "EvalError" | "RangeError" | "ReferenceError"
            | "SyntaxError" | "TypeError" | "URIError" | "Number" | "Object" | "String" | "Symbol")
}

#[rustfmt::skip]
fn is_pure_constructor(name: &str) -> bool {
    matches!(name, "Set" | "Map" | "WeakSet" | "WeakMap" | "ArrayBuffer" | "Date"
            | "Boolean" | "Error" | "EvalError" | "RangeError" | "ReferenceError"
            | "SyntaxError" | "TypeError" | "URIError" | "Number" | "Object" | "String"
            | "Int8Array" | "Uint8Array" | "Uint8ClampedArray" | "Int16Array" | "Uint16Array"
            | "Int32Array" | "Uint32Array" | "Float32Array" | "Float64Array"
            | "BigInt64Array" | "BigUint64Array" | "DataView")
}

/// Whether the name matches any known global constructors.
///
/// <https://tc39.es/ecma262/multipage/global-object.html#sec-constructor-properties-of-the-global-object>
fn is_known_global_constructor(name: &str) -> bool {
    // technically, we need to exclude the constructors that are not supported by the target
    matches!(
        name,
        "AggregateError"
            | "Array"
            | "ArrayBuffer"
            | "BigInt"
            | "BigInt64Array"
            | "BitUint64Array"
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
/// This list is ported from Rolldown's `GLOBAL_IDENT` set.
/// `NaN`, `Infinity`, `undefined` are excluded since they are already handled as special cases.
#[rustfmt::skip]
fn is_known_global_identifier(name: &str) -> bool {
    matches!(name,
        // Core JS globals
        "Array" | "Boolean" | "Function" | "Math" | "Number" | "Object" | "RegExp" | "String"
        // Other globals present in both the browser and node
        | "AbortController" | "AbortSignal" | "AggregateError" | "ArrayBuffer" | "BigInt"
        | "DataView" | "Date" | "Error" | "EvalError" | "Event" | "EventTarget"
        | "Float32Array" | "Float64Array" | "Int16Array" | "Int32Array" | "Int8Array" | "Intl"
        | "JSON" | "Map" | "MessageChannel" | "MessageEvent" | "MessagePort" | "Promise"
        | "Proxy" | "RangeError" | "ReferenceError" | "Reflect" | "Set" | "Symbol"
        | "SyntaxError" | "TextDecoder" | "TextEncoder" | "TypeError" | "URIError" | "URL"
        | "URLSearchParams" | "Uint16Array" | "Uint32Array" | "Uint8Array"
        | "Uint8ClampedArray" | "WeakMap" | "WeakSet" | "WebAssembly"
        | "clearInterval" | "clearTimeout" | "console" | "decodeURI" | "decodeURIComponent"
        | "encodeURI" | "encodeURIComponent" | "escape" | "globalThis" | "isFinite" | "isNaN"
        | "parseFloat" | "parseInt" | "queueMicrotask" | "setInterval" | "setTimeout"
        | "unescape"
        // CSSOM APIs
        | "CSSAnimation" | "CSSFontFaceRule" | "CSSImportRule" | "CSSKeyframeRule"
        | "CSSKeyframesRule" | "CSSMediaRule" | "CSSNamespaceRule" | "CSSPageRule" | "CSSRule"
        | "CSSRuleList" | "CSSStyleDeclaration" | "CSSStyleRule" | "CSSStyleSheet"
        | "CSSSupportsRule" | "CSSTransition"
        // SVG DOM
        | "SVGAElement" | "SVGAngle" | "SVGAnimateElement" | "SVGAnimateMotionElement"
        | "SVGAnimateTransformElement" | "SVGAnimatedAngle" | "SVGAnimatedBoolean"
        | "SVGAnimatedEnumeration" | "SVGAnimatedInteger" | "SVGAnimatedLength"
        | "SVGAnimatedLengthList" | "SVGAnimatedNumber" | "SVGAnimatedNumberList"
        | "SVGAnimatedPreserveAspectRatio" | "SVGAnimatedRect" | "SVGAnimatedString"
        | "SVGAnimatedTransformList" | "SVGAnimationElement" | "SVGCircleElement"
        | "SVGClipPathElement" | "SVGComponentTransferFunctionElement" | "SVGDefsElement"
        | "SVGDescElement" | "SVGElement" | "SVGEllipseElement" | "SVGFEBlendElement"
        | "SVGFEColorMatrixElement" | "SVGFEComponentTransferElement"
        | "SVGFECompositeElement" | "SVGFEConvolveMatrixElement"
        | "SVGFEDiffuseLightingElement" | "SVGFEDisplacementMapElement"
        | "SVGFEDistantLightElement" | "SVGFEDropShadowElement" | "SVGFEFloodElement"
        | "SVGFEFuncAElement" | "SVGFEFuncBElement" | "SVGFEFuncGElement"
        | "SVGFEFuncRElement" | "SVGFEGaussianBlurElement" | "SVGFEImageElement"
        | "SVGFEMergeElement" | "SVGFEMergeNodeElement" | "SVGFEMorphologyElement"
        | "SVGFEOffsetElement" | "SVGFEPointLightElement" | "SVGFESpecularLightingElement"
        | "SVGFESpotLightElement" | "SVGFETileElement" | "SVGFETurbulenceElement"
        | "SVGFilterElement" | "SVGForeignObjectElement" | "SVGGElement"
        | "SVGGeometryElement" | "SVGGradientElement" | "SVGGraphicsElement"
        | "SVGImageElement" | "SVGLength" | "SVGLengthList" | "SVGLineElement"
        | "SVGLinearGradientElement" | "SVGMPathElement" | "SVGMarkerElement"
        | "SVGMaskElement" | "SVGMatrix" | "SVGMetadataElement" | "SVGNumber"
        | "SVGNumberList" | "SVGPathElement" | "SVGPatternElement" | "SVGPoint"
        | "SVGPointList" | "SVGPolygonElement" | "SVGPolylineElement"
        | "SVGPreserveAspectRatio" | "SVGRadialGradientElement" | "SVGRect"
        | "SVGRectElement" | "SVGSVGElement" | "SVGScriptElement" | "SVGSetElement"
        | "SVGStopElement" | "SVGStringList" | "SVGStyleElement" | "SVGSwitchElement"
        | "SVGSymbolElement" | "SVGTSpanElement" | "SVGTextContentElement"
        | "SVGTextElement" | "SVGTextPathElement" | "SVGTextPositioningElement"
        | "SVGTitleElement" | "SVGTransform" | "SVGTransformList" | "SVGUnitTypes"
        | "SVGUseElement" | "SVGViewElement"
        // Other browser APIs
        | "AnalyserNode" | "Animation" | "AnimationEffect" | "AnimationEvent"
        | "AnimationPlaybackEvent" | "AnimationTimeline" | "Attr" | "Audio" | "AudioBuffer"
        | "AudioBufferSourceNode" | "AudioDestinationNode" | "AudioListener" | "AudioNode"
        | "AudioParam" | "AudioProcessingEvent" | "AudioScheduledSourceNode" | "BarProp"
        | "BeforeUnloadEvent" | "BiquadFilterNode" | "Blob" | "BlobEvent"
        | "ByteLengthQueuingStrategy" | "CDATASection" | "CSS" | "CanvasGradient"
        | "CanvasPattern" | "CanvasRenderingContext2D" | "ChannelMergerNode"
        | "ChannelSplitterNode" | "CharacterData" | "ClipboardEvent" | "CloseEvent"
        | "Comment" | "CompositionEvent" | "ConvolverNode" | "CountQueuingStrategy"
        | "Crypto" | "CustomElementRegistry" | "CustomEvent" | "DOMException"
        | "DOMImplementation" | "DOMMatrix" | "DOMMatrixReadOnly" | "DOMParser" | "DOMPoint"
        | "DOMPointReadOnly" | "DOMQuad" | "DOMRect" | "DOMRectList" | "DOMRectReadOnly"
        | "DOMStringList" | "DOMStringMap" | "DOMTokenList" | "DataTransfer"
        | "DataTransferItem" | "DataTransferItemList" | "DelayNode" | "Document"
        | "DocumentFragment" | "DocumentTimeline" | "DocumentType" | "DragEvent"
        | "DynamicsCompressorNode" | "Element" | "ErrorEvent" | "EventSource" | "File"
        | "FileList" | "FileReader" | "FocusEvent" | "FontFace" | "FormData" | "GainNode"
        | "Gamepad" | "GamepadButton" | "GamepadEvent" | "Geolocation"
        | "GeolocationPositionError" | "HTMLAllCollection" | "HTMLAnchorElement"
        | "HTMLAreaElement" | "HTMLAudioElement" | "HTMLBRElement" | "HTMLBaseElement"
        | "HTMLBodyElement" | "HTMLButtonElement" | "HTMLCanvasElement" | "HTMLCollection"
        | "HTMLDListElement" | "HTMLDataElement" | "HTMLDataListElement"
        | "HTMLDetailsElement" | "HTMLDirectoryElement" | "HTMLDivElement" | "HTMLDocument"
        | "HTMLElement" | "HTMLEmbedElement" | "HTMLFieldSetElement" | "HTMLFontElement"
        | "HTMLFormControlsCollection" | "HTMLFormElement" | "HTMLFrameElement"
        | "HTMLFrameSetElement" | "HTMLHRElement" | "HTMLHeadElement"
        | "HTMLHeadingElement" | "HTMLHtmlElement" | "HTMLIFrameElement"
        | "HTMLImageElement" | "HTMLInputElement" | "HTMLLIElement" | "HTMLLabelElement"
        | "HTMLLegendElement" | "HTMLLinkElement" | "HTMLMapElement" | "HTMLMarqueeElement"
        | "HTMLMediaElement" | "HTMLMenuElement" | "HTMLMetaElement" | "HTMLMeterElement"
        | "HTMLModElement" | "HTMLOListElement" | "HTMLObjectElement"
        | "HTMLOptGroupElement" | "HTMLOptionElement" | "HTMLOptionsCollection"
        | "HTMLOutputElement" | "HTMLParagraphElement" | "HTMLParamElement"
        | "HTMLPictureElement" | "HTMLPreElement" | "HTMLProgressElement"
        | "HTMLQuoteElement" | "HTMLScriptElement" | "HTMLSelectElement"
        | "HTMLSlotElement" | "HTMLSourceElement" | "HTMLSpanElement" | "HTMLStyleElement"
        | "HTMLTableCaptionElement" | "HTMLTableCellElement" | "HTMLTableColElement"
        | "HTMLTableElement" | "HTMLTableRowElement" | "HTMLTableSectionElement"
        | "HTMLTemplateElement" | "HTMLTextAreaElement" | "HTMLTimeElement"
        | "HTMLTitleElement" | "HTMLTrackElement" | "HTMLUListElement"
        | "HTMLUnknownElement" | "HTMLVideoElement" | "HashChangeEvent" | "Headers"
        | "History" | "IDBCursor" | "IDBCursorWithValue" | "IDBDatabase" | "IDBFactory"
        | "IDBIndex" | "IDBKeyRange" | "IDBObjectStore" | "IDBOpenDBRequest" | "IDBRequest"
        | "IDBTransaction" | "IDBVersionChangeEvent" | "Image" | "ImageData" | "InputEvent"
        | "IntersectionObserver" | "IntersectionObserverEntry" | "KeyboardEvent"
        | "KeyframeEffect" | "Location" | "MediaCapabilities"
        | "MediaElementAudioSourceNode" | "MediaEncryptedEvent" | "MediaError"
        | "MediaList" | "MediaQueryList" | "MediaQueryListEvent" | "MediaRecorder"
        | "MediaSource" | "MediaStream" | "MediaStreamAudioDestinationNode"
        | "MediaStreamAudioSourceNode" | "MediaStreamTrack" | "MediaStreamTrackEvent"
        | "MimeType" | "MimeTypeArray" | "MouseEvent" | "MutationEvent"
        | "MutationObserver" | "MutationRecord" | "NamedNodeMap" | "Navigator" | "Node"
        | "NodeFilter" | "NodeIterator" | "NodeList" | "Notification"
        | "OfflineAudioCompletionEvent" | "Option" | "OscillatorNode"
        | "PageTransitionEvent" | "Path2D" | "Performance" | "PerformanceEntry"
        | "PerformanceMark" | "PerformanceMeasure" | "PerformanceNavigation"
        | "PerformanceObserver" | "PerformanceObserverEntryList"
        | "PerformanceResourceTiming" | "PerformanceTiming" | "PeriodicWave" | "Plugin"
        | "PluginArray" | "PointerEvent" | "PopStateEvent" | "ProcessingInstruction"
        | "ProgressEvent" | "PromiseRejectionEvent" | "RTCCertificate" | "RTCDTMFSender"
        | "RTCDTMFToneChangeEvent" | "RTCDataChannel" | "RTCDataChannelEvent"
        | "RTCIceCandidate" | "RTCPeerConnection" | "RTCPeerConnectionIceEvent"
        | "RTCRtpReceiver" | "RTCRtpSender" | "RTCRtpTransceiver"
        | "RTCSessionDescription" | "RTCStatsReport" | "RTCTrackEvent" | "RadioNodeList"
        | "Range" | "ReadableStream" | "Request" | "ResizeObserver"
        | "ResizeObserverEntry" | "Response" | "Screen" | "ScriptProcessorNode"
        | "SecurityPolicyViolationEvent" | "Selection" | "ShadowRoot" | "SourceBuffer"
        | "SourceBufferList" | "SpeechSynthesisEvent" | "SpeechSynthesisUtterance"
        | "StaticRange" | "Storage" | "StorageEvent" | "StyleSheet" | "StyleSheetList"
        | "Text" | "TextMetrics" | "TextTrack" | "TextTrackCue" | "TextTrackCueList"
        | "TextTrackList" | "TimeRanges" | "TrackEvent" | "TransitionEvent" | "TreeWalker"
        | "UIEvent" | "VTTCue" | "ValidityState" | "VisualViewport" | "WaveShaperNode"
        | "WebGLActiveInfo" | "WebGLBuffer" | "WebGLContextEvent" | "WebGLFramebuffer"
        | "WebGLProgram" | "WebGLQuery" | "WebGLRenderbuffer" | "WebGLRenderingContext"
        | "WebGLSampler" | "WebGLShader" | "WebGLShaderPrecisionFormat" | "WebGLSync"
        | "WebGLTexture" | "WebGLUniformLocation" | "WebKitCSSMatrix" | "WebSocket"
        | "WheelEvent" | "Window" | "Worker" | "XMLDocument" | "XMLHttpRequest"
        | "XMLHttpRequestEventTarget" | "XMLHttpRequestUpload" | "XMLSerializer"
        | "XPathEvaluator" | "XPathExpression" | "XPathResult" | "XSLTProcessor"
        | "alert" | "atob" | "blur" | "btoa" | "cancelAnimationFrame" | "captureEvents"
        | "close" | "closed" | "confirm" | "customElements" | "devicePixelRatio"
        | "document" | "event" | "fetch" | "find" | "focus" | "frameElement" | "frames"
        | "getComputedStyle" | "getSelection" | "history" | "indexedDB" | "isSecureContext"
        | "length" | "location" | "locationbar" | "matchMedia" | "menubar" | "moveBy"
        | "moveTo" | "name" | "navigator"
        | "onabort" | "onafterprint" | "onanimationend" | "onanimationiteration"
        | "onanimationstart" | "onbeforeprint" | "onbeforeunload" | "onblur" | "oncanplay"
        | "oncanplaythrough" | "onchange" | "onclick" | "oncontextmenu" | "oncuechange"
        | "ondblclick" | "ondrag" | "ondragend" | "ondragenter" | "ondragleave"
        | "ondragover" | "ondragstart" | "ondrop" | "ondurationchange" | "onemptied"
        | "onended" | "onerror" | "onfocus" | "ongotpointercapture" | "onhashchange"
        | "oninput" | "oninvalid" | "onkeydown" | "onkeypress" | "onkeyup"
        | "onlanguagechange" | "onload" | "onloadeddata" | "onloadedmetadata"
        | "onloadstart" | "onlostpointercapture" | "onmessage" | "onmousedown"
        | "onmouseenter" | "onmouseleave" | "onmousemove" | "onmouseout" | "onmouseover"
        | "onmouseup" | "onoffline" | "ononline" | "onpagehide" | "onpageshow" | "onpause"
        | "onplay" | "onplaying" | "onpointercancel" | "onpointerdown" | "onpointerenter"
        | "onpointerleave" | "onpointermove" | "onpointerout" | "onpointerover"
        | "onpointerup" | "onpopstate" | "onprogress" | "onratechange"
        | "onrejectionhandled" | "onreset" | "onresize" | "onscroll" | "onseeked"
        | "onseeking" | "onselect" | "onstalled" | "onstorage" | "onsubmit" | "onsuspend"
        | "ontimeupdate" | "ontoggle" | "ontransitioncancel" | "ontransitionend"
        | "ontransitionrun" | "ontransitionstart" | "onunhandledrejection" | "onunload"
        | "onvolumechange" | "onwaiting" | "onwebkitanimationend"
        | "onwebkitanimationiteration" | "onwebkitanimationstart"
        | "onwebkittransitionend" | "onwheel"
        | "open" | "opener" | "origin" | "outerHeight" | "outerWidth" | "parent"
        | "performance" | "personalbar" | "postMessage" | "print" | "prompt"
        | "releaseEvents" | "requestAnimationFrame" | "resizeBy" | "resizeTo" | "screen"
        | "screenLeft" | "screenTop" | "screenX" | "screenY" | "scroll" | "scrollBy"
        | "scrollTo" | "scrollbars" | "self" | "speechSynthesis" | "status" | "statusbar"
        | "stop" | "toolbar" | "top" | "webkitURL" | "window"
    )
}

/// Whether the property read on a known global is side-effect-free.
///
/// For example, `Math.PI`, `console.log`, `Object.keys` are all side-effect-free to read.
/// Lists ported from Rolldown's global_reference.rs.
#[rustfmt::skip]
fn is_known_global_property(global: &str, property: &str) -> bool {
    match global {
        "Math" => matches!(property,
            // Static properties
            "E" | "LN10" | "LN2" | "LOG10E" | "LOG2E" | "PI" | "SQRT1_2" | "SQRT2"
            // Static methods
            | "abs" | "acos" | "acosh" | "asin" | "asinh" | "atan" | "atan2" | "atanh"
            | "cbrt" | "ceil" | "clz32" | "cos" | "cosh" | "exp" | "expm1" | "floor"
            | "fround" | "hypot" | "imul" | "log" | "log10" | "log1p" | "log2" | "max"
            | "min" | "pow" | "random" | "round" | "sign" | "sin" | "sinh" | "sqrt"
            | "tan" | "tanh" | "trunc"
        ),
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
fn is_known_global_property_deep(global: &str, middle: &str, property: &str) -> bool {
    global == "Object" && middle == "prototype" && matches!(property,
        "__defineGetter__" | "__defineSetter__" | "__lookupGetter__" | "__lookupSetter__"
        | "hasOwnProperty" | "isPrototypeOf" | "propertyIsEnumerable" | "toLocaleString"
        | "toString" | "unwatch" | "valueOf" | "watch"
    )
}

impl<'a> MayHaveSideEffects<'a> for LogicalExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if self.left.may_have_side_effects(ctx) {
            return true;
        }
        match self.operator {
            LogicalOperator::And => {
                // Pattern: typeof x !== 'undefined' && x
                if is_side_effect_free_unbound_identifier_ref(&self.right, &self.left, true, ctx) {
                    return false;
                }
            }
            LogicalOperator::Or => {
                // Pattern: typeof x === 'undefined' || x
                if is_side_effect_free_unbound_identifier_ref(&self.right, &self.left, false, ctx) {
                    return false;
                }
            }
            LogicalOperator::Coalesce => {}
        }
        self.right.may_have_side_effects(ctx)
    }
}

impl<'a> MayHaveSideEffects<'a> for ArrayExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.elements.iter().any(|element| element.may_have_side_effects(ctx))
    }
}

impl<'a> MayHaveSideEffects<'a> for ArrayExpressionElement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            ArrayExpressionElement::SpreadElement(e) => match &e.argument {
                Expression::ArrayExpression(arr) => arr.may_have_side_effects(ctx),
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => t.may_have_side_effects(ctx),
                Expression::Identifier(ident) => {
                    // FIXME: we should treat `arguments` outside a function scope to have sideeffects
                    !(ident.name == "arguments" && ctx.is_global_reference(ident))
                }
                _ => true,
            },
            match_expression!(ArrayExpressionElement) => {
                self.to_expression().may_have_side_effects(ctx)
            }
            ArrayExpressionElement::Elision(_) => false,
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for ObjectPropertyKind<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            ObjectPropertyKind::ObjectProperty(o) => o.may_have_side_effects(ctx),
            ObjectPropertyKind::SpreadProperty(e) => {
                if ctx.property_read_side_effects() == PropertyReadSideEffects::None {
                    e.argument.may_have_side_effects(ctx)
                } else {
                    match &e.argument {
                        Expression::ArrayExpression(arr) => arr.may_have_side_effects(ctx),
                        Expression::StringLiteral(_) => false,
                        Expression::TemplateLiteral(t) => t.may_have_side_effects(ctx),
                        _ => true,
                    }
                }
            }
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for ObjectProperty<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.key.may_have_side_effects(ctx) || self.value.may_have_side_effects(ctx)
    }
}

impl<'a> MayHaveSideEffects<'a> for PropertyKey<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => false,
            match_expression!(PropertyKey) => {
                // ToPropertyKey(key) throws an error when ToPrimitive(key) throws an Error
                // But we can ignore that by using the assumption.
                self.to_expression().may_have_side_effects(ctx)
            }
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for Class<'a> {
    /// Based on <https://github.com/evanw/esbuild/blob/v0.25.0/internal/js_ast/js_ast_helpers.go#L2320>
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if !self.decorators.is_empty() {
            return true;
        }

        // NOTE: extending a value that is neither constructors nor null, throws an error
        // but that error is ignored here (it is included in the assumption)
        // Example cases: `class A extends 0 {}`, `class A extends (async function() {}) {}`
        // Considering these cases is difficult and requires to de-opt most classes with a super class.
        // To allow classes with a super class to be removed, we ignore this side effect.
        if self.super_class.as_ref().is_some_and(|sup| {
            // `(class C extends (() => {}))` is TypeError.
            matches!(sup.without_parentheses(), Expression::ArrowFunctionExpression(_))
                || sup.may_have_side_effects(ctx)
        }) {
            return true;
        }

        self.body.body.iter().any(|element| element.may_have_side_effects(ctx))
    }
}

impl<'a> MayHaveSideEffects<'a> for ClassElement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            ClassElement::StaticBlock(block) => {
                block.body.iter().any(|stmt| stmt.may_have_side_effects(ctx))
            }
            ClassElement::MethodDefinition(e) => {
                !e.decorators.is_empty()
                    || e.key.may_have_side_effects(ctx)
                    || e.value.params.items.iter().any(|item| !item.decorators.is_empty())
            }
            ClassElement::PropertyDefinition(e) => {
                !e.decorators.is_empty()
                    || e.key.may_have_side_effects(ctx)
                    || (e.r#static
                        && e.value.as_ref().is_some_and(|v| v.may_have_side_effects(ctx)))
            }
            ClassElement::AccessorProperty(e) => {
                !e.decorators.is_empty()
                    || e.key.may_have_side_effects(ctx)
                    || e.value.as_ref().is_some_and(|init| init.may_have_side_effects(ctx))
            }
            ClassElement::TSIndexSignature(_) => false,
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for ChainElement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            ChainElement::CallExpression(e) => e.may_have_side_effects(ctx),
            ChainElement::TSNonNullExpression(e) => e.expression.may_have_side_effects(ctx),
            match_member_expression!(ChainElement) => {
                self.to_member_expression().may_have_side_effects(ctx)
            }
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for MemberExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            MemberExpression::ComputedMemberExpression(e) => e.may_have_side_effects(ctx),
            MemberExpression::StaticMemberExpression(e) => e.may_have_side_effects(ctx),
            MemberExpression::PrivateFieldExpression(_) => {
                ctx.property_read_side_effects() != PropertyReadSideEffects::None
            }
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for StaticMemberExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        property_access_may_have_side_effects(&self.object, &self.property.name, ctx)
    }
}

impl<'a> MayHaveSideEffects<'a> for ComputedMemberExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match &self.expression {
            Expression::StringLiteral(s) => {
                property_access_may_have_side_effects(&self.object, &s.value, ctx)
            }
            Expression::TemplateLiteral(t) => t.single_quasi().is_some_and(|quasi| {
                property_access_may_have_side_effects(&self.object, &quasi, ctx)
            }),
            Expression::NumericLiteral(n) => !n.value.to_integer_index().is_some_and(|n| {
                !integer_index_property_access_may_have_side_effects(&self.object, n, ctx)
            }),
            Expression::BigIntLiteral(b) => {
                if b.is_negative() {
                    return true;
                }
                !b.to_big_int(ctx).and_then(ToIntegerIndex::to_integer_index).is_some_and(|b| {
                    !integer_index_property_access_may_have_side_effects(&self.object, b, ctx)
                })
            }
            _ => {
                // Non-literal keys (e.g. `obj[expr]`) may trigger toString/valueOf on the key,
                // which is a side effect. But if property read side effects are disabled,
                // only check the key expression and object for their own side effects.
                if ctx.property_read_side_effects() == PropertyReadSideEffects::None {
                    self.expression.may_have_side_effects(ctx)
                        || self.object.may_have_side_effects(ctx)
                } else {
                    true
                }
            }
        }
    }
}

fn property_access_may_have_side_effects<'a>(
    object: &Expression<'a>,
    property: &str,
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> bool {
    if object.may_have_side_effects(ctx) {
        return true;
    }
    if ctx.property_read_side_effects() == PropertyReadSideEffects::None {
        return false;
    }

    // Check known global property reads (e.g. Math.PI, console.log)
    if let Expression::Identifier(ident) = object
        && ctx.is_global_reference(ident)
        && is_known_global_property(ident.name.as_str(), property)
    {
        return false;
    }

    // Check known 3-level chains (e.g. Object.prototype.hasOwnProperty)
    if let Expression::StaticMemberExpression(member) = object
        && let Expression::Identifier(ident) = &member.object
        && ctx.is_global_reference(ident)
        && is_known_global_property_deep(
            ident.name.as_str(),
            member.property.name.as_str(),
            property,
        )
    {
        return false;
    }

    match property {
        "length" => {
            !(matches!(object, Expression::ArrayExpression(_))
                || object.value_type(ctx).is_string())
        }
        _ => true,
    }
}

fn integer_index_property_access_may_have_side_effects<'a>(
    object: &Expression<'a>,
    property: u32,
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> bool {
    if object.may_have_side_effects(ctx) {
        return true;
    }
    if ctx.property_read_side_effects() == PropertyReadSideEffects::None {
        return false;
    }
    match object {
        Expression::StringLiteral(s) => property as usize >= s.value.encode_utf16().count(),
        Expression::ArrayExpression(arr) => property as usize >= get_array_minimum_length(arr),
        _ => true,
    }
}

fn get_array_minimum_length(arr: &ArrayExpression) -> usize {
    arr.elements
        .iter()
        .map(|e| match e {
            ArrayExpressionElement::SpreadElement(spread) => match &spread.argument {
                Expression::ArrayExpression(arr) => get_array_minimum_length(arr),
                Expression::StringLiteral(str) => str.value.chars().count(),
                _ => 0,
            },
            _ => 1,
        })
        .sum()
}

// `PF` in <https://github.com/rollup/rollup/blob/master/src/ast/nodes/shared/knownGlobals.ts>
impl<'a> MayHaveSideEffects<'a> for CallExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if (self.pure && ctx.annotations()) || ctx.manual_pure_functions(&self.callee) {
            return self.arguments.iter().any(|e| e.may_have_side_effects(ctx));
        }

        if let Expression::Identifier(ident) = &self.callee
            && ctx.is_global_reference(ident)
        {
            let name = ident.name.as_str();
            if name == "String" {
                if self.arguments.iter().any(|e| e.may_have_side_effects(ctx)) {
                    return true;
                }
                // String(value) calls ToPrimitive on object-like/unknown values.
                return self.arguments.first().is_some_and(|arg| {
                    arg.as_expression()
                        .is_none_or(|expr| expr.to_primitive(ctx).is_string().is_none())
                });
            }
            if name == "Number" {
                if self.arguments.iter().any(|e| e.may_have_side_effects(ctx)) {
                    return true;
                }
                // Number(value) throws on Symbol and can execute user code during ToPrimitive.
                return self.arguments.first().is_some_and(|arg| {
                    arg.as_expression()
                        .is_none_or(|expr| expr.to_primitive(ctx).is_symbol() != Some(false))
                });
            }
            if name == "BigInt" {
                if self.arguments.iter().any(|e| e.may_have_side_effects(ctx)) {
                    return true;
                }
                // BigInt(value) throws for missing/invalid values and can execute user code during ToPrimitive.
                let Some(expr) = self.arguments.first().and_then(Argument::as_expression) else {
                    return true;
                };
                if matches!(
                    expr.to_primitive(ctx),
                    ToPrimitiveResult::Undetermined
                        | ToPrimitiveResult::Undefined
                        | ToPrimitiveResult::Null
                        | ToPrimitiveResult::Symbol
                ) {
                    return true;
                }
                return expr.to_big_int(ctx).is_none();
            }
            if name == "Symbol" {
                if self.arguments.iter().any(|e| e.may_have_side_effects(ctx)) {
                    return true;
                }
                // Symbol(description) applies ToString to non-undefined arguments.
                // ToString can throw on Symbol and execute user code during ToPrimitive.
                return self.arguments.first().is_some_and(|arg| {
                    arg.as_expression()
                        .is_none_or(|expr| expr.to_primitive(ctx).is_symbol() != Some(false))
                });
            }
            if is_pure_global_function(name)
                || is_pure_call(name)
                || (name == "RegExp" && is_valid_regexp(&self.arguments))
            {
                return self.arguments.iter().any(|e| e.may_have_side_effects(ctx));
            }
        }

        let (object, name) = match &self.callee {
            Expression::StaticMemberExpression(member) if !member.optional => {
                (member.object.get_identifier_reference(), member.property.name.as_str())
            }
            Expression::ComputedMemberExpression(member) if !member.optional => {
                match &member.expression {
                    Expression::StringLiteral(s) => {
                        (member.object.get_identifier_reference(), s.value.as_str())
                    }
                    _ => return true,
                }
            }
            _ => return true,
        };

        let Some(object) = object else { return true };
        if !ctx.is_global_reference(object) {
            return true;
        }

        #[rustfmt::skip]
        let is_global = match object.name.as_str() {
            "Array" => matches!(name, "isArray" | "of"),
            "ArrayBuffer" => name == "isView",
            "Date" => matches!(name, "now" | "parse" | "UTC"),
            "Math" => matches!(name, "abs" | "acos" | "acosh" | "asin" | "asinh" | "atan" | "atan2" | "atanh"
                    | "cbrt" | "ceil" | "clz32" | "cos" | "cosh" | "exp" | "expm1" | "floor" | "fround" | "hypot"
                    | "imul" | "log" | "log10" | "log1p" | "log2" | "max" | "min" | "pow" | "random" | "round"
                    | "sign" | "sin" | "sinh" | "sqrt" | "tan" | "tanh" | "trunc"),
            "Number" => matches!(name, "isFinite" | "isInteger" | "isNaN" | "isSafeInteger" | "parseFloat" | "parseInt"),
            "Object" => matches!(name, "create" | "getOwnPropertyDescriptor" | "getOwnPropertyDescriptors" | "getOwnPropertyNames"
                    | "getOwnPropertySymbols" | "getPrototypeOf" | "hasOwn" | "is" | "isExtensible" | "isFrozen" | "isSealed" | "keys"),
            "String" => matches!(name, "fromCharCode" | "fromCodePoint" | "raw"),
            "Symbol" => matches!(name, "for" | "keyFor"),
            "URL" => name == "canParse",
            "Float32Array" | "Float64Array" | "Int16Array" | "Int32Array" | "Int8Array" | "Uint16Array" | "Uint32Array" | "Uint8Array" | "Uint8ClampedArray" => name == "of",
            _ => false,
        };

        if is_global {
            return self.arguments.iter().any(|e| e.may_have_side_effects(ctx));
        }

        true
    }
}

// `[ValueProperties]: PURE` in <https://github.com/rollup/rollup/blob/master/src/ast/nodes/shared/knownGlobals.ts>
impl<'a> MayHaveSideEffects<'a> for NewExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if (self.pure && ctx.annotations()) || ctx.manual_pure_functions(&self.callee) {
            return self.arguments.iter().any(|e| e.may_have_side_effects(ctx));
        }
        if let Expression::Identifier(ident) = &self.callee
            && ctx.is_global_reference(ident)
            && let name = ident.name.as_str()
            && (is_pure_constructor(name) || (name == "RegExp" && is_valid_regexp(&self.arguments)))
        {
            return self.arguments.iter().any(|e| e.may_have_side_effects(ctx));
        }
        true
    }
}

impl<'a> MayHaveSideEffects<'a> for TaggedTemplateExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if ctx.manual_pure_functions(&self.tag) {
            self.quasi.may_have_side_effects(ctx)
        } else {
            true
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for AssignmentExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if ctx.property_write_side_effects() {
            return true;
        }
        // When property_write_side_effects is false, member expression writes are considered free.
        // Other writes (to variables, destructuring targets) still have side effects.
        match &self.left {
            AssignmentTarget::StaticMemberExpression(e) => {
                e.object.may_have_side_effects(ctx) || self.right.may_have_side_effects(ctx)
            }
            AssignmentTarget::ComputedMemberExpression(e) => {
                e.object.may_have_side_effects(ctx)
                    || e.expression.may_have_side_effects(ctx)
                    || self.right.may_have_side_effects(ctx)
            }
            AssignmentTarget::PrivateFieldExpression(e) => {
                e.object.may_have_side_effects(ctx) || self.right.may_have_side_effects(ctx)
            }
            _ => true,
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for UpdateExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if ctx.property_write_side_effects() {
            return true;
        }
        // When property_write_side_effects is false, member expression updates are considered free.
        match &self.argument {
            SimpleAssignmentTarget::StaticMemberExpression(e) => {
                e.object.may_have_side_effects(ctx)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(e) => {
                e.object.may_have_side_effects(ctx) || e.expression.may_have_side_effects(ctx)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(e) => {
                e.object.may_have_side_effects(ctx)
            }
            _ => true,
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for Argument<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            Argument::SpreadElement(e) => match &e.argument {
                Expression::ArrayExpression(arr) => arr.may_have_side_effects(ctx),
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => t.may_have_side_effects(ctx),
                _ => true,
            },
            match_expression!(Argument) => self.to_expression().may_have_side_effects(ctx),
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for AssignmentTarget<'a> {
    /// This only checks the `Evaluation of <AssignmentTarget>`.
    /// The sideeffect of `PutValue(<AssignmentTarget>)` is not considered here.
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            match_simple_assignment_target!(AssignmentTarget) => {
                self.to_simple_assignment_target().may_have_side_effects(ctx)
            }
            match_assignment_target_pattern!(AssignmentTarget) => true,
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for SimpleAssignmentTarget<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => false,
            SimpleAssignmentTarget::StaticMemberExpression(member_expr) => {
                member_expr.object.may_have_side_effects(ctx)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(member_expr) => {
                member_expr.object.may_have_side_effects(ctx)
                    || member_expr.expression.may_have_side_effects(ctx)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(member_expr) => {
                member_expr.object.may_have_side_effects(ctx)
            }
            SimpleAssignmentTarget::TSAsExpression(_)
            | SimpleAssignmentTarget::TSNonNullExpression(_)
            | SimpleAssignmentTarget::TSSatisfiesExpression(_)
            | SimpleAssignmentTarget::TSTypeAssertion(_) => true,
        }
    }
}

/// Helper function to check if accessing an unbound identifier reference is side-effect-free based on a guard condition.
///
/// This function analyzes patterns like:
/// - `typeof x === 'undefined' && x` (safe to access x in the right branch)
/// - `typeof x !== 'undefined' || x` (safe to access x in the right branch)
/// - `typeof x < 'u' && x` (safe to access x in the right branch)
///
/// Ported from: <https://github.com/evanw/esbuild/blob/d34e79e2a998c21bb71d57b92b0017ca11756912/internal/js_ast/js_ast_helpers.go#L2594-L2639>
fn is_side_effect_free_unbound_identifier_ref<'a>(
    value: &Expression<'a>,
    guard_condition: &Expression<'a>,
    mut is_yes_branch: bool,
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> bool {
    let Some(ident) = value.get_identifier_reference() else {
        return false;
    };
    if !ctx.is_global_reference(ident) {
        return false;
    }

    let Expression::BinaryExpression(bin_expr) = guard_condition else {
        return false;
    };
    match bin_expr.operator {
        BinaryOperator::StrictEquality
        | BinaryOperator::StrictInequality
        | BinaryOperator::Equality
        | BinaryOperator::Inequality => {
            let (mut ty_of, mut string) = (&bin_expr.left, &bin_expr.right);
            if matches!(ty_of, Expression::StringLiteral(_)) {
                std::mem::swap(&mut string, &mut ty_of);
            }

            let Expression::UnaryExpression(unary) = ty_of else {
                return false;
            };
            if !(unary.operator == UnaryOperator::Typeof
                && matches!(unary.argument, Expression::Identifier(_)))
            {
                return false;
            }

            let Expression::StringLiteral(string) = string else {
                return false;
            };

            let is_undefined_check = string.value == "undefined";
            if (is_undefined_check == is_yes_branch)
                == matches!(
                    bin_expr.operator,
                    BinaryOperator::Inequality | BinaryOperator::StrictInequality
                )
                && unary.argument.is_specific_id(&ident.name)
            {
                return true;
            }
        }
        BinaryOperator::LessThan
        | BinaryOperator::LessEqualThan
        | BinaryOperator::GreaterThan
        | BinaryOperator::GreaterEqualThan => {
            let (mut ty_of, mut string) = (&bin_expr.left, &bin_expr.right);
            if matches!(ty_of, Expression::StringLiteral(_)) {
                std::mem::swap(&mut string, &mut ty_of);
                is_yes_branch = !is_yes_branch;
            }

            let Expression::UnaryExpression(unary) = ty_of else {
                return false;
            };
            if !(unary.operator == UnaryOperator::Typeof
                && matches!(unary.argument, Expression::Identifier(_)))
            {
                return false;
            }

            let Expression::StringLiteral(string) = string else {
                return false;
            };
            if string.value != "u" {
                return false;
            }

            if is_yes_branch
                == matches!(
                    bin_expr.operator,
                    BinaryOperator::LessThan | BinaryOperator::LessEqualThan
                )
                && unary.argument.is_specific_id(&ident.name)
            {
                return true;
            }
        }
        _ => {}
    }

    false
}
