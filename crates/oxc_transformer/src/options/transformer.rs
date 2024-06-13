use std::path::PathBuf;

use oxc_diagnostics::{Error, OxcDiagnostic};
use serde_json::{from_value, json, Value};

use crate::{
    compiler_assumptions::CompilerAssumptions,
    env::{can_enable_plugin, EnvOptions, Versions},
    es2015::{ArrowFunctionsOptions, ES2015Options},
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
}

impl TransformOptions {
    /// # Errors
    ///
    pub fn from_babel_options(options: &BabelOptions) -> Result<Self, Vec<Error>> {
        let mut errors = Vec::<Error>::new();

        let env_options = {
            let preset_name = "env";
            from_value::<EnvOptions>(get_preset_options(preset_name, options)).unwrap_or_else(
                |err| {
                    report_error(preset_name, &err, true, &mut errors);
                    EnvOptions::default()
                },
            )
        };
        let targets = match env_options.get_targets() {
            Ok(t) => t,
            Err(err) => {
                errors.push(OxcDiagnostic::error(err.to_string()).into());
                return Err(errors);
            }
        };

        let preset_name = "react";
        let react = if options.has_preset(preset_name) {
            from_value::<ReactOptions>(get_preset_options(preset_name, options)).unwrap_or_else(
                |err| {
                    report_error(preset_name, &err, true, &mut errors);
                    ReactOptions::default()
                },
            )
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

        let es2015 = ES2015Options::default().with_arrow_function({
            let plugin_name = "transform-arrow-functions";
            enable_plugin(plugin_name, options, &env_options, &targets).map(|options| {
                from_value::<ArrowFunctionsOptions>(options).unwrap_or_else(|err| {
                    report_error(plugin_name, &err, false, &mut errors);
                    ArrowFunctionsOptions::default()
                })
            })
        });

        let typescript = {
            let plugin_name = "transform-typescript";
            from_value::<TypeScriptOptions>(get_plugin_options(plugin_name, options))
                .unwrap_or_else(|err| {
                    report_error(plugin_name, &err, false, &mut errors);
                    TypeScriptOptions::default()
                })
        };

        let assumptions = if options.assumptions.is_null() {
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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self {
            cwd: options.cwd.clone().unwrap_or_default(),
            assumptions,
            typescript,
            react,
            es2015,
        })
    }
}

fn get_plugin_options(name: &str, babel_options: &BabelOptions) -> Value {
    let plugin = babel_options.get_plugin(name);
    plugin.and_then(|options| options).unwrap_or_else(|| json!({}))
}

fn get_preset_options(name: &str, babel_options: &BabelOptions) -> Value {
    let preset = babel_options.get_preset(name);
    preset.and_then(|options| options).unwrap_or_else(|| json!({}))
}

fn enable_plugin(
    plugin_name: &str,
    babel_options: &BabelOptions,
    env_options: &EnvOptions,
    targets: &Versions,
) -> Option<Value> {
    let can_enable = can_enable_plugin(plugin_name, targets, env_options.bugfixes)
        || babel_options.has_plugin(plugin_name);

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
