//! Methods defined in the [ECMAScript Language Specification](https://tc39.es/ecma262).

mod bound_names;
mod has_proto;
mod is_simple_parameter_list;
mod private_bound_identifiers;
mod prop_name;

pub use self::{
    bound_names::BoundNames, has_proto::HasProto, is_simple_parameter_list::IsSimpleParameterList,
    private_bound_identifiers::PrivateBoundIdentifiers, prop_name::PropName,
};
