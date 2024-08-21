//! Utility to check correctness of `ScopeTree` and `SymbolTable` after transformer has run.
//!
//! ## What it's for
//!
//! The transformer should keep `ScopeTree` and `SymbolTable` in sync with the AST as it makes changes.
//! This utility checks the correctness of the semantic data after transformer has processed AST,
//! to make sure it's working correctly.
//!
//! ## How
//!
//! We do this by:
//! 1. Taking `ScopeTree` and `SymbolTable` after transformer has run.
//! 2. Cloning the post-transform AST.
//! 3. Running a fresh semantic analysis on that AST.
//! 4. Comparing the 2 copies of `ScopeTree` and `SymbolTable` from after the transformer
//!    vs from the fresh semantic analysis.
//!
//! We now have 2 sets of semantic data:
//! * "transformer": Semantic data from after the transformer has run
//! * "rebuild": Semantic data from the fresh semantic analysis
//!
//! If the transformer has behaved correctly, the state of `ScopeTree` and `SymbolTable` should match
//! between "transformer" and "rebuild".
//!
//! ## Complication
//!
//! The complication is in the word "match".
//!
//! For example if this is the original input:
//! ```ts
//! if (x) enum Foo {}
//! function f() {}
//! ```
//!
//! The output of transformer is:
//! ```js
//! if (x) {}
//! function f() {}
//! ```
//!
//! The scope IDs are:
//!
//! Before transform:
//! ```ts
//! // Scope ID 0
//! if (x) enum Foo { /* Scope ID 1 */ }
//! function f() { /* Scope ID 2 */ }
//! ```
//!
//! After transform:
//! ```js
//! // Scope ID 0
//! if (x) { /* Scope ID 3 */ } // <-- newly created scope
//! function f() { /* Scope ID 2 */ }
//! ```
//!
//! vs fresh semantic analysis of post-transform AST:
//! ```js
//! // Scope ID 0
//! if (x) { /* Scope ID 1 */ } // <-- numbered 1 as it's 2nd scope in visitation order
//! function f() { /* Scope ID 2 */ }
//! ```
//!
//! Scope IDs of the `if {}` block are different in the 2 versions, because in the post-transform version,
//! that scope was newly created in transformer, so was pushed last to `ScopeTree`.
//!
//! However, despite the scope IDs being different, these 2 sets of semantic data *are* equivalent.
//! The scope IDs are different, but they represent the same scopes.
//! i.e. IDs don't need to be equal, but they do need to used in a consistent pattern between the 2
//! semantic data sets. If scope ID 3 is used in the post-transform semantic data everywhere that
//! scope ID 1 is used in the rebuilt semantic data, then the 2 are equivalent, and the tests pass.
//!
//! Same principle for `SymbolId`s and `ReferenceId`s.
//!
//! ## Mechanism for matching
//!
//! `SemanticCollector` visits the AST, and builds lists of `ScopeId`s, `SymbolId`s and `ReferenceId`s
//! in visitation order. We run `SemanticCollector` once on the AST coming out of the transformer,
//! and a 2nd time on the AST after the fresh semantic analysis.
//!
//! When we ZIP these lists together, we have pairs of `(transformer_id, rebuild_id)` which give the
//! mapping between the 2 sets of IDs.
//!
//! ## Other notes
//!
//! See also: <https://github.com/oxc-project/oxc/issues/4790>

use std::{cell::Cell, mem};

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
    collector_transformer: SemanticCollector,
}

