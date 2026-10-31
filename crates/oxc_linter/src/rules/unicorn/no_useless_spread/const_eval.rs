use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, ConditionalExpression, Expression, NewExpression,
        match_expression,
    },
};

use crate::{
    ast_util::{
        get_declaration_of_variable, is_method_call, is_new_expression, variable_declaration_kind,
    },
    context::LintContext,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrayKind {
    Array,
    NonArray,
    Unknown,
}

/// Classify a method receiver without treating a reference to an existing array
/// as a new allocation. A bare `[...array]` must still be allowed.
pub(super) fn array_receiver_kind<'a>(
    call: &CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> ArrayKind {
    let Some(member) = call.callee.get_member_expr() else {
        return ArrayKind::Unknown;
    };
    // Bound the total work, including cyclic aliases and branching initializers.
    array_kind(member.object(), ctx, &mut 32)
}

fn array_kind<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>, remaining: &mut u8) -> ArrayKind {
    if *remaining == 0 {
        return ArrayKind::Unknown;
    }
    *remaining -= 1;

    match expr.get_inner_expression() {
        Expression::Identifier(ident) => {
            let Some(node) = get_declaration_of_variable(ident, ctx) else {
                return ArrayKind::Unknown;
            };
            let AstKind::VariableDeclarator(declarator) = node.kind() else {
                return ArrayKind::Unknown;
            };
            if !variable_declaration_kind(declarator, ctx).is_const()
                || declarator.id.get_binding_identifier().is_none()
            {
                return ArrayKind::Unknown;
            }
            declarator
                .init
                .as_ref()
                .map_or(ArrayKind::Unknown, |init| array_kind(init, ctx, remaining))
        }
        Expression::StringLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::ObjectExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_) => ArrayKind::NonArray,
        Expression::NewExpression(new) => {
            if is_new_array(new) {
                ArrayKind::Array
            } else {
                ArrayKind::NonArray
            }
        }
        Expression::CallExpression(call)
            if is_slice_method(call) || is_functional_array_method(call) =>
        {
            let Some(member) = call.callee.get_member_expr() else {
                return ArrayKind::Unknown;
            };
            let kind = array_kind(member.object(), ctx, remaining);
            if kind == ArrayKind::Unknown
                && !is_method_call(call, None, Some(&["slice", "concat"]), None, None)
            {
                ArrayKind::Array
            } else {
                kind
            }
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .map_or(ArrayKind::Unknown, |last| array_kind(last, ctx, remaining)),
        Expression::ConditionalExpression(conditional) => {
            let consequent = array_kind(&conditional.consequent, ctx, remaining);
            let alternate = array_kind(&conditional.alternate, ctx, remaining);
            if consequent == alternate { consequent } else { ArrayKind::Unknown }
        }
        _ => match expr.const_eval() {
            ValueHint::NewArray => ArrayKind::Array,
            ValueHint::NewTypedArray | ValueHint::NewIterable | ValueHint::NewObject => {
                ArrayKind::NonArray
            }
            _ => ArrayKind::Unknown,
        },
    }
}

#[derive(Debug, Clone)]
pub(super) enum ValueHint {
    NewObject,
    NewArray,
    /// A non-array iterable.
    ///
    /// Note that typed arrays are considered arrays, not iterables.
    NewIterable,
    /// A typed array: `new Uint8Array(x)`, `Uint8Array.from(x)`, or a clone
    /// method called on either.
    ///
    /// Deliberately distinct from [`ValueHint::NewArray`]. Spreading a typed
    /// array produces a plain array, so `[...typedArray.slice()]` performs a
    /// real conversion rather than a useless clone.
    NewTypedArray,
    Promise(Box<ValueHint>),
    Unknown,
}

impl ValueHint {
    pub fn r#await(self) -> Self {
        match self {
            Self::Promise(inner) => *inner,
            _ => self,
        }
    }

    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::NewObject)
    }

    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::NewArray)
    }

    #[inline]
    pub fn is_typed_array(&self) -> bool {
        matches!(self, Self::NewTypedArray)
    }
}

