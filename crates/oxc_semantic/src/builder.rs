//! Semantic Builder

use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind, Trivias, Visit};
use oxc_diagnostics::Error;
use oxc_span::{CompactStr, SourceType, Span};
use oxc_syntax::{
    identifier::is_identifier_name,
    module_record::{ExportImportName, ExportLocalName, ModuleRecord},
    operator::AssignmentOperator,
};

use crate::{
    binder::Binder,
    checker::{EarlyErrorJavaScript, EarlyErrorTypeScript},
    class::ClassTableBuilder,
    control_flow::{
        AssignmentValue, ControlFlowGraph, EdgeType, Register, StatementControlFlowType,
    },
    diagnostics::Redeclaration,
    jsdoc::JSDocBuilder,
    label::LabelBuilder,
    module_record::ModuleRecordBuilder,
    node::{AstNode, AstNodeId, AstNodes, NodeFlags},
    reference::{Reference, ReferenceFlag, ReferenceId},
    scope::{ScopeFlags, ScopeId, ScopeTree},
    symbol::{SymbolFlags, SymbolId, SymbolTable},
    Semantic,
};

pub struct SemanticBuilder<'a> {
    pub source_text: &'a str,

    pub source_type: SourceType,

    trivias: Rc<Trivias>,

    /// Semantic early errors such as redeclaration errors.
    errors: RefCell<Vec<Error>>,

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
    /// If true, the current node is in the type definition
    in_type_definition: bool,
    current_reference_flag: ReferenceFlag,

    // builders
    pub nodes: AstNodes<'a>,
    pub scope: ScopeTree,
    pub symbols: SymbolTable,

    pub(crate) module_record: Arc<ModuleRecord>,

    pub label_builder: LabelBuilder<'a>,

    jsdoc: JSDocBuilder<'a>,

    check_syntax_error: bool,

    pub cfg: ControlFlowGraph,

    pub class_table_builder: ClassTableBuilder,
}

pub struct SemanticBuilderReturn<'a> {
    pub semantic: Semantic<'a>,
    pub errors: Vec<Error>,
}

impl<'a> SemanticBuilder<'a> {
    pub fn new(source_text: &'a str, source_type: SourceType) -> Self {
        let scope = ScopeTree::default();
        let current_scope_id = scope.root_scope_id();

        let trivias = Rc::new(Trivias::default());
        Self {
            source_text,
            source_type,
            trivias: Rc::clone(&trivias),
            errors: RefCell::new(vec![]),
            current_node_id: AstNodeId::new(0),
            current_node_flags: NodeFlags::empty(),
            current_symbol_flags: SymbolFlags::empty(),
            in_type_definition: false,
            current_reference_flag: ReferenceFlag::empty(),
            current_scope_id,
            function_stack: vec![],
            namespace_stack: vec![],
            nodes: AstNodes::default(),
            scope,
            symbols: SymbolTable::default(),
            module_record: Arc::new(ModuleRecord::default()),
            label_builder: LabelBuilder::default(),
            jsdoc: JSDocBuilder::new(source_text, &trivias),
            check_syntax_error: false,
            cfg: ControlFlowGraph::new(),
            class_table_builder: ClassTableBuilder::new(),
        }
    }

    #[must_use]
    pub fn with_trivias(mut self, trivias: Trivias) -> Self {
        self.trivias = Rc::new(trivias);
        self.jsdoc = JSDocBuilder::new(self.source_text, &self.trivias);
        self
    }

    #[must_use]
    pub fn with_check_syntax_error(mut self, yes: bool) -> Self {
        self.check_syntax_error = yes;
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
            self.scope.add_scope(None, ScopeFlags::Top);
        } else {
            self.visit_program(program);

            // Checking syntax error on module record requires scope information from the previous AST pass
            if self.check_syntax_error {
                EarlyErrorJavaScript::check_module_record(&self);
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
            cfg: self.cfg,
        };
        SemanticBuilderReturn { semantic, errors: self.errors.into_inner() }
    }

