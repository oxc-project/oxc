use oxc_ast::ast::{
    match_expression, Argument, CallExpression, ConditionalExpression, Expression, NewExpression,
};

use crate::ast_util::{is_method_call, is_new_expression};

#[derive(Debug, Clone)]
pub(super) enum ValueHint {
    NewObject,
    NewArray,
    /// A non-array iterable.
    ///
    /// Note that typed arrays are considered arrays, not iterables.
    NewIterable,
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

/// Matches `new <TypedArray>(a, [other args])` with >= 1 arg
pub fn is_new_typed_array(new_expr: &NewExpression) -> bool {
    is_new_expression(
        new_expr,
        &[
            "Int8Array",
            "Uint8Array",
            "Uint8ClampedArray",
            "Int16Array",
            "Uint16Array",
            "Int32Array",
            "Uint32Array",
            "Float32Array",
            "Float64Array",
            "BigInt64Array",
            "BigUint64Array",
        ],
        Some(1),
        None,
    )
}

impl ConstEval for CallExpression<'_> {
    fn const_eval(&self) -> ValueHint {
        if is_array_from(self)
            || is_split_method(self)
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
            "slice",
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
