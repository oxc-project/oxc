use std::{cell::Cell, collections::HashSet, path::PathBuf};

use oxc::allocator::{Allocator, CloneIn};
#[allow(clippy::wildcard_imports)]
use oxc::ast::{ast::*, visit::walk, Trivias, Visit};
use oxc::codegen::{CodeGenerator, CommentOptions, WhitespaceRemover};
use oxc::diagnostics::OxcDiagnostic;
use oxc::minifier::{CompressOptions, Compressor};
use oxc::parser::{Parser, ParserReturn};
use oxc::semantic::{ReferenceId, ScopeFlags, ScopeTree, SemanticBuilder, SymbolId, SymbolTable};
use oxc::span::{CompactStr, SourceType, Span};
use oxc::syntax::scope::ScopeId;
use oxc::transformer::{TransformOptions, Transformer, TransformerReturn};

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
    pub check_semantic: bool,
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
        let ParserReturn { mut program, errors, trivias, panicked } =
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

        let check1 = if self.check_semantic {
            let mut check1 = SemanticCollector::default();
            if let Some(errors) = check1.check(&program) {
                self.errors.extend(errors);
                return;
            }
            Some(check1)
        } else {
            None
        };

        if let Some(options) = self.transform.clone() {
            let (symbols, scopes) = semantic_ret.semantic.into_symbol_table_and_scope_tree();
            let TransformerReturn { symbols, scopes, errors } = Transformer::new(
                &allocator,
                &self.path,
                source_type,
                source_text,
                trivias.clone(),
                options,
            )
            .build_with_symbols_and_scopes(symbols, scopes, &mut program);

            if !errors.is_empty() {
                self.errors.extend(errors);
                return;
            }

            if let Some(check1) = check1 {
                if self.check_semantic(&check1, &symbols, &scopes, &program) {
                    return;
                }
            }
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

    fn check_semantic(
        &mut self,
        previous_collect: &SemanticCollector,
        previous_symbols: &SymbolTable,
        previous_scopes: &ScopeTree,
        program: &Program<'_>,
    ) -> bool {
        let mut current_collect = SemanticCollector::default();

        let allocator = Allocator::default();
        let program = program.clone_in(&allocator);
        let (current_symbols, current_scopes) = SemanticBuilder::new("", program.source_type)
            .build(&program)
            .semantic
            .into_symbol_table_and_scope_tree();
        if let Some(errors) = current_collect.check(&program) {
            self.errors.extend(errors);
            return true;
        }

        let errors_count = self.errors.len();

        self.check_bindings(
            previous_collect,
            previous_symbols,
            previous_scopes,
            &current_collect,
            &current_scopes,
        );
        self.check_symbols(
            previous_collect,
            previous_symbols,
            previous_scopes,
            &current_collect,
            &current_symbols,
            &current_scopes,
        );
        self.check_references(
            previous_collect,
            previous_symbols,
            previous_scopes,
            &current_collect,
            &current_symbols,
            &current_scopes,
        );

        errors_count != self.errors.len()
    }

    fn check_bindings(
        &mut self,
        previous_collect: &SemanticCollector,
        _previous_symbols: &SymbolTable,
        previous_scopes: &ScopeTree,
        current_collect: &SemanticCollector,
        current_scopes: &ScopeTree,
    ) {
        if previous_collect.scope_ids.len() != current_collect.scope_ids.len() {
            self.errors.push(OxcDiagnostic::error("Scopes mismatch after transform"));
            return;
        }

        // Check whether bindings are the same for scopes in the same visitation order.
        for (prev_scope_id, cur_scope_id) in
            previous_collect.scope_ids.iter().zip(current_collect.scope_ids.iter())
        {
            let mut prev_bindings =
                previous_scopes.get_bindings(*prev_scope_id).keys().cloned().collect::<Vec<_>>();
            prev_bindings.sort_unstable();
            let mut current_bindings =
                current_scopes.get_bindings(*cur_scope_id).keys().cloned().collect::<Vec<_>>();
            current_bindings.sort_unstable();

            if prev_bindings.iter().collect::<HashSet<&CompactStr>>()
                != current_bindings.iter().collect::<HashSet<&CompactStr>>()
            {
                let message = format!(
                    "
Bindings Mismatch:
previous scope {prev_scope_id:?}: {prev_bindings:?}
current  scope {cur_scope_id:?}: {current_bindings:?}
                    "
                );
                self.errors.push(OxcDiagnostic::error(message.trim().to_string()));
            }
        }
    }

    fn check_symbols(
        &mut self,
        previous_collect: &SemanticCollector,
        previous_symbols: &SymbolTable,
        _previous_scopes: &ScopeTree,
        current_collect: &SemanticCollector,
        current_symbols: &SymbolTable,
        _current_scopes: &ScopeTree,
    ) {
        if previous_collect.symbol_ids.len() != current_collect.symbol_ids.len() {
            self.errors.push(OxcDiagnostic::error("Symbols mismatch after transform"));
            return;
        }

        // Check whether symbols match
        for (prev_symbol_id, cur_symbol_id) in
            previous_collect.symbol_ids.iter().zip(current_collect.symbol_ids.iter())
        {
            let prev_symbol_name = &previous_symbols.names[*prev_symbol_id];
            let cur_symbol_name = &current_symbols.names[*cur_symbol_id];
            if prev_symbol_name != cur_symbol_name {
                let message = format!(
                    "
Symbol Mismatch:
previous symbol {prev_symbol_id:?}: {prev_symbol_id:?}
current  symbol {cur_symbol_id:?}: {cur_symbol_id:?}
                    "
                );
                self.errors.push(OxcDiagnostic::error(message.trim().to_string()));
            }
        }
    }

    fn check_references(
        &mut self,
        previous_collect: &SemanticCollector,
        previous_symbols: &SymbolTable,
        _previous_scopes: &ScopeTree,
        current_collect: &SemanticCollector,
        current_symbols: &SymbolTable,
        _current_scopes: &ScopeTree,
    ) {
        if previous_collect.reference_ids.len() != current_collect.reference_ids.len() {
            self.errors.push(OxcDiagnostic::error("ReferenceId mismatch after transform"));
            return;
        }

        // Check whether symbols match
        for (prev_reference_id, cur_reference_id) in
            previous_collect.reference_ids.iter().zip(current_collect.reference_ids.iter())
        {
            let prev_symbol_id = previous_symbols.references[*prev_reference_id].symbol_id();
            let prev_symbol_name = prev_symbol_id.map(|id| previous_symbols.names[id].clone());
            let cur_symbol_id = &current_symbols.references[*cur_reference_id].symbol_id();
            let cur_symbol_name = cur_symbol_id.map(|id| current_symbols.names[id].clone());
            if prev_symbol_name != cur_symbol_name {
                let message = format!(
                    "
reference Mismatch:
previous reference {prev_reference_id:?}: {prev_symbol_name:?}
current  reference {cur_reference_id:?}: {cur_symbol_name:?}
                    "
                );
                self.errors.push(OxcDiagnostic::error(message.trim().to_string()));
            }
        }
    }
}

