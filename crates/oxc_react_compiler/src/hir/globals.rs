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
        BUILT_IN_ARRAY_ID, BUILT_IN_EFFECT_EVENT_ID, BUILT_IN_MAP_ID, BUILT_IN_MIXED_READONLY_ID,
        BUILT_IN_OBJECT_ID, BUILT_IN_SET_ID, BUILT_IN_USE_ACTION_STATE_ID,
        BUILT_IN_USE_ACTION_STATE_HOOK_ID, BUILT_IN_USE_CONTEXT_HOOK_ID,
        BUILT_IN_USE_EFFECT_EVENT_ID, BUILT_IN_USE_EFFECT_HOOK_ID,
        BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID, BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID,
        BUILT_IN_USE_OPERATOR_ID, BUILT_IN_USE_OPTIMISTIC_HOOK_ID, BUILT_IN_USE_OPTIMISTIC_ID,
        BUILT_IN_USE_REDUCER_HOOK_ID, BUILT_IN_USE_REDUCER_ID, BUILT_IN_USE_REF_HOOK_ID,
        BUILT_IN_USE_REF_ID, BUILT_IN_USE_STATE_HOOK_ID, BUILT_IN_USE_STATE_ID,
        BUILT_IN_USE_TRANSITION_HOOK_ID, BUILT_IN_USE_TRANSITION_ID,
        BUILT_IN_WEAK_MAP_ID, BUILT_IN_WEAK_SET_ID, FunctionSignature, HookKind, ShapeRegistry,
        add_function, add_hook, add_object,
    },
    types::{FunctionType, ObjectType, Type},
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
    // Note: Object, Array, Math, console, Map, Set, WeakMap, WeakSet are now typed globals
    // with proper shape information. They are added in add_global_function_globals().
    [
        "Function",
        "RegExp",
        "Date",
        "Error",
        "TypeError",
        "RangeError",
        "ReferenceError",
        "SyntaxError",
        "URIError",
        "EvalError",
        "DataView",
        "Float32Array",
        "Float64Array",
        "Int8Array",
        "Int16Array",
        "Int32Array",
        "Uint8Array",
        "Uint8ClampedArray",
        "Uint16Array",
        "Uint32Array",
        "ArrayBuffer",
        "JSON",
        "performance",
        "window",
        "document",
        "navigator",
        "Promise",
        "Symbol",
        "Proxy",
        "Reflect",
        "Intl",
        "Number",
        "String",
        "Boolean",
        "globalThis",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// Helper: create a method property (Function type referencing a shape).
fn method_prop(registry: &mut ShapeRegistry, sig: FunctionSignature, return_type: Type) -> Type {
    let id = add_function(registry, None, Vec::new(), sig);
    Type::Function(FunctionType {
        shape_id: Some(id),
        return_type: Box::new(return_type),
        is_constructor: false,
    })
}

/// Build the default shape registry with built-in type definitions.
pub fn default_shapes() -> ShapeRegistry {
    use super::object_shape::{
        BUILT_IN_DISPATCH_ID, BUILT_IN_REF_VALUE_ID, BUILT_IN_SET_ACTION_STATE_ID,
        BUILT_IN_SET_OPTIMISTIC_ID, BUILT_IN_SET_STATE_ID, BUILT_IN_START_TRANSITION_ID,
    };

    let mut registry = ShapeRegistry::default();

    let array_type = Type::Object(ObjectType { shape_id: Some(BUILT_IN_ARRAY_ID.to_string()) });
    let set_type = Type::Object(ObjectType { shape_id: Some(BUILT_IN_SET_ID.to_string()) });
    let map_type = Type::Object(ObjectType { shape_id: Some(BUILT_IN_MAP_ID.to_string()) });
    let weak_set_type =
        Type::Object(ObjectType { shape_id: Some(BUILT_IN_WEAK_SET_ID.to_string()) });
    let weak_map_type =
        Type::Object(ObjectType { shape_id: Some(BUILT_IN_WEAK_MAP_ID.to_string()) });

    // =========================================================================
    // Built-in Array shape — instance methods
    // =========================================================================
    {
        let r = &mut registry;
        let mut props = Vec::new();
        props.push(("length".to_string(), Type::Primitive));

        // indexOf, includes, join — read-only
        for name in &["indexOf", "includes", "join"] {
            let t = method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::Read),
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Read,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            );
            props.push(((*name).to_string(), t));
        }

        // pop — Store effect, returns Poly
        props.push((
            "pop".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    return_type: Type::Poly,
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                Type::Poly,
            ),
        ));

        // at — Capture, returns Poly
        props.push((
            "at".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Poly,
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Capture,
                    ..FunctionSignature::default()
                },
                Type::Poly,
            ),
        ));

        // concat — Capture, returns Array
        props.push((
            "concat".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::Capture),
                    return_type: array_type.clone(),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Capture,
                    ..FunctionSignature::default()
                },
                array_type.clone(),
            ),
        ));

        // push — Store, returns Primitive
        props.push((
            "push".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::Capture),
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        // slice — Capture, returns Array
        props.push((
            "slice".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::Read),
                    return_type: array_type.clone(),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Capture,
                    ..FunctionSignature::default()
                },
                array_type.clone(),
            ),
        ));

        // map, flatMap, filter — ConditionallyMutate, returns Array
        for name in &["map", "flatMap", "filter"] {
            let t = method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: array_type.clone(),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::ConditionallyMutate,
                    ..FunctionSignature::default()
                },
                array_type.clone(),
            );
            props.push(((*name).to_string(), t));
        }

        // every, some, findIndex — ConditionallyMutate, returns Primitive
        for name in &["every", "some", "findIndex"] {
            let t = method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::ConditionallyMutate,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            );
            props.push(((*name).to_string(), t));
        }

        // find — ConditionallyMutate, returns Poly
        props.push((
            "find".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: Type::Poly,
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::ConditionallyMutate,
                    ..FunctionSignature::default()
                },
                Type::Poly,
            ),
        ));

        add_object(&mut registry, BUILT_IN_ARRAY_ID, props);
    }

    // =========================================================================
    // Built-in Object shape (empty — instances are generic objects)
    // =========================================================================
    add_object(&mut registry, BUILT_IN_OBJECT_ID, Vec::new());

    // =========================================================================
    // Built-in Set shape — instance methods
    // =========================================================================
    {
        let r = &mut registry;
        let mut props = Vec::new();
        props.push(("size".to_string(), Type::Primitive));

        props.push((
            "add".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Capture],
                    return_type: set_type.clone(),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                set_type.clone(),
            ),
        ));

        props.push((
            "clear".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        props.push((
            "delete".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        props.push((
            "has".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Read,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        // forEach — ConditionallyMutate
        props.push((
            "forEach".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::ConditionallyMutate,
                    mutable_only_if_operands_are_mutable: true,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        // entries, keys, values — Capture, returns Poly
        for name in &["entries", "keys", "values"] {
            let t = method_prop(
                r,
                FunctionSignature {
                    return_type: Type::Poly,
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Capture,
                    ..FunctionSignature::default()
                },
                Type::Poly,
            );
            props.push(((*name).to_string(), t));
        }

        add_object(&mut registry, BUILT_IN_SET_ID, props);
    }

    // =========================================================================
    // Built-in Map shape — instance methods
    // =========================================================================
    {
        let r = &mut registry;
        let mut props = Vec::new();
        props.push(("size".to_string(), Type::Primitive));

        props.push((
            "clear".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        props.push((
            "delete".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        props.push((
            "get".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Poly,
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Capture,
                    ..FunctionSignature::default()
                },
                Type::Poly,
            ),
        ));

        props.push((
            "has".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Read,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        props.push((
            "set".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Capture, Effect::Capture],
                    return_type: map_type.clone(),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                map_type.clone(),
            ),
        ));

        // forEach — ConditionallyMutate
        props.push((
            "forEach".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::ConditionallyMutate,
                    mutable_only_if_operands_are_mutable: true,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        // entries, keys, values — Capture
        for name in &["entries", "keys", "values"] {
            let t = method_prop(
                r,
                FunctionSignature {
                    return_type: Type::Poly,
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Capture,
                    ..FunctionSignature::default()
                },
                Type::Poly,
            );
            props.push(((*name).to_string(), t));
        }

        add_object(&mut registry, BUILT_IN_MAP_ID, props);
    }

    // =========================================================================
    // Built-in WeakSet shape
    // =========================================================================
    {
        let r = &mut registry;
        let mut props = Vec::new();

        props.push((
            "add".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Capture],
                    return_type: weak_set_type.clone(),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                weak_set_type,
            ),
        ));

        props.push((
            "delete".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        props.push((
            "has".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Read,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        add_object(&mut registry, BUILT_IN_WEAK_SET_ID, props);
    }

    // =========================================================================
    // Built-in WeakMap shape
    // =========================================================================
    {
        let r = &mut registry;
        let mut props = Vec::new();

        props.push((
            "delete".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        props.push((
            "get".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Poly,
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Capture,
                    ..FunctionSignature::default()
                },
                Type::Poly,
            ),
        ));

        props.push((
            "has".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Read,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        ));

        props.push((
            "set".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Capture, Effect::Capture],
                    return_type: weak_map_type.clone(),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Store,
                    ..FunctionSignature::default()
                },
                weak_map_type,
            ),
        ));

        add_object(&mut registry, BUILT_IN_WEAK_MAP_ID, props);
    }

    // =========================================================================
    // Built-in MixedReadonly shape — readonly version of array-like methods
    // =========================================================================
    {
        let r = &mut registry;
        let mixed_readonly_type =
            Type::Object(ObjectType { shape_id: Some(BUILT_IN_MIXED_READONLY_ID.to_string()) });
        let mut props = Vec::new();

        // Wildcard — recursive MixedReadonly
        props.push(("*".to_string(), mixed_readonly_type.clone()));

        // toString, indexOf, includes, join — read-only
        for name in &["toString", "indexOf", "includes", "join"] {
            let t = method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::Read),
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Read,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            );
            props.push(((*name).to_string(), t));
        }

        // at — returns Frozen (unlike Array which returns Mutable)
        props.push((
            "at".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![Effect::Read],
                    return_type: Type::Poly,
                    return_value_kind: ValueKind::Frozen,
                    callee_effect: Effect::Read,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..FunctionSignature::default()
                },
                Type::Poly,
            ),
        ));

        // map, flatMap, filter — returns Array
        for name in &["map", "flatMap", "filter"] {
            let t = method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: array_type.clone(),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::ConditionallyMutate,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..FunctionSignature::default()
                },
                array_type.clone(),
            );
            props.push(((*name).to_string(), t));
        }

        // concat, slice — returns Array
        props.push((
            "concat".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::Capture),
                    return_type: array_type.clone(),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Capture,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..FunctionSignature::default()
                },
                array_type.clone(),
            ),
        ));

        props.push((
            "slice".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::Read),
                    return_type: array_type.clone(),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Capture,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..FunctionSignature::default()
                },
                array_type,
            ),
        ));

        // every, some, findIndex — ConditionallyMutate, returns Primitive
        for name in &["every", "some", "findIndex"] {
            let t = method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::ConditionallyMutate,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            );
            props.push(((*name).to_string(), t));
        }

        // find — returns Frozen (unlike Array)
        props.push((
            "find".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: Type::Poly,
                    return_value_kind: ValueKind::Frozen,
                    callee_effect: Effect::ConditionallyMutate,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..FunctionSignature::default()
                },
                Type::Poly,
            ),
        ));

        add_object(&mut registry, BUILT_IN_MIXED_READONLY_ID, props);
    }

    // =========================================================================
    // Hook return shapes — useState, useReducer, useTransition, etc.
    // =========================================================================

    // BuiltInUseStateId: { '0': Poly (state), '1': setState function }
    {
        let set_state_sig = FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        };
        // Register the SetState shape with the well-known id so is_set_state_type() matches
        add_function(
            &mut registry,
            Some(BUILT_IN_SET_STATE_ID),
            Vec::new(),
            set_state_sig,
        );
        let set_state_fn = Type::Function(FunctionType {
            shape_id: Some(BUILT_IN_SET_STATE_ID.to_string()),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        });

        add_object(
            &mut registry,
            BUILT_IN_USE_STATE_ID,
            vec![("0".to_string(), Type::Poly), ("1".to_string(), set_state_fn)],
        );
    }

    // BuiltInUseTransitionId: { '0': Primitive (isPending), '1': startTransition function }
    {
        add_function(
            &mut registry,
            Some(BUILT_IN_START_TRANSITION_ID),
            Vec::new(),
            FunctionSignature {
                return_type: Type::Primitive,
                return_value_kind: ValueKind::Primitive,
                callee_effect: Effect::Read,
                ..FunctionSignature::default()
            },
        );
        let start_transition_fn = Type::Function(FunctionType {
            shape_id: Some(BUILT_IN_START_TRANSITION_ID.to_string()),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        });

        add_object(
            &mut registry,
            BUILT_IN_USE_TRANSITION_ID,
            vec![("0".to_string(), Type::Primitive), ("1".to_string(), start_transition_fn)],
        );
    }

    // BuiltInUseOptimisticId: { '0': Poly (optimistic), '1': setOptimistic function }
    {
        add_function(
            &mut registry,
            Some(BUILT_IN_SET_OPTIMISTIC_ID),
            Vec::new(),
            FunctionSignature {
                rest_param: Some(Effect::Freeze),
                return_type: Type::Primitive,
                return_value_kind: ValueKind::Primitive,
                callee_effect: Effect::Read,
                ..FunctionSignature::default()
            },
        );
        let set_optimistic_fn = Type::Function(FunctionType {
            shape_id: Some(BUILT_IN_SET_OPTIMISTIC_ID.to_string()),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        });

        add_object(
            &mut registry,
            BUILT_IN_USE_OPTIMISTIC_ID,
            vec![("0".to_string(), Type::Poly), ("1".to_string(), set_optimistic_fn)],
        );
    }

    // BuiltInUseActionStateId: { '0': Poly (state), '1': dispatch function }
    {
        add_function(
            &mut registry,
            Some(BUILT_IN_SET_ACTION_STATE_ID),
            Vec::new(),
            FunctionSignature {
                rest_param: Some(Effect::Freeze),
                return_type: Type::Primitive,
                return_value_kind: ValueKind::Primitive,
                callee_effect: Effect::Read,
                ..FunctionSignature::default()
            },
        );
        let set_action_state_fn = Type::Function(FunctionType {
            shape_id: Some(BUILT_IN_SET_ACTION_STATE_ID.to_string()),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        });

        add_object(
            &mut registry,
            BUILT_IN_USE_ACTION_STATE_ID,
            vec![("0".to_string(), Type::Poly), ("1".to_string(), set_action_state_fn)],
        );
    }

    // BuiltInUseReducerId: { '0': Poly (state), '1': dispatch function }
    {
        add_function(
            &mut registry,
            Some(BUILT_IN_DISPATCH_ID),
            Vec::new(),
            FunctionSignature {
                rest_param: Some(Effect::Freeze),
                return_type: Type::Primitive,
                return_value_kind: ValueKind::Primitive,
                callee_effect: Effect::Read,
                ..FunctionSignature::default()
            },
        );
        let dispatch_fn = Type::Function(FunctionType {
            shape_id: Some(BUILT_IN_DISPATCH_ID.to_string()),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        });

        add_object(
            &mut registry,
            BUILT_IN_USE_REDUCER_ID,
            vec![("0".to_string(), Type::Poly), ("1".to_string(), dispatch_fn)],
        );
    }

    // BuiltInUseRefId: { 'current': BuiltInRefValue }
    {
        let ref_value_type =
            Type::Object(ObjectType { shape_id: Some(BUILT_IN_REF_VALUE_ID.to_string()) });
        add_object(
            &mut registry,
            BUILT_IN_USE_REF_ID,
            vec![("current".to_string(), ref_value_type)],
        );
    }

    // BuiltInRefValueId: { '*': BuiltInRefValue (recursive) }
    {
        let ref_value_type =
            Type::Object(ObjectType { shape_id: Some(BUILT_IN_REF_VALUE_ID.to_string()) });
        add_object(&mut registry, BUILT_IN_REF_VALUE_ID, vec![("*".to_string(), ref_value_type)]);
    }

    // BuiltInEffectEventId: function shape
    add_function(
        &mut registry,
        Some(BUILT_IN_EFFECT_EVENT_ID),
        Vec::new(),
        FunctionSignature {
            rest_param: Some(Effect::ConditionallyMutate),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::ConditionallyMutate,
            ..FunctionSignature::default()
        },
    );

    // ReanimatedSharedValueId: empty shape
    add_object(&mut registry, super::object_shape::REANIMATED_SHARED_VALUE_ID, Vec::new());

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

