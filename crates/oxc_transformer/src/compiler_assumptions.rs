use serde::Deserialize;

/// Compiler assumptions
///
/// For producing smaller output.
///
/// See <https://babeljs.io/docs/assumptions>
#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerAssumptions {
    pub array_like_is_iterable: bool,
    pub constant_reexports: bool,
    pub constant_super: bool,
    pub enumerable_module_meta: bool,
    pub ignore_function_length: bool,
    pub ignore_to_primitive_hint: bool,
    pub iterable_is_array: bool,
    pub mutable_template_object: bool,
    pub no_class_calls: bool,
    pub no_document_all: bool,
    pub no_incomplete_ns_import_detection: bool,
    pub no_new_arrows: bool,
    pub no_uninitialized_private_field_access: bool,
    pub object_rest_no_symbols: bool,
    pub private_fields_as_symbols: bool,
    pub private_fields_as_properties: bool,
    pub pure_getters: bool,
    pub set_class_methods: bool,
    pub set_computed_properties: bool,
    pub set_public_class_fields: bool,
    pub set_spread_properties: bool,
    pub skip_for_of_iterator_closing: bool,
    pub super_is_callable_constructor: bool,
}
