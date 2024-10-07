//! Traits for JavaScript syntax operations
//!
//! * BoundNames
//! * IsSimpleParameterList
//! * PrivateBoundIdentifiers
//! * PropName
//! * Constant Folding
//!
//! See [ECMA262 Syntax-Directed Operations](https://tc39.es/ecma262/#sec-syntax-directed-operations)

mod bound_names;
mod has_proto;
mod is_simple_parameter_list;
mod private_bound_identifiers;
mod prop_name;

pub use self::{
    bound_names::BoundNames, has_proto::HasProto, is_simple_parameter_list::IsSimpleParameterList,
    private_bound_identifiers::PrivateBoundIdentifiers, prop_name::PropName,
};
