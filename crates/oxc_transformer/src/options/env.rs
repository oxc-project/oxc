use serde::Deserialize;

use crate::{
    es2015::{ArrowFunctionsOptions, ES2015Options},
    es2016::ES2016Options,
    es2017::ES2017Options,
    es2018::{ES2018Options, ObjectRestSpreadOptions},
    es2019::ES2019Options,
    es2020::ES2020Options,
    es2021::ES2021Options,
    es2022::{ClassPropertiesOptions, ES2022Options},
    regexp::RegExpOptions,
};

use super::babel::BabelEnvOptions;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(try_from = "BabelEnvOptions")]
pub struct EnvOptions {
    pub regexp: RegExpOptions,

    pub es2015: ES2015Options,

    pub es2016: ES2016Options,

    pub es2017: ES2017Options,

    pub es2018: ES2018Options,

    pub es2019: ES2019Options,

    pub es2020: ES2020Options,

    pub es2021: ES2021Options,

    pub es2022: ES2022Options,
}

impl EnvOptions {
    /// Explicitly enable all plugins that are ready, mainly for testing purposes.
    ///
    /// NOTE: for internal use only
    #[doc(hidden)]
    pub fn enable_all(include_unfinished_plugins: bool) -> Self {
        Self {
            regexp: RegExpOptions {
                sticky_flag: true,
                unicode_flag: true,
                dot_all_flag: true,
                look_behind_assertions: true,
                named_capture_groups: true,
                unicode_property_escapes: true,
                match_indices: true,
                set_notation: true,
            },
            es2015: ES2015Options {
                // Turned off because it is not ready.
                arrow_function: if include_unfinished_plugins {
                    Some(ArrowFunctionsOptions::default())
                } else {
                    None
                },
            },
            es2016: ES2016Options { exponentiation_operator: true },
            es2017: ES2017Options {
                // Turned off because it is not ready.
                async_to_generator: include_unfinished_plugins,
            },
            es2018: ES2018Options {
                // Turned off because it is not ready.
                object_rest_spread: if include_unfinished_plugins {
                    Some(ObjectRestSpreadOptions::default())
                } else {
                    None
                },
                // Turned off because it is not ready.
                async_generator_functions: include_unfinished_plugins,
            },
            es2019: ES2019Options { optional_catch_binding: true },
            es2020: ES2020Options { nullish_coalescing_operator: true },
            es2021: ES2021Options { logical_assignment_operators: true },
            es2022: ES2022Options {
                class_static_block: true,
                class_properties: if include_unfinished_plugins {
                    Some(ClassPropertiesOptions::default())
                } else {
                    None
                },
            },
        }
    }
}

impl TryFrom<BabelEnvOptions> for EnvOptions {
    type Error = String;

    /// If there are any errors in the `options.targets``, they will be returned as a list of errors.
    fn try_from(o: BabelEnvOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            regexp: RegExpOptions {
                sticky_flag: o.can_enable_plugin("transform-sticky-regex"),
                unicode_flag: o.can_enable_plugin("transform-unicode-regex"),
                dot_all_flag: o.can_enable_plugin("transform-dotall-regex"),
                look_behind_assertions: o.can_enable_plugin("esbuild-regexp-lookbehind-assertions"),
                named_capture_groups: o.can_enable_plugin("transform-named-capturing-groups-regex"),
                unicode_property_escapes: o.can_enable_plugin("transform-unicode-property-regex"),
                match_indices: o.can_enable_plugin("esbuild-regexp-match-indices"),
                set_notation: o.can_enable_plugin("transform-unicode-sets-regex"),
            },
            es2015: ES2015Options {
                arrow_function: o
                    .can_enable_plugin("transform-arrow-functions")
                    .then(Default::default),
            },
            es2016: ES2016Options {
                exponentiation_operator: o.can_enable_plugin("transform-exponentiation-operator"),
            },
            es2017: ES2017Options {
                async_to_generator: o.can_enable_plugin("transform-async-to-generator"),
            },
            es2018: ES2018Options {
                object_rest_spread: o
                    .can_enable_plugin("transform-object-rest-spread")
                    .then(Default::default),
                async_generator_functions: o
                    .can_enable_plugin("transform-async-generator-functions"),
            },
            es2019: ES2019Options {
                optional_catch_binding: o.can_enable_plugin("transform-optional-catch-binding"),
            },
            es2020: ES2020Options {
                nullish_coalescing_operator: o
                    .can_enable_plugin("transform-nullish-coalescing-operator"),
            },
            es2021: ES2021Options {
                logical_assignment_operators: o
                    .can_enable_plugin("transform-logical-assignment-operators"),
            },
            es2022: ES2022Options {
                class_static_block: o.can_enable_plugin("transform-class-static-block"),
                class_properties: o
                    .can_enable_plugin("transform-class-properties")
                    .then(Default::default),
            },
        })
    }
}
