pub struct Es2020Options {
    /// https://babeljs.io/docs/babel-plugin-proposal-dynamic-import
    pub dynamic_import: bool,
    /// https://babeljs.io/docs/babel-plugin-transform-export-namespace-from
    pub export_namespace_from: bool,
    /// https://babeljs.io/docs/babel-plugin-transform-nullish-coalescing-operator
    pub nullish_coalescing_operator: bool,
}

impl Default for Es2020Options {
    fn default() -> Self {
        Self {
            dynamic_import: false, // Let bundlers handle it!
            export_namespace_from: true,
            nullish_coalescing_operator: true,
        }
    }
}
