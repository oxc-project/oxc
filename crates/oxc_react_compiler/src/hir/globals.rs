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
        BUILT_IN_ARRAY_ID, BUILT_IN_DEFAULT_MUTATING_HOOK_ID, BUILT_IN_DEFAULT_NONMUTATING_HOOK_ID,
        BUILT_IN_EFFECT_EVENT_ID, BUILT_IN_MAP_ID, BUILT_IN_MIXED_READONLY_ID, BUILT_IN_OBJECT_ID,
        BUILT_IN_PROPS_ID, BUILT_IN_SET_ID, BUILT_IN_USE_ACTION_STATE_HOOK_ID,
        BUILT_IN_USE_ACTION_STATE_ID, BUILT_IN_USE_CONTEXT_HOOK_ID, BUILT_IN_USE_EFFECT_EVENT_ID,
        BUILT_IN_USE_EFFECT_HOOK_ID, BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID,
        BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID, BUILT_IN_USE_OPERATOR_ID,
        BUILT_IN_USE_OPTIMISTIC_HOOK_ID, BUILT_IN_USE_OPTIMISTIC_ID, BUILT_IN_USE_REDUCER_HOOK_ID,
        BUILT_IN_USE_REDUCER_ID, BUILT_IN_USE_REF_HOOK_ID, BUILT_IN_USE_REF_ID,
        BUILT_IN_USE_STATE_HOOK_ID, BUILT_IN_USE_STATE_ID, BUILT_IN_USE_TRANSITION_HOOK_ID,
        BUILT_IN_USE_TRANSITION_ID, BUILT_IN_WEAK_MAP_ID, BUILT_IN_WEAK_SET_ID, FunctionSignature,
        HookKind, ShapeRegistry, add_function, add_hook, add_object,
        parse_aliasing_signature_config,
    },
    type_schema::{AliasingEffectConfig, AliasingSignatureConfig},
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
    // Note: String, Number, Boolean, parseInt, parseFloat, etc. are also typed globals
    // that return Primitive values, and are added in add_global_function_globals().
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
        // noAlias=true: the callback args don't escape into the return value
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

        // every, some, findIndex — ConditionallyMutate, returns Primitive
        // noAlias=true: arguments do not escape into the return value
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

        // find — ConditionallyMutate, returns Poly
        // noAlias=true: arguments do not escape into the return value
        props.push((
            "find".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: Type::Poly,
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::ConditionallyMutate,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..FunctionSignature::default()
                },
                Type::Poly,
            ),
        ));

        add_object(&mut registry, BUILT_IN_ARRAY_ID, props);
    }

    // =========================================================================
    // Built-in Object shape — prototype methods
    // =========================================================================
    {
        let r = &mut registry;
        let props = vec![(
            "toString".to_string(),
            method_prop(
                r,
                FunctionSignature {
                    positional_params: vec![],
                    return_type: Type::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    callee_effect: Effect::Read,
                    ..FunctionSignature::default()
                },
                Type::Primitive,
            ),
        )];

        add_object(r, BUILT_IN_OBJECT_ID, props);
    }

    // =========================================================================
    // Built-in Set shape — instance methods
    // =========================================================================
    {
        let r = &mut registry;
        let mut props = vec![("size".to_string(), Type::Primitive)];

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
                set_type,
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
        let mut props = vec![("size".to_string(), Type::Primitive)];

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
                map_type,
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
        let mut props = vec![(
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
        )];

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
        let mut props = vec![(
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
        )];

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
        props.push(("*".to_string(), mixed_readonly_type));

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
        add_function(&mut registry, Some(BUILT_IN_SET_STATE_ID), Vec::new(), set_state_sig);
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

    // BuiltInPropsId: { 'ref': Object(BuiltInUseRefId) }
    // Port of TS ObjectShape.ts:
    //   addObject(BUILTIN_SHAPES, BuiltInPropsId, [["ref", { kind: "Object", shapeId: BuiltInUseRefId }]]);
    // If the `ref` prop exists, it has the ref type.
    {
        let use_ref_type =
            Type::Object(ObjectType { shape_id: Some(BUILT_IN_USE_REF_ID.to_string()) });
        add_object(&mut registry, BUILT_IN_PROPS_ID, vec![("ref".to_string(), use_ref_type)]);
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

    // DefaultNonmutatingHook: the default shape for custom hooks detected by name.
    // Port of `DefaultNonmutatingHook` from ObjectShape.ts.
    // This shape has `hookKind: Custom` so that `get_hook_kind_for_type` can identify
    // hook calls via the type system (critical for FlattenScopesWithHooksOrUse).
    // Used when enableAssumeHooksFollowRulesOfReact is true (the default).
    add_hook(
        &mut registry,
        Some(BUILT_IN_DEFAULT_NONMUTATING_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::HookReturn),
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::Custom),
            ..FunctionSignature::default()
        },
    );

    // DefaultMutatingHook: the default shape for custom hooks when
    // enableAssumeHooksFollowRulesOfReact is false.
    // Port of `DefaultMutatingHook` from ObjectShape.ts.
    // Arguments are conditionally mutated and the return value is mutable.
    add_hook(
        &mut registry,
        Some(BUILT_IN_DEFAULT_MUTATING_HOOK_ID),
        FunctionSignature {
            rest_param: Some(Effect::ConditionallyMutate),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::Custom),
            ..FunctionSignature::default()
        },
    );

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

    // React namespace object — mirrors the TS `React` global in Globals.ts.
    // The React object contains all REACT_APIS hooks plus createElement,
    // cloneElement, and createRef as properties.
    add_react_namespace_global(&mut globals, shapes);

    globals
}

