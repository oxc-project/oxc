use std::{mem, ops::ControlFlow, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};
use oxc_diagnostics::Diagnostics;
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_mangler::{MangleOptions, Mangler, ManglerReturn};
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::{ParseOptions, ParserReturn, Source};
use oxc_semantic::{Scoping, SemanticBuilder, SemanticBuilderReturn, Stats};
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer, TransformerReturn};
use oxc_transformer_plugins::{
    InjectGlobalVariables, InjectGlobalVariablesConfig, ReplaceGlobalDefines,
    ReplaceGlobalDefinesConfig,
};

#[derive(Default)]
pub struct Compiler {
    printed: String,
    errors: Diagnostics,
}

impl CompilerInterface for Compiler {
    fn handle_errors(&mut self, errors: Diagnostics) {
        self.errors.extend(errors);
    }

    fn after_codegen(&mut self, ret: CodegenReturn<'_>) {
        self.printed = ret.code;
    }
}

impl Compiler {
    /// # Errors
    ///
    /// * The accumulated [Diagnostics].
    pub fn execute(
        &mut self,
        source_text: &str,
        source_type: SourceType,
        source_path: &Path,
    ) -> Result<String, Diagnostics> {
        let allocator = Allocator::default();
        let source = Source::from_str(source_text, source_type, &allocator);
        self.execute_source(source, source_path).map_err(|(errors, _)| errors)
    }

    /// Compile a [`Source`], and return the printed code.
    ///
    /// Same as [`Compiler::execute`], but takes a [`Source`] instead of source text,
    /// so the source text does not need to be copied.
    /// AST and all other data are allocated in the [`Source`]'s allocator.
    ///
    /// # Errors
    ///
    /// * The accumulated [Diagnostics], and the source text, for rendering them.
    ///   The source text lives in the [`Source`]'s allocator.
    pub fn execute_source<'a>(
        &mut self,
        source: Source<'a>,
        source_path: &Path,
    ) -> Result<String, (Diagnostics, &'a str)> {
        let source_text = self.compile_source(source, source_path);
        if self.errors.is_empty() {
            Ok(mem::take(&mut self.printed))
        } else {
            Err((mem::take(&mut self.errors), source_text))
        }
    }
}

pub trait CompilerInterface {
    fn handle_errors(&mut self, _errors: Diagnostics) {}

    fn enable_sourcemap(&self) -> bool {
        false
    }

    fn parse_options(&self) -> ParseOptions {
        ParseOptions::default()
    }

    fn isolated_declaration_options(&self) -> Option<IsolatedDeclarationsOptions> {
        None
    }