impl std::ops::BitAnd for ValueHint {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        // NOTE: what about (NewArray, NewIterable), e.g. in
        // `foo ? new Set() : []`
        match (self, rhs) {
            (Self::NewArray, Self::NewArray) => Self::NewArray,
            (Self::NewObject, Self::NewObject) => Self::NewObject,
            (Self::NewIterable, Self::NewIterable) => Self::NewIterable,
            (Self::NewTypedArray, Self::NewTypedArray) => Self::NewTypedArray,
            _ => Self::Unknown,
        }
    }
}
pub(super) trait ConstEval {
    fn const_eval(&self) -> ValueHint;
}

impl ConstEval for Expression<'_> {
    fn const_eval(&self) -> ValueHint {
        match self.get_inner_expression() {
            Self::ArrayExpression(_) => ValueHint::NewArray,
            Self::ObjectExpression(_) => ValueHint::NewObject,
            Self::AwaitExpression(expr) => expr.argument.const_eval().r#await(),
            Self::SequenceExpression(expr) => {
                expr.expressions.last().map_or(ValueHint::Unknown, ConstEval::const_eval)
            }
            Self::ConditionalExpression(cond) => cond.const_eval(),
            Self::CallExpression(call) => call.const_eval(),
            Self::NewExpression(new) => new.const_eval(),
            _ => ValueHint::Unknown,
        }
    }
}

impl ConstEval for ConditionalExpression<'_> {
    fn const_eval(&self) -> ValueHint {
        self.consequent.const_eval() & self.alternate.const_eval()
    }
}

impl ConstEval for Argument<'_> {
    fn const_eval(&self) -> ValueHint {
        match self {
            // using a spread as an initial accumulator value creates a new
            // object or array
            Self::SpreadElement(spread) => spread.argument.const_eval(),
            expr @ match_expression!(Argument) => expr.as_expression().unwrap().const_eval(),
        }
    }
}

impl ConstEval for NewExpression<'_> {
    fn const_eval(&self) -> ValueHint {
        if is_new_array(self) {
            ValueHint::NewArray
        } else if is_new_typed_array(self) {
            ValueHint::NewTypedArray
        } else if is_new_map_or_set(self) {
            ValueHint::NewIterable
        } else if is_new_object(self) {
            ValueHint::NewObject
        } else {
            ValueHint::Unknown
        }
    }
}

fn is_new_array(new_expr: &NewExpression) -> bool {
    is_new_expression(new_expr, &["Array"], None, None)
}

/// Matches `new {Set,WeakSet,Map,WeakMap}(iterable?)`
fn is_new_map_or_set(new_expr: &NewExpression) -> bool {
    is_new_expression(new_expr, &["Map", "WeakMap", "Set", "WeakSet"], None, Some(1))
}

/// Matches `new Object()` with any number of args.
fn is_new_object(new_expr: &NewExpression) -> bool {
    is_new_expression(new_expr, &["Object"], None, None)
}

/// Every typed array constructor.
const TYPED_ARRAY_NAMES: [&str; 12] = [
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float16Array",
    "Float32Array",
    "Float64Array",
    "BigInt64Array",
    "BigUint64Array",
];

/// Matches `new <TypedArray>(a, [other args])` with >= 1 arg
pub fn is_new_typed_array(new_expr: &NewExpression) -> bool {
    is_new_expression(new_expr, &TYPED_ARRAY_NAMES, Some(1), None)
}

/// Matches `<TypedArray>.from(x)`, whose result is a typed array.
fn is_typed_array_from(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, Some(&TYPED_ARRAY_NAMES), Some(&["from"]), Some(1), Some(1))
}

