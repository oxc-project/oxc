use std::str::FromStr;

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

use super::{babel::BabelEnvOptions, ESFeature, ESTarget, Engine, Module};

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(try_from = "BabelEnvOptions")]
pub struct EnvOptions {
    /// Specify what module code is generated.
    pub module: Module,

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
            module: Module::default(),
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
                object_rest_spread: Some(ObjectRestSpreadOptions::default()),
                async_generator_functions: true,
            },
            es2019: ES2019Options { optional_catch_binding: true },
            es2020: ES2020Options {
                nullish_coalescing_operator: true,
                // Turn this on would throw error for all bigints.
                big_int: false,
                optional_chaining: true,
            },
            es2021: ES2021Options { logical_assignment_operators: true },
            es2022: ES2022Options {
                class_static_block: true,
                class_properties: Some(ClassPropertiesOptions::default()),
            },
        }
    }

    /// Initialize from a [browserslist] query.
    ///
    /// # Errors
    ///
    /// * When the query failed to parse.
    ///
    /// [browserslist]: <https://github.com/browserslist/browserslist>
    pub fn from_browserslist_query(query: &str) -> Result<Self, String> {
        EngineTargets::try_from_query(query).map(Self::from)
    }

    /// # Errors
    ///
    /// * When the query failed to parse.
    pub fn from_target(s: &str) -> Result<Self, String> {
        if s.contains(',') {
            Self::from_target_list(&s.split(',').collect::<Vec<_>>())
        } else {
            Self::from_target_list(&[s])
        }
    }

    /// # Errors
    ///
    /// * When the query failed to parse.
    pub fn from_target_list<S: AsRef<str>>(list: &[S]) -> Result<Self, String> {
        use crate::options::es_target::ESVersion;
        let mut es_target = None;
        let mut engine_targets = EngineTargets::default();

        for s in list {
            let s = s.as_ref();
            // Parse `esXXXX`.
            if let Ok(target) = ESTarget::from_str(s) {
                if let Some(target) = es_target {
                    return Err(format!("'{target}' is already specified."));
                }
                es_target = Some(target);
            } else {
                // Parse `chromeXX`, `edgeXX` etc.
                let (engine, version) = Engine::parse_name_and_version(s)?;
                if engine_targets.insert(engine, version).is_some() {
                    return Err(format!("'{s}' is already specified."));
                }
            }
        }
        engine_targets.insert(Engine::Es, es_target.unwrap_or(ESTarget::default()).version());
        Ok(EnvOptions::from(engine_targets))
    }
}

impl From<BabelEnvOptions> for EnvOptions {
    fn from(o: BabelEnvOptions) -> Self {
        Self::from(o.targets)
    }
}

impl From<EngineTargets> for EnvOptions {
    #[allow(clippy::enum_glob_use)]
    fn from(o: EngineTargets) -> Self {
        use ESFeature::*;
        Self {
            module: Module::default(),
            regexp: RegExpOptions {
                sticky_flag: o.has_feature(ES2015StickyRegex),
                unicode_flag: o.has_feature(ES2015UnicodeRegex),
                unicode_property_escapes: o.has_feature(ES2018UnicodePropertyRegex),
                dot_all_flag: o.has_feature(ES2018DotallRegex),
                named_capture_groups: o.has_feature(ES2018NamedCapturingGroupsRegex),
                look_behind_assertions: o.has_feature(ES2018LookbehindRegex),
                match_indices: o.has_feature(ES2022MatchIndicesRegex),
                set_notation: o.has_feature(ES2024UnicodeSetsRegex),
            },
            es2015: ES2015Options {
                arrow_function: o.has_feature(ES2015ArrowFunctions).then(Default::default),
            },
            es2016: ES2016Options {
                exponentiation_operator: o.has_feature(ES2016ExponentiationOperator),
            },
            es2017: ES2017Options { async_to_generator: o.has_feature(ES2017AsyncToGenerator) },
            es2018: ES2018Options {
                object_rest_spread: o.has_feature(ES2018ObjectRestSpread).then(Default::default),
                async_generator_functions: o.has_feature(ES2018AsyncGeneratorFunctions),
            },
            es2019: ES2019Options {
                optional_catch_binding: o.has_feature(ES2019OptionalCatchBinding),
            },
            es2020: ES2020Options {
                nullish_coalescing_operator: o.has_feature(ES2020NullishCoalescingOperator),
                big_int: o.has_feature(ES2020BigInt),
                optional_chaining: o.has_feature(ES2020OptionalChaining),
            },
            es2021: ES2021Options {
                logical_assignment_operators: o.has_feature(ES2021LogicalAssignmentOperators),
            },
            es2022: ES2022Options {
                class_static_block: o.has_feature(ES2022ClassStaticBlock),
                class_properties: o.has_feature(ES2022ClassProperties).then(Default::default),
            },
        }
    }
}