    pub fn build2(self) -> Semantic<'a> {
        Semantic {
            source_text: self.source_text,
            source_type: self.source_type,
            trivias: self.trivias,
            nodes: self.nodes,
            scopes: self.scope,
            symbols: self.symbols,
            classes: self.class_table_builder.build(),
            module_record: Arc::new(ModuleRecord::default()),
            jsdoc: self.jsdoc.build(),
            unused_labels: self.label_builder.unused_node_ids,
            cfg: self.cfg,
        }
    }

    /// Push a Syntax Error
    pub fn error<T: Into<Error>>(&self, error: T) {
        self.errors.borrow_mut().push(error.into());
    }

    fn create_ast_node(&mut self, kind: AstKind<'a>) {
        let mut flags = self.current_node_flags;
        if self.jsdoc.retrieve_attached_jsdoc(&kind) {
            flags |= NodeFlags::JSDoc;
        }

        let ast_node = AstNode::new(kind, self.current_scope_id, self.cfg.current_node_ix, flags);
        let parent_node_id =
            if matches!(kind, AstKind::Program(_)) { None } else { Some(self.current_node_id) };
        self.current_node_id = self.nodes.add_node(ast_node, parent_node_id);
    }

    fn pop_ast_node(&mut self) {
        if let Some(parent_id) = self.nodes.parent_id(self.current_node_id) {
            self.current_node_id = parent_id;
        }
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
        let symbol_id = self.symbols.create_symbol(span, name, includes, scope_id);
        self.symbols.add_declaration(self.current_node_id);
        self.scope.add_binding(scope_id, CompactStr::from(name), symbol_id);
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
            self.error(Redeclaration(CompactStr::from(name), symbol_span, span));
        }
        Some(symbol_id)
    }

    pub fn declare_reference(&mut self, reference: Reference) -> ReferenceId {
        let reference_name = reference.name().clone();
        let reference_id = self.symbols.create_reference(reference);
        self.scope.add_unresolved_reference(self.current_scope_id, reference_name, reference_id);
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
        let symbol_id = self.symbols.create_symbol(span, name, includes, self.current_scope_id);
        self.symbols.add_declaration(self.current_node_id);
        self.scope.get_bindings_mut(scope_id).insert(CompactStr::from(name), symbol_id);
        symbol_id
    }

    fn resolve_references_for_current_scope(&mut self) {
        let all_references = self
            .scope
            .unresolved_references_mut(self.current_scope_id)
            .drain()
            .collect::<Vec<(_, Vec<_>)>>();

        let parent_scope_id =
            self.scope.get_parent_id(self.current_scope_id).unwrap_or(self.current_scope_id);

        for (name, reference_ids) in all_references {
            if let Some(symbol_id) = self.scope.get_binding(self.current_scope_id, &name) {
                for reference_id in &reference_ids {
                    self.symbols.references[*reference_id].set_symbol_id(symbol_id);
                }
                self.symbols.resolved_references[symbol_id].extend(reference_ids);
            } else {
                self.scope.extend_unresolved_reference(parent_scope_id, name, reference_ids);
            }
        }
    }

    pub fn add_redeclare_variable(&mut self, symbol_id: SymbolId, span: Span) {
        self.symbols.add_redeclare_variable(symbol_id, span);
    }

    fn add_export_flag_for_export_identifier(&mut self) {
        self.module_record.indirect_export_entries.iter().for_each(|entry| {
            if let ExportImportName::Name(name) = &entry.import_name {
                if let Some(symbol_id) = self.symbols.get_symbol_id_from_name(name.name()) {
                    self.symbols.union_flag(symbol_id, SymbolFlags::Export);
                }
            }
        });

        self.module_record.local_export_entries.iter().for_each(|entry| {
            match &entry.local_name {
                ExportLocalName::Name(name_span) => {
                    if let Some(symbol_id) = self.scope.get_root_binding(name_span.name()) {
                        self.symbols.union_flag(symbol_id, SymbolFlags::Export);
                    }
                }
                ExportLocalName::Default(_) => {
                    // export default identifier
                    //                ^^^^^^^^^^
                    let identifier = entry.span.source_text(self.source_text);
                    if is_identifier_name(identifier) {
                        if let Some(symbol_id) = self.scope.get_root_binding(identifier) {
                            self.symbols.union_flag(symbol_id, SymbolFlags::Export);
                        }
                    }
                }
                ExportLocalName::Null => {}
            }
        });
    }
}

