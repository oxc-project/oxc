//! Semantic Builder

use std::{cell::RefCell, path::PathBuf, sync::Arc};

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind, Trivias, Visit};
use oxc_cfg::{
    ControlFlowGraphBuilder, CtxCursor, CtxFlags, EdgeType, ErrorEdgeKind,
    IterationInstructionKind, ReturnInstructionKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{CompactStr, SourceType, Span};
use oxc_syntax::{module_record::ModuleRecord, operator::AssignmentOperator};

use crate::{
    binder::Binder,
    checker,
    class::ClassTableBuilder,
    diagnostics::redeclaration,
    jsdoc::JSDocBuilder,
    label::LabelBuilder,
    module_record::ModuleRecordBuilder,
    node::{AstNode, AstNodeId, AstNodes, NodeFlags},
    reference::{Reference, ReferenceFlag, ReferenceId},
    scope::{ScopeFlags, ScopeId, ScopeTree},
    symbol::{SymbolFlags, SymbolId, SymbolTable},
    Semantic,
};

macro_rules! control_flow {
    (|$self:ident, $cfg:tt| $body:expr) => {
        if let Some(ref mut $cfg) = $self.cfg {
            $body
        } else {
            Default::default()
        }
    };
}

pub struct SemanticBuilder<'a> {
    pub source_text: &'a str,

    pub source_type: SourceType,

    trivias: Trivias,

    /// Semantic early errors such as redeclaration errors.
    errors: RefCell<Vec<OxcDiagnostic>>,

    // states
    pub current_node_id: AstNodeId,
    pub current_node_flags: NodeFlags,
    pub current_symbol_flags: SymbolFlags,
    pub current_scope_id: ScopeId,
    /// Stores current `AstKind::Function` and `AstKind::ArrowFunctionExpression` during AST visit
    pub function_stack: Vec<AstNodeId>,
    // To make a namespace/module value like
    // we need the to know the modules we are inside
    // and when we reach a value declaration we set it
    // to value like
    pub namespace_stack: Vec<SymbolId>,
    /// symbol meaning criteria stack. For resolving symbol references.
    meaning_stack: Vec<SymbolFlags>,
    current_reference_flag: ReferenceFlag,

    // builders
    pub nodes: AstNodes<'a>,
    pub scope: ScopeTree,
    pub symbols: SymbolTable,

    pub(crate) module_record: Arc<ModuleRecord>,

    pub label_builder: LabelBuilder<'a>,

    jsdoc: JSDocBuilder<'a>,

    check_syntax_error: bool,

    pub cfg: Option<ControlFlowGraphBuilder<'a>>,

    pub class_table_builder: ClassTableBuilder,

    ast_nodes_records: Vec<Vec<AstNodeId>>,
}

pub struct SemanticBuilderReturn<'a> {
    pub semantic: Semantic<'a>,
    pub errors: Vec<OxcDiagnostic>,
}

