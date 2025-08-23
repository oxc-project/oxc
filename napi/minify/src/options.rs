use std::str::FromStr;

use napi::Either;
use napi_derive::napi;

use oxc_minifier::TreeShakeOptions;
use oxc_syntax::es_target::ESTarget;

#[napi(object)]
pub struct CompressOptions {
    /// Set desired EcmaScript standard version for output.
    ///
    /// Set `esnext` to enable all target highering.
    ///
    /// e.g.
    ///
    /// * catch optional binding when >= es2019
    /// * `??` operator >= es2020
    ///
    /// @default 'esnext'
    #[napi(
        ts_type = "'esnext' | 'es2015' | 'es2016' | 'es2017' | 'es2018' | 'es2019' | 'es2020' | 'es2021' | 'es2022' | 'es2023' | 'es2024'"
    )]
    pub target: Option<String>,

    /// Pass true to discard calls to `console.*`.
    ///
    /// @default false
    pub drop_console: Option<bool>,

    /// Remove `debugger;` statements.
    ///
    /// @default true
    pub drop_debugger: Option<bool>,

    /// Drop unreferenced functions and variables.
    ///
    /// Simple direct variable assignments do not count as references unless set to "keep_assign".
    #[napi(ts_type = "true | false | 'keep_assign'")]
    pub unused: Option<String>,

    /// Keep function / class names.
    pub keep_names: Option<CompressOptionsKeepNames>,
}

impl TryFrom<&CompressOptions> for oxc_minifier::CompressOptions {
    type Error = String;
    fn try_from(o: &CompressOptions) -> Result<Self, Self::Error> {
        let default = oxc_minifier::CompressOptions::default();
        Ok(oxc_minifier::CompressOptions {
            target: o
                .target
                .as_ref()
                .map(|s| ESTarget::from_str(s))
                .transpose()?
                .unwrap_or(default.target),
            drop_console: o.drop_console.unwrap_or(default.drop_console),
            drop_debugger: o.drop_debugger.unwrap_or(default.drop_debugger),
            // TODO
            join_vars: true,
            sequences: true,
            // TODO
            unused: oxc_minifier::CompressOptionsUnused::Keep,
            keep_names: o.keep_names.as_ref().map(Into::into).unwrap_or_default(),
            treeshake: TreeShakeOptions::default(),
        })
    }
}

#[napi(object)]
pub struct CompressOptionsKeepNames {
    /// Keep function names so that `Function.prototype.name` is preserved.
    ///
    /// This does not guarantee that the `undefined` name is preserved.
    ///
    /// @default false
    pub function: bool,

    /// Keep class names so that `Class.prototype.name` is preserved.
    ///
    /// This does not guarantee that the `undefined` name is preserved.
    ///
    /// @default false
    pub class: bool,
}

impl From<&CompressOptionsKeepNames> for oxc_minifier::CompressOptionsKeepNames {
    fn from(o: &CompressOptionsKeepNames) -> Self {
        oxc_minifier::CompressOptionsKeepNames { function: o.function, class: o.class }
    }
}

#[napi(object)]
#[derive(Default)]
pub struct MangleOptions {
    /// Pass `true` to mangle names declared in the top level scope.
    ///
    /// @default false
    pub toplevel: Option<bool>,

    /// Preserve `name` property for functions and classes.
    ///
    /// @default false
    pub keep_names: Option<Either<bool, MangleOptionsKeepNames>>,

    /// Debug mangled names.
    pub debug: Option<bool>,
}

impl From<&MangleOptions> for oxc_minifier::MangleOptions {
    fn from(o: &MangleOptions) -> Self {
        let default = oxc_minifier::MangleOptions::default();
        Self {
            top_level: o.toplevel.unwrap_or(default.top_level),
            keep_names: match &o.keep_names {
                Some(Either::A(false)) => oxc_minifier::MangleOptionsKeepNames::all_false(),
                Some(Either::A(true)) => oxc_minifier::MangleOptionsKeepNames::all_true(),
                Some(Either::B(o)) => oxc_minifier::MangleOptionsKeepNames::from(o),
                None => default.keep_names,
            },
            debug: o.debug.unwrap_or(default.debug),
        }
    }
}

#[napi(object)]
pub struct MangleOptionsKeepNames {
    /// Preserve `name` property for functions.
    ///
    /// @default false
    pub function: bool,

    /// Preserve `name` property for classes.
    ///
    /// @default false
    pub class: bool,
}

impl From<&MangleOptionsKeepNames> for oxc_minifier::MangleOptionsKeepNames {
    fn from(o: &MangleOptionsKeepNames) -> Self {
        oxc_minifier::MangleOptionsKeepNames { function: o.function, class: o.class }
    }
}

#[napi(object)]
pub struct CodegenOptions {
    /// Remove whitespace.
    ///
    /// @default true
    pub remove_whitespace: Option<bool>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self { remove_whitespace: Some(true) }
    }
}

impl From<&CodegenOptions> for oxc_codegen::CodegenOptions {
    fn from(o: &CodegenOptions) -> Self {
        if o.remove_whitespace.is_some_and(|b| b) {
            oxc_codegen::CodegenOptions::minify()
        } else {
            // Need to remove all comments.
            oxc_codegen::CodegenOptions { minify: false, ..oxc_codegen::CodegenOptions::minify() }
        }
    }
}

#[napi(object)]
#[derive(Default)]
pub struct MinifyOptions {
    /// Use when minifying an ES6 module.
    pub module: Option<bool>,

    pub compress: Option<Either<bool, CompressOptions>>,

    pub mangle: Option<Either<bool, MangleOptions>>,

    pub codegen: Option<Either<bool, CodegenOptions>>,

    pub sourcemap: Option<bool>,
}

impl TryFrom<&MinifyOptions> for oxc_minifier::MinifierOptions {
    type Error = String;

    fn try_from(o: &MinifyOptions) -> Result<Self, Self::Error> {
        let compress = match &o.compress {
            Some(Either::A(false)) => None,
            None | Some(Either::A(true)) => Some(oxc_minifier::CompressOptions::default()),
            Some(Either::B(o)) => Some(oxc_minifier::CompressOptions::try_from(o)?),
        };
        let mangle = match &o.mangle {
            Some(Either::A(false)) => None,
            None | Some(Either::A(true)) => Some(oxc_minifier::MangleOptions::default()),
            Some(Either::B(o)) => Some(oxc_minifier::MangleOptions::from(o)),
        };
        Ok(oxc_minifier::MinifierOptions { compress, mangle })
    }
}
