use std::{cell::OnceCell, fmt, iter};

use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, BindingIdentifier, BindingPattern, IdentifierReference,
        ImportDeclarationSpecifier, VariableDeclarator,
    },
};
use oxc_semantic::{
    AstNode, AstNodes, NodeId, Reference, ScopeId, Scoping, Semantic, SymbolFlags, SymbolId,
};
use oxc_span::{GetSpan, Span};

use crate::ModuleRecord;

#[derive(Clone)]
pub(super) struct Symbol<'s, 'a> {
    semantic: &'s Semantic<'a>,
    module_record: &'s ModuleRecord,
    id: SymbolId,
    flags: SymbolFlags,
    span: OnceCell<Span>,
}

impl PartialEq for Symbol<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// constructor and simple getters
impl<'s, 'a> Symbol<'s, 'a> {
    pub fn new(
        semantic: &'s Semantic<'a>,
        module_record: &'s ModuleRecord,
        symbol_id: SymbolId,
    ) -> Self {
        let flags = semantic.scoping().symbol_flags(symbol_id);
        Self { semantic, module_record, id: symbol_id, flags, span: OnceCell::new() }
    }

    #[inline]
    pub fn id(&self) -> SymbolId {
        self.id
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.scoping().symbol_name(self.id)
    }

    #[inline]
    pub fn flags(&self) -> SymbolFlags {
        self.flags
    }

    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scoping().symbol_scope_id(self.id)
    }

    #[inline]
    pub fn declaration(&self) -> &AstNode<'a> {
        self.nodes().get_node(self.declaration_id())
    }

    /// Returns `true` if this symbol has any references of any kind. Does not
    /// check if a references is "used" under the criteria of this rule.
    #[inline]
    pub fn has_references(&self) -> bool {
        !self.scoping().symbol_is_unused(self.id)
    }

    #[inline]
    pub fn references(&self) -> impl DoubleEndedIterator<Item = &Reference> + '_ + use<'_> {
        self.scoping().get_resolved_references(self.id)
    }

    /// Is this [`Symbol`] declared in the root scope?
    pub fn is_root(&self) -> bool {
        self.scoping().symbol_scope_id(self.id) == self.scoping().root_scope_id()
    }

    #[inline]
    fn declaration_id(&self) -> NodeId {
        self.scoping().symbol_declaration(self.id)
    }

    #[inline]
    pub fn nodes(&self) -> &AstNodes<'a> {
        self.semantic.nodes()
    }

    #[inline]
    pub fn scoping(&self) -> &Scoping {
        self.semantic.scoping()
    }

    #[inline]
    pub fn iter_parents(&self) -> impl Iterator<Item = &AstNode<'a>> + '_ {
        self.nodes().ancestors(self.declaration_id())
    }

    pub fn iter_self_and_parents(&self) -> impl Iterator<Item = &AstNode<'a>> + '_ {
        let node_id = self.declaration_id();
        let node = self.nodes().get_node(node_id);
        iter::once(node).chain(self.nodes().ancestors(node_id))
    }

    pub fn iter_relevant_parents_of(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = &AstNode<'a>> + Clone + '_ {
        self.nodes().ancestors(node_id).filter(|n| Self::is_relevant_kind(n.kind()))
    }

    pub fn iter_relevant_parent_and_grandparent_kinds(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = (/* parent */ AstKind<'a>, /* grandparent */ AstKind<'a>)> + Clone + '_
    {
        let parents_iter = iter::once(self.nodes().kind(node_id)).chain(
            self.nodes().ancestor_kinds(node_id).filter(|kind| Self::is_relevant_kind(*kind)),
        );

        let grandparents_iter = parents_iter.clone().skip(1);

        parents_iter.zip(grandparents_iter)
    }

    #[inline]
    const fn is_relevant_kind(kind: AstKind<'a>) -> bool {
        !matches!(
            kind,
            AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSInstantiationExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSTypeAssertion(_)
        )
    }

    /// <https://github.com/oxc-project/oxc/issues/4739>
    fn derive_span(&self) -> Span {
        for kind in self.iter_self_and_parents().map(AstNode::kind) {
            match kind {
                AstKind::BindingIdentifier(_) => {}
                AstKind::BindingRestElement(rest) => return rest.span,
                AstKind::VariableDeclarator(decl) => return self.clean_binding_id(&decl.id),
                AstKind::FormalParameter(param) => return self.clean_binding_id(&param.pattern),
                _ => break,
            }
        }
        self.scoping().symbol_span(self.id)
    }

    /// <https://github.com/oxc-project/oxc/issues/4739>
    fn clean_binding_id(&self, binding: &BindingPattern) -> Span {
        if binding.kind.is_destructuring_pattern() {
            return self.scoping().symbol_span(self.id);
        }
        let own = binding.kind.span();
        binding.type_annotation.as_ref().map_or(own, |ann| Span::new(own.start, ann.span.start))
    }
}

impl<'a> Symbol<'_, 'a> {
    /// Is this [`Symbol`] exported?
    ///
    /// NOTE: does not support CJS right now.
    pub fn is_exported(&self) -> bool {
        let is_in_exportable_scope = self.is_root() || self.is_in_ts_namespace();
        is_in_exportable_scope
            && (self.module_record.exported_bindings.contains_key(self.name())
                || self.in_export_node())
    }

    #[inline]
    fn is_in_ts_namespace(&self) -> bool {
        self.scoping().scope_flags(self.scope_id()).is_ts_module_block()
    }

    /// We need to do this due to limitations of [`Semantic`].
    fn in_export_node(&self) -> bool {
        for parent in self.nodes().ancestors(self.declaration_id()) {
            match parent.kind() {
                AstKind::ExportNamedDeclaration(_) | AstKind::ExportDefaultDeclaration(_) => {
                    return true;
                }
                AstKind::VariableDeclaration(_)
                | AstKind::ArrayExpression(_)
                | AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_) => {}
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

impl GetSpan for Symbol<'_, '_> {
    /// [`Span`] for the node declaring the [`Symbol`].
    #[inline]
    fn span(&self) -> Span {
        // TODO: un-comment and replace when BindingIdentifier spans are fixed
        // https://github.com/oxc-project/oxc/issues/4739

        // self.symbols().get_span(self.id)
        *self.span.get_or_init(|| self.derive_span())
    }
}

impl<'a> PartialEq<IdentifierReference<'a>> for Symbol<'_, 'a> {
    fn eq(&self, other: &IdentifierReference<'a>) -> bool {
        // cheap: no resolved reference means its a global reference
        let reference = self.scoping().get_reference(other.reference_id());
        reference.symbol_id().is_some_and(|symbol_id| self.id == symbol_id)
    }
}

impl<'a> PartialEq<BindingIdentifier<'a>> for Symbol<'_, 'a> {
    fn eq(&self, id: &BindingIdentifier<'a>) -> bool {
        self.id == id.symbol_id()
    }
}

impl<'a> PartialEq<VariableDeclarator<'a>> for Symbol<'_, 'a> {
    fn eq(&self, decl: &VariableDeclarator<'a>) -> bool {
        self == &decl.id
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

impl<'a> PartialEq<ImportDeclarationSpecifier<'a>> for Symbol<'_, 'a> {
    fn eq(&self, import: &ImportDeclarationSpecifier<'a>) -> bool {
        self == import.local()
    }
}

impl<'s, 'a, T> PartialEq<&T> for Symbol<'s, 'a>
where
    Symbol<'s, 'a>: PartialEq<T>,
{
    fn eq(&self, other: &&T) -> bool {
        self == *other
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