impl<'a> SemanticBuilder<'a> {
    pub fn new(source_text: &'a str, source_type: SourceType) -> Self {
        let scope = ScopeTree::default();
        let current_scope_id = scope.root_scope_id();

        let trivias = Trivias::default();
        Self {
            source_text,
            source_type,
            trivias: trivias.clone(),
            errors: RefCell::new(vec![]),
            current_node_id: AstNodeId::new(0),
            current_node_flags: NodeFlags::empty(),
            current_symbol_flags: SymbolFlags::empty(),
            current_reference_flag: ReferenceFlag::empty(),
            current_scope_id,
            function_stack: vec![],
            namespace_stack: vec![],
            meaning_stack: vec![SymbolFlags::Value],
            nodes: AstNodes::default(),
            scope,
            symbols: SymbolTable::default(),
            module_record: Arc::new(ModuleRecord::default()),
            label_builder: LabelBuilder::default(),
            jsdoc: JSDocBuilder::new(source_text, trivias),
            check_syntax_error: false,
            cfg: None,
            class_table_builder: ClassTableBuilder::new(),
            ast_nodes_records: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_trivias(mut self, trivias: Trivias) -> Self {
        self.trivias = trivias.clone();
        self.jsdoc = JSDocBuilder::new(self.source_text, trivias);
        self
    }

    #[must_use]
    pub fn with_check_syntax_error(mut self, yes: bool) -> Self {
        self.check_syntax_error = yes;
        self
    }

    #[must_use]
    pub fn with_cfg(mut self, cfg: bool) -> Self {
        self.cfg = if cfg { Some(ControlFlowGraphBuilder::default()) } else { None };
        self
    }

    /// Get the built module record from `build_module_record`
    pub fn module_record(&self) -> Arc<ModuleRecord> {
        Arc::clone(&self.module_record)
    }

    /// Build the module record with a shallow AST visit
    #[must_use]
    pub fn build_module_record(
        mut self,
        resolved_absolute_path: PathBuf,
        program: &Program<'a>,
    ) -> Self {
        let mut module_record_builder = ModuleRecordBuilder::new(resolved_absolute_path);
        module_record_builder.visit(program);
        self.module_record = Arc::new(module_record_builder.build());
        self
    }

    pub fn build(mut self, program: &Program<'a>) -> SemanticBuilderReturn<'a> {
        if self.source_type.is_typescript_definition() {
            let scope_id = self.scope.add_scope(None, ScopeFlags::Top);
            program.scope_id.set(Some(scope_id));
        } else {
            self.visit_program(program);
            if self.check_syntax_error {
                checker::check_last(&self);
            }

            // Checking syntax error on module record requires scope information from the previous AST pass
            if self.check_syntax_error {
                checker::check_module_record(&self);
            }
        }

        let semantic = Semantic {
            source_text: self.source_text,
            source_type: self.source_type,
            trivias: self.trivias,
            nodes: self.nodes,
            scopes: self.scope,
            symbols: self.symbols,
            classes: self.class_table_builder.build(),
            module_record: Arc::clone(&self.module_record),
            jsdoc: self.jsdoc.build(),
            unused_labels: self.label_builder.unused_node_ids,
            cfg: self.cfg.map(ControlFlowGraphBuilder::build),
        };
        SemanticBuilderReturn { semantic, errors: self.errors.into_inner() }
    }

    /// Push a Syntax Error
    pub fn error(&self, error: OxcDiagnostic) {
        self.errors.borrow_mut().push(error);
    }

    fn create_ast_node(&mut self, kind: AstKind<'a>) {
        let mut flags = self.current_node_flags;
        if self.jsdoc.retrieve_attached_jsdoc(&kind) {
            flags |= NodeFlags::JSDoc;
        }

        let ast_node = AstNode::new(
            kind,
            self.current_scope_id,
            control_flow!(|self, cfg| cfg.current_node_ix),
            flags,
        );
        self.current_node_id = if matches!(kind, AstKind::Program(_)) {
            let id = self.nodes.add_node(ast_node, None);
            #[allow(unsafe_code)]
            // SAFETY: `ast_node` is a `Program` and hence the root of the tree.
            unsafe {
                self.nodes.set_root(&ast_node);
            }
            id
        } else {
            self.nodes.add_node(ast_node, Some(self.current_node_id))
        };
        self.record_ast_node();
    }

    fn pop_ast_node(&mut self) {
        if let Some(parent_id) = self.nodes.parent_id(self.current_node_id) {
            self.current_node_id = parent_id;
        }
    }

    fn record_ast_nodes(&mut self) {
        self.ast_nodes_records.push(Vec::new());
    }

    fn retrieve_recorded_ast_nodes(&mut self) -> Vec<AstNodeId> {
        self.ast_nodes_records.pop().expect("there is no ast nodes record to stop.")
    }

    fn record_ast_node(&mut self) {
        if let Some(records) = self.ast_nodes_records.last_mut() {
            records.push(self.current_node_id);
        }
    }

    pub fn current_meaning(&self) -> SymbolFlags {
        let meaning = self.meaning_stack.last().copied();
        #[cfg(debug_assertions)]
        return meaning.unwrap();
        #[cfg(not(debug_assertions))]
        return meaning.unwrap_or(SymbolFlags::all());
    }

    pub fn current_scope_flags(&self) -> ScopeFlags {
        self.scope.get_flags(self.current_scope_id)
    }

    pub fn strict_mode(&self) -> bool {
        self.current_scope_flags().is_strict_mode()
            || self.current_node_flags.contains(NodeFlags::Class)
    }

    pub fn set_function_node_flag(&mut self, flag: NodeFlags) {
        if let Some(current_function) = self.function_stack.last() {
            *self.nodes.get_node_mut(*current_function).flags_mut() |= flag;
        }
    }

    /// Declares a `Symbol` for the node, adds it to symbol table, and binds it to the scope.
    ///
    /// includes: the `SymbolFlags` that node has in addition to its declaration type (eg: export, ambient, etc.)
    /// excludes: the flags which node cannot be declared alongside in a symbol table. Used to report forbidden declarations.
    ///
    /// Reports errors for conflicting identifier names.
    pub fn declare_symbol_on_scope(
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

        let includes = includes | self.current_symbol_flags;
        let name = CompactStr::new(name);
        let symbol_id = self.symbols.create_symbol(span, name.clone(), includes, scope_id);
        self.symbols.add_declaration(self.current_node_id);
        self.scope.add_binding(scope_id, name, symbol_id);
        symbol_id
    }

    pub fn declare_symbol(
        &mut self,
        span: Span,
        name: &str,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        self.declare_symbol_on_scope(span, name, self.current_scope_id, includes, excludes)
    }

    pub fn check_redeclaration(
        &mut self,
        scope_id: ScopeId,
        span: Span,
        name: &str,
        excludes: SymbolFlags,
        report_error: bool,
    ) -> Option<SymbolId> {
        let symbol_id = self.scope.get_binding(scope_id, name)?;
        if report_error && self.symbols.get_flag(symbol_id).intersects(excludes) {
            let symbol_span = self.symbols.get_span(symbol_id);
            self.error(redeclaration(name, symbol_span, span));
        }
        Some(symbol_id)
    }

    pub fn declare_reference(
        &mut self,
        reference: Reference,
        add_unresolved_reference: bool,
        meaning: SymbolFlags,
    ) -> ReferenceId {
        let reference_name = reference.name().clone();
        let reference_id = self.symbols.create_reference(reference);
        if add_unresolved_reference {
            self.scope.add_unresolved_reference(
                self.current_scope_id,
                reference_name,
                reference_id,
                meaning,
            );
        } else {
            self.resolve_reference_ids(reference_name.clone(), vec![(reference_id, meaning)]);
        }
        reference_id
    }

    /// Declares a `Symbol` for the node, shadowing previous declarations in the same scope.
    pub fn declare_shadow_symbol(
        &mut self,
        name: &str,
        span: Span,
        scope_id: ScopeId,
        includes: SymbolFlags,
    ) -> SymbolId {
        let includes = includes | self.current_symbol_flags;
        let name = CompactStr::new(name);
        let symbol_id =
            self.symbols.create_symbol(span, name.clone(), includes, self.current_scope_id);
        self.symbols.add_declaration(self.current_node_id);
        self.scope.get_bindings_mut(scope_id).insert(name, symbol_id);
        symbol_id
    }

    fn resolve_references_for_current_scope(&mut self) {
        let all_references = self
            .scope
            .unresolved_references_mut(self.current_scope_id)
            .drain()
            .collect::<Vec<(_, Vec<_>)>>();

        for (name, reference_ids) in all_references {
            self.resolve_reference_ids(name, reference_ids);
        }
    }

    fn resolve_reference_ids(
        &mut self,
        name: CompactStr,
        reference_ids: Vec<(ReferenceId, SymbolFlags)>,
    ) {
        let parent_scope_id =
            self.scope.get_parent_id(self.current_scope_id).unwrap_or(self.current_scope_id);

        if let Some(symbol_id) = self.scope.get_binding(self.current_scope_id, &name) {
            let symbol_flags = self.symbols.get_flag(symbol_id);
            let mut unresolved: Vec<(ReferenceId, SymbolFlags)> =
                Vec::with_capacity(reference_ids.len());
            for (reference_id, meaning) in reference_ids {
                let reference = &mut self.symbols.references[reference_id];
                // if dbg!(symbol_flags).intersects(dbg!(meaning)) {
                if symbol_flags.intersects(meaning) {
                    // println!("true");
                    reference.set_symbol_id(symbol_id);
                    self.symbols.resolved_references[symbol_id].push(reference_id);
                } else {
                    // println!("false");
                    unresolved.push((reference_id, meaning))
                }
            }
            self.scope.extend_unresolved_reference(parent_scope_id, name, unresolved);
        } else {
            self.scope.extend_unresolved_reference(parent_scope_id, name, reference_ids);
        }
    }

    pub fn add_redeclare_variable(&mut self, symbol_id: SymbolId, span: Span) {
        self.symbols.add_redeclare_variable(symbol_id, span);
    }

    fn add_export_flag_to_export_identifiers(&mut self, program: &Program<'a>) {
        for stmt in &program.body {
            if let Statement::ExportDefaultDeclaration(decl) = stmt {
                if let ExportDefaultDeclarationKind::Identifier(ident) = &decl.declaration {
                    self.add_export_flag_to_identifier(ident.name.as_str());
                }
            }
            if let Statement::ExportNamedDeclaration(decl) = stmt {
                for specifier in &decl.specifiers {
                    if specifier.export_kind.is_value() {
                        if let Some(name) = specifier.local.identifier_name() {
                            self.add_export_flag_to_identifier(name.as_str());
                        }
                    }
                }
            }
        }
    }

    fn add_export_flag_to_identifier(&mut self, name: &str) {
        if let Some(symbol_id) = self.scope.get_binding(self.current_scope_id, name) {
            self.symbols.union_flag(symbol_id, SymbolFlags::Export);
        }
    }
}

impl<'a> Visit<'a> for SemanticBuilder<'a> {
    fn enter_scope(&mut self, flags: ScopeFlags) {
        let parent_scope_id =
            if flags.contains(ScopeFlags::Top) { None } else { Some(self.current_scope_id) };

        let mut flags = flags;
        if let Some(parent_scope_id) = parent_scope_id {
            flags = self.scope.get_new_scope_flags(flags, parent_scope_id);
        }

        self.current_scope_id = self.scope.add_scope(parent_scope_id, flags);
    }

    fn leave_scope(&mut self) {
        self.resolve_references_for_current_scope();
        if let Some(parent_id) = self.scope.get_parent_id(self.current_scope_id) {
            self.current_scope_id = parent_id;
        }
    }

    // Setup all the context for the binder.
    // The order is important here.
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
        self.enter_scope({
            let mut flags = ScopeFlags::Top;
            if program.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
        program.scope_id.set(Some(self.current_scope_id));

        /* cfg */
        let error_harness = control_flow!(|self, cfg| {
            let error_harness = cfg.attach_error_harness(ErrorEdgeKind::Implicit);
            let _program_basic_block = cfg.new_basic_block_normal();
            error_harness
        });
        /* cfg - must be above directives as directives are in cfg */

        self.enter_node(kind);

        for directive in &program.directives {
            self.visit_directive(directive);
        }

        self.visit_statements(&program.body);

        /* cfg */
        control_flow!(|self, cfg| cfg.release_error_harness(error_harness));
        /* cfg */

        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_block_statement(&mut self, stmt: &BlockStatement<'a>) {
        let kind = AstKind::BlockStatement(self.alloc(stmt));
        self.enter_scope(ScopeFlags::empty());
        stmt.scope_id.set(Some(self.current_scope_id));
        self.enter_node(kind);

        self.visit_statements(&stmt.body);

        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_break_statement(&mut self, stmt: &BreakStatement<'a>) {
        let kind = AstKind::BreakStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let node_id = self.current_node_id;
        /* cfg */

        if let Some(ref break_target) = stmt.label {
            self.visit_label_identifier(break_target);
        }

        /* cfg */
        control_flow!(
            |self, cfg| cfg.append_break(node_id, stmt.label.as_ref().map(|it| it.name.as_str()))
        );
        /* cfg */

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
        control_flow!(|self, cfg| cfg
            .append_continue(node_id, stmt.label.as_ref().map(|it| it.name.as_str())));
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_debugger_statement(&mut self, stmt: &DebuggerStatement) {
        let kind = AstKind::DebuggerStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_do_while_statement(&mut self, stmt: &DoWhileStatement<'a>) {
        let kind = AstKind::DoWhileStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let (before_do_while_stmt_graph_ix, start_body_graph_ix) = control_flow!(|self, cfg| {
            let before_do_while_stmt_graph_ix = cfg.current_node_ix;
            let start_body_graph_ix = cfg.new_basic_block_normal();
            cfg.ctx(None).default().allow_break().allow_continue();
            (before_do_while_stmt_graph_ix, start_body_graph_ix)
        });
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg - condition basic block */
        let (after_body_graph_ix, start_of_condition_graph_ix) = control_flow!(|self, cfg| {
            let after_body_graph_ix = cfg.current_node_ix;
            let start_of_condition_graph_ix = cfg.new_basic_block_normal();
            (after_body_graph_ix, start_of_condition_graph_ix)
        });
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&stmt.test);
        let test_node = self.retrieve_recorded_ast_nodes().into_iter().next();

        /* cfg */
        control_flow!(|self, cfg| {
            cfg.append_condition_to(start_of_condition_graph_ix, test_node);
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

    fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
        let kind = AstKind::ExpressionStatement(self.alloc(stmt));
        self.enter_node(kind);

        self.visit_expression(&stmt.expression);

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
        let (left_expr_end_ix, right_expr_start_ix) = control_flow!(|self, cfg| {
            let left_expr_end_ix = cfg.current_node_ix;
            let right_expr_start_ix = cfg.new_basic_block_normal();
            (left_expr_end_ix, right_expr_start_ix)
        });
        /* cfg  */

        self.visit_expression(&expr.right);

        /* cfg */
        control_flow!(|self, cfg| {
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
        self.visit_assignment_target(&expr.left);

        /* cfg  */
        let cfg_ixs = control_flow!(|self, cfg| {
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
        control_flow!(|self, cfg| {
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
            control_flow!(|self, cfg| {
                let before_conditional_graph_ix = cfg.current_node_ix;
                let start_of_condition_graph_ix = cfg.new_basic_block_normal();
                (before_conditional_graph_ix, start_of_condition_graph_ix)
            });
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&expr.test);
        let test_node = self.retrieve_recorded_ast_nodes().into_iter().next();

        /* cfg */
        let (after_condition_graph_ix, before_consequent_expr_graph_ix) =
            control_flow!(|self, cfg| {
                cfg.append_condition_to(start_of_condition_graph_ix, test_node);
                let after_condition_graph_ix = cfg.current_node_ix;
                // conditional expression basic block
                let before_consequent_expr_graph_ix = cfg.new_basic_block_normal();
                (after_condition_graph_ix, before_consequent_expr_graph_ix)
            });
        /* cfg */

        self.visit_expression(&expr.consequent);

        /* cfg */
        let (after_consequent_expr_graph_ix, start_alternate_graph_ix) =
            control_flow!(|self, cfg| {
                let after_consequent_expr_graph_ix = cfg.current_node_ix;
                let start_alternate_graph_ix = cfg.new_basic_block_normal();
                (after_consequent_expr_graph_ix, start_alternate_graph_ix)
            });
        /* cfg */

        self.visit_expression(&expr.alternate);

        /* cfg */
        control_flow!(|self, cfg| {
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
        let is_lexical_declaration =
            stmt.init.as_ref().is_some_and(ForStatementInit::is_lexical_declaration);
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
            stmt.scope_id.set(Some(self.current_scope_id));
        }
        self.enter_node(kind);
        if let Some(init) = &stmt.init {
            self.visit_for_statement_init(init);
        }
        /* cfg */
        let (before_for_graph_ix, test_graph_ix) = control_flow!(|self, cfg| {
            let before_for_graph_ix = cfg.current_node_ix;
            let test_graph_ix = cfg.new_basic_block_normal();
            (before_for_graph_ix, test_graph_ix)
        });
        /* cfg */

        if let Some(test) = &stmt.test {
            self.record_ast_nodes();
            self.visit_expression(test);
            let test_node = self.retrieve_recorded_ast_nodes().into_iter().next();

            /* cfg */
            control_flow!(|self, cfg| cfg.append_condition_to(test_graph_ix, test_node));
            /* cfg */
        }

        /* cfg */
        let (after_test_graph_ix, update_graph_ix) =
            control_flow!(|self, cfg| (cfg.current_node_ix, cfg.new_basic_block_normal()));
        /* cfg */

        if let Some(update) = &stmt.update {
            self.visit_expression(update);
        }

        /* cfg */
        let before_body_graph_ix = control_flow!(|self, cfg| {
            let before_body_graph_ix = cfg.new_basic_block_normal();
            cfg.ctx(None).default().allow_break().allow_continue();
            before_body_graph_ix
        });
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg */
        control_flow!(|self, cfg| {
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

        self.leave_node(kind);
        if is_lexical_declaration {
            self.leave_scope();
        }
    }

    fn visit_for_statement_init(&mut self, init: &ForStatementInit<'a>) {
        let kind = AstKind::ForStatementInit(self.alloc(init));
        self.enter_node(kind);
        match init {
            ForStatementInit::UsingDeclaration(decl) => {
                self.visit_using_declaration(decl);
            }
            ForStatementInit::VariableDeclaration(decl) => {
                self.visit_variable_declaration(decl);
            }
            match_expression!(ForStatementInit) => self.visit_expression(init.to_expression()),
        }
        self.leave_node(kind);
    }

    fn visit_for_in_statement(&mut self, stmt: &ForInStatement<'a>) {
        let kind = AstKind::ForInStatement(self.alloc(stmt));
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
            stmt.scope_id.set(Some(self.current_scope_id));
        }
        self.enter_node(kind);

        self.visit_for_statement_left(&stmt.left);

        /* cfg */
        let (before_for_stmt_graph_ix, start_prepare_cond_graph_ix) =
            control_flow!(|self, cfg| (cfg.current_node_ix, cfg.new_basic_block_normal(),));
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&stmt.right);
        let right_node = self.retrieve_recorded_ast_nodes().into_iter().next();

        /* cfg */
        let (end_of_prepare_cond_graph_ix, iteration_graph_ix, body_graph_ix) =
            control_flow!(|self, cfg| {
                let end_of_prepare_cond_graph_ix = cfg.current_node_ix;
                let iteration_graph_ix = cfg.new_basic_block_normal();
                cfg.append_iteration(right_node, IterationInstructionKind::In);
                let body_graph_ix = cfg.new_basic_block_normal();

                cfg.ctx(None).default().allow_break().allow_continue();
                (end_of_prepare_cond_graph_ix, iteration_graph_ix, body_graph_ix)
            });
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg */
        control_flow!(|self, cfg| {
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

        self.leave_node(kind);
        if is_lexical_declaration {
            self.leave_scope();
        }
    }

    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        let kind = AstKind::ForOfStatement(self.alloc(stmt));
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
            stmt.scope_id.set(Some(self.current_scope_id));
        }
        self.enter_node(kind);

        self.visit_for_statement_left(&stmt.left);

        /* cfg */
        let (before_for_stmt_graph_ix, start_prepare_cond_graph_ix) =
            control_flow!(|self, cfg| (cfg.current_node_ix, cfg.new_basic_block_normal()));
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&stmt.right);
        let right_node = self.retrieve_recorded_ast_nodes().into_iter().next();

        /* cfg */
        let (end_of_prepare_cond_graph_ix, iteration_graph_ix, body_graph_ix) =
            control_flow!(|self, cfg| {
                let end_of_prepare_cond_graph_ix = cfg.current_node_ix;
                let iteration_graph_ix = cfg.new_basic_block_normal();
                cfg.append_iteration(right_node, IterationInstructionKind::Of);
                let body_graph_ix = cfg.new_basic_block_normal();
                cfg.ctx(None).default().allow_break().allow_continue();
                (end_of_prepare_cond_graph_ix, iteration_graph_ix, body_graph_ix)
            });
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg */
        control_flow!(|self, cfg| {
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

        self.leave_node(kind);
        if is_lexical_declaration {
            self.leave_scope();
        }
    }

    fn visit_if_statement(&mut self, stmt: &IfStatement<'a>) {
        let kind = AstKind::IfStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg - condition basic block */
        let (before_if_stmt_graph_ix, start_of_condition_graph_ix) =
            control_flow!(|self, cfg| (cfg.current_node_ix, cfg.new_basic_block_normal(),));
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&stmt.test);
        let test_node = self.retrieve_recorded_ast_nodes().into_iter().next();

        /* cfg */
        let (after_test_graph_ix, before_consequent_stmt_graph_ix) = control_flow!(|self, cfg| {
            cfg.append_condition_to(start_of_condition_graph_ix, test_node);
            (cfg.current_node_ix, cfg.new_basic_block_normal())
        });
        /* cfg */

        self.visit_statement(&stmt.consequent);

        /* cfg */
        let after_consequent_stmt_graph_ix = control_flow!(|self, cfg| cfg.current_node_ix);
        /* cfg */

        let else_graph_ix = if let Some(alternate) = &stmt.alternate {
            /* cfg */
            let else_graph_ix = control_flow!(|self, cfg| cfg.new_basic_block_normal());
            /* cfg */

            self.visit_statement(alternate);

            control_flow!(|self, cfg| Some((else_graph_ix, cfg.current_node_ix)))
        } else {
            None
        };

        /* cfg - bb after if statement joins consequent and alternate */
        control_flow!(|self, cfg| {
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
        control_flow!(|self, cfg| {
            let ctx = cfg.ctx(Some(label.as_str())).default().allow_break();
            if stmt.body.is_iteration_statement() {
                ctx.allow_continue();
            }
        });
        /* cfg */

        self.visit_label_identifier(&stmt.label);

        self.visit_statement(&stmt.body);

        /* cfg */
        control_flow!(|self, cfg| {
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
        control_flow!(|self, cfg| {
            cfg.push_return(ret_kind, node_id);
            cfg.append_unreachable();
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_switch_statement(&mut self, stmt: &SwitchStatement<'a>) {
        let kind = AstKind::SwitchStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&stmt.discriminant);
        self.enter_scope(ScopeFlags::empty());
        stmt.scope_id.set(Some(self.current_scope_id));

        /* cfg */
        let discriminant_graph_ix = control_flow!(|self, cfg| {
            let discriminant_graph_ix = cfg.current_node_ix;
            cfg.ctx(None).default().allow_break();
            discriminant_graph_ix
        });
        let mut switch_case_graph_spans = vec![];
        let mut have_default_case = false;
        /* cfg */

        for case in &stmt.cases {
            let before_case_graph_ix = control_flow!(|self, cfg| cfg.new_basic_block_normal());
            self.visit_switch_case(case);
            if case.is_default_case() {
                have_default_case = true;
            }
            control_flow!(|self, cfg| switch_case_graph_spans
                .push((before_case_graph_ix, cfg.current_node_ix)));
        }

        /* cfg */
        // for each switch case
        control_flow!(|self, cfg| {
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
            let test_node = self.retrieve_recorded_ast_nodes().into_iter().next();
            control_flow!(|self, cfg| cfg.append_condition_to(cfg.current_node_ix, test_node));
        }

        /* cfg */
        control_flow!(|self, cfg| {
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
        control_flow!(|self, cfg| cfg.append_throw(node_id));
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
        ) = control_flow!(|self, cfg| {
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
        let after_try_block_graph_ix = control_flow!(|self, cfg| cfg.current_node_ix);
        /* cfg */

        let catch_block_end_ix = if let Some(handler) = &stmt.handler {
            /* cfg */
            control_flow!(|self, cfg| {
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
            control_flow!(|self, cfg| {
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
            control_flow!(|self, cfg| {
                let Some(before_finalizer_graph_ix) = before_finalizer_graph_ix else {
                    unreachable!("we always create a finalizer when there is a finally block.");
                };
                cfg.release_finalizer(before_finalizer_graph_ix);
                let start_finally_graph_ix = cfg.new_basic_block_normal();
                cfg.add_edge(before_finalizer_graph_ix, start_finally_graph_ix, EdgeType::Normal);
            });
            /* cfg */

            self.visit_finally_clause(finalizer);

            /* cfg */
            control_flow!(|self, cfg| {
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
        control_flow!(|self, cfg| {
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
                        if cfg.basic_block(after_try_block_graph_ix).unreachable {
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

    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        let kind = AstKind::CatchClause(self.alloc(clause));
        self.enter_scope(ScopeFlags::empty());
        clause.scope_id.set(Some(self.current_scope_id));
        self.enter_node(kind);
        if let Some(param) = &clause.param {
            self.visit_catch_parameter(param);
        }
        self.visit_statements(&clause.body.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_finally_clause(&mut self, clause: &BlockStatement<'a>) {
        let kind = AstKind::FinallyClause(self.alloc(clause));
        self.enter_scope(ScopeFlags::empty());
        clause.scope_id.set(Some(self.current_scope_id));
        self.enter_node(kind);
        self.visit_statements(&clause.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_while_statement(&mut self, stmt: &WhileStatement<'a>) {
        let kind = AstKind::WhileStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg - condition basic block */
        let (before_while_stmt_graph_ix, condition_graph_ix) =
            control_flow!(|self, cfg| (cfg.current_node_ix, cfg.new_basic_block_normal()));
        /* cfg */

        self.record_ast_nodes();
        self.visit_expression(&stmt.test);
        let test_node = self.retrieve_recorded_ast_nodes().into_iter().next();

        /* cfg - body basic block */
        let body_graph_ix = control_flow!(|self, cfg| {
            cfg.append_condition_to(condition_graph_ix, test_node);
            let body_graph_ix = cfg.new_basic_block_normal();

            cfg.ctx(None).default().allow_break().allow_continue();
            body_graph_ix
        });
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg - after body basic block */
        control_flow!(|self, cfg| {
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
            control_flow!(|self, cfg| (cfg.current_node_ix, cfg.new_basic_block_normal()));
        /* cfg */

        self.visit_expression(&stmt.object);

        /* cfg - body basic block */
        let body_graph_ix = control_flow!(|self, cfg| cfg.new_basic_block_normal());
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg - after body basic block */
        control_flow!(|self, cfg| {
            let after_body_graph_ix = cfg.new_basic_block_normal();

            cfg.add_edge(before_with_stmt_graph_ix, condition_graph_ix, EdgeType::Normal);
            cfg.add_edge(condition_graph_ix, body_graph_ix, EdgeType::Normal);
            cfg.add_edge(body_graph_ix, after_body_graph_ix, EdgeType::Normal);
            cfg.add_edge(condition_graph_ix, after_body_graph_ix, EdgeType::Normal);
        });
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: Option<ScopeFlags>) {
        let kind = AstKind::Function(self.alloc(func));
        self.enter_scope({
            let mut flags = flags.unwrap_or(ScopeFlags::empty()) | ScopeFlags::Function;
            if func.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
        func.scope_id.set(Some(self.current_scope_id));

        /* cfg */
        let (before_function_graph_ix, error_harness, function_graph_ix) =
            control_flow!(|self, cfg| {
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
        self.enter_node(kind);

        /* cfg */
        control_flow!(|self, cfg| cfg.add_edge(
            before_function_graph_ix,
            function_graph_ix,
            EdgeType::NewFunction
        ));
        /* cfg */

        if let Some(ident) = &func.id {
            self.visit_binding_identifier(ident);
        }
        self.visit_formal_parameters(&func.params);
        if let Some(body) = &func.body {
            self.visit_function_body(body);
        }

        /* cfg */
        control_flow!(|self, cfg| {
            cfg.ctx(None).resolve_expect(CtxFlags::FUNCTION);
            cfg.release_error_harness(error_harness);
            cfg.pop_finalization_stack();
            let after_function_graph_ix = cfg.new_basic_block_normal();
            cfg.add_edge(before_function_graph_ix, after_function_graph_ix, EdgeType::Normal);
        });
        /* cfg */

        if let Some(parameters) = &func.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &func.return_type {
            self.visit_ts_type_annotation(annotation);
        }
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        // Class level decorators are transpiled as functions outside of the class taking the class
        // itself as argument. They should be visited before class is entered. E.g., they inherit
        // strict mode from the enclosing scope rather than from class.
        for decorator in &class.decorators {
            self.visit_decorator(decorator);
        }
        let kind = AstKind::Class(self.alloc(class));

        // FIXME(don): Should we enter a scope when visiting class declarations?
        let is_class_expr = class.r#type == ClassType::ClassExpression;
        if is_class_expr {
            // Class expressions create a temporary scope with the class name as its only variable
            // E.g., `let c = class A { foo() { console.log(A) } }`
            self.enter_scope(ScopeFlags::empty());
            class.scope_id.set(Some(self.current_scope_id));
        }

        self.enter_node(kind);

        if let Some(id) = &class.id {
            self.visit_binding_identifier(id);
        }
        if let Some(parameters) = &class.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }

        if let Some(super_class) = &class.super_class {
            self.visit_class_heritage(super_class);
        }
        if let Some(super_parameters) = &class.super_type_parameters {
            self.visit_ts_type_parameter_instantiation(super_parameters);
        }
        self.visit_class_body(&class.body);

        self.leave_node(kind);
        if is_class_expr {
            self.leave_scope();
        }
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        let kind = AstKind::StaticBlock(self.alloc(block));
        self.enter_scope(ScopeFlags::ClassStaticBlock);
        block.scope_id.set(Some(self.current_scope_id));
        self.enter_node(kind);
        self.visit_statements(&block.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_arrow_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        let kind = AstKind::ArrowFunctionExpression(self.alloc(expr));
        self.enter_scope(ScopeFlags::Function | ScopeFlags::Arrow);
        expr.scope_id.set(Some(self.current_scope_id));

        /* cfg */
        let (current_node_ix, error_harness, function_graph_ix) = control_flow!(|self, cfg| {
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
        self.enter_node(kind);

        self.visit_formal_parameters(&expr.params);

        /* cfg */
        control_flow!(|self, cfg| cfg.add_edge(
            current_node_ix,
            function_graph_ix,
            EdgeType::NewFunction
        ));
        /* cfg */

        self.visit_function_body(&expr.body);

        /* cfg */
        control_flow!(|self, cfg| {
            cfg.ctx(None).resolve_expect(CtxFlags::FUNCTION);
            cfg.release_error_harness(error_harness);
            cfg.pop_finalization_stack();
            cfg.current_node_ix = current_node_ix;
        });
        /* cfg */

        if let Some(parameters) = &expr.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_enum(&mut self, decl: &TSEnumDeclaration<'a>) {
        let kind = AstKind::TSEnumDeclaration(self.alloc(decl));
        self.enter_node(kind);
        self.visit_binding_identifier(&decl.id);
        self.enter_scope(ScopeFlags::empty());
        decl.scope_id.set(Some(self.current_scope_id));
        for member in &decl.members {
            self.visit_enum_member(member);
        }
        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_ts_module_declaration(&mut self, decl: &TSModuleDeclaration<'a>) {
        let kind = AstKind::TSModuleDeclaration(self.alloc(decl));
        self.enter_node(kind);
        match &decl.id {
            TSModuleDeclarationName::Identifier(ident) => self.visit_identifier_name(ident),
            TSModuleDeclarationName::StringLiteral(lit) => self.visit_string_literal(lit),
        }
        self.enter_scope(ScopeFlags::TsModuleBlock);
        decl.scope_id.set(Some(self.current_scope_id));
        match &decl.body {
            Some(TSModuleDeclarationBody::TSModuleDeclaration(decl)) => {
                self.visit_ts_module_declaration(decl);
            }
            Some(TSModuleDeclarationBody::TSModuleBlock(block)) => {
                self.visit_ts_module_block(block);
            }
            None => {}
        }
        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_ts_type_parameter(&mut self, ty: &TSTypeParameter<'a>) {
        let kind = AstKind::TSTypeParameter(self.alloc(ty));
        self.enter_scope(ScopeFlags::empty());
        ty.scope_id.set(Some(self.current_scope_id));
        self.enter_node(kind);
        if let Some(constraint) = &ty.constraint {
            self.visit_ts_type(constraint);
        }

        if let Some(default) = &ty.default {
            self.visit_ts_type(default);
        }
        self.leave_node(kind);
        self.leave_scope();
    }
}

impl<'a> SemanticBuilder<'a> {
    fn enter_kind(&mut self, kind: AstKind<'a>) {
        /* cfg */
        control_flow!(|self, cfg| {
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
            AstKind::ExportDefaultDeclaration(_) => {
                self.current_symbol_flags |= SymbolFlags::Export;
            }
            AstKind::ExportNamedDeclaration(decl) => {
                self.current_symbol_flags |= SymbolFlags::Export;
                if decl.export_kind.is_type() {
                    self.current_reference_flag = ReferenceFlag::Type;
                }
            }
            AstKind::ExportAllDeclaration(s) if s.export_kind.is_type() => {
                self.current_reference_flag = ReferenceFlag::Type;
            }
            AstKind::ExportSpecifier(s) if s.export_kind.is_type() => {
                self.current_reference_flag = ReferenceFlag::Type;
            }
            AstKind::ImportSpecifier(specifier) => {
                specifier.bind(self);
            }
            AstKind::ImportDefaultSpecifier(specifier) => {
                specifier.bind(self);
            }
            AstKind::ImportNamespaceSpecifier(specifier) => {
                specifier.bind(self);
            }
            AstKind::VariableDeclarator(decl) => {
                decl.bind(self);
                self.make_all_namespaces_valuelike();
            }
            AstKind::StaticBlock(_) => self.label_builder.enter_function_or_static_block(),
            AstKind::Function(func) => {
                self.function_stack.push(self.current_node_id);
                func.bind(self);
                self.label_builder.enter_function_or_static_block();
                self.add_current_node_id_to_current_scope();
                self.make_all_namespaces_valuelike();
            }
            AstKind::ArrowFunctionExpression(_) => {
                self.function_stack.push(self.current_node_id);
                self.add_current_node_id_to_current_scope();
                self.make_all_namespaces_valuelike();
            }
            AstKind::Class(class) => {
                self.current_node_flags |= NodeFlags::Class;
                class.bind(self);
                self.current_symbol_flags -= SymbolFlags::Export;
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
            AstKind::FormalParameters(_) => {
                self.current_node_flags |= NodeFlags::Parameter;
                self.current_symbol_flags -= SymbolFlags::Export;
            }
            AstKind::FormalParameter(param) => {
                param.bind(self);
            }
            AstKind::CatchParameter(param) => {
                self.current_node_flags |= NodeFlags::Parameter;
                param.bind(self);
            }
            AstKind::TSModuleDeclaration(module_declaration) => {
                module_declaration.bind(self);
                let symbol_id = self
                    .scope
                    .get_bindings(self.current_scope_id)
                    .get(module_declaration.id.name().as_str());
                self.namespace_stack.push(*symbol_id.unwrap());
            }
            AstKind::TSTypeAliasDeclaration(type_alias_declaration) => {
                self.meaning_stack.push(SymbolFlags::Type);
                type_alias_declaration.bind(self);
            }
            AstKind::TSInterfaceDeclaration(interface_declaration) => {
                self.meaning_stack.push(SymbolFlags::Type);
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
                self.meaning_stack.push(SymbolFlags::Type);
                type_parameter.bind(self);
            }
            AstKind::ExportSpecifier(s) if s.export_kind.is_type() => {
                self.current_reference_flag = ReferenceFlag::Type;
            }
            AstKind::TSTypeName(_) => {
                self.meaning_stack.push(SymbolFlags::Type);
                self.current_reference_flag = ReferenceFlag::Type;
            }
            AstKind::TSTypeQuery(_) => {
                // checks types of a value symbol (e.g. `typeof x`), so we're
                // looking for a value even though its used as a type
                self.meaning_stack.push(SymbolFlags::Value)
            }
            AstKind::TSTypeAnnotation(_) => {
                self.meaning_stack.push(SymbolFlags::Type);
            }
            AstKind::IdentifierReference(ident) => {
                self.reference_identifier(ident);
            }
            AstKind::JSXIdentifier(ident) => {
                self.reference_jsx_identifier(ident);
            }
            AstKind::UpdateExpression(_) => {
                if self.is_not_expression_statement_parent() {
                    self.current_reference_flag |= ReferenceFlag::Read;
                }
                self.current_reference_flag |= ReferenceFlag::Write;
            }
            AstKind::AssignmentExpression(expr) => {
                if expr.operator != AssignmentOperator::Assign
                    || self.is_not_expression_statement_parent()
                {
                    self.current_reference_flag |= ReferenceFlag::Read;
                }
            }
            AstKind::MemberExpression(_) => {
                self.current_reference_flag = ReferenceFlag::Read;
            }
            AstKind::AssignmentTarget(_) => {
                self.current_reference_flag |= ReferenceFlag::Write;
            }
            AstKind::LabeledStatement(stmt) => {
                self.label_builder.enter(stmt, self.current_node_id);
            }
            AstKind::ContinueStatement(ContinueStatement { label, .. })
            | AstKind::BreakStatement(BreakStatement { label, .. }) => {
                if let Some(label) = &label {
                    self.label_builder.mark_as_used(label);
                }
            }
            AstKind::YieldExpression(_) => {
                self.meaning_stack.push(SymbolFlags::Value);
                self.set_function_node_flag(NodeFlags::HasYield);
            }
            _ => {}
        }
    }

    #[allow(clippy::single_match)]
    fn leave_kind(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Program(program) => {
                self.add_export_flag_to_export_identifiers(program);
            }
            AstKind::Class(_) => {
                self.current_node_flags -= NodeFlags::Class;
                self.class_table_builder.pop_class();
            }
            AstKind::ExportDefaultDeclaration(_) => {
                self.current_symbol_flags -= SymbolFlags::Export;
            }
            AstKind::ExportNamedDeclaration(decl) => {
                self.current_symbol_flags -= SymbolFlags::Export;
                if decl.export_kind.is_type() {
                    self.current_reference_flag -= ReferenceFlag::Type;
                }
            }
            AstKind::ExportAllDeclaration(s) if s.export_kind.is_type() => {
                self.current_reference_flag -= ReferenceFlag::Type;
            }
            AstKind::ExportSpecifier(s) if s.export_kind.is_type() => {
                self.current_reference_flag -= ReferenceFlag::Type;
            }
            AstKind::LabeledStatement(_) => self.label_builder.leave(),
            AstKind::StaticBlock(_) => {
                self.label_builder.leave_function_or_static_block();
            }
            AstKind::Function(_) => {
                self.label_builder.leave_function_or_static_block();
                self.function_stack.pop();
            }
            AstKind::ArrowFunctionExpression(_) => {
                self.function_stack.pop();
            }
            AstKind::FormalParameters(_) | AstKind::CatchParameter(_) => {
                self.current_node_flags -= NodeFlags::Parameter;
            }
            AstKind::TSModuleBlock(_) => {
                self.namespace_stack.pop();
            }
            AstKind::TSTypeAliasDeclaration(_) => {
                self.meaning_stack.pop();
            }
            AstKind::TSInterfaceDeclaration(_) => {
                self.meaning_stack.pop();
            }
            AstKind::TSTypeParameter(_) => {
                self.meaning_stack.pop();
            }
            AstKind::TSTypeName(_) => {
                self.meaning_stack.pop();
                self.current_reference_flag -= ReferenceFlag::Type;
            }
            AstKind::TSTypeQuery(_) => {
                self.meaning_stack.pop();
            }
            AstKind::TSTypeAnnotation(_) => {
                self.meaning_stack.pop();
            }
            AstKind::UpdateExpression(_) => {
                if self.is_not_expression_statement_parent() {
                    self.current_reference_flag -= ReferenceFlag::Read;
                }
                self.current_reference_flag -= ReferenceFlag::Write;
            }
            AstKind::AssignmentExpression(expr) => {
                if expr.operator != AssignmentOperator::Assign
                    || self.is_not_expression_statement_parent()
                {
                    self.current_reference_flag -= ReferenceFlag::Read;
                }
            }
            AstKind::MemberExpression(_) => self.current_reference_flag = ReferenceFlag::empty(),
            AstKind::AssignmentTarget(_) => self.current_reference_flag -= ReferenceFlag::Write,
            AstKind::YieldExpression(_) => {
                self.meaning_stack.pop();
            }
            _ => {}
        }
    }

    fn add_current_node_id_to_current_scope(&mut self) {
        self.scope.add_node_id(self.current_scope_id, self.current_node_id);
    }

    fn make_all_namespaces_valuelike(&mut self) {
        for symbol_id in &self.namespace_stack {
            // Ambient modules cannot be value modules
            if self.symbols.get_flag(*symbol_id).intersects(SymbolFlags::Ambient) {
                continue;
            }
            self.symbols.union_flag(*symbol_id, SymbolFlags::ValueModule);
        }
    }

    fn reference_identifier(&mut self, ident: &IdentifierReference) {
        let flag = self.resolve_reference_usages();
        let name = ident.name.to_compact_str();
        let reference = Reference::new(ident.span, name, self.current_node_id, flag);
        // `function foo({bar: identifier_reference}) {}`
        //                     ^^^^^^^^^^^^^^^^^^^^ Parameter initializer must be resolved immediately
        //                                          to avoid binding to variables inside the scope
        let add_unresolved_reference = !self.current_node_flags.has_parameter();
        let reference_id =
            self.declare_reference(reference, add_unresolved_reference, self.current_meaning());
        // self.declare_reference(reference, add_unresolved_reference, dbg!(self.current_meaning()));
        ident.reference_id.set(Some(reference_id));
    }

    /// Resolve reference flags for the current ast node.
    fn resolve_reference_usages(&self) -> ReferenceFlag {
        if self.current_reference_flag.is_write() || self.current_reference_flag.is_type() {
            self.current_reference_flag
        } else {
            ReferenceFlag::Read
        }
    }

    fn reference_jsx_identifier(&mut self, ident: &JSXIdentifier) {
        match self.nodes.parent_kind(self.current_node_id) {
            Some(AstKind::JSXElementName(_)) => {
                if !ident.name.chars().next().is_some_and(char::is_uppercase) {
                    return;
                }
            }
            Some(AstKind::JSXMemberExpressionObject(_)) => {}
            _ => return,
        }
        let reference = Reference::new(
            ident.span,
            ident.name.to_compact_str(),
            self.current_node_id,
            ReferenceFlag::read(),
        );
        self.declare_reference(reference, true, SymbolFlags::Value);
    }

    fn is_not_expression_statement_parent(&self) -> bool {
        for node in self.nodes.iter_parents(self.current_node_id).skip(1) {
            return match node.kind() {
                AstKind::ParenthesizedExpression(_) => continue,
                AstKind::ExpressionStatement(_) => false,
                _ => true,
            };
        }
        false
    }
}