/// Build the reanimated module type.
///
/// Port of `getReanimatedModuleType()` from `HIR/Globals.ts`.
///
/// Creates an object type with properties for each reanimated hook and function,
/// each with custom type signatures that the compiler uses for correct
/// memoization behavior.
pub fn get_reanimated_module_type(shapes: &mut ShapeRegistry) -> Type {
    let mut reanimated_props: Vec<(String, Type)> = Vec::new();

    // Hooks that freeze args and return frozen value
    let frozen_hooks = [
        "useFrameCallback",
        "useAnimatedStyle",
        "useAnimatedProps",
        "useAnimatedScrollHandler",
        "useAnimatedReaction",
        "useWorkletCallback",
    ];
    for hook in frozen_hooks {
        let shape_id = add_hook(
            shapes,
            None,
            FunctionSignature {
                rest_param: Some(Effect::Freeze),
                return_type: Type::Poly,
                return_value_kind: ValueKind::Frozen,
                callee_effect: Effect::Read,
                hook_kind: Some(HookKind::Custom),
                no_alias: true,
                ..FunctionSignature::default()
            },
        );
        reanimated_props.push((
            hook.to_string(),
            Type::Function(FunctionType {
                shape_id: Some(shape_id),
                return_type: Box::new(Type::Poly),
                is_constructor: false,
            }),
        ));
    }

    // Hooks that return a mutable value (ideally modelled as refs, but this works for now)
    let mutable_hooks = ["useSharedValue", "useDerivedValue"];
    for hook in mutable_hooks {
        let shape_id = add_hook(
            shapes,
            None,
            FunctionSignature {
                rest_param: Some(Effect::Freeze),
                return_type: Type::Object(ObjectType {
                    shape_id: Some(super::object_shape::REANIMATED_SHARED_VALUE_ID.to_string()),
                }),
                return_value_kind: ValueKind::Mutable,
                callee_effect: Effect::Read,
                hook_kind: Some(HookKind::Custom),
                no_alias: true,
                ..FunctionSignature::default()
            },
        );
        reanimated_props.push((
            hook.to_string(),
            Type::Function(FunctionType {
                shape_id: Some(shape_id),
                return_type: Box::new(Type::Object(ObjectType {
                    shape_id: Some(super::object_shape::REANIMATED_SHARED_VALUE_ID.to_string()),
                })),
                is_constructor: false,
            }),
        ));
    }

    // Functions that return mutable value
    let funcs = [
        "withTiming",
        "withSpring",
        "createAnimatedPropAdapter",
        "withDecay",
        "withRepeat",
        "runOnUI",
        "executeOnUIRuntimeSync",
    ];
    for func_name in funcs {
        let shape_id = add_function(
            shapes,
            None,
            Vec::new(),
            FunctionSignature {
                rest_param: Some(Effect::Read),
                return_type: Type::Poly,
                return_value_kind: ValueKind::Mutable,
                callee_effect: Effect::Read,
                no_alias: true,
                ..FunctionSignature::default()
            },
        );
        reanimated_props.push((
            func_name.to_string(),
            Type::Function(FunctionType {
                shape_id: Some(shape_id),
                return_type: Box::new(Type::Poly),
                is_constructor: false,
            }),
        ));
    }

    let module_shape_id = add_object(shapes, "ReanimatedModule", reanimated_props);
    Type::Object(ObjectType { shape_id: Some(module_shape_id) })
}