    fn transform_options(&self) -> Option<&TransformOptions> {
        None
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

    /// Whether to build the full `AstNodes` store during semantic analysis.
    ///
    /// Off by default (the compiler pipeline only needs scoping). Override to
    /// `true` if [`Self::after_semantic`] reads [`Semantic::nodes`].
    ///
    /// [`Semantic::nodes`]: oxc_semantic::Semantic::nodes
    fn build_semantic_nodes(&self) -> bool {
        false
    }

    fn after_parse(&mut self, _parser_return: &mut ParserReturn) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn after_semantic(&mut self, _semantic_return: &mut SemanticBuilderReturn) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn after_isolated_declarations(&mut self, _ret: CodegenReturn<'_>) {}

    fn after_transform(
        &mut self,
        _program: &mut Program<'_>,
        _transformer_return: &mut TransformerReturn,
    ) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn after_codegen(&mut self, _ret: CodegenReturn<'_>) {}

    fn compile(&mut self, source_text: &str, source_type: SourceType, source_path: &Path) {
        let allocator = Allocator::default();
        let source = Source::from_str(source_text, source_type, &allocator);
        self.compile_source(source, source_path);
    }

    /// Compile a [`Source`].
    ///
    /// Same as [`CompilerInterface::compile`], but takes a [`Source`] instead of source text,
    /// so the source text does not need to be copied.
    /// AST and all other data are allocated in the [`Source`]'s allocator.
    ///
    /// Returns the source text, for rendering diagnostics. It lives in the [`Source`]'s allocator.
    /// It's an empty string if the source could not be parsed at all
    /// (it's not valid UTF-8, or is too long).
    fn compile_source<'a>(&mut self, source: Source<'a>, source_path: &Path) -> &'a str {
        let allocator = source.allocator();

        /* Parse */

        let mut parser_return = self.parse(source);
        let source_text = parser_return.program.source_text;
        if self.after_parse(&mut parser_return).is_break() {
            return source_text;
        }
        if !parser_return.diagnostics.is_empty() {
            self.handle_errors(parser_return.diagnostics);
        }

        let mut program = parser_return.program;

        /* Isolated Declarations */
        if let Some(options) = self.isolated_declaration_options() {
            self.isolated_declaration(options, allocator, &program, source_path);
        }

        /* Semantic */

        let mut semantic_return = self.semantic(&program);
        if !semantic_return.diagnostics.is_empty() {
            self.handle_errors(semantic_return.diagnostics);
            return source_text;
        }
        if self.after_semantic(&mut semantic_return).is_break() {
            return source_text;
        }

        let stats = semantic_return.semantic.stats();
        let mut scoping = semantic_return.semantic.into_scoping();

        /* Transform */

        if let Some(options) = self.transform_options() {
            let mut transformer_return =
                self.transform(options, allocator, &mut program, source_path, scoping);

            // Errors are fatal (e.g. a React Compiler error); warnings are reported
            // but codegen still runs.
            let diagnostics = mem::take(&mut transformer_return.diagnostics);
            let has_errors = diagnostics.has_errors();
            self.handle_errors(diagnostics);
            if has_errors {
                return source_text;
            }

            if self.after_transform(&mut program, &mut transformer_return).is_break() {
                return source_text;
            }

            scoping = transformer_return.scoping;
        }

        // The transformer leaves symbols and scopes out of sync with the AST;
        // the parser and any fresh semantic build leave them in sync. Track the
        // state so each consumer below only rebuilds when needed.
        let transform = self.transform_options().is_some();
        let mut scoping_dirty = transform;

        let inject_options = self.inject_options();
        let define_options = self.define_options();
        let has_define = define_options.is_some();

        // The inject/define plugins require in-sync scoping as input.
        if scoping_dirty && (inject_options.is_some() || has_define) {
            scoping = rebuild_scoping(&program, stats, transform);
            scoping_dirty = false;
        }

        if let Some(options) = inject_options {
            let ret = InjectGlobalVariables::new(allocator, options).build(scoping, &mut program);
            scoping = ret.scoping;
            scoping_dirty |= ret.changed;
        }

        if let Some(options) = define_options {
            let ret = ReplaceGlobalDefines::new(allocator, options).build(scoping, &mut program);
            scoping = ret.scoping;
            scoping_dirty |= ret.changed;
        }

        /* Compress / DCE */

        // Both consumers need in-sync scoping; only rebuild when a preceding step
        // (transform or a plugin that mutated the AST) left it dirty. The branches
        // are mutually exclusive, so `scoping` is moved into exactly one of them.
        if let Some(options) = self.compress_options() {
            if scoping_dirty {
                scoping = rebuild_scoping(&program, stats, transform);
            }
            self.compress(allocator, &mut program, scoping, options);
        } else if has_define {
            // Run DCE if minification is disabled.
            if scoping_dirty {
                scoping = rebuild_scoping(&program, stats, transform);
            }
            Compressor::new(allocator).dead_code_elimination_with_scoping(
                &mut program,
                scoping,
                CompressOptions::dce(),
            );
        }

        /* Mangler */

        let mangler =
            self.mangle_options().map(|options| self.mangle(&mut program, options, stats));

        /* Codegen */

        if let Some(options) = self.codegen_options() {
            let ret = self.codegen(&program, source_path, mangler, options);
            self.after_codegen(ret);
        }

        source_text
    }

    fn parse<'a>(&self, source: Source<'a>) -> ParserReturn<'a> {
        source.parse_with_options(self.parse_options())
    }

    fn semantic<'a>(&self, program: &'a Program<'a>) -> SemanticBuilderReturn<'a> {
        let mut builder = SemanticBuilder::new_compiler();

        if self.transform_options().is_some() {
            // Estimate transformer will triple scopes, symbols, references
            builder = builder.with_excess_capacity(2.0).with_enum_eval(true);
        }

        builder
            .with_check_syntax_error(self.check_semantic_error())
            .with_build_nodes(self.build_semantic_nodes())
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
        self.handle_errors(ret.diagnostics);
        let ret = self.codegen(
            &ret.program,
            source_path,
            None,
            self.codegen_options().unwrap_or_default(),
        );
        self.after_isolated_declarations(ret);
    }

    fn transform<'a>(
        &self,
        options: &TransformOptions,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        source_path: &Path,
        scoping: Scoping,
    ) -> TransformerReturn {
        Transformer::new(allocator, source_path, options).build_with_scoping(scoping, program)
    }

    fn compress<'a>(
        &self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        scoping: Scoping,
        options: CompressOptions,
    ) {
        Compressor::new(allocator).build_with_scoping(program, scoping, options);
    }

    /// `stats` from a prior semantic build size the mangler's internal semantic
    /// rebuild, avoiding a full-AST counting pass.
    fn mangle(
        &self,
        program: &mut Program<'_>,
        options: MangleOptions,
        stats: Stats,
    ) -> ManglerReturn {
        Mangler::new().with_options(options).with_stats(stats).build(program)
    }

    fn codegen<'a>(
        &self,
        program: &Program<'a>,
        source_path: &Path,
        mangler_return: Option<ManglerReturn>,
        options: CodegenOptions,
    ) -> CodegenReturn<'a> {
        let mut options = options;
        if self.enable_sourcemap() {
            options.source_map_path = Some(source_path.to_path_buf());
        }
        let (scoping, class_private_mappings) =
            mangler_return.map_or_default(|m| (Some(m.scoping), Some(m.class_private_mappings)));
        Codegen::new()
            .with_options(options)
            .with_scoping(scoping)
            .with_private_member_mappings(class_private_mappings)
            .build(program)
    }
}

/// Rebuild [`Scoping`] from scratch, reusing prior `stats` to size allocations.
///
/// Used to re-sync symbols and scopes with the AST after a step that left them
/// out of date (the transformer, or an inject/define plugin that mutated nodes).
///
/// `enum_eval` must match the original semantic build so downstream transforms
/// see the same const-enum resolution.
fn rebuild_scoping(program: &Program<'_>, stats: Stats, enum_eval: bool) -> Scoping {
    SemanticBuilder::new()
        .with_stats(stats)
        .with_enum_eval(enum_eval)
        .build(program)
        .semantic
        .into_scoping()
}
