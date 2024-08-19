use std::{mem, ops::ControlFlow, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::{ast::Program, Trivias};
use oxc_codegen::{CodeGenerator, CodegenOptions, CommentOptions, WhitespaceRemover};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::{ParseOptions, Parser, ParserReturn};
use oxc_span::SourceType;

use oxc_minifier::{CompressOptions, Compressor};
use oxc_semantic::{ScopeTree, SemanticBuilder, SemanticBuilderReturn, SymbolTable};
use oxc_transformer::{TransformOptions, Transformer, TransformerReturn};

#[derive(Default)]
pub struct Compiler {
    printed: String,
    errors: Vec<OxcDiagnostic>,
}

impl CompilerInterface for Compiler {
    fn handle_errors(&mut self, errors: Vec<OxcDiagnostic>) {
        self.errors.extend(errors);
    }

    fn after_codegen(&mut self, printed: String) {
        self.printed = printed;
    }
}

impl Compiler {
    /// # Errors
    ///
    /// * A list of [OxcDiagnostic].
    pub fn execute(
        &mut self,
        source_text: &str,
        source_type: SourceType,
        source_path: &Path,
    ) -> Result<String, Vec<OxcDiagnostic>> {
        self.compile(source_text, source_type, source_path);
        if self.errors.is_empty() {
            Ok(mem::take(&mut self.printed))
        } else {
            Err(mem::take(&mut self.errors))
        }
    }
}

pub trait CompilerInterface {
    fn handle_errors(&mut self, _errors: Vec<OxcDiagnostic>) {}

    fn parser_options(&self) -> ParseOptions {
        ParseOptions::default()
    }

    fn transform_options(&self) -> Option<TransformOptions> {
        Some(TransformOptions::default())
    }

    fn compress_options(&self) -> Option<CompressOptions> {
        Some(CompressOptions::all_true())
    }

    fn codegen_options(&self) -> Option<CodegenOptions> {
        Some(CodegenOptions::default())
    }

    fn remove_whitespace(&self) -> bool {
        false
    }

    fn after_parse(&mut self, _parser_return: &mut ParserReturn) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn after_semantic(
        &mut self,
        _program: &mut Program<'_>,
        _semantic_return: &mut SemanticBuilderReturn,
    ) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn after_transform(
        &mut self,
        _program: &mut Program<'_>,
        _transformer_return: &mut TransformerReturn,
    ) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn after_codegen(&mut self, _printed: String) {}

    fn compile(&mut self, source_text: &str, source_type: SourceType, source_path: &Path) {
        let allocator = Allocator::default();

        /* Parse */

        let mut parser_return = self.parse(&allocator, source_text, source_type);
        if self.after_parse(&mut parser_return).is_break() {
            return;
        }
        if !parser_return.errors.is_empty() {
            self.handle_errors(parser_return.errors);
        }

        /* Semantic */

        let mut program = parser_return.program;
        let trivias = parser_return.trivias;

        let mut semantic_return = self.semantic(&program, source_text, source_type, source_path);
        if !semantic_return.errors.is_empty() {
            self.handle_errors(semantic_return.errors);
            return;
        }
        if self.after_semantic(&mut program, &mut semantic_return).is_break() {
            return;
        }

        let (symbols, scopes) = semantic_return.semantic.into_symbol_table_and_scope_tree();

        /* Transform */

        if let Some(options) = self.transform_options() {
            let mut transformer_return = self.transform(
                options,
                &allocator,
                &mut program,
                source_path,
                source_text,
                source_type,
                &trivias,
                symbols,
                scopes,
            );

            if !transformer_return.errors.is_empty() {
                self.handle_errors(transformer_return.errors);
                return;
            }

            if self.after_transform(&mut program, &mut transformer_return).is_break() {
                return;
            }
        }

        if let Some(options) = self.compress_options() {
            self.compress(&allocator, &mut program, options);
        }

        if let Some(options) = self.codegen_options() {
            let printed = self.codegen(&program, source_text, &trivias, options);
            self.after_codegen(printed);
        }
    }

    fn parse<'a>(
        &self,
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
    ) -> ParserReturn<'a> {
        Parser::new(allocator, source_text, source_type).with_options(self.parser_options()).parse()
    }

    fn semantic<'a>(
        &self,
        program: &Program<'a>,
        source_text: &'a str,
        source_type: SourceType,
        source_path: &Path,
    ) -> SemanticBuilderReturn<'a> {
        SemanticBuilder::new(source_text, source_type)
            .with_check_syntax_error(true)
            .build_module_record(source_path.to_path_buf(), program)
            .build(program)
    }

    #[allow(clippy::too_many_arguments)]
    fn transform<'a>(
        &self,
        options: TransformOptions,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        source_path: &Path,
        source_text: &'a str,
        source_type: SourceType,
        trivias: &Trivias,
        symbols: SymbolTable,
        scopes: ScopeTree,
    ) -> TransformerReturn {
        Transformer::new(allocator, source_path, source_type, source_text, trivias.clone(), options)
            .build_with_symbols_and_scopes(symbols, scopes, program)
    }

    fn compress<'a>(
        &self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        options: CompressOptions,
    ) {
        Compressor::new(allocator, options).build(program);
    }

    fn codegen<'a>(
        &self,
        program: &Program<'a>,
        source_text: &'a str,
        trivias: &Trivias,
        options: CodegenOptions,
    ) -> String {
        let comment_options = CommentOptions { preserve_annotate_comments: true };

        if self.remove_whitespace() {
            WhitespaceRemover::new().with_options(options).build(program).source_text
        } else {
            CodeGenerator::new()
                .with_options(options)
                .enable_comment(source_text, trivias.clone(), comment_options)
                .build(program)
                .source_text
        }
    }
}