/// Build the shared-runtime module type for test fixtures.
///
/// Port of the `sharedRuntimeTypeProvider` from
/// `packages/snap/src/sprout/shared-runtime-type-provider.ts`.
///
/// The TS test harness registers a module type provider that tells the compiler
/// about hooks and typed functions exported from the `shared-runtime` test module.
/// This function creates the equivalent object type with known properties.
pub fn get_shared_runtime_module_type(shapes: &mut ShapeRegistry) -> Type {
    let mut props: Vec<(String, Type)> = Vec::new();

    // --- default: function, returns Primitive ---
    let default_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![],
            rest_param: Some(Effect::Read),
            callee_effect: Effect::Read,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..FunctionSignature::default()
        },
    );
    props.push((
        "default".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(default_id),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        }),
    ));

    // --- graphql: function, returns Primitive ---
    let graphql_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![],
            rest_param: Some(Effect::Read),
            callee_effect: Effect::Read,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..FunctionSignature::default()
        },
    );
    props.push((
        "graphql".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(graphql_id),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        }),
    ));

    // --- typedArrayPush: function(Store, Capture), returns Primitive ---
    let typed_array_push_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Store, Effect::Capture],
            rest_param: Some(Effect::Capture),
            callee_effect: Effect::Read,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..FunctionSignature::default()
        },
    );
    props.push((
        "typedArrayPush".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(typed_array_push_id),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        }),
    ));

    // --- typedLog: function, returns Primitive ---
    let typed_log_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![],
            rest_param: Some(Effect::Read),
            callee_effect: Effect::Read,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..FunctionSignature::default()
        },
    );
    props.push((
        "typedLog".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(typed_log_id),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        }),
    ));

    // --- useFreeze: hook, returns Any ---
    let use_freeze_id = add_hook(
        shapes,
        None,
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::Custom),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "useFreeze".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(use_freeze_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    ));

    // --- useFragment: hook, returns MixedReadonly, noAlias=true ---
    let use_fragment_id = add_hook(
        shapes,
        None,
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_MIXED_READONLY_ID.to_string()),
            }),
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::Custom),
            no_alias: true,
            ..FunctionSignature::default()
        },
    );
    props.push((
        "useFragment".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(use_fragment_id),
            return_type: Box::new(Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_MIXED_READONLY_ID.to_string()),
            })),
            is_constructor: false,
        }),
    ));

    // --- useNoAlias: hook, returns Any, returnValueKind=Mutable, noAlias=true ---
    let use_no_alias_id = add_hook(
        shapes,
        None,
        FunctionSignature {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::Custom),
            no_alias: true,
            ..FunctionSignature::default()
        },
    );
    props.push((
        "useNoAlias".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(use_no_alias_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    ));

    // --- typedIdentity: function(Read) -> Any, aliasing: Assign ---
    let typed_identity_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Read,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            aliasing: Some(parse_aliasing_signature_config(&AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@value".to_string()],
                rest: None,
                returns: "@return".to_string(),
                temporaries: vec![],
                effects: vec![AliasingEffectConfig::Assign {
                    from: "@value".to_string(),
                    into: "@return".to_string(),
                }],
            })),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "typedIdentity".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(typed_identity_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    ));

    // --- typedAssign: function(Read) -> Any, aliasing: Assign (same as typedIdentity) ---
    let typed_assign_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Read,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            aliasing: Some(parse_aliasing_signature_config(&AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@value".to_string()],
                rest: None,
                returns: "@return".to_string(),
                temporaries: vec![],
                effects: vec![AliasingEffectConfig::Assign {
                    from: "@value".to_string(),
                    into: "@return".to_string(),
                }],
            })),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "typedAssign".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(typed_assign_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    ));

    // --- typedAlias: function(Read) -> Any, aliasing: Create + Alias ---
    let typed_alias_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Read,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            aliasing: Some(parse_aliasing_signature_config(&AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@value".to_string()],
                rest: None,
                returns: "@return".to_string(),
                temporaries: vec![],
                effects: vec![
                    AliasingEffectConfig::Create {
                        into: "@return".to_string(),
                        value: ValueKind::Mutable,
                        reason: ValueReason::KnownReturnSignature,
                    },
                    AliasingEffectConfig::Alias {
                        from: "@value".to_string(),
                        into: "@return".to_string(),
                    },
                ],
            })),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "typedAlias".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(typed_alias_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    ));

    // --- typedCapture: function(Read) -> Array, aliasing: Create + Capture ---
    let typed_capture_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Read,
            return_type: Type::Object(ObjectType { shape_id: Some(BUILT_IN_ARRAY_ID.to_string()) }),
            return_value_kind: ValueKind::Mutable,
            aliasing: Some(parse_aliasing_signature_config(&AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@value".to_string()],
                rest: None,
                returns: "@return".to_string(),
                temporaries: vec![],
                effects: vec![
                    AliasingEffectConfig::Create {
                        into: "@return".to_string(),
                        value: ValueKind::Mutable,
                        reason: ValueReason::KnownReturnSignature,
                    },
                    AliasingEffectConfig::Capture {
                        from: "@value".to_string(),
                        into: "@return".to_string(),
                    },
                ],
            })),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "typedCapture".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(typed_capture_id),
            return_type: Box::new(Type::Object(ObjectType {
                shape_id: Some(BUILT_IN_ARRAY_ID.to_string()),
            })),
            is_constructor: false,
        }),
    ));

    // --- typedCreateFrom: function(Read) -> Any, aliasing: CreateFrom ---
    let typed_create_from_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Read,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            aliasing: Some(parse_aliasing_signature_config(&AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@value".to_string()],
                rest: None,
                returns: "@return".to_string(),
                temporaries: vec![],
                effects: vec![AliasingEffectConfig::CreateFrom {
                    from: "@value".to_string(),
                    into: "@return".to_string(),
                }],
            })),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "typedCreateFrom".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(typed_create_from_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    ));

    // --- typedMutate: function(Read, Capture) -> Primitive, aliasing: Create + Mutate + Capture ---
    let typed_mutate_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![Effect::Read, Effect::Capture],
            callee_effect: Effect::Store,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            aliasing: Some(parse_aliasing_signature_config(&AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@object".to_string(), "@value".to_string()],
                rest: None,
                returns: "@return".to_string(),
                temporaries: vec![],
                effects: vec![
                    AliasingEffectConfig::Create {
                        into: "@return".to_string(),
                        value: ValueKind::Primitive,
                        reason: ValueReason::KnownReturnSignature,
                    },
                    AliasingEffectConfig::Mutate { value: "@object".to_string() },
                    AliasingEffectConfig::Capture {
                        from: "@value".to_string(),
                        into: "@object".to_string(),
                    },
                ],
            })),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "typedMutate".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(typed_mutate_id),
            return_type: Box::new(Type::Primitive),
            is_constructor: false,
        }),
    ));

    let module_shape_id = add_object(shapes, "SharedRuntimeModule", props);
    Type::Object(ObjectType { shape_id: Some(module_shape_id) })
}

