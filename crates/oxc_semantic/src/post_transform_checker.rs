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
//! * "after transform": Semantic data from after the transformer has run
//! * "rebuilt": Semantic data from the fresh semantic analysis
//!
//! If the transformer has behaved correctly, the state of `ScopeTree` and `SymbolTable` should match
//! between "after transform" and "rebuilt".
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
//! When we ZIP these lists together, we have pairs of `(after_transform_id, rebuilt_id)` which give the
//! mapping between the 2 sets of IDs.
//!
//! ## Other notes
//!
//! See also: <https://github.com/oxc-project/oxc/issues/4790>

use std::cell::Cell;

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

/// Check `ScopeTree` and `SymbolTable` are correct after transform
pub fn check_semantic_after_transform(
    symbols_after_transform: &SymbolTable,
    scopes_after_transform: &ScopeTree,
    program: &Program<'_>,
) -> Option<Vec<OxcDiagnostic>> {
    // Collect `ScopeId`s, `SymbolId`s and `ReferenceId`s from AST after transformer
    let mut ids_after_transform = SemanticIds::default();
    if let Some(mut errors) = ids_after_transform.check(program) {
        errors.insert(0, OxcDiagnostic::error("Semantic Collector failed after transform"));
        return Some(errors);
    }
    let data_after_transform = SemanticData {
        symbols: symbols_after_transform,
        scopes: scopes_after_transform,
        ids: ids_after_transform,
    };

    // Clone the post-transform AST, re-run semantic analysis on it, and collect `ScopeId`s,
    // `SymbolId`s and `ReferenceId`s from AST.
    // NB: `CloneIn` sets all `scope_id`, `symbol_id` and `reference_id` fields in AST to `None`,
    // so the cloned AST will be "clean" of all semantic data, as if it had come fresh from the parser.
    let allocator = Allocator::default();
    let program = program.clone_in(&allocator);
    let (symbols_rebuilt, scopes_rebuilt) = SemanticBuilder::new("", program.source_type)
        .build(&program)
        .semantic
        .into_symbol_table_and_scope_tree();

    let mut ids_rebuilt = SemanticIds::default();
    if let Some(mut errors) = ids_rebuilt.check(&program) {
        errors.insert(0, OxcDiagnostic::error("Semantic Collector failed after rebuild"));
        return Some(errors);
    }
    let data_rebuilt =
        SemanticData { symbols: &symbols_rebuilt, scopes: &scopes_rebuilt, ids: ids_rebuilt };

    // Compare post-transform semantic data to semantic data from fresh semantic analysis
    let mut checker = PostTransformChecker {
        after_transform: data_after_transform,
        rebuilt: data_rebuilt,
        errors: Errors::default(),
    };
    checker.check_bindings();
    checker.check_symbols();
    checker.check_references();

    checker.errors.get()
}

struct PostTransformChecker<'s> {
    after_transform: SemanticData<'s>,
    rebuilt: SemanticData<'s>,
    errors: Errors,
}

struct SemanticData<'s> {
    symbols: &'s SymbolTable,
    scopes: &'s ScopeTree,
    ids: SemanticIds,
}

#[derive(Default)]
struct Errors(Vec<OxcDiagnostic>);

impl Errors {
    fn push<S: AsRef<str>>(&mut self, message: S) {
        self.0.push(OxcDiagnostic::error(message.as_ref().trim().to_string()));
    }

    fn get(self) -> Option<Vec<OxcDiagnostic>> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0)
        }
    }
}

impl<'s> PostTransformChecker<'s> {
    fn check_bindings(&mut self) {
        if self.after_transform.ids.scope_ids.len() != self.rebuilt.ids.scope_ids.len() {
            self.errors.push("Scopes mismatch after transform");
            return;
        }

        // Check whether bindings are the same for scopes in the same visitation order.
        for (&scope_id_after_transform, &scope_id_rebuilt) in
            self.after_transform.ids.scope_ids.iter().zip(self.rebuilt.ids.scope_ids.iter())
        {
            fn get_sorted_bindings(scopes: &ScopeTree, scope_id: ScopeId) -> Vec<CompactStr> {
                let mut bindings =
                    scopes.get_bindings(scope_id).keys().cloned().collect::<Vec<_>>();
                bindings.sort_unstable();
                bindings
            }

            let (result_after_transform, result_rebuilt) =
                match (scope_id_after_transform, scope_id_rebuilt) {
                    (None, None) => continue,
                    (Some(scope_id_after_transform), Some(scope_id_rebuilt)) => {
                        let bindings_after_transform = get_sorted_bindings(
                            self.after_transform.scopes,
                            scope_id_after_transform,
                        );
                        let bindings_rebuilt =
                            get_sorted_bindings(self.rebuilt.scopes, scope_id_rebuilt);
                        if bindings_after_transform == bindings_rebuilt {
                            continue;
                        }
                        (
                            format!("{scope_id_after_transform:?}: {bindings_after_transform:?}"),
                            format!("{scope_id_rebuilt:?}: {bindings_rebuilt:?}"),
                        )
                    }
                    (Some(scope_id_after_transform), None) => {
                        let bindings_after_transform = get_sorted_bindings(
                            self.after_transform.scopes,
                            scope_id_after_transform,
                        );
                        (
                            format!("{scope_id_after_transform:?}: {bindings_after_transform:?}"),
                            "No scope".to_string(),
                        )
                    }
                    (None, Some(scope_id_rebuilt)) => {
                        let bindings_rebuilt =
                            get_sorted_bindings(self.rebuilt.scopes, scope_id_rebuilt);
                        (
                            "No scope".to_string(),
                            format!("{scope_id_rebuilt:?}: {bindings_rebuilt:?}"),
                        )
                    }
                };

            self.errors.push(format!(
                "
Bindings mismatch:
after transform: {result_after_transform}
rebuilt        : {result_rebuilt}
                "
            ));
        }
    }

