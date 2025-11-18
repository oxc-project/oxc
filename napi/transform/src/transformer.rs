// NOTE: Types must be aligned with [@types/babel__core](https://github.com/DefinitelyTyped/DefinitelyTyped/blob/b5dc32740d9b45d11cff9b025896dd333c795b39/types/babel__core/index.d.ts).
#![expect(rustdoc::bare_urls)]

use std::{
    ops::ControlFlow,
    path::{Path, PathBuf},
};

use napi::{Either, Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;
use rustc_hash::FxHashMap;

use oxc::{
    CompilerInterface,
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions, CodegenReturn},
    diagnostics::OxcDiagnostic,
    parser::Parser,
    semantic::{SemanticBuilder, SemanticBuilderReturn},
    span::SourceType,
    transformer::{
        EnvOptions, HelperLoaderMode, HelperLoaderOptions, JsxRuntime, ProposalOptions,
        RewriteExtensionsMode,
    },
    transformer_plugins::{
        InjectGlobalVariablesConfig, InjectImport, ModuleRunnerTransform,
        ReplaceGlobalDefinesConfig,
    },
};
use oxc_napi::{OxcError, get_source_type};
use oxc_sourcemap::napi::SourceMap;

use crate::IsolatedDeclarationsOptions;

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

    /// Helpers used.
    ///
    /// @internal
    ///
    /// Example:
    ///
    /// ```text
    /// { "_objectSpread": "@oxc-project/runtime/helpers/objectSpread2" }
    /// ```
    #[napi(ts_type = "Record<string, string>")]
    pub helpers_used: FxHashMap<String, String>,

    /// Parse and transformation errors.
    ///
    /// Oxc's parser recovers from common syntax errors, meaning that
    /// transformed code may still be available even if there are errors in this
    /// list.
    pub errors: Vec<OxcError>,
}

/// Options for transforming a JavaScript or TypeScript file.
///
/// @see {@link transform}
#[napi(object)]
#[derive(Default)]
pub struct TransformOptions {
    /// Treat the source text as `js`, `jsx`, `ts`, `tsx`, or `dts`.
    #[napi(ts_type = "'js' | 'jsx' | 'ts' | 'tsx' | 'dts'")]
    pub lang: Option<String>,

    /// Treat the source text as `script` or `module` code.
    #[napi(ts_type = "'script' | 'module' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,

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

    /// Set assumptions in order to produce smaller output.
    pub assumptions: Option<CompilerAssumptions>,

    /// Configure how TypeScript is transformed.
    pub typescript: Option<TypeScriptOptions>,

    /// Configure how TSX and JSX are transformed.
    #[napi(ts_type = "'preserve' | JsxOptions")]
    pub jsx: Option<Either<String, JsxOptions>>,

    /// Sets the target environment for the generated JavaScript.
    ///
    /// The lowest target is `es2015`.
    ///
    /// Example:
    ///
    /// * `'es2015'`
    /// * `['es2020', 'chrome58', 'edge16', 'firefox57', 'node12', 'safari11']`
    ///
    /// @default `esnext` (No transformation)
    ///
    /// @see [esbuild#target](https://esbuild.github.io/api/#target)
    pub target: Option<Either<String, Vec<String>>>,

    /// Behaviour for runtime helpers.
    pub helpers: Option<Helpers>,

    /// Define Plugin
    #[napi(ts_type = "Record<string, string>")]
    pub define: Option<FxHashMap<String, String>>,

    /// Inject Plugin
    #[napi(ts_type = "Record<string, string | [string, string]>")]
    pub inject: Option<FxHashMap<String, Either<String, Vec<String>>>>,

    /// Decorator plugin
    pub decorator: Option<DecoratorOptions>,

    /// Third-party plugins to use.
    pub plugins: Option<PluginsOptions>,
}

impl TryFrom<TransformOptions> for oxc::transformer::TransformOptions {
    type Error = String;

