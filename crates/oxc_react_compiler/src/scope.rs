use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast::ast::{
    BindingIdentifier, BindingPattern, IdentifierReference, ImportDeclaration, ModuleExportName,
    PropertyKind, TSModuleDeclarationName,
};
use oxc_semantic::{AstNodes, NodeId, Scoping, Semantic};
use oxc_span::GetSpan;
use oxc_str::{Ident, Str};
use oxc_syntax::scope::ScopeFlags;
use oxc_syntax::symbol::SymbolFlags;
use rustc_hash::FxHashSet;

pub use oxc_syntax::reference::ReferenceId;
pub use oxc_syntax::scope::ScopeId;
pub use oxc_syntax::symbol::SymbolId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Program,
    Function,
    Block,
    For,
    Class,
    Switch,
    Catch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Var,
    Let,
    Const,
    Param,
    /// Import bindings (import declarations).
    Module,
    /// Function declarations (hoisted).
    Hoisted,
    /// Other local bindings (class declarations, etc.).
    Local,
    /// Binding kind not recognized by the serializer.
    Unknown,
}

/// The kind of a binding's declaration AST node, using Babel's node type
/// names. Interfaces and other unlisted declarations are `Unknown`, matching
/// the old string conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclKind {
    BindingIdentifier,
    VariableDeclarator,
    FunctionDeclaration,
    FunctionExpression,
    ClassDeclaration,
    ClassExpression,
    FormalParameter,
    ImportSpecifier,
    ImportDefaultSpecifier,
    ImportNamespaceSpecifier,
    CatchClause,
    TSTypeAliasDeclaration,
    TSEnumDeclaration,
    TSModuleDeclaration,
    Unknown,
}

impl DeclKind {
    /// The Babel-style node type name; appears verbatim in diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            DeclKind::BindingIdentifier => "BindingIdentifier",
            DeclKind::VariableDeclarator => "VariableDeclarator",
            DeclKind::FunctionDeclaration => "FunctionDeclaration",
            DeclKind::FunctionExpression => "FunctionExpression",
            DeclKind::ClassDeclaration => "ClassDeclaration",
            DeclKind::ClassExpression => "ClassExpression",
            DeclKind::FormalParameter => "FormalParameter",
            DeclKind::ImportSpecifier => "ImportSpecifier",
            DeclKind::ImportDefaultSpecifier => "ImportDefaultSpecifier",
            DeclKind::ImportNamespaceSpecifier => "ImportNamespaceSpecifier",
            DeclKind::CatchClause => "CatchClause",
            DeclKind::TSTypeAliasDeclaration => "TSTypeAliasDeclaration",
            DeclKind::TSEnumDeclaration => "TSEnumDeclaration",
            DeclKind::TSModuleDeclaration => "TSModuleDeclaration",
            DeclKind::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportBindingData<'a> {
    /// The module specifier string (e.g., "react" in `import {useState} from 'react'`).
    pub source: Str<'a>,
    pub kind: ImportBindingKind,
    /// For named imports: the imported name (e.g., "bar" in `import {bar as baz} from 'foo'`).
    /// None for default and namespace imports.
    pub imported: Option<Ident<'a>>,
}

#[derive(Debug, Clone)]
pub enum ImportBindingKind {
    Default,
    Named,
    Namespace,
}

/// Read-through view over `Semantic`, replacing the old materialized
/// `ScopeInfo` copy. Scope and symbol identity comes from the semantic ID
/// cells on the AST (`scope_id`/`symbol_id`/`reference_id`); everything else
/// is derived from `Scoping` on demand.
pub struct ScopeResolver<'s, 'a> {
    scoping: &'s Scoping,
    nodes: &'s AstNodes<'s>,
    allocator: &'a Allocator,
    /// All Function-kind scopes, in scope-tree order.
    function_scopes: Vec<ScopeId>,
    /// `(start, end)` source windows of Function-kind scopes, used by the
    /// hoisting analysis's position checks. Object methods get an extra window
    /// starting at the enclosing ObjectProperty (Babel's ObjectMethod spans the
    /// whole property, including `get `/`set ` prefixes and computed keys).
    function_scope_ranges: Vec<(u32, u32)>,
}

