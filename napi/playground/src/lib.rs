use std::{
    cell::Cell,
    path::{Path, PathBuf},
    sync::Arc,
};

use options::OxcCodegenOptions;
use rustc_hash::FxHashMap;

use napi::Either;
use napi_derive::napi;
use serde::Serialize;

use oxc::{
    allocator::Allocator,
    ast::ast::Program,
    ast_visit::Visit,
    codegen::{Codegen, CodegenOptions, CommentOptions, LegalComment},
    diagnostics::OxcDiagnostic,
    isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions},
    mangler::{MangleOptions, MangleOptionsKeepNames},
    minifier::{CompressOptions, Minifier, MinifierOptions},
    parser::{ParseOptions, Parser, ParserReturn},
    semantic::{
        ReferenceId, ScopeFlags, ScopeId, Scoping, SemanticBuilder, SymbolFlags, SymbolId,
        dot::{DebugDot, DebugDotContext},
    },
    span::{SourceType, Span},
    syntax::reference::ReferenceFlags,
    transformer::{TransformOptions, Transformer},
};
use oxc_formatter::{FormatOptions, Formatter};
use oxc_index::Idx;
use oxc_linter::{
    ConfigStore, ConfigStoreBuilder, ContextSubHost, ExternalPluginStore, LintOptions, Linter,
    ModuleRecord, Oxlintrc,
};
use oxc_napi::{Comment, OxcError, convert_utf8_to_utf16};
use oxc_transformer_plugins::{
    InjectGlobalVariables, InjectGlobalVariablesConfig, InjectImport, ReplaceGlobalDefines,
    ReplaceGlobalDefinesConfig,
};

use crate::options::{
    OxcControlFlowOptions, OxcDefineOptions, OxcInjectOptions, OxcIsolatedDeclarationsOptions,
    OxcLinterOptions, OxcOptions, OxcParserOptions, OxcRunOptions, OxcTransformerOptions,
};

mod options;

#[derive(Default)]
#[napi]
pub struct Oxc {
    #[napi(ts_type = "object")]
    pub ast: (),
    pub ast_json: String,
    pub ir: String,
    pub control_flow_graph: String,
    pub symbols_json: String,
    pub scope_text: String,
    pub codegen_text: String,
    pub codegen_sourcemap_text: Option<String>,
    pub formatted_text: String,
    pub formatter_formatted_text: String,
    pub formatter_ir_text: String,
    comments: Vec<Comment>,
    diagnostics: Vec<OxcDiagnostic>,
    source_text: String,
}

#[napi]
impl Oxc {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[napi]
    pub fn get_diagnostics(&self) -> Vec<OxcError> {
        OxcError::from_diagnostics("", &self.source_text, self.diagnostics.clone())
    }

    #[napi]
    pub fn get_comments(&self) -> Vec<Comment> {
        self.comments.clone()
    }