    fn check_symbols(&mut self) {
        // Check whether symbols are valid
        for symbol_id in self.rebuilt.ids.symbol_ids.iter().copied() {
            if self.rebuilt.symbols.get_flags(symbol_id).is_empty() {
                let name = self.rebuilt.symbols.get_name(symbol_id);
                self.errors
                    .push(format!("Expect non-empty SymbolFlags for BindingIdentifier({name})"));
                if !self
                    .rebuilt
                    .scopes
                    .has_binding(self.rebuilt.symbols.get_scope_id(symbol_id), name)
                {
                    self.errors.push(format!(
                        "Cannot find BindingIdentifier({name}) in the Scope corresponding to the Symbol"
                    ));
                }
            }
        }

        if self.after_transform.ids.symbol_ids.len() != self.rebuilt.ids.symbol_ids.len() {
            self.errors.push("Symbols mismatch after transform");
            return;
        }

        // Check whether symbols match
        for (&symbol_id_after_transform, &symbol_id_rebuilt) in
            self.after_transform.ids.symbol_ids.iter().zip(self.rebuilt.ids.symbol_ids.iter())
        {
            let symbol_name_after_transform =
                &self.after_transform.symbols.names[symbol_id_after_transform];
            let symbol_name_rebuilt = &self.rebuilt.symbols.names[symbol_id_rebuilt];
            if symbol_name_after_transform != symbol_name_rebuilt {
                self.errors.push(format!(
                    "
Symbol mismatch:
after transform: {symbol_id_after_transform:?}: {symbol_name_after_transform:?}
rebuilt        : {symbol_id_rebuilt:?}: {symbol_name_rebuilt:?}
                    "
                ));
            }
        }
    }

    fn check_references(&mut self) {
        // Check whether references are valid
        for reference_id in self.rebuilt.ids.reference_ids.iter().copied() {
            let reference = self.rebuilt.symbols.get_reference(reference_id);
            if reference.flags().is_empty() {
                self.errors.push(format!(
                    "Expect ReferenceFlags for IdentifierReference({reference_id:?}) to not be empty"
                ));
            }
        }

        if self.after_transform.ids.reference_ids.len() != self.rebuilt.ids.reference_ids.len() {
            self.errors.push("ReferenceId mismatch after transform");
            return;
        }

        // Check whether symbols match
        for (&reference_id_after_transform, &reference_id_rebuilt) in
            self.after_transform.ids.reference_ids.iter().zip(self.rebuilt.ids.reference_ids.iter())
        {
            let symbol_id_after_transform =
                self.after_transform.symbols.references[reference_id_after_transform].symbol_id();
            let symbol_name_after_transform =
                symbol_id_after_transform.map(|id| self.after_transform.symbols.names[id].clone());
            let symbol_id_rebuilt =
                &self.rebuilt.symbols.references[reference_id_rebuilt].symbol_id();
            let symbol_name_rebuilt =
                symbol_id_rebuilt.map(|id| self.rebuilt.symbols.names[id].clone());
            if symbol_name_after_transform != symbol_name_rebuilt {
                self.errors.push(format!(
                    "
Reference mismatch:
after transform: {reference_id_after_transform:?}: {symbol_name_after_transform:?}
rebuilt        : {reference_id_rebuilt:?}: {symbol_name_rebuilt:?}
                    "
                ));
            }
        }
    }
}

/// Collection of `ScopeId`s, `SymbolId`s and `ReferenceId`s from an AST.
///
/// `scope_ids`, `symbol_ids` and `reference_ids` lists are filled in visitation order.
#[derive(Default)]
pub struct SemanticIds {
    scope_ids: Vec<Option<ScopeId>>,
    symbol_ids: Vec<SymbolId>,
    reference_ids: Vec<ReferenceId>,
}

impl SemanticIds {
    /// Collect IDs and check for errors
    pub fn check(&mut self, program: &Program<'_>) -> Option<Vec<OxcDiagnostic>> {
        if program.source_type.is_typescript_definition() {
            return None;
        }

        let mut collector = SemanticIdsCollector::new(self);
        collector.visit_program(program);

        let errors = collector.errors;
        (!errors.is_empty()).then_some(errors)
    }
}

struct SemanticIdsCollector<'c> {
    ids: &'c mut SemanticIds,
    errors: Vec<OxcDiagnostic>,
}

impl<'c> SemanticIdsCollector<'c> {
    fn new(ids: &'c mut SemanticIds) -> Self {
        Self { ids, errors: vec![] }
    }
}

impl<'a, 'c> Visit<'a> for SemanticIdsCollector<'c> {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        self.ids.scope_ids.push(scope_id.get());
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(reference_id) = ident.reference_id.get() {
            self.ids.reference_ids.push(reference_id);
        } else {
            let message = format!("Missing ReferenceId: {}", ident.name);
            self.errors.push(OxcDiagnostic::error(message).with_label(ident.span));
        }
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if let Some(symbol_id) = ident.symbol_id.get() {
            self.ids.symbol_ids.push(symbol_id);
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
