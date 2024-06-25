use std::path::Path;

use napi_derive::napi;
use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenReturn};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_transformer::{
    ArrowFunctionsOptions, ES2015Options, ReactJsxRuntime, ReactOptions, TransformOptions,
    Transformer, TypeScriptOptions,
};

#[napi(object)]
pub struct TypeScriptBindingOptions {
    pub jsx_pragma: String,
    pub jsx_pragma_frag: String,
    pub only_remove_type_imports: bool,
    pub allow_namespaces: bool,
    pub allow_declare_fields: bool,
}

impl From<TypeScriptBindingOptions> for TypeScriptOptions {
    fn from(options: TypeScriptBindingOptions) -> Self {
        TypeScriptOptions {
            jsx_pragma: options.jsx_pragma.into(),
            jsx_pragma_frag: options.jsx_pragma_frag.into(),
            only_remove_type_imports: options.only_remove_type_imports,
            allow_namespaces: options.allow_namespaces,
            allow_declare_fields: options.allow_declare_fields,
        }
    }
}

#[napi(object)]
pub struct ReactBindingOptions {
    #[napi(ts_type = "'classic' | 'automatic'")]
    pub runtime: String,
    pub development: bool,
    pub throw_if_namespace: bool,
    pub pure: bool,
    pub import_source: Option<String>,
    pub pragma: Option<String>,
    pub pragma_frag: Option<String>,
    pub use_built_ins: Option<bool>,
    pub use_spread: Option<bool>,
}

#[allow(clippy::wildcard_in_or_patterns)]
impl From<ReactBindingOptions> for ReactOptions {
    fn from(options: ReactBindingOptions) -> Self {
        ReactOptions {
            runtime: match options.runtime.as_str() {
                "classic" => ReactJsxRuntime::Classic,
                "automatic" | _ => ReactJsxRuntime::Automatic,
            },
            development: options.development,
            throw_if_namespace: options.throw_if_namespace,
            pure: options.pure,
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
    pub spec: bool,
}

impl From<ArrowFunctionsBindingOptions> for ArrowFunctionsOptions {
    fn from(options: ArrowFunctionsBindingOptions) -> Self {
        ArrowFunctionsOptions { spec: options.spec }
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
pub struct TransformBindingOptions {
    pub typescript: TypeScriptBindingOptions,
    pub react: ReactBindingOptions,
    pub es2015: ES2015BindingOptions,
    pub sourcemap: bool,
}

impl From<TransformBindingOptions> for TransformOptions {
    fn from(options: TransformBindingOptions) -> Self {
        TransformOptions {
            typescript: options.typescript.into(),
            react: options.react.into(),
            es2015: options.es2015.into(),
            ..Default::default()
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
    options: TransformBindingOptions,
) -> TransformResult {
    let mut errors = vec![];

    let source_path = Path::new(&filename);
    let source_type = SourceType::from_path(source_path).unwrap_or_default();
    let allocator = Allocator::default();
    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
    if !parser_ret.errors.is_empty() {
        errors.extend(parser_ret.errors.into_iter().map(|error| error.message.to_string()));
    }

    let enable_sourcemap = options.sourcemap;

    let mut program = parser_ret.program;
    let transform_options = options.into();
    if let Err(e) = Transformer::new(
        &allocator,
        source_path,
        source_type,
        &source_text,
        parser_ret.trivias.clone(),
        transform_options,
    )
    .build(&mut program)
    {
        errors.extend(e.into_iter().map(|error| error.to_string()));
    }

    let CodegenReturn { source_text, source_map } = if enable_sourcemap {
        CodeGenerator::new()
            .enable_source_map(source_path.to_string_lossy().as_ref(), &source_text)
            .build(&program)
    } else {
        CodeGenerator::new().build(&program)
    };

    TransformResult {
        source_text,
        map: source_map.map(|sourcemap| {
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