impl<'a> Visit<'a> for SemanticBuilder<'a> {
    fn enter_scope(&mut self, flags: ScopeFlags) {
        let parent_scope_id =
            if flags.contains(ScopeFlags::Top) { None } else { Some(self.current_scope_id) };

        let mut flags = flags;
        // Inherit strict mode for functions
        // https://tc39.es/ecma262/#sec-strict-mode-code
        if let Some(parent_scope_id) = parent_scope_id {
            let mut strict_mode = self.scope.root_flags().is_strict_mode();
            let parent_scope_flags = self.scope.get_flags(parent_scope_id);

            if !strict_mode
                && parent_scope_flags.is_function()
                && parent_scope_flags.is_strict_mode()
            {
                strict_mode = true;
            }

            // inherit flags for non-function scopes
            if !flags.contains(ScopeFlags::Function) {
                flags |= parent_scope_flags & ScopeFlags::Modifiers;
            };

            if strict_mode {
                flags |= ScopeFlags::StrictMode;
            }
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
            EarlyErrorJavaScript::run(node, self);
            EarlyErrorTypeScript::run(node, self);
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
        self.enter_node(kind);

        /* cfg */
        let _program_basic_block = self.cfg.new_basic_block();
        /* cfg - must be above directives as directives are in cfg */

        for directive in &program.directives {
            self.visit_directive(directive);
        }

        self.visit_statements(&program.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_block_statement(&mut self, stmt: &BlockStatement<'a>) {
        let kind = AstKind::BlockStatement(self.alloc(stmt));
        self.enter_scope(ScopeFlags::empty());
        self.enter_node(kind);

        /* cfg */
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);
        /* cfg */

        self.visit_statements(&stmt.body);

        /* cfg */
        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            self.cfg.current_node_ix,
            None,
        );
        /* cfg */

        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_break_statement(&mut self, stmt: &BreakStatement<'a>) {
        let kind = AstKind::BreakStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);
        /* cfg */

        if let Some(break_target) = &stmt.label {
            self.visit_label_identifier(break_target);

            /* cfg */
            if let Some(label_found) =
                self.cfg.label_to_ast_node_ix.iter().rev().find(|x| x.0 == break_target.name)
            {
                let (_, break_, _) = self.cfg.ast_node_to_break_continue.iter().rev().find(|x| x.0 == label_found.1)
                                                .expect("expected a corresponding break/continue array for a found label owning ast node");
                self.cfg.basic_blocks_with_breaks[*break_].push(self.cfg.current_node_ix);
            } else {
                self.cfg
                    .basic_blocks_with_breaks
                    .last_mut()
                    .expect(
                        "expected there to be a stack of control flows that a break can belong to",
                    )
                    .push(self.cfg.current_node_ix);
            }
            /* cfg */
        }
        /* cfg */
        else {
            self.cfg
                .basic_blocks_with_breaks
                .last_mut()
                .expect("expected there to be a stack of control flows that a break can belong to")
                .push(self.cfg.current_node_ix);
        }
        self.cfg.put_unreachable();

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            self.cfg.current_node_ix,
            None,
        );
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_continue_statement(&mut self, stmt: &ContinueStatement<'a>) {
        let kind = AstKind::ContinueStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);
        /* cfg */

        if let Some(continue_target) = &stmt.label {
            self.visit_label_identifier(continue_target);
            /* cfg */
            if let Some(label_found) =
                self.cfg.label_to_ast_node_ix.iter().rev().find(|x| x.0 == continue_target.name)
            {
                let (_, _, continue_) = self.cfg.ast_node_to_break_continue.iter().rev().find(|x| x.0 == label_found.1)
                                        .expect("expected a corresponding break/continue array for a found label owning ast node");
                if let Some(continue_) = continue_ {
                    self.cfg.basic_blocks_with_breaks[*continue_].push(self.cfg.current_node_ix);
                } else {
                    self.cfg
                    .basic_blocks_with_breaks
                    .last_mut()
                    .expect(
                        "expected there to be a stack of control flows that a break can belong to",
                    )
                    .push(self.cfg.current_node_ix);
                }
            } else {
                self.cfg
                    .basic_blocks_with_breaks
                    .last_mut()
                    .expect(
                        "expected there to be a stack of control flows that a break can belong to",
                    )
                    .push(self.cfg.current_node_ix);
            }
            /* cfg */
        }
        /* cfg */
        else {
            self.cfg
                .basic_blocks_with_breaks
                .last_mut()
                .expect("expected there to be a stack of control flows that a break can belong to")
                .push(self.cfg.current_node_ix);
        }
        self.cfg.put_unreachable();
        /* cfg */

        /* cfg */
        let current_node_ix = self.cfg.current_node_ix;
        // todo: assert on this instead when continues which
        // aren't in iterations are nonrecoverable errors
        if let Some(continues) = self.cfg.basic_blocks_with_continues.last_mut() {
            continues.push(current_node_ix);
        }
        self.cfg.put_unreachable();

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            self.cfg.current_node_ix,
            None,
        );
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_debugger_statement(&mut self, stmt: &DebuggerStatement) {
        let kind = AstKind::DebuggerStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        // just take the next_label since it should be taken by the next
        // statement regardless of whether the statement can use it or not
        self.cfg.next_label.take();
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_do_while_statement(&mut self, stmt: &DoWhileStatement<'a>) {
        let kind = AstKind::DoWhileStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let before_do_while_stmt_graph_ix = self.cfg.current_node_ix;
        let start_body_graph_ix = self.cfg.new_basic_block();
        let statement_state =
            self.cfg.before_statement(self.current_node_id, StatementControlFlowType::UsesContinue);
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg - condition basic block */
        let start_of_condition_graph_ix = self.cfg.new_basic_block();
        /* cfg */

        self.visit_expression(&stmt.test);

        /* cfg */
        let end_of_condition_graph_ix = self.cfg.current_node_ix;

        let end_do_while_graph_ix = self.cfg.new_basic_block();

        // before do while to start of condition basic block
        self.cfg.add_edge(
            before_do_while_stmt_graph_ix,
            start_of_condition_graph_ix,
            EdgeType::Normal,
        );
        // body of do-while to start of condition
        self.cfg.add_edge(start_body_graph_ix, start_of_condition_graph_ix, EdgeType::Backedge);
        // end of condition to after do while
        self.cfg.add_edge(end_of_condition_graph_ix, end_do_while_graph_ix, EdgeType::Normal);
        // end of condition to after start of body
        self.cfg.add_edge(end_of_condition_graph_ix, start_body_graph_ix, EdgeType::Normal);

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            // all basic blocks are break here so we connect them to the
            // basic block after the do-while statement
            end_do_while_graph_ix,
            // all basic blocks are continues here so we connect them to the
            // basic block of the condition
            Some(start_of_condition_graph_ix),
        );
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
        let kind = AstKind::ExpressionStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);
        /* cfg */

        self.visit_expression(&stmt.expression);

        /* cfg */
        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            self.cfg.current_node_ix,
            None,
        );
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
        let left_expr_end_ix = self.cfg.current_node_ix;
        let right_expr_start_ix = self.cfg.new_basic_block();
        /* cfg  */

        self.visit_expression(&expr.right);

        /* cfg */
        let right_expr_end_ix = self.cfg.current_node_ix;
        let after_logical_expr_ix = self.cfg.new_basic_block();

        self.cfg.add_edge(left_expr_end_ix, right_expr_start_ix, EdgeType::Normal);
        self.cfg.add_edge(left_expr_end_ix, after_logical_expr_ix, EdgeType::Normal);
        self.cfg.add_edge(right_expr_end_ix, after_logical_expr_ix, EdgeType::Normal);
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
        let cfg_ixs = if expr.operator.is_logical() {
            let target_end_ix = self.cfg.current_node_ix;
            let expr_start_ix = self.cfg.new_basic_block();
            Some((target_end_ix, expr_start_ix))
        } else {
            None
        };
        /* cfg  */

        self.visit_expression(&expr.right);

        /* cfg */
        if let Some((target_end_ix, expr_start_ix)) = cfg_ixs {
            let expr_end_ix = self.cfg.current_node_ix;
            let after_assignment_ix = self.cfg.new_basic_block();

            self.cfg.add_edge(target_end_ix, expr_start_ix, EdgeType::Normal);
            self.cfg.add_edge(target_end_ix, after_assignment_ix, EdgeType::Normal);
            self.cfg.add_edge(expr_end_ix, after_assignment_ix, EdgeType::Normal);
        }
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_for_statement(&mut self, stmt: &ForStatement<'a>) {
        let kind = AstKind::ForStatement(self.alloc(stmt));
        let is_lexical_declaration =
            stmt.init.as_ref().is_some_and(ForStatementInit::is_lexical_declaration);
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
        }
        self.enter_node(kind);
        if let Some(init) = &stmt.init {
            self.visit_for_statement_init(init);
        }
        /* cfg */
        let before_for_graph_ix = self.cfg.current_node_ix;
        let test_graph_ix = self.cfg.new_basic_block();
        /* cfg */
        if let Some(test) = &stmt.test {
            self.visit_expression(test);
        }

        /* cfg */
        let update_graph_ix = self.cfg.new_basic_block();
        /* cfg */

        if let Some(update) = &stmt.update {
            self.visit_expression(update);
        }

        /* cfg */
        let body_graph_ix = self.cfg.new_basic_block();
        let statement_state =
            self.cfg.before_statement(self.current_node_id, StatementControlFlowType::UsesContinue);
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg */
        let after_for_stmt = self.cfg.new_basic_block();
        self.cfg.add_edge(before_for_graph_ix, test_graph_ix, EdgeType::Normal);
        self.cfg.add_edge(test_graph_ix, body_graph_ix, EdgeType::Normal);
        self.cfg.add_edge(body_graph_ix, update_graph_ix, EdgeType::Backedge);
        self.cfg.add_edge(update_graph_ix, test_graph_ix, EdgeType::Backedge);
        self.cfg.add_edge(test_graph_ix, after_for_stmt, EdgeType::Normal);

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            // all basic blocks are break here so we connect them to the
            // basic block after the for statement
            self.cfg.current_node_ix,
            // all basic blocks are continues here so we connect them to the
            // basic block of the condition
            Some(test_graph_ix),
        );

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
            ForStatementInit::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_for_in_statement(&mut self, stmt: &ForInStatement<'a>) {
        let kind = AstKind::ForInStatement(self.alloc(stmt));
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
        }
        self.enter_node(kind);
        self.visit_for_statement_left(&stmt.left);

        /* cfg */
        let before_for_stmt_graph_ix = self.cfg.current_node_ix;
        let start_prepare_cond_graph_ix = self.cfg.new_basic_block();
        /* cfg */

        self.visit_expression(&stmt.right);

        /* cfg */
        let end_of_prepare_cond_graph_ix = self.cfg.current_node_ix;
        // this basic block is always empty since there's no update condition in a for-in loop.
        let basic_block_with_backedge_graph_ix = self.cfg.new_basic_block();
        let body_graph_ix = self.cfg.new_basic_block();
        let statement_state =
            self.cfg.before_statement(self.current_node_id, StatementControlFlowType::UsesContinue);
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg */
        let end_of_body_graph_ix = self.cfg.current_node_ix;
        let after_for_graph_ix = self.cfg.new_basic_block();
        // connect before for statement to the iterable expression
        self.cfg.add_edge(before_for_stmt_graph_ix, start_prepare_cond_graph_ix, EdgeType::Normal);
        // connect the end of the iterable expression to the basic block with back edge
        self.cfg.add_edge(
            end_of_prepare_cond_graph_ix,
            basic_block_with_backedge_graph_ix,
            EdgeType::Normal,
        );
        // connect the basic block with back edge to the start of the body
        self.cfg.add_edge(basic_block_with_backedge_graph_ix, body_graph_ix, EdgeType::Normal);
        // connect the end of the body back to the basic block
        // with back edge for the next iteration
        self.cfg.add_edge(
            end_of_body_graph_ix,
            basic_block_with_backedge_graph_ix,
            EdgeType::Backedge,
        );
        // connect the basic block with back edge to the basic block after the for loop
        // for when there are no more iterations left in the iterable
        self.cfg.add_edge(basic_block_with_backedge_graph_ix, after_for_graph_ix, EdgeType::Normal);

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            // all basic blocks are break here so we connect them to the
            // basic block after the for-in statement
            self.cfg.current_node_ix,
            // all basic blocks are continues here so we connect them to the
            // basic block of the condition
            Some(basic_block_with_backedge_graph_ix),
        );
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
        }
        self.enter_node(kind);
        self.visit_for_statement_left(&stmt.left);

        /* cfg */
        let before_for_stmt_graph_ix = self.cfg.current_node_ix;
        let start_prepare_cond_graph_ix = self.cfg.new_basic_block();
        /* cfg */

        self.visit_expression(&stmt.right);

        /* cfg */
        let end_of_prepare_cond_graph_ix = self.cfg.current_node_ix;
        // this basic block is always empty since there's no update condition in a for-of loop.
        let basic_block_with_backedge_graph_ix = self.cfg.new_basic_block();
        let body_graph_ix = self.cfg.new_basic_block();
        let statement_state =
            self.cfg.before_statement(self.current_node_id, StatementControlFlowType::UsesContinue);
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg */
        let end_of_body_graph_ix = self.cfg.current_node_ix;
        let after_for_graph_ix = self.cfg.new_basic_block();
        // connect before for statement to the iterable expression
        self.cfg.add_edge(before_for_stmt_graph_ix, start_prepare_cond_graph_ix, EdgeType::Normal);
        // connect the end of the iterable expression to the basic block with back edge
        self.cfg.add_edge(
            end_of_prepare_cond_graph_ix,
            basic_block_with_backedge_graph_ix,
            EdgeType::Normal,
        );
        // connect the basic block with back edge to the start of the body
        self.cfg.add_edge(basic_block_with_backedge_graph_ix, body_graph_ix, EdgeType::Normal);
        // connect the end of the body back to the basic block
        // with back edge for the next iteration
        self.cfg.add_edge(
            end_of_body_graph_ix,
            basic_block_with_backedge_graph_ix,
            EdgeType::Backedge,
        );
        // connect the basic block with back edge to the basic block after the for loop
        // for when there are no more iterations left in the iterable
        self.cfg.add_edge(basic_block_with_backedge_graph_ix, after_for_graph_ix, EdgeType::Normal);

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            // all basic blocks are break here so we connect them to the
            // basic block after the for-of statement
            self.cfg.current_node_ix,
            // all basic blocks are continues here so we connect them to the
            // basic block of the condition
            Some(basic_block_with_backedge_graph_ix),
        );
        /* cfg */

        self.leave_node(kind);
        if is_lexical_declaration {
            self.leave_scope();
        }
    }

    fn visit_if_statement(&mut self, stmt: &IfStatement<'a>) {
        let kind = AstKind::IfStatement(self.alloc(stmt));
        self.enter_node(kind);

        self.visit_expression(&stmt.test);

        /* cfg */
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);

        let before_if_stmt_graph_ix = self.cfg.current_node_ix;

        // if statement basic block
        let before_consequent_stmt_graph_ix = self.cfg.new_basic_block();
        /* cfg */

        self.visit_statement(&stmt.consequent);

        /* cfg */
        let after_consequent_stmt_graph_ix = self.cfg.current_node_ix;
        /* cfg */

        let else_graph_ix = if let Some(alternate) = &stmt.alternate {
            /* cfg */
            let else_graph_ix = self.cfg.new_basic_block();
            /* cfg */

            self.visit_statement(alternate);

            Some((else_graph_ix, self.cfg.current_node_ix))
        } else {
            None
        };

        /* cfg - bb after if statement joins consequent and alternate */
        let after_if_graph_ix = self.cfg.new_basic_block();

        if stmt.alternate.is_some() {
            self.cfg.put_unreachable();
        }
        //  else {
        self.cfg.add_edge(after_consequent_stmt_graph_ix, after_if_graph_ix, EdgeType::Normal);
        // }

        self.cfg.add_edge(
            before_if_stmt_graph_ix,
            before_consequent_stmt_graph_ix,
            EdgeType::Normal,
        );

        if let Some((start_of_alternate_stmt_graph_ix, after_alternate_stmt_graph_ix)) =
            else_graph_ix
        {
            self.cfg.add_edge(
                before_if_stmt_graph_ix,
                start_of_alternate_stmt_graph_ix,
                EdgeType::Normal,
            );
            self.cfg.add_edge(after_alternate_stmt_graph_ix, after_if_graph_ix, EdgeType::Normal);
        } else {
            self.cfg.add_edge(before_if_stmt_graph_ix, after_if_graph_ix, EdgeType::Normal);
        }

        self.cfg.after_statement(&statement_state, self.current_node_id, after_if_graph_ix, None);
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_labeled_statement(&mut self, stmt: &LabeledStatement<'a>) {
        let kind = AstKind::LabeledStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);
        /* cfg */

        self.visit_label_identifier(&stmt.label);

        /* cfg */
        self.cfg.next_label = Some(stmt.label.name.to_compact_str());
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg */
        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            self.cfg.current_node_ix,
            None,
        );

        /* cfg */

        self.leave_node(kind);
    }

    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        let kind = AstKind::ReturnStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);

        // returning something is an assignment to the return register
        self.cfg.use_this_register = Some(Register::Return);
        /* cfg */

        if let Some(arg) = &stmt.argument {
            self.visit_expression(arg);
            /* cfg */
            self.cfg.put_x_in_register(AssignmentValue::NotImplicitUndefined);
            /* cfg */
        }
        /* cfg - put implicit undefined as return arg  */
        else {
            self.cfg.put_undefined();
        }
        /* cfg */

        /* cfg - put unreachable after return */
        let _ = self.cfg.new_basic_block();
        self.cfg.put_unreachable();

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            self.cfg.current_node_ix,
            None,
        );
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_switch_statement(&mut self, stmt: &SwitchStatement<'a>) {
        let kind = AstKind::SwitchStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&stmt.discriminant);
        self.enter_scope(ScopeFlags::empty());

        /* cfg */
        let discriminant_graph_ix = self.cfg.current_node_ix;
        self.cfg.switch_case_conditions.push(vec![]);
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);
        let mut ends_of_switch_cases = vec![];
        /* cfg */

        for case in &stmt.cases {
            self.visit_switch_case(case);
            ends_of_switch_cases.push(self.cfg.current_node_ix);
        }

        /* cfg */
        let switch_case_conditions = self.cfg.switch_case_conditions.pop().expect(
            "there must be a corresponding previous_switch_case_last_block in a switch statement",
        );

        // for each switch case
        for i in 0..switch_case_conditions.len() {
            let switch_case_condition_graph_ix = switch_case_conditions[i];

            // every switch case condition can be skipped,
            // so there's a possible jump from it to the next switch case condition
            for y in switch_case_conditions.iter().skip(i + 1) {
                self.cfg.add_edge(switch_case_condition_graph_ix, *y, EdgeType::Normal);
            }

            // connect the end of each switch statement to
            // the condition of the next switch statement
            if switch_case_conditions.len() > i + 1 {
                let end_of_switch_case = ends_of_switch_cases[i];
                let next_switch_statement_condition = switch_case_conditions[i + 1];

                self.cfg.add_edge(
                    end_of_switch_case,
                    next_switch_statement_condition,
                    EdgeType::Normal,
                );
            }

            self.cfg.add_edge(
                discriminant_graph_ix,
                switch_case_condition_graph_ix,
                EdgeType::Normal,
            );
        }

        if let Some(last) = switch_case_conditions.last() {
            self.cfg.add_edge(*last, self.cfg.current_node_ix, EdgeType::Normal);
        }

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            self.cfg.current_node_ix,
            None,
        );
        /* cfg */

        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_switch_case(&mut self, case: &SwitchCase<'a>) {
        let kind = AstKind::SwitchCase(self.alloc(case));
        self.enter_node(kind);

        /* cfg */
        // make a new basic block so that we can jump to it later from the switch
        //   discriminant and the switch cases above it (if they don't test ss true)
        let switch_cond_graph_ix = self.cfg.new_basic_block();
        self.cfg
            .switch_case_conditions
            .last_mut()
            .expect("there must be a switch_case_conditions while in a switch case")
            .push(switch_cond_graph_ix);
        /* cfg */

        if let Some(expr) = &case.test {
            self.visit_expression(expr);
        }

        /* cfg */
        let statements_in_switch_graph_ix = self.cfg.new_basic_block();
        self.cfg.add_edge(switch_cond_graph_ix, statements_in_switch_graph_ix, EdgeType::Normal);
        /* cfg */

        self.visit_statements(&case.consequent);

        self.leave_node(kind);
    }

    fn visit_throw_statement(&mut self, stmt: &ThrowStatement<'a>) {
        let kind = AstKind::ThrowStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg */
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);
        let throw_expr = self.cfg.new_register();
        self.cfg.use_this_register = Some(throw_expr);
        /* cfg */

        self.visit_expression(&stmt.argument);
        // todo - put unreachable after throw statement

        /* cfg */
        self.cfg.put_throw(throw_expr);
        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            self.cfg.current_node_ix,
            None,
        );
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_try_statement(&mut self, stmt: &TryStatement<'a>) {
        let kind = AstKind::TryStatement(self.alloc(stmt));
        self.enter_node(kind);

        // There are 3 possible kinds of Try Statements (See
        //    <https://tc39.es/ecma262/#sec-try-statement>):
        // 1. try-catch
        // 2. try-finally
        // 3. try-catch-finally
        //
        // We will consider each kind of try statement separately.
        //
        // For a try-catch, there are only 2 ways to reach
        // the outgoing node (after the entire statement):
        //
        // 1. after the try block completing successfully
        // 2. after the catch block completing successfully,
        //    in which case some statement in the try block
        //    must have thrown.
        //
        // For a try-finally, there is only 1 way to reach
        // the outgoing node, whereby:
        // - the try block completed successfully, and
        // - the finally block completed successfully
        //
        // But the finally block can also be reached when the try
        // fails. We thus need to fork the control flow graph into
        // 2 different finally statements:
        //    1. one where the try block completes successfully, (finally_succ)
        //    2. one where some statement in the try block throws (finally_err)
        // Only the end of the try block will have an incoming edge to the
        // finally_succ, and only finally_succ will have an outgoing node to
        // the next statement.
        //
        // For a try-catch-finally, we have seemlingly more cases:
        //   1. after the try block completing successfully
        //   2. after the catch block completing successfully
        //   3. after the try block if the catch block throws
        // Despite having 3 distings scenarios, we can simplify the control flow
        // graph by still only using a finally_succ and a finally_err node.
        // The key is that the outgoing edge going past the entire
        // try-catch-finally statement is guaranteed that all code paths have
        // either completed the try block or the catch block in full.

        // Implementation notes:
        // We will use the following terminology:
        //
        // the "parent after_throw block" is the block that would be the target
        // of a throw if there were no try-catch-finally.
        //
        // Within the try block, a throw will not go to the parent after_throw
        // block. Instead, it will go to the catch block in a try-catch or to
        // the finally_err block in a try-catch-finally.
        //
        // In a catch block, a throw will go to the finally_err block in a
        // try-catch-finally, or to the parent after_throw block in a basic
        // try-catch.
        //
        // In a finally block, a throw will always go to the parent after_throw
        // block, both for finally_succ and finally_err.

        /* cfg */
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);

        // TODO: support unwinding finally/catch blocks that aren't in this function
        // even if something throws.
        let parent_after_throw_block_ix = self.cfg.after_throw_block;

        let try_stmt_pre_start_ix = self.cfg.current_node_ix;

        let try_stmt_start_ix = self.cfg.new_basic_block();
        self.cfg.add_edge(try_stmt_pre_start_ix, try_stmt_start_ix, EdgeType::Normal);
        let try_after_throw_block_ix = self.cfg.new_basic_block();

        self.cfg.current_node_ix = try_stmt_start_ix;

        // every statement created with this active adds an edge from that node to this node
        //
        // NOTE: we oversimplify here, realistically even in between basic blocks we
        // do throwsy things which could cause problems, but for the most part simply
        // pointing the end of every basic block to the catch block is enough
        self.cfg.after_throw_block = Some(try_after_throw_block_ix);
        // The one case that needs to be handled specially is if the first statement in the
        // try block throws. In that case, it is not sufficient to rely on an edge after that
        // statement, because the catch will run before that edge is taken.
        self.cfg.add_edge(try_stmt_pre_start_ix, try_after_throw_block_ix, EdgeType::Normal);
        /* cfg */

        self.visit_block_statement(&stmt.block);

        /* cfg */
        let end_of_try_block_ix = self.cfg.current_node_ix;
        self.cfg.add_edge(end_of_try_block_ix, try_after_throw_block_ix, EdgeType::Normal);
        self.cfg.after_throw_block = parent_after_throw_block_ix;

        let start_of_finally_err_block_ix = if stmt.finalizer.is_some() {
            if stmt.handler.is_some() {
                // try-catch-finally
                Some(self.cfg.new_basic_block())
            } else {
                // try-finally
                Some(try_after_throw_block_ix)
            }
        } else {
            // try-catch
            None
        };
        /* cfg */

        let catch_block_end_ix = if let Some(handler) = &stmt.handler {
            /* cfg */
            let catch_after_throw_block_ix = if stmt.finalizer.is_some() {
                start_of_finally_err_block_ix
            } else {
                parent_after_throw_block_ix
            };
            self.cfg.after_throw_block = catch_after_throw_block_ix;

            let catch_block_start_ix = try_after_throw_block_ix;
            self.cfg.current_node_ix = catch_block_start_ix;

            if let Some(catch_after_throw_block_ix) = catch_after_throw_block_ix {
                self.cfg.add_edge(
                    catch_block_start_ix,
                    catch_after_throw_block_ix,
                    EdgeType::Normal,
                );
            }
            /* cfg */

            self.visit_catch_clause(handler);

            /* cfg */
            Some(self.cfg.current_node_ix)
            /* cfg */
        } else {
            None
        };

        // Restore the after_throw_block
        self.cfg.after_throw_block = parent_after_throw_block_ix;

        if let Some(finalizer) = &stmt.finalizer {
            /* cfg */
            let finally_err_block_start_ix =
                start_of_finally_err_block_ix.expect("this try statement has a finally_err block");

            self.cfg.current_node_ix = finally_err_block_start_ix;
            /* cfg */

            self.visit_finally_clause(finalizer);

            /* cfg */
            // put an unreachable after the finally_err block
            self.cfg.put_unreachable();

            let finally_succ_block_start_ix = self.cfg.new_basic_block();

            // The end_of_try_block has an outgoing edge to finally_succ also
            // for when the try block completes successfully.
            self.cfg.add_edge(end_of_try_block_ix, finally_succ_block_start_ix, EdgeType::Normal);

            // The end_of_catch_block has an outgoing edge to finally_succ for
            // when the catch block in a try-catch-finally completes successfully.
            if let Some(end_of_catch_block_ix) = catch_block_end_ix {
                // try-catch-finally
                self.cfg.add_edge(
                    end_of_catch_block_ix,
                    finally_succ_block_start_ix,
                    EdgeType::Normal,
                );
            }
            /* cfg */

            self.visit_finally_clause(finalizer);
        }

        /* cfg */
        let try_statement_block_end_ix = self.cfg.current_node_ix;
        let after_try_statement_block_ix = self.cfg.new_basic_block();
        self.cfg.add_edge(
            try_statement_block_end_ix,
            after_try_statement_block_ix,
            EdgeType::Normal,
        );

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            self.cfg.current_node_ix,
            None,
        );
        /* cfg */

        self.leave_node(kind);
    }

    fn visit_while_statement(&mut self, stmt: &WhileStatement<'a>) {
        let kind = AstKind::WhileStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg - condition basic block */
        let before_while_stmt_graph_ix = self.cfg.current_node_ix;
        let condition_graph_ix = self.cfg.new_basic_block();
        /* cfg */

        self.visit_expression(&stmt.test);

        /* cfg - body basic block */
        let body_graph_ix = self.cfg.new_basic_block();
        let statement_state =
            self.cfg.before_statement(self.current_node_id, StatementControlFlowType::UsesContinue);
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg - after body basic block */
        let after_body_graph_ix = self.cfg.new_basic_block();

        self.cfg.add_edge(before_while_stmt_graph_ix, condition_graph_ix, EdgeType::Normal);
        self.cfg.add_edge(condition_graph_ix, body_graph_ix, EdgeType::Normal);
        self.cfg.add_edge(body_graph_ix, after_body_graph_ix, EdgeType::Normal);
        self.cfg.add_edge(body_graph_ix, condition_graph_ix, EdgeType::Backedge);
        self.cfg.add_edge(condition_graph_ix, after_body_graph_ix, EdgeType::Normal);

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            // all basic blocks are break here so we connect them to the
            // basic block after the while statement
            after_body_graph_ix,
            // all basic blocks are continues here so we connect them to the
            // basic block of the condition
            Some(condition_graph_ix),
        );
        /* cfg */
        self.leave_node(kind);
    }

    fn visit_with_statement(&mut self, stmt: &WithStatement<'a>) {
        let kind = AstKind::WithStatement(self.alloc(stmt));
        self.enter_node(kind);

        /* cfg - condition basic block */
        let before_with_stmt_graph_ix = self.cfg.current_node_ix;
        let statement_state = self
            .cfg
            .before_statement(self.current_node_id, StatementControlFlowType::DoesNotUseContinue);
        let condition_graph_ix = self.cfg.new_basic_block();
        /* cfg */

        self.visit_expression(&stmt.object);

        /* cfg - body basic block */
        let body_graph_ix = self.cfg.new_basic_block();
        /* cfg */

        self.visit_statement(&stmt.body);

        /* cfg - after body basic block */
        let after_body_graph_ix = self.cfg.new_basic_block();

        self.cfg.add_edge(before_with_stmt_graph_ix, condition_graph_ix, EdgeType::Normal);
        self.cfg.add_edge(condition_graph_ix, body_graph_ix, EdgeType::Normal);
        self.cfg.add_edge(body_graph_ix, after_body_graph_ix, EdgeType::Normal);
        self.cfg.add_edge(condition_graph_ix, after_body_graph_ix, EdgeType::Normal);

        self.cfg.after_statement(
            &statement_state,
            self.current_node_id,
            self.cfg.current_node_ix,
            None,
        );
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

        /* cfg */
        let preserved = self.cfg.preserve_expression_state();

        let before_function_graph_ix = self.cfg.current_node_ix;
        let function_graph_ix = self.cfg.new_basic_block_for_function();
        /* cfg */

        // We add a new basic block to the cfg before entering the node
        // so that the correct cfg_ix is associated with the ast node.
        self.enter_node(kind);

        /* cfg */
        self.cfg.add_edge(before_function_graph_ix, function_graph_ix, EdgeType::NewFunction);
        /* cfg */

        if let Some(ident) = &func.id {
            self.visit_binding_identifier(ident);
        }
        self.visit_formal_parameters(&func.params);
        if let Some(body) = &func.body {
            self.visit_function_body(body);
        }

        /* cfg */
        self.cfg.restore_expression_state(preserved);
        let after_function_graph_ix = self.cfg.new_basic_block();
        self.cfg.add_edge(before_function_graph_ix, after_function_graph_ix, EdgeType::Normal);
        // self.cfg.put_x_in_register(AssignmentValue::Function(self.current_node_id));
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
        }

        self.enter_node(kind);

        /* cfg */
        let preserved = self.cfg.preserve_expression_state();
        self.cfg.store_final_assignments_into_this_array.push(vec![]);
        /* cfg */

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

        /* cfg */
        let _elements = self.cfg.store_final_assignments_into_this_array.pop().expect(
            "expected there to be atleast one vec in the store_final_assignments_into_this_arrays",
        );
        self.cfg.restore_expression_state(preserved);
        self.cfg.spread_indices.push(vec![]);
        // self.cfg.put_collection_in_register(self.current_node_id, CollectionType::Class, elements);
        /* cfg */

        self.leave_node(kind);
        if is_class_expr {
            self.leave_scope();
        }
    }

    fn visit_arrow_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        let kind = AstKind::ArrowFunctionExpression(self.alloc(expr));
        self.enter_scope(ScopeFlags::Function | ScopeFlags::Arrow);

        /* cfg */
        let preserved = self.cfg.preserve_expression_state();
        let current_node_ix = self.cfg.current_node_ix;
        let function_graph_ix = self.cfg.new_basic_block_for_function();
        /* cfg */

        // We add a new basic block to the cfg before entering the node
        // so that the correct cfg_ix is associated with the ast node.
        self.enter_node(kind);

        self.visit_formal_parameters(&expr.params);

        /* cfg */
        self.cfg.add_edge(current_node_ix, function_graph_ix, EdgeType::NewFunction);
        if expr.expression {
            self.cfg.store_assignments_into_this_array.push(vec![]);
            self.cfg.use_this_register = Some(Register::Return);
        }
        /* cfg */
        self.visit_function_body(&expr.body);

        /* cfg */
        self.cfg.restore_expression_state(preserved);
        self.cfg.current_node_ix = current_node_ix;
        // self.cfg.put_x_in_register(AssignmentValue::Function(self.current_node_id));
        /* cfg */
        if let Some(parameters) = &expr.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.leave_node(kind);
        self.leave_scope();
    }
}

