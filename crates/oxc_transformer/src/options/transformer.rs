use std::path::PathBuf;

use serde_json::{json, Value};

use oxc_diagnostics::{Error, OxcDiagnostic};

use crate::{
    common::helper_loader::{HelperLoaderMode, HelperLoaderOptions},
    compiler_assumptions::CompilerAssumptions,
    env::{can_enable_plugin, EnvOptions, Versions},
    es2015::{ArrowFunctionsOptions, ES2015Options},
    es2016::ES2016Options,
    es2017::options::ES2017Options,
    es2018::{ES2018Options, ObjectRestSpreadOptions},
    es2019::ES2019Options,
    es2020::ES2020Options,
    es2021::ES2021Options,
    es2022::ES2022Options,
    jsx::JsxOptions,
    options::babel::BabelOptions,
    regexp::RegExpOptions,
    typescript::TypeScriptOptions,
    ReactRefreshOptions,
};

/// <https://babel.dev/docs/options>
#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    //
    // Primary Options
    //
    /// The working directory that all paths in the programmatic options will be resolved relative to.
    pub cwd: PathBuf,

    // Core
    /// Set assumptions in order to produce smaller output.
    /// For more information, check the [assumptions](https://babel.dev/docs/assumptions) documentation page.
    pub assumptions: CompilerAssumptions,

    // Plugins
    /// [preset-typescript](https://babeljs.io/docs/babel-preset-typescript)
    pub typescript: TypeScriptOptions,

    /// Jsx Transform
    ///
    /// See [preset-react](https://babeljs.io/docs/babel-preset-react)
    pub jsx: JsxOptions,

    pub regexp: RegExpOptions,

    pub es2015: ES2015Options,

    pub es2016: ES2016Options,

    pub es2017: ES2017Options,

    pub es2018: ES2018Options,

    pub es2019: ES2019Options,

    pub es2020: ES2020Options,

    pub es2021: ES2021Options,

    pub es2022: ES2022Options,

    pub helper_loader: HelperLoaderOptions,
}

impl TransformOptions {
    /// Explicitly enable all plugins that are ready, mainly for testing purposes.
    pub fn enable_all() -> Self {
        Self {
            cwd: PathBuf::new(),
            assumptions: CompilerAssumptions::default(),
            typescript: TypeScriptOptions::default(),
            jsx: JsxOptions {
                development: true,
                refresh: Some(ReactRefreshOptions::default()),
                ..JsxOptions::default()
            },
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
                arrow_function: None,
            },
            es2016: ES2016Options { exponentiation_operator: true },
            es2017: ES2017Options {
                // Turned off because it is not ready.
                async_to_generator: false,
            },
            es2018: ES2018Options { object_rest_spread: Some(ObjectRestSpreadOptions::default()) },
            es2019: ES2019Options { optional_catch_binding: true },
            es2020: ES2020Options { nullish_coalescing_operator: true },
            es2021: ES2021Options { logical_assignment_operators: true },
            es2022: ES2022Options { class_static_block: true },
            helper_loader: HelperLoaderOptions {
                mode: HelperLoaderMode::Runtime,
                ..Default::default()
            },
        }
    }
}

impl TryFrom<&EnvOptions> for TransformOptions {
    type Error = Vec<Error>;

    /// If there are any errors in the `options.targets``, they will be returned as a list of errors.
    fn try_from(options: &EnvOptions) -> Result<Self, Self::Error> {
        let targets = match options.targets.clone().get_targets() {
            Ok(targets) => Some(targets),
            Err(err) => return Err(vec![err]),
        };
        let bugfixes = options.bugfixes;
        let targets = targets.as_ref();
        Ok(Self {
            regexp: RegExpOptions {
                sticky_flag: can_enable_plugin("transform-sticky-regex", targets, bugfixes),
                unicode_flag: can_enable_plugin("transform-unicode-regex", targets, bugfixes),
                dot_all_flag: can_enable_plugin("transform-dotall-regex", targets, bugfixes),
                look_behind_assertions: can_enable_plugin(
                    "esbuild-regexp-lookbehind-assertions",
                    targets,
                    bugfixes,
                ),
                named_capture_groups: can_enable_plugin(
                    "transform-named-capturing-groups-regex",
                    targets,
                    bugfixes,
                ),
                unicode_property_escapes: can_enable_plugin(
                    "transform-unicode-property-regex",
                    targets,
                    bugfixes,
                ),
                match_indices: can_enable_plugin("esbuild-regexp-match-indices", targets, bugfixes),
                set_notation: can_enable_plugin("transform-unicode-sets-regex", targets, bugfixes),
            },
            es2015: ES2015Options {
                arrow_function: can_enable_plugin("transform-arrow-functions", targets, bugfixes)
                    .then(Default::default),
            },
            es2016: ES2016Options {
                exponentiation_operator: can_enable_plugin(
                    "transform-exponentiation-operator",
                    targets,
                    bugfixes,
                ),
            },
            es2017: ES2017Options {
                async_to_generator: can_enable_plugin(
                    "transform-async-to-generator",
                    targets,
                    bugfixes,
                ),
            },
            es2018: ES2018Options {
                object_rest_spread: can_enable_plugin(
                    "transform-object-rest-spread",
                    targets,
                    bugfixes,
                )
                .then(Default::default),
            },
            es2019: ES2019Options {
                optional_catch_binding: can_enable_plugin(
                    "transform-optional-catch-binding",
                    targets,
                    bugfixes,
                ),
            },
            es2020: ES2020Options {
                nullish_coalescing_operator: can_enable_plugin(
                    "transform-nullish-coalescing-operator",
                    targets,
                    bugfixes,
                ),
            },
            es2021: ES2021Options {
                logical_assignment_operators: can_enable_plugin(
                    "transform-logical-assignment-operators",
                    targets,
                    bugfixes,
                ),
            },
            es2022: ES2022Options {
                class_static_block: can_enable_plugin(
                    "transform-class-static-block",
                    targets,
                    bugfixes,
                ),
            },
            ..Default::default()
        })
    }
}