    fn try_from(options: TransformOptions) -> Result<Self, Self::Error> {
        let env = match options.target {
            Some(Either::A(s)) => EnvOptions::from_target(&s)?,
            Some(Either::B(list)) => EnvOptions::from_target_list(&list)?,
            _ => EnvOptions::default(),
        };
        Ok(Self {
            cwd: options.cwd.map(PathBuf::from).unwrap_or_default(),
            assumptions: options.assumptions.map(Into::into).unwrap_or_default(),
            typescript: options
                .typescript
                .map(oxc::transformer::TypeScriptOptions::from)
                .unwrap_or_default(),
            decorator: options
                .decorator
                .map(oxc::transformer::DecoratorOptions::from)
                .unwrap_or_default(),
            jsx: match options.jsx {
                Some(Either::A(s)) => {
                    if s == "preserve" {
                        oxc::transformer::JsxOptions::disable()
                    } else {
                        return Err(format!("Invalid jsx option: `{s}`."));
                    }
                }
                Some(Either::B(options)) => oxc::transformer::JsxOptions::from(options),
                None => oxc::transformer::JsxOptions::enable(),
            },
            env,
            proposals: ProposalOptions::default(),
            helper_loader: options
                .helpers
                .map_or_else(HelperLoaderOptions::default, HelperLoaderOptions::from),
            plugins: options
                .plugins
                .map(oxc::transformer::PluginsOptions::from)
                .unwrap_or_default(),
        })
    }
}

#[napi(object)]
#[derive(Default, Debug)]
pub struct CompilerAssumptions {
    pub ignore_function_length: Option<bool>,
    pub no_document_all: Option<bool>,
    pub object_rest_no_symbols: Option<bool>,
    pub pure_getters: Option<bool>,
    /// When using public class fields, assume that they don't shadow any getter in the current class,
    /// in its subclasses or in its superclass. Thus, it's safe to assign them rather than using
    /// `Object.defineProperty`.
    ///
    /// For example:
    ///
    /// Input:
    /// ```js
    /// class Test {
    ///  field = 2;
    ///
    ///  static staticField = 3;
    /// }
    /// ```
    ///
    /// When `set_public_class_fields` is `true`, the output will be:
    /// ```js
    /// class Test {
    ///  constructor() {
    ///    this.field = 2;
    ///  }
    /// }
    /// Test.staticField = 3;
    /// ```
    ///
    /// Otherwise, the output will be:
    /// ```js
    /// import _defineProperty from "@oxc-project/runtime/helpers/defineProperty";
    /// class Test {
    ///   constructor() {
    ///     _defineProperty(this, "field", 2);
    ///   }
    /// }
    /// _defineProperty(Test, "staticField", 3);
    /// ```
    ///
    /// NOTE: For TypeScript, if you wanted behavior is equivalent to `useDefineForClassFields: false`, you should
    /// set both `set_public_class_fields` and [`crate::TypeScriptOptions::remove_class_fields_without_initializer`]
    /// to `true`.
    pub set_public_class_fields: Option<bool>,
}

