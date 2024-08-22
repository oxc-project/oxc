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

use std::{cell::Cell, fmt::Debug};

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
use rustc_hash::FxHashMap;

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
        scope_ids_map: FxHashMap::default(),
        symbol_ids_map: FxHashMap::default(),
        reference_ids_map: FxHashMap::default(),
        errors: Errors::default(),
    };
    checker.create_mappings();
    checker.check_scopes();
    checker.check_symbols();
    checker.check_references();

    checker.errors.get()
}

struct PostTransformChecker<'s> {
    after_transform: SemanticData<'s>,
    rebuilt: SemanticData<'s>,
    // Mappings from after transform ID to rebuilt ID
    scope_ids_map: FxHashMap<ScopeId, ScopeId>,
    symbol_ids_map: FxHashMap<SymbolId, SymbolId>,
    reference_ids_map: FxHashMap<ReferenceId, ReferenceId>,
    errors: Errors,
}

struct SemanticData<'s> {
    symbols: &'s SymbolTable,
    scopes: &'s ScopeTree,
    ids: SemanticIds,
}

/// Pair of values from after transform and rebuilt
struct Pair<T> {
    after_transform: T,
    rebuilt: T,
}

impl<T> Pair<T> {
    fn new(after_transform: T, rebuilt: T) -> Self {
        Self { after_transform, rebuilt }
    }

    fn from_tuple(values: (T, T)) -> Self {
        Self::new(values.0, values.1)
    }

    fn parts(&self) -> (&T, &T) {
        (&self.after_transform, &self.rebuilt)
    }

    fn into_parts(self) -> (T, T) {
        (self.after_transform, self.rebuilt)
    }
}

impl<T: PartialEq> Pair<T> {
    fn is_match(&self) -> bool {
        self.after_transform == self.rebuilt
    }

    fn is_mismatch(&self) -> bool {
        !self.is_match()
    }
}

impl<T> AsRef<Pair<T>> for Pair<T> {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[allow(clippy::expl_impl_clone_on_copy)]
impl<T: Clone> Clone for Pair<T> {
    fn clone(&self) -> Self {
        Self::new(self.after_transform.clone(), self.rebuilt.clone())
    }
}

impl<T: Copy> Copy for Pair<T> {}

/// Errors collection
#[derive(Default)]
struct Errors(Vec<OxcDiagnostic>);

impl Errors {
    /// Add an error string
    fn push<S: AsRef<str>>(&mut self, message: S) {
        self.0.push(OxcDiagnostic::error(message.as_ref().trim().to_string()));
    }

    /// Add an error for a mismatch between a pair of values, with IDs
    fn push_mismatch<Id, Ids, Value, Values>(&mut self, title: &str, ids: Ids, values: Values)
    where
        Id: Debug,
        Value: Debug,
        Ids: AsRef<Pair<Id>>,
        Values: AsRef<Pair<Value>>,
    {
        let (id_after_transform, id_rebuilt) = ids.as_ref().parts();
        let (value_after_transform, value_rebuilt) = values.as_ref().parts();
        let str_after_transform = format!("{id_after_transform:?}: {value_after_transform:?}");
        let str_rebuilt = format!("{id_rebuilt:?}: {value_rebuilt:?}");
        self.push_mismatch_strs(title, Pair::new(str_after_transform, str_rebuilt));
    }

    /// Add an error for a mismatch between a pair of values
    fn push_mismatch_strs<Value, Values>(&mut self, title: &str, values: Values)
    where
        Value: AsRef<str>,
        Values: AsRef<Pair<Value>>,
    {
        let (value_after_transform, value_rebuilt) = values.as_ref().parts();
        let value_after_transform = value_after_transform.as_ref();
        let value_rebuilt = value_rebuilt.as_ref();
        self.push(format!(
            "
{title}:
after transform: {value_after_transform}
rebuilt        : {value_rebuilt}
            "
        ));
    }

    /// Get errors
    fn get(self) -> Option<Vec<OxcDiagnostic>> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0)
        }
    }
}

impl<'s> PostTransformChecker<'s> {
    fn create_mappings(&mut self) {
        // Scope IDs
        for (&scope_id_after_transform, &scope_id_rebuilt) in
            self.after_transform.ids.scope_ids.iter().zip(self.rebuilt.ids.scope_ids.iter())
        {
            let (Some(scope_id_after_transform), Some(scope_id_rebuilt)) =
                (scope_id_after_transform, scope_id_rebuilt)
            else {
                continue;
            };
            self.scope_ids_map.insert(scope_id_after_transform, scope_id_rebuilt);
        }

        // Symbol IDs
        for (&symbol_id_after_transform, &symbol_id_rebuilt) in
            self.after_transform.ids.symbol_ids.iter().zip(self.rebuilt.ids.symbol_ids.iter())
        {
            self.symbol_ids_map.insert(symbol_id_after_transform, symbol_id_rebuilt);
        }

        // Reference IDs
        for (&reference_id_after_transform, &reference_id_rebuilt) in
            self.after_transform.ids.reference_ids.iter().zip(self.rebuilt.ids.reference_ids.iter())
        {
            self.reference_ids_map.insert(reference_id_after_transform, reference_id_rebuilt);
        }
    }