/// Helper to insert a hook global as a Function type.
fn insert_hook_global(
    globals: &mut GlobalRegistry,
    name: &str,
    shape_id: String,
    return_type: Type,
) {
    globals.insert(
        name.to_string(),
        Global::Typed(Type::Function(FunctionType {
            shape_id: Some(shape_id),
            return_type: Box::new(return_type),
            is_constructor: false,
        })),
    );
}

fn add_react_hook_globals(globals: &mut GlobalRegistry, shapes: &mut ShapeRegistry) {
    // --- useContext ---
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_CONTEXT_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::Read),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::Context),
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseContext),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useContext", id, Type::Poly);

    // --- useState ---
    let use_state_ret =
        Type::Object(ObjectType { shape_id: Some(BUILT_IN_USE_STATE_ID.to_string()) });
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_STATE_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: use_state_ret.clone(),
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::State),
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseState),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useState", id, use_state_ret);

    // --- useActionState ---
    let use_action_state_ret =
        Type::Object(ObjectType { shape_id: Some(BUILT_IN_USE_ACTION_STATE_ID.to_string()) });
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_ACTION_STATE_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: use_action_state_ret.clone(),
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::State),
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseActionState),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useActionState", id, use_action_state_ret);

    // --- useReducer ---
    let use_reducer_ret =
        Type::Object(ObjectType { shape_id: Some(BUILT_IN_USE_REDUCER_ID.to_string()) });
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_REDUCER_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: use_reducer_ret.clone(),
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::ReducerState),
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseReducer),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useReducer", id, use_reducer_ret);

    // --- useRef ---
    let use_ref_ret = Type::Object(ObjectType { shape_id: Some(BUILT_IN_USE_REF_ID.to_string()) });
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_REF_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::Capture),
            return_type: use_ref_ret.clone(),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseRef),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useRef", id, use_ref_ret);

    // --- useImperativeHandle ---
    let id = add_hook(
        shapes,
        None,
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseImperativeHandle),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useImperativeHandle", id, Type::Primitive);

    // --- useMemo ---
    let id = add_hook(
        shapes,
        None,
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseMemo),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useMemo", id, Type::Poly);

    // --- useCallback ---
    let id = add_hook(
        shapes,
        None,
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseCallback),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useCallback", id, Type::Poly);

    // --- useEffect ---
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_EFFECT_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseEffect),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useEffect", id, Type::Primitive);

    // --- useLayoutEffect ---
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseLayoutEffect),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useLayoutEffect", id, Type::Poly);

    // --- useInsertionEffect ---
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseInsertionEffect),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useInsertionEffect", id, Type::Poly);

    // --- useTransition ---
    let use_transition_ret =
        Type::Object(ObjectType { shape_id: Some(BUILT_IN_USE_TRANSITION_ID.to_string()) });
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_TRANSITION_HOOK_ID),
        FunctionSignature {
            return_type: use_transition_ret.clone(),
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseTransition),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useTransition", id, use_transition_ret);

    // --- useOptimistic ---
    let use_optimistic_ret =
        Type::Object(ObjectType { shape_id: Some(BUILT_IN_USE_OPTIMISTIC_ID.to_string()) });
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_OPTIMISTIC_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: use_optimistic_ret.clone(),
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::State),
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseOptimistic),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useOptimistic", id, use_optimistic_ret);

    // --- useEffectEvent ---
    let effect_event_ret = Type::Function(FunctionType {
        shape_id: Some(BUILT_IN_EFFECT_EVENT_ID.to_string()),
        return_type: Box::new(Type::Poly),
        is_constructor: false,
    });
    let id = add_hook(
        shapes,
        Some(BUILT_IN_USE_EFFECT_EVENT_ID),
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: effect_event_ret.clone(),
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::UseEffectEvent),
            ..FunctionSignature::default()
        },
    );
    insert_hook_global(globals, "useEffectEvent", id, effect_event_ret);

    // --- use (the `use()` API) ---
    let id = add_function(
        shapes,
        Some(BUILT_IN_USE_OPERATOR_ID),
        Vec::new(),
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    globals.insert(
        "use".to_string(),
        Global::Typed(Type::Function(FunctionType {
            shape_id: Some(id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        })),
    );
}

fn add_global_function_globals(globals: &mut GlobalRegistry, shapes: &mut ShapeRegistry) {
    // --- React global methods ---

    // React.createElement / React.cloneElement
    let jsx_fn_sig = FunctionSignature {
        rest_param: Some(Effect::Freeze),
        return_type: Type::Poly,
        return_value_kind: ValueKind::Frozen,
        callee_effect: Effect::Read,
        ..FunctionSignature::default()
    };

    for name in &["createElement", "cloneElement", "_jsx", "_jsxs", "_jsxDEV"] {
        let id = add_function(shapes, None, Vec::new(), jsx_fn_sig.clone());
        globals.insert(
            (*name).to_string(),
            Global::Typed(Type::Function(FunctionType {
                shape_id: Some(id),
                return_type: Box::new(Type::Poly),
                is_constructor: false,
            })),
        );
    }

    // React.createRef — same shape as useRef
    let create_ref_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            rest_param: Some(Effect::Capture),
            return_type: Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_USE_REF_ID.to_string()),
            }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    globals.insert(
        "createRef".to_string(),
        Global::Typed(Type::Function(FunctionType {
            shape_id: Some(create_ref_id),
            return_type: Box::new(Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_USE_REF_ID.to_string()),
            })),
            is_constructor: false,
        })),
    );

    // --- Object methods (as a shaped global) ---
    let object_keys_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Read],
            return_type: Type::Object(ObjectType { shape_id: Some(BUILT_IN_ARRAY_ID.to_string()) }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    let object_values_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Capture],
            return_type: Type::Object(ObjectType { shape_id: Some(BUILT_IN_ARRAY_ID.to_string()) }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    let object_entries_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Capture],
            return_type: Type::Object(ObjectType { shape_id: Some(BUILT_IN_ARRAY_ID.to_string()) }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    let object_from_entries_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::ConditionallyMutate],
            return_type: Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_OBJECT_ID.to_string()),
            }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );

    let object_shape_id = add_object(
        shapes,
        "Global$Object",
        vec![
            (
                "keys".to_string(),
                Type::Function(FunctionType {
                    shape_id: Some(object_keys_id),
                    return_type: Box::new(Type::Object(ObjectType {
                        shape_id: Some(BUILT_IN_ARRAY_ID.to_string()),
                    })),
                    is_constructor: false,
                }),
            ),
            (
                "values".to_string(),
                Type::Function(FunctionType {
                    shape_id: Some(object_values_id),
                    return_type: Box::new(Type::Object(ObjectType {
                        shape_id: Some(BUILT_IN_ARRAY_ID.to_string()),
                    })),
                    is_constructor: false,
                }),
            ),
            (
                "entries".to_string(),
                Type::Function(FunctionType {
                    shape_id: Some(object_entries_id),
                    return_type: Box::new(Type::Object(ObjectType {
                        shape_id: Some(BUILT_IN_ARRAY_ID.to_string()),
                    })),
                    is_constructor: false,
                }),
            ),
            (
                "fromEntries".to_string(),
                Type::Function(FunctionType {
                    shape_id: Some(object_from_entries_id),
                    return_type: Box::new(Type::Object(ObjectType {
                        shape_id: Some(BUILT_IN_OBJECT_ID.to_string()),
                    })),
                    is_constructor: false,
                }),
            ),
        ],
    );
    globals.insert(
        "Object".to_string(),
        Global::Typed(Type::Object(ObjectType { shape_id: Some(object_shape_id) })),
    );

    // --- Array static methods ---
    let array_is_array_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Read],
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    let array_from_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![
                Effect::ConditionallyMutateIterator,
                Effect::ConditionallyMutate,
                Effect::ConditionallyMutate,
            ],
            rest_param: Some(Effect::Read),
            return_type: Type::Object(ObjectType { shape_id: Some(BUILT_IN_ARRAY_ID.to_string()) }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    let array_of_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            rest_param: Some(Effect::Read),
            return_type: Type::Object(ObjectType { shape_id: Some(BUILT_IN_ARRAY_ID.to_string()) }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );

    let array_shape_id = add_object(
        shapes,
        "Global$Array",
        vec![
            (
                "isArray".to_string(),
                Type::Function(FunctionType {
                    shape_id: Some(array_is_array_id),
                    return_type: Box::new(Type::Primitive),
                    is_constructor: false,
                }),
            ),
            (
                "from".to_string(),
                Type::Function(FunctionType {
                    shape_id: Some(array_from_id),
                    return_type: Box::new(Type::Object(ObjectType {
                        shape_id: Some(BUILT_IN_ARRAY_ID.to_string()),
                    })),
                    is_constructor: false,
                }),
            ),
            (
                "of".to_string(),
                Type::Function(FunctionType {
                    shape_id: Some(array_of_id),
                    return_type: Box::new(Type::Object(ObjectType {
                        shape_id: Some(BUILT_IN_ARRAY_ID.to_string()),
                    })),
                    is_constructor: false,
                }),
            ),
        ],
    );
    globals.insert(
        "Array".to_string(),
        Global::Typed(Type::Object(ObjectType { shape_id: Some(array_shape_id) })),
    );

    // --- Console methods ---
    let console_method_sig = FunctionSignature {
        rest_param: Some(Effect::Read),
        return_type: Type::Primitive,
        return_value_kind: ValueKind::Primitive,
        callee_effect: Effect::Read,
        ..FunctionSignature::default()
    };
    let mut console_props = Vec::new();
    for method in &["log", "error", "warn", "info", "table", "trace"] {
        let method_id = add_function(shapes, None, Vec::new(), console_method_sig.clone());
        console_props.push((
            (*method).to_string(),
            Type::Function(FunctionType {
                shape_id: Some(method_id),
                return_type: Box::new(Type::Primitive),
                is_constructor: false,
            }),
        ));
    }
    let console_shape_id = add_object(shapes, "Global$console", console_props);
    globals.insert(
        "console".to_string(),
        Global::Typed(Type::Object(ObjectType { shape_id: Some(console_shape_id) })),
    );

    // --- Math methods ---
    let math_pure_sig = FunctionSignature {
        rest_param: Some(Effect::Read),
        return_type: Type::Primitive,
        return_value_kind: ValueKind::Primitive,
        callee_effect: Effect::Read,
        ..FunctionSignature::default()
    };
    let math_random_sig = FunctionSignature {
        rest_param: Some(Effect::Read),
        return_type: Type::Poly,
        return_value_kind: ValueKind::Mutable,
        callee_effect: Effect::Read,
        impure: true,
        canonical_name: Some("Math.random".to_string()),
        ..FunctionSignature::default()
    };

    let mut math_props = vec![("PI".to_string(), Type::Primitive)];
    for method in &["max", "min", "trunc", "ceil", "floor", "pow", "round", "abs"] {
        let method_id = add_function(shapes, None, Vec::new(), math_pure_sig.clone());
        math_props.push((
            (*method).to_string(),
            Type::Function(FunctionType {
                shape_id: Some(method_id),
                return_type: Box::new(Type::Primitive),
                is_constructor: false,
            }),
        ));
    }
    let random_id = add_function(shapes, None, Vec::new(), math_random_sig);
    math_props.push((
        "random".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(random_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    ));
    let math_shape_id = add_object(shapes, "Global$Math", math_props);
    globals.insert(
        "Math".to_string(),
        Global::Typed(Type::Object(ObjectType { shape_id: Some(math_shape_id) })),
    );

    // --- Constructor globals (Map, Set, WeakMap, WeakSet) ---
    let map_ctor_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::ConditionallyMutateIterator],
            return_type: Type::Object(ObjectType { shape_id: Some(BUILT_IN_MAP_ID.to_string()) }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    globals.insert(
        "Map".to_string(),
        Global::Typed(Type::Function(FunctionType {
            shape_id: Some(map_ctor_id),
            return_type: Box::new(Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_MAP_ID.to_string()),
            })),
            is_constructor: true,
        })),
    );

    let set_ctor_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::ConditionallyMutateIterator],
            return_type: Type::Object(ObjectType { shape_id: Some(BUILT_IN_SET_ID.to_string()) }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    globals.insert(
        "Set".to_string(),
        Global::Typed(Type::Function(FunctionType {
            shape_id: Some(set_ctor_id),
            return_type: Box::new(Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_SET_ID.to_string()),
            })),
            is_constructor: true,
        })),
    );

    let weak_map_ctor_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::ConditionallyMutateIterator],
            return_type: Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_WEAK_MAP_ID.to_string()),
            }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    globals.insert(
        "WeakMap".to_string(),
        Global::Typed(Type::Function(FunctionType {
            shape_id: Some(weak_map_ctor_id),
            return_type: Box::new(Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_WEAK_MAP_ID.to_string()),
            })),
            is_constructor: true,
        })),
    );

    let weak_set_ctor_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::ConditionallyMutateIterator],
            return_type: Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_WEAK_SET_ID.to_string()),
            }),
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            ..FunctionSignature::default()
        },
    );
    globals.insert(
        "WeakSet".to_string(),
        Global::Typed(Type::Function(FunctionType {
            shape_id: Some(weak_set_ctor_id),
            return_type: Box::new(Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_WEAK_SET_ID.to_string()),
            })),
            is_constructor: true,
        })),
    );

    // --- Common pure global functions ---
    for name in &[
        "parseInt",
        "parseFloat",
        "isNaN",
        "isFinite",
        "encodeURIComponent",
        "decodeURIComponent",
        "encodeURI",
        "decodeURI",
        "btoa",
        "atob",
    ] {
        globals.insert(
            (*name).to_string(),
            Global::Typed(Type::Function(FunctionType {
                shape_id: None,
                return_type: Box::new(Type::Primitive),
                is_constructor: false,
            })),
        );
    }

    // --- Primitive globals ---
    globals.insert("undefined".to_string(), Global::Typed(Type::Primitive));
    globals.insert("NaN".to_string(), Global::Typed(Type::Primitive));
    globals.insert("Infinity".to_string(), Global::Typed(Type::Primitive));
    globals.insert("null".to_string(), Global::Typed(Type::Primitive));
}