impl From<CompilerAssumptions> for oxc::transformer::CompilerAssumptions {
    fn from(value: CompilerAssumptions) -> Self {
        let ops = oxc::transformer::CompilerAssumptions::default();
        Self {
            ignore_function_length: value
                .ignore_function_length
                .unwrap_or(ops.ignore_function_length),
            no_document_all: value.no_document_all.unwrap_or(ops.no_document_all),
            object_rest_no_symbols: value
                .object_rest_no_symbols
                .unwrap_or(ops.object_rest_no_symbols),
            pure_getters: value.pure_getters.unwrap_or(ops.pure_getters),
            set_public_class_fields: value
                .set_public_class_fields
                .unwrap_or(ops.set_public_class_fields),
            ..ops
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
    /// When enabled, type-only class fields are only removed if they are prefixed with the declare modifier:
    ///
    /// @deprecated
    ///
    /// Allowing `declare` fields is built-in support in Oxc without any option. If you want to remove class fields
    /// without initializer, you can use `remove_class_fields_without_initializer: true` instead.
    pub allow_declare_fields: Option<bool>,
    /// When enabled, class fields without initializers are removed.
    ///
    /// For example:
    /// ```ts
    /// class Foo {
    ///    x: number;
    ///    y: number = 0;
    /// }
    /// ```
    /// // transform into
    /// ```js
    /// class Foo {
    ///    x: number;
    /// }
    /// ```
    ///
    /// The option is used to align with the behavior of TypeScript's `useDefineForClassFields: false` option.
    /// When you want to enable this, you also need to set [`crate::CompilerAssumptions::set_public_class_fields`]
    /// to `true`. The `set_public_class_fields: true` + `remove_class_fields_without_initializer: true` is
    /// equivalent to `useDefineForClassFields: false` in TypeScript.
    ///
    /// When `set_public_class_fields` is true and class-properties plugin is enabled, the above example transforms into:
    ///
    /// ```js
    /// class Foo {
    ///   constructor() {
    ///     this.y = 0;
    ///   }
    /// }
    /// ```
    ///
    /// Defaults to `false`.
    pub remove_class_fields_without_initializer: Option<bool>,
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

impl From<TypeScriptOptions> for oxc::transformer::TypeScriptOptions {
    fn from(options: TypeScriptOptions) -> Self {
        let ops = oxc::transformer::TypeScriptOptions::default();
        oxc::transformer::TypeScriptOptions {
            jsx_pragma: options.jsx_pragma.map(Into::into).unwrap_or(ops.jsx_pragma),
            jsx_pragma_frag: options.jsx_pragma_frag.map(Into::into).unwrap_or(ops.jsx_pragma_frag),
            only_remove_type_imports: options
                .only_remove_type_imports
                .unwrap_or(ops.only_remove_type_imports),
            allow_namespaces: options.allow_namespaces.unwrap_or(ops.allow_namespaces),
            allow_declare_fields: options.allow_declare_fields.unwrap_or(ops.allow_declare_fields),
            optimize_const_enums: false,
            remove_class_fields_without_initializer: options
                .remove_class_fields_without_initializer
                .unwrap_or(ops.remove_class_fields_without_initializer),
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

#[napi(object)]
#[derive(Default)]
pub struct DecoratorOptions {
    /// Enables experimental support for decorators, which is a version of decorators that predates the TC39 standardization process.
    ///
    /// Decorators are a language feature which hasn’t yet been fully ratified into the JavaScript specification.
    /// This means that the implementation version in TypeScript may differ from the implementation in JavaScript when it it decided by TC39.
    ///
    /// @see https://www.typescriptlang.org/tsconfig/#experimentalDecorators
    /// @default false
    pub legacy: Option<bool>,

    /// Enables emitting decorator metadata.
    ///
    /// This option the same as [emitDecoratorMetadata](https://www.typescriptlang.org/tsconfig/#emitDecoratorMetadata)
    /// in TypeScript, and it only works when `legacy` is true.
    ///
    /// @see https://www.typescriptlang.org/tsconfig/#emitDecoratorMetadata
    /// @default false
    pub emit_decorator_metadata: Option<bool>,
}

impl From<DecoratorOptions> for oxc::transformer::DecoratorOptions {
    fn from(options: DecoratorOptions) -> Self {
        oxc::transformer::DecoratorOptions {
            legacy: options.legacy.unwrap_or_default(),
            emit_decorator_metadata: options.emit_decorator_metadata.unwrap_or_default(),
        }
    }
}

/// Configure how styled-components are transformed.
///
/// @see {@link https://styled-components.com/docs/tooling#babel-plugin}
#[napi(object)]
#[derive(Default)]
pub struct StyledComponentsOptions {
    /// Enhances the attached CSS class name on each component with richer output to help
    /// identify your components in the DOM without React DevTools.
    ///
    /// @default true
    pub display_name: Option<bool>,

    /// Controls whether the `displayName` of a component will be prefixed with the filename
    /// to make the component name as unique as possible.
    ///
    /// @default true
    pub file_name: Option<bool>,

    /// Adds a unique identifier to every styled component to avoid checksum mismatches
    /// due to different class generation on the client and server during server-side rendering.
    ///
    /// @default true
    pub ssr: Option<bool>,

    /// Transpiles styled-components tagged template literals to a smaller representation
    /// than what Babel normally creates, helping to reduce bundle size.
    ///
    /// @default true
    pub transpile_template_literals: Option<bool>,

    /// Minifies CSS content by removing all whitespace and comments from your CSS,
    /// keeping valuable bytes out of your bundles.
    ///
    /// @default true
    pub minify: Option<bool>,

    /// Enables transformation of JSX `css` prop when using styled-components.
    ///
    /// **Note: This feature is not yet implemented in oxc.**
    ///
    /// @default true
    pub css_prop: Option<bool>,

    /// Enables "pure annotation" to aid dead code elimination by bundlers.
    ///
    /// @default false
    pub pure: Option<bool>,

    /// Adds a namespace prefix to component identifiers to ensure class names are unique.
    ///
    /// Example: With `namespace: "my-app"`, generates `componentId: "my-app__sc-3rfj0a-1"`
    pub namespace: Option<String>,

    /// List of file names that are considered meaningless for component naming purposes.
    ///
    /// When the `fileName` option is enabled and a component is in a file with a name
    /// from this list, the directory name will be used instead of the file name for
    /// the component's display name.
    ///
    /// @default `["index"]`
    pub meaningless_file_names: Option<Vec<String>>,

    /// Import paths to be considered as styled-components imports at the top level.
    ///
    /// **Note: This feature is not yet implemented in oxc.**
    pub top_level_import_paths: Option<Vec<String>>,
}

#[napi(object)]
#[derive(Default)]
pub struct PluginsOptions {
    pub styled_components: Option<StyledComponentsOptions>,
    pub tagged_template_escape: Option<bool>,
}

impl From<PluginsOptions> for oxc::transformer::PluginsOptions {
    fn from(options: PluginsOptions) -> Self {
        oxc::transformer::PluginsOptions {
            styled_components: options
                .styled_components
                .map(oxc::transformer::StyledComponentsOptions::from),
            tagged_template_transform: options.tagged_template_escape.unwrap_or(false),
        }
    }
}

impl From<StyledComponentsOptions> for oxc::transformer::StyledComponentsOptions {
    fn from(options: StyledComponentsOptions) -> Self {
        let ops = oxc::transformer::StyledComponentsOptions::default();
        oxc::transformer::StyledComponentsOptions {
            display_name: options.display_name.unwrap_or(ops.display_name),
            file_name: options.file_name.unwrap_or(ops.file_name),
            ssr: options.ssr.unwrap_or(ops.ssr),
            transpile_template_literals: options
                .transpile_template_literals
                .unwrap_or(ops.transpile_template_literals),
            minify: options.minify.unwrap_or(ops.minify),
            css_prop: options.css_prop.unwrap_or(ops.css_prop),
            pure: options.pure.unwrap_or(ops.pure),
            namespace: options.namespace,
            meaningless_file_names: options
                .meaningless_file_names
                .unwrap_or(ops.meaningless_file_names),
            top_level_import_paths: options
                .top_level_import_paths
                .unwrap_or(ops.top_level_import_paths),
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
    /// It will mark JSX elements and top-level React method calls as pure for tree shaking.
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
    /// Conforms to the implementation in {@link https://github.com/facebook/react/tree/v18.3.1/packages/react-refresh}
    ///
    /// @default false
    pub refresh: Option<Either<bool, ReactRefreshOptions>>,
}

impl From<JsxOptions> for oxc::transformer::JsxOptions {
    fn from(options: JsxOptions) -> Self {
        let ops = oxc::transformer::JsxOptions::default();
        oxc::transformer::JsxOptions {
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
                Either::A(b) => b.then(oxc::transformer::ReactRefreshOptions::default),
                Either::B(options) => Some(oxc::transformer::ReactRefreshOptions::from(options)),
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

impl From<ReactRefreshOptions> for oxc::transformer::ReactRefreshOptions {
    fn from(options: ReactRefreshOptions) -> Self {
        let ops = oxc::transformer::ReactRefreshOptions::default();
        oxc::transformer::ReactRefreshOptions {
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

impl From<ArrowFunctionsOptions> for oxc::transformer::ArrowFunctionsOptions {
    fn from(options: ArrowFunctionsOptions) -> Self {
        oxc::transformer::ArrowFunctionsOptions { spec: options.spec.unwrap_or_default() }
    }
}

#[napi(object)]
pub struct Es2015Options {
    /// Transform arrow functions into function expressions.
    pub arrow_function: Option<ArrowFunctionsOptions>,
}

impl From<Es2015Options> for oxc::transformer::ES2015Options {
    fn from(options: Es2015Options) -> Self {
        oxc::transformer::ES2015Options { arrow_function: options.arrow_function.map(Into::into) }
    }
}

#[napi(object)]
#[derive(Default)]
pub struct Helpers {
    pub mode: Option<HelperMode>,
}

#[derive(Default, Clone, Copy)]
#[napi(string_enum)]
pub enum HelperMode {
    /// Runtime mode (default): Helper functions are imported from a runtime package.
    ///
    /// Example:
    ///
    /// ```js
    /// import helperName from "@oxc-project/runtime/helpers/helperName";
    /// helperName(...arguments);
    /// ```
    #[default]
    Runtime,
    /// External mode: Helper functions are accessed from a global `babelHelpers` object.
    ///
    /// Example:
    ///
    /// ```js
    /// babelHelpers.helperName(...arguments);
    /// ```
    External,
}

impl From<Helpers> for HelperLoaderOptions {
    fn from(value: Helpers) -> Self {
        Self {
            mode: value.mode.map(HelperLoaderMode::from).unwrap_or_default(),
            ..HelperLoaderOptions::default()
        }
    }
}

impl From<HelperMode> for HelperLoaderMode {
    fn from(value: HelperMode) -> Self {
        match value {
            HelperMode::Runtime => Self::Runtime,
            HelperMode::External => Self::External,
        }
    }
}

#[derive(Default)]
struct Compiler {
    transform_options: oxc::transformer::TransformOptions,
    isolated_declaration_options: Option<oxc::isolated_declarations::IsolatedDeclarationsOptions>,

    sourcemap: bool,

    printed: String,
    printed_sourcemap: Option<SourceMap>,

    declaration: Option<String>,
    declaration_map: Option<SourceMap>,

    define: Option<ReplaceGlobalDefinesConfig>,
    inject: Option<InjectGlobalVariablesConfig>,

    helpers_used: FxHashMap<String, String>,
    errors: Vec<OxcDiagnostic>,
}

impl Compiler {
    fn new(options: Option<TransformOptions>) -> Result<Self, Vec<OxcDiagnostic>> {
        let mut options = options;

        let isolated_declaration_options = options
            .as_ref()
            .and_then(|o| o.typescript.as_ref())
            .and_then(|o| o.declaration)
            .map(oxc::isolated_declarations::IsolatedDeclarationsOptions::from);

        let sourcemap = options.as_ref().and_then(|o| o.sourcemap).unwrap_or_default();

        let define = options
            .as_mut()
            .and_then(|options| options.define.take())
            .map(|map| {
                let define = map.into_iter().collect::<Vec<_>>();
                ReplaceGlobalDefinesConfig::new(&define)
            })
            .transpose()?;

        let inject = options
            .as_mut()
            .and_then(|options| options.inject.take())
            .map(|map| {
                map.into_iter()
                    .map(|(local, value)| match value {
                        Either::A(source) => Ok(InjectImport::default_specifier(&source, &local)),
                        Either::B(v) => {
                            if v.len() != 2 {
                                return Err(vec![OxcDiagnostic::error(
                                    "Inject plugin did not receive a tuple [string, string].",
                                )]);
                            }
                            let source = &v[0];
                            Ok(if v[1] == "*" {
                                InjectImport::namespace_specifier(source, &local)
                            } else {
                                InjectImport::named_specifier(source, Some(&v[1]), &local)
                            })
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .map(InjectGlobalVariablesConfig::new);

        let transform_options = match options {
            Some(options) => oxc::transformer::TransformOptions::try_from(options)
                .map_err(|err| vec![OxcDiagnostic::error(err)])?,
            None => oxc::transformer::TransformOptions::default(),
        };

        Ok(Self {
            transform_options,
            isolated_declaration_options,
            sourcemap,
            printed: String::default(),
            printed_sourcemap: None,
            declaration: None,
            declaration_map: None,
            define,
            inject,
            helpers_used: FxHashMap::default(),
            errors: vec![],
        })
    }
}

impl CompilerInterface for Compiler {
    fn handle_errors(&mut self, errors: Vec<OxcDiagnostic>) {
        self.errors.extend(errors);
    }

    fn enable_sourcemap(&self) -> bool {
        self.sourcemap
    }

    fn transform_options(&self) -> Option<&oxc::transformer::TransformOptions> {
        Some(&self.transform_options)
    }

    fn isolated_declaration_options(
        &self,
    ) -> Option<oxc::isolated_declarations::IsolatedDeclarationsOptions> {
        self.isolated_declaration_options
    }

    fn define_options(&self) -> Option<ReplaceGlobalDefinesConfig> {
        self.define.clone()
    }

    fn inject_options(&self) -> Option<InjectGlobalVariablesConfig> {
        self.inject.clone()
    }

    fn after_codegen(&mut self, ret: CodegenReturn) {
        self.printed = ret.code;
        self.printed_sourcemap = ret.map.map(SourceMap::from);
    }

    fn after_isolated_declarations(&mut self, ret: CodegenReturn) {
        self.declaration.replace(ret.code);
        self.declaration_map = ret.map.map(SourceMap::from);
    }

    #[expect(deprecated)]
    fn after_transform(
        &mut self,
        _program: &mut oxc::ast::ast::Program<'_>,
        transformer_return: &mut oxc::transformer::TransformerReturn,
    ) -> ControlFlow<()> {
        self.helpers_used = transformer_return
            .helpers_used
            .drain()
            .map(|(helper, source)| (helper.name().to_string(), source))
            .collect();
        ControlFlow::Continue(())
    }
}

/// Transpile a JavaScript or TypeScript into a target ECMAScript version.
///
/// @param filename The name of the file being transformed. If this is a
/// relative path, consider setting the {@link TransformOptions#cwd} option..
/// @param sourceText the source code itself
/// @param options The options for the transformation. See {@link
/// TransformOptions} for more information.
///
/// @returns an object containing the transformed code, source maps, and any
/// errors that occurred during parsing or transformation.
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
#[napi]
pub fn transform_sync(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> TransformResult {
    let source_path = Path::new(&filename);

    let source_type = get_source_type(
        &filename,
        options.as_ref().and_then(|options| options.lang.as_deref()),
        options.as_ref().and_then(|options| options.source_type.as_deref()),
    );

    let mut compiler = match Compiler::new(options) {
        Ok(compiler) => compiler,
        Err(errors) => {
            return TransformResult {
                errors: OxcError::from_diagnostics(&filename, &source_text, errors),
                ..Default::default()
            };
        }
    };

    compiler.compile(&source_text, source_type, source_path);

    TransformResult {
        code: compiler.printed,
        map: compiler.printed_sourcemap,
        declaration: compiler.declaration,
        declaration_map: compiler.declaration_map,
        helpers_used: compiler.helpers_used,
        errors: OxcError::from_diagnostics(&filename, &source_text, compiler.errors),
    }
}

pub struct TransformTask {
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
}

#[napi]
impl Task for TransformTask {
    type JsValue = TransformResult;
    type Output = TransformResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let source_path = Path::new(&self.filename);

        let source_type = get_source_type(
            &self.filename,
            self.options.as_ref().and_then(|options| options.lang.as_deref()),
            self.options.as_ref().and_then(|options| options.source_type.as_deref()),
        );

        let mut compiler = match Compiler::new(self.options.take()) {
            Ok(compiler) => compiler,
            Err(errors) => {
                return Ok(TransformResult {
                    errors: OxcError::from_diagnostics(&self.filename, &self.source_text, errors),
                    ..Default::default()
                });
            }
        };

        compiler.compile(&self.source_text, source_type, source_path);

        Ok(TransformResult {
            code: compiler.printed,
            map: compiler.printed_sourcemap,
            declaration: compiler.declaration,
            declaration_map: compiler.declaration_map,
            helpers_used: compiler.helpers_used,
            errors: OxcError::from_diagnostics(&self.filename, &self.source_text, compiler.errors),
        })
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// Transpile a JavaScript or TypeScript into a target ECMAScript version, asynchronously.
///
/// Note: This function can be slower than `transform` due to the overhead of spawning a thread.
///
/// @param filename The name of the file being transformed. If this is a
/// relative path, consider setting the {@link TransformOptions#cwd} option.
/// @param sourceText the source code itself
/// @param options The options for the transformation. See {@link
/// TransformOptions} for more information.
///
/// @returns a promise that resolves to an object containing the transformed code,
/// source maps, and any errors that occurred during parsing or transformation.
#[napi]
pub fn transform(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> AsyncTask<TransformTask> {
    AsyncTask::new(TransformTask { filename, source_text, options })
}

#[derive(Default)]
#[napi(object)]
pub struct ModuleRunnerTransformOptions {
    /// Enable source map generation.
    ///
    /// When `true`, the `sourceMap` field of transform result objects will be populated.
    ///
    /// @default false
    ///
    /// @see {@link SourceMap}
    pub sourcemap: Option<bool>,
}

#[derive(Default)]
#[napi(object)]
pub struct ModuleRunnerTransformResult {
    /// The transformed code.
    ///
    /// If parsing failed, this will be an empty string.
    pub code: String,

    /// The source map for the transformed code.
    ///
    /// This will be set if {@link TransformOptions#sourcemap} is `true`.
    pub map: Option<SourceMap>,

    // Import sources collected during transformation.
    pub deps: Vec<String>,

    // Dynamic import sources collected during transformation.
    pub dynamic_deps: Vec<String>,

    /// Parse and transformation errors.
    ///
    /// Oxc's parser recovers from common syntax errors, meaning that
    /// transformed code may still be available even if there are errors in this
    /// list.
    pub errors: Vec<OxcError>,
}

/// Transform JavaScript code to a Vite Node runnable module.
///
/// @param filename The name of the file being transformed.
/// @param sourceText the source code itself
/// @param options The options for the transformation. See {@link
/// ModuleRunnerTransformOptions} for more information.
///
/// @returns an object containing the transformed code, source maps, and any
/// errors that occurred during parsing or transformation.
///
fn module_runner_transform_impl(
    filename: &str,
    source_text: &str,
    options: Option<ModuleRunnerTransformOptions>,
) -> ModuleRunnerTransformResult {
    let file_path = Path::new(filename);
    let source_type = SourceType::from_path(file_path);
    let source_type = match source_type {
        Ok(s) => s,
        Err(err) => {
            return ModuleRunnerTransformResult {
                code: String::default(),
                map: None,
                deps: vec![],
                dynamic_deps: vec![],
                errors: vec![OxcError::new(err.to_string())],
            };
        }
    };

    let allocator = Allocator::default();
    let mut parser_ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = parser_ret.program;

    let SemanticBuilderReturn { semantic, errors } =
        SemanticBuilder::new().with_check_syntax_error(true).build(&program);
    parser_ret.errors.extend(errors);

    let scoping = semantic.into_scoping();
    let (deps, dynamic_deps) =
        ModuleRunnerTransform::default().transform(&allocator, &mut program, scoping);

    let CodegenReturn { code, map, .. } = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: options.and_then(|opts| {
                opts.sourcemap.as_ref().and_then(|s| s.then(|| file_path.to_path_buf()))
            }),
            ..Default::default()
        })
        .build(&program);

    ModuleRunnerTransformResult {
        code,
        map: map.map(Into::into),
        deps: deps.into_iter().collect::<Vec<String>>(),
        dynamic_deps: dynamic_deps.into_iter().collect::<Vec<String>>(),
        errors: OxcError::from_diagnostics(filename, source_text, parser_ret.errors),
    }
}

/// @deprecated Only works for Vite.
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
#[napi]
pub fn module_runner_transform_sync(
    filename: String,
    source_text: String,
    options: Option<ModuleRunnerTransformOptions>,
) -> ModuleRunnerTransformResult {
    module_runner_transform_impl(&filename, &source_text, options)
}

pub struct ModuleRunnerTransformTask {
    filename: String,
    source_text: String,
    options: Option<ModuleRunnerTransformOptions>,
}

#[napi]
impl Task for ModuleRunnerTransformTask {
    type JsValue = ModuleRunnerTransformResult;
    type Output = ModuleRunnerTransformResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(module_runner_transform_impl(&self.filename, &self.source_text, self.options.take()))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// Transform JavaScript code to a Vite Node runnable module.
///
/// @param filename The name of the file being transformed.
/// @param sourceText the source code itself
/// @param options The options for the transformation. See {@link
/// ModuleRunnerTransformOptions} for more information.
///
/// @returns an object containing the transformed code, source maps, and any
/// errors that occurred during parsing or transformation.
///
/// Note: This function can be slower than `moduleRunnerTransformSync` due to the overhead of spawning a thread.
///
/// @deprecated Only works for Vite.
#[napi]
pub fn module_runner_transform(
    filename: String,
    source_text: String,
    options: Option<ModuleRunnerTransformOptions>,
) -> AsyncTask<ModuleRunnerTransformTask> {
    AsyncTask::new(ModuleRunnerTransformTask { filename, source_text, options })
}
