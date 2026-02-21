use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::{self, Expression, IdentifierReference, MemberExpression};
use oxc_span::Ident;
use oxc_syntax::operator::{
    AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
};

use super::{MayHaveSideEffectsContext, is_valid_regexp};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PrimitiveType {
    Null,
    Undefined,
    Boolean,
    Number,
    String,
    BigInt,
    Mixed,
    Unknown,
}

fn merged_known_primitive_types<'a>(
    ctx: &impl MayHaveSideEffectsContext<'a>,
    left: &Expression<'a>,
    right: &Expression<'a>,
) -> PrimitiveType {
    let left_type = known_primitive_type(ctx, left);
    if left_type == PrimitiveType::Unknown {
        return PrimitiveType::Unknown;
    }
    let right_type = known_primitive_type(ctx, right);
    if right_type == PrimitiveType::Unknown {
        return PrimitiveType::Unknown;
    }
    if right_type == left_type {
        return right_type;
    }
    PrimitiveType::Mixed
}

pub fn known_primitive_type<'a>(
    ctx: &impl MayHaveSideEffectsContext<'a>,
    expr: &Expression<'a>,
) -> PrimitiveType {
    match expr {
        Expression::NullLiteral(_) => PrimitiveType::Null,
        Expression::Identifier(id) if id.name == "undefined" && ctx.is_global_reference(id) => {
            PrimitiveType::Undefined
        }
        Expression::BooleanLiteral(_) => PrimitiveType::Boolean,
        Expression::NumericLiteral(_) => PrimitiveType::Number,
        Expression::StringLiteral(_) => PrimitiveType::String,
        Expression::BigIntLiteral(_) => PrimitiveType::BigInt,
        Expression::TemplateLiteral(e) => {
            if e.expressions.is_empty() {
                PrimitiveType::String
            } else {
                PrimitiveType::Unknown
            }
        }
        Expression::UpdateExpression(e) => {
            match e.operator {
                UpdateOperator::Increment | UpdateOperator::Decrement => {
                    PrimitiveType::Mixed // Can be number or bigint
                }
            }
        }
        Expression::UnaryExpression(e) => match e.operator {
            UnaryOperator::Void => PrimitiveType::Undefined,
            UnaryOperator::Typeof => PrimitiveType::String,
            UnaryOperator::LogicalNot | UnaryOperator::Delete => PrimitiveType::Boolean,
            UnaryOperator::UnaryPlus => PrimitiveType::Number, // Cannot be bigint because that throws an exception
            UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                let value = known_primitive_type(ctx, &e.argument);
                if value == PrimitiveType::BigInt {
                    return PrimitiveType::BigInt;
                }
                if value != PrimitiveType::Unknown && value != PrimitiveType::Mixed {
                    return PrimitiveType::Number;
                }
                PrimitiveType::Mixed // Can be number or bigint
            }
        },
        Expression::LogicalExpression(e) => match e.operator {
            LogicalOperator::Or | LogicalOperator::And => {
                merged_known_primitive_types(ctx, &e.left, &e.right)
            }
            LogicalOperator::Coalesce => {
                let left = known_primitive_type(ctx, &e.left);
                let right = known_primitive_type(ctx, &e.right);
                if left == PrimitiveType::Null || left == PrimitiveType::Undefined {
                    return right;
                }
                if left != PrimitiveType::Unknown {
                    if left != PrimitiveType::Mixed {
                        return left; // Definitely not null or undefined
                    }
                    if right != PrimitiveType::Unknown {
                        return PrimitiveType::Mixed; // Definitely some kind of primitive
                    }
                }
                PrimitiveType::Unknown
            }
        },
        Expression::BinaryExpression(e) => match e.operator {
            BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::Instanceof
            | BinaryOperator::In => PrimitiveType::Boolean,
            BinaryOperator::Addition => {
                let left = known_primitive_type(ctx, &e.left);
                let right = known_primitive_type(ctx, &e.right);
                if left == PrimitiveType::String || right == PrimitiveType::String {
                    PrimitiveType::String
                } else if left == PrimitiveType::BigInt && right == PrimitiveType::BigInt {
                    PrimitiveType::BigInt
                } else if !matches!(
                    left,
                    PrimitiveType::Unknown | PrimitiveType::Mixed | PrimitiveType::BigInt
                ) && !matches!(
                    right,
                    PrimitiveType::Unknown | PrimitiveType::Mixed | PrimitiveType::BigInt
                ) {
                    PrimitiveType::Number
                } else {
                    PrimitiveType::Mixed // Can be number or bigint or string (or an exception)
                }
            }
            BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Exponential
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOR
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRightZeroFill
            | BinaryOperator::BitwiseXOR => PrimitiveType::Mixed,
        },

        Expression::AssignmentExpression(e) => match e.operator {
            AssignmentOperator::Assign => known_primitive_type(ctx, &e.right),
            AssignmentOperator::Addition => {
                let right = known_primitive_type(ctx, &e.right);
                if right == PrimitiveType::String {
                    PrimitiveType::String
                } else {
                    PrimitiveType::Mixed // Can be number or bigint or string (or an exception)
                }
            }
            AssignmentOperator::Subtraction
            | AssignmentOperator::Multiplication
            | AssignmentOperator::Division
            | AssignmentOperator::Remainder
            | AssignmentOperator::ShiftLeft
            | AssignmentOperator::ShiftRight
            | AssignmentOperator::ShiftRightZeroFill
            | AssignmentOperator::BitwiseOR
            | AssignmentOperator::BitwiseXOR
            | AssignmentOperator::BitwiseAnd
            | AssignmentOperator::LogicalAnd
            | AssignmentOperator::LogicalOr
            | AssignmentOperator::LogicalNullish
            | AssignmentOperator::Exponential => PrimitiveType::Mixed,
        },
        _ => PrimitiveType::Unknown,
    }
}