    fn check_scopes(&mut self) {
        if self.get_static_pair(|data| data.ids.scope_ids.len()).is_mismatch() {
            self.errors.push("Scopes mismatch after transform");
        }

        for scope_ids in self
            .after_transform
            .ids
            .scope_ids
            .iter()
            .copied()
            .zip(self.rebuilt.ids.scope_ids.iter().copied())
            .map(Pair::from_tuple)
        {
            // Check bindings are the same
            fn get_sorted_bindings(data: &SemanticData, scope_id: ScopeId) -> Vec<CompactStr> {
                let mut bindings =
                    data.scopes.get_bindings(scope_id).keys().cloned().collect::<Vec<_>>();
                bindings.sort_unstable();
                bindings
            }

            let scope_ids = match scope_ids.into_parts() {
                (None, None) => continue,
                (Some(scope_id_after_transform), Some(scope_id_rebuilt)) => {
                    let scope_ids = Pair::new(scope_id_after_transform, scope_id_rebuilt);
                    let bindings = self.get_pair(scope_ids, get_sorted_bindings);
                    if bindings.is_mismatch() {
                        self.errors.push_mismatch("Bindings mismatch", scope_ids, bindings);
                    }
                    scope_ids
                }
                (Some(scope_id), None) => {
                    let bindings = get_sorted_bindings(&self.after_transform, scope_id);
                    self.errors.push_mismatch_strs(
                        "Bindings mismatch",
                        Pair::new(format!("{scope_id:?}: {bindings:?}").as_str(), "No scope"),
                    );
                    continue;
                }
                (None, Some(scope_id)) => {
                    let bindings = get_sorted_bindings(&self.rebuilt, scope_id);
                    self.errors.push_mismatch_strs(
                        "Bindings mismatch",
                        Pair::new("No scope", format!("{scope_id:?}: {bindings:?}").as_str()),
                    );
                    continue;
                }
            };

            // Check flags match
            let flags = self.get_pair(scope_ids, |data, scope_id| data.scopes.get_flags(scope_id));
            if flags.is_mismatch() {
                self.errors.push_mismatch("Scope flags mismatch", scope_ids, flags);
            }

            // Check parents match
            let parent_ids =
                self.get_pair(scope_ids, |data, scope_id| data.scopes.get_parent_id(scope_id));
            let is_match = match parent_ids.into_parts() {
                (Some(parent_id_after_transform), Some(parent_id_rebuilt)) => {
                    let parent_id_after_transform =
                        self.scope_ids_map.get(&parent_id_after_transform);
                    parent_id_after_transform == Some(&parent_id_rebuilt)
                }
                (None, None) => true,
                _ => false,
            };
            if !is_match {
                self.errors.push_mismatch("Scope parent mismatch", scope_ids, parent_ids);
            }
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

        if self.get_static_pair(|data| data.ids.symbol_ids.len()).is_mismatch() {
            self.errors.push("Symbols mismatch after transform");
            return;
        }

        // Check whether symbols match
        for symbol_ids in self
            .after_transform
            .ids
            .symbol_ids
            .iter()
            .copied()
            .zip(self.rebuilt.ids.symbol_ids.iter().copied())
            .map(Pair::from_tuple)
        {
            let symbol_names =
                self.get_pair(symbol_ids, |data, symbol_id| data.symbols.names[symbol_id].clone());
            if symbol_names.is_mismatch() {
                self.errors.push_mismatch("Symbol mismatch", symbol_ids, symbol_names);
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

        if self.get_static_pair(|data| data.ids.reference_ids.len()).is_mismatch() {
            self.errors.push("ReferenceId mismatch after transform");
            return;
        }

        // Check whether symbols match
        for reference_ids in self
            .after_transform
            .ids
            .reference_ids
            .iter()
            .copied()
            .zip(self.rebuilt.ids.reference_ids.iter().copied())
            .map(Pair::from_tuple)
        {
            let symbol_ids = self.get_pair(reference_ids, |data, reference_id| {
                data.symbols.references[reference_id].symbol_id()
            });
            let symbol_names = self.get_pair(symbol_ids, |data, symbol_id| {
                symbol_id.map(|symbol_id| data.symbols.names[symbol_id].clone())
            });
            if symbol_names.is_mismatch() {
                self.errors.push_mismatch("Reference mismatch", reference_ids, symbol_names);
            }
        }
    }

    /// Get same data from `after_transform` and `rebuilt` from a pair of IDs
    fn get_pair<R, I: Copy, F: Fn(&SemanticData, I) -> R>(
        &self,
        ids: Pair<I>,
        getter: F,
    ) -> Pair<R> {
        Pair::new(
            getter(&self.after_transform, ids.after_transform),
            getter(&self.rebuilt, ids.rebuilt),
        )
    }

    /// Get same data from `after_transform` and `rebuilt`
    fn get_static_pair<R, F: Fn(&SemanticData) -> R>(&self, getter: F) -> Pair<R> {
        Pair::new(getter(&self.after_transform), getter(&self.rebuilt))
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
