use oxc_ast::AstKind;
use oxc_ast::ast::{
    BindingIdentifier, BindingPattern, ExportDefaultDeclarationKind, IdentifierReference,
    ImportDeclaration, ModuleExportName, Program, PropertyKind, Statement, TSModuleDeclarationName,
};
use oxc_semantic::{AstNodes, NodeId, Scoping, Semantic};
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use oxc_syntax::symbol::SymbolFlags;
use rustc_hash::FxHashSet;

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
pub struct ImportBindingData {
    /// The module specifier string (e.g., "react" in `import {useState} from 'react'`).
    pub source: String,
    pub kind: ImportBindingKind,
    /// For named imports: the imported name (e.g., "bar" in `import {bar as baz} from 'foo'`).
    /// None for default and namespace imports.
    pub imported: Option<String>,
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
    nodes: &'s AstNodes<'a>,
    /// Positions of resolved identifier references, declaration identifiers, and
    /// `export default function` exports (Babel counts the export statement as a
    /// reference to the function's binding).
    reference_positions: FxHashSet<u32>,
    /// `(start, end, scope_id)` source windows of Function-kind scopes, used for
    /// position-range containment checks. Object methods get an extra window
    /// starting at the enclosing ObjectProperty (Babel's ObjectMethod spans the
    /// whole property, including `get `/`set ` prefixes and computed keys).
    function_scope_ranges: Vec<(u32, u32, ScopeId)>,
}

impl<'s, 'a> ScopeResolver<'s, 'a> {
    pub fn new(semantic: &'s Semantic<'a>, program: &Program<'_>) -> Self {
        let scoping = semantic.scoping();
        let nodes = semantic.nodes();

        let mut reference_positions = FxHashSet::default();
        for symbol_id in scoping.symbol_ids() {
            for &ref_id in scoping.get_resolved_reference_ids(symbol_id) {
                let reference = scoping.get_reference(ref_id);
                reference_positions.insert(nodes.get_node(reference.node_id()).kind().span().start);
            }
            let decl_node = nodes.get_node(scoping.symbol_declaration(symbol_id));
            let name = scoping.symbol_name(symbol_id);
            if let Some(start) = find_binding_identifier_start(decl_node.kind(), name) {
                reference_positions.insert(start);
            }
        }
        for stmt in &program.body {
            let Statement::ExportDefaultDeclaration(export) = stmt else { continue };
            let ExportDefaultDeclarationKind::FunctionDeclaration(func) = &export.declaration
            else {
                continue;
            };
            let Some(id) = &func.id else { continue };
            if id.symbol_id.get().is_some()
                || scoping.symbol_ids().any(|s| scoping.symbol_name(s) == id.name.as_str())
            {
                reference_positions.insert(export.span.start);
            }
        }

        let mut resolver =
            Self { scoping, nodes, reference_positions, function_scope_ranges: Vec::new() };

        let mut function_scope_ranges = Vec::new();
        for scope_id in scoping.scope_descendants_from_root() {
            if resolver.scope_kind(scope_id) != ScopeKind::Function {
                continue;
            }
            let node = nodes.get_node(scoping.get_node_id(scope_id));
            let span = node.kind().span();
            if span.end > span.start {
                function_scope_ranges.push((span.start, span.end, scope_id));
            }
            // For function scopes inside object methods, also record a window from
            // the parent ObjectProperty's start, so range checks match Babel's
            // ObjectMethod extent (which starts at the property, not the function).
            if let AstKind::Function(_) = node.kind() {
                let parent = nodes.parent_node(node.id());
                if let AstKind::ObjectProperty(prop) = parent.kind() {
                    if prop.method || matches!(prop.kind, PropertyKind::Get | PropertyKind::Set) {
                        let prop_start = parent.kind().span().start;
                        if prop_start != span.start {
                            function_scope_ranges.push((prop_start, span.end, scope_id));
                        }
                    }
                }
            }
        }
        resolver.function_scope_ranges = function_scope_ranges;

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
        (self.declaration_start(symbol_id) == Some(ident.span.start)).then_some(symbol_id)
    }

