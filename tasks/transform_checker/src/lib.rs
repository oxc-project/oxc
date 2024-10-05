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

use std::{
    cell::Cell,
    fmt::{Debug, Display},
    hash::{BuildHasherDefault, Hash},
};

use indexmap::IndexMap;
use rustc_hash::FxHasher;

use oxc_allocator::{Allocator, CloneIn};
#[allow(clippy::wildcard_imports, clippy::allow_attributes)]
use oxc_ast::{ast::*, visit::walk, Visit};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::{ScopeTree, SemanticBuilder, SymbolTable};
use oxc_span::CompactStr;
use oxc_syntax::{
    reference::ReferenceId,
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolId,
};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

/// Check `ScopeTree` and `SymbolTable` are correct after transform
pub fn check_semantic_after_transform(
    symbols_after_transform: &SymbolTable,
    scopes_after_transform: &ScopeTree,
    program: &Program,
) -> Option<Vec<OxcDiagnostic>> {
    let mut errors = Errors::default();

    let source_type = program.source_type;
    if !source_type.is_typescript_definition() && !source_type.is_javascript() {
        errors.push(format!("SourceType is not javascript: {source_type:?}"));
    }

    // Collect `ScopeId`s, `SymbolId`s and `ReferenceId`s from AST after transformer
    let scoping_after_transform =
        Scoping { symbols: symbols_after_transform, scopes: scopes_after_transform };
    let (
        scope_ids_after_transform,
        symbol_ids_after_transform,
        reference_ids_after_transform,
        reference_names,
    ) = SemanticIdsCollector::new(&mut errors).collect(program);

    // Clone the post-transform AST, re-run semantic analysis on it, and collect `ScopeId`s,
    // `SymbolId`s and `ReferenceId`s from AST.
    // NB: `CloneIn` sets all `scope_id`, `symbol_id` and `reference_id` fields in AST to `None`,
    // so the cloned AST will be "clean" of all semantic data, as if it had come fresh from the parser.
    let allocator = Allocator::default();
    let program = program.clone_in(&allocator);
    let (symbols_rebuilt, scopes_rebuilt) = SemanticBuilder::new("")
        .with_scope_tree_child_ids(scopes_after_transform.has_child_ids())
        .build(&program)
        .semantic
        .into_symbol_table_and_scope_tree();
    let scoping_rebuilt = Scoping { symbols: &symbols_rebuilt, scopes: &scopes_rebuilt };

    let (scope_ids_rebuilt, symbol_ids_rebuilt, reference_ids_rebuilt, _) =
        SemanticIdsCollector::new(&mut errors).collect(&program);

    // Create mappings from after transform IDs to rebuilt IDs
    let scope_ids_map = IdMapping::new(scope_ids_after_transform, scope_ids_rebuilt);
    let symbol_ids_map = IdMapping::new(symbol_ids_after_transform, symbol_ids_rebuilt);
    let reference_ids_map = IdMapping::new(reference_ids_after_transform, reference_ids_rebuilt);

    // Compare post-transform semantic data to semantic data from fresh semantic analysis
    let mut checker = PostTransformChecker {
        scoping_after_transform,
        scoping_rebuilt,
        scope_ids_map,
        symbol_ids_map,
        reference_ids_map,
        reference_names,
        errors,
    };
    checker.check_scopes();
    checker.check_symbols();
    checker.check_references();
    checker.check_unresolved_references();

    checker.errors.get()
}

/// Check all AST nodes have scope, symbol and reference IDs
pub fn check_semantic_ids(program: &Program) -> Option<Vec<OxcDiagnostic>> {
    let mut errors = Errors::default();
    SemanticIdsCollector::new(&mut errors).collect(program);
    errors.get()
}

