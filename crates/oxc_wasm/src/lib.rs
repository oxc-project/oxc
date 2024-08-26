// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

mod options;

use std::{cell::RefCell, path::PathBuf, rc::Rc};

use options::OxcOptions;
use oxc::{
    allocator::Allocator,
    ast::{CommentKind, Trivias},
    codegen::{CodeGenerator, CodegenOptions},
    diagnostics::Error,
    minifier::{CompressOptions, Minifier, MinifierOptions},
    parser::{ParseOptions, Parser},
    semantic::{ScopeId, Semantic, SemanticBuilder},
    span::SourceType,
    transformer::{TransformOptions, Transformer},
};
use oxc_index::Idx;
use oxc_linter::Linter;
use oxc_prettier::{Prettier, PrettierOptions};
use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Oxc {
    // source_text: String,
    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "Program")]
    pub ast: JsValue,

    #[wasm_bindgen(readonly, skip_typescript)]
    pub ir: String,

    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "SymbolTable")]
    pub symbols: JsValue,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "scopeText")]
    pub scope_text: String,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "codegenText")]
    pub codegen_text: String,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "formattedText")]
    pub formatted_text: String,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "prettierFormattedText")]
    pub prettier_formatted_text: String,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "prettierIrText")]
    pub prettier_ir_text: String,

    comments: Vec<Comment>,

    diagnostics: RefCell<Vec<Error>>,

    #[serde(skip)]
    serializer: serde_wasm_bindgen::Serializer,
}

