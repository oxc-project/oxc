// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

mod options;

use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::Rc,
};

use oxc::{
    allocator::Allocator,
    ast::{ast::Program, Comment as OxcComment, CommentKind, Visit},
    codegen::{CodeGenerator, CodegenOptions},
    diagnostics::Error,
    minifier::{CompressOptions, Minifier, MinifierOptions},
    parser::{ParseOptions, Parser, ParserReturn},
    semantic::{
        dot::{DebugDot, DebugDotContext},
        ScopeFlags, ScopeId, ScopeTree, SemanticBuilder, SymbolTable,
    },
    span::SourceType,
    transformer::{EnvOptions, Targets, TransformOptions, Transformer},
};
use oxc_index::Idx;
use oxc_linter::Linter;
use oxc_prettier::{Prettier, PrettierOptions};
use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::options::{OxcOptions, OxcRunOptions};

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Oxc {
    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "Program")]
    pub ast: JsValue,

    #[wasm_bindgen(readonly, skip_typescript)]
    pub ir: String,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "controlFlowGraph")]
    pub control_flow_graph: String,

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
        console_error_panic_hook::set_once();
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
            transformer: transform_options,
            codegen: codegen_options,
            minifier: minifier_options,
            control_flow: control_flow_options,
        } = options;
        let run_options = run_options.unwrap_or_default();
        let parser_options = parser_options.unwrap_or_default();
        let _linter_options = linter_options.unwrap_or_default();
        let minifier_options = minifier_options.unwrap_or_default();
        let _codegen_options = codegen_options.unwrap_or_default();
        let _transform_options = transform_options.unwrap_or_default();
        let control_flow_options = control_flow_options.unwrap_or_default();

        let allocator = Allocator::default();

        let path = PathBuf::from(
            parser_options.source_filename.clone().unwrap_or_else(|| "test.tsx".to_string()),
        );
        let source_type = SourceType::from_path(&path).unwrap_or_default();
        let source_type = match parser_options.source_type.as_deref() {
            Some("script") => source_type.with_script(true),
            Some("module") => source_type.with_module(true),
            _ => source_type,
        };

        let default_parser_options = ParseOptions::default();
        let oxc_parser_options = ParseOptions {
            parse_regular_expression: true,
            allow_return_outside_function: parser_options
                .allow_return_outside_function
                .unwrap_or(default_parser_options.allow_return_outside_function),
            preserve_parens: parser_options
                .preserve_parens
                .unwrap_or(default_parser_options.preserve_parens),
        };
        let ParserReturn { mut program, errors, .. } =
            Parser::new(&allocator, source_text, source_type)
                .with_options(oxc_parser_options)
                .parse();

        self.comments = Self::map_comments(source_text, &program.comments);
        self.ir = format!("{:#?}", program.body);
        self.ast = program.serialize(&self.serializer)?;

        let mut semantic_builder = SemanticBuilder::new();
        if run_options.transform.unwrap_or_default() {
            // Estimate transformer will triple scopes, symbols, references
            semantic_builder = semantic_builder.with_excess_capacity(2.0);
        }
        let semantic_ret = semantic_builder
            .with_check_syntax_error(true)
            .with_cfg(true)
            .build_module_record(&path, &program)
            .build(&program);

        self.control_flow_graph = semantic_ret.semantic.cfg().map_or_else(String::default, |cfg| {
            cfg.debug_dot(DebugDotContext::new(
                semantic_ret.semantic.nodes(),
                control_flow_options.verbose.unwrap_or_default(),
            ))
        });
        if run_options.syntax.unwrap_or_default() {
            self.save_diagnostics(
                errors.into_iter().chain(semantic_ret.errors).map(Error::from).collect::<Vec<_>>(),
            );
        }

        self.run_linter(&run_options, &path, &program);

        self.run_prettier(&run_options, source_text, source_type);

        let (symbols, scopes) = semantic_ret.semantic.into_symbol_table_and_scope_tree();

        if !source_type.is_typescript_definition() {
            if run_options.scope.unwrap_or_default() {
                self.scope_text = Self::get_scope_text(&program, &symbols, &scopes);
            }
            if run_options.symbol.unwrap_or_default() {
                self.symbols = symbols.serialize(&self.serializer)?;
            }
        }

        if run_options.transform.unwrap_or_default() {
            if let Ok(options) = TransformOptions::from_preset_env(&EnvOptions {
                targets: Targets::from_query("chrome 51"),
                ..EnvOptions::default()
            }) {
                let result = Transformer::new(&allocator, &path, options)
                    .build_with_symbols_and_scopes(symbols, scopes, &mut program);
                if !result.errors.is_empty() {
                    self.save_diagnostics(
                        result.errors.into_iter().map(Error::from).collect::<Vec<_>>(),
                    );
                }
            }
        }

        let mangler = if minifier_options.compress.unwrap_or_default()
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
            Minifier::new(options).build(&allocator, &mut program).mangler
        } else {
            None
        };

        self.codegen_text = CodeGenerator::new()
            .with_mangler(mangler)
            .with_options(CodegenOptions {
                minify: minifier_options.whitespace.unwrap_or_default(),
                ..CodegenOptions::default()
            })
            .build(&program)
            .code;

        Ok(())
    }

    fn run_linter(&mut self, run_options: &OxcRunOptions, path: &Path, program: &Program) {
        // Only lint if there are no syntax errors
        if run_options.lint.unwrap_or_default() && self.diagnostics.borrow().is_empty() {
            let semantic_ret = SemanticBuilder::new()
                .with_cfg(true)
                .build_module_record(path, program)
                .build(program);
            let semantic = Rc::new(semantic_ret.semantic);
            let linter_ret = Linter::default().run(path, Rc::clone(&semantic));
            let diagnostics = linter_ret.into_iter().map(|e| Error::from(e.error)).collect();
            self.save_diagnostics(diagnostics);
        }
    }

    fn run_prettier(
        &mut self,
        run_options: &OxcRunOptions,
        source_text: &str,
        source_type: SourceType,
    ) {
        let allocator = Allocator::default();
        if run_options.prettier_format.unwrap_or_default()
            || run_options.prettier_ir.unwrap_or_default()
        {
            let ret = Parser::new(&allocator, source_text, source_type)
                .with_options(ParseOptions { preserve_parens: false, ..ParseOptions::default() })
                .parse();

            let mut prettier = Prettier::new(&allocator, PrettierOptions::default());

            if run_options.prettier_format.unwrap_or_default() {
                self.prettier_formatted_text = prettier.build(&ret.program);
            }

            if run_options.prettier_ir.unwrap_or_default() {
                let prettier_doc = prettier.doc(&ret.program).to_string();
                self.prettier_ir_text = {
                    let ret = Parser::new(&allocator, &prettier_doc, SourceType::default()).parse();
                    Prettier::new(&allocator, PrettierOptions::default()).build(&ret.program)
                };
            }
        }
    }

    fn get_scope_text(program: &Program<'_>, symbols: &SymbolTable, scopes: &ScopeTree) -> String {
        struct ScopesTextWriter<'s> {
            symbols: &'s SymbolTable,
            scopes: &'s ScopeTree,
            scope_text: String,
            indent: usize,
            space: String,
        }

        impl<'s> ScopesTextWriter<'s> {
            fn new(symbols: &'s SymbolTable, scopes: &'s ScopeTree) -> Self {
                Self { symbols, scopes, scope_text: String::new(), indent: 0, space: String::new() }
            }

            fn write_line<S: AsRef<str>>(&mut self, line: S) {
                self.scope_text.push_str(&self.space[0..self.indent]);
                self.scope_text.push_str(line.as_ref());
                self.scope_text.push('\n');
            }

            fn indent_in(&mut self) {
                self.indent += 2;
                if self.space.len() < self.indent {
                    self.space.push_str("  ");
                }
            }

            fn indent_out(&mut self) {
                self.indent -= 2;
            }
        }

        impl<'a, 's> Visit<'a> for ScopesTextWriter<'s> {
            fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
                let scope_id = scope_id.get().unwrap();
                self.write_line(format!("Scope {} ({flags:?}) {{", scope_id.index()));
                self.indent_in();

                let bindings = self.scopes.get_bindings(scope_id);
                if !bindings.is_empty() {
                    self.write_line("Bindings: {");
                    bindings.iter().for_each(|(name, &symbol_id)| {
                        let symbol_flags = self.symbols.get_flags(symbol_id);
                        self.write_line(format!("  {name} ({symbol_id:?} {symbol_flags:?})",));
                    });
                    self.write_line("}");
                }
            }

            fn leave_scope(&mut self) {
                self.indent_out();
                self.write_line("}");
            }
        }

        let mut writer = ScopesTextWriter::new(symbols, scopes);
        writer.visit_program(program);
        writer.scope_text
    }

    fn save_diagnostics(&self, diagnostics: Vec<Error>) {
        self.diagnostics.borrow_mut().extend(diagnostics);
    }

    fn map_comments(source_text: &str, comments: &[OxcComment]) -> Vec<Comment> {
        comments
            .iter()
            .map(|comment| Comment {
                r#type: match comment.kind {
                    CommentKind::Line => CommentType::Line,
                    CommentKind::Block => CommentType::Block,
                },
                value: comment.span.source_text(source_text).to_string(),
                start: comment.span.start,
                end: comment.span.end,
            })
            .collect()
    }
}