impl<'s, 'a> ScopeResolver<'s, 'a> {
    pub fn new<'ast>(semantic: &'s Semantic<'ast>, allocator: &'a Allocator) -> Self {
        let scoping = semantic.scoping();
        // `AstNodes` is covariant in its lifetime; shrink the AST references to
        // the `Semantic` borrow so the resolver needs no third lifetime.
        let nodes: &'s AstNodes<'s> = semantic.nodes();

        let mut resolver = Self {
            scoping,
            nodes,
            allocator,
            function_scopes: Vec::new(),
            function_scope_ranges: Vec::new(),
        };

        for scope_id in scoping.scope_descendants_from_root() {
            if resolver.scope_kind(scope_id) != ScopeKind::Function {
                continue;
            }
            resolver.function_scopes.push(scope_id);
            let node = nodes.get_node(scoping.get_node_id(scope_id));
            let span = node.kind().span();
            if span.end > span.start {
                resolver.function_scope_ranges.push((span.start, span.end));
            }
            // For function scopes inside object methods, also record a window from
            // the parent ObjectProperty's start, so range checks match Babel's
            // ObjectMethod extent (which starts at the property, not the function).
            if let AstKind::Function(_) = node.kind() {
                let parent = nodes.parent_node(node.id());
                if let AstKind::ObjectProperty(prop) = parent.kind()
                    && is_object_method_property(prop)
                {
                    let prop_start = parent.kind().span().start;
                    if prop_start != span.start {
                        resolver.function_scope_ranges.push((prop_start, span.end));
                    }
                }
            }
        }

        resolver
    }

    fn scoping(&self) -> &'s Scoping {
        self.scoping
    }

    /// Resolve an identifier reference to the symbol it refers to.
    /// `None` means global/unresolved.
    pub fn resolve_reference(&self, ident: &IdentifierReference) -> Option<SymbolId> {
        self.scoping().get_reference(ident.reference_id.get()?).symbol_id()
    }

    /// Resolve a binding identifier to its symbol, but only when this
    /// identifier is the symbol's original declaration site. Redeclarations
    /// (`function x() {} function x() {}`) return `None`, so callers fall back
    /// to name-based resolution — matching the old position-map behavior where
    /// only the first declaration's position was mapped.
    pub fn resolve_binding_identifier(&self, ident: &BindingIdentifier) -> Option<SymbolId> {
        let symbol_id = ident.symbol_id.get()?;
        self.declaration_ident(symbol_id)
            .is_some_and(|decl| std::ptr::eq(decl, ident))
            .then_some(symbol_id)
    }

    /// All symbols in the program, in symbol-table order.
    pub fn symbols(&self) -> impl Iterator<Item = SymbolId> + '_ {
        self.scoping().symbol_ids()
    }

    pub fn symbol_name(&self, symbol_id: SymbolId) -> &'s str {
        self.scoping().symbol_name(symbol_id)
    }

    /// The symbol's name as an arena-lifetime `Ident`, copied into the arena.
    /// The name in `Scoping` lives in the `Semantic` borrow, not the arena, so
    /// a copy decouples the compiled output from that borrow.
    pub fn symbol_ident(&self, symbol_id: SymbolId) -> Ident<'a> {
        Ident::from_str_in(self.symbol_name(symbol_id), &self.allocator)
    }

    /// The scope a symbol is declared in.
    ///
    /// A symbol whose binding-map entry was overwritten by a same-name shadow
    /// (e.g. the function-expression name in `(function n(n) {})`) was invisible
    /// to the old scope conversion, which left its scope at the program-scope
    /// placeholder; consumers classify such symbols as module-level. Preserved.
    pub fn symbol_scope(&self, symbol_id: SymbolId) -> ScopeId {
        let scoping = self.scoping();
        let scope_id = scoping.symbol_scope_id(symbol_id);
        if scoping.get_binding(scope_id, scoping.symbol_name(symbol_id).into()) == Some(symbol_id) {
            scope_id
        } else {
            scoping.root_scope_id()
        }
    }

    /// Map the symbol's flags and declaration node to a Babel-style binding kind.
    pub fn binding_kind(&self, symbol_id: SymbolId) -> BindingKind {
        let flags = self.scoping().symbol_flags(symbol_id);
        if flags.contains(SymbolFlags::Import) {
            return BindingKind::Module;
        }

        // Check the declaration node first — FormalParameter and CatchParameter
        // need to be detected before the FunctionScopedVariable flag check, because
        // OXC marks function parameters and catch parameters with FunctionScopedVariable.
        let decl_node = self.nodes.get_node(self.scoping.symbol_declaration(symbol_id));
        match decl_node.kind() {
            AstKind::FormalParameter(_) => return BindingKind::Param,
            AstKind::FormalParameterRest(_) => return BindingKind::Param,
            AstKind::CatchParameter(_) => return BindingKind::Let,
            AstKind::TSTypeAliasDeclaration(_) => return BindingKind::Local,
            AstKind::TSEnumDeclaration(_) => return BindingKind::Local,
            AstKind::TSModuleDeclaration(_) => return BindingKind::Local,
            AstKind::Function(_) => {
                if flags.contains(SymbolFlags::Function) {
                    return BindingKind::Hoisted;
                }
                return BindingKind::Local;
            }
            AstKind::Class(_) => return BindingKind::Local,
            _ => {}
        }

        if flags.contains(SymbolFlags::FunctionScopedVariable) {
            return BindingKind::Var;
        }

        if flags.contains(SymbolFlags::BlockScopedVariable) {
            if flags.contains(SymbolFlags::ConstVariable) {
                return BindingKind::Const;
            }
            return BindingKind::Let;
        }

        if flags.contains(SymbolFlags::Function) {
            BindingKind::Hoisted
        } else if flags.contains(SymbolFlags::Class) {
            BindingKind::Local
        } else {
            BindingKind::Unknown
        }
    }

    /// Whether a symbol is a type-only binding — a type parameter, interface,
    /// or a pure type alias / type-only import.
    ///
    /// Babel never registers these in `scope.bindings`, so its BuildHIR hoisting
    /// analysis can never see them: `stmt.scope.getBinding(name)` returns `null`
    /// for a type name, so the "referenced before declaration" check bails out
    /// early and the identifier is never hoisted. OXC's semantic model *does*
    /// give type parameters and interfaces a symbol, which lands them in the
    /// hoistable set with an unclassifiable declaration kind and makes every
    /// pure-type-position reference inside a nested function look like a
    /// referenced-before-declared use — over-bailing whole generic functions
    /// that Babel compiles. Excluding type-only symbols mirrors Babel's binding
    /// table exactly.
    ///
    /// Symbols that are *also* values (a `class`, an `enum`, a value module, or
    /// an `interface`/`class` merged declaration) are not type-only and remain
    /// hoistable, matching Babel, which does register those.
    pub fn is_type_only_binding(&self, symbol_id: SymbolId) -> bool {
        let flags = self.scoping().symbol_flags(symbol_id);
        flags.is_type() && !flags.is_value()
    }

    /// The kind of the symbol's declaration AST node.
    pub fn decl_kind(&self, symbol_id: SymbolId) -> DeclKind {
        match self.nodes.get_node(self.scoping.symbol_declaration(symbol_id)).kind() {
            AstKind::BindingIdentifier(_) => DeclKind::BindingIdentifier,
            AstKind::VariableDeclarator(_) => DeclKind::VariableDeclarator,
            AstKind::Function(f) => {
                if f.is_declaration() {
                    DeclKind::FunctionDeclaration
                } else {
                    DeclKind::FunctionExpression
                }
            }
            AstKind::Class(c) => {
                if c.is_declaration() {
                    DeclKind::ClassDeclaration
                } else {
                    DeclKind::ClassExpression
                }
            }
            AstKind::FormalParameter(_) | AstKind::FormalParameterRest(_) => {
                DeclKind::FormalParameter
            }
            AstKind::ImportSpecifier(_) => DeclKind::ImportSpecifier,
            AstKind::ImportDefaultSpecifier(_) => DeclKind::ImportDefaultSpecifier,
            AstKind::ImportNamespaceSpecifier(_) => DeclKind::ImportNamespaceSpecifier,
            AstKind::CatchClause(_) | AstKind::CatchParameter(_) => DeclKind::CatchClause,
            AstKind::TSTypeAliasDeclaration(_) => DeclKind::TSTypeAliasDeclaration,
            AstKind::TSEnumDeclaration(_) => DeclKind::TSEnumDeclaration,
            AstKind::TSModuleDeclaration(_) => DeclKind::TSModuleDeclaration,
            _ => DeclKind::Unknown,
        }
    }

    /// The symbol's declaration identifier (the first declaration for
    /// redeclared symbols). `None` for declaration kinds the old conversion
    /// did not map (e.g. interfaces).
    pub fn declaration_ident(&self, symbol_id: SymbolId) -> Option<&'s BindingIdentifier<'s>> {
        let decl_node = self.nodes.get_node(self.scoping.symbol_declaration(symbol_id));
        find_binding_identifier(decl_node.kind(), self.symbol_name(symbol_id))
    }

    /// For import bindings: the source module and import details.
    pub fn import_data(&self, symbol_id: SymbolId) -> Option<ImportBindingData<'a>> {
        let decl_node = self.nodes.get_node(self.scoping.symbol_declaration(symbol_id));
        match decl_node.kind() {
            AstKind::ImportDefaultSpecifier(_) => {
                let import_decl = self.find_import_declaration(decl_node.id())?;
                Some(ImportBindingData {
                    source: Str::from_str_in(import_decl.source.value.as_str(), &self.allocator),
                    kind: ImportBindingKind::Default,
                    imported: None,
                })
            }
            AstKind::ImportNamespaceSpecifier(_) => {
                let import_decl = self.find_import_declaration(decl_node.id())?;
                Some(ImportBindingData {
                    source: Str::from_str_in(import_decl.source.value.as_str(), &self.allocator),
                    kind: ImportBindingKind::Namespace,
                    imported: None,
                })
            }
            AstKind::ImportSpecifier(spec) => {
                let import_decl = self.find_import_declaration(decl_node.id())?;
                let imported_name = match &spec.imported {
                    ModuleExportName::IdentifierName(ident) => ident.name.as_str(),
                    ModuleExportName::IdentifierReference(ident) => ident.name.as_str(),
                    ModuleExportName::StringLiteral(lit) => lit.value.as_str(),
                };
                Some(ImportBindingData {
                    source: Str::from_str_in(import_decl.source.value.as_str(), &self.allocator),
                    kind: ImportBindingKind::Named,
                    imported: Some(Ident::from_str_in(imported_name, &self.allocator)),
                })
            }
            _ => None,
        }
    }

    /// Find the ImportDeclaration node that contains the given import specifier.
    fn find_import_declaration(
        &self,
        specifier_node_id: NodeId,
    ) -> Option<&'s ImportDeclaration<'s>> {
        let mut current_id = specifier_node_id;
        // Walk up the parent chain (max 10 levels to avoid infinite loop)
        for _ in 0..10 {
            let parent_id = self.nodes.parent_id(current_id);
            if parent_id == current_id {
                // Root node, no more parents
                return None;
            }
            if let AstKind::ImportDeclaration(decl) = self.nodes.get_node(parent_id).kind() {
                return Some(decl);
            }
            current_id = parent_id;
        }
        None
    }

    /// The symbol's resolved reference IDs (including TS type references).
    pub fn reference_ids(&self, symbol_id: SymbolId) -> &'s [ReferenceId] {
        self.scoping().get_resolved_reference_ids(symbol_id)
    }

    /// The program-level (module) scope.
    pub fn program_scope(&self) -> ScopeId {
        self.scoping().root_scope_id()
    }

    /// Map the scope's flags and creating node to a Babel-style scope kind.
    pub fn scope_kind(&self, scope_id: ScopeId) -> ScopeKind {
        let flags = self.scoping().scope_flags(scope_id);
        if flags.contains(ScopeFlags::Top) {
            return ScopeKind::Program;
        }
        if flags.intersects(ScopeFlags::Function) {
            return ScopeKind::Function;
        }
        if flags.contains(ScopeFlags::CatchClause) {
            return ScopeKind::Catch;
        }
        if flags.contains(ScopeFlags::ClassStaticBlock) {
            return ScopeKind::Class;
        }
        let node_id = self.scoping().get_node_id(scope_id);
        match self.nodes.get_node(node_id).kind() {
            AstKind::ForStatement(_) | AstKind::ForInStatement(_) | AstKind::ForOfStatement(_) => {
                ScopeKind::For
            }
            AstKind::Class(_) => ScopeKind::Class,
            AstKind::SwitchStatement(_) => ScopeKind::Switch,
            _ => ScopeKind::Block,
        }
    }

    pub fn scope_parent(&self, scope_id: ScopeId) -> Option<ScopeId> {
        self.scoping().scope_parent_id(scope_id)
    }

    /// The scope and its ancestors, innermost first (starts with `scope_id` itself).
    pub fn ancestors(&self, scope_id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
        self.scoping().scope_ancestors(scope_id)
    }

    /// Look up a binding by name in a single scope.
    pub fn get_binding(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        self.scoping().get_binding(scope_id, name.into())
    }

    /// Look up a binding by name starting from the given scope,
    /// walking up the parent chain. Returns None for globals.
    pub fn find_binding(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        self.scoping().find_binding(scope_id, name.into())
    }

    /// Whether `name` is referenced as a global (unresolved reference) anywhere in the file.
    pub fn has_unresolved_reference(&self, name: &str) -> bool {
        self.scoping().root_unresolved_references().contains_key(name)
    }

    /// Bindings declared directly in a scope.
    pub fn bindings_in(&self, scope_id: ScopeId) -> impl Iterator<Item = SymbolId> + '_ {
        self.scoping().iter_bindings_in(scope_id)
    }

    /// Get bindings from a scope AND its direct child block scopes.
    /// In Babel, a function body's BlockStatement shares the function's scope,
    /// so all bindings (var, const, let) appear in one scope. But our scope
    /// extraction may split them: function scope has params/var, a child block
    /// scope has const/let. This method merges them to match TS behavior.
    pub fn bindings_with_child_blocks(&self, scope_id: ScopeId) -> Vec<SymbolId> {
        let scoping = self.scoping();
        let mut symbols: Vec<SymbolId> = scoping.iter_bindings_in(scope_id).collect();
        for child in scoping.scope_descendants_from_root() {
            if scoping.scope_parent_id(child) == Some(scope_id)
                && self.scope_kind(child) == ScopeKind::Block
            {
                symbols.extend(scoping.iter_bindings_in(child));
            }
        }
        symbols
    }

    /// `(start, end)` source windows of Function-kind scopes, for the hoisting
    /// analysis's position checks.
    pub fn function_scope_ranges(&self) -> &[(u32, u32)] {
        &self.function_scope_ranges
    }

    /// All Function-kind scopes, in scope-tree order.
    pub fn function_scopes(&self) -> impl Iterator<Item = ScopeId> + '_ {
        self.function_scopes.iter().copied()
    }

    /// The AST node a reference sits on.
    pub fn reference_node_id(&self, reference_id: ReferenceId) -> NodeId {
        self.scoping().get_reference(reference_id).node_id()
    }

    /// The subtree root for capture analysis of a function scope: the function
    /// node itself, widened to the enclosing ObjectProperty for object methods
    /// and accessors (Babel's ObjectMethod node spans the whole property,
    /// including `get `/`set ` prefixes and computed keys).
    pub fn capture_root_node(&self, function_scope: ScopeId) -> NodeId {
        let node_id = self.scoping().get_node_id(function_scope);
        let node = self.nodes.get_node(node_id);
        if let AstKind::Function(_) = node.kind() {
            let parent = self.nodes.parent_node(node_id);
            if let AstKind::ObjectProperty(prop) = parent.kind()
                && is_object_method_property(prop)
            {
                return parent.id();
            }
        }
        node_id
    }

    /// Whether `node_id` is `root` or one of its descendants.
    pub fn node_within(&self, node_id: NodeId, root: NodeId) -> bool {
        node_id == root || self.nodes.ancestor_ids(node_id).any(|id| id == root)
    }

    /// Scopes of the functions lexically containing `node_id`, innermost first.
    /// Mirrors Babel's ObjectMethod extent: a node inside a method property's
    /// computed key or accessor prefix counts as inside that method's function.
    pub fn containing_function_scopes(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = ScopeId> + '_ {
        self.nodes.ancestor_kinds(node_id).filter_map(|kind| match kind {
            AstKind::Function(func) => func.scope_id.get(),
            AstKind::ArrowFunctionExpression(arrow) => arrow.scope_id.get(),
            AstKind::ObjectProperty(prop) if is_object_method_property(prop) => {
                match prop.value.kind() {
                    oxc_ast::ast::ExpressionKind::FunctionExpression(func) => func.scope_id.get(),
                    _ => None,
                }
            }
            _ => None,
        })
    }

    /// Find a binding by name within the descendants of a given scope
    /// (including the scope itself).
    pub fn find_binding_in_descendants(&self, name: &str, ancestor: ScopeId) -> Option<SymbolId> {
        for &raw in &self.descendant_scopes(ancestor) {
            if let Some(symbol_id) = self.get_binding(ScopeId::new(raw as usize), name) {
                return Some(symbol_id);
            }
        }
        None
    }

    /// Fixed-point descendant set as raw indices. Kept as `FxHashSet<u32>` with
    /// the same insertion sequence as the old ScopeInfo searches so the
    /// arbitrary-but-deterministic pick among same-named bindings is preserved.
    fn descendant_scopes(&self, ancestor: ScopeId) -> FxHashSet<u32> {
        let scoping = self.scoping();
        let mut descendants = FxHashSet::default();
        descendants.insert(ancestor.index() as u32);
        let mut changed = true;
        while changed {
            changed = false;
            for scope_id in scoping.scope_descendants_from_root() {
                let raw = scope_id.index() as u32;
                if descendants.contains(&raw) {
                    continue;
                }
                if let Some(parent) = scoping.scope_parent_id(scope_id)
                    && descendants.contains(&(parent.index() as u32))
                {
                    descendants.insert(raw);
                    changed = true;
                }
            }
        }
        descendants
    }
}