impl PostTransformChecker {
    pub fn after_transform(
        &mut self,
        symbols_transformer: &SymbolTable,
        scopes_transformer: &ScopeTree,
        program: &Program<'_>,
    ) -> Option<Vec<OxcDiagnostic>> {
        // Collect `ScopeId`s, `SymbolId`s and `ReferenceId`s from AST after transformer
        self.collector_transformer = SemanticCollector::default();
        if let Some(errors) = self.collector_transformer.check(program) {
            self.errors.push(OxcDiagnostic::error("Semantic Collector failed after transform"));
            self.errors.extend(errors);
            return Some(mem::take(&mut self.errors));
        }

        // Clone the post-transform AST, re-run semantic analysis on it, and collect `ScopeId`s,
        // `SymbolId`s and `ReferenceId`s from AST.
        // NB: `CloneIn` sets all `scope_id`, `symbol_id` and `reference_id` fields in AST to `None`,
        // so the cloned AST will be "clean" of all semantic data, as if it had come fresh from the parser.
        let allocator = Allocator::default();
        let program = program.clone_in(&allocator);
        let (symbols_rebuild, scopes_rebuild) = SemanticBuilder::new("", program.source_type)
            .build(&program)
            .semantic
            .into_symbol_table_and_scope_tree();

        let mut collector_rebuild = SemanticCollector::default();
        if let Some(errors) = collector_rebuild.check(&program) {
            self.errors.push(OxcDiagnostic::error("Semantic Collector failed after rebuild"));
            self.errors.extend(errors);
            return Some(mem::take(&mut self.errors));
        }

        let errors_count = self.errors.len();

        // Compare post-transform semantic data to semantic data from fresh semantic analysis
        self.check_bindings(scopes_transformer, &scopes_rebuild, &collector_rebuild);

        self.check_symbols(
            symbols_transformer,
            &symbols_rebuild,
            &scopes_rebuild,
            &collector_rebuild,
        );
        self.check_references(symbols_transformer, &symbols_rebuild, &collector_rebuild);

        (errors_count != self.errors.len()).then(|| mem::take(&mut self.errors))
    }

    fn check_bindings(
        &mut self,
        scopes_transformer: &ScopeTree,
        scopes_rebuild: &ScopeTree,
        collector_rebuild: &SemanticCollector,
    ) {
        if self.collector_transformer.scope_ids.len() != collector_rebuild.scope_ids.len() {
            self.errors.push(OxcDiagnostic::error("Scopes mismatch after transform"));
            return;
        }

        // Check whether bindings are the same for scopes in the same visitation order.
        for (&scope_id_transformer, &scope_id_rebuild) in
            self.collector_transformer.scope_ids.iter().zip(collector_rebuild.scope_ids.iter())
        {
            fn get_sorted_bindings(scopes: &ScopeTree, scope_id: ScopeId) -> Vec<CompactStr> {
                let mut bindings =
                    scopes.get_bindings(scope_id).keys().cloned().collect::<Vec<_>>();
                bindings.sort_unstable();
                bindings
            }

            let (result_transformer, result_rebuild) =
                match (scope_id_transformer, scope_id_rebuild) {
                    (None, None) => continue,
                    (Some(scope_id_transformer), Some(scope_id_rebuild)) => {
                        let bindings_transformer =
                            get_sorted_bindings(scopes_transformer, scope_id_transformer);
                        let bindings_rebuild =
                            get_sorted_bindings(scopes_rebuild, scope_id_rebuild);
                        if bindings_transformer == bindings_rebuild {
                            continue;
                        }
                        (
                            format!("{scope_id_transformer:?}: {bindings_transformer:?}"),
                            format!("{scope_id_rebuild:?}: {bindings_rebuild:?}"),
                        )
                    }
                    (Some(scope_id_transformer), None) => {
                        let bindings_transformer =
                            get_sorted_bindings(scopes_transformer, scope_id_transformer);
                        (
                            format!("{scope_id_transformer:?}: {bindings_transformer:?}"),
                            "No scope".to_string(),
                        )
                    }
                    (None, Some(scope_id_rebuild)) => {
                        let bindings_rebuild =
                            get_sorted_bindings(scopes_rebuild, scope_id_rebuild);
                        (
                            "No scope".to_string(),
                            format!("{scope_id_rebuild:?}: {bindings_rebuild:?}"),
                        )
                    }
                };

            let message = format!(
                "
Bindings mismatch:
after transform: {result_transformer}
rebuilt        : {result_rebuild}
                "
            );
            self.errors.push(OxcDiagnostic::error(message.trim().to_string()));
        }
    }

