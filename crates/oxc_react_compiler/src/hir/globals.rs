/// Global type definitions for built-in JavaScript and React objects.
///
/// Port of `HIR/Globals.ts` from the React Compiler.
///
/// Defines the default shape registry and global registry used for
/// type and effect inference of built-in JavaScript globals and React hooks.
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
    hir_types::{Effect, ValueKind, ValueReason},
    object_shape::{
        FunctionSignature, HookKind, ShapeRegistry,
        BUILT_IN_ARRAY_ID, BUILT_IN_MAP_ID, BUILT_IN_MIXED_READONLY_ID,
        BUILT_IN_OBJECT_ID, BUILT_IN_SET_ID, BUILT_IN_USE_CONTEXT_HOOK_ID,
        BUILT_IN_USE_EFFECT_HOOK_ID, BUILT_IN_USE_REF_ID, BUILT_IN_USE_STATE_ID,
        BUILT_IN_WEAK_MAP_ID, BUILT_IN_WEAK_SET_ID,
        add_hook, add_object,
    },
    types::{ObjectType, Type},
};

/// A global type entry — either a hook or a non-hook global.
#[derive(Debug, Clone)]
pub enum Global {
    /// A global variable with known type.
    Typed(Type),
    /// An untyped global (we don't have shape information for it).
    Untyped,
}

/// Registry mapping global names to their types.
pub type GlobalRegistry = FxHashMap<String, Global>;

/// Set of global names that are untyped (no shape information).
fn untyped_globals() -> FxHashSet<String> {
    [
        "Object", "Function", "RegExp", "Date", "Error", "TypeError",
        "RangeError", "ReferenceError", "SyntaxError", "URIError", "EvalError",
        "DataView", "Float32Array", "Float64Array", "Int8Array", "Int16Array",
        "Int32Array", "WeakMap", "Uint8Array", "Uint8ClampedArray", "Uint16Array",
        "Uint32Array", "ArrayBuffer", "JSON", "console", "performance", "window",
        "document", "navigator", "Promise", "Symbol", "Proxy", "Reflect",
        "Intl", "Number", "String", "Boolean", "Math", "globalThis",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// Build the default shape registry with built-in type definitions.
pub fn default_shapes() -> ShapeRegistry {
    let mut registry = ShapeRegistry::default();

    // Built-in array shape
    add_object(&mut registry, BUILT_IN_ARRAY_ID, Vec::new());

    // Built-in object shape
    add_object(&mut registry, BUILT_IN_OBJECT_ID, Vec::new());

    // Built-in Set shape
    add_object(&mut registry, BUILT_IN_SET_ID, Vec::new());

    // Built-in Map shape
    add_object(&mut registry, BUILT_IN_MAP_ID, Vec::new());

    // Built-in WeakSet/WeakMap
    add_object(&mut registry, BUILT_IN_WEAK_SET_ID, Vec::new());
    add_object(&mut registry, BUILT_IN_WEAK_MAP_ID, Vec::new());

    // Built-in MixedReadonly
    add_object(&mut registry, BUILT_IN_MIXED_READONLY_ID, Vec::new());

    registry
}

/// Build the default global registry with React hooks and JS globals.
pub fn default_globals(shapes: &mut ShapeRegistry) -> GlobalRegistry {
    let mut globals = GlobalRegistry::default();

    // Add untyped globals
    for name in untyped_globals() {
        globals.insert(name, Global::Untyped);
    }

    // React hooks
    add_react_hook_globals(&mut globals, shapes);

    // Common global functions
    add_global_function_globals(&mut globals, shapes);

    globals
}

fn add_react_hook_globals(globals: &mut GlobalRegistry, shapes: &mut ShapeRegistry) {
    // useState
    let use_state_id = add_hook(
        shapes,
        Some(BUILT_IN_USE_STATE_ID),
        FunctionSignature {
            positional_params: vec![Effect::Freeze],
            rest_param: None,
            return_type: Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_USE_STATE_ID.to_string()),
            }),
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::State),
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseState),
            ..FunctionSignature::default()
        },
    );
    globals.insert("useState".to_string(), Global::Typed(Type::Function(
        crate::hir::types::FunctionType {
            shape_id: Some(use_state_id),
            return_type: Box::new(Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_USE_STATE_ID.to_string()),
            })),
            is_constructor: false,
        },
    )));

    // useRef
    let use_ref_id = add_hook(
        shapes,
        Some(BUILT_IN_USE_REF_ID),
        FunctionSignature {
            positional_params: vec![Effect::Freeze],
            rest_param: None,
            return_type: Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_USE_REF_ID.to_string()),
            }),
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::KnownReturnSignature),
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseRef),
            ..FunctionSignature::default()
        },
    );
    globals.insert("useRef".to_string(), Global::Typed(Type::Function(
        crate::hir::types::FunctionType {
            shape_id: Some(use_ref_id),
            return_type: Box::new(Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_USE_REF_ID.to_string()),
            })),
            is_constructor: false,
        },
    )));

    // useEffect
    let use_effect_id = add_hook(
        shapes,
        Some(BUILT_IN_USE_EFFECT_HOOK_ID),
        FunctionSignature {
            positional_params: vec![Effect::Freeze, Effect::Freeze],
            rest_param: None,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseEffect),
            ..FunctionSignature::default()
        },
    );
    globals.insert("useEffect".to_string(), Global::Typed(Type::Function(
        crate::hir::types::FunctionType {
            shape_id: Some(use_effect_id),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        },
    )));

    // useContext
    let use_context_id = add_hook(
        shapes,
        Some(BUILT_IN_USE_CONTEXT_HOOK_ID),
        FunctionSignature {
            positional_params: vec![Effect::Read],
            rest_param: None,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::Context),
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseContext),
            ..FunctionSignature::default()
        },
    );
    globals.insert("useContext".to_string(), Global::Typed(Type::Function(
        crate::hir::types::FunctionType {
            shape_id: Some(use_context_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        },
    )));

    // useMemo
    globals.insert("useMemo".to_string(), Global::Typed(Type::Function(
        crate::hir::types::FunctionType {
            shape_id: None,
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        },
    )));

    // useCallback
    globals.insert("useCallback".to_string(), Global::Typed(Type::Function(
        crate::hir::types::FunctionType {
            shape_id: None,
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        },
    )));
}

fn add_global_function_globals(globals: &mut GlobalRegistry, _shapes: &mut ShapeRegistry) {
    // Array.isArray
    globals.insert("Array".to_string(), Global::Untyped);

    // Common pure functions
    for name in &["parseInt", "parseFloat", "isNaN", "isFinite", "encodeURIComponent",
                   "decodeURIComponent", "encodeURI", "decodeURI", "btoa", "atob"] {
        globals.insert((*name).to_string(), Global::Typed(Type::Function(
            crate::hir::types::FunctionType {
                shape_id: None,
                return_type: Box::new(Type::Primitive),
                is_constructor: false,
            },
        )));
    }

    // undefined, NaN, Infinity
    globals.insert("undefined".to_string(), Global::Typed(Type::Primitive));
    globals.insert("NaN".to_string(), Global::Typed(Type::Primitive));
    globals.insert("Infinity".to_string(), Global::Typed(Type::Primitive));

    // null is not a global in JS (it's a keyword/literal), but we track it
    globals.insert("null".to_string(), Global::Typed(Type::Primitive));
}
