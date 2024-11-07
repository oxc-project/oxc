use std::str::FromStr;

use cow_utils::CowUtils;
use oxc_diagnostics::Error;
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
    EngineTargets,
};

use super::{babel::BabelEnvOptions, ESFeature};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum ESTarget {
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2023,
    ES2024,
    #[default]
    ESNext,
}

impl FromStr for ESTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.cow_to_lowercase().as_ref() {
            "es5" => Ok(Self::ES5),
            "es2015" => Ok(Self::ES2015),
            "es2016" => Ok(Self::ES2016),
            "es2017" => Ok(Self::ES2017),
            "es2018" => Ok(Self::ES2018),
            "es2019" => Ok(Self::ES2019),
            "es2020" => Ok(Self::ES2020),
            "es2021" => Ok(Self::ES2021),
            "es2022" => Ok(Self::ES2022),
            "es2023" => Ok(Self::ES2023),
            "es2024" => Ok(Self::ES2024),
            "esnext" => Ok(Self::ESNext),
            _ => Err(format!("Invalid target \"{s}\".")),
        }
    }
}

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
                unicode_property_escapes: true,
                dot_all_flag: true,
                named_capture_groups: true,
                look_behind_assertions: true,
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
            es2017: ES2017Options { async_to_generator: true },
            es2018: ES2018Options {
                // Turned off because it is not ready.
                object_rest_spread: if include_unfinished_plugins {
                    Some(ObjectRestSpreadOptions::default())
                } else {
                    None
                },
                async_generator_functions: true,
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

    /// # Errors
    ///
    /// * When the query failed to parse.
    pub fn from_browserslist_query(query: &str) -> Result<Self, Error> {
        Self::try_from(BabelEnvOptions {
            targets: EngineTargets::try_from_query(query)?,
            ..BabelEnvOptions::default()
        })
        .map_err(|err| Error::msg(err))
    }
}

impl From<ESTarget> for EnvOptions {
    fn from(target: ESTarget) -> Self {
        Self {
            regexp: RegExpOptions {
                sticky_flag: target < ESTarget::ES2015,
                unicode_flag: target < ESTarget::ES2015,
                unicode_property_escapes: target < ESTarget::ES2018,
                dot_all_flag: target < ESTarget::ES2015,
                named_capture_groups: target < ESTarget::ES2018,
                look_behind_assertions: target < ESTarget::ES2018,
                match_indices: target < ESTarget::ES2022,
                set_notation: target < ESTarget::ES2024,
            },
            es2015: ES2015Options {
                arrow_function: (target < ESTarget::ES2015).then(ArrowFunctionsOptions::default),
            },
            es2016: ES2016Options { exponentiation_operator: target < ESTarget::ES2016 },
            es2017: ES2017Options { async_to_generator: target < ESTarget::ES2017 },
            es2018: ES2018Options {
                object_rest_spread: (target < ESTarget::ES2018)
                    .then(ObjectRestSpreadOptions::default),
                async_generator_functions: target < ESTarget::ES2018,
            },
            es2019: ES2019Options { optional_catch_binding: target < ESTarget::ES2019 },
            es2020: ES2020Options { nullish_coalescing_operator: target < ESTarget::ES2020 },
            es2021: ES2021Options { logical_assignment_operators: target < ESTarget::ES2021 },
            es2022: ES2022Options {
                class_static_block: target < ESTarget::ES2022,
                class_properties: (target < ESTarget::ES2022).then(ClassPropertiesOptions::default),
            },
        }
    }
}

impl TryFrom<BabelEnvOptions> for EnvOptions {
    type Error = String;

    #[allow(clippy::enum_glob_use)]
    /// If there are any errors in the `options.targets``, they will be returned as a list of errors.
    fn try_from(o: BabelEnvOptions) -> Result<Self, Self::Error> {
        use ESFeature::*;
        Ok(Self {
            regexp: RegExpOptions {
                sticky_flag: o.can_enable(ES2015StickyRegex),
                unicode_flag: o.can_enable(ES2015UnicodeRegex),
                unicode_property_escapes: o.can_enable(ES2018UnicodePropertyRegex),
                dot_all_flag: o.can_enable(ES2018DotallRegex),
                named_capture_groups: o.can_enable(ES2018NamedCapturingGroupsRegex),
                // FIXME
                look_behind_assertions: false, // o.can_enable("esbuild-regexp-lookbehind-assertions"),
                // FIXME
                match_indices: false, // o.can_enable("esbuild-regexp-match-indices"),
                set_notation: o.can_enable(ES2024UnicodeSetsRegex),
            },
            es2015: ES2015Options {
                arrow_function: o.can_enable(ES2015ArrowFunctions).then(Default::default),
            },
            es2016: ES2016Options {
                exponentiation_operator: o.can_enable(ES2016ExponentiationOperator),
            },
            es2017: ES2017Options { async_to_generator: o.can_enable(ES2017AsyncToGenerator) },
            es2018: ES2018Options {
                object_rest_spread: o.can_enable(ES2018ObjectRestSpread).then(Default::default),
                async_generator_functions: o.can_enable(ES2018AsyncGeneratorFunctions),
            },
            es2019: ES2019Options {
                optional_catch_binding: o.can_enable(ES2018OptionalCatchBinding),
            },
            es2020: ES2020Options {
                nullish_coalescing_operator: o.can_enable(ES2020NullishCoalescingOperator),
            },
            es2021: ES2021Options {
                logical_assignment_operators: o.can_enable(ES2020LogicalAssignmentOperators),
            },
            es2022: ES2022Options {
                class_static_block: o.can_enable(ES2022ClassStaticBlock),
                class_properties: o.can_enable(ES2022ClassProperties).then(Default::default),
            },
        })
    }
}