struct PostTransformChecker<'a, 's> {
    scoping_after_transform: Scoping<'s>,
    scoping_rebuilt: Scoping<'s>,
    // Mappings from after transform ID to rebuilt ID
    scope_ids_map: IdMapping<ScopeId>,
    symbol_ids_map: IdMapping<SymbolId>,
    reference_ids_map: IdMapping<ReferenceId>,
    reference_names: Vec<Atom<'a>>,
    errors: Errors,
}

struct Scoping<'s> {
    symbols: &'s SymbolTable,
    scopes: &'s ScopeTree,
}

/// Mapping from "after transform" ID to "rebuilt" ID
struct IdMapping<Id>(FxIndexMap<Id, Id>);

impl<Id: Copy + Eq + Hash> IdMapping<Id> {
    fn new(after_transform: Vec<Option<Id>>, rebuilt: Vec<Option<Id>>) -> Self {
        let map = after_transform
            .into_iter()
            .zip(rebuilt)
            .filter_map(|ids| match ids {
                (Some(after_transform_id), Some(rebuilt_id)) => {
                    Some((after_transform_id, rebuilt_id))
                }
                _ => None,
            })
            .collect();
        Self(map)
    }

    fn get(&self, after_transform_id: Id) -> Option<Id> {
        self.0.get(&after_transform_id).copied()
    }

