//! Semantic Builder

use std::{
    cell::{Cell, RefCell},
    mem,
};

use oxc_data_structures::stack::Stack;
use rustc_hash::FxHashMap;

use oxc_ast::{ast::*, AstKind, Visit};
use oxc_cfg::{
    ControlFlowGraphBuilder, CtxCursor, CtxFlags, EdgeType, ErrorEdgeKind, InstructionKind,
    IterationInstructionKind, ReturnInstructionKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Atom, SourceType, Span};
use oxc_syntax::{
    node::{NodeFlags, NodeId},
    reference::{Reference, ReferenceFlags, ReferenceId},
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

use crate::{
    binder::Binder,
    checker,
    class::ClassTableBuilder,
    diagnostics::redeclaration,
    jsdoc::JSDocBuilder,
    label::UnusedLabels,
    node::AstNodes,
    scope::{Bindings, ScopeTree},
    stats::Stats,
    symbol::SymbolTable,
    unresolved_stack::UnresolvedReferencesStack,
    JSDocFinder, Semantic,
};

macro_rules! control_flow {
    ($self:ident, |$cfg:tt| $body:expr) => {
        if let Some($cfg) = &mut $self.cfg {
            $body
        } else {
            Default::default()
        }
    };
}

/// Semantic Builder
///
/// Traverses a parsed AST and builds a [`Semantic`] representation of the
/// program.
///
/// The main API is the [`build`] method.
///
/// # Example
///
/// ```rust
#[doc = include_str!("../examples/semantic.rs")]
/// ```
///
/// [`build`]: SemanticBuilder::build
pub struct SemanticBuilder<'a> {
    /// source code of the parsed program
    pub(crate) source_text: &'a str,

    /// source type of the parsed program
    pub(crate) source_type: SourceType,

    /// Semantic early errors such as redeclaration errors.
    pub(crate) errors: RefCell<Vec<OxcDiagnostic>>,

    // states
    pub(crate) current_node_id: NodeId,
    pub(crate) current_node_flags: NodeFlags,
    pub(crate) current_scope_id: ScopeId,
    /// Stores current `AstKind::Function` and `AstKind::ArrowFunctionExpression` during AST visit
    pub(crate) function_stack: Stack<NodeId>,
    // To make a namespace/module value like
    // we need the to know the modules we are inside
    // and when we reach a value declaration we set it
    // to value like
    pub(crate) namespace_stack: Vec<Option<SymbolId>>,
    current_reference_flags: ReferenceFlags,
    pub(crate) hoisting_variables: FxHashMap<ScopeId, FxHashMap<Atom<'a>, SymbolId>>,

    // builders
    pub(crate) nodes: AstNodes<'a>,
    pub(crate) scope: ScopeTree,
    pub(crate) symbols: SymbolTable,

    pub(crate) unresolved_references: UnresolvedReferencesStack<'a>,

    unused_labels: UnusedLabels<'a>,
    build_jsdoc: bool,
    jsdoc: JSDocBuilder<'a>,
    stats: Option<Stats>,
    excess_capacity: f64,

    /// Should additional syntax checks be performed?
    ///
    /// See: [`crate::checker::check`]
    check_syntax_error: bool,

    pub(crate) cfg: Option<ControlFlowGraphBuilder<'a>>,

    pub(crate) class_table_builder: ClassTableBuilder,

    ast_node_records: Vec<NodeId>,
}

/// Data returned by [`SemanticBuilder::build`].
pub struct SemanticBuilderReturn<'a> {
    pub semantic: Semantic<'a>,
    pub errors: Vec<OxcDiagnostic>,
}

