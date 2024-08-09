use std::path::PathBuf;

use napi_derive::napi;

use oxc_transformer::{
    ArrowFunctionsOptions, ES2015Options, ReactJsxRuntime, ReactOptions, TypeScriptOptions,
};

#[napi(object)]
#[derive(Default)]
pub struct TypeScriptBindingOptions {
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
    pub declaration: Option<bool>,
}

impl From<TypeScriptBindingOptions> for TypeScriptOptions {
    fn from(options: TypeScriptBindingOptions) -> Self {
        let ops = TypeScriptOptions::default();
        TypeScriptOptions {
            jsx_pragma: options.jsx_pragma.map(Into::into).unwrap_or(ops.jsx_pragma),
            jsx_pragma_frag: options.jsx_pragma_frag.map(Into::into).unwrap_or(ops.jsx_pragma_frag),
            only_remove_type_imports: options
                .only_remove_type_imports
                .unwrap_or(ops.only_remove_type_imports),
            allow_namespaces: options.allow_namespaces.unwrap_or(ops.allow_namespaces),
            allow_declare_fields: options.allow_declare_fields.unwrap_or(ops.allow_declare_fields),
        }
    }
}

/// Configure how TSX and JSX are transformed.
///
/// @see [@babel/plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx#options)
#[napi(object)]
pub struct ReactBindingOptions {
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
    /// @see [@babel/plugin-transform-react-jsx-development](https://babeljs.io/docs/babel-plugin-transform-react-jsx-development)
    pub development: Option<bool>,

    /// Toggles whether or not to throw an error if an XML namespaced tag name
    /// is used.
    ///
    /// Though the JSX spec allows this, it is disabled by default since React's
    /// JSX does not currently have support for it.
    ///
    /// @default true
    pub throw_if_namespace: Option<bool>,

    /// Enables [@babel/plugin-transform-react-pure-annotations](https://babeljs.io/docs/en/babel-plugin-transform-react-pure-annotations).
    ///
    /// It will mark top-level React method calls as pure for tree shaking.
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
}

impl From<ReactBindingOptions> for ReactOptions {
    fn from(options: ReactBindingOptions) -> Self {
        let ops = ReactOptions::default();
        ReactOptions {
            runtime: match options.runtime.as_deref() {
                Some("classic") => ReactJsxRuntime::Classic,
                /* "automatic" */ _ => ReactJsxRuntime::Automatic,
            },
            development: options.development.unwrap_or(ops.development),
            throw_if_namespace: options.throw_if_namespace.unwrap_or(ops.throw_if_namespace),
            pure: options.pure.unwrap_or(ops.pure),
            import_source: options.import_source,
            pragma: options.pragma,
            pragma_frag: options.pragma_frag,
            use_built_ins: options.use_built_ins,
            use_spread: options.use_spread,
            ..Default::default()
        }
    }
}

#[napi(object)]
pub struct ArrowFunctionsBindingOptions {
    /// This option enables the following:
    /// * Wrap the generated function in .bind(this) and keeps uses of this inside the function as-is, instead of using a renamed this.
    /// * Add a runtime check to ensure the functions are not instantiated.
    /// * Add names to arrow functions.
    ///
    /// @default false
    pub spec: Option<bool>,
}

impl From<ArrowFunctionsBindingOptions> for ArrowFunctionsOptions {
    fn from(options: ArrowFunctionsBindingOptions) -> Self {
        ArrowFunctionsOptions { spec: options.spec.unwrap_or_default() }
    }
}

#[napi(object)]
pub struct ES2015BindingOptions {
    /// Transform arrow functions into function expressions.
    pub arrow_function: Option<ArrowFunctionsBindingOptions>,
}

impl From<ES2015BindingOptions> for ES2015Options {
    fn from(options: ES2015BindingOptions) -> Self {
        ES2015Options { arrow_function: options.arrow_function.map(Into::into) }
    }
}

/// Options for transforming a JavaScript or TypeScript file.
///
/// @see {@link transform}
#[napi(object)]
pub struct TransformOptions {
    #[napi(ts_type = "'script' | 'module' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,

    /// The current working directory. Used to resolve relative paths in other
    /// options.
    pub cwd: Option<String>,

    /// Force jsx parsing,
    ///
    /// @default false
    pub jsx: Option<bool>,

    /// Configure how TypeScript is transformed.
    pub typescript: Option<TypeScriptBindingOptions>,

    /// Configure how TSX and JSX are transformed.
    pub react: Option<ReactBindingOptions>,

    /// Enable ES2015 transformations.
    pub es2015: Option<ES2015BindingOptions>,

    /// Enable source map generation.
    ///
    /// When `true`, the `sourceMap` field of transform result objects will be populated.
    ///
    /// @default false
    ///
    /// @see {@link SourceMap}
    pub sourcemap: Option<bool>,

    // todo: should we use oxc_minifier here, or make it a separate
    // function/package?
    /// Enable minification.
    ///
    /// Currently, minification will only remove comments and unnecessary
    /// whitespace. Declaration files will not be minified at all.
    ///
    /// @default false
    pub minify: Option<bool>,
}

impl From<TransformOptions> for oxc_transformer::TransformOptions {
    fn from(options: TransformOptions) -> Self {
        Self {
            cwd: options.cwd.map(PathBuf::from).unwrap_or_default(),
            typescript: options.typescript.map(Into::into).unwrap_or_default(),
            react: options.react.map(Into::into).unwrap_or_default(),
            es2015: options.es2015.map(Into::into).unwrap_or_default(),
            ..Self::default()
        }
    }
}