    /// Iterate over pairs of after transform ID and rebuilt ID
    fn pairs(&self) -> impl Iterator<Item = Pair<Id>> + '_ {
        self.0
            .iter()
            .map(|(&after_transform_id, &rebuilt_id)| Pair::new(after_transform_id, rebuilt_id))
    }
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

    fn parts(&self) -> (&T, &T) {
        (&self.after_transform, &self.rebuilt)
    }

    fn into_parts(self) -> (T, T) {
        (self.after_transform, self.rebuilt)
    }

    fn map<U, F: Fn(&T) -> U>(&self, mapper: F) -> Pair<U> {
        Pair::new(mapper(&self.after_transform), mapper(&self.rebuilt))
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

#[expect(clippy::expl_impl_clone_on_copy)]
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
    fn push_mismatch_single<Value, Values>(&mut self, title: &str, values: Values)
    where
        Value: Debug,
        Values: AsRef<Pair<Value>>,
    {
        self.push_mismatch_strs(title, values.as_ref().map(|value| format!("{value:?}")));
    }

    /// Add an error for a mismatch between a pair of values, without `Debug` formatting
    fn push_mismatch_strs<Value, Values>(&mut self, title: &str, values: Values)
    where
        Value: Display,
        Values: AsRef<Pair<Value>>,
    {
        let (value_after_transform, value_rebuilt) = values.as_ref().parts();
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

impl<'a, 's> PostTransformChecker<'a, 's> {
    fn check_scopes(&mut self) {
        for scope_ids in self.scope_ids_map.pairs() {
            // Check bindings are the same
            fn get_sorted_binding_names(scoping: &Scoping, scope_id: ScopeId) -> Vec<CompactStr> {
                let mut binding_names =
                    scoping.scopes.get_bindings(scope_id).keys().cloned().collect::<Vec<_>>();
                binding_names.sort_unstable();
                binding_names
            }

            let binding_names = self.get_pair(scope_ids, get_sorted_binding_names);
            if binding_names.is_mismatch() {
                self.errors.push_mismatch("Bindings mismatch", scope_ids, binding_names);
            } else {
                let symbol_ids = self.get_pair(scope_ids, |scoping, scope_id| {
                    scoping.scopes.get_bindings(scope_id).values().copied().collect::<Vec<_>>()
                });
                if self.remap_symbol_ids_sets(&symbol_ids).is_mismatch() {
                    self.errors.push_mismatch("Binding symbols mismatch", scope_ids, symbol_ids);
                }
            }

            // Check flags match
            let flags =
                self.get_pair(scope_ids, |scoping, scope_id| scoping.scopes.get_flags(scope_id));
            if flags.is_mismatch() {
                self.errors.push_mismatch("Scope flags mismatch", scope_ids, flags);
            }

            // Check parents match
            let parent_ids = self
                .get_pair(scope_ids, |scoping, scope_id| scoping.scopes.get_parent_id(scope_id));
            let is_match = match parent_ids.into_parts() {
                (Some(parent_id_after_transform), Some(parent_id_rebuilt)) => {
                    let parent_ids = Pair::new(parent_id_after_transform, parent_id_rebuilt);
                    self.remap_scope_ids(parent_ids).is_match()
                }
                (None, None) => true,
                _ => false,
            };
            if !is_match {
                self.errors.push_mismatch("Scope parent mismatch", scope_ids, parent_ids);
            }

            // Check children match
            if self.scoping_after_transform.scopes.has_child_ids() {
                let child_ids = self.get_pair(scope_ids, |scoping, scope_id| {
                    scoping.scopes.get_child_ids(scope_id).to_vec()
                });
                if self.remap_scope_ids_sets(&child_ids).is_mismatch() {
                    self.errors.push_mismatch("Scope children mismatch", scope_ids, child_ids);
                }
            }

            // NB: Skip checking node IDs match - transformer does not set `NodeId`s
        }
    }

    fn check_symbols(&mut self) {
        for symbol_ids in self.symbol_ids_map.pairs() {
            // Check names match
            let names = self.get_pair(symbol_ids, |scoping, symbol_id| {
                scoping.symbols.names[symbol_id].clone()
            });
            if names.is_mismatch() {
                self.errors.push_mismatch("Symbol name mismatch", symbol_ids, &names);
            }
            let symbol_name = names.rebuilt.as_str();
            let mismatch_title = |field| format!("Symbol {field} mismatch for {symbol_name:?}");

            // Check flags match
            let flags =
                self.get_pair(symbol_ids, |scoping, symbol_id| scoping.symbols.flags[symbol_id]);
            if flags.is_mismatch() {
                self.errors.push_mismatch(&mismatch_title("flags"), symbol_ids, flags);
            }

            // Check spans match
            let spans =
                self.get_pair(symbol_ids, |scoping, symbol_id| scoping.symbols.spans[symbol_id]);
            if spans.is_mismatch() {
                self.errors.push_mismatch(&mismatch_title("span"), symbol_ids, spans);
            }

            // Check scope IDs match
            let scope_ids = self
                .get_pair(symbol_ids, |scoping, symbol_id| scoping.symbols.scope_ids[symbol_id]);
            if self.remap_scope_ids(scope_ids).is_mismatch() {
                self.errors.push_mismatch(&mismatch_title("scope ID"), symbol_ids, scope_ids);
            }

            // NB: Skip checking declarations match - transformer does not set `NodeId`s

            // Check resolved references match
            let reference_ids = self.get_pair(symbol_ids, |scoping, symbol_id| {
                scoping.symbols.resolved_references[symbol_id].clone()
            });
            if self.remap_reference_ids_sets(&reference_ids).is_mismatch() {
                self.errors.push_mismatch(
                    &mismatch_title("reference IDs"),
                    symbol_ids,
                    reference_ids,
                );
            }

            // Check redeclarations match
            let redeclaration_spans = self.get_pair(symbol_ids, |scoping, symbol_id| {
                let mut spans = scoping.symbols.get_redeclarations(symbol_id).to_vec();
                spans.sort_unstable();
                spans
            });
            if redeclaration_spans.is_mismatch() {
                self.errors.push_mismatch(
                    &mismatch_title("redeclarations"),
                    symbol_ids,
                    redeclaration_spans,
                );
            }
        }
    }

    fn check_references(&mut self) {
        for (reference_ids, name) in self.reference_ids_map.pairs().zip(&self.reference_names) {
            // Check symbol IDs match
            let symbol_ids = self.get_pair(reference_ids, |scoping, reference_id| {
                scoping.symbols.references[reference_id].symbol_id()
            });
            let symbol_ids_remapped = Pair::new(
                symbol_ids.after_transform.map(|symbol_id| self.symbol_ids_map.get(symbol_id)),
                symbol_ids.rebuilt.map(Option::Some),
            );
            if symbol_ids_remapped.is_mismatch() {
                let mismatch_strs = self.get_pair(reference_ids, |scoping, reference_id| {
                    match scoping.symbols.references[reference_id].symbol_id() {
                        Some(symbol_id) => {
                            let symbol_name = &scoping.symbols.names[symbol_id];
                            format!("{symbol_id:?} {symbol_name:?}")
                        }
                        None => "<None>".to_string(),
                    }
                });
                self.errors.push_mismatch_strs(
                    &format!("Reference symbol mismatch for {name:?}"),
                    mismatch_strs,
                );
            }

            // Check flags match
            let flags = self.get_pair(reference_ids, |scoping, reference_id| {
                scoping.symbols.references[reference_id].flags()
            });
            if flags.is_mismatch() {
                self.errors.push_mismatch(
                    &format!("Reference flags mismatch for {name:?}"),
                    reference_ids,
                    flags,
                );
            }
        }
    }

    fn check_unresolved_references(&mut self) {
        let unresolved_names = self.get_static_pair(|scoping| {
            let mut names =
                scoping.scopes.root_unresolved_references().keys().cloned().collect::<Vec<_>>();
            names.sort_unstable();
            names
        });
        if unresolved_names.is_mismatch() {
            self.errors.push_mismatch_single("Unresolved references mismatch", unresolved_names);
        }

        for (name, reference_ids_after_transform) in
            self.scoping_after_transform.scopes.root_unresolved_references()
        {
            if let Some(reference_ids_rebuilt) =
                self.scoping_rebuilt.scopes.root_unresolved_references().get(name)
            {
                let reference_ids = Pair::new(reference_ids_after_transform, reference_ids_rebuilt);
                if self.remap_reference_ids_sets(&reference_ids).is_mismatch() {
                    self.errors.push_mismatch_single(
                        &format!("Unresolved reference IDs mismatch for {name:?}"),
                        reference_ids,
                    );
                }
            }
        }
    }

    /// Get same data from `after_transform` and `rebuilt` from a pair of IDs
    fn get_pair<R, I: Copy, F: Fn(&Scoping, I) -> R>(&self, ids: Pair<I>, getter: F) -> Pair<R> {
        Pair::new(
            getter(&self.scoping_after_transform, ids.after_transform),
            getter(&self.scoping_rebuilt, ids.rebuilt),
        )
    }

    /// Get same data from `after_transform` and `rebuilt`
    fn get_static_pair<R, F: Fn(&Scoping) -> R>(&self, getter: F) -> Pair<R> {
        Pair::new(getter(&self.scoping_after_transform), getter(&self.scoping_rebuilt))
    }

    /// Map `after_transform` scope ID to `rebuilt` scope ID
    fn remap_scope_ids(&self, scope_ids: Pair<ScopeId>) -> Pair<Option<ScopeId>> {
        Pair::new(self.scope_ids_map.get(scope_ids.after_transform), Some(scope_ids.rebuilt))
    }

    /// Remap pair of arrays of `ScopeId`s.
    /// Map `after_transform` IDs to `rebuilt` IDs.
    /// Sort both sets.
    fn remap_scope_ids_sets<V: AsRef<Vec<ScopeId>>>(
        &self,
        scope_ids: &Pair<V>,
    ) -> Pair<Vec<Option<ScopeId>>> {
        let mut after_transform = scope_ids
            .after_transform
            .as_ref()
            .iter()
            .map(|&scope_id| self.scope_ids_map.get(scope_id))
            .collect::<Vec<_>>();
        let mut rebuilt =
            scope_ids.rebuilt.as_ref().iter().copied().map(Option::Some).collect::<Vec<_>>();

        after_transform.sort_unstable();
        rebuilt.sort_unstable();

        Pair::new(after_transform, rebuilt)
    }

    /// Remap pair of arrays of `SymbolId`s.
    /// Map `after_transform` IDs to `rebuilt` IDs.
    /// Sort both sets.
    fn remap_symbol_ids_sets<V: AsRef<Vec<SymbolId>>>(
        &self,
        symbol_ids: &Pair<V>,
    ) -> Pair<Vec<Option<SymbolId>>> {
        let mut after_transform = symbol_ids
            .after_transform
            .as_ref()
            .iter()
            .map(|&symbol_id| self.symbol_ids_map.get(symbol_id))
            .collect::<Vec<_>>();
        let mut rebuilt =
            symbol_ids.rebuilt.as_ref().iter().copied().map(Option::Some).collect::<Vec<_>>();

        after_transform.sort_unstable();
        rebuilt.sort_unstable();

        Pair::new(after_transform, rebuilt)
    }

    /// Remap pair of arrays of `ReferenceId`s.
    /// Map `after_transform` IDs to `rebuilt` IDs.
    /// Sort both sets.
    fn remap_reference_ids_sets<V: AsRef<Vec<ReferenceId>>>(
        &self,
        reference_ids: &Pair<V>,
    ) -> Pair<Vec<Option<ReferenceId>>> {
        let mut after_transform = reference_ids
            .after_transform
            .as_ref()
            .iter()
            .map(|&reference_id| self.reference_ids_map.get(reference_id))
            .collect::<Vec<_>>();
        let mut rebuilt =
            reference_ids.rebuilt.as_ref().iter().copied().map(Option::Some).collect::<Vec<_>>();

        after_transform.sort_unstable();
        rebuilt.sort_unstable();

        Pair::new(after_transform, rebuilt)
    }
}

/// Collector of `ScopeId`s, `SymbolId`s and `ReferenceId`s from an AST.
///
/// `scope_ids`, `symbol_ids` and `reference_ids` lists are filled in visitation order.
struct SemanticIdsCollector<'a, 'e> {
    scope_ids: Vec<Option<ScopeId>>,
    symbol_ids: Vec<Option<SymbolId>>,
    reference_ids: Vec<Option<ReferenceId>>,
    reference_names: Vec<Atom<'a>>,
    errors: &'e mut Errors,
}

impl<'a, 'e> SemanticIdsCollector<'a, 'e> {
    fn new(errors: &'e mut Errors) -> Self {
        Self {
            scope_ids: vec![],
            symbol_ids: vec![],
            reference_ids: vec![],
            reference_names: vec![],
            errors,
        }
    }

    /// Collect IDs and check for errors
    #[expect(clippy::type_complexity)]
    fn collect(
        mut self,
        program: &Program<'a>,
    ) -> (Vec<Option<ScopeId>>, Vec<Option<SymbolId>>, Vec<Option<ReferenceId>>, Vec<Atom<'a>>)
    {
        if !program.source_type.is_typescript_definition() {
            self.visit_program(program);
        }
        (self.scope_ids, self.symbol_ids, self.reference_ids, self.reference_names)
    }
}

impl<'a, 'e> Visit<'a> for SemanticIdsCollector<'a, 'e> {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        let scope_id = scope_id.get();
        self.scope_ids.push(scope_id);
        if scope_id.is_none() {
            self.errors.push("Missing ScopeId");
        }
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let reference_id = ident.reference_id.get();
        self.reference_ids.push(reference_id);
        if reference_id.is_some() {
            self.reference_names.push(ident.name.clone());
        } else {
            self.errors.push(format!("Missing ReferenceId: {:?}", ident.name));
        }
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        let symbol_id = ident.symbol_id.get();
        self.symbol_ids.push(symbol_id);
        if symbol_id.is_none() {
            self.errors.push(format!("Missing SymbolId: {:?}", ident.name));
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

    fn visit_ts_type(&mut self, _it: &TSType<'a>) {
        /* noop */
    }
}