impl Default for SemanticBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SemanticBuilder<'a> {
    pub fn new() -> Self {
        let scope = ScopeTree::default();
        let current_scope_id = scope.root_scope_id();

        Self {
            source_text: "",
            source_type: SourceType::default(),
            errors: RefCell::new(vec![]),
            current_node_id: NodeId::new(0),
            current_node_flags: NodeFlags::empty(),
            current_reference_flags: ReferenceFlags::empty(),
            current_scope_id,
            function_stack: Stack::with_capacity(16),
            namespace_stack: vec![],
            nodes: AstNodes::default(),
            hoisting_variables: FxHashMap::default(),
            scope,
            symbols: SymbolTable::default(),
            unresolved_references: UnresolvedReferencesStack::new(),
            unused_labels: UnusedLabels::default(),
            build_jsdoc: false,
            jsdoc: JSDocBuilder::default(),
            stats: None,
            excess_capacity: 0.0,
            check_syntax_error: false,
            cfg: None,
            class_table_builder: ClassTableBuilder::new(),
            ast_node_records: Vec::new(),
        }
    }

    /// Enable/disable additional syntax checks.
    ///
    /// Set this to `true` to enable additional syntax checks. Without these,
    /// there is no guarantee that the parsed program follows the ECMAScript
    /// spec.
    ///
    /// By default, this is `false`.
    #[must_use]
    pub fn with_check_syntax_error(mut self, yes: bool) -> Self {
        self.check_syntax_error = yes;
        self
    }

    /// Enable/disable JSDoc parsing.
    #[must_use]
    pub fn with_build_jsdoc(mut self, yes: bool) -> Self {
        self.build_jsdoc = yes;
        self
    }

    /// Enable or disable building a [`ControlFlowGraph`].
    ///
    /// [`ControlFlowGraph`]: oxc_cfg::ControlFlowGraph
    #[must_use]
    pub fn with_cfg(mut self, cfg: bool) -> Self {
        self.cfg = if cfg { Some(ControlFlowGraphBuilder::default()) } else { None };
        self
    }

    #[must_use]
    pub fn with_scope_tree_child_ids(mut self, yes: bool) -> Self {
        self.scope.build_child_ids = yes;
        self
    }

    /// Provide statistics about AST to optimize memory usage of semantic analysis.
    ///
    /// Accurate statistics can greatly improve performance, especially for large ASTs.
    /// If no stats are provided, [`SemanticBuilder::build`] will compile stats by performing
    /// a complete AST traversal.
    /// If semantic analysis has already been performed on this AST, get the existing stats with
    /// [`Semantic::stats`], and pass them in with this method, to avoid the stats collection AST pass.
    #[must_use]
    pub fn with_stats(mut self, stats: Stats) -> Self {
        self.stats = Some(stats);
        self
    }

    /// Request `SemanticBuilder` to allocate excess capacity for scopes, symbols, and references.
    ///
    /// `excess_capacity` is provided as a fraction.
    /// e.g. to over-allocate by 20%, pass `0.2` as `excess_capacity`.
    ///
    /// Has no effect if a `Stats` object is provided with [`SemanticBuilder::with_stats`],
    /// only if `SemanticBuilder` is calculating stats itself.
    ///
    /// This is useful when you intend to modify `Semantic`, adding more `nodes`, `scopes`, `symbols`,
    /// or `references`. Allocating excess capacity for these additions at the outset prevents
    /// `Semantic`'s data structures needing to grow later on which involves memory copying.
    /// For large ASTs with a lot of semantic data, re-allocation can be very costly.
    #[must_use]
    pub fn with_excess_capacity(mut self, excess_capacity: f64) -> Self {
        self.excess_capacity = excess_capacity;
        self
    }

    /// Finalize the builder.
    ///
    /// # Panics
    pub fn build(mut self, program: &'a Program<'a>) -> SemanticBuilderReturn<'a> {
        self.source_text = program.source_text;
        self.source_type = program.source_type;
        if self.build_jsdoc {
            self.jsdoc = JSDocBuilder::new(self.source_text, &program.comments);
        }
        if self.source_type.is_typescript_definition() {
            let scope_id = self.scope.add_scope(None, NodeId::DUMMY, ScopeFlags::Top);
            program.scope_id.set(Some(scope_id));
        } else {
            // Use counts of nodes, scopes, symbols, and references to pre-allocate sufficient capacity
            // in `AstNodes`, `ScopeTree` and `SymbolTable`.
            //
            // This means that as we traverse the AST and fill up these structures with data,
            // they never need to grow and reallocate - which is an expensive operation as it
            // involves copying all the memory from the old allocation to the new one.
            // For large source files, these structures are very large, so growth is very costly
            // as it involves copying massive chunks of memory.
            // Avoiding this growth produces up to 30% perf boost on our benchmarks.
            //
            // If user did not provide existing `Stats`, calculate them by visiting AST.
            #[cfg_attr(not(debug_assertions), expect(unused_variables))]
            let (stats, check_stats) = if let Some(stats) = self.stats {
                (stats, None)
            } else {
                let stats = Stats::count(program);
                let stats_with_excess = stats.increase_by(self.excess_capacity);
                (stats_with_excess, Some(stats))
            };
            self.nodes.reserve(stats.nodes as usize);
            self.scope.reserve(stats.scopes as usize);
            self.symbols.reserve(stats.symbols as usize, stats.references as usize);

            // Visit AST to generate scopes tree etc
            self.visit_program(program);

            // Check that estimated counts accurately (unless in release mode)
            #[cfg(debug_assertions)]
            if let Some(stats) = check_stats {
                #[allow(clippy::cast_possible_truncation)]
                let actual_stats = Stats::new(
                    self.nodes.len() as u32,
                    self.scope.len() as u32,
                    self.symbols.len() as u32,
                    self.symbols.references.len() as u32,
                );
                stats.assert_accurate(actual_stats);
            }
        }

        let comments = self.alloc(&program.comments);

        debug_assert_eq!(self.unresolved_references.scope_depth(), 1);
        if self.check_syntax_error && !self.source_type.is_typescript() {
            checker::check_unresolved_exports(&self);
        }
        self.scope.set_root_unresolved_references(
            self.unresolved_references.into_root().into_iter().map(|(k, v)| (k.as_str(), v)),
        );

        let jsdoc = if self.build_jsdoc { self.jsdoc.build() } else { JSDocFinder::default() };

        let semantic = Semantic {
            source_text: self.source_text,
            source_type: self.source_type,
            comments,
            irregular_whitespaces: [].into(),
            nodes: self.nodes,
            scopes: self.scope,
            symbols: self.symbols,
            classes: self.class_table_builder.build(),
            jsdoc,
            unused_labels: self.unused_labels.labels,
            cfg: self.cfg.map(ControlFlowGraphBuilder::build),
        };
        SemanticBuilderReturn { semantic, errors: self.errors.into_inner() }
    }

    /// Push a Syntax Error
    pub(crate) fn error(&self, error: OxcDiagnostic) {
        self.errors.borrow_mut().push(error);
    }

    pub(crate) fn in_declare_scope(&self) -> bool {
        self.source_type.is_typescript_definition()
            || self
                .scope
                .ancestors(self.current_scope_id)
                .any(|scope_id| self.scope.get_flags(scope_id).is_ts_module_block())
    }

    fn create_ast_node(&mut self, kind: AstKind<'a>) {
        let mut flags = self.current_node_flags;
        if self.build_jsdoc && self.jsdoc.retrieve_attached_jsdoc(&kind) {
            flags |= NodeFlags::JSDoc;
        }

        self.current_node_id = self.nodes.add_node(
            kind,
            self.current_scope_id,
            self.current_node_id,
            control_flow!(self, |cfg| cfg.current_node_ix),
            flags,
        );
        self.record_ast_node();
    }

    fn pop_ast_node(&mut self) {
        if let Some(parent_id) = self.nodes.parent_id(self.current_node_id) {
            self.current_node_id = parent_id;
        }
    }

    #[inline]
    fn record_ast_nodes(&mut self) {
        if self.cfg.is_some() {
            self.ast_node_records.push(NodeId::DUMMY);
        }
    }

    #[inline]
    #[allow(clippy::unnecessary_wraps)]
    fn retrieve_recorded_ast_node(&mut self) -> Option<NodeId> {
        if self.cfg.is_some() {
            Some(self.ast_node_records.pop().expect("there is no ast node record to stop."))
        } else {
            None
        }
    }

    #[inline]
    fn record_ast_node(&mut self) {
        // The `self.cfg.is_some()` check here could be removed, since `ast_node_records` is empty
        // if CFG is disabled. But benchmarks showed removing the extra check is a perf regression.
        // <https://github.com/oxc-project/oxc/pull/4273>
        if self.cfg.is_some() {
            if let Some(record) = self.ast_node_records.last_mut() {
                if *record == NodeId::DUMMY {
                    *record = self.current_node_id;
                }
            }
        }
    }

    #[inline]
    pub(crate) fn current_scope_flags(&self) -> ScopeFlags {
        self.scope.get_flags(self.current_scope_id)
    }

    /// Is the current scope in strict mode?
    pub(crate) fn strict_mode(&self) -> bool {
        self.current_scope_flags().is_strict_mode()
    }

    pub(crate) fn set_function_node_flags(&mut self, flags: NodeFlags) {
        if let Some(current_function) = self.function_stack.last() {
            *self.nodes.get_node_mut(*current_function).flags_mut() |= flags;
        }
    }

    /// Declares a `Symbol` for the node, adds it to symbol table, and binds it to the scope.
    ///
    /// includes: the `SymbolFlags` that node has in addition to its declaration type (eg: export, ambient, etc.)
    /// excludes: the flags which node cannot be declared alongside in a symbol table. Used to report forbidden declarations.
    ///
    /// Reports errors for conflicting identifier names.
    pub(crate) fn declare_symbol_on_scope(
        &mut self,
        span: Span,
        name: &str,
        scope_id: ScopeId,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        if let Some(symbol_id) = self.check_redeclaration(scope_id, span, name, excludes, true) {
            self.symbols.union_flag(symbol_id, includes);
            self.add_redeclare_variable(symbol_id, span);
            return symbol_id;
        }

        let symbol_id =
            self.symbols.create_symbol(span, name, includes, scope_id, self.current_node_id);

        self.scope.add_binding(scope_id, name, symbol_id);
        symbol_id
    }

    /// Declare a new symbol on the current scope.
    pub(crate) fn declare_symbol(
        &mut self,
        span: Span,
        name: &str,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        self.declare_symbol_on_scope(span, name, self.current_scope_id, includes, excludes)
    }

    /// Check if a symbol with the same name has already been declared in the
    /// current scope. Returns the symbol ID if it exists and is not excluded by `excludes`.
    ///
    /// Only records a redeclaration error if `report_error` is `true`.
    pub(crate) fn check_redeclaration(
        &self,
        scope_id: ScopeId,
        span: Span,
        name: &str,
        excludes: SymbolFlags,
        report_error: bool,
    ) -> Option<SymbolId> {
        let symbol_id = self.scope.get_binding(scope_id, name).or_else(|| {
            self.hoisting_variables.get(&scope_id).and_then(|symbols| symbols.get(name).copied())
        })?;
        if report_error && self.symbols.get_flags(symbol_id).intersects(excludes) {
            let symbol_span = self.symbols.get_span(symbol_id);
            self.error(redeclaration(name, symbol_span, span));
        }
        Some(symbol_id)
    }

    /// Declare an unresolved reference in the current scope.
    ///
    /// # Panics
    pub(crate) fn declare_reference(
        &mut self,
        name: Atom<'a>,
        reference: Reference,
    ) -> ReferenceId {
        let reference_id = self.symbols.create_reference(reference);

        self.unresolved_references.current_mut().entry(name).or_default().push(reference_id);
        reference_id
    }

    /// Declares a `Symbol` for the node, shadowing previous declarations in the same scope.
    pub(crate) fn declare_shadow_symbol(
        &mut self,
        name: &str,
        span: Span,
        scope_id: ScopeId,
        includes: SymbolFlags,
    ) -> SymbolId {
        let symbol_id = self.symbols.create_symbol(
            span,
            name,
            includes,
            self.current_scope_id,
            self.current_node_id,
        );
        self.scope.insert_binding(scope_id, name, symbol_id);
        symbol_id
    }

    /// Try to resolve all references from the current scope that are not
    /// already resolved.
    ///
    /// This gets called every time [`SemanticBuilder`] exits a scope.
    fn resolve_references_for_current_scope(&mut self) {
        let (current_refs, parent_refs) = self.unresolved_references.current_and_parent_mut();

        for (name, mut references) in current_refs.drain() {
            // Try to resolve a reference.
            // If unresolved, transfer it to parent scope's unresolved references.
            let bindings = self.scope.get_bindings(self.current_scope_id);
            if let Some(symbol_id) = bindings.get(name.as_str()).copied() {
                let symbol_flags = self.symbols.get_flags(symbol_id);
                references.retain(|&reference_id| {
                    let reference = &mut self.symbols.references[reference_id];

                    let flags = reference.flags_mut();

                    // Determine the symbol whether can be referenced by this reference.
                    let resolved = (flags.is_value() && symbol_flags.can_be_referenced_by_value())
                        || (flags.is_type() && symbol_flags.can_be_referenced_by_type())
                        || (flags.is_value_as_type()
                            && symbol_flags.can_be_referenced_by_value_as_type());

                    if !resolved {
                        return true;
                    }

                    if symbol_flags.is_value() && flags.is_value() {
                        // The non type-only ExportSpecifier can reference both type/value symbols,
                        // if the symbol is a value symbol and reference flag is not type-only,
                        // remove the type flag. For example: `const B = 1; export { B };`
                        *flags -= ReferenceFlags::Type;
                    } else {
                        // 1. ReferenceFlags::ValueAsType -> ReferenceFlags::Type
                        // `const ident = 0; typeof ident`
                        //                          ^^^^^ -> The ident is a value symbols,
                        //                                   but it used as a type.
                        // 2. ReferenceFlags::Value | ReferenceFlags::Type -> ReferenceFlags::Type
                        // `type ident = string; export default ident;
                        //                                      ^^^^^ We have confirmed the symbol is
                        //                                            not a value symbol, so we need to
                        //                                            make sure the reference is a type only.
                        *flags = ReferenceFlags::Type;
                    }
                    reference.set_symbol_id(symbol_id);
                    self.symbols.add_resolved_reference(symbol_id, reference_id);

                    false
                });

                if references.is_empty() {
                    continue;
                }
            }

            if let Some(parent_reference_ids) = parent_refs.get_mut(&name) {
                parent_reference_ids.extend(references);
            } else {
                parent_refs.insert(name, references);
            }
        }
    }

    pub(crate) fn add_redeclare_variable(&mut self, symbol_id: SymbolId, span: Span) {
        self.symbols.add_redeclaration(symbol_id, span);
    }
}