    fn check_symbols(
        &mut self,
        symbols_transformer: &SymbolTable,
        symbols_rebuild: &SymbolTable,
        scopes_rebuild: &ScopeTree,
        collector_rebuild: &SemanticCollector,
    ) {
        // Check whether symbols are valid
        for symbol_id in collector_rebuild.symbol_ids.iter().copied() {
            if symbols_rebuild.get_flags(symbol_id).is_empty() {
                let name = symbols_rebuild.get_name(symbol_id);
                self.errors.push(OxcDiagnostic::error(format!(
                    "Expect non-empty SymbolFlags for BindingIdentifier({name})"
                )));
                if !scopes_rebuild.has_binding(symbols_rebuild.get_scope_id(symbol_id), name) {
                    self.errors.push(OxcDiagnostic::error(
                        format!("Cannot find BindingIdentifier({name}) in the Scope corresponding to the Symbol"),
                    ));
                }
            }
        }

        if self.collector_transformer.symbol_ids.len() != collector_rebuild.symbol_ids.len() {
            self.errors.push(OxcDiagnostic::error("Symbols mismatch after transform"));
            return;
        }

        // Check whether symbols match
        for (symbol_id_transformer, symbol_id_rebuild) in
            self.collector_transformer.symbol_ids.iter().zip(collector_rebuild.symbol_ids.iter())
        {
            let symbol_name_transformer = &symbols_transformer.names[*symbol_id_transformer];
            let symbol_name_rebuild = &symbols_rebuild.names[*symbol_id_rebuild];
            if symbol_name_transformer != symbol_name_rebuild {
                let message = format!(
                    "
Symbol mismatch:
after transform: {symbol_id_transformer:?}: {symbol_name_transformer:?}
rebuilt        : {symbol_id_rebuild:?}: {symbol_name_rebuild:?}
                    "
                );
                self.errors.push(OxcDiagnostic::error(message.trim().to_string()));
            }
        }
    }

    fn check_references(
        &mut self,
        symbols_transformer: &SymbolTable,
        symbols_rebuild: &SymbolTable,
        collector_rebuild: &SemanticCollector,
    ) {
        // Check whether references are valid
        for reference_id in collector_rebuild.reference_ids.iter().copied() {
            let reference = symbols_rebuild.get_reference(reference_id);
            if reference.flags().is_empty() {
                self.errors.push(OxcDiagnostic::error(format!(
                    "Expect ReferenceFlags for IdentifierReference({reference_id:?}) to not be empty",
                )));
            }
        }

        if self.collector_transformer.reference_ids.len() != collector_rebuild.reference_ids.len() {
            self.errors.push(OxcDiagnostic::error("ReferenceId mismatch after transform"));
            return;
        }

        // Check whether symbols match
        for (reference_id_transformer, reference_id_rebuild) in self
            .collector_transformer
            .reference_ids
            .iter()
            .zip(collector_rebuild.reference_ids.iter())
        {
            let symbol_id_transformer =
                symbols_transformer.references[*reference_id_transformer].symbol_id();
            let symbol_name_transformer =
                symbol_id_transformer.map(|id| symbols_transformer.names[id].clone());
            let symbol_id_rebuild = &symbols_rebuild.references[*reference_id_rebuild].symbol_id();
            let symbol_name_rebuild = symbol_id_rebuild.map(|id| symbols_rebuild.names[id].clone());
            if symbol_name_transformer != symbol_name_rebuild {
                let message = format!(
                    "
Reference mismatch:
after transform: {reference_id_transformer:?}: {symbol_name_transformer:?}
rebuilt        : {reference_id_rebuild:?}: {symbol_name_rebuild:?}
                    "
                );
                self.errors.push(OxcDiagnostic::error(message.trim().to_string()));
            }
        }
    }
}

/// Collector of `ScopeId`s, `SymbolId`s and `ReferenceId`s from an AST.
///
/// `scope_ids`, `symbol_ids` and `reference_ids` lists are filled in visitation order.
#[derive(Default)]
pub struct SemanticCollector {
    scope_ids: Vec<Option<ScopeId>>,
    symbol_ids: Vec<SymbolId>,
    reference_ids: Vec<ReferenceId>,

    errors: Vec<OxcDiagnostic>,
}

impl<'a> Visit<'a> for SemanticCollector {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        self.scope_ids.push(scope_id.get());
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(reference_id) = ident.reference_id.get() {
            self.reference_ids.push(reference_id);
        } else {
            let message = format!("Missing ReferenceId: {}", ident.name);
            self.errors.push(OxcDiagnostic::error(message).with_label(ident.span));
        }
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if let Some(symbol_id) = ident.symbol_id.get() {
            self.symbol_ids.push(symbol_id);
        } else {
            let message = format!("Missing SymbolId: {}", ident.name);
            self.errors.push(OxcDiagnostic::error(message).with_label(ident.span));
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