impl TryFrom<&BabelOptions> for TransformOptions {
    type Error = Vec<Error>;

    /// If the `options` contains any unknown fields, they will be returned as a list of errors.
    fn try_from(options: &BabelOptions) -> Result<Self, Self::Error> {
        let mut errors = Vec::<Error>::new();

        let assumptions = if options.assumptions.is_null() {
            CompilerAssumptions::default()
        } else {
            serde_json::from_value::<CompilerAssumptions>(options.assumptions.clone())
                .inspect_err(|err| errors.push(OxcDiagnostic::error(err.to_string()).into()))
                .unwrap_or_default()
        };

        let typescript = if options.has_preset("typescript") {
            serde_json::from_value::<TypeScriptOptions>(
                options.get_preset("typescript").flatten().unwrap_or_else(|| json!({})),
            )
            .inspect_err(|err| report_error("typescript", err, true, &mut errors))
        } else {
            serde_json::from_value::<TypeScriptOptions>(get_plugin_options(
                "transform-typescript",
                options,
            ))
            .inspect_err(|err| report_error("typescript", err, false, &mut errors))
        }
        .unwrap_or_default();

        let jsx = if let Some(value) = options.get_preset("react").flatten() {
            serde_json::from_value::<JsxOptions>(value)
                .inspect_err(|err| report_error("react", err, true, &mut errors))
                .unwrap_or_default()
        } else {
            let jsx_plugin_name = "transform-react-jsx";
            let jsx_dev_name = "transform-react-jsx-development";
            let has_jsx_plugin = options.has_plugin(jsx_plugin_name);
            let mut react_options = if has_jsx_plugin {
                serde_json::from_value::<JsxOptions>(get_plugin_options(jsx_plugin_name, options))
                    .inspect_err(|err| report_error(jsx_plugin_name, err, false, &mut errors))
            } else {
                serde_json::from_value::<JsxOptions>(get_plugin_options(jsx_dev_name, options))
                    .inspect_err(|err| report_error(jsx_dev_name, err, false, &mut errors))
            }
            .unwrap_or_default();
            react_options.development = options.has_plugin(jsx_dev_name);
            react_options.jsx_plugin = has_jsx_plugin;
            react_options.display_name_plugin = options.has_plugin("transform-react-display-name");
            react_options.jsx_self_plugin = options.has_plugin("transform-react-jsx-self");
            react_options.jsx_source_plugin = options.has_plugin("transform-react-jsx-source");
            react_options
        };

        let env = options.get_preset("env").flatten().and_then(|value| {
            serde_json::from_value::<EnvOptions>(value)
                .inspect_err(|err| report_error("env", err, true, &mut errors))
                .ok()
        });

        let targets = env.as_ref().and_then(|env| {
            env.targets
                .clone()
                .get_targets()
                .inspect_err(|err| errors.push(OxcDiagnostic::error(err.to_string()).into()))
                .ok()
        });

        let bugfixes = env.as_ref().is_some_and(|o| o.bugfixes);

        let targets = targets.as_ref();

        let regexp = RegExpOptions {
            sticky_flag: can_enable_plugin("transform-sticky-regex", targets, bugfixes)
                || options.has_plugin("transform-sticky-regex"),
            unicode_flag: can_enable_plugin("transform-unicode-regex", targets, bugfixes)
                || options.has_plugin("transform-unicode-regex"),
            dot_all_flag: can_enable_plugin("transform-dotall-regex", targets, bugfixes)
                || options.has_plugin("transform-dotall-regex"),
            look_behind_assertions: can_enable_plugin(
                "esbuild-regexp-lookbehind-assertions",
                targets,
                bugfixes,
            ),
            named_capture_groups: can_enable_plugin(
                "transform-named-capturing-groups-regex",
                targets,
                bugfixes,
            ) || options.has_plugin("transform-named-capturing-groups-regex"),
            unicode_property_escapes: can_enable_plugin(
                "transform-unicode-property-regex",
                targets,
                bugfixes,
            ) || options.has_plugin("transform-unicode-property-regex"),
            match_indices: can_enable_plugin("esbuild-regexp-match-indices", targets, bugfixes),
            set_notation: can_enable_plugin("transform-unicode-sets-regex", targets, bugfixes)
                || options.has_plugin("transform-unicode-sets-regex"),
        };

        let es2015 = ES2015Options {
            arrow_function: {
                let plugin_name = "transform-arrow-functions";
                get_enabled_plugin_options(plugin_name, options, targets, bugfixes).map(|options| {
                    serde_json::from_value::<ArrowFunctionsOptions>(options)
                        .inspect_err(|err| report_error(plugin_name, err, false, &mut errors))
                        .unwrap_or_default()
                })
            },
        };

        let es2016 = ES2016Options {
            exponentiation_operator: get_enabled_plugin_options(
                "transform-exponentiation-operator",
                options,
                targets,
                bugfixes,
            )
            .is_some(),
        };

        let es2017 = ES2017Options {
            async_to_generator: get_enabled_plugin_options(
                "transform-async-to-generator",
                options,
                targets,
                bugfixes,
            )
            .is_some(),
        };

        let es2018 = ES2018Options {
            object_rest_spread: {
                let plugin_name = "transform-object-rest-spread";
                get_enabled_plugin_options(plugin_name, options, targets, bugfixes).map(|options| {
                    serde_json::from_value::<ObjectRestSpreadOptions>(options)
                        .inspect_err(|err| report_error(plugin_name, err, false, &mut errors))
                        .unwrap_or_default()
                })
            },
        };

        let es2019 = ES2019Options {
            optional_catch_binding: {
                get_enabled_plugin_options(
                    "transform-optional-catch-binding",
                    options,
                    targets,
                    bugfixes,
                )
                .is_some()
            },
        };

        let es2020 = ES2020Options {
            nullish_coalescing_operator: get_enabled_plugin_options(
                "transform-nullish-coalescing-operator",
                options,
                targets,
                bugfixes,
            )
            .is_some(),
        };

        let es2021 = ES2021Options {
            logical_assignment_operators: get_enabled_plugin_options(
                "transform-logical-assignment-operators",
                options,
                targets,
                bugfixes,
            )
            .is_some(),
        };

        let es2022 = ES2022Options {
            class_static_block: get_enabled_plugin_options(
                "transform-class-static-block",
                options,
                targets,
                bugfixes,
            )
            .is_some(),
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        let helper_loader = HelperLoaderOptions {
            mode: if options.external_helpers {
                HelperLoaderMode::External
            } else {
                HelperLoaderMode::default()
            },
            ..HelperLoaderOptions::default()
        };

        Ok(Self {
            cwd: options.cwd.clone().unwrap_or_default(),
            assumptions,
            typescript,
            jsx,
            regexp,
            es2015,
            es2016,
            es2017,
            es2018,
            es2019,
            es2020,
            es2021,
            es2022,
            helper_loader,
        })
    }
}

fn get_plugin_options(name: &str, babel_options: &BabelOptions) -> Value {
    babel_options.get_plugin(name).and_then(|options| options).unwrap_or_else(|| json!({}))
}

fn get_enabled_plugin_options(
    plugin_name: &str,
    babel_options: &BabelOptions,
    targets: Option<&Versions>,
    bugfixes: bool,
) -> Option<Value> {
    let can_enable =
        can_enable_plugin(plugin_name, targets, bugfixes) || babel_options.has_plugin(plugin_name);

    if can_enable {
        get_plugin_options(plugin_name, babel_options).into()
    } else {
        None
    }
}

fn report_error(name: &str, err: &serde_json::Error, is_preset: bool, errors: &mut Vec<Error>) {
    let message =
        if is_preset { format!("preset-{name}: {err}",) } else { format!("{name}: {err}",) };
    errors.push(OxcDiagnostic::error(message).into());
}

#[test]
fn test_deny_unknown_fields() {
    let options = serde_json::json!({
        "plugins": [["transform-react-jsx", { "runtime": "automatic", "filter": 1 }]],
        "sourceType": "module"
    });
    let babel_options = serde_json::from_value::<BabelOptions>(options).unwrap();
    let result = TransformOptions::try_from(&babel_options);
    assert!(result.is_err());
    let err_message =
        result.err().unwrap().iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
    assert!(err_message.contains("transform-react-jsx: unknown field `filter`"));
}
