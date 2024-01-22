//! Semantic Builder

use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

use itertools::Itertools;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind, Trivias, TriviasMap, Visit};
use oxc_diagnostics::Error;
use oxc_span::{Atom, SourceType, Span};
use oxc_syntax::{module_record::ModuleRecord, operator::AssignmentOperator};
use rustc_hash::FxHashMap;

use crate::{
    binder::Binder,
    checker::{EarlyErrorJavaScript, EarlyErrorTypeScript},
    class::ClassTableBuilder,
    diagnostics::Redeclaration,
    jsdoc::JSDocBuilder,
    module_record::ModuleRecordBuilder,
    node::{AstNode, AstNodeId, AstNodes, NodeFlags},
    reference::{Reference, ReferenceFlag, ReferenceId},
    scope::{ScopeFlags, ScopeId, ScopeTree},
    symbol::{SymbolFlags, SymbolId, SymbolTable},
    Semantic,
};

pub struct LabeledScope<'a> {
    name: &'a str,
    used: bool,
    parent: usize,
}

struct UnusedLabels<'a> {
    scopes: Vec<LabeledScope<'a>>,
    curr_scope: usize,
    labels: Vec<AstNodeId>,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub span: Span,
    pub symbol_id: SymbolId,
}

#[derive(Debug)]
pub struct RedeclareVariables {
    pub variables: Vec<VariableInfo>,
}

