use std::fmt;

use oxc_ast::{
    ast::{AssignmentTarget, BindingIdentifier, BindingPattern, IdentifierReference},
    AstKind,
};
use oxc_semantic::{
    AstNode, AstNodeId, AstNodes, Reference, ScopeId, ScopeTree, Semantic, SymbolFlags, SymbolId,
    SymbolTable,
};
use oxc_span::Span;

#[derive(Clone)]
pub(super) struct Symbol<'s, 'a> {
    semantic: &'s Semantic<'a>,
    id: SymbolId,
    flags: SymbolFlags,
}

impl PartialEq for Symbol<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// constructor and simple getters
impl<'s, 'a> Symbol<'s, 'a> {
    pub fn new(semantic: &'s Semantic<'a>, symbol_id: SymbolId) -> Self {
        let flags = semantic.symbols().get_flag(symbol_id);
        Self { semantic, id: symbol_id, flags }
    }

    #[inline]
    pub fn id(&self) -> SymbolId {
        self.id
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.symbols().get_name(self.id)
    }

    /// [`Span`] for the node declaring the [`Symbol`].
    #[inline]
    pub fn span(&self) -> Span {
        self.symbols().get_span(self.id)
    }

    #[inline]
    pub const fn flags(&self) -> SymbolFlags {
        self.flags
    }

    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.symbols().get_scope_id(self.id)
    }

    #[inline]
    pub fn declaration(&self) -> &AstNode<'a> {
        self.nodes().get_node(self.declaration_id())
    }

    #[inline]
    pub fn references(&self) -> impl DoubleEndedIterator<Item = &Reference> + '_ {
        self.symbols().get_resolved_references(self.id)
    }

    /// Is this [`Symbol`] declared in the root scope?
    pub fn is_root(&self) -> bool {
        self.symbols().get_scope_id(self.id) == self.scopes().root_scope_id()
    }

    #[inline]
    fn declaration_id(&self) -> AstNodeId {
        self.symbols().get_declaration(self.id)
    }

    #[inline]
    pub fn nodes(&self) -> &AstNodes<'a> {
        self.semantic.nodes()
    }

    #[inline]
    pub fn scopes(&self) -> &ScopeTree {
        self.semantic.scopes()
    }

    #[inline]
    pub fn symbols(&self) -> &SymbolTable {
        self.semantic.symbols()
    }

    pub fn iter_parents(&self) -> impl Iterator<Item = &AstNode<'a>> + '_ {
        self.nodes().iter_parents(self.declaration_id()).skip(1)
    }

    pub fn iter_relevant_parents(
        &self,
        node_id: AstNodeId,
    ) -> impl Iterator<Item = &AstNode<'a>> + Clone + '_ {
        self.nodes().iter_parents(node_id).skip(1).filter(|n| Self::is_relevant_kind(n.kind()))
    }

    pub fn iter_relevant_parent_and_grandparent_kinds(
        &self,
        node_id: AstNodeId,
    ) -> impl Iterator<Item = (/* parent */ AstKind<'a>, /* grandparent */ AstKind<'a>)> + Clone + '_
    {
        let parents_iter = self
            .nodes()
            .iter_parents(node_id)
            .map(AstNode::kind)
            // no skip
            .filter(|kind| Self::is_relevant_kind(*kind));

        let grandparents_iter = parents_iter.clone().skip(1);

        parents_iter.zip(grandparents_iter)
    }

    #[inline]
    const fn is_relevant_kind(kind: AstKind<'a>) -> bool {
        !matches!(kind, AstKind::ParenthesizedExpression(_))
    }
}

impl<'s, 'a> Symbol<'s, 'a> {
    /// Is this [`Symbol`] exported?
    ///
    /// NOTE: does not support CJS right now.
    pub fn is_exported(&self) -> bool {
        let is_in_exportable_scope = self.is_root() || self.is_in_ts_namespace();
        (is_in_exportable_scope
            && (self.flags.contains(SymbolFlags::Export)
                || self.semantic.module_record().exported_bindings.contains_key(self.name())))
            || self.in_export_node()
    }

    #[inline]
    fn is_in_ts_namespace(&self) -> bool {
        self.scopes().get_flags(self.scope_id()).is_ts_module_block()
    }

    /// We need to do this due to limitations of [`Semantic`].
    fn in_export_node(&self) -> bool {
        for parent in self.nodes().iter_parents(self.declaration_id()).skip(1) {
            match parent.kind() {
                AstKind::ModuleDeclaration(module) => {
                    return module.is_export();
                }
                AstKind::VariableDeclaration(_) => {
                    continue;
                }
                _ => {
                    return false;
                }
            }
        }
        false
    }

    #[inline]
    pub fn is_in_jsx(&self) -> bool {
        self.semantic.source_type().is_jsx()
    }

    #[inline]
    pub fn is_in_ts(&self) -> bool {
        self.semantic.source_type().is_typescript()
    }

    #[inline]
    pub fn get_snippet(&self, span: Span) -> &'a str {
        span.source_text(self.semantic.source_text())
    }
}

impl<'a> PartialEq<IdentifierReference<'a>> for Symbol<'_, 'a> {
    fn eq(&self, other: &IdentifierReference<'a>) -> bool {
        // cheap: no resolved reference means its a global reference
        let Some(reference_id) = other.reference_id.get() else {
            return false;
        };
        let reference = self.symbols().get_reference(reference_id);
        reference.symbol_id().is_some_and(|symbol_id| self.id == symbol_id)
    }
}

impl<'a> PartialEq<BindingIdentifier<'a>> for Symbol<'_, 'a> {
    fn eq(&self, id: &BindingIdentifier<'a>) -> bool {
        id.symbol_id.get().is_some_and(|id| self.id == id)
    }
}

impl<'a> PartialEq<BindingPattern<'a>> for Symbol<'_, 'a> {
    fn eq(&self, id: &BindingPattern<'a>) -> bool {
        id.get_binding_identifier().is_some_and(|id| self == id)
    }
}

impl<'a> PartialEq<AssignmentTarget<'a>> for Symbol<'_, 'a> {
    fn eq(&self, target: &AssignmentTarget<'a>) -> bool {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(id) => self == id.as_ref(),
            _ => false,
        }
    }
}

impl fmt::Debug for Symbol<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Symbol")
            .field("id", &self.id)
            .field("name", &self.name())
            .field("flags", &self.flags)
            .field("declaration_node", &self.declaration().kind().debug_name())
            .field("references", &self.references().collect::<Vec<_>>())
            .finish()
    }
}
