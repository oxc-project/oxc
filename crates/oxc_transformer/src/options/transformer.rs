use std::path::PathBuf;

use serde_json::{from_value, json, Value};

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
    options::babel::BabelOptions,
    react::JsxOptions,
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

    /// [preset-react](https://babeljs.io/docs/babel-preset-react)
    pub react: JsxOptions,

    pub regexp: RegExpOptions,

    pub es2015: ES2015Options,

    pub es2016: ES2016Options,

    pub es2017: ES2017Options,

    pub es2018: ES2018Options,

    pub es2019: ES2019Options,

    pub es2020: ES2020Options,

    pub es2021: ES2021Options,

    pub helper_loader: HelperLoaderOptions,
}

impl TransformOptions {
    /// Explicitly enable all plugins that are ready, mainly for testing purposes.
    pub fn enable_all() -> Self {
        Self {
            cwd: PathBuf::new(),
            assumptions: CompilerAssumptions::default(),
            typescript: TypeScriptOptions::default(),
            react: JsxOptions {
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
            es2018: ES2018Options { object_rest_spread: Some(ObjectRestSpreadOptions::default()) },
            es2017: ES2017Options {
                // Turned off because it is not ready.
                async_to_generator: false,
            },
            es2019: ES2019Options { optional_catch_binding: true },
            es2020: ES2020Options { nullish_coalescing_operator: true },
            es2021: ES2021Options { logical_assignment_operators: true },
            helper_loader: HelperLoaderOptions {
                mode: HelperLoaderMode::Runtime,
                ..Default::default()
            },
        }
    }

    fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            es2015: ES2015Options::from_targets_and_bugfixes(targets, bugfixes),
            es2016: ES2016Options::from_targets_and_bugfixes(targets, bugfixes),
            es2017: ES2017Options::from_targets_and_bugfixes(targets, bugfixes),
            es2018: ES2018Options::from_targets_and_bugfixes(targets, bugfixes),
            es2019: ES2019Options::from_targets_and_bugfixes(targets, bugfixes),
            es2020: ES2020Options::from_targets_and_bugfixes(targets, bugfixes),
            es2021: ES2021Options::from_targets_and_bugfixes(targets, bugfixes),
            regexp: RegExpOptions::from_targets_and_bugfixes(targets, bugfixes),
            ..Default::default()
        }
    }

    /// # Errors
    ///
    /// If there are any errors in the `options.targets``, they will be returned as a list of errors.
    pub fn from_preset_env(env_options: &EnvOptions) -> Result<Self, Vec<Error>> {
        let mut errors = Vec::<Error>::new();

        let targets = match env_options.get_targets() {
            Ok(t) => Some(t),
            Err(err) => {
                errors.push(OxcDiagnostic::error(err.to_string()).into());
                None
            }
        };
        let bugfixes = env_options.bugfixes;
        Ok(Self::from_targets_and_bugfixes(targets.as_ref(), bugfixes))
    }

    /// # Errors
    ///
    /// If the `options` contains any unknown fields, they will be returned as a list of errors.
    pub fn from_babel_options(options: &BabelOptions) -> Result<Self, Vec<Error>> {
        let mut errors = Vec::<Error>::new();

        let env_options = {
            let preset_name = "env";
            get_preset_options(preset_name, options).and_then(|value| {
                match from_value::<EnvOptions>(value) {
                    Ok(res) => Some(res),
                    Err(err) => {
                        report_error(preset_name, &err, true, &mut errors);
                        None
                    }
                }
            })
        };

        let targets = env_options.as_ref().and_then(|env| match env.get_targets() {
            Ok(res) => Some(res),
            Err(err) => {
                errors.push(OxcDiagnostic::error(err.to_string()).into());
                None
            }
        });
        let bugfixes = env_options.as_ref().is_some_and(|o| o.bugfixes);

        let mut transformer_options = if env_options.is_some() {
            TransformOptions::from_targets_and_bugfixes(targets.as_ref(), bugfixes)
        } else {
            TransformOptions::default()
        };

        let preset_name = "react";
        transformer_options.react = if let Some(value) = get_preset_options(preset_name, options) {
            match from_value::<JsxOptions>(value) {
                Ok(res) => res,
                Err(err) => {
                    report_error(preset_name, &err, true, &mut errors);
                    JsxOptions::default()
                }
            }
        } else {
            let has_jsx_plugin = options.has_plugin("transform-react-jsx");
            let has_jsx_development_plugin = options.has_plugin("transform-react-jsx-development");
            let mut react_options =
                if has_jsx_plugin {
                    let plugin_name = "transform-react-jsx";
                    from_value::<JsxOptions>(get_plugin_options(plugin_name, options))
                        .unwrap_or_else(|err| {
                            report_error(plugin_name, &err, false, &mut errors);
                            JsxOptions::default()
                        })
                } else {
                    let plugin_name = "transform-react-jsx-development";
                    from_value::<JsxOptions>(get_plugin_options(plugin_name, options))
                        .unwrap_or_else(|err| {
                            report_error(plugin_name, &err, false, &mut errors);
                            JsxOptions::default()
                        })
                };
            react_options.development = has_jsx_development_plugin;
            react_options.jsx_plugin = has_jsx_plugin;
            react_options.display_name_plugin = options.has_plugin("transform-react-display-name");
            react_options.jsx_self_plugin = options.has_plugin("transform-react-jsx-self");
            react_options.jsx_source_plugin = options.has_plugin("transform-react-jsx-source");
            react_options
        };

        transformer_options.es2015.with_arrow_function({
            let plugin_name = "transform-arrow-functions";
            get_enabled_plugin_options(plugin_name, options, targets.as_ref(), bugfixes).map(
                |options| {
                    from_value::<ArrowFunctionsOptions>(options).unwrap_or_else(|err| {
                        report_error(plugin_name, &err, false, &mut errors);
                        ArrowFunctionsOptions::default()
                    })
                },
            )
        });

        transformer_options.es2016.with_exponentiation_operator({
            let plugin_name = "transform-exponentiation-operator";
            get_enabled_plugin_options(plugin_name, options, targets.as_ref(), bugfixes).is_some()
        });

        transformer_options.es2017.with_async_to_generator({
            let plugin_name = "transform-async-to-generator";
            get_enabled_plugin_options(plugin_name, options, targets.as_ref(), bugfixes).is_some()
        });

        transformer_options.es2018.with_object_rest_spread({
            let plugin_name = "transform-object-rest-spread";
            get_enabled_plugin_options(plugin_name, options, targets.as_ref(), bugfixes).map(
                |options| {
                    from_value::<ObjectRestSpreadOptions>(options).unwrap_or_else(|err| {
                        report_error(plugin_name, &err, false, &mut errors);
                        ObjectRestSpreadOptions::default()
                    })
                },
            )
        });

        transformer_options.es2019.with_optional_catch_binding({
            let plugin_name = "transform-optional-catch-binding";
            get_enabled_plugin_options(plugin_name, options, targets.as_ref(), bugfixes).is_some()
        });

        transformer_options.es2020.with_nullish_coalescing_operator({
            let plugin_name = "transform-nullish-coalescing-operator";
            get_enabled_plugin_options(plugin_name, options, targets.as_ref(), bugfixes).is_some()
        });

        transformer_options.es2021.with_logical_assignment_operators({
            let plugin_name = "transform-logical-assignment-operators";
            get_enabled_plugin_options(plugin_name, options, targets.as_ref(), bugfixes).is_some()
        });

        transformer_options.typescript = {
            let preset_name = "typescript";
            if options.has_preset("typescript") {
                from_value::<TypeScriptOptions>(
                    get_preset_options("typescript", options).unwrap_or_else(|| json!({})),
                )
                .unwrap_or_else(|err| {
                    report_error(preset_name, &err, true, &mut errors);
                    TypeScriptOptions::default()
                })
            } else {
                let plugin_name = "transform-typescript";
                from_value::<TypeScriptOptions>(get_plugin_options(plugin_name, options))
                    .unwrap_or_else(|err| {
                        report_error(plugin_name, &err, false, &mut errors);
                        TypeScriptOptions::default()
                    })
            }
        };

        let regexp = transformer_options.regexp;
        if !regexp.sticky_flag {
            transformer_options.regexp.sticky_flag = options.has_plugin("transform-sticky-regex");
        }
        if !regexp.unicode_flag {
            transformer_options.regexp.unicode_flag = options.has_plugin("transform-unicode-regex");
        }
        if !regexp.dot_all_flag {
            transformer_options.regexp.dot_all_flag = options.has_plugin("transform-dotall-regex");
        }
        if !regexp.named_capture_groups {
            transformer_options.regexp.named_capture_groups =
                options.has_plugin("transform-named-capturing-groups-regex");
        }
        if !regexp.unicode_property_escapes {
            transformer_options.regexp.unicode_property_escapes =
                options.has_plugin("transform-unicode-property-regex");
        }
        if !regexp.set_notation {
            transformer_options.regexp.set_notation =
                options.has_plugin("transform-unicode-sets-regex");
        }

        transformer_options.assumptions = if options.assumptions.is_null() {
            CompilerAssumptions::default()
        } else {
            match serde_json::from_value::<CompilerAssumptions>(options.assumptions.clone()) {
                Ok(value) => value,
                Err(err) => {
                    errors.push(OxcDiagnostic::error(err.to_string()).into());
                    CompilerAssumptions::default()
                }
            }
        };

        if options.external_helpers {
            transformer_options.helper_loader.mode = HelperLoaderMode::External;
        }

        transformer_options.cwd = options.cwd.clone().unwrap_or_default();

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(transformer_options)
    }
}

fn get_plugin_options(name: &str, babel_options: &BabelOptions) -> Value {
    let plugin = babel_options.get_plugin(name);
    plugin.and_then(|options| options).unwrap_or_else(|| json!({}))
}

fn get_preset_options(name: &str, babel_options: &BabelOptions) -> Option<Value> {
    let preset = babel_options.get_preset(name);
    preset.and_then(|options| options)
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
    let result = TransformOptions::from_babel_options(&babel_options);
    assert!(result.is_err());
    let err_message =
        result.err().unwrap().iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
    assert!(err_message.contains("transform-react-jsx: unknown field `filter`"));
}