#[derive(Default)]
struct SemanticCollector {
    scope_ids: Vec<ScopeId>,
    symbol_ids: Vec<SymbolId>,
    reference_ids: Vec<ReferenceId>,
    missing_references: Vec<Span>,
    missing_symbols: Vec<Span>,
}

impl<'a> Visit<'a> for SemanticCollector {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        if let Some(scope_id) = scope_id.get() {
            self.scope_ids.push(scope_id);
        }
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(reference_id) = ident.reference_id.get() {
            self.reference_ids.push(reference_id);
        } else {
            self.missing_references.push(ident.span);
        }
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if let Some(symbol_id) = ident.symbol_id.get() {
            self.symbol_ids.push(symbol_id);
        } else {
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

impl SemanticCollector {
    fn check(&mut self, program: &Program<'_>) -> Option<Vec<OxcDiagnostic>> {
        if program.source_type.is_typescript_definition() {
            return None;
        }
        self.check_ast(program)
    }

    fn check_ast(&mut self, program: &Program<'_>) -> Option<Vec<OxcDiagnostic>> {
        self.visit_program(program);

        let diagnostics = self
            .missing_references
            .iter()
            .map(|span| OxcDiagnostic::error("Missing ReferenceId").with_label(*span))
            .chain(
                self.missing_symbols
                    .iter()
                    .map(|span| OxcDiagnostic::error("Missing SymbolId").with_label(*span)),
            )
            .collect::<Vec<_>>();

        (!diagnostics.is_empty()).then_some(diagnostics)
    }
}
