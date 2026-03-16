use serde::Deserialize;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// ES2020 transform options.
pub struct ES2020Options {
    /// Enable `export * as ns from` transform.
    #[serde(skip)]
    pub export_namespace_from: bool,

    /// Enable nullish coalescing transform.
    #[serde(skip)]
    pub nullish_coalescing_operator: bool,

    /// Enable bigint syntax transform.
    #[serde(skip)]
    pub big_int: bool,

    /// Enable optional chaining transform.
    #[serde(skip)]
    pub optional_chaining: bool,

    /// Enable arbitrary module namespace name transform.
    #[serde(skip)]
    pub arbitrary_module_namespace_names: bool,
}