#[derive(Clone, Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct Comment {
    pub r#type: CommentType,
    pub value: String,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub enum CommentType {
    Line,
    Block,
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
            .collect::<Vec<_>>())
    }

    /// Returns comments
    /// # Errors
    #[wasm_bindgen(js_name = getComments)]
    pub fn get_comments(&self) -> Result<Vec<JsValue>, serde_wasm_bindgen::Error> {
        self.comments.iter().map(|c| c.serialize(&self.serializer)).collect()
    }

    /// # Errors
    /// Serde serialization error
    #[wasm_bindgen]
    pub fn run(
        &mut self,
        source_text: &str,
        options: OxcOptions,
    ) -> Result<(), serde_wasm_bindgen::Error> {
        self.diagnostics = RefCell::default();

        let OxcOptions {
            run: run_options,
            parser: parser_options,
            linter: linter_options,
            codegen: codegen_options,
            minifier: minifier_options,
            type_checking: type_checking_options,
        } = options;
        let run_options = run_options.unwrap_or_default();
        let parser_options = parser_options.unwrap_or_default();
        let _linter_options = linter_options.unwrap_or_default();
        let _codegen_options = codegen_options.unwrap_or_default();
        let minifier_options = minifier_options.unwrap_or_default();
        let _type_checking_options = type_checking_options.unwrap_or_default();

        let allocator = Allocator::default();
        // let source_text = &self.source_text;
        let path = PathBuf::from(
            parser_options.source_filename.clone().unwrap_or_else(|| "test.tsx".to_string()),
        );
        let source_type = SourceType::from_path(&path).unwrap_or_default();
        let source_type = match parser_options.source_type.as_deref() {
            Some("script") => source_type.with_script(true),
            Some("module") => source_type.with_module(true),
            _ => source_type,
        };

        let oxc_parser_options = ParseOptions {
            allow_return_outside_function: parser_options
                .allow_return_outside_function
                .unwrap_or_default(),
            ..ParseOptions::default()
        };

        let ret = Parser::new(&allocator, source_text, source_type)
            .with_options(oxc_parser_options)
            .parse();

        self.comments = Oxc::map_comments(source_text, &ret.trivias);

        self.save_diagnostics(ret.errors.into_iter().map(Error::from).collect::<Vec<_>>());

        self.ir = format!("{:#?}", ret.program.body);

        let program = allocator.alloc(ret.program);

        let semantic_ret = SemanticBuilder::new(source_text, source_type)
            .with_cfg(true)
            .with_trivias(ret.trivias.clone())
            .with_check_syntax_error(true)
            .build(program);

        if run_options.syntax.unwrap_or_default() {
            self.save_diagnostics(
                semantic_ret.errors.into_iter().map(Error::from).collect::<Vec<_>>(),
            );
        }

        let semantic = Rc::new(semantic_ret.semantic);
        // Only lint if there are not syntax errors
        if run_options.lint.unwrap_or_default() && self.diagnostics.borrow().is_empty() {
            let linter_ret = Linter::default().run(&path, Rc::clone(&semantic));
            let diagnostics = linter_ret.into_iter().map(|e| Error::from(e.error)).collect();
            self.save_diagnostics(diagnostics);
        }

        self.ast = program.serialize(&self.serializer)?;

        if run_options.prettier_format.unwrap_or_default() {
            let ret = Parser::new(&allocator, source_text, source_type)
                .with_options(oxc_parser_options)
                .parse();
            let printed =
                Prettier::new(&allocator, source_text, ret.trivias, PrettierOptions::default())
                    .build(&ret.program);
            self.prettier_formatted_text = printed;
        }

        if run_options.prettier_ir.unwrap_or_default() {
            let ret = Parser::new(&allocator, source_text, source_type)
                .with_options(oxc_parser_options)
                .parse();
            let prettier_doc = Prettier::new(
                &allocator,
                source_text,
                ret.trivias.clone(),
                PrettierOptions::default(),
            )
            .doc(&ret.program)
            .to_string();
            self.prettier_ir_text = {
                let ret = Parser::new(&allocator, &prettier_doc, SourceType::default()).parse();
                Prettier::new(&allocator, &prettier_doc, ret.trivias, PrettierOptions::default())
                    .build(&ret.program)
            };
        }

        if run_options.transform.unwrap_or_default() {
            let (symbols, scopes) = SemanticBuilder::new(source_text, source_type)
                .build(program)
                .semantic
                .into_symbol_table_and_scope_tree();
            let options = TransformOptions::default();
            let result = Transformer::new(
                &allocator,
                &path,
                source_type,
                source_text,
                ret.trivias.clone(),
                options,
            )
            .build_with_symbols_and_scopes(symbols, scopes, program);
            if !result.errors.is_empty() {
                let errors = result.errors.into_iter().map(Error::from).collect::<Vec<_>>();
                self.save_diagnostics(errors);
            }
        }

        if run_options.scope.unwrap_or_default() || run_options.symbol.unwrap_or_default() {
            let semantic = SemanticBuilder::new(source_text, source_type)
                .build_module_record(PathBuf::new(), program)
                .build(program)
                .semantic;
            if run_options.scope.unwrap_or_default() {
                self.scope_text = Self::get_scope_text(&semantic);
            } else if run_options.symbol.unwrap_or_default() {
                self.symbols = semantic.symbols().serialize(&self.serializer)?;
            }
        }

        let program = allocator.alloc(program);

        if minifier_options.compress.unwrap_or_default()
            || minifier_options.mangle.unwrap_or_default()
        {
            let compress_options = minifier_options.compress_options.unwrap_or_default();
            let options = MinifierOptions {
                mangle: minifier_options.mangle.unwrap_or_default(),
                compress: if minifier_options.compress.unwrap_or_default() {
                    CompressOptions {
                        booleans: compress_options.booleans,
                        drop_console: compress_options.drop_console,
                        drop_debugger: compress_options.drop_debugger,
                        evaluate: compress_options.evaluate,
                        join_vars: compress_options.join_vars,
                        loops: compress_options.loops,
                        typeofs: compress_options.typeofs,
                        ..CompressOptions::default()
                    }
                } else {
                    CompressOptions::all_false()
                },
            };
            Minifier::new(options).build(&allocator, program);
        }

        self.codegen_text = CodeGenerator::new()
            .with_options(CodegenOptions {
                minify: minifier_options.whitespace.unwrap_or_default(),
                ..CodegenOptions::default()
            })
            .build(program)
            .source_text;

        Ok(())
    }

    fn get_scope_text(semantic: &Semantic) -> String {
        fn write_scope_text(
            semantic: &Semantic,
            scope_text: &mut String,
            depth: usize,
            scope_ids: &[ScopeId],
        ) {
            let space = " ".repeat(depth * 2);

            for scope_id in scope_ids {
                let flags = semantic.scopes().get_flags(*scope_id);
                let next_scope_ids = semantic.scopes().get_child_ids(*scope_id);

                scope_text
                    .push_str(&format!("{space}Scope{:?} ({flags:?}) {{\n", scope_id.index() + 1));
                let bindings = semantic.scopes().get_bindings(*scope_id);
                let binding_space = " ".repeat((depth + 1) * 2);
                if !bindings.is_empty() {
                    scope_text.push_str(&format!("{binding_space}Bindings: {{"));
                }
                bindings.iter().for_each(|(name, symbol_id)| {
                    let symbol_flags = semantic.symbols().get_flags(*symbol_id);
                    scope_text.push_str(&format!("\n{binding_space}  {name} ({symbol_flags:?})",));
                });
                if !bindings.is_empty() {
                    scope_text.push_str(&format!("\n{binding_space}}}\n"));
                }

                write_scope_text(semantic, scope_text, depth + 1, next_scope_ids);
                scope_text.push_str(&format!("{space}}}\n"));
            }
        }

        let mut scope_text = String::default();
        write_scope_text(semantic, &mut scope_text, 0, &[semantic.scopes().root_scope_id()]);
        scope_text
    }

    fn save_diagnostics(&self, diagnostics: Vec<Error>) {
        self.diagnostics.borrow_mut().extend(diagnostics);
    }

    fn map_comments(source_text: &str, trivias: &Trivias) -> Vec<Comment> {
        trivias
            .comments()
            .map(|comment| Comment {
                r#type: match comment.kind {
                    CommentKind::SingleLine => CommentType::Line,
                    CommentKind::MultiLine => CommentType::Block,
                },
                value: comment.span.source_text(source_text).to_string(),
                start: comment.span.start,
                end: comment.span.end,
            })
            .collect()
    }
}
