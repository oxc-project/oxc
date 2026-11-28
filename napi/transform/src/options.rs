// NOTE: Types must be aligned with [@types/babel__core](https://github.com/DefinitelyTyped/DefinitelyTyped/blob/b5dc32740d9b45d11cff9b025896dd333c795b39/types/babel__core/index.d.ts).
#![expect(rustdoc::bare_urls)]

use std::path::PathBuf;

use napi::Either;
use napi_derive::napi;
use rustc_hash::FxHashMap;

use oxc::transformer::{
    EnvOptions, HelperLoaderMode, HelperLoaderOptions, JsxRuntime, RewriteExtensionsMode,
};

use crate::IsolatedDeclarationsOptions;

/// Options for transforming a JavaScript or TypeScript file.
///
/// Options are listed in evaluation order: the source is parsed (`lang`,
/// `sourceType`), declarations are emitted (`typescript.declaration`), then
/// transforms run (`typescript`, `decorator`, `plugins`,
/// `jsx`, `target`), followed by the `inject` and `define` plugins, and
/// finally codegen (`sourcemap`). `helpers` configures the runtime helpers
/// the transforms emit.
///
/// @see {@link transform}
#[napi(object)]
#[derive(Default)]
pub struct TransformOptions {
    /// Treat the source text as `js`, `jsx`, `ts`, `tsx`, or `dts`.
    #[napi(ts_type = "'js' | 'jsx' | 'ts' | 'tsx' | 'dts'")]
    pub lang: Option<String>,

    /// Treat the source text as `script` or `module` code.
    #[napi(ts_type = "'script' | 'module' | 'commonjs' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,

    /// The current working directory. Used to resolve relative paths in other
    /// options.
    pub cwd: Option<String>,

    /// Set assumptions in order to produce smaller output.
    pub assumptions: Option<CompilerAssumptions>,

    /// Configure how TypeScript is transformed.
    ///
    /// `typescript.declaration` is evaluated before all transforms.
    ///
    /// @see {@link https://oxc.rs/docs/guide/usage/transformer/typescript}
    pub typescript: Option<TypeScriptOptions>,

    /// Decorator plugin
    pub decorator: Option<DecoratorOptions>,

    /// Third-party plugins to use.
    /// @see {@link https://oxc.rs/docs/guide/usage/transformer/plugins}
    pub plugins: Option<PluginsOptions>,

    /// Configure how TSX and JSX are transformed.
    /// @see {@link https://oxc.rs/docs/guide/usage/transformer/jsx}
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
    /// @see {@link https://oxc.rs/docs/guide/usage/transformer/lowering#target}
    pub target: Option<Either<String, Vec<String>>>,

    /// Behaviour for runtime helpers.
    pub helpers: Option<Helpers>,

    /// Inject Plugin
    ///
    /// Runs after all transforms.
    ///
    /// @see {@link https://oxc.rs/docs/guide/usage/transformer/global-variable-replacement#inject}
    #[napi(ts_type = "Record<string, string | [string, string]>")]
    pub inject: Option<FxHashMap<String, Either<String, Vec<String>>>>,

    /// Define Plugin
    ///
    /// Runs after the inject plugin.
    ///
    /// @see {@link https://oxc.rs/docs/guide/usage/transformer/global-variable-replacement#define}
    #[napi(ts_type = "Record<string, string>")]
    pub define: Option<FxHashMap<String, String>>,

    /// Enable source map generation.
    ///
    /// When `true`, the `sourceMap` field of transform result objects will be populated.
    ///
    /// @default false
    ///
    /// @see {@link SourceMap}
    pub sourcemap: Option<bool>,
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
            cwd: options.cwd.map_or_default(PathBuf::from),
            assumptions: options.assumptions.map_or_default(Into::into),
            typescript: options
                .typescript
                .map_or_default(oxc::transformer::TypeScriptOptions::from),
            decorator: options.decorator.map_or_default(oxc::transformer::DecoratorOptions::from),
            plugins: options.plugins.map_or_default(oxc::transformer::PluginsOptions::from),
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
            helper_loader: options
                .helpers
                .map_or_else(HelperLoaderOptions::default, HelperLoaderOptions::from),
            // `..Default` supplies `proposals` (TC39, none implemented) and, when a
            // workspace build enables it via Cargo feature unification, the gated
            // `oxc_transformer` `react_compiler` field this binding no longer exposes.
            // Keeps the literal valid in every feature config (and avoids
            // `clippy::needless_update`).
            ..Default::default()
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
    /// When true, optimize const enums by inlining their values at usage sites
    /// and removing the enum declaration.
    ///
    /// @default false
    pub optimize_const_enums: Option<bool>,
    /// When true, optimize regular (non-const) enums by inlining their member
    /// accesses at usage sites when the member value is statically known.
    ///
    /// Non-exported enum declarations are also removed when all members are
    /// evaluable and no references to the enum as a runtime value exist
    /// (e.g., `console.log(Foo)`, `typeof Foo`, or passing the enum as an argument).
    ///
    /// @default false
    pub optimize_enums: Option<bool>,
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
            optimize_const_enums: options.optimize_const_enums.unwrap_or(ops.optimize_const_enums),
            optimize_enums: options.optimize_enums.unwrap_or(ops.optimize_enums),
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

    /// Aligns nullable-union `design:type` emission with `--strictNullChecks`.
    ///
    /// When `true` (default), `T | null` and `T | undefined` emit `Object`, matching tsc strict.
    /// When `false`, `null` and `undefined` are elided from the union so the underlying
    /// primitive constructor is emitted, matching tsc with `--strictNullChecks=false`
    /// and `babel-plugin-transform-typescript-metadata`.
    ///
    /// @see https://www.typescriptlang.org/tsconfig/#strictNullChecks
    /// @default true
    pub strict_null_checks: Option<bool>,
}

impl From<DecoratorOptions> for oxc::transformer::DecoratorOptions {
    fn from(options: DecoratorOptions) -> Self {
        oxc::transformer::DecoratorOptions {
            legacy: options.legacy.unwrap_or_default(),
            emit_decorator_metadata: options.emit_decorator_metadata.unwrap_or_default(),
            strict_null_checks: options.strict_null_checks.unwrap_or(true),
        }
    }
}

/// Configure how styled-components are transformed.
///
/// @see {@link https://oxc.rs/docs/guide/usage/transformer/plugins#styled-components}
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
    /// Disabled by default because Oxc does not down-level template literals, so this
    /// transform only increases output size.
    ///
    /// @default false
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
/// @see {@link https://oxc.rs/docs/guide/usage/transformer/jsx}
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
    pub development: Option<bool>,

    /// Toggles whether or not to throw an error if an XML namespaced tag name
    /// is used.
    ///
    /// Though the JSX spec allows this, it is disabled by default since React's
    /// JSX does not currently have support for it.
    ///
    /// @default true
    pub throw_if_namespace: Option<bool>,

    /// Mark JSX elements and top-level React method calls as pure for tree shaking.
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
            use_built_ins: None,
            use_spread: None,
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
            mode: value.mode.map_or_default(HelperLoaderMode::from),
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