impl<'a> SemanticBuilder<'a> {
    fn enter_kind(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::ExportDefaultDeclaration(_) | AstKind::ExportNamedDeclaration(_) => {
                self.current_symbol_flags |= SymbolFlags::Export;
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
            AstKind::CatchClause(clause) => {
                clause.bind(self);
            }
            AstKind::TSModuleDeclaration(module_declaration) => {
                module_declaration.bind(self);
                let symbol_id = self
                    .scope
                    .get_bindings(self.current_scope_id)
                    .get(module_declaration.id.name().as_str());
                self.namespace_stack.push(*symbol_id.unwrap());
                self.in_type_definition = true;
            }
            AstKind::TSTypeAliasDeclaration(type_alias_declaration) => {
                type_alias_declaration.bind(self);
                self.in_type_definition = true;
            }
            AstKind::TSInterfaceDeclaration(interface_declaration) => {
                interface_declaration.bind(self);
                self.in_type_definition = true;
            }
            AstKind::TSEnumDeclaration(enum_declaration) => {
                enum_declaration.bind(self);
                // TODO: const enum?
                self.make_all_namespaces_valuelike();
                self.in_type_definition = true;
            }
            AstKind::TSTypeParameterInstantiation(_) | AstKind::TSTypeAnnotation(_) => {
                self.in_type_definition = true;
            }
            AstKind::TSEnumMember(enum_member) => {
                enum_member.bind(self);
            }
            AstKind::TSTypeParameter(type_parameter) => {
                type_parameter.bind(self);
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
                self.set_function_node_flag(NodeFlags::HasYield);
            }
            _ => {}
        }
    }

