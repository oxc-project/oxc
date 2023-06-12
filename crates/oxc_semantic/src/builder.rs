//! Semantic Builder
//! This builds:
//!   * The untyped and flattened ast nodes into an indextree

use std::{cell::RefCell, rc::Rc};

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, module_record::ModuleRecord, AstKind, Trivias, Visit};
use oxc_diagnostics::Error;
use oxc_span::{Atom, GetSpan, SourceType, Span};

use crate::{
    binder::Binder,
    checker::{EarlyErrorJavaScript, EarlyErrorTypeScript},
    diagnostics::Redeclaration,
    jsdoc::JSDocBuilder,
    module_record::ModuleRecordBuilder,
    node::{AstNodeId, AstNodes, NodeFlags, SemanticNode},
    scope::{ScopeBuilder, ScopeId},
    symbol::{Reference, ReferenceFlag, SymbolFlags, SymbolId, SymbolTableBuilder},
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

    // builders
    pub nodes: AstNodes<'a>,
    pub scope: ScopeBuilder,
    pub symbols: SymbolTableBuilder,

    with_module_record_builder: bool,
    pub module_record_builder: ModuleRecordBuilder,
    unused_labels: UnusedLabels<'a>,

    jsdoc: JSDocBuilder<'a>,

    check_syntax_error: bool,
}

pub struct SemanticBuilderReturn<'a> {
    pub semantic: Semantic<'a>,
    pub errors: Vec<Error>,
}

impl<'a> SemanticBuilder<'a> {
    pub fn new(source_text: &'a str, source_type: SourceType, trivias: &Rc<Trivias>) -> Self {
        let scope = ScopeBuilder::new(source_type);
        let mut nodes = AstNodes::default();
        let semantic_node =
            SemanticNode::new(AstKind::Root, scope.current_scope_id, NodeFlags::empty());
        let current_node_id = nodes.new_node(semantic_node).into();
        Self {
            source_text,
            source_type,
            trivias: Rc::clone(trivias),
            errors: RefCell::new(vec![]),
            current_node_id,
            current_node_flags: NodeFlags::empty(),
            current_symbol_flags: SymbolFlags::empty(),
            nodes,
            scope,
            symbols: SymbolTableBuilder::default(),
            with_module_record_builder: false,
            module_record_builder: ModuleRecordBuilder::default(),
            unused_labels: UnusedLabels { scopes: vec![], curr_scope: 0, labels: vec![] },
            jsdoc: JSDocBuilder::new(source_text, trivias),
            check_syntax_error: false,
        }
    }

    #[must_use]
    pub fn with_module_record_builder(mut self, yes: bool) -> Self {
        self.with_module_record_builder = yes;
        self
    }

    #[must_use]
    pub fn with_check_syntax_error(mut self, yes: bool) -> Self {
        self.check_syntax_error = yes;
        self
    }

    pub fn build(mut self, program: &'a Program<'a>) -> SemanticBuilderReturn<'a> {
        // First AST pass
        if !self.source_type.is_typescript_definition() {
            self.visit_program(program);
        }

        // Second partial AST pass on top level import / export statements
        let module_record = if self.with_module_record_builder {
            self.module_record_builder.visit(program);
            if self.check_syntax_error {
                EarlyErrorJavaScript::check_module_record(&self);
            }
            self.module_record_builder.build()
        } else {
            ModuleRecord::default()
        };

