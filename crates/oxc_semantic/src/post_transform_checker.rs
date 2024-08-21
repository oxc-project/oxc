use std::{cell::Cell, collections::HashSet, mem};

use oxc_allocator::{Allocator, CloneIn};
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, visit::walk, Visit};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::CompactStr;
use oxc_syntax::{
    reference::ReferenceId,
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolId,
};

use crate::{ScopeTree, SemanticBuilder, SymbolTable};

#[derive(Default)]
pub struct PostTransformChecker {
    errors: Vec<OxcDiagnostic>,
    previous_collect: SemanticCollector,
}

impl PostTransformChecker {
    pub fn before_transform(&mut self, program: &Program<'_>) -> Option<Vec<OxcDiagnostic>> {
        self.previous_collect.check(program)
    }

    pub fn after_transform(
        &mut self,
        previous_symbols: &SymbolTable,
        previous_scopes: &ScopeTree,
        program: &Program<'_>,
    ) -> Option<Vec<OxcDiagnostic>> {
        let mut current_collect = SemanticCollector::default();

        let allocator = Allocator::default();
        let program = program.clone_in(&allocator);
        let (current_symbols, current_scopes) = SemanticBuilder::new("", program.source_type)
            .build(&program)
            .semantic
            .into_symbol_table_and_scope_tree();

        if let Some(errors) = current_collect.check(&program) {
            return Some(errors);
        }

        let errors_count = self.errors.len();

        self.check_bindings(previous_symbols, previous_scopes, &current_collect, &current_scopes);

        self.check_symbols(
            previous_symbols,
            previous_scopes,
            &current_collect,
            &current_symbols,
            &current_scopes,
        );
        self.check_references(
            previous_symbols,
            previous_scopes,
            &current_collect,
            &current_symbols,
            &current_scopes,
        );

        (errors_count != self.errors.len()).then(|| mem::take(&mut self.errors))
    }

    fn check_bindings(
        &mut self,
        _previous_symbols: &SymbolTable,
        previous_scopes: &ScopeTree,
        current_collect: &SemanticCollector,
        current_scopes: &ScopeTree,
    ) {
        if self.previous_collect.scope_ids.len() != current_collect.scope_ids.len() {
            self.errors.push(OxcDiagnostic::error("Scopes mismatch after transform"));
            return;
        }

        // Check whether bindings are the same for scopes in the same visitation order.
        for (prev_scope_id, cur_scope_id) in
            self.previous_collect.scope_ids.iter().zip(current_collect.scope_ids.iter())
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
        previous_symbols: &SymbolTable,
        _previous_scopes: &ScopeTree,
        current_collect: &SemanticCollector,
        current_symbols: &SymbolTable,
        current_scopes: &ScopeTree,
    ) {
        // Check whether symbols are valid
        for symbol_id in current_collect.symbol_ids.iter().copied() {
            if current_symbols.get_flag(symbol_id).is_empty() {
                let name = current_symbols.get_name(symbol_id);
                self.errors.push(OxcDiagnostic::error(format!(
                    "Expect non-empty SymbolFlags for BindingIdentifier({name})"
                )));
                if !current_scopes.has_binding(current_symbols.get_scope_id(symbol_id), name) {
                    self.errors.push(OxcDiagnostic::error(
                    format!("Cannot find BindingIdentifier({name}) in the Scope corresponding to the Symbol"),
                ));
                }
            }
        }

        if self.previous_collect.symbol_ids.len() != current_collect.symbol_ids.len() {
            self.errors.push(OxcDiagnostic::error("Symbols mismatch after transform"));
            return;
        }

        // Check whether symbols match
        for (prev_symbol_id, cur_symbol_id) in
            self.previous_collect.symbol_ids.iter().zip(current_collect.symbol_ids.iter())
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
        previous_symbols: &SymbolTable,
        _previous_scopes: &ScopeTree,
        current_collect: &SemanticCollector,
        current_symbols: &SymbolTable,
        _current_scopes: &ScopeTree,
    ) {
        // Check whether references are valid
        for reference_id in current_collect.reference_ids.iter().copied() {
            let reference = current_symbols.get_reference(reference_id);
            if reference.flags().is_empty() {
                self.errors.push(OxcDiagnostic::error(format!(
                    "Expect ReferenceFlags for IdentifierReference({reference_id:?}) to not be empty",
                )));
            }
        }

        if self.previous_collect.reference_ids.len() != current_collect.reference_ids.len() {
            self.errors.push(OxcDiagnostic::error("ReferenceId mismatch after transform"));
            return;
        }

        // Check whether symbols match
        for (prev_reference_id, cur_reference_id) in
            self.previous_collect.reference_ids.iter().zip(current_collect.reference_ids.iter())
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

    errors: Vec<OxcDiagnostic>,
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
            self.errors.push(OxcDiagnostic::error("Missing ReferenceId").with_label(ident.span));
        }
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if let Some(symbol_id) = ident.symbol_id.get() {
            self.symbol_ids.push(symbol_id);
        } else {
            self.errors.push(OxcDiagnostic::error("Missing SymbolId").with_label(ident.span));
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
    pub fn check(&mut self, program: &Program<'_>) -> Option<Vec<OxcDiagnostic>> {
        if program.source_type.is_typescript_definition() {
            return None;
        }
        self.check_ast(program)
    }

    fn check_ast(&mut self, program: &Program<'_>) -> Option<Vec<OxcDiagnostic>> {
        self.visit_program(program);

        (!self.errors.is_empty()).then(|| mem::take(&mut self.errors))
    }
}