/// Create type definitions for the `ReactCompilerKnownIncompatibleTest` test module.
///
/// Port of the `ReactCompilerKnownIncompatibleTest` case in
/// `packages/snap/src/sprout/shared-runtime-type-provider.ts`.
///
/// This fake module is used for testing validation of known incompatible API detection.
pub fn get_known_incompatible_test_module_type(shapes: &mut ShapeRegistry) -> Type {
    let mut props: Vec<(String, Type)> = Vec::new();

    // --- useKnownIncompatible: hook, knownIncompatible ---
    let use_known_incompatible_id = add_hook(
        shapes,
        None,
        FunctionSignature {
            positional_params: vec![],
            rest_param: Some(Effect::Read),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::Custom),
            known_incompatible: Some(
                "useKnownIncompatible is known to be incompatible".to_string(),
            ),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "useKnownIncompatible".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(use_known_incompatible_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    ));

    // --- useKnownIncompatibleIndirect: hook, returns object with incompatible function ---
    // The return type has a property `incompatible` which is a function with knownIncompatible.
    let incompatible_fn_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![],
            rest_param: Some(Effect::Read),
            callee_effect: Effect::Read,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            known_incompatible: Some(
                "useKnownIncompatibleIndirect returns an incompatible() function that is known incompatible"
                    .to_string(),
            ),
            ..FunctionSignature::default()
        },
    );
    let return_obj_props = vec![(
        "incompatible".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(incompatible_fn_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    )];
    let return_obj_id = add_object(shapes, "KnownIncompatibleIndirectReturn", return_obj_props);
    let use_known_incompatible_indirect_id = add_hook(
        shapes,
        None,
        FunctionSignature {
            positional_params: vec![],
            rest_param: Some(Effect::Read),
            return_type: Type::Object(ObjectType { shape_id: Some(return_obj_id.clone()) }),
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::Custom),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "useKnownIncompatibleIndirect".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(use_known_incompatible_indirect_id),
            return_type: Box::new(Type::Object(ObjectType { shape_id: Some(return_obj_id) })),
            is_constructor: false,
        }),
    ));

    // --- knownIncompatible: function, knownIncompatible ---
    let known_incompatible_fn_id = add_function(
        shapes,
        None,
        Vec::new(),
        FunctionSignature {
            positional_params: vec![],
            rest_param: Some(Effect::Read),
            callee_effect: Effect::Read,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            known_incompatible: Some(
                "useKnownIncompatible is known to be incompatible".to_string(),
            ),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "knownIncompatible".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(known_incompatible_fn_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    ));

    let module_shape_id = add_object(shapes, "ReactCompilerKnownIncompatibleTestModule", props);
    Type::Object(ObjectType { shape_id: Some(module_shape_id) })
}

