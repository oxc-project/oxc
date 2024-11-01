pub mod babel;
mod env;

use std::path::PathBuf;

use env::EnvOptions;
use oxc_diagnostics::Error;

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
        errors.extend(options.plugins.errors.iter().map(|err| Error::msg(err.clone())));
        errors.extend(options.presets.errors.iter().map(|err| Error::msg(err.clone())));

        let assumptions = if options.assumptions.is_null() {
            CompilerAssumptions::default()
        } else {
            serde_json::from_value::<CompilerAssumptions>(options.assumptions.clone())
                .map_err(|err| errors.push(Error::msg(err)))
                .unwrap_or_default()
        };

        let typescript = options
            .presets
            .typescript
            .clone()
            .or_else(|| options.plugins.typescript.clone())
            .unwrap_or_default();

        let jsx = if let Some(options) = &options.presets.jsx {
            options.clone()
        } else {
            let mut jsx_options = if let Some(options) = &options.plugins.react_jsx_dev {
                options.clone()
            } else if let Some(options) = &options.plugins.react_jsx {
                options.clone()
            } else {
                JsxOptions::default()
            };
            jsx_options.development = options.plugins.react_jsx_dev.is_some();
            jsx_options.jsx_plugin = options.plugins.react_jsx.is_some();
            jsx_options.display_name_plugin = options.plugins.react_display_name;
            jsx_options.jsx_self_plugin = options.plugins.react_jsx_self;
            jsx_options.jsx_source_plugin = options.plugins.react_jsx_source;
            jsx_options
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