/// Matches a functional array method called on a typed array, e.g.
/// `Uint8Array.from(x).map(f)`.
///
/// These return a typed array rather than a plain array, so the surrounding
/// spread converts and cannot be removed.
fn is_typed_array_method(call_expr: &CallExpression) -> bool {
    if !is_functional_array_method(call_expr) {
        return false;
    }
    call_expr
        .callee
        .get_member_expr()
        .is_some_and(|member_expr| member_expr.object().const_eval().is_typed_array())
}

impl ConstEval for CallExpression<'_> {
    fn const_eval(&self) -> ValueHint {
        if is_typed_array_from(self) {
            ValueHint::NewTypedArray
        } else if is_slice_method(self) {
            self.callee.get_member_expr().map_or(
                ValueHint::Unknown,
                |member_expr| match member_expr.object().const_eval() {
                    ValueHint::NewArray => ValueHint::NewArray,
                    ValueHint::NewTypedArray => ValueHint::NewTypedArray,
                    _ => ValueHint::Unknown,
                },
            )
        } else if is_typed_array_method(self) {
            ValueHint::NewTypedArray
        } else if is_split_method(self)
            || is_array_factory(self)
            || is_functional_array_method(self)
            || is_array_producing_obj_method(self)
        {
            ValueHint::NewArray
        } else if is_array_reduce(self) {
            self.arguments[1].const_eval()
        } else if is_promise_array_method(self) {
            ValueHint::Promise(Box::new(ValueHint::NewArray))
        } else if is_obj_factory(self) {
            ValueHint::NewObject
        } else {
            // TODO: check initial value for arr.reduce() accumulators
            ValueHint::Unknown
        }
    }
}

/// - `Array.from(x)`
/// - `Int8Array.from(x)`
/// - plus all other typed arrays
///
/// Used to decide whether a spread passed *into* `from` is useless, which it is
/// for every constructor here because they all accept an iterable. Do NOT use
/// this to infer the type of the *result*: `Uint8Array.from(x)` returns a typed
/// array. See [`is_typed_array_from`].
pub fn is_array_from(call_expr: &CallExpression) -> bool {
    is_method_call(
        call_expr,
        Some(&[
            "Array",
            "Int8Array",
            "Uint8Array",
            "Uint8ClampedArray",
            "Int16Array",
            "Uint16Array",
            "Int32Array",
            "Uint32Array",
            "Float16Array",
            "Float32Array",
            "Float64Array",
            "BigInt64Array",
            "BigUint64Array",
        ]),
        Some(&["from"]),
        Some(1),
        Some(1),
    )
}
/// `<expr>.{concat,map,filter,...}`
fn is_functional_array_method(call_expr: &CallExpression) -> bool {
    is_method_call(
        call_expr,
        None,
        Some(&[
            "concat",
            "copyWithin",
            "filter",
            "flat",
            "flatMap",
            "map",
            "splice",
            "toReversed",
            "toSorted",
            "toSpliced",
            "with",
        ]),
        None,
        None,
    )
}

fn is_slice_method(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, None, Some(&["slice"]), None, None)
}

/// Matches `<expr>.reduce(a, b)`, which usually looks like
/// ```ts
/// arr.reduce(reducerRn, initialAccumulator)
/// ```
fn is_array_reduce(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, None, Some(&["reduce"]), Some(2), Some(2))
}

/// Matches `<expr>.split(...)`, which usually is `String.prototype.split(pattern)`
fn is_split_method(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, None, Some(&["split"]), None, None)
}

/// Matches `Object.{fromEntries,create}(x)`
fn is_obj_factory(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, Some(&["Object"]), Some(&["fromEntries", "create"]), Some(1), Some(1))
}

/// Matches `Object.{keys,values,entries}(...)`
fn is_array_producing_obj_method(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, Some(&["Object"]), Some(&["keys", "values", "entries"]), None, None)
}

/// Matches `Array.{from,of}(...)`
fn is_array_factory(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, Some(&["Array"]), Some(&["from", "of"]), None, None)
}

/// Matches `Promise.{all,allSettled}(x)`
fn is_promise_array_method(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, Some(&["Promise"]), Some(&["all", "allSettled"]), Some(1), Some(1))
}
