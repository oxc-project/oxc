pub mod babel;
mod env;

use std::path::PathBuf;

use env::EnvOptions;
use oxc_diagnostics::{Error, OxcDiagnostic};

use crate::{
    common::helper_loader::{HelperLoaderMode, HelperLoaderOptions},
    compiler_assumptions::CompilerAssumptions,
    jsx::JsxOptions,
    typescript::TypeScriptOptions,
    ReactRefreshOptions,
};

use babel::BabelOptions;

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

    /// ECMAScript Env Options
    pub env: EnvOptions,

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
            env: EnvOptions::enable_all(),
            helper_loader: HelperLoaderOptions {
                mode: HelperLoaderMode::Runtime,
                ..Default::default()
            },
        }
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
            options.get_preset("typescript").and_then(|options| {
                options
                    .map(|options| {
                        serde_json::from_value::<TypeScriptOptions>(options)
                            .inspect_err(|err| report_error("typescript", err, true, &mut errors))
                            .ok()
                    })
                    .unwrap_or_default()
            })
        } else {
            options.get_plugin("transform-typescript").and_then(|options| {
                options
                    .map(|options| {
                        serde_json::from_value::<TypeScriptOptions>(options)
                            .inspect_err(|err| report_error("typescript", err, false, &mut errors))
                            .ok()
                    })
                    .unwrap_or_default()
            })
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
                options.get_plugin(jsx_plugin_name).and_then(|options| {
                    options.and_then(|options| {
                        serde_json::from_value::<JsxOptions>(options)
                            .inspect_err(|err| {
                                report_error(jsx_plugin_name, err, false, &mut errors);
                            })
                            .ok()
                    })
                })
            } else {
                options.get_plugin(jsx_dev_name).and_then(|options| {
                    options.and_then(|options| {
                        serde_json::from_value::<JsxOptions>(options)
                            .inspect_err(|err| report_error(jsx_dev_name, err, false, &mut errors))
                            .ok()
                    })
                })
            }
            .unwrap_or_default();
            react_options.development = options.has_plugin(jsx_dev_name);
            react_options.jsx_plugin = has_jsx_plugin;
            react_options.display_name_plugin = options.has_plugin("transform-react-display-name");
            react_options.jsx_self_plugin = options.has_plugin("transform-react-jsx-self");
            react_options.jsx_source_plugin = options.has_plugin("transform-react-jsx-source");
            react_options
        };

        let env = match EnvOptions::try_from(options) {
            Ok(env) => Some(env),
            Err(errs) => {
                errors.extend(errs);
                None
            }
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
            env: env.unwrap_or_default(),
            helper_loader,
        })
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
