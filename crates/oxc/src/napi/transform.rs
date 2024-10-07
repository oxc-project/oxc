// NOTE: Types must be aligned with [@types/babel__core](https://github.com/DefinitelyTyped/DefinitelyTyped/blob/master/types/babel__core/index.d.ts).

#![allow(rustdoc::bare_urls)]

use std::path::PathBuf;

use napi::Either;
use napi_derive::napi;
use rustc_hash::FxHashMap;

use oxc_transformer::{JsxRuntime, RewriteExtensionsMode};

use super::{isolated_declarations::IsolatedDeclarationsOptions, source_map::SourceMap};

#[derive(Default)]
#[napi(object)]
pub struct TransformResult {
    /// The transformed code.
    ///
    /// If parsing failed, this will be an empty string.
    pub code: String,

    /// The source map for the transformed code.
    ///
    /// This will be set if {@link TransformOptions#sourcemap} is `true`.
    pub map: Option<SourceMap>,

    /// The `.d.ts` declaration file for the transformed code. Declarations are
    /// only generated if `declaration` is set to `true` and a TypeScript file
    /// is provided.
    ///
    /// If parsing failed and `declaration` is set, this will be an empty string.
    ///
    /// @see {@link TypeScriptOptions#declaration}
    /// @see [declaration tsconfig option](https://www.typescriptlang.org/tsconfig/#declaration)
    pub declaration: Option<String>,

    /// Declaration source map. Only generated if both
    /// {@link TypeScriptOptions#declaration declaration} and
    /// {@link TransformOptions#sourcemap sourcemap} are set to `true`.
    pub declaration_map: Option<SourceMap>,

    /// Parse and transformation errors.
    ///
    /// Oxc's parser recovers from common syntax errors, meaning that
    /// transformed code may still be available even if there are errors in this
    /// list.
    pub errors: Vec<String>,
}

/// Options for transforming a JavaScript or TypeScript file.
///
/// @see {@link transform}
#[napi(object)]
#[derive(Default)]
pub struct TransformOptions {
    #[napi(ts_type = "'script' | 'module' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,

    /// Treat the source text as `js`, `jsx`, `ts`, or `tsx`.
    #[napi(ts_type = "'js' | 'jsx' | 'ts' | 'tsx'")]
    pub lang: Option<String>,

    /// The current working directory. Used to resolve relative paths in other
    /// options.
    pub cwd: Option<String>,

    /// Enable source map generation.
    ///
    /// When `true`, the `sourceMap` field of transform result objects will be populated.
    ///
    /// @default false
    ///
    /// @see {@link SourceMap}
    pub sourcemap: Option<bool>,

    /// Configure how TypeScript is transformed.
    pub typescript: Option<TypeScriptOptions>,

    /// Configure how TSX and JSX are transformed.
    pub jsx: Option<JsxOptions>,

    /// Enable ES2015 transformations.
    pub es2015: Option<Es2015Options>,

    /// Define Plugin
    #[napi(ts_type = "Record<string, string>")]
    pub define: Option<FxHashMap<String, String>>,

    /// Inject Plugin
    #[napi(ts_type = "Record<string, string | [string, string]>")]
    pub inject: Option<FxHashMap<String, Either<String, Vec<String>>>>,
}

impl From<TransformOptions> for oxc_transformer::TransformOptions {
    fn from(options: TransformOptions) -> Self {
        Self {
            cwd: options.cwd.map(PathBuf::from).unwrap_or_default(),
            typescript: options.typescript.map(Into::into).unwrap_or_default(),
            react: options.jsx.map(Into::into).unwrap_or_default(),
            es2015: options.es2015.map(Into::into).unwrap_or_default(),
            ..Self::default()
        }
    }
}