pub fn can_change_strict_to_loose<'a>(
    ctx: &impl MayHaveSideEffectsContext<'a>,
    a: &Expression<'a>,
    b: &Expression<'a>,
) -> bool {
    let x = known_primitive_type(ctx, a);
    let y = known_primitive_type(ctx, b);
    x == y && !matches!(x, PrimitiveType::Unknown | PrimitiveType::Mixed)
}

pub fn is_primitive_literal<'a>(
    ctx: &impl MayHaveSideEffectsContext<'a>,
    expr: &Expression<'a>,
) -> bool {
    match expr {
        Expression::NullLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BigIntLiteral(_) => true,
        // Include `+1` / `-1`.
        Expression::UnaryExpression(e) => match e.operator {
            UnaryOperator::Void => is_primitive_literal(ctx, &e.argument),
            UnaryOperator::UnaryNegation | UnaryOperator::UnaryPlus => {
                matches!(e.argument, Expression::NumericLiteral(_))
            }
            _ => false,
        },
        Expression::Identifier(id) if id.name == "undefined" && ctx.is_global_reference(id) => true,
        _ => false,
    }
}

pub fn extract_member_expr_chain<'a, 'b>(
    expr: &'b MemberExpression<'a>,
    max_len: usize,
) -> Option<(&'b IdentifierReference<'a>, Vec<Ident<'a>>)> {
    if max_len == 0 {
        return None;
    }

    let mut chain = vec![];
    let mut cur = match expr {
        MemberExpression::ComputedMemberExpression(computed_expr) => {
            let Expression::StringLiteral(ref str) = computed_expr.expression else {
                return None;
            };
            chain.push(str.value.into());
            &computed_expr.object
        }
        MemberExpression::StaticMemberExpression(static_expr) => {
            chain.push(static_expr.property.name);
            &static_expr.object
        }
        MemberExpression::PrivateFieldExpression(_) => return None,
    };

    // extract_rest_member_expr_chain
    loop {
        match cur {
            Expression::StaticMemberExpression(expr) => {
                cur = &expr.object;
                chain.push(expr.property.name);
            }
            Expression::ComputedMemberExpression(expr) => {
                let Expression::StringLiteral(ref str) = expr.expression else {
                    break;
                };
                chain.push(str.value.into());
                cur = &expr.object;
            }
            Expression::Identifier(ident) => {
                chain.push(ident.name);
                chain.reverse();
                return Some((ident, chain));
            }
            _ => break,
        }
        // If chain exceeds the max length, that means we are not interest in this member expression.
        // return `None`
        if chain.len() >= max_len {
            return None;
        }
    }
    None
}

/// https://github.com/evanw/esbuild/blob/d34e79e2a998c21bb71d57b92b0017ca11756912/internal/js_ast/js_ast_helpers.go#L2594-L2639
pub fn is_side_effect_free_unbound_identifier_ref<'a>(
    ctx: &impl MayHaveSideEffectsContext<'a>,
    value: &Expression<'a>,
    guard_condition: &Expression<'a>,
    mut is_yes_branch: bool,
) -> Option<bool> {
    let Expression::Identifier(ident) = value else {
        return None;
    };
    if !ctx.is_global_reference(ident) {
        return Some(false);
    }
    let Expression::BinaryExpression(bin_expr) = guard_condition else {
        return None;
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
                return Some(false);
            };
            if !(unary.operator == UnaryOperator::Typeof
                && matches!(unary.argument, Expression::Identifier(_)))
            {
                return Some(false);
            }
            let Expression::StringLiteral(string) = string else {
                return Some(false);
            };

            if (string.value.eq("undefined") == is_yes_branch)
                == matches!(
                    bin_expr.operator,
                    BinaryOperator::Inequality | BinaryOperator::StrictInequality
                )
            {
                let Expression::Identifier(type_of_value) = &unary.argument else {
                    return Some(false);
                };
                if type_of_value.name == ident.name {
                    return Some(true);
                }
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
                return Some(false);
            };
            if !(unary.operator == UnaryOperator::Typeof
                && matches!(unary.argument, Expression::Identifier(_)))
            {
                return Some(false);
            }

            let Expression::StringLiteral(string) = string else {
                return Some(false);
            };

            if string.value == "u"
                && is_yes_branch
                    == matches!(
                        bin_expr.operator,
                        BinaryOperator::LessThan | BinaryOperator::LessEqualThan
                    )
            {
                let Expression::Identifier(type_of_value) = &unary.argument else {
                    return Some(false);
                };
                if type_of_value.name == ident.name {
                    return Some(true);
                }
            }
        }
        _ => {}
    }
    Some(false)
}