    /// # Errors
    /// Serde serialization error
    #[napi]
    #[allow(clippy::allow_attributes, clippy::needless_pass_by_value)]
    pub fn run(&mut self, source_text: String, options: OxcOptions) -> napi::Result<()> {
        // Initialize state
        self.source_text.clone_from(&source_text);
        self.diagnostics = vec![];
        self.scope_text = String::new();
        self.symbols_json = String::new();

        // Extract options
        let OxcOptions {
            run: ref run_options,
            parser: ref parser_options,
            linter: ref linter_options,
            transformer: ref transform_options,
            control_flow: ref control_flow_options,
            isolated_declarations: ref isolated_declarations_options,
            codegen: ref codegen_options,
            inject: ref inject_options,
            define: ref define_options,
            ..
        } = options;
        let linter_options = linter_options.clone().unwrap_or_default();
        let transform_options = transform_options.clone().unwrap_or_default();
        let control_flow_options = control_flow_options.clone().unwrap_or_default();
        let codegen_options = codegen_options.clone().unwrap_or_default();
        let inject_options = inject_options.clone();
        let define_options = define_options.clone();

        let allocator = Allocator::default();

        // Setup path and source type
        let filename = format!("test.{}", parser_options.extension);
        let path = PathBuf::from(filename);
        let source_type =
            SourceType::from_path(&path).map_err(|e| napi::Error::from_reason(e.to_string()))?;

        // Phase 1: Parse source
        let (mut program, mut module_record) =
            self.parse_source(&allocator, &source_text, source_type, parser_options);

        // Phase 2: Build semantic analysis
        let semantic = self.build_semantic(&program, run_options, &control_flow_options);

        // Phase 3: Run linter
        let linter_module_record = Arc::new(ModuleRecord::new(&path, &module_record, &semantic));
        self.run_linter(
            run_options,
            &linter_options,
            &path,
            &program,
            &linter_module_record,
            &allocator,
        );

        // Phase 4: Run formatter
        let parse_options = ParseOptions {
            parse_regular_expression: true,
            allow_return_outside_function: parser_options.allow_return_outside_function,
            preserve_parens: parser_options.preserve_parens,
            allow_v8_intrinsics: parser_options.allow_v8_intrinsics,
        };
        self.run_formatter(run_options, parse_options, &source_text, source_type);

        let mut scoping = semantic.into_scoping();

        // Extract scope and symbol information if needed
        if !source_type.is_typescript_definition() {
            if run_options.scope {
                self.scope_text = Self::get_scope_text(&program, &scoping);
            }
            if run_options.symbol {
                self.symbols_json = Self::get_symbols_text(&scoping)?;
            }
        }

        // Phase 5: Handle isolated declarations (early return path)
        if run_options.isolated_declarations {
            self.process_isolated_declarations(
                &allocator,
                &path,
                &program,
                run_options,
                &codegen_options,
                isolated_declarations_options.clone(),
            );
            return Ok(());
        }

        // Phase 5.5: Apply ReplaceGlobalDefines (before transformations)
        if let Some(define_opts) = define_options {
            let define_config = Self::build_define_config(&define_opts)?;
            let ret =
                ReplaceGlobalDefines::new(&allocator, define_config).build(scoping, &mut program);
            scoping = ret.scoping;
        }

        // Phase 6: Apply transformations
        if run_options.transform {
            scoping = self.apply_transformations(
                &allocator,
                &path,
                &mut program,
                scoping,
                &transform_options,
            );
        }

        // Phase 6.5: Apply InjectGlobalVariables (after transformations)
        if let Some(inject_opts) = inject_options {
            let inject_config = Self::build_inject_config(&inject_opts)?;
            let _ =
                InjectGlobalVariables::new(&allocator, inject_config).build(scoping, &mut program);
        }

        // Phase 7: Apply minification
        let scoping = Self::apply_minification(&allocator, &mut program, &options);

        // Phase 8: Generate code
        self.codegen(&path, &program, scoping, run_options, &codegen_options);

        // Phase 9: Finalize output
        self.finalize_output(&source_text, &mut program, &mut module_record, source_type);

        Ok(())
    }

