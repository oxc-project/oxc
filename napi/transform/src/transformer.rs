use std::path::Path;

use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_transformer::{
    ArrowFunctionsOptions, ES2015Options, ReactJsxRuntime, ReactOptions, Transformer,
    TypeScriptOptions,
};

#[napi(object)]
pub struct TypeScriptBindingOptions {
    pub jsx_pragma: Option<String>,
    pub jsx_pragma_frag: Option<String>,
    pub only_remove_type_imports: Option<bool>,
    pub allow_namespaces: Option<bool>,
    pub allow_declare_fields: Option<bool>,
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

#[napi(object)]
pub struct ReactBindingOptions {
    #[napi(ts_type = "'classic' | 'automatic'")]
    pub runtime: Option<String>,
    pub development: Option<bool>,
    pub throw_if_namespace: Option<bool>,
    pub pure: Option<bool>,
    pub import_source: Option<String>,
    pub pragma: Option<String>,
    pub pragma_frag: Option<String>,
    pub use_built_ins: Option<bool>,
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
    pub spec: Option<bool>,
}

impl From<ArrowFunctionsBindingOptions> for ArrowFunctionsOptions {
    fn from(options: ArrowFunctionsBindingOptions) -> Self {
        ArrowFunctionsOptions { spec: options.spec.unwrap_or_default() }
    }
}

#[napi(object)]
pub struct ES2015BindingOptions {
    pub arrow_function: Option<ArrowFunctionsBindingOptions>,
}

impl From<ES2015BindingOptions> for ES2015Options {
    fn from(options: ES2015BindingOptions) -> Self {
        ES2015Options { arrow_function: options.arrow_function.map(Into::into) }
    }
}

#[napi(object)]
pub struct TransformOptions {
    #[napi(ts_type = "'script' | 'module' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,
    pub typescript: Option<TypeScriptBindingOptions>,
    pub react: Option<ReactBindingOptions>,
    pub es2015: Option<ES2015BindingOptions>,
    /// Enable Sourcemap
    ///
    /// * `true` to generate a sourcemap for the code and include it in the result object.
    ///
    /// Default: false
    pub sourcemap: Option<bool>,
}

impl From<TransformOptions> for oxc_transformer::TransformOptions {
    fn from(options: TransformOptions) -> Self {
        Self {
            typescript: options.typescript.map(Into::into).unwrap_or_default(),
            react: options.react.map(Into::into).unwrap_or_default(),
            es2015: options.es2015.map(Into::into).unwrap_or_default(),
            ..Self::default()
        }
    }
}

#[napi(object)]
pub struct Sourcemap {
    pub file: Option<String>,
    pub mappings: Option<String>,
    pub source_root: Option<String>,
    pub sources: Option<Vec<Option<String>>>,
    pub sources_content: Option<Vec<Option<String>>>,
    pub names: Option<Vec<String>>,
}

#[napi(object)]
pub struct TransformResult {
    pub source_text: String,
    pub map: Option<Sourcemap>,
    pub errors: Vec<String>,
}

#[allow(clippy::needless_pass_by_value, dead_code)]
#[napi]
pub fn transform(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> TransformResult {
    let sourcemap = options.as_ref().is_some_and(|x| x.sourcemap.unwrap_or_default());
    let mut errors = vec![];

    let source_type = SourceType::from_path(&filename).unwrap_or_default();
    let source_type = match options.as_ref().and_then(|options| options.source_type.as_deref()) {
        Some("script") => source_type.with_script(true),
        Some("module") => source_type.with_module(true),
        _ => source_type,
    };

    let allocator = Allocator::default();
    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
    if !parser_ret.errors.is_empty() {
        errors.extend(parser_ret.errors.into_iter().map(|error| error.message.to_string()));
    }

    let mut program = parser_ret.program;
    let transform_options = options.map(Into::into).unwrap_or_default();
    let ret = Transformer::new(
        &allocator,
        Path::new(&filename),
        source_type,
        &source_text,
        parser_ret.trivias.clone(),
        transform_options,
    )
    .build(&mut program);

    if !ret.errors.is_empty() {
        errors.extend(ret.errors.into_iter().map(|error| error.to_string()));
    }

    let mut codegen = CodeGenerator::new();
    if sourcemap {
        codegen = codegen.enable_source_map(&filename, &source_text);
    }
    let ret = codegen.build(&program);

    TransformResult {
        source_text: ret.source_text,
        map: ret.source_map.map(|sourcemap| {
            let json = sourcemap.to_json();
            Sourcemap {
                file: json.file,
                mappings: json.mappings,
                source_root: json.source_root,
                sources: json.sources,
                sources_content: json.sources_content,
                names: json.names,
            }
        }),
        errors,
    }
}
