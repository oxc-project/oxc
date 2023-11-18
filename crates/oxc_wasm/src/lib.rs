mod options;

use std::{cell::RefCell, collections::BTreeMap, path::PathBuf, rc::Rc, sync::Arc};

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions},
    diagnostics::Error,
    formatter::{Formatter, FormatterOptions},
    minifier::{CompressOptions, Minifier, MinifierOptions},
    parser::{Parser, ParserReturn},
    semantic::{SemanticBuilder, SemanticBuilderReturn},
    span::SourceType,
    transformer::{TransformOptions, TransformTarget, Transformer},
};
use oxc_linter::{LintContext, Linter};
use oxc_prettier::{Prettier, PrettierOptions};
use oxc_query::{schema, Adapter, SCHEMA_TEXT};
use oxc_type_synthesis::{synthesize_program, Diagnostic as TypeCheckDiagnostic};
use serde::Serialize;
use trustfall::{execute_query, TransparentValue};
use wasm_bindgen::prelude::*;

use crate::options::{
    OxcFormatterOptions, OxcLinterOptions, OxcMinifierOptions, OxcParserOptions, OxcRunOptions,
    OxcTypeCheckingOptions,
};

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn graphql_schema_text() -> String {
    SCHEMA_TEXT.to_string()
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Oxc {
    source_text: String,

    ast: JsValue,
    ir: JsValue,

    codegen_text: String,
    formatted_text: String,
    prettier_formatted_text: String,
    prettier_ir: String,

    diagnostics: RefCell<Vec<Error>>,

    type_check_diagnostics: RefCell<Vec<TypeCheckDiagnostic>>,

    serializer: serde_wasm_bindgen::Serializer,
}

#[derive(Default, Clone, Serialize)]
pub struct OxcDiagnostic {
    pub start: usize,
    pub end: usize,
    pub severity: String,
    pub message: String,
}

#[wasm_bindgen]
impl Oxc {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { serializer: serde_wasm_bindgen::Serializer::json_compatible(), ..Self::default() }
    }

    #[wasm_bindgen(getter = sourceText)]
    pub fn source_text(&self) -> String {
        self.source_text.clone()
    }

    #[wasm_bindgen(setter = sourceText)]
    pub fn set_source_text(&mut self, source_text: String) {
        self.diagnostics = RefCell::default();
        self.source_text = source_text;
    }

    /// Returns AST in JSON
    #[wasm_bindgen(getter)]
    pub fn ast(&self) -> JsValue {
        self.ast.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn ir(&self) -> JsValue {
        self.ir.clone()
    }

    #[wasm_bindgen(getter = prettierIr)]
    pub fn prettier_ir(&self) -> String {
        self.prettier_ir.clone()
    }

    #[wasm_bindgen(getter = prettierFormattedText)]
    pub fn prettier_formatted_text(&self) -> String {
        self.prettier_formatted_text.clone()
    }

    #[wasm_bindgen(getter = formattedText)]
    pub fn formatted_text(&self) -> String {
        self.formatted_text.clone()
    }

    #[wasm_bindgen(getter = codegenText)]
    pub fn codegen_text(&self) -> String {
        self.codegen_text.clone()
    }

    /// Returns Array of String
    /// # Errors
    /// # Panics
    #[wasm_bindgen(js_name = getDiagnostics)]
    pub fn get_diagnostics(&self) -> Result<Vec<JsValue>, serde_wasm_bindgen::Error> {
        Ok(self
            .diagnostics
            .borrow()
            .iter()
            .flat_map(|error| {
                let Some(labels) = error.labels() else { return vec![] };
                labels
                    .map(|label| {
                        OxcDiagnostic {
                            start: label.offset(),
                            end: label.offset() + label.len(),
                            severity: format!("{:?}", error.severity().unwrap_or_default()),
                            message: format!("{error}"),
                        }
                        .serialize(&self.serializer)
                        .unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .chain(self.type_check_diagnostics.borrow().iter().filter_map(|diagnostic| {
                match diagnostic {
                    TypeCheckDiagnostic::Global { .. } => None,
                    TypeCheckDiagnostic::PositionWithAdditionLabels {
                        reason,
                        position,
                        kind,
                        labels: _,
                    }
                    | TypeCheckDiagnostic::Position { reason, position, kind } => Some(
                        OxcDiagnostic {
                            start: position.start as usize,
                            end: position.end as usize,
                            severity: format!("{kind:?}"),
                            message: reason.to_string(),
                        }
                        .serialize(&self.serializer)
                        .unwrap(),
                    ),
                }
            }))
            .collect::<Vec<_>>())
    }

    /// # Errors
    /// Serde serialization error
    #[wasm_bindgen]
    pub fn run(
        &mut self,
        run_options: &OxcRunOptions,
        parser_options: &OxcParserOptions,
        _linter_options: &OxcLinterOptions,
        formatter_options: &OxcFormatterOptions,
        minifier_options: &OxcMinifierOptions,
        _type_checking_options: &OxcTypeCheckingOptions,
    ) -> Result<(), serde_wasm_bindgen::Error> {
        self.diagnostics = RefCell::default();

        let allocator = Allocator::default();
        let source_text = &self.source_text;
        let path = PathBuf::from("test.tsx");
        let source_type = SourceType::from_path(&path).unwrap_or_default();

        let ret = Parser::new(&allocator, source_text, source_type)
            .allow_return_outside_function(parser_options.allow_return_outside_function)
            .parse();
        let trivias = ret.trivias.clone();
        self.save_diagnostics(ret.errors);

        self.ast = ret.program.serialize(&self.serializer)?;
        self.ir = format!("{:#?}", ret.program.body).into();

        let program = allocator.alloc(ret.program);

        let prettier_doc =
            Prettier::new(&allocator, source_text, trivias.clone(), PrettierOptions::default())
                .doc(program);

        self.prettier_ir = prettier_doc.to_string();

        if run_options.syntax() && !run_options.lint() {
            let semantic_ret = SemanticBuilder::new(source_text, source_type)
                .with_trivias(ret.trivias)
                .with_check_syntax_error(true)
                .build(program);
            self.save_diagnostics(semantic_ret.errors);
        } else if run_options.lint() {
            let semantic_ret = SemanticBuilder::new(source_text, source_type)
                .with_trivias(ret.trivias)
                .with_check_syntax_error(true)
                .build(program);
            self.save_diagnostics(semantic_ret.errors);

            let semantic = Rc::new(semantic_ret.semantic);
            let lint_ctx = LintContext::new(path.into_boxed_path(), &semantic);
            let linter_ret = Linter::new().run(lint_ctx);
            let diagnostics = linter_ret.into_iter().map(|e| e.error).collect();
            self.save_diagnostics(diagnostics);
        }

        if run_options.format() {
            let formatter_options = FormatterOptions {
                indentation: formatter_options.indentation,
                ..Default::default()
            };
            let printed = Formatter::new(source_text.len(), formatter_options).build(program);
            self.formatted_text = printed;
        }

        if run_options.prettier_format() {
            let printed =
                Prettier::new(&allocator, source_text, trivias, PrettierOptions::default())
                    .build(program);
            self.prettier_formatted_text = printed;
        }

        if run_options.type_check() {
            let (diagnostics, ..) = synthesize_program(program, |_: &std::path::Path| None);
            *self.type_check_diagnostics.borrow_mut() = diagnostics.get_diagnostics();
        }

        if run_options.transform() {
            // FIXME: this should not be duplicated with the linter semantic,
            // we need to fix the API so symbols and scopes can be shared.
            let semantic = SemanticBuilder::new(source_text, source_type).build(program).semantic;
            let options =
                TransformOptions { target: TransformTarget::ES2015, ..TransformOptions::default() };
            let result =
                Transformer::new(&allocator, source_type, semantic, options).build(program);
            if let Err(errs) = result {
                self.save_diagnostics(errs);
            }
        }

        let program = allocator.alloc(program);

        if minifier_options.compress() || minifier_options.mangle() {
            let options = MinifierOptions {
                mangle: minifier_options.mangle(),
                compress: if minifier_options.compress() {
                    CompressOptions::all_true()
                } else {
                    CompressOptions::all_false()
                },
            };
            Minifier::new(options).build(&allocator, program);
        }

        self.codegen_text = if minifier_options.whitespace() {
            Codegen::<true>::new(source_text.len(), CodegenOptions).build(program)
        } else {
            Codegen::<false>::new(source_text.len(), CodegenOptions).build(program)
        };

        Ok(())
    }

    fn save_diagnostics(&self, diagnostics: Vec<Error>) {
        self.diagnostics.borrow_mut().extend(diagnostics);
    }

    /// # Errors
    /// Will return `Err` only if a serde wasm bindgen serialization error occurs.
    #[wasm_bindgen]
    pub fn run_query(
        &self,
        parser_options: &OxcParserOptions,
        query: &str,
        query_arguments: &str,
    ) -> Result<wasm_bindgen::JsValue, serde_wasm_bindgen::Error> {
        let allocator = Allocator::default();
        let source_text = &self.source_text;
        let Ok(source_type) = SourceType::from_path("test.tsx") else {
            return "'test.tsx' source type invalid, this should never happen.\nPlease open an issue at https://github.com/oxc-project/oxc".to_string().serialize(&self.serializer);
        };

        let ParserReturn { errors: parse_errors, panicked, program: returned_program, trivias } =
            Parser::new(&allocator, source_text, source_type)
                .allow_return_outside_function(parser_options.allow_return_outside_function)
                .parse();

        let allocated_program = allocator.alloc(returned_program);

        if panicked {
            return "Panicked when parsing code.".to_string().serialize(&self.serializer);
        }

        if !parse_errors.is_empty() {
            return format!("Errors when parsing: \n\n{parse_errors:#?}")
                .serialize(&self.serializer);
        }

        let SemanticBuilderReturn { errors: semantic_errors, semantic } =
            SemanticBuilder::new(source_text, source_type)
                .with_trivias(trivias)
                .with_check_syntax_error(true)
                .build(allocated_program);

        if !semantic_errors.is_empty() {
            return format!("Semantic errors: \n\n{semantic_errors:#?}")
                .serialize(&self.serializer);
        }

        let inner = Rc::new(semantic);

        let adapter = Adapter::new(inner, vec![Some("index.tsx".to_owned())]);

        let arc_adapter = Arc::from(&adapter);

        let Ok(arguments): Result<BTreeMap<Arc<str>, TransparentValue>, _> =
            serde_json::from_str(query_arguments)
        else {
            return "Query arguments is not valid json string, this should never happen.\nPlease open an issue at https://github.com/oxc-project/oxc".serialize(&self.serializer);
        };

        execute_query(schema(), arc_adapter, query, arguments).map_or_else(
            |e| e.to_string().serialize(&self.serializer),
            |f| {
                f.collect::<Vec<_>>()
                    .into_iter()
                    .map(|x| {
                        // The default `FieldValue` JSON representation is explicit about its type, so we can get
                        // reliable round-trip serialization of types tricky in JSON like integers and floats.
                        //
                        // The `TransparentValue` type is like `FieldValue` minus the explicit type representation,
                        // so it's more like what we'd expect to normally find in JSON.
                        let transparent: BTreeMap<_, TransparentValue> =
                            x.into_iter().map(|(k, v)| (k, v.into())).collect();
                        transparent
                    })
                    .collect::<Vec<_>>()
                    .serialize(&self.serializer)
            },
        )
    }
}
