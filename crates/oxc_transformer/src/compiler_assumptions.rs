use serde::Deserialize;

/// Compiler assumptions
///
/// For producing smaller output.
///
/// See <https://babeljs.io/docs/assumptions>
#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompilerAssumptions {
    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub array_like_is_iterable: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub constant_reexports: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub constant_super: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub enumerable_module_meta: bool,

    #[serde(default)]
    pub ignore_function_length: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub ignore_to_primitive_hint: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub iterable_is_array: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub mutable_template_object: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub no_class_calls: bool,

    #[serde(default)]
    pub no_document_all: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub no_incomplete_ns_import_detection: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub no_new_arrows: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub no_uninitialized_private_field_access: bool,

    #[serde(default)]
    pub object_rest_no_symbols: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub private_fields_as_symbols: bool,

    #[serde(default)]
    pub private_fields_as_properties: bool,

    #[serde(default)]
    pub pure_getters: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub set_class_methods: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub set_computed_properties: bool,

    #[serde(default)]
    pub set_public_class_fields: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub set_spread_properties: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub skip_for_of_iterator_closing: bool,

    #[serde(default)]
    #[deprecated = "Not Implemented"]
    pub super_is_callable_constructor: bool,
}