    fn parse_source<'a>(
        &mut self,
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        parser_options: &OxcParserOptions,
    ) -> (Program<'a>, oxc::syntax::module_record::ModuleRecord<'a>) {
        let parser_options = ParseOptions {
            parse_regular_expression: true,
            allow_return_outside_function: parser_options.allow_return_outside_function,
            preserve_parens: parser_options.preserve_parens,
            allow_v8_intrinsics: parser_options.allow_v8_intrinsics,
        };
        let ParserReturn { program, errors, module_record, .. } =
            Parser::new(allocator, source_text, source_type).with_options(parser_options).parse();
        self.diagnostics.extend(errors);
        (program, module_record)
    }

    fn build_semantic<'a>(
        &mut self,
        program: &'a Program<'a>,
        run_options: &OxcRunOptions,
        control_flow_options: &OxcControlFlowOptions,
    ) -> oxc::semantic::Semantic<'a> {
        let mut semantic_builder = SemanticBuilder::new();
        if run_options.transform {
            // Estimate transformer will triple scopes, symbols, references
            semantic_builder = semantic_builder.with_excess_capacity(2.0);
        }
        let semantic_ret =
            semantic_builder.with_check_syntax_error(true).with_cfg(true).build(program);
        self.diagnostics.extend(semantic_ret.errors);

        self.control_flow_graph = semantic_ret.semantic.cfg().map_or_else(String::default, |cfg| {
            cfg.debug_dot(DebugDotContext::new(
                semantic_ret.semantic.nodes(),
                control_flow_options.verbose.unwrap_or_default(),
            ))
        });

        semantic_ret.semantic
    }

    fn process_isolated_declarations<'a>(
        &mut self,
        allocator: &'a Allocator,
        path: &Path,
        program: &Program<'a>,
        run_options: &OxcRunOptions,
        codegen_options: &OxcCodegenOptions,
        isolated_declarations_options: Option<OxcIsolatedDeclarationsOptions>,
    ) {
        let id_options = isolated_declarations_options
            .map(|o| IsolatedDeclarationsOptions { strip_internal: o.strip_internal })
            .unwrap_or_default();
        let ret = IsolatedDeclarations::new(allocator, id_options).build(program);
        if ret.errors.is_empty() {
            self.codegen(path, &ret.program, None, run_options, codegen_options);
        } else {
            self.diagnostics.extend(ret.errors);
            self.codegen_text = String::new();
            self.codegen_sourcemap_text = None;
        }
    }

    fn apply_transformations<'a>(
        &mut self,
        allocator: &'a Allocator,
        path: &Path,
        program: &mut Program<'a>,
        scoping: Scoping,
        transform_options: &OxcTransformerOptions,
    ) -> Scoping {
        let mut options = transform_options
            .target
            .as_ref()
            .and_then(|target| {
                TransformOptions::from_target(target)
                    .map_err(|err| {
                        self.diagnostics.push(OxcDiagnostic::error(err));
                    })
                    .ok()
            })
            .unwrap_or_default();
        options.assumptions.set_public_class_fields =
            !transform_options.use_define_for_class_fields;
        options.typescript.remove_class_fields_without_initializer =
            !transform_options.use_define_for_class_fields;
        options.decorator.legacy = transform_options.experimental_decorators;
        options.decorator.emit_decorator_metadata = transform_options.emit_decorator_metadata;
        let result =
            Transformer::new(allocator, path, &options).build_with_scoping(scoping, program);
        self.diagnostics.extend(result.errors);
        result.scoping
    }

    fn apply_minification<'a>(
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        options: &OxcOptions,
    ) -> Option<Scoping> {
        if options.run.compress || options.run.mangle {
            let compress = options.compress.map(|_| CompressOptions::smallest());
            let mangle = options.mangle.map(|o| MangleOptions {
                top_level: o.top_level,
                keep_names: MangleOptionsKeepNames { function: o.keep_names, class: o.keep_names },
                debug: false,
            });
            Minifier::new(MinifierOptions { mangle, compress }).minify(allocator, program).scoping
        } else {
            None
        }
    }

    fn finalize_output<'a>(
        &mut self,
        source_text: &str,
        program: &mut Program<'a>,
        module_record: &mut oxc::syntax::module_record::ModuleRecord<'a>,
        source_type: SourceType,
    ) {
        self.ir = format!("{:#?}", program.body);
        let mut comments = convert_utf8_to_utf16(source_text, program, module_record, &mut []);

        self.ast_json = if source_type.is_javascript() {
            // Add hashbang to start of comments
            if let Some(hashbang) = &program.hashbang {
                comments.insert(
                    0,
                    Comment {
                        r#type: "Line".to_string(),
                        value: hashbang.value.to_string(),
                        start: hashbang.span.start,
                        end: hashbang.span.end,
                    },
                );
            }

            program.to_pretty_estree_js_json_with_fixes(false)
        } else {
            program.to_pretty_estree_ts_json_with_fixes(false)
        };
        self.comments = comments;
    }

    fn run_linter(
        &mut self,
        run_options: &OxcRunOptions,
        linter_options: &OxcLinterOptions,
        path: &Path,
        program: &Program,
        module_record: &Arc<ModuleRecord>,
        allocator: &Allocator,
    ) {
        // Only lint if there are no syntax errors
        if run_options.lint && self.diagnostics.is_empty() {
            let external_plugin_store = ExternalPluginStore::default();
            let semantic_ret = SemanticBuilder::new().with_cfg(true).build(program);
            let semantic = semantic_ret.semantic;
            let lint_config = if linter_options.config.is_some() {
                let oxlintrc =
                    Oxlintrc::from_string(&linter_options.config.as_ref().unwrap().to_string())
                        .unwrap_or_default();
                let config_builder = ConfigStoreBuilder::from_oxlintrc(
                    false,
                    oxlintrc,
                    None,
                    &mut ExternalPluginStore::default(),
                )
                .unwrap_or_default();
                config_builder.build(&external_plugin_store)
            } else {
                ConfigStoreBuilder::default().build(&external_plugin_store)
            };
            let lint_config = lint_config.unwrap();
            let linter_ret = Linter::new(
                LintOptions::default(),
                ConfigStore::new(lint_config, FxHashMap::default(), external_plugin_store),
                None,
            )
            .run(
                path,
                vec![ContextSubHost::new(semantic, Arc::clone(module_record), 0)],
                allocator,
            );
            self.diagnostics.extend(linter_ret.into_iter().map(|e| e.error));
        }
    }

    fn run_formatter(
        &mut self,
        run_options: &OxcRunOptions,
        parser_options: ParseOptions,
        source_text: &str,
        source_type: SourceType,
    ) {
        let allocator = Allocator::default();
        if run_options.formatter || run_options.formatter_ir {
            let ret = Parser::new(&allocator, source_text, source_type)
                .with_options(ParseOptions { preserve_parens: false, ..parser_options })
                .parse();

            let formatter = Formatter::new(&allocator, FormatOptions::default());
            self.formatter_formatted_text = formatter.build(&ret.program);

            // if run_options.formatter_ir.unwrap_or_default() {
            // let formatter_doc = formatter.doc(&ret.program).to_string();
            // self.formatter_ir_text = {
            // let ret =
            // Parser::new(&allocator, &formatter_doc, SourceType::default()).parse();
            // Formatter::new(&allocator, FormatOptions::default()).build(&ret.program)
            // };
            // }
        }
    }

    fn codegen(
        &mut self,
        path: &Path,
        program: &Program<'_>,
        scoping: Option<Scoping>,
        run_options: &OxcRunOptions,
        codegen_options: &OxcCodegenOptions,
    ) {
        let options = CodegenOptions {
            minify: run_options.whitespace,
            comments: CommentOptions {
                normal: codegen_options.normal,
                jsdoc: codegen_options.jsdoc,
                annotation: codegen_options.annotation,
                legal: if codegen_options.legal {
                    LegalComment::Inline
                } else {
                    LegalComment::None
                },
            },
            source_map_path: Some(path.to_path_buf()),
            ..CodegenOptions::default()
        };
        let codegen_result =
            Codegen::new().with_scoping(scoping).with_options(options).build(program);
        self.codegen_text = codegen_result.code;
        self.codegen_sourcemap_text = codegen_result.map.map(|map| map.to_json_string());
    }

    fn get_scope_text(program: &Program<'_>, scoping: &Scoping) -> String {
        struct ScopesTextWriter<'s> {
            scoping: &'s Scoping,
            scope_text: String,
            indent: usize,
            space: String,
        }

        impl<'s> ScopesTextWriter<'s> {
            fn new(scoping: &'s Scoping) -> Self {
                Self { scoping, scope_text: String::new(), indent: 0, space: String::new() }
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
                let flags = self.scoping.scope_flags(scope_id);
                self.write_line(format!("Scope {} ({flags:?}) {{", scope_id.index()));
                self.indent_in();

                let bindings = self.scoping.get_bindings(scope_id);
                if !bindings.is_empty() {
                    self.write_line("Bindings: {");
                    for (name, &symbol_id) in bindings {
                        let symbol_flags = self.scoping.symbol_flags(symbol_id);
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

        let mut writer = ScopesTextWriter::new(scoping);
        writer.visit_program(program);
        writer.scope_text
    }

    fn build_inject_config(
        inject_opts: &OxcInjectOptions,
    ) -> napi::Result<InjectGlobalVariablesConfig> {
        let mut imports = Vec::new();

        for (local, value) in &inject_opts.inject {
            let import = match value {
                Either::A(source) => InjectImport::default_specifier(source, local),
                Either::B(v) => {
                    if v.len() != 2 {
                        return Err(napi::Error::from_reason(
                            "Inject plugin did not receive a tuple [string, string].",
                        ));
                    }
                    let source = &v[0];
                    if v[1] == "*" {
                        InjectImport::namespace_specifier(source, local)
                    } else {
                        InjectImport::named_specifier(source, Some(&v[1]), local)
                    }
                }
            };
            imports.push(import);
        }

        Ok(InjectGlobalVariablesConfig::new(imports))
    }

    fn build_define_config(
        define_opts: &OxcDefineOptions,
    ) -> napi::Result<ReplaceGlobalDefinesConfig> {
        let define_pairs: Vec<(String, String)> =
            define_opts.define.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        ReplaceGlobalDefinesConfig::new(&define_pairs).map_err(|errors| {
            let error_messages: Vec<String> = errors.iter().map(ToString::to_string).collect();
            napi::Error::from_reason(format!(
                "Invalid define config: {}",
                error_messages.join(", ")
            ))
        })
    }

    fn get_symbols_text(scoping: &Scoping) -> napi::Result<String> {
        #[derive(Serialize)]
        struct Data {
            symbol_id: SymbolId,
            span: Span,
            name: String,
            flags: SymbolFlags,
            scope_id: ScopeId,
            references: Vec<ReferenceData>,
        }
        #[derive(Serialize)]
        struct ReferenceData {
            reference_id: ReferenceId,
            symbol_id: Option<SymbolId>,
            flags: ReferenceFlags,
        }
        let data = scoping
            .symbol_ids()
            .map(|symbol_id| Data {
                symbol_id,
                span: scoping.symbol_span(symbol_id),
                name: scoping.symbol_name(symbol_id).into(),
                flags: scoping.symbol_flags(symbol_id),
                scope_id: scoping.symbol_scope_id(symbol_id),
                references: scoping
                    .get_resolved_reference_ids(symbol_id)
                    .iter()
                    .map(|&reference_id| {
                        let reference = scoping.get_reference(reference_id);
                        ReferenceData {
                            reference_id,
                            symbol_id: reference.symbol_id(),
                            flags: reference.flags(),
                        }
                    })
                    .collect::<Vec<_>>(),
            })
            .collect::<Vec<_>>();

        serde_json::to_string_pretty(&data).map_err(|e| napi::Error::from_reason(e.to_string()))
    }
}