/// Build the `ReactCompilerTest` module type for testing type-provider validation.
///
/// This module contains deliberately mismatched types:
/// - `useHookNotTypedAsHook`: hook-like name but typed as a plain type (not a hook)
/// - `notAhookTypedAsHook`: non-hook name but typed as a hook
///
/// These are used to test that the compiler validates hook name / hook type consistency.
pub fn get_react_compiler_test_module_type(shapes: &mut ShapeRegistry) -> Type {
    let mut props: Vec<(String, Type)> = Vec::new();

    // useHookNotTypedAsHook: { kind: "type", name: "Any" } → Poly (not a hook)
    props.push(("useHookNotTypedAsHook".to_string(), Type::Poly));

    // notAhookTypedAsHook: { kind: "hook", returnType: { kind: "type", name: "Any" } }
    let not_a_hook_typed_as_hook_id = add_hook(
        shapes,
        None,
        FunctionSignature {
            positional_params: vec![],
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            callee_effect: Effect::Read,
            hook_kind: Some(HookKind::Custom),
            ..FunctionSignature::default()
        },
    );
    props.push((
        "notAhookTypedAsHook".to_string(),
        Type::Function(FunctionType {
            shape_id: Some(not_a_hook_typed_as_hook_id),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
    ));

    let module_shape_id = add_object(shapes, "ReactCompilerTestModule", props);
    Type::Object(ObjectType { shape_id: Some(module_shape_id) })
}

/// Build the `useDefaultExportNotTypedAsHook` module type for testing type-provider validation.
///
/// This module has a default export typed as a plain type (not a hook), but the module name
/// starts with `use` which suggests it should be a hook. This tests the module-level
/// hook name / hook type consistency validation.
pub fn get_use_default_export_not_typed_as_hook_module_type(shapes: &mut ShapeRegistry) -> Type {
    // default: { kind: "type", name: "Any" } → Poly (not a hook)
    let props: Vec<(String, Type)> = vec![("default".to_string(), Type::Poly)];

    let module_shape_id = add_object(shapes, "UseDefaultExportNotTypedAsHookModule", props);
    Type::Object(ObjectType { shape_id: Some(module_shape_id) })
}

/// Names that are part of REACT_APIS in the TS version.
/// These are registered as both top-level globals and as properties of the React namespace object.
const REACT_API_NAMES: &[&str] = &[
    "useContext",
    "useState",
    "useActionState",
    "useReducer",
    "useRef",
    "useImperativeHandle",
    "useMemo",
    "useCallback",
    "useEffect",
    "useLayoutEffect",
    "useInsertionEffect",
    "useTransition",
    "useOptimistic",
    "use",
    "useEffectEvent",
];

/// Additional React static methods that are properties of the React namespace object
/// but NOT part of REACT_APIS (they aren't hooks).
const REACT_STATIC_METHOD_NAMES: &[&str] = &["createElement", "cloneElement", "createRef"];

/// Build the `React` global namespace object.
///
/// Port of the `TYPED_GLOBALS.push(['React', addObject(...)])` block in `Globals.ts`.
/// The React object's shape includes all REACT_APIS hooks plus createElement,
/// cloneElement, and createRef, so that `React.useState(...)` etc. resolve correctly.
fn add_react_namespace_global(globals: &mut GlobalRegistry, shapes: &mut ShapeRegistry) {
    let mut react_props: Vec<(String, Type)> = Vec::new();

    // Gather all REACT_APIS types from the already-registered globals.
    for name in REACT_API_NAMES {
        if let Some(Global::Typed(type_)) = globals.get(*name) {
            react_props.push(((*name).to_string(), type_.clone()));
        }
    }

    // Gather React static methods (createElement, cloneElement, createRef).
    for name in REACT_STATIC_METHOD_NAMES {
        if let Some(Global::Typed(type_)) = globals.get(*name) {
            react_props.push(((*name).to_string(), type_.clone()));
        }
    }

    let react_shape_id = add_object(shapes, "Global$React", react_props);
    globals.insert(
        "React".to_string(),
        Global::Typed(Type::Object(ObjectType { shape_id: Some(react_shape_id) })),
    );
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
            aliasing: Some(parse_aliasing_signature_config(&AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec![],
                rest: Some("@rest".to_string()),
                returns: "@returns".to_string(),
                temporaries: vec!["@effect".to_string()],
                effects: vec![
                    // Freezes the function and deps
                    AliasingEffectConfig::Freeze {
                        value: "@rest".to_string(),
                        reason: ValueReason::Effect,
                    },
                    // Internally creates an effect object that captures the function and deps
                    AliasingEffectConfig::Create {
                        into: "@effect".to_string(),
                        value: ValueKind::Frozen,
                        reason: ValueReason::KnownReturnSignature,
                    },
                    // The effect stores the function and dependencies
                    AliasingEffectConfig::Capture {
                        from: "@rest".to_string(),
                        into: "@effect".to_string(),
                    },
                    // Returns undefined
                    AliasingEffectConfig::Create {
                        into: "@returns".to_string(),
                        value: ValueKind::Primitive,
                        reason: ValueReason::KnownReturnSignature,
                    },
                ],
            })),
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
            aliasing: Some(parse_aliasing_signature_config(&AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@object".to_string()],
                rest: None,
                returns: "@returns".to_string(),
                temporaries: vec![],
                effects: vec![
                    AliasingEffectConfig::Create {
                        into: "@returns".to_string(),
                        reason: ValueReason::KnownReturnSignature,
                        value: ValueKind::Mutable,
                    },
                    // Only keys are captured, and keys are immutable
                    AliasingEffectConfig::ImmutableCapture {
                        from: "@object".to_string(),
                        into: "@returns".to_string(),
                    },
                ],
            })),
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
            aliasing: Some(parse_aliasing_signature_config(&AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@object".to_string()],
                rest: None,
                returns: "@returns".to_string(),
                temporaries: vec![],
                effects: vec![
                    AliasingEffectConfig::Create {
                        into: "@returns".to_string(),
                        reason: ValueReason::KnownReturnSignature,
                        value: ValueKind::Mutable,
                    },
                    // Object values are captured into the return
                    AliasingEffectConfig::Capture {
                        from: "@object".to_string(),
                        into: "@returns".to_string(),
                    },
                ],
            })),
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
            aliasing: Some(parse_aliasing_signature_config(&AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@object".to_string()],
                rest: None,
                returns: "@returns".to_string(),
                temporaries: vec![],
                effects: vec![
                    AliasingEffectConfig::Create {
                        into: "@returns".to_string(),
                        reason: ValueReason::KnownReturnSignature,
                        value: ValueKind::Mutable,
                    },
                    // Object values are captured into the return
                    AliasingEffectConfig::Capture {
                        from: "@object".to_string(),
                        into: "@returns".to_string(),
                    },
                ],
            })),
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

    // --- Common pure global functions that return Primitive ---
    // These match the TypeScript compiler's Globals.ts where String, Number, Boolean,
    // parseInt, parseFloat, isNaN, isFinite, encodeURI*, decodeURI* are all typed as
    // returning Primitive. This is important because mayAllocate() returns false for
    // CallExpressions whose lvalue type is Primitive, preventing them from getting
    // spurious reactive scopes.
    let primitive_fn_sig = FunctionSignature {
        rest_param: Some(Effect::Read),
        return_type: Type::Primitive,
        return_value_kind: ValueKind::Primitive,
        callee_effect: Effect::Read,
        ..FunctionSignature::default()
    };
    for name in &[
        "String",
        "Number",
        "Boolean",
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
        let fn_id = add_function(shapes, None, Vec::new(), primitive_fn_sig.clone());
        globals.insert(
            (*name).to_string(),
            Global::Typed(Type::Function(FunctionType {
                shape_id: Some(fn_id),
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

    // --- Recursive global types: globalThis and global ---
    //
    // Port of `Globals.ts` lines 933-941: both `globalThis` and `global` are
    // registered as objects whose properties mirror the typed globals. This allows
    // patterns like `global.console.log(x)` or `globalThis.Array.from(...)` to
    // resolve types correctly through property access chains.
    let typed_global_props: Vec<(String, Type)> = globals
        .iter()
        .filter_map(|(name, global)| match global {
            Global::Typed(t) => Some((name.clone(), t.clone())),
            Global::Untyped => None,
        })
        .collect();

    let global_this_shape_id = add_object(shapes, "Global$globalThis", typed_global_props.clone());
    let global_this_type =
        Type::Object(ObjectType { shape_id: Some(global_this_shape_id.clone()) });
    globals.insert("globalThis".to_string(), Global::Typed(global_this_type.clone()));

    // Add self-referencing property: globalThis.globalThis → globalThis
    // This matches the TS reference where TYPED_GLOBALS includes globalThis itself.
    if let Some(shape) = shapes.get_mut(&global_this_shape_id) {
        shape.properties.insert("globalThis".to_string(), global_this_type.clone());
    }

    let global_shape_id = add_object(shapes, "Global$global", typed_global_props);
    let global_type = Type::Object(ObjectType { shape_id: Some(global_shape_id.clone()) });
    globals.insert("global".to_string(), Global::Typed(global_type.clone()));

    // Add self-referencing property: global.global → global
    if let Some(shape) = shapes.get_mut(&global_shape_id) {
        shape.properties.insert("global".to_string(), global_type);
        shape.properties.insert("globalThis".to_string(), global_this_type);
    }
}
