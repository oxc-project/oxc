use std::{mem, ops::ControlFlow, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};
use oxc_diagnostics::{Diagnostics, OxcDiagnostic};
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_mangler::{MangleOptions, Mangler, ManglerReturn};
use oxc_minifier::{
    CompressOptions, Compressor, ManglePropertiesOptions, PropertyMangleBailKind, PropertyMangler,
};
use oxc_parser::{ParseOptions, Parser, ParserReturn};
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
        self.compile(source_text, source_type, source_path);
        if self.errors.is_empty() {
            Ok(mem::take(&mut self.printed))
        } else {
            Err(mem::take(&mut self.errors))
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

    /// Opt-in property-name mangling (`obj.longName` -> `obj.a`). Off by default.
    ///
    /// Runs around compress + variable mangle: candidates are collected on the
    /// original (pre-compress) program and rewritten after variable mangling.
    ///
    /// Collection runs on the post-transform program. When JSX is transformed in
    /// this pipeline, JSX attribute names are lowered to plain object keys before
    /// collection and are NOT auto-reserved — users mangling JSX-transformed code
    /// must exclude component-prop names via the `exclude` regex / `reserved` list.
    fn mangle_properties_options(&self) -> Option<ManglePropertiesOptions> {
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

        /* Parse */

        let mut parser_return = self.parse(&allocator, source_text, source_type);
        if self.after_parse(&mut parser_return).is_break() {
            return;
        }
        if !parser_return.diagnostics.is_empty() {
            self.handle_errors(parser_return.diagnostics);
        }

        let mut program = parser_return.program;

        /* Isolated Declarations */
        if let Some(options) = self.isolated_declaration_options() {
            self.isolated_declaration(options, &allocator, &program, source_path);
        }

        /* Semantic */

        let mut semantic_return = self.semantic(&program);
        if !semantic_return.diagnostics.is_empty() {
            self.handle_errors(semantic_return.diagnostics);
            return;
        }
        if self.after_semantic(&mut semantic_return).is_break() {
            return;
        }

        let stats = semantic_return.semantic.stats();
        let mut scoping = semantic_return.semantic.into_scoping();

        /* Transform */

        if let Some(options) = self.transform_options() {
            let mut transformer_return =
                self.transform(options, &allocator, &mut program, source_path, scoping);

            // Errors are fatal (e.g. a React Compiler error); warnings are reported
            // but codegen still runs.
            let diagnostics = mem::take(&mut transformer_return.diagnostics);
            let has_errors = diagnostics.has_errors();
            self.handle_errors(diagnostics);
            if has_errors {
                return;
            }

            if self.after_transform(&mut program, &mut transformer_return).is_break() {
                return;
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
            let ret = InjectGlobalVariables::new(&allocator, options).build(scoping, &mut program);
            scoping = ret.scoping;
            scoping_dirty |= ret.changed;
        }

        if let Some(options) = define_options {
            let ret = ReplaceGlobalDefines::new(&allocator, options).build(scoping, &mut program);
            scoping = ret.scoping;
            scoping_dirty |= ret.changed;
        }

        /* Property mangling (collect) */

        // Collect candidates/reserved names on the original program, before compress
        // un-quotes keys. The driver is held until after variable mangling, where the
        // in-place rewrite runs (mirrors `Minifier::build`).
        let property_mangler = self
            .mangle_properties_options()
            .map(|options| self.property_mangle_collect(options, &allocator, &mut program));

        // A whole-file bail (`with` / direct `eval` / `Function` constructor) makes the
        // rewrite below a no-op; surface it as a warning so it isn't silent.
        if let Some(bail) = property_mangler.as_ref().and_then(PropertyMangler::bail) {
            let reason = match bail.kind {
                PropertyMangleBailKind::With => "a `with` statement",
                PropertyMangleBailKind::DirectEval => "a direct `eval(...)` call",
                PropertyMangleBailKind::FunctionConstructor => "the `Function` constructor",
            };
            self.handle_errors(Diagnostics::from(
                OxcDiagnostic::warn(format!(
                    "Property mangling was skipped for the whole file because it contains \
                     {reason}, which can reference property names dynamically."
                ))
                .with_label(bail.span),
            ));
        }

        /* Compress / DCE */

        // Both consumers need in-sync scoping; only rebuild when a preceding step
        // (transform or a plugin that mutated the AST) left it dirty. The branches
        // are mutually exclusive, so `scoping` is moved into exactly one of them.
        if let Some(options) = self.compress_options() {
            if scoping_dirty {
                scoping = rebuild_scoping(&program, stats, transform);
            }
            self.compress(&allocator, &mut program, scoping, options);
        } else if has_define {
            // Run DCE if minification is disabled.
            if scoping_dirty {
                scoping = rebuild_scoping(&program, stats, transform);
            }
            Compressor::new(&allocator).dead_code_elimination_with_scoping(
                &mut program,
                scoping,
                CompressOptions::dce(),
            );
        }

        /* Mangler */

        let mangler =
            self.mangle_options().map(|options| self.mangle(&mut program, options, stats));

        /* Property mangling (rewrite) */

        // Rewrite property names in place, after variable mangling.
        if let Some(property_mangler) = property_mangler {
            self.property_mangle_rewrite(property_mangler, &allocator, &mut program);
        }

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

    /// Collect property-mangling candidates on the original (pre-compress) program and
    /// rename `@__KEY__`-annotated literals, returning the driver to finish after mangling.
    ///
    /// The program is post-transform at this point (collect must see transform output,
    /// e.g. quoted keys emitted by TS enum lowering), so lowered JSX attribute names are
    /// NOT auto-reserved; reserve component-prop names explicitly when mangling JSX.
    fn property_mangle_collect<'a>(
        &self,
        options: ManglePropertiesOptions,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
    ) -> PropertyMangler {
        let mut property_mangler = PropertyMangler::new(options);
        property_mangler.collect(program);
        property_mangler.rename_annotated_literals(program, allocator);
        property_mangler
    }

    /// Rewrite property names in place, after compress and variable mangling.
    fn property_mangle_rewrite<'a>(
        &self,
        property_mangler: PropertyMangler,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
    ) {
        property_mangler.rewrite(program, allocator);
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
        let (scoping, class_private_mappings) = mangler_return
            .map(|m| (Some(m.scoping), Some(m.class_private_mappings)))
            .unwrap_or_default();
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
