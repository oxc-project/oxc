use std::{
    cell::Cell,
    path::{Path, PathBuf},
    sync::Arc,
};

use options::OxcCodegenOptions;
use rustc_hash::FxHashMap;

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

use crate::options::{OxcLinterOptions, OxcOptions, OxcRunOptions};

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
        self.source_text.clone_from(&source_text);
        self.diagnostics = vec![];
        self.scope_text = String::new();
        self.symbols_json = String::new();

        let OxcOptions {
            run: run_options,
            parser: parser_options,
            linter: linter_options,
            transformer: transform_options,
            control_flow: control_flow_options,
            isolated_declarations: isolated_declarations_options,
            codegen: codegen_options,
            ..
        } = options;
        let linter_options = linter_options.unwrap_or_default();
        let transform_options = transform_options.unwrap_or_default();
        let control_flow_options = control_flow_options.unwrap_or_default();
        let codegen_options = codegen_options.unwrap_or_default();

        let allocator = Allocator::default();

        let filename = format!("test.{}", parser_options.extension);
        let path = PathBuf::from(filename);
        let source_type =
            SourceType::from_path(&path).map_err(|e| napi::Error::from_reason(e.to_string()))?;

        let parser_options = ParseOptions {
            parse_regular_expression: true,
            allow_return_outside_function: parser_options.allow_return_outside_function,
            preserve_parens: parser_options.preserve_parens,
            allow_v8_intrinsics: parser_options.allow_v8_intrinsics,
        };
        let ParserReturn { mut program, errors, mut module_record, .. } =
            Parser::new(&allocator, &source_text, source_type).with_options(parser_options).parse();
        self.diagnostics.extend(errors);

        let mut semantic_builder = SemanticBuilder::new();
        if run_options.transform {
            // Estimate transformer will triple scopes, symbols, references
            semantic_builder = semantic_builder.with_excess_capacity(2.0);
        }
        let semantic_ret =
            semantic_builder.with_check_syntax_error(true).with_cfg(true).build(&program);
        let semantic = semantic_ret.semantic;
        self.diagnostics.extend(semantic_ret.errors);

        self.control_flow_graph = semantic.cfg().map_or_else(String::default, |cfg| {
            cfg.debug_dot(DebugDotContext::new(
                semantic.nodes(),
                control_flow_options.verbose.unwrap_or_default(),
            ))
        });

        let linter_module_record = Arc::new(ModuleRecord::new(&path, &module_record, &semantic));
        self.run_linter(
            &run_options,
            &linter_options,
            &path,
            &program,
            &linter_module_record,
            &allocator,
        );

        self.run_formatter(&run_options, parser_options, &source_text, source_type);

        let scoping = semantic.into_scoping();

        if !source_type.is_typescript_definition() {
            if run_options.scope {
                self.scope_text = Self::get_scope_text(&program, &scoping);
            }
            if run_options.symbol {
                self.symbols_json = Self::get_symbols_text(&scoping)?;
            }
        }

        if run_options.isolated_declarations {
            let id_options = isolated_declarations_options
                .map(|o| IsolatedDeclarationsOptions { strip_internal: o.strip_internal })
                .unwrap_or_default();
            let ret = IsolatedDeclarations::new(&allocator, id_options).build(&program);
            if ret.errors.is_empty() {
                self.codegen(&path, &ret.program, None, &run_options, &codegen_options);
            } else {
                self.diagnostics.extend(ret.errors);
                self.codegen_text = String::new();
                self.codegen_sourcemap_text = None;
            }
            return Ok(());
        }

        if run_options.transform {
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
            let result = Transformer::new(&allocator, &path, &options)
                .build_with_scoping(scoping, &mut program);
            self.diagnostics.extend(result.errors);
        }

        let scoping = if options.run.compress || options.run.mangle {
            let compress = options.compress.map(|_| CompressOptions::smallest());
            let mangle = options.mangle.map(|o| MangleOptions {
                top_level: o.top_level,
                keep_names: MangleOptionsKeepNames { function: o.keep_names, class: o.keep_names },
                debug: false,
            });
            Minifier::new(MinifierOptions { mangle, compress })
                .minify(&allocator, &mut program)
                .scoping
        } else {
            None
        };

        self.codegen(&path, &program, scoping, &run_options, &codegen_options);
        self.ir = format!("{:#?}", program.body);
        let mut comments =
            convert_utf8_to_utf16(&source_text, &mut program, &mut module_record, &mut []);

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

        Ok(())
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