/// Whether an object property is what Babel models as an `ObjectMethod`
/// (a method or a `get`/`set` accessor), whose node spans the whole property.
fn is_object_method_property(prop: &oxc_ast::ast::ObjectProperty) -> bool {
    prop.method || matches!(prop.kind, PropertyKind::Get | PropertyKind::Set)
}

/// Find the binding identifier with the given name within a declaration AST node.
fn find_binding_identifier<'a>(kind: AstKind<'a>, name: &str) -> Option<&'a BindingIdentifier<'a>> {
    match kind {
        AstKind::BindingIdentifier(ident) => (ident.name.as_str() == name).then_some(ident),
        AstKind::VariableDeclarator(decl) => find_identifier_in_pattern(&decl.id, name),
        AstKind::Function(func) => func.id.as_ref().filter(|id| id.name.as_str() == name),
        AstKind::Class(class) => class.id.as_ref().filter(|id| id.name.as_str() == name),
        AstKind::FormalParameter(param) => find_identifier_in_pattern(&param.pattern, name),
        AstKind::FormalParameterRest(rest) => find_identifier_in_pattern(&rest.rest.argument, name),
        AstKind::ImportSpecifier(spec) => Some(&spec.local),
        AstKind::ImportDefaultSpecifier(spec) => Some(&spec.local),
        AstKind::ImportNamespaceSpecifier(spec) => Some(&spec.local),
        AstKind::CatchClause(catch) => {
            catch.param.as_ref().and_then(|p| find_identifier_in_pattern(&p.pattern, name))
        }
        AstKind::CatchParameter(param) => find_identifier_in_pattern(&param.pattern, name),
        AstKind::TSTypeAliasDeclaration(decl) => {
            (decl.id.name.as_str() == name).then_some(&decl.id)
        }
        AstKind::TSEnumDeclaration(decl) => (decl.id.name.as_str() == name).then_some(&decl.id),
        AstKind::TSModuleDeclaration(decl) => match &decl.id {
            TSModuleDeclarationName::Identifier(id) => (id.name.as_str() == name).then_some(id),
            _ => None,
        },
        _ => None,
    }
}

/// Recursively find a binding identifier within a binding pattern.
fn find_identifier_in_pattern<'a>(
    pattern: &'a BindingPattern<'a>,
    name: &str,
) -> Option<&'a BindingIdentifier<'a>> {
    match pattern {
        BindingPattern::BindingIdentifier(ident) => (ident.name.as_str() == name).then_some(ident),
        BindingPattern::ObjectPattern(obj) => obj
            .properties
            .iter()
            .find_map(|prop| find_identifier_in_pattern(&prop.value, name))
            .or_else(|| {
                obj.rest.as_ref().and_then(|rest| find_identifier_in_pattern(&rest.argument, name))
            }),
        BindingPattern::ArrayPattern(arr) => arr
            .elements
            .iter()
            .flatten()
            .find_map(|element| find_identifier_in_pattern(element, name))
            .or_else(|| {
                arr.rest.as_ref().and_then(|rest| find_identifier_in_pattern(&rest.argument, name))
            }),
        BindingPattern::AssignmentPattern(assign) => find_identifier_in_pattern(&assign.left, name),
    }
}
