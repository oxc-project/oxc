mod dynamic_import;
mod export_namespace_from;
mod nullish_coalescing_operator;

use serde::Deserialize;

use crate::impl_preset_transformation;
use crate::options::default_as_true;
use crate::preset_plugin::BoxedTransformation;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Es2020Options {
    /// https://babeljs.io/docs/babel-plugin-proposal-dynamic-import
    pub dynamic_import: bool,

    /// https://babeljs.io/docs/babel-plugin-transform-export-namespace-from
    #[serde(default = "default_as_true")]
    pub export_namespace_from: bool,

    /// https://babeljs.io/docs/babel-plugin-transform-nullish-coalescing-operator
    #[serde(default = "default_as_true")]
    pub nullish_coalescing_operator: bool,
}

impl Default for Es2020Options {
    fn default() -> Self {
        Self {
            dynamic_import: false, // Let bundlers handle it by default!
            export_namespace_from: default_as_true(),
            nullish_coalescing_operator: default_as_true(),
        }
    }
}

#[allow(dead_code)]
pub struct Es2020 {
    options: Es2020Options,
    plugins: Vec<BoxedTransformation>,
}

impl Es2020 {
    pub fn new(options: Es2020Options) -> Self {
        let mut plugins: Vec<BoxedTransformation> = vec![];

        // Ordered from most complex to least complex!
        if options.nullish_coalescing_operator {
            plugins.push(Box::new(nullish_coalescing_operator::NullishCoalescingOperator));
        }

        if options.export_namespace_from {
            plugins.push(Box::new(export_namespace_from::ExportNamespaceFrom));
        }

        if options.dynamic_import {
            plugins.push(Box::new(dynamic_import::DynamicImport));
        }

        Self { options, plugins }
    }
}

impl_preset_transformation!(Es2020);
