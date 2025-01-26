use std::str::FromStr;

use napi::Either;
use napi_derive::napi;

use oxc_sourcemap::napi::SourceMap;
use oxc_syntax::es_target::ESTarget;

#[napi(object)]
pub struct CompressOptions {
    /// Enables optional catch or nullish-coalescing operator if targeted higher.
    ///
    /// @default 'es2015'
    pub target: Option<String>,

    /// Pass true to discard calls to `console.*`.
    ///
    /// @default false
    pub drop_console: Option<bool>,

    /// Remove `debugger;` statements.
    ///
    /// @default true
    pub drop_debugger: Option<bool>,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self { target: None, drop_console: None, drop_debugger: Some(true) }
    }
}

impl TryFrom<&CompressOptions> for oxc_minifier::CompressOptions {
    type Error = String;
    fn try_from(o: &CompressOptions) -> Result<Self, Self::Error> {
        Ok(oxc_minifier::CompressOptions {
            target: o
                .target
                .as_ref()
                .map(|s| ESTarget::from_str(s))
                .transpose()?
                .unwrap_or(ESTarget::ES2015),
            drop_debugger: o.drop_debugger.unwrap_or(false),
            drop_console: o.drop_console.unwrap_or(true),
        })
    }
}

#[napi(object)]
#[derive(Default)]
pub struct MangleOptions {
    /// Pass true to mangle names declared in the top level scope.
    pub toplevel: Option<bool>,

    /// Debug mangled names.
    pub debug: Option<bool>,
}

impl From<&MangleOptions> for oxc_minifier::MangleOptions {
    fn from(o: &MangleOptions) -> Self {
        Self { top_level: o.toplevel.unwrap_or(false), debug: o.debug.unwrap_or(false) }
    }
}

#[napi(object)]
pub struct CodegenOptions {
    /// Remove whitespace.
    ///
    /// @default true
    pub whitespace: Option<bool>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self { whitespace: Some(true) }
    }
}

impl From<&CodegenOptions> for oxc_codegen::CodegenOptions {
    fn from(o: &CodegenOptions) -> Self {
        oxc_codegen::CodegenOptions {
            minify: o.whitespace.unwrap_or(true),
            ..oxc_codegen::CodegenOptions::default()
        }
    }
}

#[napi(object)]
#[derive(Default)]
pub struct MinifyOptions {
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

#[napi(object)]
pub struct MinifyResult {
    pub code: String,

    pub map: Option<SourceMap>,
}
