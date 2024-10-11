use std::{mem, ops::ControlFlow, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_codegen::{CodeGenerator, CodegenOptions, CodegenReturn, CommentOptions};
use oxc_diagnostics::OxcDiagnostic;
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_mangler::{MangleOptions, Mangler};
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::{ParseOptions, Parser, ParserReturn};
use oxc_semantic::{ScopeTree, SemanticBuilder, SemanticBuilderReturn, SymbolTable};
use oxc_span::SourceType;
use oxc_transformer::{
    InjectGlobalVariables, InjectGlobalVariablesConfig, ReplaceGlobalDefines,
    ReplaceGlobalDefinesConfig, TransformOptions, Transformer, TransformerReturn,
};

#[derive(Default)]
pub struct Compiler {
    printed: String,
    errors: Vec<OxcDiagnostic>,
}

impl CompilerInterface for Compiler {
    fn handle_errors(&mut self, errors: Vec<OxcDiagnostic>) {
        self.errors.extend(errors);
    }

    fn after_codegen(&mut self, ret: CodegenReturn) {
        self.printed = ret.code;
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

    fn enable_sourcemap(&self) -> bool {
        false
    }

    fn parse_options(&self) -> ParseOptions {
        ParseOptions::default()
    }

    fn isolated_declaration_options(&self) -> Option<IsolatedDeclarationsOptions> {
        None
    }

    fn transform_options(&self) -> Option<TransformOptions> {
        Some(TransformOptions::default())
    }

    fn define_options(&self) -> Option<ReplaceGlobalDefinesConfig> {
        None
    }

    fn inject_options(&self) -> Option<InjectGlobalVariablesConfig> {
        None
    }

    fn compress_options(&self) -> Option<CompressOptions> {
        None
    }

    fn mangle_options(&self) -> Option<MangleOptions> {
        None
    }

    fn codegen_options(&self) -> Option<CodegenOptions> {
        Some(CodegenOptions::default())
    }

    fn check_semantic_error(&self) -> bool {
        true
    }

    fn semantic_child_scope_ids(&self) -> bool {
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

    fn after_isolated_declarations(&mut self, _ret: CodegenReturn) {}

    fn after_transform(
        &mut self,
        _program: &mut Program<'_>,
        _transformer_return: &mut TransformerReturn,
    ) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn after_codegen(&mut self, _ret: CodegenReturn) {}

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

        let mut program = parser_return.program;

        /* Isolated Declarations */
        if let Some(options) = self.isolated_declaration_options() {
            self.isolated_declaration(options, &allocator, &program, source_path);
        }

        /* Semantic */

        let mut semantic_return = self.semantic(&program, source_path);
        if !semantic_return.errors.is_empty() {
            self.handle_errors(semantic_return.errors);
            return;
        }
        if self.after_semantic(&mut program, &mut semantic_return).is_break() {
            return;
        }

        let (mut symbols, mut scopes) = semantic_return.semantic.into_symbol_table_and_scope_tree();

        /* Transform */

        if let Some(options) = self.transform_options() {
            let mut transformer_return =
                self.transform(options, &allocator, &mut program, source_path, symbols, scopes);

            if !transformer_return.errors.is_empty() {
                self.handle_errors(transformer_return.errors);
                return;
            }

            if self.after_transform(&mut program, &mut transformer_return).is_break() {
                return;
            }

            symbols = transformer_return.symbols;
            scopes = transformer_return.scopes;
        }

        if let Some(config) = self.inject_options() {
            let ret =
                InjectGlobalVariables::new(&allocator, config).build(symbols, scopes, &mut program);
            symbols = ret.symbols;
            scopes = ret.scopes;
        }

        if let Some(config) = self.define_options() {
            let ret =
                ReplaceGlobalDefines::new(&allocator, config).build(symbols, scopes, &mut program);
            Compressor::new(&allocator, CompressOptions::dead_code_elimination())
                .build_with_symbols_and_scopes(ret.symbols, ret.scopes, &mut program);
            // symbols = ret.symbols;
            // scopes = ret.scopes;
        }

        /* Compress */

        if let Some(options) = self.compress_options() {
            self.compress(&allocator, &mut program, options);
        }

        /* Mangler */

        let mangler = self.mangle_options().map(|options| self.mangle(&mut program, options));

        /* Codegen */

        if let Some(options) = self.codegen_options() {
            let ret = self.codegen(&program, source_path, mangler, options);
            self.after_codegen(ret);
        }
    }

    fn parse<'a>(
        &self,
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
    ) -> ParserReturn<'a> {
        Parser::new(allocator, source_text, source_type).with_options(self.parse_options()).parse()
    }

    fn semantic<'a>(&self, program: &Program<'a>, source_path: &Path) -> SemanticBuilderReturn<'a> {
        let mut builder = SemanticBuilder::new();

        if self.transform_options().is_some() {
            // Estimate transformer will triple scopes, symbols, references
            builder = builder.with_excess_capacity(2.0);
        }

        builder
            .with_check_syntax_error(self.check_semantic_error())
            .with_scope_tree_child_ids(self.semantic_child_scope_ids())
            .build_module_record(source_path, program)
            .build(program)
    }

    fn isolated_declaration<'a>(
        &mut self,
        options: IsolatedDeclarationsOptions,
        allocator: &'a Allocator,
        program: &Program<'a>,
        source_path: &Path,
    ) {
        let ret = IsolatedDeclarations::new(allocator, options).build(program);
        self.handle_errors(ret.errors);
        let ret = self.codegen(
            &ret.program,
            source_path,
            None,
            self.codegen_options().unwrap_or_default(),
        );
        self.after_isolated_declarations(ret);
    }

    #[allow(clippy::too_many_arguments)]
    fn transform<'a>(
        &self,
        options: TransformOptions,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        source_path: &Path,
        symbols: SymbolTable,
        scopes: ScopeTree,
    ) -> TransformerReturn {
        Transformer::new(allocator, source_path, options)
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

    fn mangle(&self, program: &mut Program<'_>, options: MangleOptions) -> Mangler {
        Mangler::new().with_options(options).build(program)
    }

    fn codegen(
        &self,
        program: &Program<'_>,
        source_path: &Path,
        mangler: Option<Mangler>,
        options: CodegenOptions,
    ) -> CodegenReturn {
        let comment_options = CommentOptions { preserve_annotate_comments: true };
        let mut codegen = CodeGenerator::new()
            .with_options(options)
            .with_mangler(mangler)
            .enable_comment(program, comment_options);
        if self.enable_sourcemap() {
            codegen = codegen
                .enable_source_map(source_path.to_string_lossy().as_ref(), program.source_text);
        }
        codegen.build(program)
    }
}