        let semantic = Semantic {
            source_text: self.source_text,
            source_type: self.source_type,
            trivias: self.trivias,
            nodes: self.nodes,
            scopes: self.scope.scopes,
            symbols: Rc::new(self.symbols.build()),
            module_record,
            jsdoc: self.jsdoc.build(),
            unused_labels: self.unused_labels.labels,
        };
        SemanticBuilderReturn { semantic, errors: self.errors.into_inner() }
    }

    /// Push a Syntax Error
    pub fn error<T: Into<Error>>(&self, error: T) {
        self.errors.borrow_mut().push(error.into());
    }

    /// # Panics
    /// The parent of `AstKind::Program` is `AstKind::Root`,
    /// it is logic error if this panics.
    pub fn parent_kind(&self) -> AstKind<'a> {
        let parent_id = self.nodes[*self.current_node_id].parent().unwrap();
        let parent_node = self.nodes[parent_id].get();
        parent_node.kind()
    }

    fn create_ast_node(&mut self, kind: AstKind<'a>) {
        let mut flags = self.current_node_flags;
        if self.jsdoc.retrieve_jsdoc_comment(kind) {
            flags |= NodeFlags::JSDoc;
        }
        let ast_node = SemanticNode::new(kind, self.scope.current_scope_id, flags);
        let node_id = self.current_node_id.append_value(ast_node, &mut self.nodes);
        self.current_node_id = node_id.into();
    }

    fn pop_ast_node(&mut self) {
        self.current_node_id =
            self.nodes[self.current_node_id.indextree_id()].parent().unwrap().into();
    }

    fn try_enter_scope(&mut self, kind: AstKind<'a>) {
        fn is_strict(directives: &[Directive]) -> bool {
            directives.iter().any(|d| d.directive == "use strict")
        }
        if let Some(flags) = ScopeBuilder::scope_flags_from_ast_kind(kind) {
            self.scope.enter(flags);
        }
        let strict_mode = match kind {
            AstKind::Program(program) => is_strict(&program.directives),
            AstKind::Function(func) => {
                func.body.as_ref().is_some_and(|body| is_strict(&body.directives))
            }
            _ => false,
        };
        if strict_mode {
            self.scope.current_scope_mut().strict_mode = true;
        }
    }

    fn try_leave_scope(&mut self, kind: AstKind<'a>) {
        if ScopeBuilder::scope_flags_from_ast_kind(kind).is_some()
            || matches!(kind, AstKind::Program(_))
        {
            self.scope.resolve_reference(&mut self.symbols);
            self.scope.leave();
        }
    }

    pub fn strict_mode(&self) -> bool {
        self.scope.current_scope().strict_mode()
            || self.current_node_flags.contains(NodeFlags::Class)
    }

    /// Declares a `Symbol` for the node, adds it to symbol table, and binds it to the scope.
    /// Reports errors for conflicting identifier names.
    pub fn declare_symbol(
        &mut self,
        name: &Atom,
        span: Span,
        scope_id: ScopeId,
        // The SymbolFlags that node has in addition to its declaration type (eg: export, ambient, etc.)
        includes: SymbolFlags,
        // The flags which node cannot be declared alongside in a symbol table. Used to report forbidden declarations.
        excludes: SymbolFlags,
    ) -> SymbolId {
        if let Some(symbol_id) = self.check_redeclaration(scope_id, name, span, excludes) {
            return symbol_id;
        }
        let includes = includes | self.current_symbol_flags;
        let symbol_id = self.symbols.create(self.current_node_id, name.clone(), span, includes);
        self.scope.scopes[scope_id].variables.insert(name.clone(), symbol_id);
        symbol_id
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
        let symbol_id = self.symbols.create(self.current_node_id, name.clone(), span, includes);
        self.scope.scopes[scope_id].variables.insert(name.clone(), symbol_id);
        symbol_id
    }

    pub fn check_redeclaration(
        &mut self,
        scope_id: ScopeId,
        name: &Atom,
        span: Span,
        excludes: SymbolFlags,
    ) -> Option<SymbolId> {
        self.scope.scopes[scope_id].get_variable_symbol_id(name).map(|symbol_id| {
            let symbol = &self.symbols[symbol_id];
            if symbol.flags().intersects(excludes) {
                self.error(Redeclaration(name.clone(), symbol.span(), span));
            }
            symbol_id
        })
    }
}

impl<'a> Visit<'a> for SemanticBuilder<'a> {
    // Setup all the context for the binder,
    // the order is important here.
    fn enter_node(&mut self, kind: AstKind<'a>) {
        // create new self.scope.current_scope_id
        self.try_enter_scope(kind);

        // create new self.current_node_id
        self.create_ast_node(kind);

        self.enter_kind(kind);
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        if self.check_syntax_error {
            let node = &self.nodes[*self.current_node_id];
            EarlyErrorJavaScript::run(node, self);
            EarlyErrorTypeScript::run(node, self);
        }
        self.leave_kind(kind);
        self.pop_ast_node();
        self.try_leave_scope(kind);
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
            }
            AstKind::Function(func) => {
                func.bind(self);
            }
            AstKind::Class(class) => {
                self.current_node_flags |= NodeFlags::Class;
                class.bind(self);
            }
            AstKind::FormalParameters(params) => {
                params.bind(self);
            }
            AstKind::CatchClause(clause) => {
                clause.bind(self);
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
            _ => {}
        }
    }

    #[allow(clippy::single_match)]
    fn leave_kind(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Class(_) => {
                self.current_node_flags -= NodeFlags::Class;
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
            _ => {}
        }
    }

    fn reference_identifier(&mut self, ident: &IdentifierReference) {
        let flag = if matches!(
            self.parent_kind(),
            AstKind::SimpleAssignmentTarget(_) | AstKind::AssignmentTarget(_)
        ) {
            ReferenceFlag::Write
        } else {
            ReferenceFlag::Read
        };
        let reference = Reference::new(self.current_node_id, ident.span, flag);
        self.scope.reference_identifier(&ident.name, reference);
    }

    fn reference_jsx_element_name(&mut self, elem: &JSXElementName) {
        if matches!(self.parent_kind(), AstKind::JSXOpeningElement(_)) {
            if let Some(ident) = match elem {
                JSXElementName::Identifier(ident)
                    if ident.name.chars().next().is_some_and(char::is_uppercase) =>
                {
                    Some(ident)
                }
                JSXElementName::MemberExpression(expr) => Some(expr.get_object_identifier()),
                _ => None,
            } {
                let reference =
                    Reference::new(self.current_node_id, elem.span(), ReferenceFlag::Read);
                self.scope.reference_identifier(&ident.name, reference);
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