    /// All symbols in the program, in symbol-table order.
    pub fn symbols(&self) -> impl Iterator<Item = SymbolId> + '_ {
        self.scoping().symbol_ids()
    }

    pub fn symbol_name(&self, symbol_id: SymbolId) -> &'s str {
        self.scoping().symbol_name(symbol_id)
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

    /// The start offset of the symbol's declaration identifier (the first
    /// declaration for redeclared symbols). `None` for declaration kinds the
    /// old conversion did not map (e.g. interfaces).
    pub fn declaration_start(&self, symbol_id: SymbolId) -> Option<u32> {
        let decl_node = self.nodes.get_node(self.scoping.symbol_declaration(symbol_id));
        find_binding_identifier_start(decl_node.kind(), self.symbol_name(symbol_id))
    }

    /// For import bindings: the source module and import details.
    pub fn import_data(&self, symbol_id: SymbolId) -> Option<ImportBindingData> {
        let decl_node = self.nodes.get_node(self.scoping.symbol_declaration(symbol_id));
        match decl_node.kind() {
            AstKind::ImportDefaultSpecifier(_) => {
                let import_decl = self.find_import_declaration(decl_node.id())?;
                Some(ImportBindingData {
                    source: import_decl.source.value.to_string(),
                    kind: ImportBindingKind::Default,
                    imported: None,
                })
            }
            AstKind::ImportNamespaceSpecifier(_) => {
                let import_decl = self.find_import_declaration(decl_node.id())?;
                Some(ImportBindingData {
                    source: import_decl.source.value.to_string(),
                    kind: ImportBindingKind::Namespace,
                    imported: None,
                })
            }
            AstKind::ImportSpecifier(spec) => {
                let import_decl = self.find_import_declaration(decl_node.id())?;
                let imported_name = match &spec.imported {
                    ModuleExportName::IdentifierName(ident) => ident.name.to_string(),
                    ModuleExportName::IdentifierReference(ident) => ident.name.to_string(),
                    ModuleExportName::StringLiteral(lit) => lit.value.to_string(),
                };
                Some(ImportBindingData {
                    source: import_decl.source.value.to_string(),
                    kind: ImportBindingKind::Named,
                    imported: Some(imported_name),
                })
            }
            _ => None,
        }
    }

    /// Find the ImportDeclaration node that contains the given import specifier.
    fn find_import_declaration(
        &self,
        specifier_node_id: NodeId,
    ) -> Option<&'s ImportDeclaration<'a>> {
        let nodes = self.nodes;
        let mut current_id = specifier_node_id;
        // Walk up the parent chain (max 10 levels to avoid infinite loop)
        for _ in 0..10 {
            let parent_id = nodes.parent_id(current_id);
            if parent_id == current_id {
                // Root node, no more parents
                return None;
            }
            if let AstKind::ImportDeclaration(decl) = nodes.get_node(parent_id).kind() {
                return Some(decl);
            }
            current_id = parent_id;
        }
        None
    }

    /// Positions of the symbol's resolved references (including TS type references).
    pub fn reference_positions(&self, symbol_id: SymbolId) -> impl Iterator<Item = u32> + '_ {
        let nodes = self.nodes;
        self.scoping().get_resolved_reference_ids(symbol_id).iter().map(move |&ref_id| {
            let reference = self.scoping().get_reference(ref_id);
            nodes.get_node(reference.node_id()).kind().span().start
        })
    }

    /// The positions of every resolved reference, declaration identifier, and
    /// export-default pseudo-reference. Feeds `Environment::reference_node_ids`.
    pub fn all_reference_positions(&self) -> &FxHashSet<u32> {
        &self.reference_positions
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

    /// `(start, end, scope_id)` windows of Function-kind scopes, for
    /// position-range containment checks.
    pub fn function_scope_ranges(&self) -> &[(u32, u32, ScopeId)] {
        &self.function_scope_ranges
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
                if let Some(parent) = scoping.scope_parent_id(scope_id) {
                    if descendants.contains(&(parent.index() as u32)) {
                        descendants.insert(raw);
                        changed = true;
                    }
                }
            }
        }
        descendants
    }
}

/// Find the binding identifier's start position within a declaration AST node.
fn find_binding_identifier_start(kind: AstKind, name: &str) -> Option<u32> {
    match kind {
        AstKind::BindingIdentifier(ident) => {
            if ident.name.as_str() == name {
                Some(ident.span.start)
            } else {
                None
            }
        }
        AstKind::VariableDeclarator(decl) => find_identifier_in_pattern(&decl.id, name),
        AstKind::Function(func) => func
            .id
            .as_ref()
            .and_then(|id| if id.name.as_str() == name { Some(id.span.start) } else { None }),
        AstKind::Class(class) => class
            .id
            .as_ref()
            .and_then(|id| if id.name.as_str() == name { Some(id.span.start) } else { None }),
        AstKind::FormalParameter(param) => find_identifier_in_pattern(&param.pattern, name),
        AstKind::FormalParameterRest(rest) => find_identifier_in_pattern(&rest.rest.argument, name),
        AstKind::ImportSpecifier(spec) => Some(spec.local.span.start),
        AstKind::ImportDefaultSpecifier(spec) => Some(spec.local.span.start),
        AstKind::ImportNamespaceSpecifier(spec) => Some(spec.local.span.start),
        AstKind::CatchClause(catch) => {
            catch.param.as_ref().and_then(|p| find_identifier_in_pattern(&p.pattern, name))
        }
        AstKind::CatchParameter(param) => find_identifier_in_pattern(&param.pattern, name),
        AstKind::TSTypeAliasDeclaration(decl) => {
            if decl.id.name.as_str() == name {
                Some(decl.id.span.start)
            } else {
                None
            }
        }
        AstKind::TSEnumDeclaration(decl) => {
            if decl.id.name.as_str() == name {
                Some(decl.id.span.start)
            } else {
                None
            }
        }
        AstKind::TSModuleDeclaration(decl) => match &decl.id {
            TSModuleDeclarationName::Identifier(id) => {
                if id.name.as_str() == name {
                    Some(id.span.start)
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    }
}

/// Recursively find a binding identifier within a binding pattern.
fn find_identifier_in_pattern(pattern: &BindingPattern, name: &str) -> Option<u32> {
    match pattern {
        BindingPattern::BindingIdentifier(ident) => {
            if ident.name.as_str() == name {
                Some(ident.span.start)
            } else {
                None
            }
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                if let Some(start) = find_identifier_in_pattern(&prop.value, name) {
                    return Some(start);
                }
            }
            if let Some(rest) = &obj.rest {
                if let Some(start) = find_identifier_in_pattern(&rest.argument, name) {
                    return Some(start);
                }
            }
            None
        }
        BindingPattern::ArrayPattern(arr) => {
            for element in arr.elements.iter().flatten() {
                if let Some(start) = find_identifier_in_pattern(element, name) {
                    return Some(start);
                }
            }
            if let Some(rest) = &arr.rest {
                if let Some(start) = find_identifier_in_pattern(&rest.argument, name) {
                    return Some(start);
                }
            }
            None
        }
        BindingPattern::AssignmentPattern(assign) => find_identifier_in_pattern(&assign.left, name),
    }
}