#[napi(object)]
#[derive(Default)]
pub struct TypeScriptOptions {
    pub jsx_pragma: Option<String>,
    pub jsx_pragma_frag: Option<String>,
    pub only_remove_type_imports: Option<bool>,
    pub allow_namespaces: Option<bool>,
    pub allow_declare_fields: Option<bool>,
    /// Also generate a `.d.ts` declaration file for TypeScript files.
    ///
    /// The source file must be compliant with all
    /// [`isolatedDeclarations`](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-5.html#isolated-declarations)
    /// requirements.
    ///
    /// @default false
    pub declaration: Option<IsolatedDeclarationsOptions>,
    /// Rewrite or remove TypeScript import/export declaration extensions.
    ///
    /// - When set to `rewrite`, it will change `.ts`, `.mts`, `.cts` extensions to `.js`, `.mjs`, `.cjs` respectively.
    /// - When set to `remove`, it will remove `.ts`/`.mts`/`.cts`/`.tsx` extension entirely.
    /// - When set to `true`, it's equivalent to `rewrite`.
    /// - When set to `false` or omitted, no changes will be made to the extensions.
    ///
    /// @default false
    #[napi(ts_type = "'rewrite' | 'remove' | boolean")]
    pub rewrite_import_extensions: Option<Either<bool, String>>,
}

impl From<TypeScriptOptions> for oxc_transformer::TypeScriptOptions {
    fn from(options: TypeScriptOptions) -> Self {
        let ops = oxc_transformer::TypeScriptOptions::default();
        oxc_transformer::TypeScriptOptions {
            jsx_pragma: options.jsx_pragma.map(Into::into).unwrap_or(ops.jsx_pragma),
            jsx_pragma_frag: options.jsx_pragma_frag.map(Into::into).unwrap_or(ops.jsx_pragma_frag),
            only_remove_type_imports: options
                .only_remove_type_imports
                .unwrap_or(ops.only_remove_type_imports),
            allow_namespaces: options.allow_namespaces.unwrap_or(ops.allow_namespaces),
            allow_declare_fields: options.allow_declare_fields.unwrap_or(ops.allow_declare_fields),
            optimize_const_enums: false,
            rewrite_import_extensions: options.rewrite_import_extensions.and_then(|value| {
                match value {
                    Either::A(v) => {
                        if v {
                            Some(RewriteExtensionsMode::Rewrite)
                        } else {
                            None
                        }
                    }
                    Either::B(v) => match v.as_str() {
                        "rewrite" => Some(RewriteExtensionsMode::Rewrite),
                        "remove" => Some(RewriteExtensionsMode::Remove),
                        _ => None,
                    },
                }
            }),
        }
    }
}

/// Configure how TSX and JSX are transformed.
///
/// @see {@link https://babeljs.io/docs/babel-plugin-transform-react-jsx#options}
#[napi(object)]
pub struct JsxOptions {
    /// Decides which runtime to use.
    ///
    /// - 'automatic' - auto-import the correct JSX factories
    /// - 'classic' - no auto-import
    ///
    /// @default 'automatic'
    #[napi(ts_type = "'classic' | 'automatic'")]
    pub runtime: Option<String>,

    /// Emit development-specific information, such as `__source` and `__self`.
    ///
    /// @default false
    ///
    /// @see {@link https://babeljs.io/docs/babel-plugin-transform-react-jsx-development}
    pub development: Option<bool>,

    /// Toggles whether or not to throw an error if an XML namespaced tag name
    /// is used.
    ///
    /// Though the JSX spec allows this, it is disabled by default since React's
    /// JSX does not currently have support for it.
    ///
    /// @default true
    pub throw_if_namespace: Option<bool>,

    /// Enables `@babel/plugin-transform-react-pure-annotations`.
    ///
    /// It will mark top-level React method calls as pure for tree shaking.
    ///
    /// @see {@link https://babeljs.io/docs/en/babel-plugin-transform-react-pure-annotations}
    ///
    /// @default true
    pub pure: Option<bool>,

    /// Replaces the import source when importing functions.
    ///
    /// @default 'react'
    pub import_source: Option<String>,

    /// Replace the function used when compiling JSX expressions. It should be a
    /// qualified name (e.g. `React.createElement`) or an identifier (e.g.
    /// `createElement`).
    ///
    /// Only used for `classic` {@link runtime}.
    ///
    /// @default 'React.createElement'
    pub pragma: Option<String>,

