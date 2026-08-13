use std::path::PathBuf;

use napi_derive::napi;

use oxc::diagnostics::OxcDiagnostic;
use oxc_relay::{RelayLanguage, RelayOptions};

/// Options for the Relay transform.
///
/// `lang`, `sourceType`, and `sourcemap` configure the surrounding Oxc
/// parse/codegen pipeline; the remaining fields mirror the options of
/// `babel-plugin-relay` / `@swc/plugin-relay`.
#[napi(object)]
#[derive(Default, Debug)]
pub struct TransformOptions {
    /// Treat the source as `js`, `jsx`, `ts`, `tsx`, or `dts`.
    #[napi(ts_type = "'js' | 'jsx' | 'ts' | 'tsx' | 'dts'")]
    pub lang: Option<String>,

    /// Treat the source as script, module, CommonJS, or infer it from syntax.
    #[napi(ts_type = "'script' | 'module' | 'commonjs' | 'unambiguous'")]
    pub source_type: Option<String>,

    /// Generate a source map.
    ///
    /// @default false
    pub sourcemap: Option<bool>,

    /// Directory `relay-compiler` emits all artifacts to (its
    /// `artifactDirectory` setting). When set, artifacts are imported via a
    /// relative path from the file being transformed to this directory; the
    /// path is computed lexically, so both must either be absolute or relative
    /// to the same base directory. When unset, artifacts are imported from the
    /// `__generated__` directory next to the file being transformed.
    pub artifact_directory: Option<String>,

    /// Artifact language, determining the imported file extension:
    /// `Name.graphql.ts` for `typescript`, `Name.graphql.js` otherwise.
    ///
    /// @default 'javascript'
    #[napi(ts_type = "'typescript' | 'javascript' | 'flow'")]
    pub language: Option<String>,

    /// Emit a hoisted default import per `graphql` tag instead of an inline
    /// `require()` call.
    ///
    /// Defaults to `true`, matching `babel-plugin-relay` since Relay v17.
    /// `@swc/plugin-relay` and Next.js default to `false`.
    ///
    /// @default true
    pub eager_es_modules: Option<bool>,
}

impl TransformOptions {
    pub(crate) fn resolve(self) -> Result<RelayOptions, OxcDiagnostic> {
        let language = match self.language.as_deref() {
            None => RelayLanguage::default(),
            Some("typescript") => RelayLanguage::Typescript,
            Some("javascript") => RelayLanguage::Javascript,
            Some("flow") => RelayLanguage::Flow,
            Some(value) => {
                return Err(OxcDiagnostic::error(format!("Invalid `language` option: `{value}`.")));
            }
        };
        Ok(RelayOptions {
            artifact_directory: self.artifact_directory.map(PathBuf::from),
            language,
            eager_es_modules: self.eager_es_modules.unwrap_or(true),
        })
    }
}