    #[allow(clippy::single_match)]
    fn leave_kind(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Program(_) => {
                self.add_export_flag_for_export_identifier();
            }
            AstKind::Class(_) => {
                self.current_node_flags -= NodeFlags::Class;
                self.class_table_builder.pop_class();
            }
            AstKind::ExportDefaultDeclaration(_) | AstKind::ExportNamedDeclaration(_) => {
                self.current_symbol_flags -= SymbolFlags::Export;
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
            AstKind::TSModuleBlock(_) => {
                self.namespace_stack.pop();
            }
            AstKind::TSEnumDeclaration(_)
            | AstKind::TSTypeAliasDeclaration(_)
            | AstKind::TSInterfaceDeclaration(_)
            | AstKind::TSModuleDeclaration(_)
            | AstKind::TSTypeParameterInstantiation(_)
            | AstKind::TSTypeAnnotation(_) => {
                self.in_type_definition = false;
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
            AstKind::AssignmentTarget(_) => self.current_reference_flag -= ReferenceFlag::Write,
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
        let reference =
            Reference::new(ident.span, ident.name.to_compact_str(), self.current_node_id, flag);
        let reference_id = self.declare_reference(reference);
        ident.reference_id.set(Some(reference_id));
    }

    /// Resolve reference flags for the current ast node.
    fn resolve_reference_usages(&self) -> ReferenceFlag {
        if self.in_type_definition {
            ReferenceFlag::Type
        } else if self.current_reference_flag.is_write() {
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
        self.declare_reference(reference);
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