/// https://github.com/evanw/esbuild/blob/d34e79e2a998c21bb71d57b92b0017ca11756912/internal/js_parser/js_parser.go#L16119-L16237
pub fn maybe_side_effect_free_global_constructor<'a>(
    ctx: &impl MayHaveSideEffectsContext<'a>,
    expr: &ast::NewExpression<'a>,
) -> bool {
    let Expression::Identifier(ident) = &expr.callee else {
        return false;
    };

    if ctx.is_global_reference(ident) {
        match ident.name.as_str() {
            // TypedArray constructors - considered side-effect free with no args, null, or undefined
            "Int8Array" | "Uint8Array" | "Uint8ClampedArray" | "Int16Array" | "Uint16Array"
            | "Int32Array" | "Uint32Array" | "Float32Array" | "Float64Array" | "BigInt64Array"
            | "BigUint64Array" => match expr.arguments.len() {
                0 => return true,
                1 => {
                    let arg = &expr.arguments[0];
                    match arg {
                        ast::Argument::NullLiteral(_) => return true,
                        ast::Argument::Identifier(id)
                            if id.name == "undefined" && ctx.is_global_reference(id) =>
                        {
                            return true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            "WeakSet" | "WeakMap" => match expr.arguments.len() {
                0 => return true,
                1 => {
                    let arg = &expr.arguments[0];
                    match arg {
                        ast::Argument::NullLiteral(_) => return true,
                        ast::Argument::Identifier(id)
                            if id.name == "undefined" && ctx.is_global_reference(id) =>
                        {
                            return true;
                        }
                        ast::Argument::ArrayExpression(arr) if arr.elements.is_empty() => {
                            return true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            "Date" => match expr.arguments.len() {
                0 => return true,
                1 => {
                    let arg = &expr.arguments[0];
                    let known_primitive_type =
                        arg.as_expression().map(|item| known_primitive_type(ctx, item));
                    if let Some(primitive_ty) = known_primitive_type {
                        if matches!(
                            primitive_ty,
                            PrimitiveType::Number
                                | PrimitiveType::String
                                | PrimitiveType::Null
                                | PrimitiveType::Undefined
                                | PrimitiveType::Boolean
                        ) {
                            return true;
                        }
                    }
                }
                _ => {}
            },
            "Set" => match expr.arguments.len() {
                0 => return true,
                1 => {
                    let arg = &expr.arguments[0];
                    match arg {
                        ast::Argument::NullLiteral(_) | ast::Argument::ArrayExpression(_) => {
                            return true;
                        }
                        ast::Argument::Identifier(id)
                            if id.name == "undefined" && ctx.is_global_reference(id) =>
                        {
                            return true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            "Map" => match expr.arguments.len() {
                0 => return true,
                1 => {
                    let arg = &expr.arguments[0];
                    match arg {
                        ast::Argument::NullLiteral(_) => return true,
                        ast::Argument::Identifier(id)
                            if id.name == "undefined" && ctx.is_global_reference(id) =>
                        {
                            return true;
                        }
                        ast::Argument::ArrayExpression(arr) => {
                            let all_entries_are_arrays = arr.elements.iter().all(|item| {
                                item.as_expression().is_some_and(|expr| {
                                    matches!(expr, ast::Expression::ArrayExpression(_))
                                })
                            });
                            if all_entries_are_arrays {
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            _ => {
                return check_global_free_constructor_args(
                    ident.name.as_str(),
                    &expr.arguments,
                    InvocationKind::New,
                    ctx,
                );
            }
        }
    }
    false
}

/// Represents the kind of invocation expression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationKind {
    /// CallExpression: `foo()`
    Call,
    /// NewExpression: `new Foo()`
    New,
}

/// Checks if a BigInt argument is safe (won't throw at runtime).
/// BigInt() throws for:
/// - Non-integer numbers (1.5, NaN, Infinity, -Infinity)
/// - Non-numeric strings ("abc")
///
/// We can only confidently say BigInt() is safe for:
/// - Integer numeric literals
/// - Boolean literals (true -> 1n, false -> 0n)
/// - BigInt literals
fn is_safe_bigint_argument(arg: &Expression) -> bool {
    match arg {
        // Boolean literals are always safe for BigInt (true -> 1n, false -> 0n)
        // BigInt literals are always safe
        Expression::BooleanLiteral(_) | Expression::BigIntLiteral(_) => true,
        // Numeric literals are safe only if they are integers (no decimal, not NaN, not Infinity)
        Expression::NumericLiteral(num) => {
            let value = num.value;
            // Check if it's a finite integer
            value.is_finite() && value.fract() == 0.0
        }
        // Unary expressions like -1 or +1
        Expression::UnaryExpression(unary) => {
            matches!(unary.operator, UnaryOperator::UnaryNegation | UnaryOperator::UnaryPlus)
                && matches!(unary.argument, Expression::NumericLiteral(ref num) if num.value.is_finite() && num.value.fract() == 0.0)
        }
        // String literals could be numeric but we can't easily validate, so consider them unsafe
        // For example, BigInt("123") is safe but BigInt("abc") or BigInt("1.5") throws
        _ => false,
    }
}

/// Checks if the arguments for a global free constructor/function are safe (side-effect free).
/// This function validates that all arguments are primitive types appropriate for the given symbol.
///
/// # Arguments
/// * `symbol_name` - The name of the global constructor/function (e.g., "Symbol", "BigInt", "String")
/// * `arguments` - The arguments being passed to the constructor/function
/// * `kind` - Whether this is a CallExpression or NewExpression
/// * `ctx` - Side effect context for global reference checks
///
/// # Note
/// The caller is responsible for ensuring that the symbol is global (unresolved) before calling this function.
pub fn check_global_free_constructor_args<'a>(
    symbol_name: &str,
    arguments: &ArenaVec<'a, ast::Argument<'a>>,
    kind: InvocationKind,
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> bool {
    // Note: `_kind` is reserved for future use to differentiate between Call and New expression logic
    match symbol_name {
        // BigInt() is special - it throws for non-integer numbers and non-numeric strings
        // We need to be more conservative and only allow proven-safe arguments
        "BigInt" => {
            if matches!(kind, InvocationKind::New) {
                // new BigInt() always throws TypeError
                return false;
            }
            // BigInt() requires at least one argument - BigInt() with no args throws TypeError
            if arguments.is_empty() {
                return false;
            }
            // BigInt() as a function call is only safe with proven-safe arguments
            arguments.iter().all(|arg| {
                if matches!(arg, ast::Argument::SpreadElement(_)) {
                    return false;
                }
                is_safe_bigint_argument(arg.to_expression())
            })
        }
        // RegExp() and new RegExp() - validate using oxc's regex parser
        // Invalid patterns or flags throw SyntaxError at runtime
        "RegExp" => is_valid_regexp(arguments),
        // Symbol() is side-effect-free only when arguments are primitive types
        // Calling toString() on an object can have side effects
        "Symbol" | "String" | "Number" | "Boolean" | "Object" => {
            // Check if all arguments are safe (primitives or no arguments)
            let is_side_effect_free = arguments.iter().all(|arg| {
                if matches!(arg, ast::Argument::SpreadElement(_)) {
                    return false;
                }
                let arg_expr = arg.to_expression();
                let prim_type = known_primitive_type(ctx, arg_expr);
                matches!(
                    prim_type,
                    PrimitiveType::Null
                        | PrimitiveType::Undefined
                        | PrimitiveType::Boolean
                        | PrimitiveType::Number
                        | PrimitiveType::String
                        | PrimitiveType::BigInt
                )
            });

            if matches!(kind, InvocationKind::New) {
                matches!(symbol_name, "Object" | "Number" | "String" | "Boolean")
                    && is_side_effect_free
            } else {
                is_side_effect_free
            }
        }
        _ => false,
    }
}

pub fn maybe_side_effect_free_global_function_call<'a>(
    ctx: &impl MayHaveSideEffectsContext<'a>,
    expr: &ast::CallExpression<'a>,
) -> bool {
    let Expression::Identifier(ident) = &expr.callee else {
        return false;
    };

    if ctx.is_global_reference(ident) {
        check_global_free_constructor_args(
            ident.name.as_str(),
            &expr.arguments,
            InvocationKind::Call,
            ctx,
        )
    } else {
        false
    }
}

// Ported from Rolldown global reference tables for side-effect detection parity.
static GLOBAL_IDENT: phf::Set<&str> = phf::phf_set![
    // Rolldown specific, because oxc treated them as identifiers, esbuild did not
    "Infinity",
    "undefined",
    "NaN",
    // These global identifiers should exist in all JavaScript environments.
    "Array",
    "Boolean",
    "Function",
    "Math",
    "Number",
    "Object",
    "RegExp",
    "String",
    // Other globals present in both the browser and node (except "eval" because
    // it has special behavior)
    "AbortController",
    "AbortSignal",
    "AggregateError",
    "ArrayBuffer",
    "BigInt",
    "DataView",
    "Date",
    "Error",
    "EvalError",
    "Event",
    "EventTarget",
    "Float32Array",
    "Float64Array",
    "Int16Array",
    "Int32Array",
    "Int8Array",
    "Intl",
    "JSON",
    "Map",
    "MessageChannel",
    "MessageEvent",
    "MessagePort",
    "Promise",
    "Proxy",
    "RangeError",
    "ReferenceError",
    "Reflect",
    "Set",
    "Symbol",
    "SyntaxError",
    "TextDecoder",
    "TextEncoder",
    "TypeError",
    "URIError",
    "URL",
    "URLSearchParams",
    "Uint16Array",
    "Uint32Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "WeakMap",
    "WeakSet",
    "WebAssembly",
    "clearInterval",
    "clearTimeout",
    "console",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
    "escape",
    "globalThis",
    "isFinite",
    "isNaN",
    "parseFloat",
    "parseInt",
    "queueMicrotask",
    "setInterval",
    "setTimeout",
    "unescape",
    // CSSOM APIs
    "CSSAnimation",
    "CSSFontFaceRule",
    "CSSImportRule",
    "CSSKeyframeRule",
    "CSSKeyframesRule",
    "CSSMediaRule",
    "CSSNamespaceRule",
    "CSSPageRule",
    "CSSRule",
    "CSSRuleList",
    "CSSStyleDeclaration",
    "CSSStyleRule",
    "CSSStyleSheet",
    "CSSSupportsRule",
    "CSSTransition",
    // SVG DOM
    "SVGAElement",
    "SVGAngle",
    "SVGAnimateElement",
    "SVGAnimateMotionElement",
    "SVGAnimateTransformElement",
    "SVGAnimatedAngle",
    "SVGAnimatedBoolean",
    "SVGAnimatedEnumeration",
    "SVGAnimatedInteger",
    "SVGAnimatedLength",
    "SVGAnimatedLengthList",
    "SVGAnimatedNumber",
    "SVGAnimatedNumberList",
    "SVGAnimatedPreserveAspectRatio",
    "SVGAnimatedRect",
    "SVGAnimatedString",
    "SVGAnimatedTransformList",
    "SVGAnimationElement",
    "SVGCircleElement",
    "SVGClipPathElement",
    "SVGComponentTransferFunctionElement",
    "SVGDefsElement",
    "SVGDescElement",
    "SVGElement",
    "SVGEllipseElement",
    "SVGFEBlendElement",
    "SVGFEColorMatrixElement",
    "SVGFEComponentTransferElement",
    "SVGFECompositeElement",
    "SVGFEConvolveMatrixElement",
    "SVGFEDiffuseLightingElement",
    "SVGFEDisplacementMapElement",
    "SVGFEDistantLightElement",
    "SVGFEDropShadowElement",
    "SVGFEFloodElement",
    "SVGFEFuncAElement",
    "SVGFEFuncBElement",
    "SVGFEFuncGElement",
    "SVGFEFuncRElement",
    "SVGFEGaussianBlurElement",
    "SVGFEImageElement",
    "SVGFEMergeElement",
    "SVGFEMergeNodeElement",
    "SVGFEMorphologyElement",
    "SVGFEOffsetElement",
    "SVGFEPointLightElement",
    "SVGFESpecularLightingElement",
    "SVGFESpotLightElement",
    "SVGFETileElement",
    "SVGFETurbulenceElement",
    "SVGFilterElement",
    "SVGForeignObjectElement",
    "SVGGElement",
    "SVGGeometryElement",
    "SVGGradientElement",
    "SVGGraphicsElement",
    "SVGImageElement",
    "SVGLength",
    "SVGLengthList",
    "SVGLineElement",
    "SVGLinearGradientElement",
    "SVGMPathElement",
    "SVGMarkerElement",
    "SVGMaskElement",
    "SVGMatrix",
    "SVGMetadataElement",
    "SVGNumber",
    "SVGNumberList",
    "SVGPathElement",
    "SVGPatternElement",
    "SVGPoint",
    "SVGPointList",
    "SVGPolygonElement",
    "SVGPolylineElement",
    "SVGPreserveAspectRatio",
    "SVGRadialGradientElement",
    "SVGRect",
    "SVGRectElement",
    "SVGSVGElement",
    "SVGScriptElement",
    "SVGSetElement",
    "SVGStopElement",
    "SVGStringList",
    "SVGStyleElement",
    "SVGSwitchElement",
    "SVGSymbolElement",
    "SVGTSpanElement",
    "SVGTextContentElement",
    "SVGTextElement",
    "SVGTextPathElement",
    "SVGTextPositioningElement",
    "SVGTitleElement",
    "SVGTransform",
    "SVGTransformList",
    "SVGUnitTypes",
    "SVGUseElement",
    "SVGViewElement",
    // Other browser APIs
    //
    // This list contains all globals present in modern versions of Chrome, Safari,
    // and Firefox except for the following properties, since they have a side effect
    // of triggering layout (https://gist.github.com/paulirish/5d52fb081b3570c81e3a):
    //
    //   - scrollX
    //   - scrollY
    //   - innerWidth
    //   - innerHeight
    //   - pageXOffset
    //   - pageYOffset
    //
    // The following globals have also been removed since they sometimes throw an
    // exception when accessed, which is a side effect (for more information see
    // https://stackoverflow.com/a/33047477):
    //
    //   - localStorage
    //   - sessionStorage
    //
    "AnalyserNode",
    "Animation",
    "AnimationEffect",
    "AnimationEvent",
    "AnimationPlaybackEvent",
    "AnimationTimeline",
    "Attr",
    "Audio",
    "AudioBuffer",
    "AudioBufferSourceNode",
    "AudioDestinationNode",
    "AudioListener",
    "AudioNode",
    "AudioParam",
    "AudioProcessingEvent",
    "AudioScheduledSourceNode",
    "BarProp",
    "BeforeUnloadEvent",
    "BiquadFilterNode",
    "Blob",
    "BlobEvent",
    "ByteLengthQueuingStrategy",
    "CDATASection",
    "CSS",
    "CanvasGradient",
    "CanvasPattern",
    "CanvasRenderingContext2D",
    "ChannelMergerNode",
    "ChannelSplitterNode",
    "CharacterData",
    "ClipboardEvent",
    "CloseEvent",
    "Comment",
    "CompositionEvent",
    "ConvolverNode",
    "CountQueuingStrategy",
    "Crypto",
    "CustomElementRegistry",
    "CustomEvent",
    "DOMException",
    "DOMImplementation",
    "DOMMatrix",
    "DOMMatrixReadOnly",
    "DOMParser",
    "DOMPoint",
    "DOMPointReadOnly",
    "DOMQuad",
    "DOMRect",
    "DOMRectList",
    "DOMRectReadOnly",
    "DOMStringList",
    "DOMStringMap",
    "DOMTokenList",
    "DataTransfer",
    "DataTransferItem",
    "DataTransferItemList",
    "DelayNode",
    "Document",
    "DocumentFragment",
    "DocumentTimeline",
    "DocumentType",
    "DragEvent",
    "DynamicsCompressorNode",
    "Element",
    "ErrorEvent",
    "EventSource",
    "File",
    "FileList",
    "FileReader",
    "FocusEvent",
    "FontFace",
    "FormData",
    "GainNode",
    "Gamepad",
    "GamepadButton",
    "GamepadEvent",
    "Geolocation",
    "GeolocationPositionError",
    "HTMLAllCollection",
    "HTMLAnchorElement",
    "HTMLAreaElement",
    "HTMLAudioElement",
    "HTMLBRElement",
    "HTMLBaseElement",
    "HTMLBodyElement",
    "HTMLButtonElement",
    "HTMLCanvasElement",
    "HTMLCollection",
    "HTMLDListElement",
    "HTMLDataElement",
    "HTMLDataListElement",
    "HTMLDetailsElement",
    "HTMLDirectoryElement",
    "HTMLDivElement",
    "HTMLDocument",
    "HTMLElement",
    "HTMLEmbedElement",
    "HTMLFieldSetElement",
    "HTMLFontElement",
    "HTMLFormControlsCollection",
    "HTMLFormElement",
    "HTMLFrameElement",
    "HTMLFrameSetElement",
    "HTMLHRElement",
    "HTMLHeadElement",
    "HTMLHeadingElement",
    "HTMLHtmlElement",
    "HTMLIFrameElement",
    "HTMLImageElement",
    "HTMLInputElement",
    "HTMLLIElement",
    "HTMLLabelElement",
    "HTMLLegendElement",
    "HTMLLinkElement",
    "HTMLMapElement",
    "HTMLMarqueeElement",
    "HTMLMediaElement",
    "HTMLMenuElement",
    "HTMLMetaElement",
    "HTMLMeterElement",
    "HTMLModElement",
    "HTMLOListElement",
    "HTMLObjectElement",
    "HTMLOptGroupElement",
    "HTMLOptionElement",
    "HTMLOptionsCollection",
    "HTMLOutputElement",
    "HTMLParagraphElement",
    "HTMLParamElement",
    "HTMLPictureElement",
    "HTMLPreElement",
    "HTMLProgressElement",
    "HTMLQuoteElement",
    "HTMLScriptElement",
    "HTMLSelectElement",
    "HTMLSlotElement",
    "HTMLSourceElement",
    "HTMLSpanElement",
    "HTMLStyleElement",
    "HTMLTableCaptionElement",
    "HTMLTableCellElement",
    "HTMLTableColElement",
    "HTMLTableElement",
    "HTMLTableRowElement",
    "HTMLTableSectionElement",
    "HTMLTemplateElement",
    "HTMLTextAreaElement",
    "HTMLTimeElement",
    "HTMLTitleElement",
    "HTMLTrackElement",
    "HTMLUListElement",
    "HTMLUnknownElement",
    "HTMLVideoElement",
    "HashChangeEvent",
    "Headers",
    "History",
    "IDBCursor",
    "IDBCursorWithValue",
    "IDBDatabase",
    "IDBFactory",
    "IDBIndex",
    "IDBKeyRange",
    "IDBObjectStore",
    "IDBOpenDBRequest",
    "IDBRequest",
    "IDBTransaction",
    "IDBVersionChangeEvent",
    "Image",
    "ImageData",
    "InputEvent",
    "IntersectionObserver",
    "IntersectionObserverEntry",
    "KeyboardEvent",
    "KeyframeEffect",
    "Location",
    "MediaCapabilities",
    "MediaElementAudioSourceNode",
    "MediaEncryptedEvent",
    "MediaError",
    "MediaList",
    "MediaQueryList",
    "MediaQueryListEvent",
    "MediaRecorder",
    "MediaSource",
    "MediaStream",
    "MediaStreamAudioDestinationNode",
    "MediaStreamAudioSourceNode",
    "MediaStreamTrack",
    "MediaStreamTrackEvent",
    "MimeType",
    "MimeTypeArray",
    "MouseEvent",
    "MutationEvent",
    "MutationObserver",
    "MutationRecord",
    "NamedNodeMap",
    "Navigator",
    "Node",
    "NodeFilter",
    "NodeIterator",
    "NodeList",
    "Notification",
    "OfflineAudioCompletionEvent",
    "Option",
    "OscillatorNode",
    "PageTransitionEvent",
    "Path2D",
    "Performance",
    "PerformanceEntry",
    "PerformanceMark",
    "PerformanceMeasure",
    "PerformanceNavigation",
    "PerformanceObserver",
    "PerformanceObserverEntryList",
    "PerformanceResourceTiming",
    "PerformanceTiming",
    "PeriodicWave",
    "Plugin",
    "PluginArray",
    "PointerEvent",
    "PopStateEvent",
    "ProcessingInstruction",
    "ProgressEvent",
    "PromiseRejectionEvent",
    "RTCCertificate",
    "RTCDTMFSender",
    "RTCDTMFToneChangeEvent",
    "RTCDataChannel",
    "RTCDataChannelEvent",
    "RTCIceCandidate",
    "RTCPeerConnection",
    "RTCPeerConnectionIceEvent",
    "RTCRtpReceiver",
    "RTCRtpSender",
    "RTCRtpTransceiver",
    "RTCSessionDescription",
    "RTCStatsReport",
    "RTCTrackEvent",
    "RadioNodeList",
    "Range",
    "ReadableStream",
    "Request",
    "ResizeObserver",
    "ResizeObserverEntry",
    "Response",
    "Screen",
    "ScriptProcessorNode",
    "SecurityPolicyViolationEvent",
    "Selection",
    "ShadowRoot",
    "SourceBuffer",
    "SourceBufferList",
    "SpeechSynthesisEvent",
    "SpeechSynthesisUtterance",
    "StaticRange",
    "Storage",
    "StorageEvent",
    "StyleSheet",
    "StyleSheetList",
    "Text",
    "TextMetrics",
    "TextTrack",
    "TextTrackCue",
    "TextTrackCueList",
    "TextTrackList",
    "TimeRanges",
    "TrackEvent",
    "TransitionEvent",
    "TreeWalker",
    "UIEvent",
    "VTTCue",
    "ValidityState",
    "VisualViewport",
    "WaveShaperNode",
    "WebGLActiveInfo",
    "WebGLBuffer",
    "WebGLContextEvent",
    "WebGLFramebuffer",
    "WebGLProgram",
    "WebGLQuery",
    "WebGLRenderbuffer",
    "WebGLRenderingContext",
    "WebGLSampler",
    "WebGLShader",
    "WebGLShaderPrecisionFormat",
    "WebGLSync",
    "WebGLTexture",
    "WebGLUniformLocation",
    "WebKitCSSMatrix",
    "WebSocket",
    "WheelEvent",
    "Window",
    "Worker",
    "XMLDocument",
    "XMLHttpRequest",
    "XMLHttpRequestEventTarget",
    "XMLHttpRequestUpload",
    "XMLSerializer",
    "XPathEvaluator",
    "XPathExpression",
    "XPathResult",
    "XSLTProcessor",
    "alert",
    "atob",
    "blur",
    "btoa",
    "cancelAnimationFrame",
    "captureEvents",
    "close",
    "closed",
    "confirm",
    "customElements",
    "devicePixelRatio",
    "document",
    "event",
    "fetch",
    "find",
    "focus",
    "frameElement",
    "frames",
    "getComputedStyle",
    "getSelection",
    "history",
    "indexedDB",
    "isSecureContext",
    "length",
    "location",
    "locationbar",
    "matchMedia",
    "menubar",
    "moveBy",
    "moveTo",
    "name",
    "navigator",
    "onabort",
    "onafterprint",
    "onanimationend",
    "onanimationiteration",
    "onanimationstart",
    "onbeforeprint",
    "onbeforeunload",
    "onblur",
    "oncanplay",
    "oncanplaythrough",
    "onchange",
    "onclick",
    "oncontextmenu",
    "oncuechange",
    "ondblclick",
    "ondrag",
    "ondragend",
    "ondragenter",
    "ondragleave",
    "ondragover",
    "ondragstart",
    "ondrop",
    "ondurationchange",
    "onemptied",
    "onended",
    "onerror",
    "onfocus",
    "ongotpointercapture",
    "onhashchange",
    "oninput",
    "oninvalid",
    "onkeydown",
    "onkeypress",
    "onkeyup",
    "onlanguagechange",
    "onload",
    "onloadeddata",
    "onloadedmetadata",
    "onloadstart",
    "onlostpointercapture",
    "onmessage",
    "onmousedown",
    "onmouseenter",
    "onmouseleave",
    "onmousemove",
    "onmouseout",
    "onmouseover",
    "onmouseup",
    "onoffline",
    "ononline",
    "onpagehide",
    "onpageshow",
    "onpause",
    "onplay",
    "onplaying",
    "onpointercancel",
    "onpointerdown",
    "onpointerenter",
    "onpointerleave",
    "onpointermove",
    "onpointerout",
    "onpointerover",
    "onpointerup",
    "onpopstate",
    "onprogress",
    "onratechange",
    "onrejectionhandled",
    "onreset",
    "onresize",
    "onscroll",
    "onseeked",
    "onseeking",
    "onselect",
    "onstalled",
    "onstorage",
    "onsubmit",
    "onsuspend",
    "ontimeupdate",
    "ontoggle",
    "ontransitioncancel",
    "ontransitionend",
    "ontransitionrun",
    "ontransitionstart",
    "onunhandledrejection",
    "onunload",
    "onvolumechange",
    "onwaiting",
    "onwebkitanimationend",
    "onwebkitanimationiteration",
    "onwebkitanimationstart",
    "onwebkittransitionend",
    "onwheel",
    "open",
    "opener",
    "origin",
    "outerHeight",
    "outerWidth",
    "parent",
    "performance",
    "personalbar",
    "postMessage",
    "print",
    "prompt",
    "releaseEvents",
    "requestAnimationFrame",
    "resizeBy",
    "resizeTo",
    "screen",
    "screenLeft",
    "screenTop",
    "screenX",
    "screenY",
    "scroll",
    "scrollBy",
    "scrollTo",
    "scrollbars",
    "self",
    "speechSynthesis",
    "status",
    "statusbar",
    "stop",
    "toolbar",
    "top",
    "webkitURL",
    "window",
];

/// `Math`
static MATH_SECOND_PROP: phf::Set<&str> = phf::phf_set![
    // Math: Static properties
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math#Static_properties
    "E", "LN10", "LN2", "LOG10E", "LOG2E", "PI", "SQRT1_2", "SQRT2",
    // Math: Static methods
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math#Static_methods
    "abs", "acos", "acosh", "asin", "asinh", "atan", "atan2", "atanh", "cbrt", "ceil", "clz32",
    "cos", "cosh", "exp", "expm1", "floor", "fround", "hypot", "imul", "log", "log10", "log1p",
    "log2", "max", "min", "pow", "random", "round", "sign", "sin", "sinh", "sqrt", "tan", "tanh",
    "trunc",
];

/// Console method references are assumed to have no side effects
/// https://developer.mozilla.org/en-US/docs/Web/API/console
/// `console`
static CONSOLE_SECOND_PROP: [&str; 19] = [
    "assert",
    "clear",
    "count",
    "countReset",
    "debug",
    "dir",
    "dirxml",
    "error",
    "group",
    "groupCollapsed",
    "groupEnd",
    "info",
    "log",
    "table",
    "time",
    "timeEnd",
    "timeLog",
    "trace",
    "warn",
];

/// Reflect: Static methods
/// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect#static_methods
/// `Reflect`
static REFLECT_SECOND_PROP: [&str; 13] = [
    "apply",
    "construct",
    "defineProperty",
    "deleteProperty",
    "get",
    "getOwnPropertyDescriptor",
    "getPrototypeOf",
    "has",
    "isExtensible",
    "ownKeys",
    "preventExtensions",
    "set",
    "setPrototypeOf",
];

/// `Object`
static OBJECT_SECOND_PROP: [&str; 21] = [
    // Object: Static methods
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object#Static_methods
    "assign",
    "create",
    "defineProperties",
    "defineProperty",
    "entries",
    "freeze",
    "fromEntries",
    "getOwnPropertyDescriptor",
    "getOwnPropertyDescriptors",
    "getOwnPropertyNames",
    "getOwnPropertySymbols",
    "getPrototypeOf",
    "is",
    "isExtensible",
    "isFrozen",
    "isSealed",
    "keys",
    "preventExtensions",
    "seal",
    "setPrototypeOf",
    "values",
];

/// `Symbol`
static SYMBOL_SECOND_PROP: [&str; 15] = [
    // Symbol: Static properties
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol#static_properties
    "asyncDispose",
    "asyncIterator",
    "dispose",
    "hasInstance",
    "isConcatSpreadable",
    "iterator",
    "match",
    "matchAll",
    "replace",
    "search",
    "species",
    "split",
    "toPrimitive",
    "toStringTag",
    "unscopables",
];

/// `Object.prototype`
static OBJECT_PROTOTYPE_THIRD_PROP: [&str; 12] = [
    // Object: Instance methods
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object#Instance_methods
    "__defineGetter__",
    "__defineSetter__",
    "__lookupGetter__",
    "__lookupSetter__",
    "hasOwnProperty",
    "isPrototypeOf",
    "propertyIsEnumerable",
    "toLocaleString",
    "toString",
    "unwatch",
    "valueOf",
    "watch",
];

#[inline]
pub fn is_well_known_global_ident_ref(ident: &str) -> bool {
    GLOBAL_IDENT.contains(ident)
}

pub fn is_side_effect_free_member_expr_of_len_two(member_expr: &[Ident]) -> bool {
    match member_expr {
        [first, second] => {
            let second = second.as_str();
            match first.as_str() {
                "console" => CONSOLE_SECOND_PROP.contains(&second),
                "Reflect" => REFLECT_SECOND_PROP.contains(&second),
                "Math" => MATH_SECOND_PROP.contains(second),
                "Object" => OBJECT_SECOND_PROP.contains(&second),
                "Symbol" => SYMBOL_SECOND_PROP.contains(&second),
                "JSON" => second == "stringify" || second == "parse",
                _ => false,
            }
        }
        _ => false,
    }
}

pub fn is_side_effect_free_member_expr_of_len_three(member_expr: &[Ident]) -> bool {
    match member_expr {
        [first, second, third] => {
            first == "Object"
                && second == "prototype"
                && OBJECT_PROTOTYPE_THIRD_PROP.contains(&third.as_str())
        }
        _ => false,
    }
}
