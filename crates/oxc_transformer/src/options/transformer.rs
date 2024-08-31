use std::path::PathBuf;

use oxc_diagnostics::{Error, OxcDiagnostic};
use serde_json::{from_value, json, Value};

use crate::{
    compiler_assumptions::CompilerAssumptions,
    env::{can_enable_plugin, EnvOptions, Versions},
    es2015::{ArrowFunctionsOptions, ES2015Options},
    es2016::ES2016Options,
    es2018::{ES2018Options, ObjectRestSpreadOptions},
    es2019::ES2019Options,
    es2020::ES2020Options,
    es2021::ES2021Options,
    options::babel::BabelOptions,
    react::ReactOptions,
    typescript::TypeScriptOptions,
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
    pub react: ReactOptions,

    pub es2015: ES2015Options,

    pub es2016: ES2016Options,

    pub es2018: ES2018Options,

    pub es2019: ES2019Options,

    pub es2020: ES2020Options,

    pub es2021: ES2021Options,
}

impl TransformOptions {
    fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            es2015: ES2015Options::from_targets_and_bugfixes(targets, bugfixes),
            es2016: ES2016Options::from_targets_and_bugfixes(targets, bugfixes),
            es2018: ES2018Options::from_targets_and_bugfixes(targets, bugfixes),
            es2019: ES2019Options::from_targets_and_bugfixes(targets, bugfixes),
            es2020: ES2020Options::from_targets_and_bugfixes(targets, bugfixes),
            es2021: ES2021Options::from_targets_and_bugfixes(targets, bugfixes),
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
            match from_value::<ReactOptions>(value) {
                Ok(res) => res,
                Err(err) => {
                    report_error(preset_name, &err, true, &mut errors);
                    ReactOptions::default()
                }
            }
        } else {
            let has_jsx_plugin = options.has_plugin("transform-react-jsx");
            let has_jsx_development_plugin = options.has_plugin("transform-react-jsx-development");
            let mut react_options =
                if has_jsx_plugin {
                    let plugin_name = "transform-react-jsx";
                    from_value::<ReactOptions>(get_plugin_options(plugin_name, options))
                        .unwrap_or_else(|err| {
                            report_error(plugin_name, &err, false, &mut errors);
                            ReactOptions::default()
                        })
                } else {
                    let plugin_name = "transform-react-jsx-development";
                    from_value::<ReactOptions>(get_plugin_options(plugin_name, options))
                        .unwrap_or_else(|err| {
                            report_error(plugin_name, &err, false, &mut errors);
                            ReactOptions::default()
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
            let plugin_name = "transform-typescript";
            from_value::<TypeScriptOptions>(get_plugin_options(plugin_name, options))
                .unwrap_or_else(|err| {
                    report_error(plugin_name, &err, false, &mut errors);
                    TypeScriptOptions::default()
                })
        };

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
