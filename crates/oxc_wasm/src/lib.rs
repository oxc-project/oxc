mod options;

use std::{cell::RefCell, path::PathBuf, rc::Rc};

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions},
    diagnostics::Error,
    minifier::{CompressOptions, Minifier, MinifierOptions},
    parser::Parser,
    semantic::{ScopeId, Semantic, SemanticBuilder},
    span::SourceType,
    transformer::{TransformOptions, TransformTarget, Transformer},
};
use oxc_linter::{LintContext, Linter};
use oxc_prettier::{Prettier, PrettierOptions};
use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::options::{
    OxcCodegenOptions, OxcLinterOptions, OxcMinifierOptions, OxcParserOptions, OxcRunOptions,
};

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Tsify)]
pub struct Oxc {
    source_text: String,
    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "Program")]
    pub ast: JsValue,
    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "Statement[]")]
    pub ir: JsValue,
    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "SymbolTable")]
    pub symbols: JsValue,

    #[wasm_bindgen(readonly, skip_typescript, js_name = "scopeText")]
    #[serde(rename = "scopeText")]
    pub scope_text: String,
    #[wasm_bindgen(readonly, skip_typescript, js_name = "codegenText")]
    #[serde(rename = "codegenText")]
    pub codegen_text: String,
    #[wasm_bindgen(readonly, skip_typescript, js_name = "formattedText")]
    #[serde(rename = "formattedText")]
    pub formatted_text: String,
    #[wasm_bindgen(readonly, skip_typescript, js_name = "prettierFormattedText")]
    #[serde(rename = "prettierFormattedText")]
    pub prettier_formatted_text: String,
    #[wasm_bindgen(readonly, skip_typescript, js_name = "prettierIrText")]
    #[serde(rename = "prettierIrText")]
    pub prettier_ir_text: String,

    diagnostics: RefCell<Vec<Error>>,
    #[serde(skip)]
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

    /// # Errors
    /// Serde serialization error
    #[wasm_bindgen]
    pub fn run(
        &mut self,
        run_options: &OxcRunOptions,
        parser_options: &OxcParserOptions,
        _linter_options: &OxcLinterOptions,
        _codegen_options: &OxcCodegenOptions,
        minifier_options: &OxcMinifierOptions,
    ) -> Result<(), serde_wasm_bindgen::Error> {
        self.diagnostics = RefCell::default();

        let allocator = Allocator::default();
        let source_text = &self.source_text;
        let path = PathBuf::from("test.tsx");
        let source_type = SourceType::from_path(&path).unwrap_or_default();

        let ret = Parser::new(&allocator, source_text, source_type)
            .allow_return_outside_function(parser_options.allow_return_outside_function)
            .parse();
        self.save_diagnostics(ret.errors);

        self.ir = format!("{:#?}", ret.program.body).into();

        let program = allocator.alloc(ret.program);

        let semantic_builder = SemanticBuilder::new(source_text, source_type);

        if run_options.syntax() && !run_options.lint() {
            let semantic_ret = semantic_builder
                .with_trivias(ret.trivias)
                .with_check_syntax_error(true)
                .build(program);
            self.save_diagnostics(semantic_ret.errors);
        } else if run_options.lint() {
            let semantic_ret = semantic_builder
                .with_trivias(ret.trivias)
                .with_check_syntax_error(true)
                .build(program);
            self.save_diagnostics(semantic_ret.errors);

            let semantic = Rc::new(semantic_ret.semantic);
            let lint_ctx = LintContext::new(path.into_boxed_path(), &semantic);
            let linter_ret = Linter::default().run(lint_ctx);
            let diagnostics = linter_ret.into_iter().map(|e| e.error).collect();
            self.save_diagnostics(diagnostics);
        } else {
            semantic_builder.build(program);
        }
        self.ast = program.serialize(&self.serializer)?;

        if run_options.prettier_format() {
            let ret = Parser::new(&allocator, source_text, source_type)
                .allow_return_outside_function(parser_options.allow_return_outside_function)
                .preserve_parens(false)
                .parse();
            let printed =
                Prettier::new(&allocator, source_text, ret.trivias, PrettierOptions::default())
                    .build(&ret.program);
            self.prettier_formatted_text = printed;
        }

        if run_options.prettier_ir() {
            let ret = Parser::new(&allocator, source_text, source_type)
                .allow_return_outside_function(parser_options.allow_return_outside_function)
                .preserve_parens(false)
                .parse();
            let prettier_doc =
                Prettier::new(&allocator, source_text, ret.trivias, PrettierOptions::default())
                    .doc(&ret.program)
                    .to_string();
            self.prettier_ir_text = {
                let ret = Parser::new(&allocator, &prettier_doc, SourceType::default()).parse();
                Prettier::new(&allocator, &prettier_doc, ret.trivias, PrettierOptions::default())
                    .build(&ret.program)
            };
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

        if run_options.scope() || run_options.symbol() {
            let semantic = SemanticBuilder::new(source_text, source_type).build(program).semantic;
            if run_options.scope() {
                self.scope_text = Self::get_scope_text(&semantic);
            } else if run_options.symbol() {
                self.symbols = semantic.symbols().serialize(&self.serializer)?;
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

    fn get_scope_text(semantic: &Semantic) -> String {
        fn write_scope_text(
            semantic: &Semantic,
            scope_text: &mut String,
            depth: usize,
            scope_ids: &Vec<ScopeId>,
        ) {
            let space = " ".repeat(depth * 2);

            for scope_id in scope_ids {
                let flag = semantic.scopes().get_flags(*scope_id);
                let next_scope_ids = semantic.scopes().get_child_ids(*scope_id);

                scope_text.push_str(&format!("{space}Scope{:?} ({flag:?}) {{\n", *scope_id + 1));
                let bindings = semantic.scopes().get_bindings(*scope_id);
                let binding_space = " ".repeat((depth + 1) * 2);
                if !bindings.is_empty() {
                    scope_text.push_str(&format!("{binding_space}Bindings: {{"));
                }
                bindings.iter().for_each(|(name, symbol_id)| {
                    let symbol_flag = semantic.symbols().get_flag(*symbol_id);
                    scope_text.push_str(&format!("\n{binding_space}  {name} ({symbol_flag:?})",));
                });
                if !bindings.is_empty() {
                    scope_text.push_str(&format!("\n{binding_space}}}\n"));
                }

                if let Some(next_scope_ids) = next_scope_ids {
                    write_scope_text(semantic, scope_text, depth + 1, next_scope_ids);
                }
                scope_text.push_str(&format!("{space}}}\n"));
            }
        }

        let mut scope_text = String::default();
        write_scope_text(semantic, &mut scope_text, 0, &vec![semantic.scopes().root_scope_id()]);
        scope_text
    }

    fn save_diagnostics(&self, diagnostics: Vec<Error>) {
        self.diagnostics.borrow_mut().extend(diagnostics);
    }
}
