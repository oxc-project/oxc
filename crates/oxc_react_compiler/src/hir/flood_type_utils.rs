/// Type utility functions for the Flood type system.
///
/// Port of `Flood/TypeUtils.ts` from the React Compiler.
///
/// Provides substitution, mapping, and structural operations on Flood types.
use rustc_hash::FxHashMap;

use super::flood_types::{
    ConcreteType, FloodType, Platform, TypeParameter, TypeParameterId,
};

/// Substitute type parameters with concrete type arguments.
pub fn substitute(
    ty: &ConcreteType,
    type_parameters: &[TypeParameter],
    type_arguments: &[FloodType],
) -> FloodType {
    let mut subst_map: FxHashMap<TypeParameterId, FloodType> = FxHashMap::default();
    for (param, arg) in type_parameters.iter().zip(type_arguments.iter()) {
        subst_map.insert(param.id, arg.clone());
    }

    let substituted = map_type(&|t: &FloodType| -> FloodType {
        if let FloodType::Concrete { ty: ConcreteType::Generic { id, .. }, .. } = t {
            if let Some(substituted) = subst_map.get(id) {
                return substituted.clone();
            }
        }
        t.clone()
    }, ty);

    FloodType::Concrete { ty: substituted, platform: Platform::Universal }
}

/// Map a function over all sub-types of a concrete type.
pub fn map_type(
    f: &dyn Fn(&FloodType) -> FloodType,
    ty: &ConcreteType,
) -> ConcreteType {
    match ty {
        ConcreteType::Number
        | ConcreteType::String
        | ConcreteType::Boolean
        | ConcreteType::Void
        | ConcreteType::Mixed
        | ConcreteType::Enum => ty.clone(),

        ConcreteType::Nullable(inner) => {
            ConcreteType::Nullable(Box::new(f(inner)))
        }
        ConcreteType::Array { element } => {
            ConcreteType::Array { element: Box::new(f(element)) }
        }
        ConcreteType::Set { element } => {
            ConcreteType::Set { element: Box::new(f(element)) }
        }
        ConcreteType::Map { key, value } => {
            ConcreteType::Map { key: Box::new(f(key)), value: Box::new(f(value)) }
        }
        ConcreteType::Function { type_parameters, params, return_type } => {
            let new_params = params.iter().map(f).collect();
            let new_return = Box::new(f(return_type));
            ConcreteType::Function {
                type_parameters: type_parameters.clone(),
                params: new_params,
                return_type: new_return,
            }
        }
        ConcreteType::Component { props, children } => {
            let new_props = props.iter().map(|(k, v)| (k.clone(), f(v))).collect();
            let new_children = children.as_ref().map(|c| Box::new(f(c)));
            ConcreteType::Component { props: new_props, children: new_children }
        }
        ConcreteType::Generic { id, bound } => {
            ConcreteType::Generic { id: *id, bound: Box::new(f(bound)) }
        }
        ConcreteType::Object { id, members } => {
            let new_members = members.iter().map(|(k, v)| (k.clone(), f(v))).collect();
            ConcreteType::Object { id: *id, members: new_members }
        }
        ConcreteType::Tuple { id, members } => {
            let new_members = members.iter().map(f).collect();
            ConcreteType::Tuple { id: *id, members: new_members }
        }
        ConcreteType::Structural { id } => ConcreteType::Structural { id: *id },
        ConcreteType::Union(types) => {
            ConcreteType::Union(types.iter().map(f).collect())
        }
        ConcreteType::Intersection(types) => {
            ConcreteType::Intersection(types.iter().map(f).collect())
        }
    }
}

/// Check if a type is nullable (contains Void or Nullable).
pub fn is_nullable(ty: &ConcreteType) -> bool {
    matches!(ty, ConcreteType::Void | ConcreteType::Nullable(_) | ConcreteType::Mixed)
}

/// Unwrap a nullable type, returning the inner type.
pub fn unwrap_nullable(ty: &ConcreteType) -> Option<&FloodType> {
    match ty {
        ConcreteType::Nullable(inner) => Some(inner),
        _ => None,
    }
}

/// Check if a type is a primitive.
pub fn is_primitive(ty: &ConcreteType) -> bool {
    matches!(
        ty,
        ConcreteType::Number | ConcreteType::String | ConcreteType::Boolean | ConcreteType::Void
    )
}