    /// Replace the component used when compiling JSX fragments. It should be a
    /// valid JSX tag name.
    ///
    /// Only used for `classic` {@link runtime}.
    ///
    /// @default 'React.Fragment'
    pub pragma_frag: Option<String>,

    /// When spreading props, use `Object.assign` directly instead of an extend helper.
    ///
    /// Only used for `classic` {@link runtime}.
    ///
    /// @default false
    pub use_built_ins: Option<bool>,

    /// When spreading props, use inline object with spread elements directly
    /// instead of an extend helper or Object.assign.
    ///
    /// Only used for `classic` {@link runtime}.
    ///
    /// @default false
    pub use_spread: Option<bool>,

    /// Enable React Fast Refresh .
    ///
    /// Conforms to the implementation in {@link https://github.com/facebook/react/tree/main/packages/react-refresh}
    ///
    /// @default false
    pub refresh: Option<Either<bool, ReactRefreshOptions>>,
}

impl From<JsxOptions> for oxc_transformer::JsxOptions {
    fn from(options: JsxOptions) -> Self {
        let ops = oxc_transformer::JsxOptions::default();
        oxc_transformer::JsxOptions {
            runtime: match options.runtime.as_deref() {
                Some("classic") => JsxRuntime::Classic,
                /* "automatic" */ _ => JsxRuntime::Automatic,
            },
            development: options.development.unwrap_or(ops.development),
            throw_if_namespace: options.throw_if_namespace.unwrap_or(ops.throw_if_namespace),
            pure: options.pure.unwrap_or(ops.pure),
            import_source: options.import_source,
            pragma: options.pragma,
            pragma_frag: options.pragma_frag,
            use_built_ins: options.use_built_ins,
            use_spread: options.use_spread,
            refresh: options.refresh.and_then(|value| match value {
                Either::A(b) => b.then(oxc_transformer::ReactRefreshOptions::default),
                Either::B(options) => Some(oxc_transformer::ReactRefreshOptions::from(options)),
            }),
            ..Default::default()
        }
    }
}

#[napi(object)]
pub struct ReactRefreshOptions {
    /// Specify the identifier of the refresh registration variable.
    ///
    /// @default `$RefreshReg$`.
    pub refresh_reg: Option<String>,

    /// Specify the identifier of the refresh signature variable.
    ///
    /// @default `$RefreshSig$`.
    pub refresh_sig: Option<String>,

    pub emit_full_signatures: Option<bool>,
}

impl From<ReactRefreshOptions> for oxc_transformer::ReactRefreshOptions {
    fn from(options: ReactRefreshOptions) -> Self {
        let ops = oxc_transformer::ReactRefreshOptions::default();
        oxc_transformer::ReactRefreshOptions {
            refresh_reg: options.refresh_reg.unwrap_or(ops.refresh_reg),
            refresh_sig: options.refresh_sig.unwrap_or(ops.refresh_sig),
            emit_full_signatures: options.emit_full_signatures.unwrap_or(ops.emit_full_signatures),
        }
    }
}

#[napi(object)]
pub struct ArrowFunctionsOptions {
    /// This option enables the following:
    /// * Wrap the generated function in .bind(this) and keeps uses of this inside the function as-is, instead of using a renamed this.
    /// * Add a runtime check to ensure the functions are not instantiated.
    /// * Add names to arrow functions.
    ///
    /// @default false
    pub spec: Option<bool>,
}

impl From<ArrowFunctionsOptions> for oxc_transformer::ArrowFunctionsOptions {
    fn from(options: ArrowFunctionsOptions) -> Self {
        oxc_transformer::ArrowFunctionsOptions { spec: options.spec.unwrap_or_default() }
    }
}

#[napi(object)]
pub struct Es2015Options {
    /// Transform arrow functions into function expressions.
    pub arrow_function: Option<ArrowFunctionsOptions>,
}

impl From<Es2015Options> for oxc_transformer::ES2015Options {
    fn from(options: Es2015Options) -> Self {
        oxc_transformer::ES2015Options { arrow_function: options.arrow_function.map(Into::into) }
    }
}
