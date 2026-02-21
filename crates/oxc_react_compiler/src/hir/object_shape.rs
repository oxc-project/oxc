/// Object shapes and function signatures for the React Compiler.
///
/// Port of `HIR/ObjectShape.ts` from the React Compiler.
///
/// This module defines the shapes (property maps and call signatures)
/// for built-in JavaScript objects, React hooks, and other known APIs.
/// These shapes are used during type inference and effect analysis.
use rustc_hash::FxHashMap;

use super::{
    hir_types::{Effect, ValueKind, ValueReason},
    types::Type,
};

/// The kind of a React hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookKind {
    UseContext,
    UseState,
    UseActionState,
    UseReducer,
    UseRef,
    UseEffect,
    UseLayoutEffect,
    UseInsertionEffect,
    UseMemo,
    UseCallback,
    UseTransition,
    UseImperativeHandle,
    UseEffectEvent,
    UseOptimistic,
    Custom,
}

/// Call signature of a function, used for type and effect inference.
///
/// Note: Param type is not recorded since it currently does not affect inference.
/// Specifically, we currently do not:
/// - infer types based on their usage in argument position
/// - handle inference for overloaded / generic functions
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub positional_params: Vec<Effect>,
    pub rest_param: Option<Effect>,
    pub return_type: Type,
    pub return_value_kind: ValueKind,
    pub return_value_reason: Option<ValueReason>,
    pub callee_effect: Effect,
    pub hook_kind: Option<HookKind>,
    pub no_alias: bool,
    pub mutable_only_if_operands_are_mutable: bool,
    pub impure: bool,
    pub known_incompatible: Option<String>,
    pub canonical_name: Option<String>,
}

impl Default for FunctionSignature {
    fn default() -> Self {
        Self {
            positional_params: Vec::new(),
            rest_param: None,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Mutable,
            return_value_reason: None,
            callee_effect: Effect::Read,
            hook_kind: None,
            no_alias: false,
            mutable_only_if_operands_are_mutable: false,
            impure: false,
            known_incompatible: None,
            canonical_name: None,
        }
    }
}

/// Shape of a function or object type.
///
/// Constructors and other functions are both represented by the `function_type` field.
/// Properties are a map from property name to the built-in type of that property.
#[derive(Debug, Clone)]
pub struct ObjectShape {
    pub properties: FxHashMap<String, Type>,
    pub function_type: Option<FunctionSignature>,
}

/// A registry mapping shape IDs to their object shapes.
pub type ShapeRegistry = FxHashMap<String, ObjectShape>;

// =====================================================================================
// Built-in shape IDs
// =====================================================================================

pub const BUILT_IN_PROPS_ID: &str = "BuiltInProps";
pub const BUILT_IN_ARRAY_ID: &str = "BuiltInArray";
pub const BUILT_IN_SET_ID: &str = "BuiltInSet";
pub const BUILT_IN_MAP_ID: &str = "BuiltInMap";
pub const BUILT_IN_WEAK_SET_ID: &str = "BuiltInWeakSet";
pub const BUILT_IN_WEAK_MAP_ID: &str = "BuiltInWeakMap";
pub const BUILT_IN_FUNCTION_ID: &str = "BuiltInFunction";
pub const BUILT_IN_JSX_ID: &str = "BuiltInJsx";
pub const BUILT_IN_OBJECT_ID: &str = "BuiltInObject";
pub const BUILT_IN_USE_STATE_ID: &str = "BuiltInUseState";
pub const BUILT_IN_SET_STATE_ID: &str = "BuiltInSetState";
pub const BUILT_IN_USE_ACTION_STATE_ID: &str = "BuiltInUseActionState";
pub const BUILT_IN_SET_ACTION_STATE_ID: &str = "BuiltInSetActionState";
pub const BUILT_IN_USE_REF_ID: &str = "BuiltInUseRefId";
pub const BUILT_IN_REF_VALUE_ID: &str = "BuiltInRefValue";
pub const BUILT_IN_MIXED_READONLY_ID: &str = "BuiltInMixedReadonly";
pub const BUILT_IN_USE_EFFECT_HOOK_ID: &str = "BuiltInUseEffectHook";
pub const BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID: &str = "BuiltInUseLayoutEffectHook";
pub const BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID: &str = "BuiltInUseInsertionEffectHook";
pub const BUILT_IN_USE_OPERATOR_ID: &str = "BuiltInUseOperator";
pub const BUILT_IN_USE_REDUCER_ID: &str = "BuiltInUseReducer";
pub const BUILT_IN_DISPATCH_ID: &str = "BuiltInDispatch";
pub const BUILT_IN_USE_CONTEXT_HOOK_ID: &str = "BuiltInUseContextHook";
pub const BUILT_IN_USE_TRANSITION_ID: &str = "BuiltInUseTransition";
pub const BUILT_IN_USE_OPTIMISTIC_ID: &str = "BuiltInUseOptimistic";
pub const BUILT_IN_SET_OPTIMISTIC_ID: &str = "BuiltInSetOptimistic";
pub const BUILT_IN_START_TRANSITION_ID: &str = "BuiltInStartTransition";
pub const BUILT_IN_USE_EFFECT_EVENT_ID: &str = "BuiltInUseEffectEvent";
pub const BUILT_IN_EFFECT_EVENT_ID: &str = "BuiltInEffectEventFunction";
pub const REANIMATED_SHARED_VALUE_ID: &str = "ReanimatedSharedValueId";

// =====================================================================================
// Helper functions for building shape registries
// =====================================================================================

/// Add a function shape to a registry.
///
/// Returns the shape_id used for the function.
pub fn add_function(
    registry: &mut ShapeRegistry,
    id: Option<&str>,
    properties: Vec<(String, Type)>,
    signature: FunctionSignature,
) -> String {
    let shape_id = id.map_or_else(|| format!("<generated_{}>", registry.len()), ToString::to_string);
    let shape = ObjectShape {
        properties: properties.into_iter().collect(),
        function_type: Some(signature),
    };
    registry.insert(shape_id.clone(), shape);
    shape_id
}

/// Add a hook shape to a registry.
///
/// Returns the shape_id used for the hook.
pub fn add_hook(
    registry: &mut ShapeRegistry,
    id: Option<&str>,
    signature: FunctionSignature,
) -> String {
    add_function(registry, id, Vec::new(), signature)
}

/// Add an object shape to a registry (no function type).
///
/// Returns the shape_id used for the object.
pub fn add_object(
    registry: &mut ShapeRegistry,
    id: &str,
    properties: Vec<(String, Type)>,
) -> String {
    let shape = ObjectShape {
        properties: properties.into_iter().collect(),
        function_type: None,
    };
    registry.insert(id.to_string(), shape);
    id.to_string()
}