impl<'a> Visit<'a> for SemanticBuilder<'a> {
    // NB: Not called for `Program`
    fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        let parent_scope_id = self.current_scope_id;
        let flags = self.scope.get_new_scope_flags(flags, parent_scope_id);
        self.current_scope_id =
            self.scope.add_scope(Some(parent_scope_id), self.current_node_id, flags);
        scope_id.set(Some(self.current_scope_id));

        self.unresolved_references.increment_scope_depth();
    }

    // NB: Not called for `Program`
    fn leave_scope(&mut self) {
        self.resolve_references_for_current_scope();

        // `get_parent_id` always returns `Some` because this method is not called for `Program`.
        // So we could `.unwrap()` here. But that seems to produce a small perf impact, probably because
        // `leave_scope` then doesn't get inlined because of its larger size due to the panic code.
        let parent_id = self.scope.get_parent_id(self.current_scope_id);
        debug_assert!(parent_id.is_some());
        if let Some(parent_id) = parent_id {
            self.current_scope_id = parent_id;
        }

        self.unresolved_references.decrement_scope_depth();
    }

    // Setup all the context for the binder.
    // The order is important here.
    // NB: Not called for `Program`.
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.create_ast_node(kind);
        self.enter_kind(kind);
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        if self.check_syntax_error {
            let node = self.nodes.get_node(self.current_node_id);
            checker::check(node, self);
        }
        self.leave_kind(kind);
        self.pop_ast_node();
    }

    fn visit_program(&mut self, program: &Program<'a>) {
        let kind = AstKind::Program(self.alloc(program));
        /* cfg */
        let error_harness = control_flow!(self, |cfg| {
            let error_harness = cfg.attach_error_harness(ErrorEdgeKind::Implicit);
            let _program_basic_block = cfg.new_basic_block_normal();
            error_harness
        });
        /* cfg - must be above directives as directives are in cfg */

        // Don't call `enter_node` here as `Program` is a special case - node has no `parent_id`.
        // Inline the specific logic for `Program` here instead.
        // This avoids `Nodes::add_node` having to handle the special case.
        // We can also skip calling `self.enter_kind`, `self.record_ast_node`
        // and `self.jsdoc.retrieve_attached_jsdoc`, as they are all no-ops for `Program`.
        self.current_node_id = self.nodes.add_program_node(
            kind,
            self.current_scope_id,
            control_flow!(self, |cfg| cfg.current_node_ix),
            self.current_node_flags,
        );

        // Don't call `enter_scope` here as `Program` is a special case - scope has no `parent_id`.
        // Inline the specific logic for `Program` here instead.
        // This simplifies logic in `enter_scope`, as it doesn't have to handle the special case.
        let mut flags = ScopeFlags::Top;
        if self.source_type.is_strict() || program.has_use_strict_directive() {
            flags |= ScopeFlags::StrictMode;
        }
        self.current_scope_id = self.scope.add_scope(None, self.current_node_id, flags);
        program.scope_id.set(Some(self.current_scope_id));
        // NB: Don't call `self.unresolved_references.increment_scope_depth()`
        // as scope depth is initialized as 1 already (the scope depth for `Program`).

        if let Some(hashbang) = &program.hashbang {
            self.visit_hashbang(hashbang);
        }

        for directive in &program.directives {
            self.visit_directive(directive);
        }

        self.visit_statements(&program.body);

        /* cfg */
        control_flow!(self, |cfg| cfg.release_error_harness(error_harness));
        /* cfg */

        // Don't call `leave_scope` here as `Program` is a special case - scope has no `parent_id`.
        // This simplifies `leave_scope`.
        self.resolve_references_for_current_scope();
        // NB: Don't call `self.unresolved_references.decrement_scope_depth()`
        // as scope depth must remain >= 1.

        self.leave_node(kind);
    }

    fn visit_break_statement(&mut self, stmt: &BreakStatement<'a>) {
        let kind = AstKind::BreakStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let node_id = self.current_node_id;
        /* cfg */

        if let Some(break_target) = &stmt.label {
            self.visit_label_identifier(break_target);
        }

        /* cfg */
        control_flow!(self, |cfg| cfg
            .append_break(node_id, stmt.label.as_ref().map(|it| it.name.as_str())));
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        let kind = AstKind::Class(self.alloc(class));
        self.enter_node(kind);

        self.visit_decorators(&class.decorators);
        if let Some(id) = &class.id {
            self.visit_binding_identifier(id);
        }

        self.enter_scope(ScopeFlags::StrictMode, &class.scope_id);
        if class.is_expression() {
            // We need to bind class expression in the class scope
            class.bind(self);
        }

        if let Some(type_parameters) = &class.type_parameters {
            self.visit_ts_type_parameter_declaration(type_parameters);
        }
        if let Some(super_class) = &class.super_class {
            self.visit_expression(super_class);
        }
        if let Some(super_type_parameters) = &class.super_type_parameters {
            self.visit_ts_type_parameter_instantiation(super_type_parameters);
        }
        if let Some(implements) = &class.implements {
            self.visit_ts_class_implementses(implements);
        }
        self.visit_class_body(&class.body);

        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_block_statement(&mut self, it: &BlockStatement<'a>) {
        let kind = AstKind::BlockStatement(self.alloc(it));
        self.enter_node(kind);

        let parent_scope_id = self.current_scope_id;
        self.enter_scope(ScopeFlags::empty(), &it.scope_id);

        // Move all bindings from catch clause param scope to catch clause body scope
        // to make it easier to resolve references and check redeclare errors
        if self.scope.get_flags(parent_scope_id).is_catch_clause() {
            self.scope.cell.with_dependent_mut(|allocator, inner| {
                if !inner.bindings[parent_scope_id].is_empty() {
                    let mut parent_bindings = Bindings::new_in(allocator);
                    mem::swap(&mut inner.bindings[parent_scope_id], &mut parent_bindings);
                    for &symbol_id in parent_bindings.values() {
                        self.symbols.set_scope_id(symbol_id, self.current_scope_id);
                    }
                    inner.bindings[self.current_scope_id] = parent_bindings;
                }
            });
        }

        self.visit_statements(&it.body);

        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_continue_statement(&mut self, stmt: &ContinueStatement<'a>) {
        let kind = AstKind::ContinueStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let node_id = self.current_node_id;
        /* cfg */

        if let Some(continue_target) = &stmt.label {
            self.visit_label_identifier(continue_target);
        }

        /* cfg */
        control_flow!(self, |cfg| cfg
            .append_continue(node_id, stmt.label.as_ref().map(|it| it.name.as_str())));
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_do_while_statement(&mut self, stmt: &DoWhileStatement<'a>) {
        let kind = AstKind::DoWhileStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let (before_do_while_stmt_graph_ix, start_body_graph_ix) = control_flow!(self, |cfg| {
            let before_do_while_stmt_graph_ix = cfg.current_node_ix;
            let start_body_graph_ix = cfg.new_basic_block_normal();
            cfg.ctx(None).default().allow_break().allow_continue();
            (before_do_while_stmt_graph_ix, start_body_graph_ix)
        });
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg - condition basic block */
        let (after_body_graph_ix, start_of_condition_graph_ix) = control_flow!(self, |cfg| {
            let after_body_graph_ix = cfg.current_node_ix;
            let start_of_condition_graph_ix = cfg.new_basic_block_normal();
            (after_body_graph_ix, start_of_condition_graph_ix)
        });
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&stmt.test);
        let test_node_id = self.retrieve_recorded_ast_node();

        /* cfg */
        control_flow!(self, |cfg| {
            cfg.append_condition_to(start_of_condition_graph_ix, test_node_id);
            let end_of_condition_graph_ix = cfg.current_node_ix;

            let end_do_while_graph_ix = cfg.new_basic_block_normal();

            // before do while to start of body basic block
            cfg.add_edge(before_do_while_stmt_graph_ix, start_body_graph_ix, EdgeType::Normal);
            // body of do-while to start of condition
            cfg.add_edge(after_body_graph_ix, start_of_condition_graph_ix, EdgeType::Normal);
            // end of condition to after do while
            cfg.add_edge(end_of_condition_graph_ix, end_do_while_graph_ix, EdgeType::Normal);
            // end of condition to after start of body
            cfg.add_edge(end_of_condition_graph_ix, start_body_graph_ix, EdgeType::Backedge);

            cfg.ctx(None)
                .mark_break(end_do_while_graph_ix)
                .mark_continue(start_of_condition_graph_ix)
                .resolve_with_upper_label();
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_logical_expression(&mut self, expr: &LogicalExpression<'a>) {
        // logical expressions are short-circuiting, and therefore
        // also represent control flow.
        // For example, in:
        //   foo && bar();
        // the bar() call will only be executed if foo is truthy.
        let kind = AstKind::LogicalExpression(self.alloc(expr));
        self.enter_node(kind);

        self.visit_expression(&expr.left);

        /* cfg  */
        let (left_expr_end_ix, right_expr_start_ix) = control_flow!(self, |cfg| {
            let left_expr_end_ix = cfg.current_node_ix;
            let right_expr_start_ix = cfg.new_basic_block_normal();
            (left_expr_end_ix, right_expr_start_ix)
        });
        /* cfg  */

        self.visit_expression(&expr.right);

        /* cfg */
        control_flow!(self, |cfg| {
            let right_expr_end_ix = cfg.current_node_ix;
            let after_logical_expr_ix = cfg.new_basic_block_normal();

            cfg.add_edge(left_expr_end_ix, right_expr_start_ix, EdgeType::Normal);
            cfg.add_edge(left_expr_end_ix, after_logical_expr_ix, EdgeType::Normal);
            cfg.add_edge(right_expr_end_ix, after_logical_expr_ix, EdgeType::Normal);
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        // assignment expressions can include an operator, which
        // can be used to determine the control flow of the expression.
        // For example, in:
        //   foo &&= super();
        // the super() call will only be executed if foo is truthy.

        let kind = AstKind::AssignmentExpression(self.alloc(expr));
        self.enter_node(kind);

        if !expr.operator.is_assign() {
            // Only when the operator is not an `=` operator, the left-hand side is both read and write.
            // <https://tc39.es/ecma262/#sec-assignment-operators-runtime-semantics-evaluation>
            self.current_reference_flags = ReferenceFlags::read_write();
        }

        self.visit_assignment_target(&expr.left);

        /* cfg  */
        let cfg_ixs = control_flow!(self, |cfg| {
            if expr.operator.is_logical() {
                let target_end_ix = cfg.current_node_ix;
                let expr_start_ix = cfg.new_basic_block_normal();
                Some((target_end_ix, expr_start_ix))
            } else {
                None
            }
        });
        /* cfg  */

        self.visit_expression(&expr.right);

        /* cfg */
        control_flow!(self, |cfg| {
            if let Some((target_end_ix, expr_start_ix)) = cfg_ixs {
                let expr_end_ix = cfg.current_node_ix;
                let after_assignment_ix = cfg.new_basic_block_normal();

                cfg.add_edge(target_end_ix, expr_start_ix, EdgeType::Normal);
                cfg.add_edge(target_end_ix, after_assignment_ix, EdgeType::Normal);
                cfg.add_edge(expr_end_ix, after_assignment_ix, EdgeType::Normal);
            }
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_conditional_expression(&mut self, expr: &ConditionalExpression<'a>) {
        let kind = AstKind::ConditionalExpression(self.alloc(expr));
        self.enter_node(kind);

        /* cfg - condition basic block */
        let (before_conditional_graph_ix, start_of_condition_graph_ix) =
            control_flow!(self, |cfg| {
                let before_conditional_graph_ix = cfg.current_node_ix;
                let start_of_condition_graph_ix = cfg.new_basic_block_normal();
                (before_conditional_graph_ix, start_of_condition_graph_ix)
            });
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&expr.test);
        let test_node_id = self.retrieve_recorded_ast_node();

        /* cfg */
        let (after_condition_graph_ix, before_consequent_expr_graph_ix) =
            control_flow!(self, |cfg| {
                cfg.append_condition_to(start_of_condition_graph_ix, test_node_id);
                let after_condition_graph_ix = cfg.current_node_ix;
                // conditional expression basic block
                let before_consequent_expr_graph_ix = cfg.new_basic_block_normal();
                (after_condition_graph_ix, before_consequent_expr_graph_ix)
            });
        /* cfg */

        self.visit_expression(&expr.consequent);

        /* cfg */
        let (after_consequent_expr_graph_ix, start_alternate_graph_ix) =
            control_flow!(self, |cfg| {
                let after_consequent_expr_graph_ix = cfg.current_node_ix;
                let start_alternate_graph_ix = cfg.new_basic_block_normal();
                (after_consequent_expr_graph_ix, start_alternate_graph_ix)
            });
        /* cfg */

        self.visit_expression(&expr.alternate);

        /* cfg */
        control_flow!(self, |cfg| {
            let after_alternate_graph_ix = cfg.current_node_ix;
            /* bb after conditional expression joins consequent and alternate */
            let after_conditional_graph_ix = cfg.new_basic_block_normal();

            cfg.add_edge(
                before_conditional_graph_ix,
                start_of_condition_graph_ix,
                EdgeType::Normal,
            );

            cfg.add_edge(
                after_consequent_expr_graph_ix,
                after_conditional_graph_ix,
                EdgeType::Normal,
            );
            cfg.add_edge(after_condition_graph_ix, before_consequent_expr_graph_ix, EdgeType::Jump);

            cfg.add_edge(after_condition_graph_ix, start_alternate_graph_ix, EdgeType::Normal);
            cfg.add_edge(after_alternate_graph_ix, after_conditional_graph_ix, EdgeType::Normal);
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_for_statement(&mut self, stmt: &ForStatement<'a>) {
        let kind = AstKind::ForStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.enter_scope(ScopeFlags::empty(), &stmt.scope_id);
        if let Some(init) = &stmt.init {
            self.visit_for_statement_init(init);
        }
        /* cfg */
        let (before_for_graph_ix, test_graph_ix) = control_flow!(self, |cfg| {
            let before_for_graph_ix = cfg.current_node_ix;
            let test_graph_ix = cfg.new_basic_block_normal();
            (before_for_graph_ix, test_graph_ix)
        });
        /* cfg */

        if let Some(test) = &stmt.test {
            self.record_ast_nodes();
            self.visit_expression(test);
            let test_node_id = self.retrieve_recorded_ast_node();

            /* cfg */
            control_flow!(self, |cfg| cfg.append_condition_to(test_graph_ix, test_node_id));
            /* cfg */
        }

        /* cfg */
        let (after_test_graph_ix, update_graph_ix) =
            control_flow!(self, |cfg| (cfg.current_node_ix, cfg.new_basic_block_normal()));
        /* cfg */

        if let Some(update) = &stmt.update {
            self.visit_expression(update);
        }

        /* cfg */
        let before_body_graph_ix = control_flow!(self, |cfg| {
            let before_body_graph_ix = cfg.new_basic_block_normal();
            cfg.ctx(None).default().allow_break().allow_continue();
            before_body_graph_ix
        });
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg */
        control_flow!(self, |cfg| {
            let after_body_graph_ix = cfg.current_node_ix;
            let after_for_stmt = cfg.new_basic_block_normal();
            cfg.add_edge(before_for_graph_ix, test_graph_ix, EdgeType::Normal);
            cfg.add_edge(after_test_graph_ix, before_body_graph_ix, EdgeType::Jump);
            cfg.add_edge(after_body_graph_ix, update_graph_ix, EdgeType::Backedge);
            cfg.add_edge(update_graph_ix, test_graph_ix, EdgeType::Backedge);
            cfg.add_edge(after_test_graph_ix, after_for_stmt, EdgeType::Normal);

            cfg.ctx(None)
                .mark_break(after_for_stmt)
                .mark_continue(update_graph_ix)
                .resolve_with_upper_label();
        });
        /* cfg */

        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_for_in_statement(&mut self, stmt: &ForInStatement<'a>) {
        let kind = AstKind::ForInStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.enter_scope(ScopeFlags::empty(), &stmt.scope_id);

        self.visit_for_statement_left(&stmt.left);

        /* cfg */
        let (before_for_stmt_graph_ix, start_prepare_cond_graph_ix) =
            control_flow!(self, |cfg| (cfg.current_node_ix, cfg.new_basic_block_normal(),));
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&stmt.right);
        let right_node_id = self.retrieve_recorded_ast_node();

        /* cfg */
        let (end_of_prepare_cond_graph_ix, iteration_graph_ix, body_graph_ix) =
            control_flow!(self, |cfg| {
                let end_of_prepare_cond_graph_ix = cfg.current_node_ix;
                let iteration_graph_ix = cfg.new_basic_block_normal();
                cfg.append_iteration(right_node_id, IterationInstructionKind::In);
                let body_graph_ix = cfg.new_basic_block_normal();

                cfg.ctx(None).default().allow_break().allow_continue();
                (end_of_prepare_cond_graph_ix, iteration_graph_ix, body_graph_ix)
            });
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg */
        control_flow!(self, |cfg| {
            let end_of_body_graph_ix = cfg.current_node_ix;
            let after_for_graph_ix = cfg.new_basic_block_normal();
            // connect before for statement to the iterable expression
            cfg.add_edge(before_for_stmt_graph_ix, start_prepare_cond_graph_ix, EdgeType::Normal);
            // connect the end of the iterable expression to the basic block with back edge
            cfg.add_edge(end_of_prepare_cond_graph_ix, iteration_graph_ix, EdgeType::Normal);
            // connect the basic block with back edge to the start of the body
            cfg.add_edge(iteration_graph_ix, body_graph_ix, EdgeType::Jump);
            // connect the end of the body back to the basic block
            // with back edge for the next iteration
            cfg.add_edge(end_of_body_graph_ix, iteration_graph_ix, EdgeType::Backedge);
            // connect the basic block with back edge to the basic block after the for loop
            // for when there are no more iterations left in the iterable
            cfg.add_edge(iteration_graph_ix, after_for_graph_ix, EdgeType::Normal);

            cfg.ctx(None)
                .mark_break(after_for_graph_ix)
                .mark_continue(iteration_graph_ix)
                .resolve_with_upper_label();
        });
        /* cfg */

        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        let kind = AstKind::ForOfStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.enter_scope(ScopeFlags::empty(), &stmt.scope_id);

        self.visit_for_statement_left(&stmt.left);

        /* cfg */
        let (before_for_stmt_graph_ix, start_prepare_cond_graph_ix) =
            control_flow!(self, |cfg| (cfg.current_node_ix, cfg.new_basic_block_normal()));
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&stmt.right);
        let right_node_id = self.retrieve_recorded_ast_node();

        /* cfg */
        let (end_of_prepare_cond_graph_ix, iteration_graph_ix, body_graph_ix) =
            control_flow!(self, |cfg| {
                let end_of_prepare_cond_graph_ix = cfg.current_node_ix;
                let iteration_graph_ix = cfg.new_basic_block_normal();
                cfg.append_iteration(right_node_id, IterationInstructionKind::Of);
                let body_graph_ix = cfg.new_basic_block_normal();
                cfg.ctx(None).default().allow_break().allow_continue();
                (end_of_prepare_cond_graph_ix, iteration_graph_ix, body_graph_ix)
            });
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg */
        control_flow!(self, |cfg| {
            let end_of_body_graph_ix = cfg.current_node_ix;
            let after_for_graph_ix = cfg.new_basic_block_normal();
            // connect before for statement to the iterable expression
            cfg.add_edge(before_for_stmt_graph_ix, start_prepare_cond_graph_ix, EdgeType::Normal);
            // connect the end of the iterable expression to the basic block with back edge
            cfg.add_edge(end_of_prepare_cond_graph_ix, iteration_graph_ix, EdgeType::Normal);
            // connect the basic block with back edge to the start of the body
            cfg.add_edge(iteration_graph_ix, body_graph_ix, EdgeType::Jump);
            // connect the end of the body back to the basic block
            // with back edge for the next iteration
            cfg.add_edge(end_of_body_graph_ix, iteration_graph_ix, EdgeType::Backedge);
            // connect the basic block with back edge to the basic block after the for loop
            // for when there are no more iterations left in the iterable
            cfg.add_edge(iteration_graph_ix, after_for_graph_ix, EdgeType::Normal);

            cfg.ctx(None)
                .mark_break(after_for_graph_ix)
                .mark_continue(iteration_graph_ix)
                .resolve_with_upper_label();
        });
        /* cfg */

        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_if_statement(&mut self, stmt: &IfStatement<'a>) {
        let kind = AstKind::IfStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg - condition basic block */
        let (before_if_stmt_graph_ix, start_of_condition_graph_ix) =
            control_flow!(self, |cfg| (cfg.current_node_ix, cfg.new_basic_block_normal(),));
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&stmt.test);
        let test_node_id = self.retrieve_recorded_ast_node();

        /* cfg */
        let (after_test_graph_ix, before_consequent_stmt_graph_ix) = control_flow!(self, |cfg| {
            cfg.append_condition_to(start_of_condition_graph_ix, test_node_id);
            (cfg.current_node_ix, cfg.new_basic_block_normal())
        });
        /* cfg */

        self.visit_statement(&stmt.consequent);

        /* cfg */
        let after_consequent_stmt_graph_ix = control_flow!(self, |cfg| cfg.current_node_ix);
        /* cfg */

        let else_graph_ix = if let Some(alternate) = &stmt.alternate {
            /* cfg */
            let else_graph_ix = control_flow!(self, |cfg| cfg.new_basic_block_normal());
            /* cfg */

            self.visit_statement(alternate);

            control_flow!(self, |cfg| Some((else_graph_ix, cfg.current_node_ix)))
        } else {
            None
        };

        /* cfg - bb after if statement joins consequent and alternate */
        control_flow!(self, |cfg| {
            let after_if_graph_ix = cfg.new_basic_block_normal();

            cfg.add_edge(before_if_stmt_graph_ix, start_of_condition_graph_ix, EdgeType::Normal);

            cfg.add_edge(after_consequent_stmt_graph_ix, after_if_graph_ix, EdgeType::Normal);

            cfg.add_edge(after_test_graph_ix, before_consequent_stmt_graph_ix, EdgeType::Jump);

            if let Some((start_of_alternate_stmt_graph_ix, after_alternate_stmt_graph_ix)) =
                else_graph_ix
            {
                cfg.add_edge(
                    before_if_stmt_graph_ix,
                    start_of_alternate_stmt_graph_ix,
                    EdgeType::Normal,
                );
                cfg.add_edge(after_alternate_stmt_graph_ix, after_if_graph_ix, EdgeType::Normal);
            } else {
                cfg.add_edge(before_if_stmt_graph_ix, after_if_graph_ix, EdgeType::Normal);
            }
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_labeled_statement(&mut self, stmt: &LabeledStatement<'a>) {
        let kind = AstKind::LabeledStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let label = &stmt.label.name;
        control_flow!(self, |cfg| {
            let ctx = cfg.ctx(Some(label.as_str())).default().allow_break();
            if stmt.body.is_iteration_statement() {
                ctx.allow_continue();
            }
        });
        /* cfg */

        self.visit_label_identifier(&stmt.label);

        self.visit_statement(&stmt.body);

        /* cfg */
        control_flow!(self, |cfg| {
            let after_body_graph_ix = cfg.current_node_ix;
            let after_labeled_stmt_graph_ix = cfg.new_basic_block_normal();
            cfg.add_edge(after_body_graph_ix, after_labeled_stmt_graph_ix, EdgeType::Normal);

            cfg.ctx(Some(label.as_str())).mark_break(after_labeled_stmt_graph_ix).resolve();
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        let kind = AstKind::ReturnStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let node_id = self.current_node_id;
        /* cfg */

        let ret_kind = if let Some(arg) = &stmt.argument {
            self.visit_expression(arg);
            ReturnInstructionKind::NotImplicitUndefined
        } else {
            ReturnInstructionKind::ImplicitUndefined
        };

        /* cfg */
        control_flow!(self, |cfg| {
            cfg.push_return(ret_kind, Some(node_id));
            cfg.append_unreachable();
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_switch_statement(&mut self, stmt: &SwitchStatement<'a>) {
        let kind = AstKind::SwitchStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&stmt.discriminant);
        self.enter_scope(ScopeFlags::empty(), &stmt.scope_id);

        /* cfg */
        let discriminant_graph_ix = control_flow!(self, |cfg| {
            let discriminant_graph_ix = cfg.current_node_ix;
            cfg.ctx(None).default().allow_break();
            discriminant_graph_ix
        });
        let mut switch_case_graph_spans = vec![];
        let mut have_default_case = false;
        /* cfg */

        for case in &stmt.cases {
            let before_case_graph_ix = control_flow!(self, |cfg| cfg.new_basic_block_normal());
            self.visit_switch_case(case);
            if case.is_default_case() {
                have_default_case = true;
            }
            control_flow!(self, |cfg| switch_case_graph_spans
                .push((before_case_graph_ix, cfg.current_node_ix)));
        }

        /* cfg */
        // for each switch case
        control_flow!(self, |cfg| {
            for i in 0..switch_case_graph_spans.len() {
                let case_graph_span = switch_case_graph_spans[i];

                // every switch case condition can be skipped,
                // so there's a possible jump from it to the next switch case condition
                for y in switch_case_graph_spans.iter().skip(i + 1) {
                    cfg.add_edge(case_graph_span.0, y.0, EdgeType::Normal);
                }

                // connect the end of each switch statement to
                // the condition of the next switch statement
                if switch_case_graph_spans.len() > i + 1 {
                    let (_, end_of_switch_case) = switch_case_graph_spans[i];
                    let (next_switch_statement_condition, _) = switch_case_graph_spans[i + 1];

                    cfg.add_edge(
                        end_of_switch_case,
                        next_switch_statement_condition,
                        EdgeType::Normal,
                    );
                }

                cfg.add_edge(discriminant_graph_ix, case_graph_span.0, EdgeType::Normal);
            }

            let end_of_switch_case_statement = cfg.new_basic_block_normal();

            if let Some(last) = switch_case_graph_spans.last() {
                cfg.add_edge(last.1, end_of_switch_case_statement, EdgeType::Normal);
            }

            // if we don't have a default case there should be an edge from discriminant to the end of
            // the statement.
            if !have_default_case {
                cfg.add_edge(discriminant_graph_ix, end_of_switch_case_statement, EdgeType::Normal);
            }

            cfg.ctx(None).mark_break(end_of_switch_case_statement).resolve();
        });
        /* cfg */

        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_switch_case(&mut self, case: &SwitchCase<'a>) {
        let kind = AstKind::SwitchCase(self.alloc(case));
        self.enter_node(kind);

        if let Some(expr) = &case.test {
            self.record_ast_nodes();
            self.visit_expression(expr);
            let test_node_id = self.retrieve_recorded_ast_node();
            control_flow!(self, |cfg| cfg.append_condition_to(cfg.current_node_ix, test_node_id));
        }

        /* cfg */
        control_flow!(self, |cfg| {
            let after_test_graph_ix = cfg.current_node_ix;
            let statements_in_switch_graph_ix = cfg.new_basic_block_normal();
            cfg.add_edge(after_test_graph_ix, statements_in_switch_graph_ix, EdgeType::Jump);
        });
        /* cfg */

        self.visit_statements(&case.consequent);

        self.leave_node(kind);
    }

    fn visit_throw_statement(&mut self, stmt: &ThrowStatement<'a>) {
        let kind = AstKind::ThrowStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let node_id = self.current_node_id;
        /* cfg */

        self.visit_expression(&stmt.argument);

        /* cfg */
        control_flow!(self, |cfg| cfg.append_throw(node_id));
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_try_statement(&mut self, stmt: &TryStatement<'a>) {
        let kind = AstKind::TryStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */

        let (
            before_try_statement_graph_ix,
            error_harness,
            before_finalizer_graph_ix,
            before_try_block_graph_ix,
        ) = control_flow!(self, |cfg| {
            let before_try_statement_graph_ix = cfg.current_node_ix;
            let error_harness =
                stmt.handler.as_ref().map(|_| cfg.attach_error_harness(ErrorEdgeKind::Explicit));
            let before_finalizer_graph_ix = stmt.finalizer.as_ref().map(|_| cfg.attach_finalizer());
            let before_try_block_graph_ix = cfg.new_basic_block_normal();

            (
                before_try_statement_graph_ix,
                error_harness,
                before_finalizer_graph_ix,
                before_try_block_graph_ix,
            )
        });
        /* cfg */

        self.visit_block_statement(&stmt.block);

        /* cfg */
        let after_try_block_graph_ix = control_flow!(self, |cfg| cfg.current_node_ix);
        /* cfg */

        let catch_block_end_ix = if let Some(handler) = &stmt.handler {
            /* cfg */
            control_flow!(self, |cfg| {
                let Some(error_harness) = error_harness else {
                    unreachable!("we always create an error harness if we have a catch block.");
                };
                cfg.release_error_harness(error_harness);
                let catch_block_start_ix = cfg.new_basic_block_normal();
                cfg.add_edge(error_harness, catch_block_start_ix, EdgeType::Normal);
            });
            /* cfg */

            self.visit_catch_clause(handler);

            /* cfg */
            control_flow!(self, |cfg| {
                let catch_block_end_ix = cfg.current_node_ix;
                // TODO: we shouldn't directly change the current node index.
                cfg.current_node_ix = after_try_block_graph_ix;
                Some(catch_block_end_ix)
            })
            /* cfg */
        } else {
            None
        };

        let finally_block_end_ix = if let Some(finalizer) = &stmt.finalizer {
            /* cfg */
            control_flow!(self, |cfg| {
                let Some(before_finalizer_graph_ix) = before_finalizer_graph_ix else {
                    unreachable!("we always create a finalizer when there is a finally block.");
                };
                cfg.release_finalizer(before_finalizer_graph_ix);
                let start_finally_graph_ix = cfg.new_basic_block_normal();
                cfg.add_edge(before_finalizer_graph_ix, start_finally_graph_ix, EdgeType::Normal);
            });
            /* cfg */

            self.visit_block_statement(finalizer);

            /* cfg */
            control_flow!(self, |cfg| {
                let finally_block_end_ix = cfg.current_node_ix;
                // TODO: we shouldn't directly change the current node index.
                cfg.current_node_ix = after_try_block_graph_ix;
                Some(finally_block_end_ix)
            })
            /* cfg */
        } else {
            None
        };

        /* cfg */
        control_flow!(self, |cfg| {
            let after_try_statement_block_ix = cfg.new_basic_block_normal();
            cfg.add_edge(
                before_try_statement_graph_ix,
                before_try_block_graph_ix,
                EdgeType::Normal,
            );
            if let Some(catch_block_end_ix) = catch_block_end_ix {
                if finally_block_end_ix.is_none() {
                    cfg.add_edge(
                        after_try_block_graph_ix,
                        after_try_statement_block_ix,
                        EdgeType::Normal,
                    );

                    cfg.add_edge(
                        catch_block_end_ix,
                        after_try_statement_block_ix,
                        EdgeType::Normal,
                    );
                }
            }
            if let Some(finally_block_end_ix) = finally_block_end_ix {
                if catch_block_end_ix.is_some() {
                    cfg.add_edge(
                        finally_block_end_ix,
                        after_try_statement_block_ix,
                        EdgeType::Normal,
                    );
                } else {
                    cfg.add_edge(
                        finally_block_end_ix,
                        after_try_statement_block_ix,
                        if cfg.basic_block(after_try_block_graph_ix).is_unreachable() {
                            EdgeType::Unreachable
                        } else {
                            EdgeType::Join
                        },
                    );
                }
            }
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_while_statement(&mut self, stmt: &WhileStatement<'a>) {
        let kind = AstKind::WhileStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg - condition basic block */
        let (before_while_stmt_graph_ix, condition_graph_ix) =
            control_flow!(self, |cfg| (cfg.current_node_ix, cfg.new_basic_block_normal()));
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&stmt.test);
        let test_node_id = self.retrieve_recorded_ast_node();

        /* cfg - body basic block */
        let body_graph_ix = control_flow!(self, |cfg| {
            cfg.append_condition_to(condition_graph_ix, test_node_id);
            let body_graph_ix = cfg.new_basic_block_normal();

            cfg.ctx(None).default().allow_break().allow_continue();
            body_graph_ix
        });
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg - after body basic block */
        control_flow!(self, |cfg| {
            let after_body_graph_ix = cfg.current_node_ix;
            let after_while_graph_ix = cfg.new_basic_block_normal();

            cfg.add_edge(before_while_stmt_graph_ix, condition_graph_ix, EdgeType::Normal);
            cfg.add_edge(condition_graph_ix, body_graph_ix, EdgeType::Jump);
            cfg.add_edge(after_body_graph_ix, condition_graph_ix, EdgeType::Backedge);
            cfg.add_edge(condition_graph_ix, after_while_graph_ix, EdgeType::Normal);

            cfg.ctx(None)
                .mark_break(after_while_graph_ix)
                .mark_continue(condition_graph_ix)
                .resolve_with_upper_label();
        });
        /* cfg */
        self.leave_node(kind);
    }

    fn visit_with_statement(&mut self, stmt: &WithStatement<'a>) {
        let kind = AstKind::WithStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg - condition basic block */
        let (before_with_stmt_graph_ix, condition_graph_ix) =
            control_flow!(self, |cfg| (cfg.current_node_ix, cfg.new_basic_block_normal()));
        /* cfg */

        self.visit_expression(&stmt.object);

        /* cfg - body basic block */
        let body_graph_ix = control_flow!(self, |cfg| cfg.new_basic_block_normal());
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg - after body basic block */
        control_flow!(self, |cfg| {
            let after_body_graph_ix = cfg.new_basic_block_normal();

            cfg.add_edge(before_with_stmt_graph_ix, condition_graph_ix, EdgeType::Normal);
            cfg.add_edge(condition_graph_ix, body_graph_ix, EdgeType::Normal);
            cfg.add_edge(body_graph_ix, after_body_graph_ix, EdgeType::Normal);
            cfg.add_edge(condition_graph_ix, after_body_graph_ix, EdgeType::Normal);
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        /* cfg */
        let (before_function_graph_ix, error_harness, function_graph_ix) =
            control_flow!(self, |cfg| {
                let before_function_graph_ix = cfg.current_node_ix;
                cfg.push_finalization_stack();
                let error_harness = cfg.attach_error_harness(ErrorEdgeKind::Implicit);
                let function_graph_ix = cfg.new_basic_block_function();
                cfg.ctx(None).new_function();
                (before_function_graph_ix, error_harness, function_graph_ix)
            });
        /* cfg */

        // We add a new basic block to the cfg before entering the node
        // so that the correct cfg_ix is associated with the ast node.
        let kind = AstKind::Function(self.alloc(func));
        self.enter_node(kind);
        self.enter_scope(
            {
                let mut flags = flags;
                if func.has_use_strict_directive() {
                    flags |= ScopeFlags::StrictMode;
                }
                flags
            },
            &func.scope_id,
        );

        if func.is_expression() {
            // We need to bind function expression in the function scope
            func.bind(self);
        }

        if let Some(id) = &func.id {
            self.visit_binding_identifier(id);
        }

        /* cfg */
        control_flow!(self, |cfg| cfg.add_edge(
            before_function_graph_ix,
            function_graph_ix,
            EdgeType::NewFunction
        ));
        /* cfg */

        if let Some(type_parameters) = &func.type_parameters {
            self.visit_ts_type_parameter_declaration(type_parameters);
        }
        if let Some(this_param) = &func.this_param {
            self.visit_ts_this_parameter(this_param);
        }
        self.visit_formal_parameters(&func.params);
        if let Some(return_type) = &func.return_type {
            self.visit_ts_type_annotation(return_type);
        }

        if func.params.has_parameter() || func.return_type.is_some() {
            // `function foo({bar: identifier_reference}) {}`
            //                     ^^^^^^^^^^^^^^^^^^^^
            // `function foo<SomeType>(v: SomeType): SomeType { return v; }`
            //                            ^^^^^^^^   ^^^^^^^^
            // Parameter initializers must be resolved after all parameters have been declared.
            // Param types and return type must be resolved after type parameters have been declared.
            // In both cases, need to avoid binding to variables/types declared inside the function body.
            self.resolve_references_for_current_scope();
        }

        if let Some(body) = &func.body {
            self.visit_function_body(body);
        }

        /* cfg */
        control_flow!(self, |cfg| {
            let c = cfg.current_basic_block();
            // If the last is an unreachable instruction, it means there is already a explicit
            // return or throw statement at the end of function body, we don't need to
            // insert an implicit return.
            if !matches!(
                c.instructions().last().map(|inst| &inst.kind),
                Some(InstructionKind::Unreachable)
            ) {
                cfg.push_implicit_return();
            }
            cfg.ctx(None).resolve_expect(CtxFlags::FUNCTION);
            cfg.release_error_harness(error_harness);
            cfg.pop_finalization_stack();
            let after_function_graph_ix = cfg.new_basic_block_normal();
            cfg.add_edge(before_function_graph_ix, after_function_graph_ix, EdgeType::Normal);
        });
        /* cfg */

        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_arrow_function_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        /* cfg */
        let (current_node_ix, error_harness, function_graph_ix) = control_flow!(self, |cfg| {
            let current_node_ix = cfg.current_node_ix;
            cfg.push_finalization_stack();
            let error_harness = cfg.attach_error_harness(ErrorEdgeKind::Implicit);
            let function_graph_ix = cfg.new_basic_block_function();
            cfg.ctx(None).new_function();
            (current_node_ix, error_harness, function_graph_ix)
        });
        /* cfg */

        // We add a new basic block to the cfg before entering the node
        // so that the correct cfg_ix is associated with the ast node.
        let kind = AstKind::ArrowFunctionExpression(self.alloc(expr));
        self.enter_node(kind);
        self.enter_scope(
            {
                let mut flags = ScopeFlags::Function | ScopeFlags::Arrow;
                if expr.has_use_strict_directive() {
                    flags |= ScopeFlags::StrictMode;
                }
                flags
            },
            &expr.scope_id,
        );

        if let Some(parameters) = &expr.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }

        self.visit_formal_parameters(&expr.params);

        /* cfg */
        control_flow!(self, |cfg| cfg.add_edge(
            current_node_ix,
            function_graph_ix,
            EdgeType::NewFunction
        ));
        /* cfg */

        if let Some(return_type) = &expr.return_type {
            self.visit_ts_type_annotation(return_type);
        }

        if expr.params.has_parameter() || expr.return_type.is_some() {
            // `let foo = ({bar: identifier_reference}) => {};`
            //                   ^^^^^^^^^^^^^^^^^^^^
            // `let foo = <SomeType>(v: SomeType): SomeType => v;`
            //                          ^^^^^^^^   ^^^^^^^^
            // Parameter initializers must be resolved after all parameters have been declared.
            // Param types and return type must be resolved after type parameters have been declared.
            // In both cases, need to avoid binding to variables/types declared inside the function body.
            self.resolve_references_for_current_scope();
        }

        self.visit_function_body(&expr.body);

        /* cfg */
        control_flow!(self, |cfg| {
            let c = cfg.current_basic_block();
            // If the last is an unreachable instruction, it means there is already a explicit
            // return or throw statement at the end of function body, we don't need to
            // insert an implicit return.
            if !matches!(
                c.instructions().last().map(|inst| &inst.kind),
                Some(InstructionKind::Unreachable)
            ) {
                cfg.push_implicit_return();
            }
            cfg.ctx(None).resolve_expect(CtxFlags::FUNCTION);
            cfg.release_error_harness(error_harness);
            cfg.pop_finalization_stack();
            cfg.current_node_ix = current_node_ix;
        });
        /* cfg */

        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        let kind = AstKind::UpdateExpression(self.alloc(it));
        self.enter_node(kind);
        // `++a` or `a--`
        //    ^      ^ We always treat `a` as Read and Write reference,
        self.current_reference_flags = ReferenceFlags::read_write();
        self.visit_simple_assignment_target(&it.argument);
        self.leave_node(kind);
    }

    fn visit_member_expression(&mut self, it: &MemberExpression<'a>) {
        let kind = AstKind::MemberExpression(self.alloc(it));
        self.enter_node(kind);

        // A.B = 1;
        // ^^^ Can't treat A as a Write reference since it's A's property(B) that changes.
        self.current_reference_flags -= ReferenceFlags::Write;

        match it {
            MemberExpression::ComputedMemberExpression(it) => {
                self.visit_computed_member_expression(it);
            }
            MemberExpression::StaticMemberExpression(it) => self.visit_static_member_expression(it),
            MemberExpression::PrivateFieldExpression(it) => self.visit_private_field_expression(it),
        }
        self.leave_node(kind);
    }

    fn visit_simple_assignment_target(&mut self, it: &SimpleAssignmentTarget<'a>) {
        let kind = AstKind::SimpleAssignmentTarget(self.alloc(it));
        self.enter_node(kind);
        // Except that the read-write flags has been set in visit_assignment_expression
        // and visit_update_expression, this is always a write-only reference here.
        if !self.current_reference_flags.is_write() {
            self.current_reference_flags = ReferenceFlags::Write;
        }

        match it {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(it) => {
                self.visit_identifier_reference(it);
            }
            SimpleAssignmentTarget::TSAsExpression(it) => {
                self.visit_ts_as_expression(it);
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(it) => {
                self.visit_ts_satisfies_expression(it);
            }
            SimpleAssignmentTarget::TSNonNullExpression(it) => {
                self.visit_ts_non_null_expression(it);
            }
            SimpleAssignmentTarget::TSTypeAssertion(it) => {
                self.visit_ts_type_assertion(it);
            }
            SimpleAssignmentTarget::TSInstantiationExpression(it) => {
                self.visit_ts_instantiation_expression(it);
            }
            match_member_expression!(SimpleAssignmentTarget) => {
                self.visit_member_expression(it.to_member_expression());
            }
        }
        self.leave_node(kind);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        it: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        // NOTE: AstKind doesn't exists!
        self.current_reference_flags = ReferenceFlags::Write;
        self.visit_identifier_reference(&it.binding);
        if let Some(init) = &it.init {
            self.visit_expression(init);
        }
    }

    fn visit_export_default_declaration_kind(&mut self, it: &ExportDefaultDeclarationKind<'a>) {
        match it {
            ExportDefaultDeclarationKind::FunctionDeclaration(it) => {
                let flags = ScopeFlags::Function;
                self.visit_function(it, flags);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(it) => self.visit_class(it),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(it) => {
                self.visit_ts_interface_declaration(it);
            }
            ExportDefaultDeclarationKind::Identifier(it) => {
                // `export default ident`
                //                 ^^^^^ -> can reference both type/value symbols
                self.current_reference_flags = ReferenceFlags::Read | ReferenceFlags::Type;
                self.visit_identifier_reference(it);
            }
            match_expression!(ExportDefaultDeclarationKind) => {
                self.visit_expression(it.to_expression());
            }
        }
    }

    fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
        let kind = AstKind::ExportNamedDeclaration(self.alloc(it));
        self.enter_node(kind);
        self.visit_span(&it.span);
        if let Some(declaration) = &it.declaration {
            self.visit_declaration(declaration);
        }

        if let Some(source) = &it.source {
            self.visit_string_literal(source);
            self.visit_export_specifiers(&it.specifiers);
        } else {
            for specifier in &it.specifiers {
                // `export type { a }` or `export { type a }` -> `a` is a type reference
                if it.export_kind.is_type() || specifier.export_kind.is_type() {
                    self.current_reference_flags = ReferenceFlags::Type;
                } else {
                    // If the export specifier is not a explicit type export, we consider it as a potential
                    // type and value reference. If it references to a value in the end, we would delete the
                    // `ReferenceFlags::Type` flag in `fn resolve_references_for_current_scope`.
                    self.current_reference_flags = ReferenceFlags::Read | ReferenceFlags::Type;
                }
                self.visit_export_specifier(specifier);
            }
        }
        if let Some(with_clause) = &it.with_clause {
            self.visit_with_clause(with_clause);
        }

        self.leave_node(kind);
    }

    fn visit_export_specifier(&mut self, it: &ExportSpecifier<'a>) {
        let kind = AstKind::ExportSpecifier(self.alloc(it));
        self.enter_node(kind);
        self.visit_span(&it.span);

        self.current_node_flags |= NodeFlags::ExportSpecifier;

        self.visit_module_export_name(&it.local);
        self.visit_module_export_name(&it.exported);

        self.current_node_flags -= NodeFlags::ExportSpecifier;

        self.leave_node(kind);
    }

    fn visit_ts_export_assignment(&mut self, it: &TSExportAssignment<'a>) {
        let kind = AstKind::TSExportAssignment(self.alloc(it));
        self.enter_node(kind);
        self.visit_span(&it.span);
        // export = a;
        //          ^ can reference type/value symbols
        if it.expression.is_identifier_reference() {
            self.current_reference_flags = ReferenceFlags::Read | ReferenceFlags::Type;
        }
        self.visit_expression(&it.expression);
        self.leave_node(kind);
    }
}

impl<'a> SemanticBuilder<'a> {
    fn enter_kind(&mut self, kind: AstKind<'a>) {
        /* cfg */
        control_flow!(self, |cfg| {
            match kind {
                AstKind::ReturnStatement(_)
                | AstKind::BreakStatement(_)
                | AstKind::ContinueStatement(_)
                | AstKind::ThrowStatement(_) => { /* These types have their own `InstructionKind`. */
                }
                it if it.is_statement() => {
                    cfg.enter_statement(self.current_node_id);
                }
                _ => { /* ignore the rest */ }
            }
        });
        /* cfg */

        match kind {
            AstKind::ImportSpecifier(specifier) => {
                specifier.bind(self);
            }
            AstKind::ImportDefaultSpecifier(specifier) => {
                specifier.bind(self);
            }
            AstKind::ImportNamespaceSpecifier(specifier) => {
                specifier.bind(self);
            }
            AstKind::TSImportEqualsDeclaration(decl) => {
                decl.bind(self);
            }
            AstKind::VariableDeclarator(decl) => {
                decl.bind(self);
                self.make_all_namespaces_valuelike();
            }
            AstKind::Function(func) => {
                self.function_stack.push(self.current_node_id);
                if func.is_declaration() {
                    func.bind(self);
                }
                self.make_all_namespaces_valuelike();
            }
            AstKind::ArrowFunctionExpression(_) => {
                self.function_stack.push(self.current_node_id);
                self.make_all_namespaces_valuelike();
            }
            AstKind::Class(class) => {
                self.current_node_flags |= NodeFlags::Class;
                if class.is_declaration() {
                    class.bind(self);
                }
                self.make_all_namespaces_valuelike();
            }
            AstKind::ClassBody(body) => {
                self.class_table_builder.declare_class_body(
                    body,
                    self.current_node_id,
                    &self.nodes,
                );
            }
            AstKind::PrivateIdentifier(ident) => {
                self.class_table_builder.add_private_identifier_reference(
                    ident,
                    self.current_node_id,
                    &self.nodes,
                );
            }
            AstKind::BindingRestElement(element) => {
                element.bind(self);
            }
            AstKind::FormalParameter(param) => {
                param.bind(self);
            }
            AstKind::CatchParameter(param) => {
                param.bind(self);
            }
            AstKind::TSModuleDeclaration(module_declaration) => {
                module_declaration.bind(self);
                let symbol_id = match &module_declaration.id {
                    TSModuleDeclarationName::Identifier(ident) => ident.symbol_id.get(),
                    TSModuleDeclarationName::StringLiteral(_) => None,
                };
                self.namespace_stack.push(symbol_id);
            }
            AstKind::TSTypeAliasDeclaration(type_alias_declaration) => {
                type_alias_declaration.bind(self);
            }
            AstKind::TSInterfaceDeclaration(interface_declaration) => {
                interface_declaration.bind(self);
            }
            AstKind::TSEnumDeclaration(enum_declaration) => {
                enum_declaration.bind(self);
                // TODO: const enum?
                self.make_all_namespaces_valuelike();
            }
            AstKind::TSEnumMember(enum_member) => {
                enum_member.bind(self);
            }
            AstKind::TSTypeParameter(type_parameter) => {
                type_parameter.bind(self);
            }
            AstKind::TSInterfaceHeritage(_) => {
                self.current_reference_flags = ReferenceFlags::Type;
            }
            AstKind::TSPropertySignature(signature) => {
                if signature.key.is_expression() {
                    // interface A { [prop]: string }
                    //               ^^^^^ The property can reference value or [`SymbolFlags::TypeImport`] symbol
                    self.current_reference_flags = ReferenceFlags::ValueAsType;
                }
            }
            AstKind::TSTypeQuery(_) => {
                // type A = typeof a;
                //          ^^^^^^^^
                self.current_reference_flags = ReferenceFlags::ValueAsType;
            }
            AstKind::TSTypeParameterInstantiation(_) => {
                // type A<T> = typeof a<T>;
                //                     ^^^ avoid treat T as a value and TSTypeQuery
                self.current_reference_flags -= ReferenceFlags::ValueAsType;
            }
            AstKind::TSTypeName(_) => {
                match self.nodes.parent_kind(self.current_node_id) {
                    Some(
                        // import A = a;
                        //            ^
                        AstKind::TSModuleReference(_),
                    ) => {
                        self.current_reference_flags = ReferenceFlags::Read;
                    }
                    Some(AstKind::TSQualifiedName(_)) => {
                        // import A = a.b
                        //            ^^^ Keep the current reference flag
                    }
                    _ => {
                        // Handled in `AstKind::PropertySignature` or `AstKind::TSTypeQuery`
                        if !self.current_reference_flags.is_value_as_type() {
                            self.current_reference_flags = ReferenceFlags::Type;
                        }
                    }
                }
            }
            AstKind::IdentifierReference(ident) => {
                self.reference_identifier(ident);
            }
            AstKind::LabeledStatement(stmt) => {
                self.unused_labels.add(stmt.label.name.as_str());
            }
            AstKind::ContinueStatement(ContinueStatement { label, .. })
            | AstKind::BreakStatement(BreakStatement { label, .. }) => {
                if let Some(label) = &label {
                    self.unused_labels.reference(&label.name);
                }
            }
            AstKind::YieldExpression(_) => {
                self.set_function_node_flags(NodeFlags::HasYield);
            }
            _ => {}
        }
    }

    #[allow(clippy::single_match)]
    fn leave_kind(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Class(_) => {
                self.current_node_flags -= NodeFlags::Class;
                self.class_table_builder.pop_class();
            }
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                self.function_stack.pop();
            }
            AstKind::CatchParameter(_) => {
                self.resolve_references_for_current_scope();
            }
            AstKind::TSModuleDeclaration(_) => {
                self.namespace_stack.pop();
            }
            AstKind::TSTypeName(_) => {
                self.current_reference_flags -= ReferenceFlags::Type;
            }
            AstKind::TSTypeQuery(_)
            // Clear the reference flags that are set in AstKind::PropertySignature
            | AstKind::PropertyKey(_) => {
                self.current_reference_flags = ReferenceFlags::empty();
            }
            AstKind::LabeledStatement(_) => self.unused_labels.mark_unused(self.current_node_id),
            _ => {}
        }
    }

    fn make_all_namespaces_valuelike(&mut self) {
        for symbol_id in self.namespace_stack.iter().copied() {
            let Some(symbol_id) = symbol_id else {
                continue;
            };

            // Ambient modules cannot be value modules
            if self.symbols.get_flags(symbol_id).intersects(SymbolFlags::Ambient) {
                continue;
            }
            self.symbols.union_flag(symbol_id, SymbolFlags::ValueModule);
        }
    }

    fn reference_identifier(&mut self, ident: &IdentifierReference<'a>) {
        let flags = self.resolve_reference_usages();
        let reference = Reference::new(self.current_node_id, flags);
        let reference_id = self.declare_reference(ident.name, reference);
        ident.reference_id.set(Some(reference_id));
    }

    /// Resolve reference flags for the current ast node.
    #[inline]
    fn resolve_reference_usages(&mut self) -> ReferenceFlags {
        if self.current_reference_flags.is_empty() {
            ReferenceFlags::Read
        } else {
            // Take the current reference flags so that we can reset it to empty
            mem::take(&mut self.current_reference_flags)
        }
    }
}