pub struct SemanticBuilder<'a> {
    pub source_text: &'a str,

    pub source_type: SourceType,

    trivias: Rc<TriviasMap>,

    /// Semantic early errors such as redeclaration errors.
    errors: RefCell<Vec<Error>>,

    // states
    pub current_node_id: AstNodeId,
    pub current_node_flags: NodeFlags,
    pub current_symbol_flags: SymbolFlags,
    pub current_scope_id: ScopeId,
    /// Stores current `AstKind::Function` and `AstKind::ArrowExpression` during AST visit
    pub function_stack: Vec<AstNodeId>,
    // To make a namespace/module value like
    // we need the to know the modules we are inside
    // and when we reach a value declaration we set it
    // to value like
    pub namespace_stack: Vec<SymbolId>,
    /// If true, the current node is in the type definition
    in_type_definition: bool,

    // builders
    pub nodes: AstNodes<'a>,
    pub scope: ScopeTree,
    pub symbols: SymbolTable,

    pub(crate) module_record: Arc<ModuleRecord>,

    unused_labels: UnusedLabels<'a>,

    jsdoc: JSDocBuilder<'a>,

    check_syntax_error: bool,

    redeclare_variables: RedeclareVariables,
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

        let trivias = Rc::new(TriviasMap::default());
        Self {
            source_text,
            source_type,
            trivias: Rc::clone(&trivias),
            errors: RefCell::new(vec![]),
            current_node_id: AstNodeId::new(0),
            current_node_flags: NodeFlags::empty(),
            current_symbol_flags: SymbolFlags::empty(),
            in_type_definition: false,
            current_scope_id,
            function_stack: vec![],
            namespace_stack: vec![],
            nodes: AstNodes::default(),
            scope,
            symbols: SymbolTable::default(),
            module_record: Arc::new(ModuleRecord::default()),
            unused_labels: UnusedLabels { scopes: vec![], curr_scope: 0, labels: vec![] },
            jsdoc: JSDocBuilder::new(source_text, &trivias),
            check_syntax_error: false,
            redeclare_variables: RedeclareVariables { variables: vec![] },
            class_table_builder: ClassTableBuilder::new(),
        }
    }

    #[must_use]
    pub fn with_trivias(mut self, trivias: Trivias) -> Self {
        self.trivias = Rc::new(TriviasMap::from(trivias));
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
        program: &'a Program<'a>,
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
            unused_labels: self.unused_labels.labels,
            redeclare_variables: self.redeclare_variables.variables,
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
            unused_labels: self.unused_labels.labels,
            redeclare_variables: self.redeclare_variables.variables,
        }
    }

    /// Push a Syntax Error
    pub fn error<T: Into<Error>>(&self, error: T) {
        self.errors.borrow_mut().push(error.into());
    }

    fn create_ast_node(&mut self, kind: AstKind<'a>) {
        let mut flags = self.current_node_flags;
        if self.jsdoc.retrieve_jsdoc_comment(kind) {
            flags |= NodeFlags::JSDoc;
        }
        let ast_node = AstNode::new(kind, self.current_scope_id, flags);
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
        name: &Atom,
        scope_id: ScopeId,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        if let Some(symbol_id) = self.check_redeclaration(scope_id, span, name, excludes, true) {
            self.symbols.union_flag(symbol_id, includes);
            if includes.is_function_scoped_declaration() {
                self.add_redeclared_variables(VariableInfo { span, symbol_id });
            }
            return symbol_id;
        }

        let includes = includes | self.current_symbol_flags;
        let symbol_id = self.symbols.create_symbol(span, name.clone(), includes, scope_id);
        self.symbols.add_declaration(self.current_node_id);
        self.scope.add_binding(scope_id, name.clone(), symbol_id);
        symbol_id
    }

    pub fn declare_symbol(
        &mut self,
        span: Span,
        name: &Atom,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        self.declare_symbol_on_scope(span, name, self.current_scope_id, includes, excludes)
    }

    pub fn check_redeclaration(
        &mut self,
        scope_id: ScopeId,
        span: Span,
        name: &Atom,
        excludes: SymbolFlags,
        report_error: bool,
    ) -> Option<SymbolId> {
        let symbol_id = self.scope.get_binding(scope_id, name)?;
        if report_error && self.symbols.get_flag(symbol_id).intersects(excludes) {
            let symbol_span = self.symbols.get_span(symbol_id);
            self.error(Redeclaration(name.clone(), symbol_span, span));
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
        name: &Atom,
        span: Span,
        scope_id: ScopeId,
        includes: SymbolFlags,
    ) -> SymbolId {
        let includes = includes | self.current_symbol_flags;
        let symbol_id =
            self.symbols.create_symbol(span, name.clone(), includes, self.current_scope_id);
        self.symbols.add_declaration(self.current_node_id);
        self.scope.get_bindings_mut(scope_id).insert(name.clone(), symbol_id);
        symbol_id
    }

    fn resolve_references_for_current_scope(&mut self) {
        let all_references = self
            .scope
            .unresolved_references_mut(self.current_scope_id)
            .drain()
            .collect::<Vec<(Atom, Vec<ReferenceId>)>>();

        let mut unresolved_references: FxHashMap<Atom, Vec<ReferenceId>> = FxHashMap::default();
        let mut resolved_references: Vec<(SymbolId, Vec<ReferenceId>)> = vec![];

        for (name, reference_ids) in all_references {
            if let Some(symbol_id) = self.scope.get_binding(self.current_scope_id, &name) {
                resolved_references.push((symbol_id, reference_ids));
            } else {
                unresolved_references.insert(name, reference_ids);
            }
        }

        let scope_id =
            self.scope.get_parent_id(self.current_scope_id).unwrap_or(self.current_scope_id);

        for (name, reference_ids) in unresolved_references {
            self.scope.extend_unresolved_reference(scope_id, name, reference_ids);
        }

        for (symbol_id, reference_ids) in resolved_references {
            for reference_id in reference_ids {
                self.symbols.references[reference_id].set_symbol_id(symbol_id);
                self.symbols.resolved_references[symbol_id].push(reference_id);
            }
        }
    }

    pub fn add_redeclared_variables(&mut self, variable: VariableInfo) {
        self.redeclare_variables.variables.push(variable);
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
}

impl<'a> SemanticBuilder<'a> {
    fn enter_kind(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::ModuleDeclaration(decl) => {
                self.current_symbol_flags |= Self::symbol_flag_from_module_declaration(decl);
                decl.bind(self);
            }
            AstKind::VariableDeclarator(decl) => {
                decl.bind(self);
                self.make_all_namespaces_valuelike();
            }
            AstKind::Function(func) => {
                self.function_stack.push(self.current_node_id);
                func.bind(self);
                self.add_current_node_id_to_current_scope();
                self.make_all_namespaces_valuelike();
            }
            AstKind::ArrowExpression(_) => {
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
                    .get(module_declaration.id.name());
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
            AstKind::JSXElementName(elem) => {
                self.reference_jsx_element_name(elem);
            }
            AstKind::LabeledStatement(stmt) => {
                self.unused_labels.scopes.push(LabeledScope {
                    name: stmt.label.name.as_str(),
                    used: false,
                    parent: self.unused_labels.curr_scope,
                });
                self.unused_labels.curr_scope = self.unused_labels.scopes.len() - 1;
            }
            AstKind::ContinueStatement(stmt) => {
                if let Some(label) = &stmt.label {
                    let scope =
                        self.unused_labels.scopes.iter_mut().rev().find(|x| x.name == label.name);
                    if let Some(scope) = scope {
                        scope.used = true;
                    }
                }
            }
            AstKind::BreakStatement(stmt) => {
                if let Some(label) = &stmt.label {
                    let scope =
                        self.unused_labels.scopes.iter_mut().rev().find(|x| x.name == label.name);
                    if let Some(scope) = scope {
                        scope.used = true;
                    }
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
            AstKind::Class(_) => {
                self.current_node_flags -= NodeFlags::Class;
                self.class_table_builder.pop_class();
            }
            AstKind::ModuleDeclaration(decl) => {
                self.current_symbol_flags -= Self::symbol_flag_from_module_declaration(decl);
            }
            AstKind::LabeledStatement(_) => {
                let scope = &self.unused_labels.scopes[self.unused_labels.curr_scope];
                if !scope.used {
                    self.unused_labels.labels.push(self.current_node_id);
                }
                self.unused_labels.curr_scope = scope.parent;
            }
            AstKind::Function(_) | AstKind::ArrowExpression(_) => {
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
        let reference = Reference::new(ident.span, ident.name.clone(), self.current_node_id, flag);
        let reference_id = self.declare_reference(reference);
        ident.reference_id.set(Some(reference_id));
    }

    /// Resolve reference flags for the current ast node.
    fn resolve_reference_usages(&self) -> ReferenceFlag {
        if self.in_type_definition {
            return ReferenceFlag::Type;
        }

        let mut flags = ReferenceFlag::None;

        if self.nodes.parent_id(self.current_node_id).is_none() {
            return ReferenceFlag::Read;
        }

        // This func should only get called when an IdentifierReference is
        // reached
        debug_assert!(matches!(
            self.nodes.get_node(self.current_node_id).kind(),
            AstKind::IdentifierReference(_)
        ));

        for (curr, parent) in self
            .nodes
            .iter_parents(self.current_node_id)
            .tuple_windows::<(&AstNode<'a>, &AstNode<'a>)>()
        {
            match (curr.kind(), parent.kind()) {
                // lhs of assignment expression
                (AstKind::SimpleAssignmentTarget(_), AstKind::AssignmentExpression(_)) => {
                    debug_assert!(!flags.is_read());
                    flags = ReferenceFlag::write();
                    // a lhs expr will not propagate upwards into a rhs
                    // expression, sow e can safely break
                    break;
                }
                (AstKind::AssignmentTarget(_), AstKind::AssignmentExpression(expr)) => {
                    flags |= if expr.operator == AssignmentOperator::Assign {
                        ReferenceFlag::write()
                    } else {
                        ReferenceFlag::read_write()
                    };
                    break;
                }
                (_, AstKind::SimpleAssignmentTarget(_) | AstKind::AssignmentTarget(_)) => {
                    flags |= ReferenceFlag::write();
                    // continue up tree
                }
                (_, AstKind::UpdateExpression(_)) => {
                    flags |= ReferenceFlag::Write;
                    // continue up tree
                }
                (
                    AstKind::AssignmentTarget(_),
                    AstKind::ForInStatement(_) | AstKind::ForOfStatement(_),
                ) => {
                    break;
                }
                (_, AstKind::ParenthesizedExpression(_)) => {
                    // continue up tree
                }
                _ => {
                    flags |= ReferenceFlag::Read;
                    break;
                }
            }
        }

        debug_assert!(flags != ReferenceFlag::None);

        flags
    }

    fn reference_jsx_element_name(&mut self, elem: &JSXElementName) {
        if matches!(
            self.nodes.parent_kind(self.current_node_id),
            Some(AstKind::JSXOpeningElement(_))
        ) {
            if let Some(ident) = match elem {
                JSXElementName::Identifier(ident)
                    if ident.name.chars().next().is_some_and(char::is_uppercase) =>
                {
                    Some(ident)
                }
                JSXElementName::MemberExpression(expr) => Some(expr.get_object_identifier()),
                _ => None,
            } {
                let reference = Reference::new(
                    ident.span,
                    ident.name.clone(),
                    self.current_node_id,
                    ReferenceFlag::read(),
                );
                self.declare_reference(reference);
            }
        }
    }

    fn symbol_flag_from_module_declaration(module: &ModuleDeclaration) -> SymbolFlags {
        if matches!(module, ModuleDeclaration::ImportDeclaration(_)) {
            SymbolFlags::Import
        } else {
            SymbolFlags::Export
        }
    }
}
