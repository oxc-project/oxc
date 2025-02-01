use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use oxc::{
    allocator::Allocator,
    ast::{ast::Program, Comment as OxcComment, CommentKind, Visit},
    codegen::{CodeGenerator, CodegenOptions},
    minifier::{CompressOptions, MangleOptions, Minifier, MinifierOptions},
    parser::{ParseOptions, Parser, ParserReturn},
    semantic::{
        dot::{DebugDot, DebugDotContext},
        ReferenceId, ScopeFlags, ScopeId, ScopeTree, SemanticBuilder, SymbolFlags, SymbolTable,
    },
    span::{SourceType, Span},
    syntax::reference::Reference,
    transformer::{TransformOptions, Transformer},
};
use oxc_index::Idx;
use oxc_linter::{ConfigStoreBuilder, LintOptions, Linter, ModuleRecord};
use oxc_prettier::{Prettier, PrettierOptions};

use crate::options::{OxcOptions, OxcRunOptions};

mod options;

#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
import type { Program, Span } from "@oxc-project/types";
export * from "@oxc-project/types";
"#;

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
    #[tsify(type = "any")]
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

    #[serde(skip)]
    comments: Vec<Comment>,

    #[serde(skip)]
    diagnostics: RefCell<Vec<oxc::diagnostics::OxcDiagnostic>>,

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
            .flat_map(|error| match &error.labels {
                Some(labels) => labels
                    .iter()
                    .map(|label| OxcDiagnostic {
                        start: label.offset(),
                        end: label.offset() + label.len(),
                        severity: format!("{:?}", error.severity),
                        message: format!("{error}"),
                    })
                    .collect::<Vec<_>>(),
                None => vec![OxcDiagnostic {
                    start: 0,
                    end: 0,
                    severity: format!("{:?}", error.severity),
                    message: format!("{error}"),
                }],
            })
            .map(|v| v.serialize(&self.serializer).unwrap())
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
        let transform_options = transform_options.unwrap_or_default();
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
        let ParserReturn { mut program, errors, module_record, .. } =
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
        let semantic_ret =
            semantic_builder.with_check_syntax_error(true).with_cfg(true).build(&program);
        let semantic = semantic_ret.semantic;

        self.control_flow_graph = semantic.cfg().map_or_else(String::default, |cfg| {
            cfg.debug_dot(DebugDotContext::new(
                semantic.nodes(),
                control_flow_options.verbose.unwrap_or_default(),
            ))
        });
        if run_options.syntax.unwrap_or_default() {
            self.save_diagnostics(
                errors.into_iter().chain(semantic_ret.errors).collect::<Vec<_>>(),
            );
        }

        let module_record = Arc::new(ModuleRecord::new(&path, &module_record, &semantic));
        self.run_linter(&run_options, &path, &program, &module_record);

        self.run_prettier(&run_options, source_text, source_type);

        let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();

        if !source_type.is_typescript_definition() {
            if run_options.scope.unwrap_or_default() {
                self.scope_text = Self::get_scope_text(&program, &symbols, &scopes);
            }
            if run_options.symbol.unwrap_or_default() {
                self.symbols = self.get_symbols_text(&symbols)?;
            }
        }

        if run_options.transform.unwrap_or_default() {
            let options = transform_options
                .target
                .as_ref()
                .and_then(|target| {
                    TransformOptions::from_target(target)
                        .map_err(|err| {
                            self.save_diagnostics(vec![oxc::diagnostics::OxcDiagnostic::error(
                                err,
                            )]);
                        })
                        .ok()
                })
                .unwrap_or_default();
            let result = Transformer::new(&allocator, &path, &options)
                .build_with_symbols_and_scopes(symbols, scopes, &mut program);
            if !result.errors.is_empty() {
                self.save_diagnostics(result.errors.into_iter().collect::<Vec<_>>());
            }
        }

        let symbol_table = if minifier_options.compress.unwrap_or_default()
            || minifier_options.mangle.unwrap_or_default()
        {
            let compress_options = minifier_options.compress_options.unwrap_or_default();
            let options = MinifierOptions {
                mangle: minifier_options.mangle.unwrap_or_default().then(MangleOptions::default),
                compress: Some(if minifier_options.compress.unwrap_or_default() {
                    CompressOptions {
                        drop_console: compress_options.drop_console,
                        drop_debugger: compress_options.drop_debugger,
                        ..CompressOptions::all_false()
                    }
                } else {
                    CompressOptions::all_false()
                }),
            };
            Minifier::new(options).build(&allocator, &mut program).symbol_table
        } else {
            None
        };

        self.codegen_text = CodeGenerator::new()
            .with_symbol_table(symbol_table)
            .with_options(CodegenOptions {
                minify: minifier_options.whitespace.unwrap_or_default(),
                ..CodegenOptions::default()
            })
            .build(&program)
            .code;

        Ok(())
    }

    fn run_linter(
        &mut self,
        run_options: &OxcRunOptions,
        path: &Path,
        program: &Program,
        module_record: &Arc<ModuleRecord>,
    ) {
        // Only lint if there are no syntax errors
        if run_options.lint.unwrap_or_default() && self.diagnostics.borrow().is_empty() {
            let semantic_ret = SemanticBuilder::new().with_cfg(true).build(program);
            let semantic = Rc::new(semantic_ret.semantic);
            let lint_config =
                ConfigStoreBuilder::default().build().expect("Failed to build config store");
            let linter_ret = Linter::new(LintOptions::default(), lint_config).run(
                path,
                Rc::clone(&semantic),
                Arc::clone(module_record),
            );
            let diagnostics = linter_ret.into_iter().map(|e| e.error).collect();
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

        impl Visit<'_> for ScopesTextWriter<'_> {
            fn enter_scope(&mut self, _: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
                let scope_id = scope_id.get().unwrap();
                let flags = self.scopes.get_flags(scope_id);
                self.write_line(format!("Scope {} ({flags:?}) {{", scope_id.index()));
                self.indent_in();

                let bindings = self.scopes.get_bindings(scope_id);
                if !bindings.is_empty() {
                    self.write_line("Bindings: {");
                    for (name, &symbol_id) in bindings {
                        let symbol_flags = self.symbols.get_flags(symbol_id);
                        self.write_line(format!("  {name} ({symbol_id:?} {symbol_flags:?})",));
                    }
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

    fn get_symbols_text(
        &self,
        symbols: &SymbolTable,
    ) -> Result<JsValue, serde_wasm_bindgen::Error> {
        #[derive(Serialize)]
        struct Data {
            span: Span,
            name: String,
            flags: SymbolFlags,
            scope_id: ScopeId,
            resolved_references: Vec<ReferenceId>,
            references: Vec<Reference>,
        }

        let data = symbols
            .symbol_ids()
            .map(|symbol_id| Data {
                span: symbols.get_span(symbol_id),
                name: symbols.get_name(symbol_id).into(),
                flags: symbols.get_flags(symbol_id),
                scope_id: symbols.get_scope_id(symbol_id),
                resolved_references: symbols
                    .get_resolved_reference_ids(symbol_id)
                    .iter()
                    .copied()
                    .collect::<Vec<_>>(),
                references: symbols
                    .get_resolved_reference_ids(symbol_id)
                    .iter()
                    .map(|reference_id| symbols.get_reference(*reference_id).clone())
                    .collect::<Vec<_>>(),
            })
            .collect::<Vec<_>>();

        data.serialize(&self.serializer)
    }

    fn save_diagnostics(&self, diagnostics: Vec<oxc::diagnostics::OxcDiagnostic>) {
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
                value: comment.content_span().source_text(source_text).to_string(),
                start: comment.span.start,
                end: comment.span.end,
            })
            .collect()
    }
}
