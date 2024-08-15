use std::{collections::HashSet, path::PathBuf};

use oxc_allocator::Allocator;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, visit::walk, Trivias, Visit};
use oxc_codegen::{CodeGenerator, CommentOptions, WhitespaceRemover};
use oxc_diagnostics::OxcDiagnostic;
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{ScopeFlags, Semantic, SemanticBuilder};
use oxc_span::{SourceType, Span};
use oxc_transformer::{TransformOptions, Transformer};

use crate::suite::TestResult;

#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct Driver {
    pub path: PathBuf,
    // options
    pub transform: Option<TransformOptions>,
    pub compress: bool,
    pub remove_whitespace: bool,
    pub codegen: bool,
    pub allow_return_outside_function: bool,
    // results
    pub panicked: bool,
    pub errors: Vec<OxcDiagnostic>,
    pub printed: String,
}

impl Driver {
    pub fn errors(&mut self) -> Vec<OxcDiagnostic> {
        std::mem::take(&mut self.errors)
    }

    pub fn idempotency(
        mut self,
        case: &'static str,
        source_text: &str,
        source_type: SourceType,
    ) -> TestResult {
        self.run(source_text, source_type);
        let printed1 = self.printed.clone();
        self.run(&printed1, source_type);
        let printed2 = self.printed.clone();
        if printed1 == printed2 {
            TestResult::Passed
        } else {
            TestResult::Mismatch(case, printed1, printed2)
        }
    }

    pub fn run(&mut self, source_text: &str, source_type: SourceType) {
        let allocator = Allocator::default();
        let ParserReturn { mut program, errors, trivias, panicked, .. } =
            Parser::new(&allocator, source_text, source_type)
                .allow_return_outside_function(self.allow_return_outside_function)
                .parse();
        self.panicked = panicked;

        if self.check_comments(&trivias) {
            return;
        }

        // Make sure serialization doesn't crash; also for code coverage.
        let _serializer = program.serializer();

        if !errors.is_empty() {
            self.errors.extend(errors);
        }

        let semantic_ret = SemanticBuilder::new(source_text, source_type)
            .with_trivias(trivias.clone())
            .with_check_syntax_error(true)
            .build_module_record(self.path.clone(), &program)
            .build(&program);

        if !semantic_ret.errors.is_empty() {
            self.errors.extend(semantic_ret.errors);
            return;
        }

        if let Some(errors) = SemanticChecker::new(&semantic_ret.semantic).check(&program) {
            self.errors.extend(errors);
            return;
        }

        if let Some(options) = self.transform.clone() {
            Transformer::new(
                &allocator,
                &self.path,
                source_type,
                source_text,
                trivias.clone(),
                options,
            )
            .build(&mut program);
        }

        if self.compress {
            Compressor::new(&allocator, CompressOptions::all_true()).build(&mut program);
        }

        if self.codegen {
            let comment_options = CommentOptions { preserve_annotate_comments: true };

            let printed = if self.remove_whitespace {
                WhitespaceRemover::new().build(&program).source_text
            } else {
                CodeGenerator::new()
                    .enable_comment(source_text, trivias, comment_options)
                    .build(&program)
                    .source_text
            };

            self.printed = printed;
        }
    }

    fn check_comments(&mut self, trivias: &Trivias) -> bool {
        let mut uniq: HashSet<Span> = HashSet::new();
        for comment in trivias.comments() {
            if !uniq.insert(comment.span) {
                self.errors
                    .push(OxcDiagnostic::error("Duplicate Comment").with_label(comment.span));
                return true;
            }
        }
        false
    }
}

struct SemanticChecker<'a, 'b> {
    #[allow(unused)]
    semantic: &'b Semantic<'a>,

    missing_references: Vec<Span>,
    missing_symbols: Vec<Span>,
}

impl<'a, 'b> Visit<'a> for SemanticChecker<'a, 'b> {
    // Check missing `ReferenceId` on `IdentifierReference`.
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if ident.reference_id.get().is_none() {
            self.missing_references.push(ident.span);
        }
    }

    // Check missing `SymbolId` on `BindingIdentifier`.
    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if ident.symbol_id.get().is_none() {
            self.missing_symbols.push(ident.span);
        }
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        if func.is_ts_declare_function() {
            return;
        }
        walk::walk_function(self, func, flags);
    }

    fn visit_declaration(&mut self, it: &Declaration<'a>) {
        if it.is_typescript_syntax() {
            return;
        }
        walk::walk_declaration(self, it);
    }

    fn visit_if_statement(&mut self, stmt: &IfStatement<'a>) {
        // skip `if (function foo() {}) {}`
        if !matches!(stmt.test, Expression::FunctionExpression(_)) {
            self.visit_expression(&stmt.test);
        }
        // skip `if (true) function foo() {} else function bar() {}`
        if !stmt.consequent.is_declaration() {
            self.visit_statement(&stmt.consequent);
        }
        if let Some(alternate) = &stmt.alternate {
            if !alternate.is_declaration() {
                self.visit_statement(alternate);
            }
        }
    }

    fn visit_ts_type(&mut self, _it: &TSType<'a>) {
        /* noop */
    }
}

impl<'a, 'b> SemanticChecker<'a, 'b> {
    fn new(semantic: &'b Semantic<'a>) -> Self {
        Self { semantic, missing_references: vec![], missing_symbols: vec![] }
    }

    fn check(mut self, program: &Program<'a>) -> Option<Vec<OxcDiagnostic>> {
        if program.source_type.is_typescript_definition() {
            return None;
        }

        self.visit_program(program);

        let diagnostics = self
            .missing_references
            .into_iter()
            .map(|span| OxcDiagnostic::error("Missing ReferenceId").with_label(span))
            .chain(
                self.missing_symbols
                    .into_iter()
                    .map(|span| OxcDiagnostic::error("Missing SymbolId").with_label(span)),
            )
            .collect::<Vec<_>>();

        (!diagnostics.is_empty()).then_some(diagnostics)
    }
}
